//! Opt-in, synchronous AVP stderr reporting. No worker, transport or runtime.
use serde_json::{Value, json};
use std::{io::Write, sync::Mutex, time::{Duration, Instant}};

pub struct Progress(Option<Mutex<State>>);
struct State {
    start: Instant,
    last: Instant,
    frames_out: u64,
    frames_encoded: u64,
    frames_total: u64,
}
impl Progress {
    pub fn new(enabled: bool) -> Self {
        Self(enabled.then(|| Mutex::new(State {
            start: Instant::now(), last: Instant::now(), frames_out: 0,
            frames_encoded: 0, frames_total: 0,
        })))
    }
    pub fn event(&self, stage: &str, event: &str, fields: Value) {
        self.update(stage, event, fields, 0, 0);
    }
    pub fn frames(&self, out: u64, encoded: u64) {
        self.update("video", "progress", json!({}), out, encoded);
    }
    fn update(&self, stage: &str, event: &str, fields: Value, out: u64, encoded: u64) {
        let Some(state) = &self.0 else { return };
        let Ok(mut s) = state.lock() else { return };
        s.frames_out += out;
        s.frames_encoded += encoded;
        if let Some(total) = fields.get("frames_total").and_then(Value::as_u64) {
            s.frames_total = total;
        }
        if event == "progress" && s.last.elapsed() < Duration::from_millis(500) { return }
        s.last = Instant::now();
        let mut value = json!({"v":1,"t":s.start.elapsed().as_secs_f64(),"stage":stage,"event":event});
        if s.frames_total > 0 {
            value["frames_out"] = json!(s.frames_out);
            value["frames_encoded"] = json!(s.frames_encoded);
            value["frames_total"] = json!(s.frames_total);
        }
        if let (Some(dst), Some(src)) = (value.as_object_mut(), fields.as_object()) {
            dst.extend(src.clone());
        }
        let mut stderr = std::io::stderr().lock();
        // Progress is observational: an absent reader cannot change media output.
        let _ = writeln!(stderr, "AVP {value}").and_then(|()| stderr.flush());
    }
    pub fn done(&self, result: &crate::timeline::Result<u8>) {
        self.event("done", "end", json!({"ok":matches!(result, Ok(0)),"receipt": "stdout"}));
    }
}

/// Remove the valueless flag before the command's value-option parser.
pub fn arguments(args: impl IntoIterator<Item = std::ffi::OsString>) -> (Vec<std::ffi::OsString>, Progress) {
    let mut enabled = false;
    let args = args.into_iter().filter(|arg| {
        if arg == "--progress" && !enabled { enabled = true; false } else { true }
    }).collect();
    (args, Progress::new(enabled))
}
