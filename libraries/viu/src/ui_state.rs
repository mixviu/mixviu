// SPDX-License-Identifier: MIT OR Apache-2.0
//! A read-only, in-process snapshot of the native view's last drawn geometry
//! and measured timing. Application protocol projection remains in Mix.
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex},
};
#[derive(Clone, Default)]
pub struct UiState(
    Arc<Mutex<Value>>,
    Arc<Mutex<Value>>,
    Arc<Mutex<Option<(u8, String)>>>,
);
impl UiState {
    pub(crate) fn stats(&self) -> Value { self.0.lock().unwrap()["stats"].clone() }
    pub fn take_exit(&self) -> Option<(u8, String)> {
        self.2.lock().unwrap().take()
    }
    pub fn exit_binding(&self) -> mix::evaluator::ExtFn {
        let state = self.clone();
        mix::sync_ext(move |args| {
            let fail = || mix::MixError::RuntimeError {
                msg: "viu_exit expects an integer exit code 0..255 and message".into(),
                span: None,
            };
            let [code, mix::value::Value::String(message)] = args.as_slice() else {
                return Err(fail());
            };
            let code = crate::runtime::to_json(code)?
                .as_u64()
                .filter(|n| *n <= 255)
                .ok_or_else(fail)?;
            *state.2.lock().unwrap() = Some((code as u8, message.clone()));
            Ok(mix::value::Value::Nil)
        })
    }
    pub fn set_outputs(&self, outputs: &BTreeMap<String, Value>) {
        *self.1.lock().unwrap() = json!(outputs);
    }
    pub fn set(&self, window: egui::Rect, geometry: &BTreeMap<String, egui::Rect>, stats: Value) {
        let rect = |r: egui::Rect| json!([r.min.x, r.min.y, r.width(), r.height()]);
        *self.0.lock().unwrap() = json!({"window":rect(window),"geometry":geometry.iter().map(|(id,r)|(id.clone(),rect(*r))).collect::<serde_json::Map<_,_>>(),"stats":stats});
    }
    pub fn binding(&self) -> mix::evaluator::ExtFn {
        let state = self.clone();
        mix::sync_ext(move |_| {
            let mut value = state.0.lock().unwrap().clone();
            if !value.is_object() {
                value = json!({});
            }
            value["outputs"] = state.1.lock().unwrap().clone();
            crate::runtime::from_json(value)
        })
    }
}
#[derive(Default)]
pub struct Measurements {
    pub keys: u64,
    pub frames: u64,
    model: VecDeque<u64>,
    view: VecDeque<u64>,
    next: VecDeque<u64>,
}
fn push(q: &mut VecDeque<u64>, n: u64) {
    if q.len() == 256 {
        q.pop_front();
    }
    q.push_back(n);
}
fn percentiles(q: &VecDeque<u64>) -> Value {
    let mut q: Vec<_> = q.iter().copied().collect();
    q.sort_unstable();
    let at = |p: usize| {
        q.get(q.len().saturating_sub(1) * p / 100)
            .copied()
            .unwrap_or(0)
    };
    json!({"p50":at(50),"p95":at(95),"p99":at(99),"max":q.last().copied().unwrap_or(0)})
}
impl Measurements {
    pub fn model(&mut self, us: u64) {
        push(&mut self.model, us);
    }
    pub fn view(&mut self, us: u64) {
        self.frames += 1;
        push(&mut self.view, us);
    }
    pub fn next(&mut self, us: u64) {
        push(&mut self.next, us);
    }
    pub fn snapshot(&self) -> Value {
        json!({"keys":self.keys,"frames":self.frames,"model_us":percentiles(&self.model),"view_us":percentiles(&self.view),"next_frame_us":percentiles(&self.next)})
    }
}
