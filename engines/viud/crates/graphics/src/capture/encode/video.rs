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
use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread::JoinHandle;

pub struct VideoEncoder {
    tx: Option<SyncSender<Vec<u8>>>,
    join: Option<JoinHandle<()>>,
    child: Option<Child>,
    temp: PathBuf,
    width: u32,
    height: u32,
    stderr: Arc<Mutex<String>>,
}

impl VideoEncoder {
    /// Spawn ffmpeg encoding `width`×`height` BGRA frames at `fps` to a temp
    /// mp4. Dimensions are forced even (yuv420p requires it). Returns `None`
    /// if ffmpeg can't be spawned (treated as "video unavailable").
    pub fn start(width: u32, height: u32, fps: u32) -> Option<Self> {
        Self::start_result(width, height, fps).ok()
    }

    pub fn start_result(width: u32, height: u32, fps: u32) -> Result<Self, String> {
        let w = width & !1;
        let h = height & !1;
        if w == 0 || h == 0 {
            return Err("video dimensions must be at least 2x2".into());
        }
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let temp = std::env::temp_dir().join(format!("viud-capture-{nanos}.mkv"));

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
            "-c:v", "libx264", "-preset", "ultrafast", "-crf", "14", "-g", "30", "-an",
        ])
        .arg(&temp)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
        let mut child = child_spawn::spawn_awaited(&mut cmd)
            .map_err(|e| format!("ffmpeg spawn: {e}"))?;

        let mut stdin = child.stdin.take().ok_or("ffmpeg stdin unavailable")?;
        let stderr = Arc::new(Mutex::new(String::new()));
        let errors = stderr.clone();
        if let Some(mut pipe) = child.stderr.take() {
            std::thread::spawn(move || {
                let mut chunk = [0; 4096];
                while let Ok(n) = pipe.read(&mut chunk) {
                    if n == 0 { break; }
                    let mut text = errors.lock().unwrap();
                    // Drain the pipe continuously, retaining at most 64 KiB.
                    if text.len() < 65536 {
                        text.push_str(&String::from_utf8_lossy(&chunk[..n]));
                    }
                }
            });
        }
        let (tx, rx) = sync_channel::<Vec<u8>>(4);
        let join = std::thread::spawn(move || {
            while let Ok(buf) = rx.recv() {
                if stdin.write_all(&buf).is_err() {
                    break;
                }
            }
            // Dropping `stdin` here signals EOF so ffmpeg flushes and exits.
        });

        Some(Self {
            tx: Some(tx),
            join: Some(join),
            child: Some(child),
            temp,
            width: w,
            height: h,
            stderr,
        })
    }

    /// The fixed encoder dimensions (even). Frames must be fit to this size.
    pub fn dims(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Queue one BGRA frame (must be exactly `width*height*4` bytes).
    pub fn push(&self, bgra: Vec<u8>) {
        if let Some(tx) = &self.tx {
            let _ = tx.try_send(bgra);
        }
    }

    /// Non-blocking admission: false means the bounded queue overran or closed.
    pub fn try_push(&self, bgra: Vec<u8>) -> bool {
        bgra.len() == self.width as usize * self.height as usize * 4
            && self.tx.as_ref().is_some_and(|tx| tx.try_send(bgra).is_ok())
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
                Ok(Some(status)) => break Err(format!("ffmpeg {status}: {}", self.stderr.lock().unwrap())),
                Err(error) => break Err(format!("ffmpeg wait: {error}")),
                _ if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(format!("ffmpeg finalise timeout: {}", self.stderr.lock().unwrap()));
                }
                _ => std::thread::sleep(Duration::from_millis(10)),
            }
        };
        if let Some(join) = self.join.take() { let _ = join.join(); }
        result
    }

    /// Stop the stream, wait for ffmpeg to flush, and return the temp mp4 path
    /// (or `None` if ffmpeg exited non-zero / produced nothing).
    pub fn finish(mut self) -> Option<PathBuf> {
        self.tx = None; // close the channel → writer thread ends → stdin EOF
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
        let status = self.child.take().and_then(|mut c| c.wait().ok());
        match status {
            Some(s) if s.success() => Some(std::mem::take(&mut self.temp)),
            other => {
                warn!("ffmpeg exited unsuccessfully: {other:?}");
                let _ = std::fs::remove_file(&self.temp);
                None
            }
        }
    }

    /// Drop the encoder and delete the temp file (discard path).
    pub fn discard(mut self) {
        self.tx = None;
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
        if let Some(mut c) = self.child.take() {
            let _ = c.wait();
        }
        let _ = std::fs::remove_file(&self.temp);
    }
}
