// SPDX-License-Identifier: MIT OR Apache-2.0
//! Small evaluator bindings to existing native primitives. The Mix program
//! chooses the file, remote-read policy and lifetime, normally on a Jobs worker.
use mix::{MixError, value::Value};
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};
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
