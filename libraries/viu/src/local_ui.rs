// SPDX-License-Identifier: MIT OR Apache-2.0
//! In-process calls to the native window owner. This is not a Bus service.
use mix::{BusHandler, MixError, MixResult, evaluator::BusFuture, value::Value};
use std::{rc::Rc, sync::mpsc};
pub struct Request {
    pub command: String,
    pub args: serde_json::Value,
    pub reply: tokio::sync::oneshot::Sender<(i32, serde_json::Value)>,
}
pub struct LocalUi {
    tx: mpsc::SyncSender<Request>,
    pub requests: mpsc::Receiver<Request>,
    wake: crate::runtime::Wake,
}
impl LocalUi {
    pub fn new(wake: crate::runtime::Wake) -> Self {
        let (tx, requests) = mpsc::sync_channel(32);
        Self { tx, requests, wake }
    }
    pub fn backend(&self, base: crate::runtime::BackendFactory) -> crate::runtime::BackendFactory {
        let tx = self.tx.clone();
        let wake = self.wake.clone();
        Box::new(move || {
            Rc::new(Backend {
                base: base(),
                tx,
                wake,
            })
        })
    }
}
struct Backend {
    base: Rc<dyn BusHandler>,
    tx: mpsc::SyncSender<Request>,
    wake: crate::runtime::Wake,
}
fn failure(msg: impl ToString) -> MixError {
    MixError::RuntimeError {
        msg: msg.to_string(),
        span: None,
    }
}
impl BusHandler for Backend {
    fn send<'a>(
        &'a self,
        t: &'a str,
        c: &'a str,
        a: &'a Value,
    ) -> BusFuture<'a, MixResult<(i32, Value)>> {
        Box::pin(async move {
            let (rc, v, _) = self.send_with_reply(t, c, a).await?;
            Ok((rc, v))
        })
    }
    fn send_with_reply<'a>(
        &'a self,
        t: &'a str,
        c: &'a str,
        a: &'a Value,
    ) -> BusFuture<'a, MixResult<(i32, Value, Value)>> {
        Box::pin(async move {
            if t == "viu-local" && c.starts_with("native.")
                && !matches!(c, "native.send" | "native.send.batch") {
                let mut args = crate::runtime::to_json(a)?;
                if let Some(body) = args.get("body") {
                    args = body.clone();
                }
                let (tx, rx) = tokio::sync::oneshot::channel();
                self.tx
                    .try_send(Request {
                        command: c.into(),
                        args,
                        reply: tx,
                    })
                    .map_err(failure)?;
                (self.wake)();
                let (rc, body) = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
                    .await
                    .map_err(failure)?
                    .map_err(failure)?;
                let body = crate::runtime::from_json(body)?;
                return Ok((rc, body.clone(), body));
            }
            self.base.send_with_reply(t, c, a).await
        })
    }
    fn emit<'a>(&'a self, t: &'a str, c: &'a str, a: &'a Value) -> BusFuture<'a, MixResult<()>> {
        self.base.emit(t, c, a)
    }
    fn port_exists<'a>(&'a self, t: &'a str) -> BusFuture<'a, MixResult<bool>> {
        self.base.port_exists(t)
    }
    fn next_incoming<'a>(&'a self) -> BusFuture<'a, Option<mix::evaluator::IncomingEvent>> {
        Box::pin(async { None })
    }
    fn subscribe_topic<'a>(&'a self, t: &'a str) -> BusFuture<'a, MixResult<()>> {
        self.base.subscribe_topic(t)
    }
    fn unsubscribe_topic<'a>(&'a self, t: &'a str) -> BusFuture<'a, MixResult<()>> {
        self.base.unsubscribe_topic(t)
    }
}
