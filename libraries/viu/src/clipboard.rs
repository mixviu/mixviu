// SPDX-License-Identifier: MIT OR Apache-2.0
//! One bounded in-process request to the owning window's native clipboard.
use serde_json::{Value, json};
use std::{rc::Rc, sync::mpsc};
pub struct Request {
    pub path: String,
    pub reply: tokio::sync::oneshot::Sender<(u8, Value)>,
}
#[derive(Clone)]
pub struct Clipboard {
    tx: mpsc::SyncSender<Request>,
    wake: crate::runtime::Wake,
}
impl Clipboard {
    pub fn channel(wake: crate::runtime::Wake) -> (Self, mpsc::Receiver<Request>) {
        let (tx, rx) = mpsc::sync_channel(1);
        (Self { tx, wake }, rx)
    }
    pub(crate) fn binding(self) -> mix::evaluator::ExtFn {
        Rc::new(move |args| {
            let service = self.clone();
            Box::pin(async move {
                let fail = |msg: &str| mix::MixError::RuntimeError {
                    msg: msg.into(),
                    span: None,
                };
                let [mix::value::Value::String(path)] = args.as_slice() else {
                    return Err(fail("viu_clipboard expects an image path"));
                };
                if !std::path::Path::new(path).is_absolute() {
                    return Err(fail("clipboard image path must be absolute"));
                }
                let (reply, response) = tokio::sync::oneshot::channel();
                service
                    .tx
                    .try_send(Request {
                        path: path.clone(),
                        reply,
                    })
                    .map_err(|_| fail("clipboard owner busy or closed"))?;
                (service.wake)();
                let (rc, body) = tokio::time::timeout(std::time::Duration::from_secs(10), response)
                    .await
                    .map_err(|_| fail("clipboard owner timeout"))?
                    .map_err(|_| fail("clipboard owner closed"))?;
                crate::runtime::from_json(json!({"rc":rc,"body":body}))
            })
        })
    }
}
