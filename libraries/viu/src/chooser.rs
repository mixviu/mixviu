// SPDX-License-Identifier: MIT OR Apache-2.0
//! Binding to the retained native chooser contract. Engine calls and filesystem
//! probes run on the caller's Mix worker. The UI observes at most eight prepared
//! requests; no extra transport, broker or event dispatcher is created.
use files::chooser::{self as contract, End, Engine, Entry, OpenRequest, Presenter, Unavailable};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Instant,
};
#[derive(Clone)]
pub struct Prepared {
    pub open: OpenRequest,
    pub ready: bool,
}
#[derive(Clone)]
struct Presentation {
    windows: Arc<Mutex<BTreeMap<String, Prepared>>>,
    wake: crate::runtime::Wake,
}
impl Presenter for Presentation {
    fn open(&mut self, open: &OpenRequest) -> Result<(), Unavailable> {
        self.windows.lock().unwrap().insert(
            open.id.clone(),
            Prepared {
                open: open.clone(),
                ready: false,
            },
        );
        (self.wake)();
        Ok(())
    }
    fn input_ready(&mut self, id: &str) {
        if let Some(window) = self.windows.lock().unwrap().get_mut(id) {
            window.ready = true;
        }
        (self.wake)();
    }
    fn close(&mut self, id: &str) {
        self.windows.lock().unwrap().remove(id);
        (self.wake)();
    }
}
struct State {
    engine: Engine,
    presentation: Presentation,
    started: Instant,
}
#[derive(Clone)]
pub struct Service(Arc<Mutex<State>>, Arc<Mutex<BTreeMap<String, Prepared>>>);
impl Service {
    pub fn new(
        incarnation: u64,
        home: Vec<u8>,
        wake: crate::runtime::Wake,
    ) -> Result<Self, String> {
        let defaults = contract::Defaults {
            open_title: "Open file".into(),
            save_title: "Save file".into(),
            save_files_title: "Save files".into(),
            open_accept: "Open".into(),
            save_accept: "Save".into(),
        };
        let windows = Arc::default();
        Ok(Self(
            Arc::new(Mutex::new(State {
                engine: Engine::new(incarnation, defaults, home).map_err(|e| e.message)?,
                presentation: Presentation {
                    windows: Arc::clone(&windows),
                    wake,
                },
                started: Instant::now(),
            })),
            windows,
        ))
    }
    pub fn prepared(&self) -> BTreeMap<String, Prepared> {
        self.1.lock().unwrap().clone()
    }
    pub fn command(&self, command: &str, args: &Value, caller: &str) -> Value {
        let mut state = self.0.lock().unwrap();
        let now = state.started.elapsed().as_millis().min(u64::MAX as u128) as u64;
        let State {
            engine,
            presentation,
            ..
        } = &mut *state;
        let body = args.to_string();
        let mut outcome = None;
        let result = match command {
            "choose" => engine.choose(&body, caller, now, presentation),
            "result" => engine.result(&body, now),
            "list" => Ok(engine.list()),
            "ready" => {
                engine.reply_sent(args["request"].as_str().unwrap_or(""), presentation);
                Ok(json!({}))
            }
            "lost" => {
                engine.reply_failed(args["request"].as_str().unwrap_or(""), presentation);
                Ok(json!({}))
            }
            "cancel" => engine.cancel(&body, now, presentation).map(|value| {
                outcome = value;
                json!({})
            }),
            "accept" => engine.accept(&body, now, presentation, probe).map(|value| {
                let body = json!(value);
                outcome = Some(value);
                body
            }),
            "failed" => engine
                .end(
                    args["request"].as_str().unwrap_or(""),
                    End::Failed(args["message"].as_str().unwrap_or("Chooser failed").into()),
                    now,
                    presentation,
                )
                .map(|value| {
                    outcome = value;
                    json!({})
                }),
            _ => Err(contract::Refusal::new(
                "ARGUMENT",
                "Unknown chooser command",
            )),
        };
        match result {
            Ok(body) => json!({"rc":0,"body":body,"outcome":outcome}),
            Err(error) => json!({"rc":10,"body":error}),
        }
    }
    pub(crate) fn binding(self) -> mix::evaluator::ExtFn {
        mix::sync_ext(move |args| {
            let fail = || mix::MixError::RuntimeError {
                msg: "viu_chooser expects command, body and caller".into(),
                span: None,
            };
            let [
                mix::value::Value::String(command),
                args,
                mix::value::Value::String(caller),
            ] = args.as_slice()
            else {
                return Err(fail());
            };
            crate::runtime::from_json(self.command(
                command,
                &crate::runtime::to_json(args)?,
                caller,
            ))
        })
    }
}
fn probe(path: &[u8]) -> Entry {
    use std::os::unix::ffi::OsStrExt;
    let path = std::path::Path::new(std::ffi::OsStr::from_bytes(path));
    match path.metadata() {
        Ok(m) if m.is_dir() => Entry::Directory,
        Ok(m) if m.is_file() => Entry::File,
        Ok(_) => Entry::Other,
        Err(_) if path.symlink_metadata().is_ok() => Entry::Other,
        Err(_) => Entry::Missing,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delivery_gate_refuses_early_accept_lost_closes_and_outcome_is_once() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("one.txt");
        std::fs::write(&file, "one").unwrap();
        let service = Service::new(
            1,
            dir.path().as_os_str().as_encoded_bytes().to_vec(),
            Arc::new(|| {}),
        )
        .unwrap();
        let opened = service.command("choose", &json!({}), "caller");
        let id = &opened["body"]["request"];
        assert_eq!(
            service.command("accept", &json!({"request":id,"path":file}), "caller")["rc"],
            10
        );
        assert!(!service.prepared().values().next().unwrap().ready);
        service.command("ready", &json!({"request":id}), "host");
        let chosen = service.command("accept", &json!({"request":id,"path":file}), "caller");
        assert_eq!(chosen["outcome"]["response"], 0);
        assert!(service.command("cancel", &json!({"request":id}), "caller")["outcome"].is_null());
        let opened = service.command("choose", &json!({}), "caller");
        let id = &opened["body"]["request"];
        service.command("lost", &json!({"request":id}), "host");
        assert!(service.prepared().is_empty());
        assert_eq!(
            service.command("result", &json!({"request":id}), "caller")["body"]["reason"],
            "unknown"
        );
    }
}
