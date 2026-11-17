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
    stop_wake: Option<Rc<dyn Fn(Instant) -> bool>>,
    stop_timer_at: Option<Instant>,
    finalise_delay: Duration,
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
            stop_wake: None,
            stop_timer_at: None,
            finalise_delay: Duration::from_millis(5),
            signature: Vec::new(),
            deferred_signature: None,
            failed: None,
            last_state: Cell::new(""),
        }));
        install_stop_waker(&session, lp.loop_handle.clone());
        CURRENT.with_borrow_mut(|current| *current = Some(session.clone()));
        let tick_session = session.clone();
        let period = Duration::from_nanos(1_000_000_000 / u64::from(fps));
        let mut next = Instant::now();
        let mut anchored = false;
        if lp
            .loop_handle
            .insert_source(Timer::immediate(), move |_, _, lp| {
                tick(&tick_session, || capture_tick(lp, &tick_session));
                let s = tick_session.borrow();
                if s.failed.is_some() && s.recording.is_none()
                    || s.recording.as_ref().is_some_and(|r| !r.is_recording())
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
    screencopy::file::release_raw(&format!("/{}-", s.pixel_secret));
    s.probe_cancel
        .store(true, std::sync::atomic::Ordering::Release);
    for answer in std::mem::take(&mut s.start_answers) {
        answer(refused(error));
    }
    let lost = u64::from(std::mem::take(&mut s.pending))
        + u64::from(s.deferred_signature.take().is_some());
    s.drain_until = None;
    if let Some(r) = s.recording.as_mut() {
        r.error = Some(error.into());
        for _ in 0..lost {
            r.superseded_capture();
        }
        r.stop(false);
        schedule_stop(&mut s, Instant::now() + Duration::from_millis(5));
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
        let until = *s
            .drain_until
            .get_or_insert_with(|| Instant::now() + Duration::from_secs(1));
        if Instant::now() < until && schedule_stop(s, until) {
            return false;
        }
    }
    s.drain_until = None;
    screencopy::file::release_raw(&format!("/{}-", s.pixel_secret));
    let lost = u64::from(std::mem::take(&mut s.pending))
        + u64::from(s.deferred_signature.take().is_some());
    if let Some(r) = s.recording.as_mut() {
        for _ in 0..lost {
            r.superseded_capture();
        }
        let sealed = r.stop(discard);
        if sealed {
            schedule_stop(s, Instant::now() + Duration::from_millis(5));
        }
        sealed
    } else {
        false
    }
}

fn schedule_stop(s: &mut Session, at: Instant) -> bool {
    if s.stop_timer_at.is_some_and(|current| current <= at) {
        return true;
    }
    if s.stop_wake.as_ref().is_some_and(|wake| wake(at)) {
        s.stop_timer_at = Some(at);
        true
    } else {
        false
    }
}

// The timer captures this Session weakly, never CURRENT. A late old-take
// timeout cannot seal a newer recording, even after CURRENT has changed.
fn install_stop_waker<D: 'static>(
    session: &Rc<RefCell<Session>>,
    handle: smithay::reexports::calloop::LoopHandle<'static, D>,
) {
    let owner = Rc::downgrade(session);
    let token = Rc::new(RefCell::new(None));
    session.borrow_mut().stop_wake = Some(Rc::new(move |at| {
        if let Some(old) = token.borrow_mut().take() {
            handle.remove(old);
        }
        let owner = owner.clone();
        let slot = token.clone();
        match handle.insert_source(Timer::from_deadline(at), move |_, _, _| {
            slot.borrow_mut().take();
            if let Some(session) = owner.upgrade() {
                let mut s = session.borrow_mut();
                s.stop_timer_at = None;
                if s.drain_until.is_some() && stop_stream(&mut s, false) {
                    event(&s, "finishing");
                }
                if !finish_recording(&mut s)
                    && s.recording.as_ref().is_some_and(|r| !r.is_recording())
                {
                    let delay = s.finalise_delay;
                    s.finalise_delay = (delay * 2).min(Duration::from_secs(1));
                    schedule_stop(&mut s, Instant::now() + delay);
                }
            }
            TimeoutAction::Drop
        }) {
            Ok(next) => {
                *token.borrow_mut() = Some(next);
                true
            }
            Err(_) => false,
        }
    }));
}

fn finish_recording(s: &mut Session) -> bool {
    let Some(r) = s.recording.as_mut() else {
        return false;
    };
    if r.terminal.is_some() {
        return true;
    }
    if !r.poll() {
        return false;
    }
    let reply = r.terminal.clone().unwrap();
    if r.error.is_some() {
        s.failed = r.error.clone();
    }
    let past = Past {
        id: s.id.clone(),
        params: s.params.clone(),
        start: s.start.clone(),
        terminal: reply.clone(),
        status: r.status(),
        start_error: None,
    };
    PAST.with_borrow_mut(|all| all.insert(s.request_id.clone(), past));
    for answer in std::mem::take(&mut s.stop_answers) {
        answer(ControlReply::Body(reply.clone()));
    }
    event(s, if s.failed.is_some() { "failed" } else { "idle" });
    true
}

fn tick(session: &Rc<RefCell<Session>>, capture: impl FnOnce()) {
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
    capture();
}

fn capture_tick(lp: &mut Loop, session: &Rc<RefCell<Session>>) {
    for output in lp.inner.space_state().state.outputs() {
        screencopy::file::set_paused(
            output,
            matches!(lp.inner.status_session, world::state::StatusSession::Paused),
        );
    }
    let clock = recorder::bus::monotonic_us();
    {
        let mut s = session.borrow_mut();
        if finish_recording(&mut s) {
            return;
        }
        if s.recording
            .as_ref()
            .is_some_and(|r| r.terminal.is_some() || !r.is_recording())
            || s.failed.is_some()
        {
            return;
        }
        if let Some(until) = s.drain_until {
            if (!s.pending || Instant::now() >= until) && stop_stream(&mut s, false) {
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
            if stop_stream(&mut s, false) {
                event(&s, "finishing");
            }
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
                    Ok(recording) => {
                        assign_first_capture(&mut s, recording, captured_us);
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
            if s.drain_until.is_some() && stop_stream(&mut s, false) {
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

fn assign_first_capture(s: &mut Session, mut recording: Recording, captured_us: u64) {
    recording.t0_us = captured_us;
    s.recording = Some(recording);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_fixture(id: &str, root: &std::path::Path) -> Rc<RefCell<Session>> {
        let path = root
            .join(format!("{id}.vfl"))
            .to_string_lossy()
            .into_owned();
        let params = comp_model::capture::parse_record(
            "comp.record.start",
            &json!({"request_id":id,"path":path,"target":{},"max_seconds":60}),
        )
        .unwrap();
        let recording =
            Recording::new_bounded(id.into(), path.clone(), 2, 2, 30, "vfl".into(), None).unwrap();
        Rc::new(RefCell::new(Session {
            frames_parked: |_| false,
            request_id: id.into(),
            admitted: Instant::now(),
            id: id.into(),
            pixel_secret: format!("test-{id}"),
            pixel_counter: 0,
            params,
            path,
            targets: vec![],
            fps: 30,
            encoder: "vfl".into(),
            probe: None,
            probe_cancel: Default::default(),
            probe_join: None,
            max_seconds: 60,
            max_bytes: None,
            start: Some(recording.started()),
            recording: Some(recording),
            start_answers: vec![],
            stop_answers: vec![],
            pending: false,
            drain_until: None,
            stop_wake: None,
            stop_timer_at: None,
            finalise_delay: Duration::from_millis(5),
            signature: vec![],
            deferred_signature: None,
            failed: None,
            last_state: Cell::new("recording"),
        }))
    }

    fn drain_fixture(id: &str, root: &std::path::Path) -> Rc<RefCell<Session>> {
        let session = session_fixture(id, root);
        session
            .borrow_mut()
            .recording
            .as_mut()
            .unwrap()
            .frame(Some((2, 2, std::sync::Arc::new(vec![7; 12]), 0)))
            .unwrap();
        session
    }

    #[test]
    fn stalled_fence_stop_timer_reclaims_slot_and_allows_same_and_other_output() {
        use smithay::backend::{
            egl::{EGLContext, EGLDevice, EGLDisplay},
            renderer::{Bind, ImportMem, gles::GlesRenderer},
        };
        use smithay::output::{Output, PhysicalProperties, Subpixel};
        use smithay::reexports::calloop::EventLoop;
        use std::sync::atomic::Ordering;
        let node = std::env::var("MIX_TEST_VIUD_RENDER_NODE").unwrap_or_else(|_| {
            EGLDevice::enumerate()
                .ok()
                .and_then(|mut devices| devices.find_map(|device| device.render_device_path().ok()))
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_else(|| "/dev/dri/no-render-device".into())
        });
        if !std::path::Path::new(&node).exists() {
            assert_ne!(
                std::env::var("MIX_TEST_REQUIRE_RECORD").as_deref(),
                Ok("1"),
                "render node required"
            );
            return;
        }
        let device = EGLDevice::enumerate()
            .unwrap()
            .find(|d| d.render_device_path().ok().as_deref() == Some(std::path::Path::new(&node)))
            .unwrap();
        // SAFETY: the owned display/context are used only on this test thread.
        let display = unsafe { EGLDisplay::new(device).unwrap() };
        let context = EGLContext::new(&display).unwrap();
        let mut renderer = unsafe { GlesRenderer::new(context).unwrap() };
        let outputs: Vec<_> = ["same", "other"]
            .into_iter()
            .map(|name| {
                Output::new(
                    name.into(),
                    PhysicalProperties {
                        size: (0, 0).into(),
                        subpixel: Subpixel::Unknown,
                        make: "test".into(),
                        model: "test".into(),
                        serial_number: name.into(),
                    },
                )
            })
            .collect();
        let root = std::env::temp_dir().join(format!(
            "record-stall-{}-{}",
            std::process::id(),
            recorder::bus::monotonic_us().unwrap()
        ));
        std::fs::create_dir(&root).unwrap();
        let mut events: EventLoop<'static, ()> = EventLoop::try_new().unwrap();

        let submit = |session: &Rc<RefCell<Session>>,
                      renderer: &mut GlesRenderer,
                      output: &Output| {
            let token = format!("/{}-0-0", session.borrow().pixel_secret);
            session.borrow_mut().pending = true;
            let owner = session.clone();
            screencopy::file::raw_answer(token.clone(), move |result| match result {
                Ok((w, h, pixels, cost)) => {
                    let mut s = owner.borrow_mut();
                    s.pending = false;
                    s.recording
                        .as_mut()
                        .unwrap()
                        .frame(Some((w, h, pixels, cost)))
                        .unwrap();
                    if s.drain_until.is_some() && stop_stream(&mut s, false) {
                        event(&s, "finishing");
                    }
                }
                Err(_) => fail(&owner, "readback expired"),
            });
            let spec = comp_model::capture::parse(&json!({"path":token,"format":"ppm"})).unwrap();
            let failed_token = token.clone();
            screencopy::file::enqueue(output, spec, move |result| {
                if let Err(error) = result {
                    screencopy::file::raw_fail(&failed_token, error);
                }
            });
            let mut texture = renderer
                .import_memory(
                    &[9; 16],
                    smithay::backend::allocator::Fourcc::Abgr8888,
                    (2, 2).into(),
                    false,
                )
                .unwrap();
            let target = renderer.bind(&mut texture).unwrap();
            screencopy::file::service(
                renderer,
                screencopy::Source {
                    framebuffer: &target,
                    readback: screencopy::Readback {
                        size: (2, 2).into(),
                        origin_bottom_left: false,
                    },
                },
                output,
                screencopy::file::CaptureSource::Offscreen,
                true,
            );
        };
        let stop = |session: &Rc<RefCell<Session>>, discard| {
            CURRENT.with_borrow_mut(|current| *current = Some(session.clone()));
            let reply = Rc::new(RefCell::new(None));
            let answer = reply.clone();
            let id = session.borrow().id.clone();
            session_command(
                RecordCommand::Stop {
                    session: id,
                    discard,
                },
                Box::new(move |value| {
                    let ControlReply::Body(value) = value else {
                        panic!("stop refused")
                    };
                    *answer.borrow_mut() = Some(value);
                }),
            );
            reply
        };
        let await_reply = |events: &mut EventLoop<'static, ()>,
                           reply: &Rc<RefCell<Option<Value>>>| {
            let limit = Instant::now() + Duration::from_secs(2);
            while reply.borrow().is_none() && Instant::now() < limit {
                events
                    .dispatch(Some(Duration::from_millis(10)), &mut ())
                    .unwrap();
            }
            assert!(
                reply.borrow().is_some(),
                "stop must complete without recording ticks or a fence answer"
            );
        };

        // No pacing timer and no renderer polls: only the real stop deadline
        // can deliver this reply. Changing CURRENT also probes timer ownership.
        let old = drain_fixture("hung", &root);
        install_stop_waker(&old, events.handle());
        let fence = screencopy::file::testing::stall_next_read();
        submit(&old, &mut renderer, &outputs[0]);
        screencopy::file::set_paused(&outputs[0], true);
        std::thread::sleep(Duration::from_millis(1100));
        screencopy::file::poll(&mut renderer, &outputs[1]);
        assert_eq!(
            screencopy::file::testing::read_count(),
            1,
            "inactive output keeps its read beyond READ_TIMEOUT"
        );
        screencopy::file::set_paused(&outputs[0], false);
        assert!(screencopy::file::pending(&outputs[0]));
        assert!(!screencopy::file::requested(&outputs[0]));
        assert!(!screencopy::file::pending_picture(&outputs[0], true));
        let began = Instant::now();
        let reply = stop(&old, false);
        assert!(reply.borrow().is_none());
        assert_eq!(
            old.borrow().last_state.get(),
            "recording",
            "deferred stop must not emit finishing"
        );
        let newer = drain_fixture("newer", &root);
        install_stop_waker(&newer, events.handle());
        CURRENT.with_borrow_mut(|current| *current = Some(newer.clone()));
        await_reply(&mut events, &reply);
        let stopped_ms = began.elapsed().as_millis();
        assert!(began.elapsed() < Duration::from_secs(2));
        assert!(
            newer.borrow().recording.as_ref().unwrap().is_recording(),
            "old timeout sealed the new take"
        );
        assert_eq!(screencopy::file::testing::read_count(), 0);
        assert_eq!(screencopy::file::testing::pool_count(), 0);
        assert_eq!(
            fence.load(Ordering::Relaxed),
            1,
            "fence ownership must be released"
        );
        await_reply(&mut events, &stop(&newer, false));

        // Reuse the real queue/GPU/writer on the same and a different output.
        for (index, output) in outputs.iter().enumerate() {
            let s = drain_fixture(&format!("reuse-{index}"), &root);
            install_stop_waker(&s, events.handle());
            submit(&s, &mut renderer, output);
            screencopy::file::set_paused(output, true);
            std::thread::sleep(Duration::from_millis(1100));
            screencopy::file::set_paused(output, false);
            let reply = stop(&s, false);
            let limit = Instant::now() + Duration::from_secs(2);
            while reply.borrow().is_none() && Instant::now() < limit {
                screencopy::file::poll(&mut renderer, output);
                events
                    .dispatch(Some(Duration::from_millis(2)), &mut ())
                    .unwrap();
            }
            assert!(reply.borrow().is_some());
            assert_eq!(reply.borrow().as_ref().unwrap()["dropped"], 0);
            assert_eq!(screencopy::file::testing::read_count(), 0);
            assert_eq!(screencopy::file::testing::pool_count(), 0);
        }
        let s = drain_fixture("discard", &root);
        install_stop_waker(&s, events.handle());
        let fence = screencopy::file::testing::stall_next_read();
        submit(&s, &mut renderer, &outputs[0]);
        let reply = stop(&s, true);
        assert_eq!(
            screencopy::file::testing::read_count(),
            0,
            "discard must release immediately"
        );
        assert_eq!(fence.load(Ordering::Relaxed), 1);
        await_reply(&mut events, &reply);

        let s = drain_fixture("expire", &root);
        install_stop_waker(&s, events.handle());
        let fence = screencopy::file::testing::stall_next_read();
        submit(&s, &mut renderer, &outputs[0]);
        assert!(screencopy::file::testing::take_wake());
        screencopy::file::testing::age_reads();
        screencopy::file::poll(&mut renderer, &outputs[1]);
        assert_eq!(
            screencopy::file::testing::read_count(),
            0,
            "expiry must apply across outputs"
        );
        assert_eq!(fence.load(Ordering::Relaxed), 1);
        assert!(
            !screencopy::file::testing::take_wake(),
            "expired reads must not re-offer 1ms wakes"
        );
        await_reply(&mut events, &stop(&s, false));
        CURRENT.with_borrow_mut(|current| *current = None);
        drop((old, newer, s));
        std::fs::remove_dir_all(root).unwrap();
        println!(
            "STALLED_FENCE stop_bound_ms=1000 reply_ms={stopped_ms} same_output=true other_output=true discard=true expiry=true"
        );
    }
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
    fn failure_during_drain_clears_capture_and_drain_state() {
        let root = std::env::temp_dir().join(recorder::bus::private_token().unwrap());
        std::fs::create_dir(&root).unwrap();
        let session = drain_fixture("failed-drain", &root);
        session.borrow_mut().pending = true;
        session.borrow_mut().drain_until = Some(Instant::now());
        fail(&session, "capture failure during drain");
        assert!(!session.borrow().pending);
        assert!(session.borrow().drain_until.is_none());
        assert_eq!(session.borrow().last_state.get(), "failed");
        while !finish_recording(&mut session.borrow_mut()) {
            std::thread::sleep(Duration::from_millis(5));
        }
        drop(session);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn host_assigns_origin_from_capture_completion() {
        let root = std::env::temp_dir().join(recorder::bus::private_token().unwrap());
        std::fs::create_dir(&root).unwrap();
        let session = session_fixture("origin-host", &root);
        let mut s = session.borrow_mut();
        let recording = s.recording.take().unwrap();
        let captured_us = recorder::bus::monotonic_us().unwrap();
        assign_first_capture(&mut s, recording, captured_us);
        s.recording
            .as_mut()
            .unwrap()
            .frame(Some((2, 2, std::sync::Arc::new(vec![7; 12]), 0)))
            .unwrap();
        assert_eq!(
            s.recording.as_ref().unwrap().started()["t0_us"],
            captured_us
        );
        let marker = s
            .recording
            .as_mut()
            .unwrap()
            .marker("aligned".into(), Value::Null)
            .unwrap();
        assert!(
            marker["t_us"].as_u64().unwrap()
                <= recorder::bus::monotonic_us().unwrap() - captured_us
        );
        stop_stream(&mut s, false);
        while !finish_recording(&mut s) {
            std::thread::sleep(Duration::from_millis(5));
        }
        let origin: Value = serde_json::from_slice(
            &std::fs::read(root.join("origin-host.vfl/origin.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(origin["t0_us"], captured_us);
        drop(s);
        drop(session);
        std::fs::remove_dir_all(root).unwrap();
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
            stop_wake: None,
            stop_timer_at: None,
            finalise_delay: Duration::from_millis(5),
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
            s.admitted = Instant::now();
            s.pending = false;
            s.probe = Some(rx);
            let got = answered.clone();
            s.start_answers.push(Box::new(move |reply| {
                assert!(!matches!(reply, ControlReply::Body(_)));
                got.set(true);
            }));
        }
        tick(&session, || {
            panic!("failed preflight must never schedule capture")
        });
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
