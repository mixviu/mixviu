//! Opt-in AVP reporting. Only the detached writer ever touches stderr.
use serde_json::{Value, json};
use std::{
    io::Write,
    sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}, mpsc::{self, SyncSender, Receiver}},
    time::{Duration, Instant},
};

const TICK: Duration = Duration::from_millis(500);
#[derive(Default)]
struct Counters {
    out: AtomicU64,
    encoded: AtomicU64,
    total: AtomicU64,
    dropped: AtomicU64,
}
struct Event { stage: String, event: String, fields: Value }
struct Reporter {
    counters: Arc<Counters>,
    events: SyncSender<Event>,
    final_result: SyncSender<Value>,
    flushed: Mutex<Receiver<()>>,
}
pub struct Progress(Option<Reporter>);
impl Progress {
    pub fn new(enabled: bool) -> Self {
        if !enabled { return Self(None); }
        let counters = Arc::new(Counters::default());
        let (events, rx) = mpsc::sync_channel(64);
        // Reserved final slot: a full event queue cannot lose done.
        let (final_result, done) = mpsc::sync_channel(1);
        let (ack, flushed) = mpsc::sync_channel(1);
        let shared = counters.clone();
        let start = Instant::now();
        let spawned = std::thread::Builder::new().name("av-progress".into()).spawn(move || {
            // A separate handle avoids holding Rust's global stderr lock while
            // blocked in write. The descriptor remains blocking and is owned
            // exclusively by this writer; never change the caller's flags.
            let Ok(mut stderr) = std::fs::OpenOptions::new().write(true).open("/dev/stderr") else { return };
            let mut next = Instant::now() + TICK;
            let mut video = false;
            loop {
                match rx.recv_timeout(next.saturating_duration_since(Instant::now())) {
                    Ok(e) => {
                        if e.stage == "video" && e.fields.get("chunk").is_none() {
                            video = e.event == "start";
                        }
                        emit(&mut stderr, start, &shared, &e.stage, &e.event, e.fields);
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                // Drain ordered boundaries before the final line. No producer
                // can enqueue after done: all chunk workers have joined.
                for e in rx.try_iter() {
                    if e.stage == "video" && e.fields.get("chunk").is_none() { video = e.event == "start"; }
                    emit(&mut stderr, start, &shared, &e.stage, &e.event, e.fields);
                }
                if let Ok(fields) = done.try_recv() {
                        // done may have arrived between the drain and this
                        // receive. Its send follows all boundary sends.
                        for e in rx.try_iter() {
                            emit(&mut stderr, start, &shared, &e.stage, &e.event, e.fields);
                        }
                        emit(&mut stderr, start, &shared, "done", "end", fields);
                        let _ = ack.try_send(());
                        break;
                }
                if Instant::now() >= next {
                    if video { emit(&mut stderr, start, &shared, "video", "progress", json!({})); }
                    next = Instant::now() + TICK;
                }
            }
        });
        if spawned.is_err() { return Self(None); }
        Self(Some(Reporter { counters, events, final_result, flushed: Mutex::new(flushed) }))
    }
    pub fn event(&self, stage: &str, event: &str, fields: Value) {
        let Some(r) = &self.0 else { return };
        if let Some(total) = fields.get("frames_total").and_then(Value::as_u64) {
            r.counters.total.store(total, Ordering::Relaxed);
        }
        if r.events.try_send(Event { stage: stage.into(), event: event.into(), fields }).is_err() {
            r.counters.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
    pub fn frames(&self, out: u64, encoded: u64) {
        let Some(r) = &self.0 else { return };
        r.counters.out.fetch_add(out, Ordering::Relaxed);
        r.counters.encoded.fetch_add(encoded, Ordering::Relaxed);
    }
    pub fn done(&self, result: &crate::timeline::Result<u8>) {
        let Some(r) = &self.0 else { return };
        let _ = r.final_result.try_send(json!({"ok":matches!(result, Ok(0)),"receipt":"stdout"}));
        if let Ok(ack) = r.flushed.lock() {
            let _ = ack.recv_timeout(Duration::from_secs(1));
        }
        // Never join: a blocked writer must not delay the command's exit.
    }
}
fn emit(out: &mut impl Write, start: Instant, c: &Counters, stage: &str, event: &str, fields: Value) {
    let mut value = json!({"v":1,"t":start.elapsed().as_secs_f64(),"stage":stage,"event":event});
    let total = c.total.load(Ordering::Relaxed);
    if total > 0 {
        value["frames_out"] = json!(c.out.load(Ordering::Relaxed));
        value["frames_encoded"] = json!(c.encoded.load(Ordering::Relaxed));
        value["frames_total"] = json!(total);
    }
    if let (Some(dst), Some(src)) = (value.as_object_mut(), fields.as_object()) { dst.extend(src.clone()); }
    if stage == "done" { value["events_dropped"] = json!(c.dropped.load(Ordering::Relaxed)); }
    let _ = writeln!(out, "AVP {value}").and_then(|()| out.flush());
}

/// Remove the valueless flag before the command's value-option parser.
pub fn arguments(args: impl IntoIterator<Item = std::ffi::OsString>) -> (Vec<std::ffi::OsString>, Progress) {
    let mut enabled = false;
    let args = args.into_iter().filter(|arg| {
        if arg == "--progress" && !enabled { enabled = true; false } else { true }
    }).collect();
    (args, Progress::new(enabled))
}
