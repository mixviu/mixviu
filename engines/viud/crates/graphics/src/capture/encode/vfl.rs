// SPDX-License-Identifier: MIT OR Apache-2.0
//! Lossless frame log. See VFL.md for the on-disk contract.
use serde_json::json;
use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        mpsc::{SyncSender, sync_channel},
    },
    thread::JoinHandle,
};

use media::vfl::{CHUNK_BYTES, DELTA, KEY, MAX_FRAME_BYTES, REPEAT, VERSION};

fn create(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_EXCL | libc::O_NOFOLLOW)
        .open(path)
}

enum Pixels {
    Bgra(Arc<[u8]>),
    Rgb(Arc<Vec<u8>>, u32, u32),
    Marker(serde_json::Value),
}
struct Frame {
    slot: u64,
    pixels: Pixels,
}

pub struct VideoEncoder {
    tx: Option<SyncSender<Frame>>,
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
}

struct Log {
    path: PathBuf,
    chunk: File,
    index: u64,
    bytes: u64,
    last: Arc<[u8]>,
    previous_rgb: Option<Arc<Vec<u8>>>,
    width: u32,
    height: u32,
    fps: u32,
    next: u64,
    frames: File,
    markers: File,
    crc: crc32fast::Hasher,
    total_bytes: u64,
}
impl Log {
    fn record(&mut self, kind: u8, payload: &[u8]) -> io::Result<()> {
        if self.bytes > 0 && self.bytes + 17 + payload.len() as u64 > CHUNK_BYTES {
            self.seal()?;
            self.index += 1;
            self.chunk = create(&self.path.join(format!("chunk-{:06}.bin", self.index)))?;
            self.bytes = 0;
            self.crc = crc32fast::Hasher::new();
        }
        let mut crc = crc32fast::Hasher::new();
        crc.update(&self.next.to_le_bytes());
        crc.update(&[kind]);
        crc.update(payload);
        for bytes in [
            &(payload.len() as u32).to_le_bytes()[..],
            &crc.finalize().to_le_bytes(),
            &self.next.to_le_bytes(),
            &[kind],
            payload,
        ] {
            self.chunk.write_all(bytes)?;
            self.crc.update(bytes);
        }
        writeln!(
            self.frames,
            "{}",
            json!({"frame":self.next,"t_us":self.next*1_000_000/u64::from(self.fps)})
        )?;
        self.bytes += 17 + payload.len() as u64;
        self.total_bytes += 17 + payload.len() as u64;
        self.next += 1;
        Ok(())
    }
    fn pixels(&mut self, pixels: Arc<[u8]>) -> io::Result<()> {
        if self.next == 0 || self.next.is_multiple_of(u64::from(self.fps) * 10) {
            self.record(KEY, &lz4_flex::block::compress(&pixels))?;
        } else if Arc::ptr_eq(&self.last, &pixels) || self.last == pixels {
            self.record(REPEAT, &[])?;
        } else {
            let delta: Vec<u8> = self
                .last
                .iter()
                .zip(pixels.iter())
                .map(|(a, b)| a ^ b)
                .collect();
            self.record(DELTA, &lz4_flex::block::compress(&delta))?;
        }
        self.last = pixels;
        Ok(())
    }
    fn repeat(&mut self) -> io::Result<()> {
        self.pixels(self.last.clone())
    }
    fn seal(&mut self) -> io::Result<()> {
        self.chunk.sync_all()?;
        let mut receipt = create(&self.path.join(format!("chunk-{:06}.json", self.index)))?;
        receipt.write_all(
            json!({"bytes":self.bytes,"crc32":self.crc.clone().finalize(),"next_frame":self.next})
                .to_string()
                .as_bytes(),
        )?;
        receipt.sync_all()
    }
    fn frame(&mut self, frame: Frame) -> io::Result<()> {
        while self.next < frame.slot {
            self.repeat()?;
        }
        let pixels = match frame.pixels {
            Pixels::Marker(row) => {
                writeln!(self.markers, "{row}")?;
                return Ok(());
            }
            Pixels::Bgra(p) => {
                self.previous_rgb = None;
                p
            }
            Pixels::Rgb(p, w, h) => {
                if self
                    .previous_rgb
                    .as_ref()
                    .is_some_and(|old| Arc::ptr_eq(old, &p))
                {
                    self.last.clone()
                } else {
                    let mut out = vec![0; self.width as usize * self.height as usize * 4];
                    for (src, dst) in p
                        .chunks_exact(w as usize * 3)
                        .take(h as usize)
                        .zip(out.chunks_exact_mut(self.width as usize * 4))
                    {
                        for (rgb, bgra) in src.chunks_exact(3).zip(dst.chunks_exact_mut(4)) {
                            bgra.copy_from_slice(&[rgb[2], rgb[1], rgb[0], 255]);
                        }
                    }
                    self.previous_rgb = Some(p);
                    out.into()
                }
            }
        };
        self.pixels(pixels)
    }
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
        if encoder != "vfl" {
            return Err("available encoder: vfl".into());
        }
        let size = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .filter(|n| *n > 0 && *n <= MAX_FRAME_BYTES)
            .ok_or("invalid frame dimensions")?;
        if !(1..=60).contains(&fps) {
            return Err("invalid fps".into());
        }
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
        let setup = move || -> io::Result<Log> {
            let path = worker_path;
            std::fs::DirBuilder::new().mode(0o700).create(&path)?;
            let manifest = json!({"version":VERSION,"width":width,"height":height,"fps":fps,"format":"bgra8","encoder":"vfl-lz4","complete":false,"key_interval":fps*10});
            let mut file = create(&path.join("manifest.json"))?;
            file.write_all(manifest.to_string().as_bytes())?;
            file.sync_all()?;
            let chunk = create(&path.join("chunk-000000.bin"))?;
            File::open(&path)?.sync_all()?;
            let frames = create(&path.join("frames.jsonl"))?;
            let markers = create(&path.join("markers.jsonl"))?;
            Ok(Log {
                path: path.clone(),
                chunk,
                index: 0,
                bytes: 0,
                last: vec![0; size].into(),
                previous_rgb: None,
                width,
                height,
                fps,
                next: 0,
                frames,
                markers,
                crc: crc32fast::Hasher::new(),
                total_bytes: 0,
            })
        };
        // Four frame slots plus four reserved control slots. Pixel memory
        // remains bounded even while compression is busy with a key frame.
        let (tx, rx) = sync_channel::<Frame>(8);
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
        let (s, d, q, e) = (
            scheduled.clone(),
            dropped.clone(),
            queued.clone(),
            error.clone(),
        );
        let join = std::thread::spawn(move || {
            let result = (|| -> io::Result<()> {
                let mut log = setup()?;
                let mut origin_written = false;
                for frame in rx {
                    q.fetch_sub(1, Ordering::Relaxed);
                    if !origin_written {
                        let mut origin = create(&log.path.join("origin.json"))?;
                        origin.write_all(
                            json!({"t0_us":o.load(Ordering::Acquire),"clock":"monotonic"})
                                .to_string()
                                .as_bytes(),
                        )?;
                        origin.sync_all()?;
                        origin_written = true;
                    }
                    log.frame(frame)?;
                    bytes.store(log.total_bytes, Ordering::Relaxed);
                }
                while log.next < s.load(Ordering::Acquire) {
                    log.repeat()?;
                }
                log.seal()?;
                log.frames.sync_all()?;
                log.markers.sync_all()?;
                if !finish.load(Ordering::Acquire) {
                    return Ok(());
                }
                let mut done = create(&log.path.join("complete.json"))?;
                done.write_all(json!({"version":1,"t0_us":o.load(Ordering::Acquire),"frames":log.next,"dropped":td.load(Ordering::Acquire).max(d.load(Ordering::Acquire)),"chunks":log.index+1,"complete":true}).to_string().as_bytes())?;
                done.sync_all()?;
                File::open(&log.path)?.sync_all()?;
                let mut total = 0;
                for file in std::fs::read_dir(&log.path)? {
                    total += file?.metadata()?.len();
                }
                bytes.store(total, Ordering::Release);
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
        self.submit(Pixels::Bgra(p))
    }
    pub fn try_push_rgb(&self, p: Arc<Vec<u8>>, w: u32, h: u32) -> bool {
        if w == 0
            || h == 0
            || w > self.width
            || h > self.height
            || p.len() != w as usize * h as usize * 3
        {
            return false;
        }
        self.submit(Pixels::Rgb(p, w, h))
    }
    fn submit(&self, pixels: Pixels) -> bool {
        let slot = self.scheduled.fetch_add(1, Ordering::AcqRel);
        let occupied = self.queued.fetch_add(1, Ordering::Relaxed);
        if occupied >= 4 {
            self.queued.fetch_sub(1, Ordering::Relaxed);
            self.dropped.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        if self
            .tx
            .as_ref()
            .is_some_and(|tx| tx.try_send(Frame { slot, pixels }).is_ok())
        {
            true
        } else {
            self.queued.fetch_sub(1, Ordering::Relaxed);
            self.dropped.fetch_add(1, Ordering::Relaxed);
            false
        }
    }
    pub fn queue(&self) -> usize {
        self.queued.load(Ordering::Relaxed)
    }
    pub fn bytes_handle(&self) -> Arc<AtomicU64> {
        self.written.clone()
    }
    pub fn set_summary(&self, dropped: u64, t0_us: u64) {
        self.total_dropped.store(dropped, Ordering::Release);
        self.origin.store(t0_us, Ordering::Release);
    }
    pub fn marker(&self, row: serde_json::Value) -> Result<(), String> {
        self.queued.fetch_add(1, Ordering::Relaxed);
        if self.tx.as_ref().is_some_and(|tx| {
            tx.try_send(Frame {
                slot: self.scheduled.load(Ordering::Acquire),
                pixels: Pixels::Marker(row),
            })
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
    pub fn finish_result(mut self) -> Result<PathBuf, String> {
        self.finishing.store(true, Ordering::Release);
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
    fn key_delta_repeat_crc_and_no_clobber() {
        let encoder = VideoEncoder::start_result(4, 4, 30).unwrap();
        let path = encoder.path.clone();
        let first: Arc<[u8]> = vec![1; 64].into();
        let mut changed = vec![1; 64];
        changed[3] = 2;
        assert!(encoder.try_push_shared(first.clone()));
        assert!(encoder.try_push_shared(first));
        assert!(encoder.try_push(changed.clone()));
        let path = encoder.finish_result().unwrap();
        assert!(create(&path.join("manifest.json")).is_err());
        let bytes = std::fs::read(path.join("chunk-000000.bin")).unwrap();
        let mut at = 0;
        let mut previous = vec![0; 64];
        for (slot, kind) in [KEY, REPEAT, DELTA].into_iter().enumerate() {
            let n = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
            let crc = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap());
            assert_eq!(crc, crc32fast::hash(&bytes[at + 8..at + 17 + n]));
            assert_eq!(bytes[at + 16], kind);
            assert_eq!(
                u64::from_le_bytes(bytes[at + 8..at + 16].try_into().unwrap()),
                slot as u64
            );
            let payload = &bytes[at + 17..at + 17 + n];
            match kind {
                KEY => previous = lz4_flex::block::decompress(payload, 64).unwrap(),
                DELTA => {
                    let delta = lz4_flex::block::decompress(payload, 64).unwrap();
                    for (p, d) in previous.iter_mut().zip(delta) {
                        *p ^= d;
                    }
                }
                _ => assert!(payload.is_empty()),
            }
            assert_eq!(
                previous,
                if slot == 2 {
                    changed.clone()
                } else {
                    vec![1; 64]
                }
            );
            at += 17 + n;
        }
        assert_eq!(at, bytes.len());
        let receipt: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path.join("chunk-000000.json")).unwrap())
                .unwrap();
        assert_eq!(receipt["crc32"], crc32fast::hash(&bytes));
        // A torn tail has no complete payload and a changed byte fails CRC.
        let mut corrupt = bytes.clone();
        corrupt[17] ^= 1;
        let n = u32::from_le_bytes(corrupt[..4].try_into().unwrap()) as usize;
        assert_ne!(
            crc32fast::hash(&corrupt[8..17 + n]),
            u32::from_le_bytes(corrupt[4..8].try_into().unwrap())
        );
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
        assert!(begin.elapsed() < std::time::Duration::from_secs(1));
        assert!(
            !encoder.path.join("complete.json").exists(),
            "drop must leave an interrupted manifest"
        );
        drop(rx);
        std::fs::remove_dir_all(&encoder.path).unwrap();
    }
}
