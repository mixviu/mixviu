// SPDX-License-Identifier: MIT OR Apache-2.0
//! viud admission/thread adapter; format and file I/O live in media-vfl.
pub use media_vfl::check_free_space;
use media_vfl::{Frame, Pixels, Writer};
use serde_json::json;
use std::{
    io,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        mpsc::{SyncSender, sync_channel},
    },
    thread::JoinHandle,
    time::Instant,
};
#[derive(Clone)]
pub struct Metrics {
    stats: Arc<Mutex<media_vfl::Stats>>,
    queue_wait_us: Arc<AtomicU64>,
    queue_wait_max_us: Arc<AtomicU64>,
    queue_overruns: Arc<AtomicU64>,
    dropped: Arc<AtomicU64>,
}
impl Metrics {
    pub fn json(&self) -> serde_json::Value {
        let stats = *self.stats.lock().unwrap();
        let mut value = stats.json();
        value["queue_wait_us"] = json!(self.queue_wait_us.load(Ordering::Relaxed));
        value["queue_wait_max_us"] = json!(self.queue_wait_max_us.load(Ordering::Relaxed));
        value["queue_overruns"] = json!(self.queue_overruns.load(Ordering::Relaxed));
        value["dropped_queue"] = json!(self.dropped.load(Ordering::Relaxed));
        value
    }
}
pub struct VideoEncoder {
    tx: Option<SyncSender<(Frame, Instant)>>,
    join: Option<JoinHandle<io::Result<()>>>,
    path: PathBuf,
    width: u32,
    height: u32,
    scheduled: Arc<AtomicU64>,
    dropped: Arc<AtomicU64>,
    queued: Arc<AtomicUsize>,
    error: Arc<Mutex<Option<String>>>,
    origin: Arc<AtomicU64>,
    total_dropped: Arc<AtomicU64>,
    finishing: Arc<AtomicBool>,
    written: Arc<AtomicU64>,
    stats: Arc<Mutex<media_vfl::Stats>>,
    queue_wait_us: Arc<AtomicU64>,
    queue_wait_max_us: Arc<AtomicU64>,
    queue_overruns: Arc<AtomicU64>,
}

impl VideoEncoder {
    pub fn start_result(w: u32, h: u32, fps: u32) -> Result<Self, String> {
        Self::start_at(w, h, fps, None)
    }
    pub fn start_at(w: u32, h: u32, fps: u32, path: Option<PathBuf>) -> Result<Self, String> {
        Self::start_with_encoder(w, h, fps, path, "vfl")
    }
    pub fn start_with_encoder(
        width: u32,
        height: u32,
        fps: u32,
        path: Option<PathBuf>,
        encoder: &str,
    ) -> Result<Self, String> {
        Self::start_bounded(width, height, fps, path, encoder, None)
    }
    pub fn start_bounded(
        width: u32,
        height: u32,
        fps: u32,
        path: Option<PathBuf>,
        encoder: &str,
        max_bytes: Option<u64>,
    ) -> Result<Self, String> {
        if encoder != "vfl" {
            return Err("available encoder: vfl".into());
        }
        media_vfl::frame_bytes(width, height, fps).map_err(|e| e.to_string())?;
        let path = path.unwrap_or_else(|| {
            std::env::temp_dir().join(format!(
                "viud-{}-{}.vfl",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ))
        });
        let worker_path = path.clone();
        // Four frame slots plus four reserved control slots. Pixel memory
        // remains bounded even while compression is busy with a key frame.
        let (tx, rx) = sync_channel::<(Frame, Instant)>(8);
        let scheduled = Arc::new(AtomicU64::new(0));
        let dropped = Arc::new(AtomicU64::new(0));
        let queued = Arc::new(AtomicUsize::new(0));
        let error = Arc::new(Mutex::new(None));
        let origin = Arc::new(AtomicU64::new(0));
        let total_dropped = Arc::new(AtomicU64::new(0));
        let (o, td) = (origin.clone(), total_dropped.clone());
        let finishing = Arc::new(AtomicBool::new(false));
        let finish = finishing.clone();
        let written = Arc::new(AtomicU64::new(0));
        let bytes = written.clone();
        let stats = Arc::new(Mutex::new(media_vfl::Stats::default()));
        let output_stats = stats.clone();
        let queue_wait_us = Arc::new(AtomicU64::new(0));
        let queue_wait_max_us = Arc::new(AtomicU64::new(0));
        let (wait_total, wait_max) = (queue_wait_us.clone(), queue_wait_max_us.clone());
        let queue_overruns = Arc::new(AtomicU64::new(0));
        let (s, d, q, e) = (
            scheduled.clone(),
            dropped.clone(),
            queued.clone(),
            error.clone(),
        );
        let join = std::thread::spawn(move || {
            let result = (|| -> io::Result<()> {
                let mut log = Writer::new_bounded(worker_path, width, height, fps, max_bytes)?;
                let mut origin_written = false;
                for (frame, admitted) in rx {
                    q.fetch_sub(1, Ordering::Relaxed);
                    let wait = admitted.elapsed().as_micros() as u64;
                    wait_total.fetch_add(wait, Ordering::Relaxed);
                    wait_max.fetch_max(wait, Ordering::Relaxed);
                    if !origin_written {
                        log.origin(o.load(Ordering::Acquire))?;
                        origin_written = true;
                    }
                    log.frame(frame)?;
                    *output_stats.lock().unwrap() = log.stats;
                    bytes.store(log.total_bytes, Ordering::Relaxed);
                }
                let error = e.lock().unwrap().clone();
                let total = log.finish(
                    s.load(Ordering::Acquire),
                    td.load(Ordering::Acquire).max(d.load(Ordering::Acquire)),
                    o.load(Ordering::Acquire),
                    error,
                    finish.load(Ordering::Acquire),
                )?;
                bytes.store(total, Ordering::Release);
                *output_stats.lock().unwrap() = log.stats;
                Ok(())
            })();
            if let Err(ref failure) = result {
                *e.lock().unwrap() = Some(failure.to_string());
            }
            result
        });
        Ok(Self {
            tx: Some(tx),
            join: Some(join),
            path,
            width,
            height,
            scheduled,
            dropped,
            queued,
            error,
            origin,
            total_dropped,
            finishing,
            written,
            stats,
            queue_wait_us,
            queue_wait_max_us,
            queue_overruns,
        })
    }
    pub fn dims(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn push(&self, p: Vec<u8>) {
        let _ = self.try_push(p);
    }
    pub fn try_push(&self, p: Vec<u8>) -> bool {
        self.try_push_shared(p.into())
    }
    pub fn try_push_shared(&self, p: Arc<[u8]>) -> bool {
        if p.len() != self.width as usize * self.height as usize * 4 {
            return false;
        }
        self.submit(Pixels::Bgra(p), true)
    }
    pub fn try_push_rgb(&self, p: Arc<Vec<u8>>, w: u32, h: u32) -> bool {
        self.try_push_rgb_change(p, w, h, true)
    }
    pub fn try_push_rgb_change(&self, p: Arc<Vec<u8>>, w: u32, h: u32, changed: bool) -> bool {
        if w == 0
            || h == 0
            || w > self.width
            || h > self.height
            || p.len() != w as usize * h as usize * 3
        {
            return false;
        }
        self.submit(Pixels::Rgb(p, w, h), changed)
    }
    fn submit(&self, pixels: Pixels, changed: bool) -> bool {
        let slot = self.scheduled.fetch_add(1, Ordering::AcqRel);
        let occupied = self.queued.fetch_add(1, Ordering::Relaxed);
        if occupied >= 4 {
            self.queued.fetch_sub(1, Ordering::Relaxed);
            self.queue_overruns.fetch_add(1, Ordering::Relaxed);
            if changed {
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
            return false;
        }
        if self.tx.as_ref().is_some_and(|tx| {
            tx.try_send((Frame { slot, pixels }, Instant::now()))
                .is_ok()
        }) {
            true
        } else {
            self.queued.fetch_sub(1, Ordering::Relaxed);
            self.queue_overruns.fetch_add(1, Ordering::Relaxed);
            if changed {
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
            false
        }
    }
    pub fn queue(&self) -> usize {
        self.queued.load(Ordering::Relaxed)
    }
    pub fn bytes_handle(&self) -> Arc<AtomicU64> {
        self.written.clone()
    }
    pub fn metrics_handle(&self) -> Metrics {
        Metrics {
            stats: self.stats.clone(),
            queue_wait_us: self.queue_wait_us.clone(),
            queue_wait_max_us: self.queue_wait_max_us.clone(),
            queue_overruns: self.queue_overruns.clone(),
            dropped: self.dropped.clone(),
        }
    }
    pub fn set_summary(&self, dropped: u64, t0_us: u64) {
        self.total_dropped.store(dropped, Ordering::Release);
        self.origin.store(t0_us, Ordering::Release);
    }
    pub fn marker(&self, row: serde_json::Value) -> Result<(), String> {
        self.queued.fetch_add(1, Ordering::Relaxed);
        if self.tx.as_ref().is_some_and(|tx| {
            tx.try_send((
                Frame {
                    slot: self.scheduled.load(Ordering::Acquire),
                    pixels: Pixels::Marker(row),
                },
                Instant::now(),
            ))
            .is_ok()
        }) {
            Ok(())
        } else {
            self.queued.fetch_sub(1, Ordering::Relaxed);
            Err("VFL marker queue full or closed".into())
        }
    }
    pub fn error(&self) -> Option<String> {
        self.error.lock().unwrap().clone()
    }
    pub fn finish_result(self) -> Result<PathBuf, String> {
        self.finish_inner(true)
    }
    pub fn finish_failed(self, error: String) -> Result<PathBuf, String> {
        *self.error.lock().unwrap() = Some(error);
        self.finish_inner(false)
    }
    fn finish_inner(mut self, success: bool) -> Result<PathBuf, String> {
        self.finishing.store(success, Ordering::Release);
        self.tx = None;
        if let Some(join) = self.join.take() {
            join.join()
                .map_err(|_| "VFL writer panicked")?
                .map_err(|e| e.to_string())?;
        }
        Ok(self.path.clone())
    }
}
impl Drop for VideoEncoder {
    fn drop(&mut self) {
        self.tx = None; /* crash recovery preserves the log; never wait on disk here */
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dropped_final_change_is_written_on_repeat_tick() {
        let encoder = VideoEncoder::start_result(2, 2, 30).unwrap();
        let old = Arc::new(vec![0; 12]);
        assert!(encoder.try_push_rgb_change(old, 2, 2, true));
        let deadline = Instant::now() + std::time::Duration::from_secs(5);
        while encoder.bytes_handle().load(Ordering::Acquire) == 0 {
            assert!(Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        // The first frame is written and the queue is empty. Reserve all four
        // slots to deterministically reject the final scene change.
        encoder.queued.store(4, Ordering::Relaxed);
        let last = Arc::new(vec![255; 12]);
        assert!(!encoder.try_push_rgb_change(last.clone(), 2, 2, true));
        encoder.queued.store(0, Ordering::Relaxed);
        // Recording::frame(None) retries its retained Arc with changed=false.
        assert!(encoder.try_push_rgb_change(last, 2, 2, false));
        let path = encoder.finish_result().unwrap();
        let data = std::fs::read(path.join("chunk-000000.bin")).unwrap();
        let mut at = 0;
        let mut pixels = vec![0; 16];
        while at < data.len() {
            let len = u32::from_le_bytes(data[at..at + 4].try_into().unwrap()) as usize;
            let payload = &data[at + 17..at + 17 + len];
            match data[at + 16] {
                media_vfl::KEY => pixels = lz4_flex::block::decompress(payload, 16).unwrap(),
                media_vfl::DELTA => {
                    for (p, d) in pixels.iter_mut().zip(lz4_flex::block::decompress(payload, 16).unwrap()) {
                        *p ^= d;
                    }
                }
                media_vfl::REPEAT => assert!(payload.is_empty()),
                kind => panic!("unexpected record kind {kind}"),
            }
            at += 17 + len;
        }
        assert_eq!(pixels, vec![255; 16]);
        let manifest: serde_json::Value = serde_json::from_slice(
            &std::fs::read(path.join("complete.json")).unwrap()).unwrap();
        assert_eq!(manifest["dropped"], 1);
        assert_eq!(manifest["frames"], 3);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn dropped_final_change_retried_as_repeat_seals_with_final_pixels() {
        let encoder = VideoEncoder::start_result(2, 2, 30).unwrap();
        // Hold admission full without racing the writer's receiver. No frame
        // is enqueued while these synthetic occupied slots are installed.
        encoder.queued.store(4, Ordering::Relaxed);
        let final_rgb = Arc::new(vec![10, 20, 30].repeat(4));
        assert!(!encoder.try_push_rgb_change(final_rgb.clone(), 2, 2, true));
        encoder.queued.store(0, Ordering::Relaxed);
        // Recording::frame(None) resubmits its immutable last readback with
        // changed=false. That flag is accounting, not a writer repeat opcode.
        assert!(encoder.try_push_rgb_change(final_rgb, 2, 2, false));
        let path = encoder.finish_result().unwrap();
        let manifest: serde_json::Value = serde_json::from_slice(
            &std::fs::read(path.join("complete.json")).unwrap()).unwrap();
        assert_eq!(manifest["dropped"], 1);
        let data = std::fs::read(path.join("chunk-000000.bin")).unwrap();
        let mut at = 0;
        let mut final_pixels = Vec::new();
        while at < data.len() {
            let len = u32::from_le_bytes(data[at..at+4].try_into().unwrap()) as usize;
            if data[at+16] == media_vfl::KEY {
                final_pixels = lz4_flex::block::decompress(&data[at+17..at+17+len], 16).unwrap();
            } else {
                assert_eq!(data[at+16], media_vfl::REPEAT);
            }
            at += 17 + len;
        }
        assert_eq!(final_pixels, vec![30, 20, 10, 255].repeat(4));
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn bounded_queue_reports_dropped_tail_and_never_waits() {
        let mut encoder = VideoEncoder::start_result(2, 2, 30).unwrap();
        // Replace the channel with a receiver held still for deterministic overload.
        let old_tx = encoder.tx.take();
        drop(old_tx);
        encoder.join.take().unwrap().join().unwrap().unwrap();
        let (tx, rx) = sync_channel(4);
        encoder.tx = Some(tx);
        let begin = std::time::Instant::now();
        for _ in 0..4 {
            assert!(encoder.try_push(vec![0; 16]));
        }
        for _ in 0..1000 {
            assert!(!encoder.try_push(vec![0; 16]));
        }
        assert_eq!(encoder.queue(), 4);
        assert_eq!(encoder.dropped.load(Ordering::Relaxed), 1000);
        for _ in 0..1000 {
            assert!(!encoder.try_push_rgb_change(Arc::new(vec![0; 12]), 2, 2, false));
        }
        assert_eq!(
            encoder.dropped.load(Ordering::Relaxed),
            1000,
            "repeat queue pressure is not lost changed content"
        );
        assert_eq!(encoder.queue_overruns.load(Ordering::Relaxed), 2000);
        assert!(begin.elapsed() < std::time::Duration::from_secs(1));
        assert!(
            !encoder.path.join("complete.json").exists(),
            "drop must leave an interrupted manifest"
        );
        drop(rx);
        std::fs::remove_dir_all(&encoder.path).unwrap();
    }
}
