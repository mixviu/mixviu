// SPDX-License-Identifier: MIT OR Apache-2.0
//! VFL container and lossless writer, shared by viud and the future reader.
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};
pub const VERSION: u32 = 2;
pub const LEGACY_VERSION: u32 = 1;
pub const RECORD_HEADER_BYTES: usize = 17;
pub const KEY: u8 = 0;
pub const DELTA: u8 = 1;
pub const REPEAT: u8 = 2;
pub const RECT_DELTA: u8 = 3;
pub const MAX_FRAME_BYTES: usize = 256 * 1024 * 1024;
pub const CHUNK_BYTES: u64 = 64 * 1024 * 1024;
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub rect_records: u64,
    pub rect_bytes: u64,
    pub delta_max_bytes: u64,
    pub setup_us: u64,
    pub sync_us: u64,
    pub sync_max_us: u64,
    pub encode_us: u64,
    pub encode_max_us: u64,
    pub convert_us: u64,
    pub write_us: u64,
    pub write_max_us: u64,
    pub keys: u64,
    pub deltas: u64,
    pub repeats: u64,
    pub key_bytes: u64,
    pub delta_bytes: u64,
}
impl Stats {
    pub fn json(self) -> Value {
        json!({"rectangle_records":self.rect_records,"rectangle_bytes":self.rect_bytes,"delta_mean_bytes":self.delta_bytes/self.deltas.max(1),"delta_max_bytes":self.delta_max_bytes,"setup_us":self.setup_us,"sync_us":self.sync_us,"sync_max_us":self.sync_max_us,"encode_mean_us":self.encode_us/(self.keys+self.deltas+self.repeats).max(1),"write_mean_us":self.write_us/(self.keys+self.deltas+self.repeats).max(1),"encode_us":self.encode_us,"encode_max_us":self.encode_max_us,"convert_us":self.convert_us,"write_us":self.write_us,"write_max_us":self.write_max_us,"key_records":self.keys,"delta_records":self.deltas,"repeat_records":self.repeats,"key_bytes":self.key_bytes,"delta_bytes":self.delta_bytes})
    }
}
pub fn frame_bytes(width: u32, height: u32, fps: u32) -> io::Result<usize> {
    if !(1..=60).contains(&fps) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid fps"));
    }
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .filter(|n| *n > 0 && *n <= MAX_FRAME_BYTES)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid dimensions"))
}
/// Derive a changed rectangle from XOR pixels, never from trusted damage.
pub fn compress_delta(delta: &[u8], width: u32, height: u32) -> io::Result<(u8, Vec<u8>)> {
    if delta.len() != frame_bytes(width, height, 30)? {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid delta size",
        ));
    }
    let Some(first) = delta.iter().position(|b| *b != 0) else {
        return Ok((REPEAT, Vec::new()));
    };
    let last = delta.iter().rposition(|b| *b != 0).unwrap();
    let stride = width as usize * 4;
    let (top, bottom) = (first / stride, last / stride + 1);
    let (mut left, mut right) = (width as usize, 0usize);
    for row in delta[top * stride..bottom * stride].chunks_exact(stride) {
        if let Some(x) = row.iter().position(|b| *b != 0) {
            left = left.min(x / 4);
            right = right.max(row.iter().rposition(|b| *b != 0).unwrap() / 4 + 1);
        }
    }
    let (w, h) = (right - left, bottom - top);
    if w * h >= width as usize * height as usize * 3 / 4 {
        return Ok((DELTA, lz4_flex::block::compress(delta)));
    }
    let mut rectangle = Vec::with_capacity(w * h * 4);
    for y in top..bottom {
        rectangle.extend_from_slice(&delta[y * stride + left * 4..y * stride + right * 4]);
    }
    let mut payload = Vec::new();
    for coordinate in [left, top, w, h] {
        payload.extend_from_slice(&(coordinate as u32).to_le_bytes());
    }
    payload.extend_from_slice(&lz4_flex::block::compress(&rectangle));
    Ok((RECT_DELTA, payload))
}
fn create(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_EXCL | libc::O_NOFOLLOW)
        .open(path)
}

pub enum Pixels {
    Bgra(Arc<[u8]>),
    Rgb(Arc<Vec<u8>>, u32, u32),
    Marker(serde_json::Value),
}
pub struct Frame {
    pub slot: u64,
    pub pixels: Pixels,
}

pub struct Writer {
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
    pub total_bytes: u64,
    pub stats: Stats,
    origin_written: bool,
}
impl Writer {
    fn record(&mut self, kind: u8, payload: &[u8]) -> io::Result<()> {
        let write_started = Instant::now();
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
        let elapsed = write_started.elapsed().as_micros() as u64;
        self.stats.write_us += elapsed;
        self.stats.write_max_us = self.stats.write_max_us.max(elapsed);
        match kind {
            KEY => {
                self.stats.keys += 1;
                self.stats.key_bytes += payload.len() as u64;
            }
            DELTA | RECT_DELTA => {
                self.stats.deltas += 1;
                self.stats.delta_bytes += payload.len() as u64;
                self.stats.delta_max_bytes = self.stats.delta_max_bytes.max(payload.len() as u64);
                if kind == RECT_DELTA {
                    self.stats.rect_records += 1;
                    self.stats.rect_bytes += payload.len() as u64;
                }
            }
            _ => self.stats.repeats += 1,
        }
        Ok(())
    }
    fn pixels(&mut self, pixels: Arc<[u8]>) -> io::Result<()> {
        let started = Instant::now();
        let (kind, payload) =
            if self.next == 0 || self.next.is_multiple_of(u64::from(self.fps) * 10) {
                (KEY, lz4_flex::block::compress(&pixels))
            } else if Arc::ptr_eq(&self.last, &pixels) || self.last == pixels {
                (REPEAT, Vec::new())
            } else {
                let delta: Vec<u8> = self
                    .last
                    .iter()
                    .zip(pixels.iter())
                    .map(|(a, b)| a ^ b)
                    .collect();
                compress_delta(&delta, self.width, self.height)?
            };
        let elapsed = started.elapsed().as_micros() as u64;
        self.stats.encode_us += elapsed;
        self.stats.encode_max_us = self.stats.encode_max_us.max(elapsed);
        self.record(kind, &payload)?;
        self.last = pixels;
        Ok(())
    }
    fn repeat(&mut self) -> io::Result<()> {
        self.pixels(self.last.clone())
    }
    fn seal(&mut self) -> io::Result<()> {
        let started = Instant::now();
        self.chunk.sync_all()?;
        let mut receipt = create(&self.path.join(format!("chunk-{:06}.json", self.index)))?;
        receipt.write_all(
            json!({"bytes":self.bytes,"crc32":self.crc.clone().finalize(),"next_frame":self.next})
                .to_string()
                .as_bytes(),
        )?;
        receipt.sync_all()?;
        File::open(&self.path)?.sync_all()?;
        let elapsed = started.elapsed().as_micros() as u64;
        self.stats.sync_us += elapsed;
        self.stats.sync_max_us = self.stats.sync_max_us.max(elapsed);
        Ok(())
    }
    pub fn frame(&mut self, frame: Frame) -> io::Result<()> {
        if frame.slot < self.next {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "frame slots out of order",
            ));
        }
        while self.next < frame.slot {
            self.repeat()?;
        }
        let pixels = match frame.pixels {
            Pixels::Marker(row) => {
                writeln!(self.markers, "{row}")?;
                return Ok(());
            }
            Pixels::Bgra(p) => {
                if p.len() != self.width as usize * self.height as usize * 4 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid BGRA frame",
                    ));
                }
                self.previous_rgb = None;
                p
            }
            Pixels::Rgb(p, w, h) => {
                if w == 0
                    || h == 0
                    || w > self.width
                    || h > self.height
                    || p.len() != w as usize * h as usize * 3
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "invalid RGB frame",
                    ));
                }
                if self
                    .previous_rgb
                    .as_ref()
                    .is_some_and(|old| Arc::ptr_eq(old, &p))
                {
                    self.last.clone()
                } else {
                    let started = Instant::now();
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
                    self.stats.convert_us += started.elapsed().as_micros() as u64;
                    out.into()
                }
            }
        };
        self.pixels(pixels)
    }
}

impl Writer {
    pub fn new(path: PathBuf, width: u32, height: u32, fps: u32) -> io::Result<Self> {
        let size = frame_bytes(width, height, fps)?;
        let setup_started = Instant::now();
        std::fs::DirBuilder::new().mode(0o700).create(&path)?;
        let manifest = json!({"version":VERSION,"width":width,"height":height,"fps":fps,"format":"bgra8","encoder":"vfl-lz4","complete":false,"key_interval":fps*10});
        let mut file = create(&path.join("manifest.json"))?;
        file.write_all(manifest.to_string().as_bytes())?;
        file.sync_all()?;
        let chunk = create(&path.join("chunk-000000.bin"))?;
        let frames = create(&path.join("frames.jsonl"))?;
        let markers = create(&path.join("markers.jsonl"))?;
        File::open(&path)?.sync_all()?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        File::open(parent)?.sync_all()?;
        Ok(Self {
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
            stats: Stats {
                setup_us: setup_started.elapsed().as_micros() as u64,
                ..Stats::default()
            },
            origin_written: false,
        })
    }
    pub fn origin(&mut self, t0_us: u64) -> io::Result<()> {
        let mut file = create(&self.path.join("origin.json"))?;
        file.write_all(
            json!({"t0_us":t0_us,"clock":"monotonic"})
                .to_string()
                .as_bytes(),
        )?;
        file.sync_all()?;
        File::open(&self.path)?.sync_all()?;
        self.origin_written = true;
        Ok(())
    }
    pub fn finish(
        &mut self,
        scheduled: u64,
        dropped: u64,
        t0_us: u64,
        error: Option<String>,
        success: bool,
    ) -> io::Result<u64> {
        let success = success && error.is_none();
        if success && !self.origin_written {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "missing capture origin",
            ));
        }
        while self.next < scheduled {
            self.repeat()?;
        }
        self.seal()?;
        self.frames.sync_all()?;
        self.markers.sync_all()?;
        let name = if success {
            "complete.json"
        } else {
            "interrupted.json"
        };
        let mut file = create(&self.path.join(name))?;
        file.write_all(json!({"version":VERSION,"complete":success,"t0_us":t0_us,"frames":self.next,"chunks":self.index+1,"dropped":dropped,"error":error,"writer":self.stats.json()}).to_string().as_bytes())?;
        file.sync_all()?;
        File::open(&self.path)?.sync_all()?;
        let mut bytes = 0;
        for entry in std::fs::read_dir(&self.path)? {
            bytes += entry?.metadata()?.len();
        }
        Ok(bytes)
    }
}
