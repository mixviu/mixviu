// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Overlay-free recording core. The host owns target validation and scheduling.
use graphics::capture::encode::VideoEncoder;
use serde_json::{Value, json};
use std::{
    fs::File,
    io::{Read, Write},
    os::unix::fs::OpenOptionsExt,
    sync::Arc,
    sync::mpsc::{Receiver, channel},
};

pub fn private_token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|e| e.to_string())?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn secure_file(path: &str) -> Result<File, String> {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_EXCL | libc::O_NOFOLLOW)
        .open(path)
        .map_err(|e| format!("create {path}: {e}"))
}

pub fn check_destination(path: &str) -> Result<(), String> {
    for name in [path.to_owned()] {
        match std::fs::symlink_metadata(&name) {
            Ok(_) => return Err("recording destination already exists".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    probe_publication(path, rename_no_replace)
}

fn rename_no_replace(src: &str, dst: &str) -> std::io::Result<()> {
    let from = std::ffi::CString::new(src)?;
    let to = std::ffi::CString::new(dst)?;
    if unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            from.as_ptr(),
            libc::AT_FDCWD,
            to.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    } == 0
    {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

fn probe_publication(
    path: &str,
    rename: impl Fn(&str, &str) -> std::io::Result<()>,
) -> Result<(), String> {
    // Both the collision and successful rename must work in this directory.
    let base = format!("{path}.{}.part", private_token()?);
    let files = TakeFiles {
        path: path.into(),
        video: format!("{base}.probe-from"),
        frames: format!("{base}.probe-to"),
        markers: format!("{base}.probe-unused"),
    };
    secure_file(&files.video)?;
    secure_file(&files.frames)?;
    match rename(&files.video, &files.frames) {
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        result => {
            let _ = files.cleanup();
            return Err(format!(
                "destination does not support no-replace publication: {result:?}"
            ));
        }
    }
    std::fs::remove_file(&files.frames).map_err(|e| e.to_string())?;
    let result = rename(&files.video, &files.frames)
        .map_err(|e| format!("destination does not support no-replace publication: {e}"));
    let _ = files.cleanup();
    result
}

fn readback_skips(readback_us: u64, fps: u32) -> u64 {
    if fps == 0 {
        return 0;
    }
    let budget = (1_000_000 / u64::from(fps) / 4).max(1);
    readback_us.saturating_sub(budget).div_ceil(budget).min(60)
}

fn spend_readback_budget(debt: &mut u64, damaged: bool) -> bool {
    if *debt == 0 {
        return false;
    }
    *debt -= 1;
    damaged
}

#[derive(Clone)]
struct TakeFiles {
    path: String,
    video: String,
    frames: String,
    markers: String,
}
impl TakeFiles {
    fn create(path: &str) -> Result<Self, String> {
        let base = format!("{path}.{}.part", private_token()?);
        Ok(Self {
            path: path.into(),
            video: base.clone(),
            frames: format!("{base}.frames"),
            markers: format!("{base}.markers"),
        })
    }
    fn cleanup(&self) -> Result<(), String> {
        for path in [&self.video, &self.frames, &self.markers] {
            let result = if std::path::Path::new(path).is_dir() {
                std::fs::remove_dir_all(path)
            } else {
                std::fs::remove_file(path)
            };
            if let Err(error) = result {
                if error.kind() != std::io::ErrorKind::NotFound {
                    return Err(format!("remove private take: {error}"));
                }
            }
        }
        Ok(())
    }
    fn publish(&self) -> Result<(), String> {
        self.publish_tracked(&mut false)
    }
    fn publish_tracked(&self, published: &mut bool) -> Result<(), String> {
        self.publish_with(published, |parent| File::open(parent)?.sync_all())
    }
    fn publish_with(
        &self,
        published: &mut bool,
        sync_parent: impl FnOnce(&std::path::Path) -> std::io::Result<()>,
    ) -> Result<(), String> {
        rename_no_replace(&self.video, &self.path).map_err(|e| e.to_string())?;
        *published = true;
        let parent = std::path::Path::new(&self.path)
            .parent()
            .ok_or("take has no parent")?;
        sync_parent(parent)
            .map_err(|e| format!("take published at {}; parent sync failed: {e}", self.path))
    }
    fn retained_path(&self, published: bool) -> Option<String> {
        let path = if published { &self.path } else { &self.video };
        std::path::Path::new(path).is_dir().then(|| path.clone())
    }
}

impl Drop for TakeFiles {
    fn drop(&mut self) {
        // Preserve interrupted logs for recovery.
    }
}

pub fn monotonic_us() -> Result<u64, String> {
    read_clock(libc::CLOCK_MONOTONIC)
}

fn read_clock(clock: libc::clockid_t) -> Result<u64, String> {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // CLOCK_MONOTONIC is the compositor presentation clock on Linux.
    if unsafe { libc::clock_gettime(clock, &mut ts) } != 0 {
        return Err(format!(
            "monotonic clock: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(ts.tv_sec as u64 * 1_000_000 + ts.tv_nsec as u64 / 1000)
}

type FinaliseResult = (Result<(), String>, Option<String>);

pub struct Recording {
    pub session: String,
    pub fps: u32,
    pub t0_us: u64,
    pub frames: u64,
    pub repeated: u64,
    pub dropped: u64,
    dropped_capture: u64,
    readback_deferred: u64,
    pending_ticks: u64,
    writer_metrics: graphics::capture::encode::vfl::Metrics,
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
    files: TakeFiles,
    bytes: Arc<std::sync::atomic::AtomicU64>,
    encoder: Option<VideoEncoder>,
    finishing: Option<Receiver<FinaliseResult>>,
    stop_discard: Option<bool>,
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
        let encoder = VideoEncoder::start_with_encoder(
            width,
            height,
            fps,
            Some(files.video.clone().into()),
            &encoder_name,
        )?;
        let bytes = encoder.bytes_handle();
        let writer_metrics = encoder.metrics_handle();
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
            bytes,
            frames: 0,
            repeated: 0,
            dropped: 0,
            dropped_capture: 0,
            readback_deferred: 0,
            pending_ticks: 0,
            writer_metrics,
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
            encoder: Some(encoder),
            finishing: None,
            stop_discard: None,
            finaliser: None,
        })
    }

    pub fn started(&self) -> Value {
        json!({"session":self.session,"path":self.path,"recovery_path":self.files.video,"width":self.width,"height":self.height,"fps":self.fps,"clock":"monotonic","t0_us":self.t0_us,"frames_path":format!("{}/frames.jsonl",self.path),"markers_path":format!("{}/markers.jsonl",self.path)})
    }

    pub fn is_recording(&self) -> bool {
        self.encoder.is_some() && self.error.is_none()
    }

    pub fn frame(&mut self, pixels: Option<(u32, u32, Arc<Vec<u8>>, u64)>) -> Result<(), String> {
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
        encoder.set_summary(self.dropped, self.t0_us);
        if let Some(error) = encoder.error() {
            return Err(error);
        }
        if !encoder.try_push_rgb_change(
            self.last.clone(),
            self.source_width,
            self.source_height,
            changed,
        ) {
            if changed {
                self.dropped += 1;
            }
            if changed {
                self.repeated += 1;
            }
        }
        self.frames += 1;
        Ok(())
    }

    pub fn marker(&mut self, label: String, data: Value) -> Result<Value, String> {
        if self.encoder.is_none() || self.frames == 0 {
            return Err("session is not recording".into());
        }
        let row = json!({"seq":self.seq,"frame":self.frames-1,"t_us":monotonic_us()?-self.t0_us,"label":label,"data":data});
        self.encoder
            .as_ref()
            .ok_or("recording is finishing")?
            .marker(row.clone())?;
        self.seq += 1;
        Ok(json!({"session":self.session,"seq":row["seq"],"frame":row["frame"],"t_us":row["t_us"]}))
    }

    /// ExportMem maps synchronously. Repay over-budget readback time with
    /// skipped captures, preserving CFR through the previous immutable frame.
    pub fn skip_readback(&mut self, damaged: bool) -> bool {
        if !spend_readback_budget(&mut self.readback_debt, damaged) {
            return false;
        }
        self.readback_deferred += 1;
        true
    }
    pub fn superseded_capture(&mut self) {
        self.dropped += 1;
        self.dropped_capture += 1;
    }
    pub fn pending_tick(&mut self) {
        self.pending_ticks += 1;
    }

    pub fn status(&self) -> Value {
        json!({"session":self.session,"state":if self.error.is_some(){"failed"}else if self.terminal.is_some(){"idle"}else if self.finishing.is_some(){"finishing"}else{"recording"},"frames":self.frames,"repeated":self.repeated,"dropped":self.dropped,"dropped_capture":self.dropped_capture,"dropped_queue":self.dropped-self.dropped_capture,"readback_deferred":self.readback_deferred,"pending_ticks":self.pending_ticks,"writer":self.writer_metrics.json(),"duration_us":self.frames*1_000_000/u64::from(self.fps),"bytes":self.bytes.load(std::sync::atomic::Ordering::Acquire),"encoders":["vfl"],"queue":self.encoder.as_ref().map(VideoEncoder::queue).unwrap_or(0),"readback_us":self.readback_us,"readbacks":self.readbacks,"readback_mean_us":self.readback_total_us/self.readbacks.max(1),"readback_max_us":self.readback_max_us,"encoder":self.encoder_name,"error":self.error})
    }

    pub fn stop(&mut self, discard: bool) -> bool {
        let Some(encoder) = self.encoder.take() else {
            return false;
        };
        self.stop_discard = Some(discard);
        self.last = Arc::new(Vec::new());
        encoder.set_summary(self.dropped, self.t0_us);
        let files = self.files.clone();
        let failure = self.error.clone();
        let failed = failure.is_some();
        let (tx, rx) = channel();
        self.finishing = Some(rx);
        self.finaliser = Some(std::thread::spawn(move || {
            let mut published = false;
            let result = match failure {
                Some(error) => encoder.finish_failed(error),
                None => encoder.finish_result(),
            };
            let result = if discard {
                let cleanup = files.cleanup();
                result.map(|_| ()).and(cleanup)
            } else if failed {
                result.and(Err(
                    "recording failed; staged VFL retained for recovery".into()
                ))
            } else {
                result.and_then(|_| files.publish_tracked(&mut published))
            };
            let recovery = files.retained_path(published);
            let _ = tx.send((result, recovery));
        }));
        true
    }

    pub fn stop_conflicts(&self, discard: bool) -> bool {
        self.finishing.is_some()
            && self
                .stop_discard
                .is_some_and(|previous| previous != discard)
    }

    pub fn poll(&mut self) -> bool {
        let Some(rx) = &self.finishing else {
            return false;
        };
        let (result, recovery_path) = match rx.try_recv() {
            Ok(result) => result,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                (Err("encoder finaliser disconnected".into()), None)
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => return false,
        };
        if let Err(error) = result {
            self.error.get_or_insert(error);
        }
        self.finishing = None;
        if let Some(join) = self.finaliser.take() {
            let _ = join.join();
        }
        let kept = self.stop_discard == Some(false) && self.error.is_none();
        self.terminal = Some(
            json!({"recovery_path":if self.error.is_some() {recovery_path} else {None},"path":self.path,"bytes":self.bytes.load(std::sync::atomic::Ordering::Acquire),"frames":self.frames,"repeated":self.repeated,"dropped":self.dropped,"dropped_capture":self.dropped_capture,"dropped_queue":self.dropped-self.dropped_capture,"readback_deferred":self.readback_deferred,"pending_ticks":self.pending_ticks,"writer":self.writer_metrics.json(),"duration_us":self.frames*1_000_000/u64::from(self.fps),"frames_path":if kept {Some(format!("{}/frames.jsonl",self.path))}else{None},"markers_path":if kept {Some(format!("{}/markers.jsonl",self.path))}else{None},"readback_us":self.readback_us,"readbacks":self.readbacks,"readback_mean_us":self.readback_total_us/self.readbacks.max(1),"readback_max_us":self.readback_max_us,"encoder":self.encoder_name,"error":self.error}),
        );
        true
    }
}

impl Drop for Recording {
    fn drop(&mut self) {
        // Closing admission leaves an interrupted VFL recoverable on disk.
        drop(self.encoder.take());
        if let Some(join) = self.finaliser.take() {
            let _ = join.join();
        }
    }
}

#[cfg(test)]
mod file_tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};
    #[test]
    fn readback_deferral_is_not_a_lost_change() {
        let dir = std::env::temp_dir().join(private_token().unwrap());
        std::fs::create_dir(&dir).unwrap();
        let mut r = Recording::new(
            "debt".into(),
            dir.join("debt.vfl").to_str().unwrap().into(),
            2,
            2,
            30,
            "vfl".into(),
        )
        .unwrap();
        r.frame(Some((2, 2, Arc::new(vec![0; 12]), 14000))).unwrap();
        assert!(r.skip_readback(true));
        assert_eq!(r.dropped, 0);
        assert_eq!(r.readback_deferred, 1);
        r.frame(None).unwrap();
        assert!(!r.skip_readback(false));
        assert_eq!(r.dropped, 0);
        r.superseded_capture();
        assert_eq!(r.dropped, 1);
        assert_eq!(r.dropped_capture, 1);
        r.stop(true);
        while !r.poll() {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn failed_parent_sync_reports_published_directory() {
        let dir = std::env::temp_dir().join(private_token().unwrap());
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("sync.vfl").to_str().unwrap().to_owned();
        let files = TakeFiles::create(&path).unwrap();
        std::fs::create_dir(&files.video).unwrap();
        let mut published = false;
        assert!(
            files
                .publish_with(&mut published, |_| Err(std::io::Error::from_raw_os_error(
                    libc::EIO
                )))
                .is_err()
        );
        assert!(published);
        assert_eq!(files.retained_path(published), Some(path));
        assert!(!std::path::Path::new(&files.video).exists());
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn failed_discard_reports_no_recovery_directory() {
        let dir = std::env::temp_dir().join(private_token().unwrap());
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("discard.vfl").to_str().unwrap().to_owned();
        let mut recording = Recording::new("test".into(), path, 2, 2, 30, "vfl".into()).unwrap();
        recording
            .frame(Some((2, 2, Arc::new(vec![0; 12]), 0)))
            .unwrap();
        recording.error = Some("capture failed".into());
        assert!(recording.stop(true));
        while !recording.poll() {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(
            recording.terminal.as_ref().unwrap()["recovery_path"],
            Value::Null
        );
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn private_staging_no_clobber_and_cleanup() {
        let dir = std::env::temp_dir().join(private_token().unwrap());
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("take.vfl").to_str().unwrap().to_owned();
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
            let _ = files.cleanup();
        }
        assert!(check_destination(&path).is_err());
        std::fs::remove_file(&path).unwrap();
        symlink("missing", &path).unwrap();
        assert!(check_destination(&path).is_err());
        assert!(secure_file(&path).is_err());
        std::fs::remove_file(&path).unwrap();
        let files = TakeFiles::create(&path).unwrap();
        for name in [&files.video, &files.frames, &files.markers] {
            secure_file(name).unwrap().write_all(b"private").unwrap();
        }
        files.publish().unwrap();
        let _ = files.cleanup();
        drop(files);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "private");
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn readback_budget_bounds_amortised_cost() {
        assert_eq!(readback_skips(100, 0), 0);
        assert_eq!(readback_skips(8000, 30), 0);
        assert_eq!(readback_skips(14000, 30), 1);
        assert_eq!(readback_skips(25000, 30), 3);
        assert_eq!(readback_skips(1_000_000, 30), 60);
        let mut debt = readback_skips(14000, 30);
        assert!(!spend_readback_budget(&mut debt, false));
        assert!(
            !spend_readback_budget(&mut debt, true),
            "idle ticks already repaid the slow read"
        );
        let mut debt = 2;
        assert!(spend_readback_budget(&mut debt, true));
        assert!(spend_readback_budget(&mut debt, true));
        assert!(!spend_readback_budget(&mut debt, true));
    }
    #[test]
    fn unsupported_publication_refused_before_take_without_residue() {
        let dir = std::env::temp_dir().join(private_token().unwrap());
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("take.vfl").to_str().unwrap().to_owned();
        let error = probe_publication(&path, |_, _| {
            Err(std::io::Error::from_raw_os_error(libc::EINVAL))
        })
        .unwrap_err();
        assert!(error.contains("no-replace publication"));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        check_destination(&path).unwrap();
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn invalid_clock_reports_error_without_panic() {
        assert!(read_clock(-1).is_err());
        assert!(monotonic_us().unwrap() > 0);
    }
    #[test]
    fn finishing_preserves_original_discard_choice() {
        let dir = std::env::temp_dir().join(private_token().unwrap());
        std::fs::create_dir(&dir).unwrap();
        for discard in [false, true] {
            let path = dir
                .join(format!("{discard}.vfl"))
                .to_str()
                .unwrap()
                .to_owned();
            let mut recording =
                Recording::new("test".into(), path, 2, 2, 30, "vfl".into()).unwrap();
            recording
                .frame(Some((2, 2, Arc::new(vec![0; 12]), 0)))
                .unwrap();
            assert!(!recording.stop_conflicts(discard));
            assert!(recording.stop(discard));
            assert!(recording.stop_conflicts(!discard));
            assert!(!recording.stop_conflicts(discard));
            while !recording.poll() {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            assert!(recording.error.is_none(), "{:?}", recording.error);
            assert!(!recording.stop_conflicts(!discard));
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn failed_encoder_setup_removes_private_sidecars() {
        let dir = std::env::temp_dir().join(private_token().unwrap());
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("failed.vfl").to_str().unwrap().to_owned();
        assert!(Recording::new("failed".into(), path, 2, 2, 30, "invalid".into()).is_err());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        std::fs::remove_dir(dir).unwrap();
    }
}
