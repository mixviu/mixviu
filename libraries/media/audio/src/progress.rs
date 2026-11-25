//! Opt-in AVP reporting. Only the detached writer ever touches stderr.
use serde_json::{Value, json};
use std::{
    io::Write,
    os::fd::FromRawFd,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
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
struct Event {
    stage: String,
    event: String,
    fields: Value,
}
struct Reporter {
    counters: Arc<Counters>,
    events: SyncSender<Event>,
    final_result: SyncSender<Value>,
    flushed: Mutex<Receiver<()>>,
}
pub struct Progress(Option<Reporter>);
impl Progress {
    pub fn new(enabled: bool) -> Self {
        if !enabled {
            return Self(None);
        }
        let counters = Arc::new(Counters::default());
        let (events, rx) = mpsc::sync_channel::<Event>(64);
        // Reserved final slot: a full event queue cannot lose done.
        let (final_result, done) = mpsc::sync_channel(1);
        let (ack, flushed) = mpsc::sync_channel(1);
        let shared = counters.clone();
        let start = Instant::now();
        let spawned = std::thread::Builder::new()
            .name("av-progress".into())
            .spawn(move || {
                // A separate handle avoids holding Rust's global stderr lock while
                // blocked in write. The descriptor remains blocking and is owned
                // exclusively by this writer; never change the caller's flags.
                // SAFETY: fcntl duplicates fd 2 without changing its flags. The
                // new descriptor shares its open file description (and offset).
                let fd = unsafe { libc::fcntl(2, libc::F_DUPFD_CLOEXEC, 3) };
                if fd < 0 {
                    let _ = crate::cli::diagnostic(format_args!(
                        "progress disabled: cannot duplicate stderr: {}",
                        std::io::Error::last_os_error()
                    ));
                    return;
                }
                // SAFETY: successful fcntl returned a fresh owned descriptor.
                let stderr = unsafe { std::fs::File::from_raw_fd(fd) };
                write_events(stderr, start, shared, rx, done, ack);
            });
        if spawned.is_err() {
            return Self(None);
        }
        Self(Some(Reporter {
            counters,
            events,
            final_result,
            flushed: Mutex::new(flushed),
        }))
    }
    pub fn event(&self, stage: &str, event: &str, fields: Value) {
        let Some(r) = &self.0 else { return };
        if let Some(total) = fields.get("frames_total").and_then(Value::as_u64) {
            r.counters.total.store(total, Ordering::Relaxed);
        }
        if r.events
            .try_send(Event {
                stage: stage.into(),
                event: event.into(),
                fields,
            })
            .is_err()
        {
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
        let _ = r
            .final_result
            .try_send(json!({"ok":matches!(result, Ok(0)),"receipt":"stdout"}));
        if let Ok(ack) = r.flushed.lock() {
            let _ = ack.recv_timeout(Duration::from_secs(1));
        }
        // Never join: a blocked writer must not delay the command's exit.
    }
}
fn update_stage(video: &mut bool, e: &Event) {
    if e.stage == "mux" || e.stage == "done" {
        // A video/end may have been dropped. Any later stage boundary
        // invalidates video sampling before that boundary is written.
        *video = false;
    } else if e.stage == "video" && e.fields.get("chunk").is_none() {
        *video = e.event == "start";
    }
}
fn write_events(
    mut stderr: impl Write,
    start: Instant,
    shared: Arc<Counters>,
    rx: Receiver<Event>,
    done: Receiver<Value>,
    ack: SyncSender<()>,
) {
                let mut next = Instant::now() + TICK;
                let mut video = false;
                loop {
                    match rx.recv_timeout(next.saturating_duration_since(Instant::now())) {
                        Ok(e) => {
                            update_stage(&mut video, &e);
                            emit(&mut stderr, start, &shared, &e.stage, &e.event, e.fields);
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                    }
                    // Drain ordered boundaries before the final line. No producer
                    // can enqueue after done: all chunk workers have joined.
                    for e in rx.try_iter() {
                        update_stage(&mut video, &e);
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
                        if video {
                            emit(&mut stderr, start, &shared, "video", "progress", json!({}));
                        }
                        next = Instant::now() + TICK;
                    }
                }
}
fn emit(
    out: &mut impl Write,
    start: Instant,
    c: &Counters,
    stage: &str,
    event: &str,
    fields: Value,
) {
    let mut value = json!({"v":1,"t":start.elapsed().as_secs_f64(),"stage":stage,"event":event});
    let total = c.total.load(Ordering::Relaxed);
    if total > 0 {
        value["frames_out"] = json!(c.out.load(Ordering::Relaxed));
        value["frames_encoded"] = json!(c.encoded.load(Ordering::Relaxed));
        value["frames_total"] = json!(total);
    }
    if let (Some(dst), Some(src)) = (value.as_object_mut(), fields.as_object()) {
        dst.extend(src.clone());
    }
    if stage == "done" {
        value["events_dropped"] = json!(c.dropped.load(Ordering::Relaxed));
    }
    let _ = writeln!(out, "AVP {value}").and_then(|()| out.flush());
}

/// Remove the valueless flag before the command's value-option parser.
pub fn arguments(
    args: impl IntoIterator<Item = std::ffi::OsString>,
) -> (Vec<std::ffi::OsString>, Progress) {
    let (args, enabled) = split_arguments(args);
    (args, Progress::new(enabled))
}

fn split_arguments(
    args: impl IntoIterator<Item = std::ffi::OsString>,
) -> (Vec<std::ffi::OsString>, bool) {
    let mut enabled = false;
    let mut args = args.into_iter();
    let mut kept = Vec::new();
    // The first argument is always the positional timeline path.
    kept.extend(args.next());
    while let Some(arg) = args.next() {
        if arg == "--progress" && !enabled {
            enabled = true;
        } else {
            kept.push(arg);
            // All remaining options consume one value, even invalid ones;
            // leave validation and duplicate errors to the command parser.
            kept.extend(args.next());
        }
    }
    (kept, enabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_only_in_flag_position() {
        for args in [
            vec!["--progress", "--out", "output"],
            vec!["timeline", "--out", "--progress"],
            vec!["timeline", "--wav", "--progress", "--out", "output"],
        ] {
            let input: Vec<_> = args.into_iter().map(std::ffi::OsString::from).collect();
            let (kept, enabled) = split_arguments(input.clone());
            assert!(!enabled);
            assert_eq!(kept, input);
        }
        let (kept, enabled) = split_arguments(
            ["--progress", "--progress", "--out", "--progress"]
                .map(std::ffi::OsString::from),
        );
        assert!(enabled);
        assert_eq!(kept, ["--progress", "--out", "--progress"]);
    }

    #[derive(Clone)]
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn dropped_video_end_cannot_sample_after_mux() {
        let counters = Arc::new(Counters::default());
        let (events, rx) = mpsc::sync_channel(64);
        let (final_result, done) = mpsc::sync_channel(1);
        let (ack, flushed) = mpsc::sync_channel(1);
        let progress = Progress(Some(Reporter {
            counters: counters.clone(),
            events,
            final_result,
            flushed: Mutex::new(flushed),
        }));
        progress.event("video", "start", json!({}));
        for chunk in 0..63 {
            progress.event("video", "end", json!({"chunk": chunk}));
        }
        progress.event("video", "end", json!({}));
        assert_eq!(counters.dropped.load(Ordering::Relaxed), 1);
        let capture = Capture(Arc::new(Mutex::new(Vec::new())));
        let output = capture.clone();
        let writer = std::thread::spawn(move || {
            write_events(capture, Instant::now(), counters, rx, done, ack);
        });
        // Wait for a queue slot, ensuring mux/start survives after the lost end.
        progress.0.as_ref().unwrap().events.send(Event {
            stage: "mux".into(), event: "start".into(), fields: json!({}),
        }).unwrap();
        std::thread::sleep(TICK * 3);
        progress.done(&Ok(0));
        writer.join().unwrap();
        let bytes = output.0.lock().unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        let lines: Vec<Value> = text.lines().map(|line| {
            serde_json::from_str(line.strip_prefix("AVP ").unwrap()).unwrap()
        }).collect();
        let mux = lines.iter().position(|e| e["stage"] == "mux").unwrap();
        assert!(!lines[mux + 1..].iter().any(|e| {
            e["stage"] == "video" && e["event"] == "progress"
        }));
        assert_eq!(lines.last().unwrap()["events_dropped"], 1);
    }
}
