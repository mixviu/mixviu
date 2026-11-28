// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Synchronous, best-effort crash evidence. Pause defers all trail I/O until
//! display and input have stopped; every other start is durable before work.
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    time::Instant,
};

const LIMIT: u64 = 1024 * 1024;
thread_local! {
    static TRAIL: RefCell<Option<Trail>> = const { RefCell::new(None) };
    static ACTIVATED: RefCell<Option<Instant>> = const { RefCell::new(None) };
}

struct Trail {
    path: PathBuf,
    socket: String,
    file: Option<File>,
    disabled: bool,
    warnings: usize,
    seq: u64,
    pending: Option<Vec<Value>>,
}
impl Trail {
    fn new(path: PathBuf, socket: String) -> Self {
        Self {
            path,
            socket,
            file: None,
            disabled: false,
            warnings: 0,
            seq: 0,
            pending: None,
        }
    }
    fn write(&mut self, mut row: Value) {
        self.seq += 1;
        row["seq"] = json!(self.seq);
        row["socket"] = json!(self.socket);
        if let Some(pending) = &mut self.pending {
            pending.push(row);
            return;
        }
        self.persist(&row);
    }
    fn persist(&mut self, row: &Value) {
        if self.disabled {
            return;
        }
        if let Err(e) = self.append(row) {
            self.disabled = true;
            self.file = None;
            self.warnings += 1;
            warn!("handover trail disabled: {e}");
        }
    }
    fn append(&mut self, row: &Value) -> io::Result<()> {
        let mut bytes = serde_json::to_vec(row)?;
        bytes.push(b'\n');
        if self.file.is_none() {
            std::fs::create_dir_all(self.path.parent().unwrap())?;
            self.file = Some(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.path)?,
            );
            // Make a new directory entry durable as well as its contents.
            File::open(self.path.parent().unwrap())?.sync_all()?;
        }
        if self.file.as_ref().unwrap().metadata()?.len() + bytes.len() as u64 > LIMIT {
            self.file.take();
            std::fs::rename(&self.path, self.path.with_extension("jsonl.1"))?;
            self.file = Some(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.path)?,
            );
            File::open(self.path.parent().unwrap())?.sync_all()?;
        }
        let file = self.file.as_mut().unwrap();
        file.write_all(&bytes)?;
        file.sync_data()
    }
    fn flush_pause(&mut self) {
        for row in self.pending.take().unwrap_or_default() {
            self.persist(&row);
        }
    }
}

pub fn init(socket: &str) {
    // A socket basename cannot contain a slash. Encode unexpected bytes rather
    // than permitting a path component to escape the state directory.
    let name: String = socket
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02x}")
            }
        })
        .collect();
    TRAIL.with_borrow_mut(|t| {
        *t = Some(Trail::new(
            slots::persist::path::base::state_dir().join(format!("handover-{name}.jsonl")),
            socket.into(),
        ))
    });
}

fn timestamp() -> (String, u64) {
    // Both clock_gettime calls and gmtime_r write only into valid stack storage.
    unsafe {
        let mut wall: libc::timespec = std::mem::zeroed();
        let mut boot: libc::timespec = std::mem::zeroed();
        libc::clock_gettime(libc::CLOCK_REALTIME, &mut wall);
        libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut boot);
        let mut tm: libc::tm = std::mem::zeroed();
        libc::gmtime_r(&wall.tv_sec, &mut tm);
        (
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                tm.tm_year + 1900,
                tm.tm_mon + 1,
                tm.tm_mday,
                tm.tm_hour,
                tm.tm_min,
                tm.tm_sec,
                wall.tv_nsec / 1_000_000
            ),
            boot.tv_sec as u64 * 1_000_000_000 + boot.tv_nsec as u64,
        )
    }
}
pub fn row(event: &str, connector: Option<u32>, crtc: Option<u32>, mode: Option<String>) -> Value {
    let (ts, boot) = timestamp();
    json!({"ts":ts,"ts_utc":ts,"boot_ns":boot,"pid":std::process::id(),"source":"comp","node":null,"kind":"handover","class":"operator","ref":null,"data":{},"event":event,"phase":"point","connector":connector,"crtc":crtc,"mode":mode})
}
pub fn pipe(event: &str, p: &crate::context::render::OutputPipe) -> Value {
    row(
        event,
        Some(p.connector.into()),
        Some(p.crtc.into()),
        Some(format!(
            "{}x{}@{:.3}",
            p.mode.size.w,
            p.mode.size.h,
            p.mode.refresh as f64 / 1000.0
        )),
    )
}
pub fn emit(row: Value) {
    TRAIL.with_borrow_mut(|t| {
        if let Some(t) = t {
            t.write(row);
        }
    });
}
pub fn point(event: &str) {
    emit(row(event, None, None, None));
}

/// Run time-critical pause work without any trail filesystem operations.
pub fn pause<R>(work: impl FnOnce() -> R) -> R {
    TRAIL.with_borrow_mut(|t| {
        if let Some(t) = t {
            t.pending = Some(Vec::new());
        }
    });
    point("PauseSession");
    let result = work();
    TRAIL.with_borrow_mut(|t| {
        if let Some(t) = t {
            t.flush_pause();
        }
    });
    result
}
/// All start rows are synced before the device-wide commit closure is called.
pub fn run<T>(rows: Vec<Value>, work: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    transaction(rows, work, |result| {
        ("end", result.is_ok(), result.as_ref().err().cloned().unwrap_or_default())
    })
}

/// Preserve the queue policy outcome while distinguishing a deferred attempt
/// from a successful submission. A queued modeset still awaits its vblank.
pub fn submit(
    rows: Vec<Value>,
    work: impl FnOnce() -> kms::scanout::flip::queue::QueueOutcome,
) -> kms::scanout::flip::queue::QueueOutcome {
    use kms::scanout::flip::queue::QueueOutcome;
    transaction(rows, work, |outcome| match outcome {
        QueueOutcome::Queued => ("end", true, String::new()),
        QueueOutcome::DeferredToWatchdog => ("deferred", false, "queue_frame deferred to resume watchdog (detail in journal)".into()),
        QueueOutcome::Failed => ("end", false, "queue_frame failed (detail in journal)".into()),
    })
}

fn transaction<T>(
    rows: Vec<Value>,
    work: impl FnOnce() -> T,
    classify: impl FnOnce(&T) -> (&'static str, bool, String),
) -> T {
    let start = Instant::now();
    for row in &rows {
        let mut row = row.clone();
        row["phase"] = json!("start");
        emit(row);
    }
    let result = work();
    let (phase, ok, error) = classify(&result);
    for mut row in rows {
        let (ts, boot) = timestamp();
        row["ts"] = json!(ts);
        row["ts_utc"] = row["ts"].clone();
        row["boot_ns"] = json!(boot);
        row["phase"] = json!(phase);
        row["ok"] = json!(ok);
        row["error"] = json!(error);
        row["elapsed_ms"] = json!(start.elapsed().as_secs_f64() * 1000.0);
        emit(row);
    }
    result
}
pub fn activated() {
    ACTIVATED.with_borrow_mut(|t| *t = Some(Instant::now()));
    point("ActivateSession");
}
pub fn first_vblank() {
    ACTIVATED.with_borrow_mut(|t| {
        if let Some(t) = t.take() {
            let mut r = row("FirstVblank", None, None, None);
            r["since_activate_ms"] = json!(t.elapsed().as_secs_f64() * 1000.0);
            emit(r);
        }
    });
}

#[cfg(test)]
mod tests;
