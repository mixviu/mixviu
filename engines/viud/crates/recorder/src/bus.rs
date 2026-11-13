// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Overlay-free recording core. The host owns target validation and scheduling.
use graphics::capture::encode::VideoEncoder;
pub use graphics::capture::encode::video::probe_vaapi;
use serde_json::{Value, json};
use std::{
    fs::File,
    io::{Write, Read, BufWriter},
    os::unix::fs::OpenOptionsExt,
    sync::Arc,
    sync::mpsc::{Receiver, channel},
};

pub fn private_token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut bytes)).map_err(|e| e.to_string())?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn secure_file(path: &str) -> Result<File, String> {
    std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600)
        .custom_flags(libc::O_NOFOLLOW).open(path).map_err(|e| format!("create {path}: {e}"))
}

pub fn check_destination(path: &str) -> Result<(), String> {
    for name in [path.to_owned(), format!("{path}.frames.jsonl"), format!("{path}.markers.jsonl")] {
        match std::fs::symlink_metadata(&name) {
            Ok(_) => return Err("recording destination already exists".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {},
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}

fn readback_skips(readback_us: u64, fps: u32) -> u64 {
    let budget = 1_000_000 / u64::from(fps) / 4;
    readback_us.saturating_sub(budget).div_ceil(budget).min(60)
}

#[derive(Clone)]
struct TakeFiles { path: String, video: String, frames: String, markers: String }
impl TakeFiles {
    fn create(path: &str) -> Result<Self, String> {
        check_destination(path)?;
        let base = format!("{path}.{}.part", private_token()?);
        Ok(Self { path: path.into(), video: base.clone(), frames: format!("{base}.frames"), markers: format!("{base}.markers") })
    }
    fn cleanup(&self) {
        for path in [&self.video, &self.frames, &self.markers] { let _ = std::fs::remove_file(path); }
    }
    fn publish(&self) -> Result<(), String> {
        let mut published = Vec::new();
        for (src, dst) in [(&self.frames, format!("{}.frames.jsonl", self.path)), (&self.markers, format!("{}.markers.jsonl", self.path)), (&self.video, self.path.clone())] {
            let from = std::ffi::CString::new(src.as_str()).map_err(|e| e.to_string())?;
            let to = std::ffi::CString::new(dst.as_str()).map_err(|e| e.to_string())?;
            if unsafe { libc::renameat2(libc::AT_FDCWD, from.as_ptr(), libc::AT_FDCWD, to.as_ptr(), libc::RENAME_NOREPLACE) } != 0 {
                let error = std::io::Error::last_os_error().to_string();
                for name in published { let _ = std::fs::remove_file(name); }
                self.cleanup();
                return Err(error);
            }
            published.push(dst);
        }
        Ok(())
    }
}

#[cfg(test)]
mod file_tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};
    #[test]
    fn private_staging_no_clobber_and_cleanup() {
        let dir = std::env::temp_dir().join(private_token().unwrap());
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("take.mkv").to_str().unwrap().to_owned();
        {
            let files = TakeFiles::create(&path).unwrap();
            for name in [&files.video, &files.frames, &files.markers] {
                let file = secure_file(name).unwrap();
                assert_eq!(file.metadata().unwrap().permissions().mode() & 0o777, 0o600);
                assert!(secure_file(name).is_err());
            }
            assert!(!std::path::Path::new(&path).exists());
            std::fs::write(&path, "keep").unwrap();
            assert!(files.publish().is_err());
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "keep");
            assert!(!std::path::Path::new(&format!("{path}.frames.jsonl")).exists());
        }
        assert!(check_destination(&path).is_err());
        std::fs::remove_file(&path).unwrap();
        symlink("missing", &path).unwrap();
        assert!(check_destination(&path).is_err());
        assert!(secure_file(&path).is_err());
        std::fs::remove_file(&path).unwrap();
        let files = TakeFiles::create(&path).unwrap();
        for name in [&files.video, &files.frames, &files.markers] { secure_file(name).unwrap().write_all(b"private").unwrap(); }
        files.publish().unwrap();
        drop(files);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "private");
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn readback_budget_bounds_amortised_cost() {
        assert_eq!(readback_skips(8000, 30), 0);
        assert_eq!(readback_skips(14000, 30), 1);
        assert_eq!(readback_skips(25000, 30), 3);
        assert_eq!(readback_skips(1_000_000, 30), 60);
    }
    #[test]
    fn invalid_clock_reports_error_without_panic() {
        assert!(read_clock(-1).is_err());
        assert!(monotonic_us().unwrap() > 0);
    }
    #[test]
    fn failed_encoder_setup_removes_private_sidecars() {
        let dir = std::env::temp_dir().join(private_token().unwrap());
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("failed.mkv").to_str().unwrap().to_owned();
        assert!(Recording::new("failed".into(), path, 2, 2, 30, "invalid".into()).is_err());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir(dir).unwrap();
    }
}
impl Drop for TakeFiles { fn drop(&mut self) { self.cleanup(); } }

pub fn monotonic_us() -> Result<u64, String> {
    read_clock(libc::CLOCK_MONOTONIC)
}

fn read_clock(clock: libc::clockid_t) -> Result<u64, String> {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // CLOCK_MONOTONIC is the compositor presentation clock on Linux.
    if unsafe { libc::clock_gettime(clock, &mut ts) } != 0 { return Err(format!("monotonic clock: {}", std::io::Error::last_os_error())); }
    Ok(ts.tv_sec as u64 * 1_000_000 + ts.tv_nsec as u64 / 1000)
}

pub struct Recording {
    pub session: String,
    pub fps: u32,
    pub t0_us: u64,
    pub frames: u64,
    pub repeated: u64,
    pub dropped: u64,
    pub error: Option<String>,
    pub terminal: Option<Value>,
    path: String,
    width: u32,
    height: u32,
    source_width: u32,
    source_height: u32,
    seq: u64,
    last: Arc<Vec<u8>>,
    encoder_name: String,
    readback_us: u64,
    readback_total_us: u64,
    readbacks: u64,
    readback_max_us: u64,
    readback_debt: u64,
    skipped_readback: bool,
    files: TakeFiles,
    bytes: u64,
    frames_file: BufWriter<File>,
    markers_file: File,
    encoder: Option<VideoEncoder>,
    finishing: Option<Receiver<Result<(), String>>>,
    finaliser: Option<std::thread::JoinHandle<()>>,
}

impl Recording {
    pub fn new(
        session: String,
        path: String,
        width: u32,
        height: u32,
        fps: u32,
        encoder_name: String,
    ) -> Result<Self, String> {
        let (source_width, source_height) = (width, height);
        let width = width.checked_add(1).ok_or("capture width overflow")? & !1;
        let height = height.checked_add(1).ok_or("capture height overflow")? & !1;
        let files = TakeFiles::create(&path)?;
        let frames_file = BufWriter::with_capacity(256 * 1024, secure_file(&files.frames)?);
        let markers_file = secure_file(&files.markers)?;
        let encoder = VideoEncoder::start_with_encoder(
            width,
            height,
            fps,
            Some(files.video.clone().into()),
            &encoder_name,
        )?;
        Ok(Self {
            session,
            path,
            width,
            height,
            source_width,
            source_height,
            fps,
            t0_us: monotonic_us()?,
            files,
            bytes: 0,
            frames: 0,
            repeated: 0,
            dropped: 0,
            error: None,
            terminal: None,
            seq: 0,
            last: Arc::new(Vec::new()),
            encoder_name,
            readback_us: 0,
            readback_total_us: 0,
            readbacks: 0,
            readback_max_us: 0,
            readback_debt: 0,
            skipped_readback: false,
            frames_file,
            markers_file,
            encoder: Some(encoder),
            finishing: None,
            finaliser: None,
        })
    }

    pub fn started(&self) -> Value {
        json!({"session":self.session,"path":self.path,"width":self.width,"height":self.height,"fps":self.fps,"clock":"monotonic","t0_us":self.t0_us,"frames_path":format!("{}.frames.jsonl",self.path),"markers_path":format!("{}.markers.jsonl",self.path)})
    }

    pub fn is_recording(&self) -> bool {
        self.encoder.is_some() && self.error.is_none()
    }

    pub fn frame(&mut self, pixels: Option<(u32, u32, Arc<Vec<u8>>, u64)>) -> Result<(), String> {
        let budget_drop = std::mem::take(&mut self.skipped_readback);
        let changed = pixels.is_some();
        if let Some((w, h, rgb, readback_us)) = pixels {
            self.readback_us = readback_us;
            self.readback_total_us += readback_us;
            self.readbacks += 1;
            self.readback_max_us = self.readback_max_us.max(readback_us);
            self.readback_debt = readback_skips(readback_us, self.fps);
            if (w, h) != (self.source_width, self.source_height) {
                return Err("target dimensions changed".into());
            }
            if rgb.len() != w as usize * h as usize * 3 {
                return Err("invalid RGB frame".into());
            }
            self.last = rgb;
        } else {
            self.readback_us = 0;
            self.repeated += 1;
        }
        let encoder = self.encoder.as_ref().ok_or("recording is finishing")?;
        if let Some(error) = encoder.error() {
            return Err(error);
        }
        if !encoder.try_push_rgb(self.last.clone(), self.source_width, self.source_height) {
            if !budget_drop { self.dropped += 1; }
            if changed {
                self.repeated += 1;
            }
        }
        let now = monotonic_us()?;
        let t_us = if self.frames == 0 {
            0
        } else {
            now - self.t0_us
        };
        writeln!(
            self.frames_file,
            "{}",
            json!({"frame":self.frames,"t_us":t_us})
        )
        .map_err(|e| e.to_string())?;
        self.frames += 1;
        Ok(())
    }

    pub fn marker(&mut self, label: String, data: Value) -> Result<Value, String> {
        if self.encoder.is_none() || self.frames == 0 {
            return Err("session is not recording".into());
        }
        self.frames_file.flush().map_err(|e| e.to_string())?;
        let row = json!({"seq":self.seq,"frame":self.frames-1,"t_us":monotonic_us()?-self.t0_us,"label":label,"data":data});
        writeln!(self.markers_file, "{row}").map_err(|e| e.to_string())?;
        self.markers_file.flush().map_err(|e| e.to_string())?;
        self.seq += 1;
        Ok(json!({"session":self.session,"seq":row["seq"],"frame":row["frame"],"t_us":row["t_us"]}))
    }

    /// ExportMem maps synchronously. Repay over-budget readback time with
    /// skipped captures, preserving CFR through the previous immutable frame.
    pub fn skip_readback(&mut self) -> bool {
        if self.readback_debt == 0 { return false; }
        self.readback_debt -= 1;
        self.dropped += 1;
        self.skipped_readback = true;
        true
    }

    pub fn status(&self) -> Value {
        json!({"state":if self.error.is_some(){"failed"}else if self.terminal.is_some(){"idle"}else if self.finishing.is_some(){"finishing"}else{"recording"},"frames":self.frames,"repeated":self.repeated,"dropped":self.dropped,"duration_us":self.frames*1_000_000/u64::from(self.fps),"bytes":self.bytes,"queue":self.encoder.as_ref().map(VideoEncoder::queue).unwrap_or(0),"readback_us":self.readback_us,"readbacks":self.readbacks,"readback_mean_us":self.readback_total_us/self.readbacks.max(1),"readback_max_us":self.readback_max_us,"encoder":self.encoder_name,"error":self.error})
    }

    pub fn stop(&mut self, discard: bool) -> bool {
        let Some(encoder) = self.encoder.take() else {
            return false;
        };
        self.last = Arc::new(Vec::new());
        if let Err(error) = self.frames_file.flush().and_then(|_| self.markers_file.flush()) { self.error = Some(error.to_string()); }
        let files = self.files.clone();
        let failed = self.error.is_some();
        let (tx, rx) = channel();
        self.finishing = Some(rx);
        self.finaliser = Some(std::thread::spawn(move || {
            let result = encoder.finish_result();
            let result = if discard || failed || result.is_err() {
                files.cleanup();
                result.map(|_| ())
            } else { files.publish() };
            let _ = tx.send(result);
        }));
        true
    }

    pub fn poll(&mut self) -> bool {
        let Some(rx) = &self.finishing else {
            return false;
        };
        let result = match rx.try_recv() {
            Ok(result) => result,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                Err("encoder finaliser disconnected".into())
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => return false,
        };
        if let Err(error) = result {
            self.error = Some(error);
        }
        self.finishing = None;
        if let Some(join) = self.finaliser.take() { let _ = join.join(); }
        self.bytes = std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);
        let kept = std::path::Path::new(&format!("{}.frames.jsonl", self.path)).exists() && self.error.is_none();
        self.terminal = Some(
            json!({"path":self.path,"frames":self.frames,"repeated":self.repeated,"dropped":self.dropped,"duration_us":self.frames*1_000_000/u64::from(self.fps),"frames_path":if kept {Some(format!("{}.frames.jsonl",self.path))}else{None},"markers_path":if kept {Some(format!("{}.markers.jsonl",self.path))}else{None},"readback_us":self.readback_us,"readbacks":self.readbacks,"readback_mean_us":self.readback_total_us/self.readbacks.max(1),"readback_max_us":self.readback_max_us,"encoder":self.encoder_name,"error":self.error}),
        );
        true
    }
}

impl Drop for Recording {
    fn drop(&mut self) {
        // VideoEncoder::drop kills/reaps its process group before returning.
        drop(self.encoder.take());
        if let Some(join) = self.finaliser.take() { let _ = join.join(); }
    }
}
