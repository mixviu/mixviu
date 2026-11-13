// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Video encoding via an ffmpeg subprocess.
//!
//! Raw BGRA frames are piped to external ffmpeg using x264 or optional VAAPI. A
//! dedicated thread owns stdin so the render path never blocks on the pipe.
//! Output goes to a temp file; the caller moves it to the final destination on
//! Save (or deletes it on Discard).

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Stdio};

use slots::library::process::child::hygiene::command;
use slots::library::process::child::spawn as child_spawn;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

enum VideoFrame {
    Bgra(Arc<[u8]>),
    Rgb {
        pixels: Arc<[u8]>,
        width: u32,
        height: u32,
    },
}

pub struct VideoEncoder {
    tx: Option<SyncSender<QueuedFrame>>,
    join: Option<JoinHandle<()>>,
    stderr_join: Option<JoinHandle<()>>,
    child: Option<Child>,
    temp: PathBuf,
    width: u32,
    height: u32,
    stderr: Arc<Mutex<String>>,
    queued: Arc<AtomicUsize>,
    scheduled: Arc<AtomicU64>,
    closed: Arc<AtomicBool>,
    cleanup: bool,
}

struct QueuedFrame {
    slot: u64,
    pixels: VideoFrame,
}

/// Fill gaps, including a dropped tail at shutdown, without retaining an
/// unbounded backlog of pixel buffers. Every scheduled slot reaches ffmpeg.
fn write_frames(writer: &mut impl Write, rx: Receiver<QueuedFrame>, scheduled: &AtomicU64,
    queued: &AtomicUsize, width:u32,height:u32) -> std::io::Result<()> {
    let mut next=0;
    let mut last:Arc<[u8]>=vec![0;width as usize*height as usize*4].into();
    let mut previous_rgb:Option<Arc<[u8]>>=None;
    for frame in rx {
        queued.fetch_sub(1,Ordering::Relaxed);
        for _ in next..frame.slot {writer.write_all(&last)?;}
        let pixels=match frame.pixels {
            VideoFrame::Bgra(pixels)=>{previous_rgb=None;pixels},
            VideoFrame::Rgb{pixels,width:source_width,height:source_height}=>{
                if previous_rgb.as_ref().is_some_and(|old|Arc::ptr_eq(old,&pixels)) {last.clone()}
                else {
                    let mut converted=vec![0;width as usize*height as usize*4];
                    for (input,output) in pixels.chunks_exact(source_width as usize*3).take(source_height as usize).zip(converted.chunks_exact_mut(width as usize*4)) {
                        for (p,out) in input.chunks_exact(3).zip(output.chunks_exact_mut(4)) {out.copy_from_slice(&[p[2],p[1],p[0],255]);}
                    }
                    previous_rgb=Some(pixels);
                    converted.into()
                }
            }
        };
        writer.write_all(&pixels)?;
        last=pixels;
        next=frame.slot+1;
    }
    for _ in next..scheduled.load(Ordering::Acquire) {writer.write_all(&last)?;}
    Ok(())
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

    pub fn start_at(
        width: u32,
        height: u32,
        fps: u32,
        path: Option<PathBuf>,
    ) -> Result<Self, String> {
        Self::start_with_encoder(width, height, fps, path, "x264")
    }

    pub fn start_with_encoder(
        width: u32,
        height: u32,
        fps: u32,
        path: Option<PathBuf>,
        encoder: &str,
    ) -> Result<Self, String> {
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
        let temp =
            path.unwrap_or_else(|| std::env::temp_dir().join(format!("viud-capture-{nanos}.mp4")));

        let mut cmd = command("ffmpeg");
        if encoder == "vaapi" {
            cmd.args(["-vaapi_device", "/dev/dri/renderD128"]);
        }
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
        ]);
        match encoder {
            "x264" => {
                cmd.args([
                    "-pix_fmt",
                    "yuv420p",
                    "-c:v",
                    "libx264",
                    "-preset",
                    "ultrafast",
                    "-crf",
                    "14",
                ]);
            }
            "vaapi" => {
                cmd.args([
                    "-vf",
                    "format=nv12,hwupload",
                    "-c:v",
                    "h264_vaapi",
                    "-qp",
                    "18",
                ]);
            }
            _ => return Err("unknown recording encoder".into()),
        }
        cmd.args(["-g", "30", "-an"])
            .arg(&temp)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        let mut child =
            child_spawn::spawn_awaited(&mut cmd).map_err(|e| format!("ffmpeg spawn: {e}"))?;

        let mut stdin = child.stdin.take().ok_or("ffmpeg stdin unavailable")?;
        let stderr = Arc::new(Mutex::new(String::new()));
        let errors = stderr.clone();
        let stderr_join = child.stderr.take().map(|mut pipe| {
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
            })
        });
        let (tx, rx) = sync_channel::<QueuedFrame>(4);
        let queued = Arc::new(AtomicUsize::new(0));
        let queue_count = queued.clone();
        let scheduled=Arc::new(AtomicU64::new(0));
        let slots=scheduled.clone();
        let closed = Arc::new(AtomicBool::new(false));
        let writer_closed = closed.clone();
        let join = std::thread::spawn(move || {
            let _ = write_frames(&mut stdin,rx,&slots,&queue_count,w,h);
            writer_closed.store(true, Ordering::Release);
            // Dropping `stdin` here signals EOF so ffmpeg flushes and exits.
        });

        Ok(Self {
            tx: Some(tx),
            join: Some(join),
            stderr_join,
            child: Some(child),
            temp,
            width: w,
            height: h,
            stderr,
            queued,
            scheduled,
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
        self.try_push_shared(bgra.into())
    }

    /// Repeated frames share the same immutable allocation, including at 4K.
    pub fn try_push_shared(&self, bgra: Arc<[u8]>) -> bool {
        if bgra.len() != self.width as usize * self.height as usize * 4 {
            return false;
        }
        self.submit(VideoFrame::Bgra(bgra))
    }

    pub fn try_push_rgb(&self, pixels: Arc<[u8]>, width: u32, height: u32) -> bool {
        if width == 0
            || height == 0
            || width > self.width
            || height > self.height
            || pixels.len() != width as usize * height as usize * 3
        {
            return false;
        }
        self.submit(VideoFrame::Rgb {
            pixels,
            width,
            height,
        })
    }

    fn submit(&self, frame: VideoFrame) -> bool {
        let slot=self.scheduled.fetch_add(1,Ordering::AcqRel);
        self.queued.fetch_add(1, Ordering::Relaxed);
        if self
            .tx
            .as_ref()
            .is_some_and(|tx| tx.try_send(QueuedFrame{slot,pixels:frame}).is_ok())
        {
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
                    break Err(format!("ffmpeg {status}"));
                }
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(format!("ffmpeg wait: {error}"));
                }
                _ if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err("ffmpeg finalise timeout".into());
                }
                _ => std::thread::sleep(Duration::from_millis(10)),
            }
        };
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        if let Some(join) = self.stderr_join.take() {
            let _ = join.join();
        }
        result.map_err(|error| format!("{error}: {}", self.stderr.lock().unwrap()))
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

/// Called on a preflight worker thread, never on the compositor thread.
pub fn probe_vaapi() -> Result<(), String> {
    let mut cmd = command("ffmpeg");
    cmd.args(["-hide_banner", "-encoders"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child =
        child_spawn::spawn_awaited(&mut cmd).map_err(|e| format!("ffmpeg encoder probe: {e}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or("ffmpeg probe stdout unavailable")?;
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stdout.take(1_048_576).read_to_end(&mut bytes);
        bytes
    });
    let deadline = Instant::now() + Duration::from_secs(3);
    let result = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                break if status.success() {
                    Ok(())
                } else {
                    Err("ffmpeg encoder probe failed".into())
                };
            }
            Err(error) => break Err(format!("ffmpeg encoder probe: {error}")),
            _ if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break Err("ffmpeg encoder probe timed out".into());
            }
            _ => std::thread::sleep(Duration::from_millis(10)),
        }
    };
    let bytes = reader.join().unwrap_or_default();
    result?;
    if String::from_utf8_lossy(&bytes)
        .split_whitespace()
        .any(|word| word == "h264_vaapi")
    {
        Ok(())
    } else {
        Err("ffmpeg does not report h264_vaapi".into())
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
                if cleanup {
                    let _ = std::fs::remove_file(path);
                }
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
    fn missing_slots_and_dropped_tail_repeat_the_previous_frame() {
        let (tx,rx)=sync_channel(4);
        tx.send(QueuedFrame{slot:0,pixels:VideoFrame::Bgra(Arc::from([1u8;16]))}).unwrap();
        tx.send(QueuedFrame{slot:2,pixels:VideoFrame::Bgra(Arc::from([2u8;16]))}).unwrap();
        drop(tx);
        let mut bytes=Vec::new();
        write_frames(&mut bytes,rx,&AtomicU64::new(4),&AtomicUsize::new(2),2,2).unwrap();
        assert_eq!(bytes.len(),64);
        assert_eq!(&bytes[..32],&[1;32]);
        assert_eq!(&bytes[32..],&[2;32]);
    }
    #[test]
    fn queue_overrun_never_waits_for_encoder() {
        // A deliberately stalled encoder: keep the receiving end alive but
        // never drain it. Exercise the real admission path deterministically.
        let (tx, _rx) = sync_channel(4);
        let encoder = VideoEncoder {
            tx: Some(tx),
            join: None,
            stderr_join: None,
            child: None,
            cleanup: true,
            temp: PathBuf::new(),
            width: 2,
            height: 2,
            stderr: Arc::new(Mutex::new(String::new())),
            queued: Arc::new(AtomicUsize::new(0)),
            scheduled:Arc::new(AtomicU64::new(0)),
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
