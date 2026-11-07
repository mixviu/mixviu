// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded native request receipts shared with the app's Mix evaluators.
//! Mix chooses whether to defer and when to finish; no intra-app Bus traffic.
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
type Reply = tokio::sync::oneshot::Sender<(u8, Value)>;
struct Entry {
    reply: Reply,
    held: bool,
    close: Arc<AtomicBool>,
    success_body: Option<Value>,
}
#[derive(Default)]
struct State {
    next: u64,
    entries: BTreeMap<String, Entry>,
}
#[derive(Clone, Default)]
pub struct Calls(Arc<Mutex<State>>);
impl Calls {
    pub fn begin(&self, reply: Reply, close: Arc<AtomicBool>) -> Result<String, Reply> {
        self.begin_with_success_body(reply, close, None)
    }
    /// Preserve an admitted native reply while Mix owns completion and refusal.
    pub fn begin_with_success_body(
        &self,
        reply: Reply,
        close: Arc<AtomicBool>,
        success_body: Option<Value>,
    ) -> Result<String, Reply> {
        let mut state = self.0.lock().unwrap();
        state.entries.retain(|_, e| !e.reply.is_closed());
        if state.entries.len() >= 32 {
            return Err(reply);
        }
        state.next += 1;
        let token = state.next.to_string();
        state.entries.insert(
            token.clone(),
            Entry {
                reply,
                held: false,
                close,
                success_body,
            },
        );
        Ok(token)
    }
    pub fn finish(&self, token: &str, rc: u8, body: Value, close: bool) -> bool {
        let entry = self.0.lock().unwrap().entries.remove(token);
        if let Some(e) = entry {
            if close && rc == 0 {
                e.close.store(true, Ordering::Release);
            }
            let body = if rc == 0 {
                e.success_body.unwrap_or(body)
            } else {
                body
            };
            e.reply.send((rc, body)).is_ok()
        } else {
            false
        }
    }
    pub fn complete(&self, token: &str, rc: u8, body: Value) {
        let held = self
            .0
            .lock()
            .unwrap()
            .entries
            .get(token)
            .is_some_and(|e| e.held);
        if !held || rc != 0 {
            self.finish(token, rc, body, false);
        }
    }
    pub fn cancelled(&self) -> Vec<String> {
        let mut state = self.0.lock().unwrap();
        let keys: Vec<_> = state
            .entries
            .iter()
            .filter(|(_, e)| e.reply.is_closed())
            .map(|(k, _)| k.clone())
            .collect();
        for k in &keys {
            state.entries.remove(k);
        }
        keys
    }
    pub fn binding(&self) -> mix::evaluator::ExtFn {
        let calls = self.clone();
        mix::sync_ext(move |args| {
            let fail = |msg: &str| mix::MixError::RuntimeError {
                msg: msg.into(),
                span: None,
            };
            let [
                mix::value::Value::String(op),
                mix::value::Value::String(token),
                body,
            ] = args.as_slice()
            else {
                return Err(fail("viu_reply expects hold/finish, native token, body"));
            };
            let body = crate::runtime::to_json(body)?;
            let ok = match op.as_str() {
                "hold" => {
                    let mut s = calls.0.lock().unwrap();
                    if let Some(e) = s.entries.get_mut(token) {
                        e.held = true;
                        true
                    } else {
                        false
                    }
                }
                "finish" => {
                    let rc = crate::widgets::whole(&body["rc"])
                        .filter(|n| *n <= 255)
                        .ok_or_else(|| fail("rc must be 0..255"))?;
                    calls.finish(token, rc as u8, body["body"].clone(), body["close"] == true)
                }
                _ => return Err(fail("unknown native reply operation")),
            };
            crate::runtime::from_json(json!(ok))
        })
    }
}
