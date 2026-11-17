// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Bus recording sessions, serviced entirely without the interactive recorder.
use comp_model::{
    capture::{CaptureFrameSpec, RecordCommand},
    reply::ControlReply,
};
use recorder::bus::Recording;
use serde_json::{Value, json};
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    rc::Rc,
    time::{Duration, Instant},
};
use world::state::Loop;

type Answer = Box<dyn FnOnce(ControlReply)>;
#[derive(Clone)]
struct Past {
    id: String,
    params: RecordCommand,
    start: Option<Value>,
    terminal: Value,
    status: Value,
    start_error: Option<String>,
}
#[derive(Default)]
struct History {
    entries: HashMap<String, Past>,
    order: VecDeque<String>,
}
impl History {
    fn get(&mut self, key: &str) -> Option<Past> {
        let past = self.entries.get(key)?.clone();
        self.order.retain(|entry| entry != key);
        self.order.push_back(key.into());
        Some(past)
    }
    fn session(&mut self, id: &str) -> Option<Past> {
        let key = self
            .entries
            .iter()
            .find(|(_, past)| past.id == id)?
            .0
            .clone();
        self.get(&key)
    }
    fn insert(&mut self, key: String, past: Past) {
        self.order.retain(|entry| *entry != key);
        self.order.push_back(key.clone());
        self.entries.insert(key, past);
        while self.order.len() > 128 {
            if let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
            }
        }
    }
}
struct Session {
    frames_parked: fn(&Loop) -> bool,
    request_id: String,
    admitted: Instant,
    id: String,
    pixel_secret: String,
    pixel_counter: u64,
    params: RecordCommand,
    path: String,
    targets: Vec<CaptureFrameSpec>,
    fps: u32,
    encoder: String,
    probe: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
    probe_cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    probe_join: Option<std::thread::JoinHandle<()>>,
    max_seconds: u64,
    max_bytes: Option<u64>,
    recording: Option<Recording>,
    start: Option<Value>,
    start_answers: Vec<Answer>,
    stop_answers: Vec<Answer>,
    pending: bool,
    drain_until: Option<Instant>,
    signature: Vec<u64>,
    deferred_signature: Option<Vec<u64>>,
    failed: Option<String>,
    last_state: Cell<&'static str>,
}
impl Drop for Session {
    fn drop(&mut self) {
        self.probe_cancel
            .store(true, std::sync::atomic::Ordering::Release);
        if let Some(join) = self.probe_join.take() {
            // A cancelled preflight may still be waiting on its filesystem.
            // Its owned inputs and cleanup stay on the worker; never join a
            // disk operation from the compositor thread.
            drop(join);
        }
    }
}
thread_local! {
    static PAST: RefCell<History> = RefCell::new(History::default());
    static CURRENT: RefCell<Option<Rc<RefCell<Session>>>> = const { RefCell::new(None) };
    static EVENTS: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
}
pub fn take_events() -> Vec<Value> {
    EVENTS.with_borrow_mut(std::mem::take)
}
fn refused(error: &str) -> ControlReply {
    ControlReply::refused("record_failed", json!({"message":error}))
}
fn empty_status(state: &str, error: Option<&str>) -> Value {
    json!({"session":null,"state":state,"frames":0,"repeated":0,"dropped":0,"dropped_capture":0,"dropped_queue":0,"readback_deferred":0,"pending_ticks":0,"writer":null,"encoder":null,"duration_us":0,"bytes":0,"queue":0,"encoders":["vfl"],"readbacks":0,"readback_us":0,"readback_mean_us":0,"readback_max_us":0,"error":error})
}
fn event(s: &Session, state: &'static str) {
    if s.last_state.replace(state) == state {
        return;
    }
    EVENTS.with_borrow_mut(|events| {
        events.push(json!({"session":s.id,"state":state,"error":s.failed}))
    });
}

pub fn command(
    lp: &mut Loop,
    op: RecordCommand,
    frames_parked: fn(&Loop) -> bool,
    answer: impl FnOnce(ControlReply) + 'static,
) {
    let answer: Answer = Box::new(answer);
    let params = op.clone();
    if let RecordCommand::Start { request_id, .. } = &op {
        if let Some(past) = PAST.with_borrow_mut(|past| past.get(request_id)) {
            if past.params != op {
                answer(refused("request_id parameters differ"));
                return;
            }
            answer(past.start.map(ControlReply::Body).unwrap_or_else(|| {
                refused(
                    past.start_error
                        .as_deref()
                        .unwrap_or("previous start failed"),
                )
            }));
            return;
        }
    } else {
        let id = match &op {
            RecordCommand::Marker { session, .. } | RecordCommand::Stop { session, .. } => {
                Some(session)
            }
            RecordCommand::Status { session } => session.as_ref(),
            _ => None,
        };
        if let Some(past) = id.and_then(|id| PAST.with_borrow_mut(|past| past.session(id))) {
            answer(match op {
                RecordCommand::Stop { .. } => ControlReply::Body(past.terminal),
                RecordCommand::Status { .. } => ControlReply::Body(past.status),
                _ => refused("session is no longer recording"),
            });
            return;
        }
    }
    if let RecordCommand::Start {
        request_id,
        mut targets,
        path,
        fps,
        encoder,
        max_seconds,
        max_bytes,
    } = op
    {
        let previous = CURRENT.with_borrow(Clone::clone);
        if let Some(previous) = previous {
            let mut s = previous.borrow_mut();
            if s.request_id == request_id {
                if s.params != params {
                    answer(refused("request_id parameters differ"));
                    return;
                }
                if let Some(reply) = &s.start {
                    answer(ControlReply::Body(reply.clone()));
                } else if let Some(error) = &s.failed {
                    answer(refused(error));
                } else {
                    s.start_answers.push(answer);
                }
                return;
            }
            if s.recording.as_ref().is_some_and(|r| r.terminal.is_none())
                || s.recording.is_none() && s.failed.is_none()
            {
                answer(ControlReply::refused(
                    "record_busy",
                    json!({"session":s.id}),
                ));
                return;
            }
        }
        for target in &mut targets {
            if target.window.is_none() {
                let outputs: Vec<_> = lp.inner.space_state().state.outputs().cloned().collect();
                let output = if let Some(name) = &target.output {
                    outputs.iter().find(|o| {
                        o.name() == *name || comp_model::snapshot::output_key(&o.name()) == *name
                    })
                } else {
                    outputs
                        .iter()
                        .find(|o| {
                            lp.inner
                                .cursor_output
                                .as_ref()
                                .is_some_and(|key| *key == world::state::output_key(o))
                        })
                        .or(outputs.first())
                };
                let Some(output) = output else {
                    answer(refused("unknown output"));
                    return;
                };
                let generation = lp.inner.comp.output_generation(&output.name());
                if target
                    .output_generation
                    .is_some_and(|expected| expected != generation)
                {
                    answer(refused("output generation changed"));
                    return;
                }
                target.output = Some(output.name());
                target.output_generation = Some(generation);
            }
        }
        let id = match recorder::bus::private_token() {
            Ok(token) => format!("record-{token}"),
            Err(error) => {
                answer(refused(&error));
                return;
            }
        };
        let probe_cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let pixel_secret = match recorder::bus::private_token() {
            Ok(secret) => secret,
            Err(error) => {
                answer(refused(&error));
                return;
            }
        };
        let probe_join;
        let probe = {
            let (tx, rx) = std::sync::mpsc::channel();
            let destination = path.clone();
            probe_join = Some(std::thread::spawn(move || {
                let _ = tx.send(recorder::bus::check_destination(&destination));
            }));
            Some(rx)
        };
        let session = Rc::new(RefCell::new(Session {
            frames_parked,
            request_id,
            admitted: Instant::now(),
            params,
            id,
            pixel_secret,
            pixel_counter: 0,
            path,
            targets,
            fps,
            encoder,
            probe,
            probe_cancel,
            probe_join,
            max_seconds,
            max_bytes,
            recording: None,
            start: None,
            start_answers: vec![answer],
            stop_answers: Vec::new(),
            pending: false,
            drain_until: None,
            signature: Vec::new(),
            deferred_signature: None,
            failed: None,
            last_state: Cell::new(""),
        }));
        CURRENT.with_borrow_mut(|current| *current = Some(session.clone()));
        let tick_session = session.clone();
        let period = Duration::from_nanos(1_000_000_000 / u64::from(fps));
        let mut next = Instant::now();
        let mut anchored = false;
        if lp
            .loop_handle
            .insert_source(Timer::immediate(), move |_, _, lp| {
                tick(lp, &tick_session);
                let s = tick_session.borrow();
                if s.failed.is_some() && s.recording.is_none()
                    || s.recording.as_ref().is_some_and(|r| r.terminal.is_some())
                {
                    TimeoutAction::Drop
                } else {
                    if !anchored && s.start.is_some() {
                        next = Instant::now();
                        anchored = true;
                    }
                    next += period;
                    TimeoutAction::ToInstant(next)
                }
            })
            .is_err()
        {
            fail(&session, "record timer unavailable");
        }
        return;
    }
    session_command(op, answer);
}

fn session_command(op: RecordCommand, answer: Answer) {
    let current = CURRENT.with_borrow(Clone::clone);
    let Some(current) = current else {
        if matches!(op, RecordCommand::Status { session: None }) {
            answer(ControlReply::Body(empty_status("idle", None)));
        } else {
            answer(refused("unknown session"));
        }
        return;
    };
    let mut s = current.borrow_mut();
    let wanted = match &op {
        RecordCommand::Marker { session, .. } | RecordCommand::Stop { session, .. } => {
            Some(session)
        }
        RecordCommand::Status { session } => session.as_ref(),
        _ => unreachable!(),
    };
    if wanted.is_some_and(|id| *id != s.id) {
        answer(refused("unknown session"));
        return;
    }
    match op {
        RecordCommand::Status { .. } => {
            let mut status = s
                .recording
                .as_ref()
                .map(Recording::status)
                .unwrap_or_else(|| {
                    empty_status(
                        if s.failed.is_some() {
                            "failed"
                        } else {
                            "recording"
                        },
                        s.failed.as_deref(),
                    )
                });
            status["session"] = json!(s.id);
            answer(ControlReply::Body(status));
        }
        RecordCommand::Marker { label, data, .. } => answer(
            match s
                .recording
                .as_mut()
                .ok_or("first frame pending")
                .and_then(|r| r.marker(label, data).map_err(|_| "marker write failed"))
            {
                Ok(value) => ControlReply::Body(value),
                Err(error) => refused(error),
            },
        ),
        RecordCommand::Stop { session, discard } => {
            if session != s.id {
                answer(refused("unknown session"));
                return;
            }
            if let Some(r) = s.recording.as_mut() {
                if let Some(reply) = &r.terminal {
                    answer(ControlReply::Body(reply.clone()));
                    return;
                }
                if r.stop_conflicts(discard) || s.drain_until.is_some() && discard {
                    answer(refused("already finishing"));
                    return;
                }
                let changed = stop_stream(&mut s, discard);
                s.stop_answers.push(answer);
                if changed {
                    event(&s, "finishing");
                }
            } else {
                drop(s);
                cancel_pending(&current, "cancelled before first frame");
                let past = PAST.with_borrow_mut(|p| p.session(&session));
                answer(match past {
                    Some(past) => ControlReply::Body(past.terminal),
                    None => refused("unknown session"),
                });
            }
        }
        _ => unreachable!(),
    }
}

fn fail(session: &Rc<RefCell<Session>>, error: &str) {
    let mut s = session.borrow_mut();
    if s.failed.is_some() {
        return;
    }
    s.failed = Some(error.into());
    s.probe_cancel
        .store(true, std::sync::atomic::Ordering::Release);
    for answer in std::mem::take(&mut s.start_answers) {
        answer(refused(error));
    }
    let lost = u64::from(std::mem::take(&mut s.pending))
        + u64::from(s.deferred_signature.take().is_some());
    if let Some(r) = s.recording.as_mut() {
        r.error = Some(error.into());
        for _ in 0..lost {
            r.superseded_capture();
        }
        r.stop(false);
    } else {
        let mut status = empty_status("failed", Some(error));
        status["session"] = json!(s.id);
        let past = Past {
            id: s.id.clone(),
            params: s.params.clone(),
            start: None,
            start_error: Some(error.into()),
            terminal: json!({"path":s.path,"frames":0,"repeated":0,"dropped":0,"duration_us":0,"frames_path":null,"markers_path":null,"encoder":s.encoder,"error":error}),
            status,
        };
        PAST.with_borrow_mut(|all| {
            all.insert(s.request_id.clone(), past);
        });
    }
    event(&s, "failed");
}

fn cancel_pending(session: &Rc<RefCell<Session>>, message: &str) {
    fail(session, message);
}

// A false result gates all capture scheduling, including the very first readback.
fn preflight_ready(session: &Rc<RefCell<Session>>) -> bool {
    let result = session
        .borrow()
        .probe
        .as_ref()
        .map(|probe| probe.try_recv());
    let error = match result {
        None => return true,
        Some(Ok(Ok(()))) => {
            session.borrow_mut().probe = None;
            return true;
        }
        Some(Ok(Err(error))) => error,
        Some(Err(std::sync::mpsc::TryRecvError::Empty)) => return false,
        Some(Err(std::sync::mpsc::TryRecvError::Disconnected)) => {
            "encoder preflight disconnected".into()
        }
    };
    fail(session, &error);
    // A refused preflight has not admitted a take; allow a corrected retry.
    let key = session.borrow().request_id.clone();
    PAST.with_borrow_mut(|past| {
        past.entries.remove(&key);
        past.order.retain(|entry| entry != &key);
    });
    CURRENT.with_borrow_mut(|current| {
        if current.as_ref().is_some_and(|s| Rc::ptr_eq(s, session)) {
            *current = None;
        }
    });
    false
}

fn superseded(pending: &mut Option<Vec<u64>>, current: &[u64]) -> bool {
    if pending.as_ref().is_some_and(|old| old != current) {
        *pending = None;
        true
    } else {
        false
    }
}

fn stop_stream(s: &mut Session, discard: bool) -> bool {
    // A submitted GPU snapshot already belongs to the take. Let its normal
    // fence callback deliver it before sealing, without blocking the Bus or
    // compositor. A failed/stalled backend still has a bounded escape path.
    if s.pending && !discard && s.failed.is_none() {
        let until = *s.drain_until.get_or_insert_with(|| Instant::now() + Duration::from_secs(1));
        if Instant::now() < until {
            return true;
        }
    }
    s.drain_until = None;
    let lost = u64::from(std::mem::take(&mut s.pending))
        + u64::from(s.deferred_signature.take().is_some());
    if let Some(r) = s.recording.as_mut() {
        for _ in 0..lost {
            r.superseded_capture();
        }
        r.stop(discard)
    } else {
        false
    }
}

fn tick(lp: &mut Loop, session: &Rc<RefCell<Session>>) {
    let clock = recorder::bus::monotonic_us();
    {
        let s = session.borrow();
        if s.recording.is_none()
            && s.failed.is_none()
            && s.admitted.elapsed() >= Duration::from_secs(s.max_seconds)
        {
            drop(s);
            cancel_pending(session, "max_seconds elapsed before first frame");
            return;
        }
    }
    if !preflight_ready(session) {
        return;
    }
    {
        let mut s = session.borrow_mut();
        if let Some(r) = s.recording.as_mut() {
            if r.poll() {
                let reply = r.terminal.clone().unwrap();
                let error = r.error.clone();
                if error.is_some() {
                    s.failed = error;
                }
                let past = Past {
                    id: s.id.clone(),
                    params: s.params.clone(),
                    start: s.start.clone(),
                    terminal: reply.clone(),
                    status: s.recording.as_ref().unwrap().status(),
                    start_error: None,
                };
                PAST.with_borrow_mut(|all| {
                    all.insert(s.request_id.clone(), past);
                });
                for answer in std::mem::take(&mut s.stop_answers) {
                    answer(ControlReply::Body(reply.clone()));
                }
                event(&s, if s.failed.is_some() { "failed" } else { "idle" });
                return;
            }
            if r.terminal.is_some() || !r.is_recording() || s.failed.is_some() {
                return;
            }
        }
        if s.failed.is_some() {
            return;
        }
        if let Some(until) = s.drain_until {
            if !s.pending || Instant::now() >= until {
                stop_stream(&mut s, false);
                event(&s, "finishing");
            }
            return;
        }
        let now_us = match clock {
            Ok(now) => now,
            Err(error) => {
                drop(s);
                fail(session, &error);
                return;
            }
        };
        if s.recording
            .as_ref()
            .is_some_and(|r| now_us.saturating_sub(r.t0_us) >= s.max_seconds * 1_000_000)
        {
            stop_stream(&mut s, false);
            event(&s, "finishing");
            return;
        }
    }
    let signature = {
        let s = session.borrow();
        let mut signature = Vec::new();
        for target in &s.targets {
            if let Some(window) = target.window {
                if lp
                    .inner
                    .comp
                    .registry
                    .resolve_window_target(window.id, Some(window.generation))
                    .is_err()
                {
                    drop(s);
                    fail(session, "window target gone or generation changed");
                    return;
                }
                let comp = &lp.inner.comp;
                let rows: Vec<_> = comp.registry.surface_rows().collect();
                for row in &rows {
                    let mut ancestor = Some(row.id());
                    for _ in 0..=rows.len() {
                        let Some(id) = ancestor else {
                            break;
                        };
                        if id.0 == window.id {
                            signature.extend([row.id().0, comp.commits(row.id())]);
                            break;
                        }
                        ancestor = comp.registry.get(id).and_then(|r| r.parent());
                    }
                }
                if let Ok(w) = world::window::draw::frame::scene::capture_window(
                    lp,
                    window.id,
                    window.generation,
                ) {
                    signature.extend([w.geometry().size.w as u64, w.geometry().size.h as u64]);
                }
            } else {
                let outputs: Vec<_> = lp.inner.space_state().state.outputs().cloned().collect();
                let output = outputs
                    .iter()
                    .find(|o| target.output.as_ref().is_none_or(|name| *name == o.name()));
                let Some(output) = output else {
                    drop(s);
                    fail(session, "output target gone");
                    return;
                };
                if target.output_generation != Some(lp.inner.comp.output_generation(&output.name()))
                {
                    drop(s);
                    fail(session, "output generation changed");
                    return;
                }
                let parked = matches!(lp.inner.status_session, world::state::StatusSession::Paused)
                    || (s.frames_parked)(lp);
                signature.extend(
                    lp.state
                        .redraw
                        .capture_stamp(&world::state::output_key(output), parked),
                );
            }
        }
        signature
    };
    let mut s = session.borrow_mut();
    let unchanged = s.signature == signature;
    if superseded(&mut s.deferred_signature, &signature) {
        if let Some(r) = s.recording.as_mut() {
            r.superseded_capture();
        }
        s.deferred_signature = None;
    }
    if s.pending {
        if !unchanged {
            s.deferred_signature = Some(signature);
        }
        let result = s.recording.as_mut().map(|r| {
            r.pending_tick();
            r.frame(None)
        });
        drop(s);
        if let Some(Err(error)) = result {
            fail(session, &error);
        }
        return;
    }
    let skipped = s
        .recording
        .as_mut()
        .is_some_and(|r| r.skip_readback(!unchanged));
    if unchanged || skipped {
        if skipped {
            s.deferred_signature = Some(signature);
        }
        let result = s.recording.as_mut().unwrap().frame(None);
        drop(s);
        if let Err(error) = result {
            fail(session, &error);
        }
        return;
    }
    s.signature = signature;
    s.deferred_signature = None;
    s.pending = true;
    let targets = s.targets.clone();
    let counter = s.pixel_counter;
    let Some(next_counter) = counter.checked_add(1) else {
        drop(s);
        fail(session, "raw capture counter exhausted");
        return;
    };
    s.pixel_counter = next_counter;
    let prefix = format!("/{}-{counter}", s.pixel_secret);
    drop(s);
    let frames = Rc::new(RefCell::new(Vec::new()));
    for (index, mut target) in targets.into_iter().enumerate() {
        let token = format!("{prefix}-{index}");
        target.path = token.clone();
        let got = frames.clone();
        let session = session.clone();
        screencopy::file::raw_answer(token.clone(), move |result| {
            {
                let s = session.borrow();
                if s.failed.is_some() || s.recording.as_ref().is_some_and(|r| !r.is_recording()) {
                    return;
                }
            }
            let captured_us = match recorder::bus::monotonic_us() {
                Ok(time) => time,
                Err(error) => {
                    fail(&session, &error);
                    return;
                }
            };
            let frame = match result {
                Ok(frame) => frame,
                Err(error) => {
                    fail(&session, &format!("capture: {:?}", error.into_wire()));
                    return;
                }
            };
            got.borrow_mut().push((index, frame));
            let count = session.borrow().targets.len();
            if got.borrow().len() != count {
                return;
            }
            let mut frames = got.borrow_mut();
            frames.sort_by_key(|(index, _)| *index);
            let width = frames.iter().map(|(_, f)| f.0).sum::<u32>();
            let height = frames.iter().map(|(_, f)| f.1).max().unwrap();
            let readback_us = frames.iter().map(|(_, frame)| frame.3).sum::<u64>();
            let pixels = if frames.len() == 1 {
                frames[0].1.2.clone()
            } else {
                let mut pixels = vec![0; width as usize * height as usize * 3];
                let mut x = 0;
                for (_, (w, h, p, _)) in frames.iter() {
                    for y in 0..*h as usize {
                        let at = (y * width as usize + x) * 3;
                        pixels[at..at + *w as usize * 3]
                            .copy_from_slice(&p[y * *w as usize * 3..(y + 1) * *w as usize * 3]);
                    }
                    x += *w as usize;
                }
                std::sync::Arc::new(pixels)
            };
            let mut s = session.borrow_mut();
            s.pending = false;
            if s.deferred_signature.as_ref() == Some(&s.signature) {
                s.deferred_signature = None;
            }
            if s.failed.is_some() || s.recording.as_ref().is_some_and(|r| !r.is_recording()) {
                return;
            }
            if s.recording.is_none() {
                match Recording::new_bounded(
                    s.id.clone(),
                    s.path.clone(),
                    width,
                    height,
                    s.fps,
                    s.encoder.clone(),
                    s.max_bytes,
                ) {
                    Ok(mut recording) => {
                        recording.t0_us = captured_us;
                        s.recording = Some(recording);
                    }
                    Err(error) => {
                        drop(s);
                        fail(&session, &error);
                        return;
                    }
                }
            }
            if let Err(error) =
                s.recording
                    .as_mut()
                    .unwrap()
                    .frame(Some((width, height, pixels, readback_us)))
            {
                drop(s);
                fail(&session, &error);
                return;
            }
            if s.start.is_none() {
                let reply = s.recording.as_ref().unwrap().started();
                s.start = Some(reply.clone());
                for answer in std::mem::take(&mut s.start_answers) {
                    answer(ControlReply::Body(reply.clone()));
                }
                event(&s, "recording");
            }
            if s.drain_until.is_some() {
                stop_stream(&mut s, false);
                event(&s, "finishing");
            }
        });
        crate::capture::start(lp, target, Instant::now(), move |reply| {
            if !matches!(reply, ControlReply::Body(_)) {
                screencopy::file::raw_fail(&token, reply);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_session_status_has_uniform_keys() {
        CURRENT.with_borrow_mut(|current| *current = None);
        let status = Rc::new(RefCell::new(Value::Null));
        let got = status.clone();
        session_command(
            RecordCommand::Status { session: None },
            Box::new(move |reply| {
                let ControlReply::Body(body) = reply else {
                    panic!("status refused")
                };
                *got.borrow_mut() = body;
            }),
        );
        let status = status.borrow();
        for key in [
            "dropped_capture",
            "dropped_queue",
            "readback_deferred",
            "pending_ticks",
        ] {
            assert_eq!(status[key], 0, "{key}");
        }
        for key in ["writer", "encoder"] {
            assert!(status.as_object().unwrap().contains_key(key));
            assert_eq!(status[key], Value::Null);
        }
    }
    #[test]
    fn stable_deferred_content_is_not_dropped_every_tick() {
        let mut waiting = Some(vec![7]);
        for _ in 0..602 {
            assert!(!superseded(&mut waiting, &[7]));
        }
        assert!(superseded(&mut waiting, &[8]));
        assert!(!superseded(&mut waiting, &[8]));
        waiting = Some(vec![8]);
        assert!(!superseded(&mut waiting, &[8]));
        assert!(superseded(&mut waiting, &[9]));
    }
    fn stop_reply(id: &str) -> bool {
        let answered = Rc::new(Cell::new(None));
        let got = answered.clone();
        session_command(
            RecordCommand::Stop {
                session: id.into(),
                discard: false,
            },
            Box::new(move |reply| got.set(Some(matches!(reply, ControlReply::Body(_))))),
        );
        answered.get().expect("stop must answer immediately")
    }
    #[test]
    fn pending_cancel_replies_and_leaves_replayable_terminal_without_sidecars() {
        CURRENT.with_borrow_mut(|current| *current = None);
        PAST.with_borrow_mut(|past| *past = History::default());
        assert!(!stop_reply("unknown"));
        let params = comp_model::capture::parse_record("comp.record.start", &json!({"request_id":"pending", "path":"/tmp/pending-record-unit.vfl", "target":{}, "max_seconds":1})).unwrap();
        let answered = Rc::new(Cell::new(false));
        let got = answered.clone();
        let session = Rc::new(RefCell::new(Session {
            frames_parked: |_| false,
            request_id: "pending".into(),
            admitted: Instant::now() - Duration::from_secs(2),
            id: "pending-session".into(),
            pixel_secret: "private-unit-secret".into(),
            pixel_counter: 0,
            params,
            path: "/tmp/pending-record-unit.vfl".into(),
            targets: vec![],
            fps: 30,
            encoder: "vfl".into(),
            probe: None,
            max_seconds: 1,
            max_bytes: None,
            probe_cancel: Default::default(),
            probe_join: None,
            recording: None,
            start: None,
            start_answers: vec![Box::new(move |reply| {
                assert!(!matches!(reply, ControlReply::Body(_)));
                got.set(true);
            })],
            stop_answers: vec![],
            pending: true,
            drain_until: None,
            signature: vec![],
            deferred_signature: None,
            failed: None,
            last_state: Cell::new(""),
        }));
        assert!(
            session.borrow().admitted.elapsed() > Duration::from_secs(session.borrow().max_seconds)
        );
        CURRENT.with_borrow_mut(|current| *current = Some(session.clone()));
        for id in ["unknown", "stale-session", "mismatched-session"] {
            assert!(!stop_reply(id));
            assert!(session.borrow().failed.is_none());
            assert!(
                !answered.get(),
                "a wrong ID must not cancel the pending start"
            );
        }
        assert!(stop_reply("pending-session"));
        assert!(answered.get());
        let past = PAST.with_borrow_mut(|p| p.get("pending")).unwrap();
        assert_eq!(past.terminal["frames"], 0);
        assert_eq!(past.terminal["frames_path"], Value::Null);
        assert_eq!(past.status["state"], "failed");
        cancel_pending(&session, "duplicate cancellation");
        assert!(session.borrow().start_answers.is_empty());
        for i in 0..256 {
            {
                let mut s = session.borrow_mut();
                s.request_id = format!("pending-{i}");
                s.id = format!("session-{i}");
                s.failed = None;
            }
            cancel_pending(&session, "max_seconds elapsed before first frame");
        }
        assert_eq!(PAST.with_borrow(|p| p.entries.len()), 128);
        assert!(
            !stop_reply("pending-session"),
            "evicted session must be refused"
        );
        assert_eq!(session.borrow().id, "session-255");
        // Inject the asynchronous publication probe failure at the same gate
        // used by tick, before it can schedule any capture.
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(Err(
            "destination does not support no-replace publication: EINVAL".into(),
        ))
        .unwrap();
        answered.set(false);
        {
            let mut s = session.borrow_mut();
            s.failed = None;
            s.pending = false;
            s.probe = Some(rx);
            let got = answered.clone();
            s.start_answers.push(Box::new(move |reply| {
                assert!(!matches!(reply, ControlReply::Body(_)));
                got.set(true);
            }));
        }
        assert!(!preflight_ready(&session));
        assert!(answered.get());
        assert!(!session.borrow().pending);
        assert!(session.borrow().recording.is_none());
        assert!(
            session
                .borrow()
                .failed
                .as_ref()
                .unwrap()
                .contains("no-replace publication")
        );
        assert!(CURRENT.with_borrow(|current| current.is_none()));
        let mut history = History::default();
        for i in 0..128 {
            history.insert(format!("key-{i}"), past.clone());
        }
        assert!(history.get("key-0").is_some());
        history.insert("new".into(), past);
        assert!(
            history.get("key-0").is_some(),
            "recently used entry must survive"
        );
        assert!(
            history.get("key-1").is_none(),
            "least recently used entry must be evicted"
        );
        CURRENT.with_borrow_mut(|current| *current = None);
    }
}
