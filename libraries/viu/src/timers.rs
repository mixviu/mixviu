// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded one-shot timers for an application's Mix owner. No idle polling,
//! transport or application policy; the native owner delivers `viu.timer`.
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
#[derive(Clone, Default)]
pub struct Timers(Arc<Mutex<BTreeMap<String, (Instant, Value)>>>);
impl Timers {
    pub(crate) fn deadline(&self) -> Option<Instant> {
        self.0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .map(|(at, _)| *at)
            .min()
    }
    pub fn arm(&self, id: &str, delay: Option<f64>, value: Value) -> Result<(), String> {
        if id.is_empty()
            || id.len() > 128
            || serde_json::to_vec(&value).map_err(|e| e.to_string())?.len() > 65536
        {
            return Err("invalid timer id or payload budget".into());
        }
        let mut timers = self.0.lock().map_err(|_| "timer owner poisoned")?;
        match delay {
            None => {
                timers.remove(id);
            }
            Some(ms) if ms.is_finite() && ms >= 0.0 => {
                if timers.len() >= 128 && !timers.contains_key(id) {
                    return Err("timer capacity exceeded".into());
                }
                let duration = Duration::try_from_secs_f64(ms / 1000.0)
                    .map_err(|_| "timer duration overflows")?;
                let deadline = Instant::now()
                    .checked_add(duration)
                    .ok_or("timer deadline overflows")?;
                timers.insert(id.into(), (deadline, value));
            }
            _ => return Err("timer delay must be finite nonnegative milliseconds".into()),
        }
        Ok(())
    }
    pub fn poll(&self) -> (Vec<Value>, Option<Instant>) {
        let mut timers = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        let due: Vec<_> = timers
            .iter()
            .filter(|(_, (at, _))| *at <= now)
            .map(|(id, _)| id.clone())
            .collect();
        let values = due
            .into_iter()
            .map(|id| {
                let (_, value) = timers.remove(&id).unwrap();
                json!({"timer":id,"value":value})
            })
            .collect();
        (values, timers.values().map(|(at, _)| *at).min())
    }
    pub fn binding(&self) -> mix::evaluator::ExtFn {
        let timers = self.clone();
        mix::sync_ext(move |args| {
            let fail = |msg| mix::MixError::RuntimeError { msg, span: None };
            let [mix::value::Value::String(id), delay, value] = args.as_slice() else {
                return Err(fail(
                    "viu_timer expects id, delay_ms or nil, payload".into(),
                ));
            };
            let delay = match delay {
                mix::value::Value::Nil => None,
                mix::value::Value::Number(n) => Some(*n),
                _ => return Err(fail("timer delay must be number or nil".into())),
            };
            timers
                .arm(id, delay, crate::runtime::to_json(value)?)
                .map_err(fail)?;
            Ok(mix::value::Value::Nil)
        })
    }
}
