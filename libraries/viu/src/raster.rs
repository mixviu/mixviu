// SPDX-License-Identifier: MIT OR Apache-2.0
//! In-process retained raster ownership. Mix chooses sources, transactions,
//! schedules and handle lifetimes. Loading runs on the caller's Jobs lane.
use crate::raster_decode::{self, Decoded};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

pub const RESIDENT_BYTES: u64 = 512 * 1024 * 1024;
pub struct TextureLease {
    bytes: u64,
    used: Arc<AtomicU64>,
}
impl Drop for TextureLease {
    fn drop(&mut self) {
        self.used.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

/// A live renderer reference still consumes the budget after handle release.
pub struct Pixels {
    pub decoded: Decoded,
    used: Arc<AtomicU64>,
}
impl Drop for Pixels {
    fn drop(&mut self) {
        self.used.fetch_sub(self.decoded.bytes(), Ordering::AcqRel);
    }
}
#[derive(Default)]
struct State {
    serial: u64,
    entries: BTreeMap<String, Arc<Pixels>>,
}
#[derive(Clone, Default)]
pub struct Rasters {
    state: Arc<Mutex<State>>,
    used: Arc<AtomicU64>,
    textures: Arc<AtomicU64>,
    // Serialise decode transients across root and child evaluators.
    lane: Arc<Mutex<()>>,
}
impl Rasters {
    pub fn texture(&self, bytes: u64) -> Result<TextureLease, String> {
        self.textures
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                used.checked_add(bytes)
                    .filter(|next| *next <= RESIDENT_BYTES)
            })
            .map_err(|_| "raster texture budget exceeded")?;
        Ok(TextureLease {
            bytes,
            used: self.textures.clone(),
        })
    }
    pub fn get(&self, handle: &str) -> Option<Arc<Pixels>> {
        self.state.lock().ok()?.entries.get(handle).cloned()
    }
    pub fn bytes(&self) -> u64 {
        self.used.load(Ordering::Acquire)
    }
    pub fn release(&self, handle: &str) {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entries
            .remove(handle);
    }
    fn insert(&self, decoded: Decoded) -> Result<Value, String> {
        let mut state = self.state.lock().map_err(|_| "raster owner poisoned")?;
        if state.entries.len() >= 64 {
            return Err("raster handle capacity exceeded".into());
        }
        let bytes = decoded.bytes();
        self.used
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                used.checked_add(bytes)
                    .filter(|next| *next <= RESIDENT_BYTES)
            })
            .map_err(|_| "resident raster budget exceeded")?;
        state.serial += 1;
        let handle = format!("raster:{}", state.serial);
        let result = json!({"handle":handle,"size":decoded.size,"original_size":decoded.original_size,"bytes":bytes});
        state.entries.insert(
            handle,
            Arc::new(Pixels {
                decoded,
                used: self.used.clone(),
            }),
        );
        Ok(result)
    }
    pub fn load(&self, path: &Path) -> Result<Value, String> {
        if !path.is_absolute() {
            return Err("raster path must be absolute".into());
        }
        let _lane = self
            .lane
            .lock()
            .map_err(|_| "raster decode lane poisoned")?;
        self.insert(raster_decode::load(path)?)
    }
    pub fn binding(&self) -> mix::evaluator::ExtFn {
        let rasters = self.clone();
        mix::sync_ext(move |args| {
            let fail = |msg: String| mix::MixError::RuntimeError { msg, span: None };
            let [
                mix::value::Value::String(op),
                mix::value::Value::String(value),
            ] = args.as_slice()
            else {
                return Err(fail(
                    "viu_raster expects load/folder/release and path/handle".into(),
                ));
            };
            let result = match op.as_str() {
                "load" => rasters.load(Path::new(value)).map_err(fail)?,
                "folder" => {
                    if !Path::new(value).is_absolute() {
                        return Err(fail("folder path must be absolute".into()));
                    }
                    json!(raster_decode::list_folder(Path::new(value)).map_err(fail)?)
                }
                "release" => {
                    rasters.release(value);
                    Value::Null
                }
                _ => return Err(fail("unknown raster operation".into())),
            };
            crate::runtime::from_json(result)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn texture_overlap_is_accounted_and_release_restores_capacity() {
        let store = Rasters::default();
        let first = store.texture(RESIDENT_BYTES / 2).unwrap();
        let second = store.texture(RESIDENT_BYTES / 2).unwrap();
        assert!(store.texture(1).is_err());
        drop(first);
        assert!(store.texture(RESIDENT_BYTES / 2 + 1).is_err());
        let replacement = store.texture(RESIDENT_BYTES / 2).unwrap();
        drop(second);
        drop(replacement);
        assert!(store.texture(RESIDENT_BYTES).is_ok());
    }
    #[test]
    fn replacement_at_same_path_has_new_pixels_and_renderer_leases_hold_budget() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("image.png");
        image::RgbaImage::from_pixel(2, 3, image::Rgba([12, 34, 56, 255]))
            .save(&path)
            .unwrap();
        let store = Rasters::default();
        let first = store.load(&path).unwrap();
        let handle = first["handle"].as_str().unwrap();
        let held = store.get(handle).unwrap();
        image::RgbaImage::from_pixel(2, 3, image::Rgba([90, 80, 70, 255]))
            .save(&path)
            .unwrap();
        let second = store.load(&path).unwrap();
        let newer = second["handle"].as_str().unwrap();
        assert_ne!(handle, newer);
        assert_eq!(held.decoded.rgba.get_pixel(0, 0).0, [12, 34, 56, 255]);
        assert_eq!(
            store.get(newer).unwrap().decoded.rgba.get_pixel(0, 0).0,
            [90, 80, 70, 255]
        );
        assert_eq!(store.bytes(), 48);
        store.release(handle);
        assert!(store.get(handle).is_none());
        assert_eq!(store.bytes(), 48, "renderer still holds outgoing pixels");
        drop(held);
        assert_eq!(store.bytes(), 24);
        store.release(newer);
        assert_eq!(store.bytes(), 0);
    }
    #[test]
    fn refused_decodes_and_budget_overflow_leave_existing_handles_intact() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("good.png");
        image::RgbaImage::new(1, 1).save(&path).unwrap();
        let store = Rasters::default();
        let first = store.load(&path).unwrap();
        assert!(store.load(&dir.path().join("missing.png")).is_err());
        // Reserve accounting without allocating 512 MiB in the test.
        store.used.store(RESIDENT_BYTES, Ordering::Release);
        assert!(store.load(&path).unwrap_err().contains("budget"));
        assert!(store.get(first["handle"].as_str().unwrap()).is_some());
        store.used.store(4, Ordering::Release);
        store.release(first["handle"].as_str().unwrap());
        assert_eq!(store.bytes(), 0);
    }
}
