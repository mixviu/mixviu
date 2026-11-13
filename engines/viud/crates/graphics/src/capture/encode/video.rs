// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Video encoding via an ffmpeg subprocess.
//!
//! We pipe raw BGRA frames to `ffmpeg`'s stdin and let it mux to mp4 (codec
//! auto-selected by the container — H.264 where available, else mpeg4). A
//! dedicated thread owns stdin so the render path never blocks on the pipe.
//! Output goes to a temp file; the caller moves it to the final destination on
//! Save (or deletes it on Discard).

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Stdio};

use slots::library::process::child::hygiene::command;
use slots::library::process::child::spawn as child_spawn;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub struct VideoEncoder {
    tx: Option<SyncSender<Vec<u8>>>,
    join: Option<JoinHandle<()>>,
    child: Option<Child>,
    temp: PathBuf,
    width: u32,
    height: u32,
    stderr: Arc<Mutex<String>>,
    queued: Arc<AtomicUsize>,
    closed: Arc<AtomicBool>,
    cleanup: bool,
}

impl VideoEncoder {
    /// Spawn ffmpeg encoding `width`×`height` BGRA frames at `fps` to a temp
    /// mp4. Dimensions are forced even (yuv420p requires it). Returns `None`
    /// if ffmpeg can't be spawned (treated as "video unavailable").
    pub fn start(width: u32, height: u32, fps: u32) -> Option<Self> {
        Self::start_result(width, height, fps).ok()
    }

    pub fn start_result(width: u32, height: u32, fps: u32) -> Result<Self, String> {
        Self::start_at(width, height, fps, None)
    }

    pub fn start_at(width: u32, height: u32, fps: u32, path: Option<PathBuf>) -> Result<Self, String> {
        let w = width & !1;
        let h = height & !1;
        if w == 0 || h == 0 {
            return Err("video dimensions must be at least 2x2".into());
        }
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let cleanup = path.is_none();
        let temp = path.unwrap_or_else(|| std::env::temp_dir().join(format!("viud-capture-{nanos}.mkv")));

        let mut cmd = command("ffmpeg");
        cmd.args([
            "-y",
            "-loglevel",
            "error",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "bgra",
            "-s",
            &format!("{w}x{h}"),
            "-r",
            &fps.max(1).to_string(),
            "-i",
            "-",
            "-pix_fmt",
            "yuv420p",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-crf",
            "14",
            "-g",
            "30",
            "-an",
        ])
        .arg(&temp)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
        let mut child =
            child_spawn::spawn_awaited(&mut cmd).map_err(|e| format!("ffmpeg spawn: {e}"))?;

        let mut stdin = child.stdin.take().ok_or("ffmpeg stdin unavailable")?;
        let stderr = Arc::new(Mutex::new(String::new()));
        let errors = stderr.clone();
        if let Some(mut pipe) = child.stderr.take() {
            std::thread::spawn(move || {
                let mut chunk = [0; 4096];
                while let Ok(n) = pipe.read(&mut chunk) {
                    if n == 0 {
                        break;
                    }
                    let mut text = errors.lock().unwrap();
                    // Drain the pipe continuously, retaining at most 64 KiB.
                    if text.len() < 65536 {
                        text.push_str(&String::from_utf8_lossy(&chunk[..n]));
                    }
                }
            });
        }
        let (tx, rx) = sync_channel::<Vec<u8>>(4);
        let queued = Arc::new(AtomicUsize::new(0));
        let queue_count = queued.clone();
        let closed = Arc::new(AtomicBool::new(false));
        let writer_closed = closed.clone();
        let join = std::thread::spawn(move || {
            while let Ok(buf) = rx.recv() {
                queue_count.fetch_sub(1, Ordering::Relaxed);
                if stdin.write_all(&buf).is_err() {
                    break;
                }
            }
            writer_closed.store(true, Ordering::Release);
            // Dropping `stdin` here signals EOF so ffmpeg flushes and exits.
        });

        Ok(Self {
            tx: Some(tx),
            join: Some(join),
            child: Some(child),
            temp,
            width: w,
            height: h,
            stderr,
            queued,
            closed,
            cleanup,
        })
    }

    /// The fixed encoder dimensions (even). Frames must be fit to this size.
    pub fn dims(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Queue one BGRA frame (must be exactly `width*height*4` bytes).
    pub fn push(&self, bgra: Vec<u8>) {
        let _ = self.try_push(bgra);
    }

    /// Non-blocking admission: false means the bounded queue overran or closed.
    pub fn try_push(&self, bgra: Vec<u8>) -> bool {
        if bgra.len() != self.width as usize * self.height as usize * 4 {
            return false;
        }
        self.queued.fetch_add(1, Ordering::Relaxed);
        if self.tx.as_ref().is_some_and(|tx| tx.try_send(bgra).is_ok()) {
            true
        } else {
            self.queued.fetch_sub(1, Ordering::Relaxed);
            false
        }
    }

    pub fn queue(&self) -> usize {
        self.queued.load(Ordering::Relaxed)
    }
    pub fn error(&self) -> Option<String> {
        self.closed
            .load(Ordering::Acquire)
            .then(|| format!("ffmpeg pipe closed: {}", self.stderr.lock().unwrap()))
    }

    /// Call on a finaliser thread. Kill a stuck encoder after the deadline,
    /// which also releases a writer blocked on its pipe.
    pub fn finish_result(mut self) -> Result<PathBuf, String> {
        self.tx = None;
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut child = self.child.take().ok_or("encoder missing")?;
        let result = loop {
            match child.try_wait() {
                Ok(Some(status)) if status.success() => break Ok(std::mem::take(&mut self.temp)),
                Ok(Some(status)) => {
                    break Err(format!("ffmpeg {status}: {}", self.stderr.lock().unwrap()));
                }
                Err(error) => break Err(format!("ffmpeg wait: {error}")),
                _ if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(format!(
                        "ffmpeg finalise timeout: {}",
                        self.stderr.lock().unwrap()
                    ));
                }
                _ => std::thread::sleep(Duration::from_millis(10)),
            }
        };
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        result
    }

    /// Stop the stream, wait for ffmpeg to flush, and return the temp mp4 path
    /// (or `None` if ffmpeg exited non-zero / produced nothing).
    pub fn finish(self) -> Option<PathBuf> {
        self.finish_result().ok()
    }

    pub fn discard(self) {
        std::thread::spawn(move || {
            if let Ok(path) = self.finish_result() {
                let _ = std::fs::remove_file(path);
            }
        });
    }
}

impl Drop for VideoEncoder {
    fn drop(&mut self) {
        self.tx = None;
        if let Some(mut child) = self.child.take() {
            let path = std::mem::take(&mut self.temp);
            let cleanup = self.cleanup;
            std::thread::spawn(move || {
                let _ = child.kill();
                let _ = child.wait();
                if cleanup { let _ = std::fs::remove_file(path); }
            });
        } else if self.cleanup && !self.temp.as_os_str().is_empty() {
            let _ = std::fs::remove_file(&self.temp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queue_overrun_never_waits_for_encoder() {
        // A deliberately stalled encoder: keep the receiving end alive but
        // never drain it. Exercise the real admission path deterministically.
        let (tx, _rx) = sync_channel(4);
        let encoder = VideoEncoder {
            tx: Some(tx),
            join: None,
            child: None,
            cleanup: true,
            temp: PathBuf::new(),
            width: 2,
            height: 2,
            stderr: Arc::new(Mutex::new(String::new())),
            queued: Arc::new(AtomicUsize::new(0)),
            closed: Arc::new(AtomicBool::new(false)),
        };
        for _ in 0..4 {
            assert!(encoder.try_push(vec![0; 16]));
        }
        let started = Instant::now();
        for _ in 0..1000 {
            assert!(!encoder.try_push(vec![0; 16]));
        }
        assert!(started.elapsed() < Duration::from_secs(1));
        assert_eq!(encoder.queue(), 4);
        assert!(!encoder.try_push(vec![0; 15]));
        assert_eq!(encoder.queue(), 4);
    }
}
