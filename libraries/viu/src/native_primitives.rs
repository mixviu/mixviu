// SPDX-License-Identifier: MIT OR Apache-2.0
//! Small evaluator bindings to existing native primitives. The Mix program
//! chooses the file, remote-read policy and lifetime, normally on a Jobs worker.
use mix::{MixError, value::Value};
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};

/// The existing descriptor-relative, durable replacement primitive. Unlike
/// Mix's path-following write_atomic this refuses a symlink destination.
/// The caller owns serialization and decides whether saving is allowed.
pub(crate) fn atomic_replace() -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        let error = |msg: String| MixError::RuntimeError { msg, span: None };
        let [Value::String(path), Value::String(text)] = args.as_slice() else {
            return Err(error("viu_atomic_replace expects absolute path and text".into()));
        };
        if !std::path::Path::new(path).is_absolute() || text.len() > 1024 * 1024 {
            return Err(error("atomic replacement requires an absolute path and at most 1 MiB".into()));
        }
        config::atomic::replace(std::path::Path::new(path), text.as_bytes())
            .map_err(|e| error(e.to_string()))?;
        Ok(Value::Nil)
    })
}

/// Bounded, nonblocking optional text read through the existing held-directory
/// primitive. Only NotFound means absent; unreadable/linked/special files fail.
pub(crate) fn read_optional() -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        let fail = |msg: String| MixError::RuntimeError { msg, span: None };
        let [Value::String(path)] = args.as_slice() else {
            return Err(fail("viu_read_optional expects one absolute path".into()));
        };
        let path = std::path::Path::new(path);
        if !path.is_absolute() { return Err(fail("expected an absolute path".into())); }
        let read = || -> std::io::Result<Vec<u8>> {
            let parent = config::atomic::open_directory(path.parent().ok_or_else(||std::io::Error::other("missing parent"))?)?;
            config::atomic::read_in(&parent,path.file_name().ok_or_else(||std::io::Error::other("missing name"))?,1024*1024)
        };
        let value = match read() {
            Ok(bytes) => serde_json::json!({"found":true,"text":String::from_utf8(bytes).map_err(|e|fail(e.to_string()))?}),
            Err(error) if error.kind()==std::io::ErrorKind::NotFound => serde_json::json!({"found":false}),
            Err(error) => return Err(fail(error.to_string())),
        };
        crate::runtime::from_json(value)
    })
}
pub(crate) fn thumbnail(stop: Arc<AtomicBool>) -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        let fail = || MixError::RuntimeError {
            msg: "viu_thumbnail requires an absolute path and optional remote bool".into(),
            span: None,
        };
        if args.is_empty() || args.len() > 2 {
            return Err(fail());
        }
        let Some(Value::String(path)) = args.first() else {
            return Err(fail());
        };
        if !std::path::Path::new(path).is_absolute() {
            return Err(fail());
        }
        let remote = match args.get(1) {
            None | Some(Value::Bool(false)) => false,
            Some(Value::Bool(true)) => true,
            _ => return Err(fail()),
        };
        static GENERATOR: OnceLock<Option<thumbs::generate::Generator>> = OnceLock::new();
        let result = GENERATOR.get_or_init(|| thumbs::generate::Generator::from_env("viu"));
        let value = match result {
            Some(generator) => match generator.thumbnail_with(
                std::path::Path::new(path),
                thumbs::Size::XLarge,
                remote,
                &|| stop.load(Ordering::Acquire),
            ) {
                thumbs::Outcome::Ready(path) => serde_json::json!({"phase":"ready","path":path}),
                thumbs::Outcome::Failed(error) => {
                    serde_json::json!({"phase":"failed","message":error})
                }
                thumbs::Outcome::Skipped(reason) => {
                    serde_json::json!({"phase":"skipped","message":reason})
                }
            },
            None => serde_json::json!({"phase":"skipped","message":"no thumbnail cache directory"}),
        };
        crate::runtime::from_json(value)
    })
}
