// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Overlay-free recording core. The host owns target validation and scheduling.
use graphics::capture::encode::VideoEncoder;
pub use graphics::capture::encode::video::probe_vaapi;
use serde_json::{Value, json};
use std::{
    fs::File,
    io::Write,
    sync::Arc,
    sync::mpsc::{Receiver, channel},
};

pub fn monotonic_us() -> u64 {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // CLOCK_MONOTONIC is the compositor presentation clock on Linux.
    assert_eq!(
        unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) },
        0
    );
    ts.tv_sec as u64 * 1_000_000 + ts.tv_nsec as u64 / 1000
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
    last: Arc<[u8]>,
    encoder_name: String,
    readback_us: u64,
    readback_total_us: u64,
    readbacks: u64,
    readback_max_us: u64,
    frames_file: File,
    markers_file: File,
    encoder: Option<VideoEncoder>,
    finishing: Option<Receiver<Result<(), String>>>,
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
        let encoder = VideoEncoder::start_with_encoder(
            width,
            height,
            fps,
            Some(path.clone().into()),
            &encoder_name,
        )?;
        let frames_file =
            File::create(format!("{path}.frames.jsonl")).map_err(|e| e.to_string())?;
        let markers_file =
            File::create(format!("{path}.markers.jsonl")).map_err(|e| e.to_string())?;
        Ok(Self {
            session,
            path,
            width,
            height,
            source_width,
            source_height,
            fps,
            t0_us: monotonic_us(),
            frames: 0,
            repeated: 0,
            dropped: 0,
            error: None,
            terminal: None,
            seq: 0,
            last: Arc::from([]),
            encoder_name,
            readback_us: 0,
            readback_total_us: 0,
            readbacks: 0,
            readback_max_us: 0,
            frames_file,
            markers_file,
            encoder: Some(encoder),
            finishing: None,
        })
    }

    pub fn started(&self) -> Value {
        json!({"session":self.session,"path":self.path,"width":self.width,"height":self.height,"fps":self.fps,"clock":"monotonic","t0_us":self.t0_us,"frames_path":format!("{}.frames.jsonl",self.path),"markers_path":format!("{}.markers.jsonl",self.path)})
    }

    pub fn is_recording(&self) -> bool {
        self.encoder.is_some() && self.error.is_none()
    }

    pub fn frame(&mut self, pixels: Option<(u32, u32, Vec<u8>, u64)>) -> Result<(), String> {
        let changed = pixels.is_some();
        if let Some((w, h, rgb, readback_us)) = pixels {
            self.readback_us = readback_us;
            self.readback_total_us += readback_us;
            self.readbacks += 1;
            self.readback_max_us = self.readback_max_us.max(readback_us);
            if (w, h) != (self.source_width, self.source_height) {
                return Err("target dimensions changed".into());
            }
            if rgb.len() != w as usize * h as usize * 3 {
                return Err("invalid RGB frame".into());
            }
            self.last = rgb.into();
        } else {
            self.readback_us = 0;
            self.repeated += 1;
        }
        let encoder = self.encoder.as_ref().ok_or("recording is finishing")?;
        if let Some(error) = encoder.error() {
            return Err(error);
        }
        if !encoder.try_push_rgb(self.last.clone(), self.source_width, self.source_height) {
            self.dropped += 1;
            if changed {
                self.repeated += 1;
            }
        }
        let now = monotonic_us();
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
        let row = json!({"seq":self.seq,"frame":self.frames-1,"t_us":monotonic_us()-self.t0_us,"label":label,"data":data});
        writeln!(self.markers_file, "{row}").map_err(|e| e.to_string())?;
        self.markers_file.flush().map_err(|e| e.to_string())?;
        self.seq += 1;
        Ok(json!({"session":self.session,"seq":row["seq"],"frame":row["frame"],"t_us":row["t_us"]}))
    }

    pub fn status(&self) -> Value {
        json!({"state":if self.error.is_some(){"failed"}else if self.terminal.is_some(){"idle"}else if self.finishing.is_some(){"finishing"}else{"recording"},"frames":self.frames,"repeated":self.repeated,"dropped":self.dropped,"duration_us":self.frames*1_000_000/u64::from(self.fps),"bytes":std::fs::metadata(&self.path).map(|m|m.len()).unwrap_or(0),"queue":self.encoder.as_ref().map(VideoEncoder::queue).unwrap_or(0),"readback_us":self.readback_us,"readbacks":self.readbacks,"readback_mean_us":self.readback_total_us/self.readbacks.max(1),"readback_max_us":self.readback_max_us,"encoder":self.encoder_name,"error":self.error})
    }

    pub fn stop(&mut self, discard: bool) -> bool {
        let Some(encoder) = self.encoder.take() else {
            return false;
        };
        self.last = Arc::from([]);
        let _ = self.frames_file.flush();
        let _ = self.markers_file.flush();
        let path = self.path.clone();
        let (tx, rx) = channel();
        self.finishing = Some(rx);
        std::thread::spawn(move || {
            let result = encoder.finish_result().and_then(|temp| {
                if discard {
                    std::fs::remove_file(temp)
                        .and_then(|_| std::fs::remove_file(format!("{path}.frames.jsonl")))
                        .and_then(|_| std::fs::remove_file(format!("{path}.markers.jsonl")))
                        .map_err(|e| e.to_string())
                } else {
                    std::fs::rename(temp, path).map_err(|e| e.to_string())
                }
            });
            let _ = tx.send(result);
        });
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
        self.terminal = Some(
            json!({"path":self.path,"frames":self.frames,"repeated":self.repeated,"dropped":self.dropped,"duration_us":self.frames*1_000_000/u64::from(self.fps),"frames_path":format!("{}.frames.jsonl",self.path),"markers_path":format!("{}.markers.jsonl",self.path),"readback_us":self.readback_us,"readbacks":self.readbacks,"readback_mean_us":self.readback_total_us/self.readbacks.max(1),"readback_max_us":self.readback_max_us,"encoder":self.encoder_name,"error":self.error}),
        );
        true
    }
}
