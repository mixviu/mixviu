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
    rc::Rc,
    time::{Duration, Instant},
};
use world::state::Loop;

type Answer = Box<dyn FnOnce(ControlReply)>;
#[derive(Clone)]
struct Past {
    id: String,
    start: Option<Value>,
    terminal: Value,
    status: Value,
    start_error: Option<String>,
}
struct Session {
    request_id: String,
    id: String,
    path: String,
    targets: Vec<CaptureFrameSpec>,
    fps: u32,
    encoder: String,
    probe: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
    max_seconds: u64,
    recording: Option<Recording>,
    start: Option<Value>,
    start_answers: Vec<Answer>,
    stop_answers: Vec<Answer>,
    pending: bool,
    signature: Vec<u64>,
    failed: Option<String>,
    last_state: Cell<&'static str>,
}
thread_local! {
    static PAST: RefCell<std::collections::HashMap<String,Past>> = RefCell::new(std::collections::HashMap::new());
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
    json!({"state":state,"frames":0,"repeated":0,"dropped":0,"duration_us":0,"bytes":0,"queue":0,"readbacks":0,"readback_us":0,"readback_mean_us":0,"readback_max_us":0,"error":error})
}
fn event(s: &Session, state: &'static str) {
    if s.last_state.replace(state) == state { return; }
    EVENTS.with_borrow_mut(|events| {
        events.push(json!({"session":s.id,"state":state,"error":s.failed}))
    });
}

pub fn command(lp: &mut Loop, op: RecordCommand, answer: impl FnOnce(ControlReply) + 'static) {
    let answer: Answer = Box::new(answer);
    if let RecordCommand::Start { request_id, .. } = &op {
        if let Some(past) = PAST.with_borrow(|past| past.get(request_id).cloned()) {
            answer(
                past.start
                    .map(ControlReply::Body)
                    .unwrap_or_else(|| refused(past.start_error.as_deref().unwrap_or("previous start failed"))),
            );
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
        if let Some(past) =
            id.and_then(|id| PAST.with_borrow(|past| past.values().find(|p| p.id == *id).cloned()))
        {
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
    } = op
    {
        let previous = CURRENT.with_borrow(Clone::clone);
        if let Some(previous) = previous {
            let mut s = previous.borrow_mut();
            if s.request_id == request_id {
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
                    outputs.iter().find(|o| o.name() == *name)
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
        let id = format!("record-{}", recorder::bus::monotonic_us());
        let probe = if encoder == "vaapi" {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = tx.send(recorder::bus::probe_vaapi());
            });
            Some(rx)
        } else {
            None
        };
        let session = Rc::new(RefCell::new(Session {
            request_id,
            id,
            path,
            targets,
            fps,
            encoder,
            probe,
            max_seconds,
            recording: None,
            start: None,
            start_answers: vec![answer],
            stop_answers: Vec::new(),
            pending: false,
            signature: Vec::new(),
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
    let current = CURRENT.with_borrow(Clone::clone);
    let Some(current) = current else {
        if matches!(op, RecordCommand::Status { session: None }) {
            answer(ControlReply::Body(
                empty_status("idle",None),
            ));
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
        RecordCommand::Status{..} => answer(ControlReply::Body(s.recording.as_ref().map(Recording::status).unwrap_or_else(||empty_status(if s.failed.is_some(){"failed"}else{"recording"},s.failed.as_deref())))),
        RecordCommand::Marker{label,data,..} => answer(match s.recording.as_mut().ok_or("first frame pending").and_then(|r|r.marker(label,data).map_err(|_|"marker write failed")) { Ok(value)=>ControlReply::Body(value),Err(error)=>refused(error) }),
        RecordCommand::Stop{discard,..} => {
            if let Some(r)=s.recording.as_mut() {
                if let Some(reply)=&r.terminal { answer(ControlReply::Body(reply.clone())); return; }
                let changed=r.stop(discard);
                s.stop_answers.push(answer);
                if changed { event(&s,"finishing"); }
            } else { answer(refused("first frame pending or session failed")); }
        }
        _=>unreachable!(),
    }
}

fn fail(session: &Rc<RefCell<Session>>, error: &str) {
    let mut s = session.borrow_mut();
    if s.failed.is_some() { return; }
    s.failed = Some(error.into());
    for answer in std::mem::take(&mut s.start_answers) {
        answer(refused(error));
    }
    if let Some(r) = s.recording.as_mut() {
        r.error = Some(error.into());
        r.stop(false);
    } else {
        let past = Past { id:s.id.clone(), start:None, start_error:Some(error.into()),
            terminal:json!({"path":s.path,"frames":0,"repeated":0,"dropped":0,"duration_us":0,"frames_path":format!("{}.frames.jsonl",s.path),"markers_path":format!("{}.markers.jsonl",s.path),"encoder":s.encoder,"error":error}),
            status:empty_status("failed",Some(error)),
        };
        PAST.with_borrow_mut(|all| { all.insert(s.request_id.clone(),past); });
    }
    event(&s, "failed");
}

fn tick(lp: &mut Loop, session: &Rc<RefCell<Session>>) {
    {
        let mut s = session.borrow_mut();
        if let Some(probe) = &s.probe {
            match probe.try_recv() {
                Ok(Ok(())) => s.probe = None,
                Ok(Err(error)) => {
                    drop(s);
                    fail(session, &error);
                    return;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    drop(s);
                    fail(session, "encoder preflight disconnected");
                    return;
                }
                _ => return,
            }
        }
        if let Some(r) = s.recording.as_mut() {
            if r.poll() {
                let reply = r.terminal.clone().unwrap();
                let error = r.error.clone();
                if error.is_some() { s.failed = error; }
                let past = Past {
                    id: s.id.clone(),
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
        if s.recording
            .as_ref()
            .is_some_and(|r| recorder::bus::monotonic_us() - r.t0_us >= s.max_seconds * 1_000_000)
        {
            s.recording.as_mut().unwrap().stop(false);
            event(&s, "finishing");
            return;
        }
        if s.pending {
            let result = s.recording.as_mut().map(|r| r.frame(None));
            drop(s);
            if let Some(Err(error)) = result {
                fail(session, &error);
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
                signature.push(
                    lp.inner
                        .comp
                        .presentation
                        .stats
                        .output(&output.name())
                        .map(|s| s.frames)
                        .unwrap_or(0),
                );
            }
        }
        signature
    };
    let mut s = session.borrow_mut();
    if s.recording.is_some() && s.signature == signature {
        let result = s.recording.as_mut().unwrap().frame(None);
        drop(s);
        if let Err(error) = result {
            fail(session, &error);
        }
        return;
    }
    s.signature = signature;
    s.pending = true;
    let targets = s.targets.clone();
    let id = s.id.clone();
    drop(s);
    let frames = Rc::new(RefCell::new(Vec::new()));
    for (index, mut target) in targets.into_iter().enumerate() {
        let token = format!("/{id}-raw-{index}");
        target.path = token.clone();
        let got = frames.clone();
        let session = session.clone();
        screencopy::file::raw_answer(token.clone(), move |result| {
            {
                let s=session.borrow();
                if s.failed.is_some() || s.recording.as_ref().is_some_and(|r|!r.is_recording()) { return; }
            }
            let captured_us = recorder::bus::monotonic_us();
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
            let mut pixels = vec![0; width as usize * height as usize * 3];
            let mut x = 0;
            let readback_us = frames.iter().map(|(_, frame)| frame.3).sum::<u64>();
            for (_, (w, h, p, _)) in frames.iter() {
                for y in 0..*h as usize {
                    let at = (y * width as usize + x) * 3;
                    pixels[at..at + *w as usize * 3]
                        .copy_from_slice(&p[y * *w as usize * 3..(y + 1) * *w as usize * 3]);
                }
                x += *w as usize;
            }
            let mut s = session.borrow_mut();
            s.pending = false;
            if s.failed.is_some() || s.recording.as_ref().is_some_and(|r| !r.is_recording()) {
                return;
            }
            if s.recording.is_none() {
                match Recording::new(
                    s.id.clone(),
                    s.path.clone(),
                    width,
                    height,
                    s.fps,
                    s.encoder.clone(),
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
        });
        crate::capture::start(lp, target, Instant::now(), move |reply| {
            if !matches!(reply, ControlReply::Body(_)) {
                screencopy::file::raw_fail(&token, reply);
            }
        });
    }
}
