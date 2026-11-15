// SPDX-License-Identifier: MIT OR Apache-2.0
//! Lossless frame log. See VFL.md for the on-disk contract.
use std::{fs::{File, OpenOptions}, io::{self, Write}, os::unix::fs::{OpenOptionsExt, DirBuilderExt}, path::{Path, PathBuf}, sync::{Arc, Mutex, atomic::{AtomicU64, AtomicUsize, Ordering}, mpsc::{sync_channel, SyncSender}}, thread::JoinHandle};
use serde_json::json;

const CHUNK_BYTES: u64 = 64 * 1024 * 1024;
const KEY: u8 = 0;
const DELTA: u8 = 1;
const REPEAT: u8 = 2;

fn create(path: &Path) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).mode(0o600)
        .custom_flags(libc::O_EXCL | libc::O_NOFOLLOW).open(path)
}

enum Pixels { Bgra(Arc<[u8]>), Rgb(Arc<Vec<u8>>, u32, u32), Marker(serde_json::Value) }
struct Frame { slot: u64, pixels: Pixels }

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
}
impl Log {
    fn record(&mut self, kind: u8, payload: &[u8]) -> io::Result<()> {
        if self.bytes > 0 && self.bytes + 17 + payload.len() as u64 > CHUNK_BYTES {
            self.chunk.sync_all()?;
            self.index += 1;
            self.chunk = create(&self.path.join(format!("chunk-{:06}.bin", self.index)))?;
            self.bytes = 0;
        }
        let mut crc = crc32fast::Hasher::new();
        crc.update(&self.next.to_le_bytes());
        crc.update(&[kind]);
        crc.update(payload);
        self.chunk.write_all(&(payload.len() as u32).to_le_bytes())?;
        self.chunk.write_all(&crc.finalize().to_le_bytes())?;
        self.chunk.write_all(&self.next.to_le_bytes())?;
        self.chunk.write_all(&[kind])?;
        self.chunk.write_all(payload)?;
        writeln!(self.frames, "{}", json!({"frame":self.next,"t_us":self.next*1_000_000/u64::from(self.fps)}))?;
        self.bytes += 17 + payload.len() as u64;
        self.next += 1;
        Ok(())
    }
    fn pixels(&mut self, pixels: Arc<[u8]>) -> io::Result<()> {
        if self.next == 0 || self.next.is_multiple_of(u64::from(self.fps) * 10) {
            self.record(KEY, &lz4_flex::block::compress(&pixels))?;
        } else if Arc::ptr_eq(&self.last, &pixels) || self.last == pixels {
            self.record(REPEAT, &[])?;
        } else {
            let delta: Vec<u8> = self.last.iter().zip(pixels.iter()).map(|(a,b)| a ^ b).collect();
            self.record(DELTA, &lz4_flex::block::compress(&delta))?;
        }
        self.last = pixels;
        Ok(())
    }
    fn repeat(&mut self) -> io::Result<()> { self.pixels(self.last.clone()) }
    fn frame(&mut self, frame: Frame) -> io::Result<()> {
        while self.next < frame.slot { self.repeat()?; }
        let pixels = match frame.pixels {
            Pixels::Marker(row) => { writeln!(self.markers, "{row}")?; return Ok(()); }
            Pixels::Bgra(p) => { self.previous_rgb = None; p }
            Pixels::Rgb(p, w, h) => {
                if self.previous_rgb.as_ref().is_some_and(|old| Arc::ptr_eq(old, &p)) {
                    self.last.clone()
                } else {
                    let mut out = vec![0; self.width as usize * self.height as usize * 4];
                    for (src, dst) in p.chunks_exact(w as usize * 3).take(h as usize).zip(out.chunks_exact_mut(self.width as usize * 4)) {
                        for (rgb, bgra) in src.chunks_exact(3).zip(dst.chunks_exact_mut(4)) { bgra.copy_from_slice(&[rgb[2],rgb[1],rgb[0],255]); }
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
    pub fn start_result(w: u32, h: u32, fps: u32) -> Result<Self, String> { Self::start_at(w,h,fps,None) }
    pub fn start_at(w: u32,h: u32,fps: u32,path: Option<PathBuf>) -> Result<Self,String> { Self::start_with_encoder(w,h,fps,path,"vfl") }
    pub fn start_with_encoder(width: u32,height: u32,fps: u32,path: Option<PathBuf>,encoder: &str) -> Result<Self,String> {
        if encoder != "vfl" { return Err("available encoder: vfl".into()); }
        let size = (width as usize).checked_mul(height as usize).and_then(|n| n.checked_mul(4)).filter(|n| *n > 0 && *n <= 256 * 1024 * 1024).ok_or("invalid frame dimensions")?;
        if !(1..=60).contains(&fps) { return Err("invalid fps".into()); }
        let path = path.unwrap_or_else(|| std::env::temp_dir().join(format!("viud-{}-{}.vfl",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos())));
        std::fs::DirBuilder::new().mode(0o700).create(&path).map_err(|e| e.to_string())?;
        let setup = || -> io::Result<Log> {
            let manifest = json!({"version":1,"width":width,"height":height,"fps":fps,"format":"bgra8","encoder":"vfl-lz4","complete":false,"key_interval":fps*10});
            let mut file = create(&path.join("manifest.json"))?;
            file.write_all(manifest.to_string().as_bytes())?; file.sync_all()?;
            let chunk = create(&path.join("chunk-000000.bin"))?;
            File::open(&path)?.sync_all()?;
            let frames = create(&path.join("frames.jsonl"))?;
            let markers = create(&path.join("markers.jsonl"))?;
            Ok(Log { path:path.clone(),chunk,index:0,bytes:0,last:vec![0;size].into(),previous_rgb:None,width,height,fps,next:0,frames,markers })
        };
        let mut log = setup().map_err(|e| e.to_string())?;
        let (tx,rx) = sync_channel::<Frame>(4);
        let scheduled = Arc::new(AtomicU64::new(0));
        let dropped = Arc::new(AtomicU64::new(0));
        let queued = Arc::new(AtomicUsize::new(0));
        let error = Arc::new(Mutex::new(None));
        let (s,d,q,e) = (scheduled.clone(),dropped.clone(),queued.clone(),error.clone());
        let join = std::thread::spawn(move || {
            let result = (|| -> io::Result<()> {
                for frame in rx { q.fetch_sub(1,Ordering::Relaxed); log.frame(frame)?; }
                while log.next < s.load(Ordering::Acquire) { log.repeat()?; }
                log.chunk.sync_all()?;
                log.frames.sync_all()?; log.markers.sync_all()?;
                let mut done = create(&log.path.join("complete.json"))?;
                done.write_all(json!({"version":1,"frames":log.next,"dropped":d.load(Ordering::Acquire),"chunks":log.index+1,"complete":true}).to_string().as_bytes())?;
                done.sync_all()?; File::open(&log.path)?.sync_all()
            })();
            if let Err(ref failure) = result { *e.lock().unwrap() = Some(failure.to_string()); }
            result
        });
        Ok(Self {tx:Some(tx),join:Some(join),path,width,height,scheduled,dropped,queued,error})
    }
    pub fn dims(&self) -> (u32,u32) { (self.width,self.height) }
    pub fn push(&self, p: Vec<u8>) { let _ = self.try_push(p); }
    pub fn try_push(&self,p:Vec<u8>) -> bool { self.try_push_shared(p.into()) }
    pub fn try_push_shared(&self,p:Arc<[u8]>) -> bool {
        if p.len() != self.width as usize * self.height as usize * 4 { return false; }
        self.submit(Pixels::Bgra(p))
    }
    pub fn try_push_rgb(&self,p:Arc<Vec<u8>>,w:u32,h:u32) -> bool {
        if w == 0 || h == 0 || w > self.width || h > self.height || p.len() != w as usize*h as usize*3 { return false; }
        self.submit(Pixels::Rgb(p,w,h))
    }
    fn submit(&self,pixels:Pixels) -> bool {
        let slot = self.scheduled.fetch_add(1,Ordering::AcqRel);
        self.queued.fetch_add(1,Ordering::Relaxed);
        if self.tx.as_ref().is_some_and(|tx| tx.try_send(Frame {slot,pixels}).is_ok()) { true } else {
            self.queued.fetch_sub(1,Ordering::Relaxed); self.dropped.fetch_add(1,Ordering::Relaxed); false
        }
    }
    pub fn queue(&self) -> usize { self.queued.load(Ordering::Relaxed) }
    pub fn marker(&self, row: serde_json::Value) -> Result<(),String> {
        self.queued.fetch_add(1,Ordering::Relaxed);
        if self.tx.as_ref().is_some_and(|tx| tx.try_send(Frame { slot:self.scheduled.load(Ordering::Acquire), pixels:Pixels::Marker(row) }).is_ok()) { Ok(()) } else {
            self.queued.fetch_sub(1,Ordering::Relaxed); Err("VFL marker queue full or closed".into())
        }
    }
    pub fn error(&self) -> Option<String> { self.error.lock().unwrap().clone() }
    pub fn finish_result(mut self) -> Result<PathBuf,String> {
        self.tx = None;
        if let Some(join) = self.join.take() { join.join().map_err(|_| "VFL writer panicked")?.map_err(|e| e.to_string())?; }
        Ok(self.path.clone())
    }
}
impl Drop for VideoEncoder { fn drop(&mut self) { self.tx = None; /* crash recovery preserves the log; never wait on disk here */ } }
