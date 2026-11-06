// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded in-process access to native clipboard requests. Mix owns intents.
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
#[derive(Clone, Default)]
pub struct Platform(Arc<Mutex<VecDeque<Value>>>);
impl Platform {
    pub fn take(&self) -> Vec<Value> {
        self.0.lock().unwrap().drain(..).collect()
    }
    pub fn binding(&self) -> mix::evaluator::ExtFn {
        let platform = self.clone();
        mix::sync_ext(move |args| {
            let fail = |msg: &str| mix::MixError::RuntimeError {
                msg: msg.into(),
                span: None,
            };
            let [
                mix::value::Value::String(kind),
                mix::value::Value::String(id),
                body,
            ] = args.as_slice()
            else {
                return Err(fail("viu_platform expects copy/paste, id, body"));
            };
            if !["copy", "paste"].contains(&kind.as_str()) || id.len() > 128 {
                return Err(fail("invalid native clipboard request"));
            }
            let body = crate::runtime::to_json(body)?;
            if kind == "copy"
                && body["text"]
                    .as_str()
                    .is_none_or(|s| s.len() > 16 * 1024 * 1024)
            {
                return Err(fail("clipboard text requires at most 16 MiB"));
            }
            let mut queue = platform.0.lock().unwrap();
            if queue.len() >= 32 {
                return Err(fail("native clipboard queue full"));
            }
            queue.push_back(json!({"kind":kind,"id":id,"body":body}));
            Ok(mix::value::Value::Nil)
        })
    }
}
