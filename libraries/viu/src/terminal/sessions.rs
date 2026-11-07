// SPDX-License-Identifier: MIT OR Apache-2.0
//! In-process ownership of native terminal sessions. Mix chooses when to
//! create/close and how to compose them; PTY bytes never traverse the model.
use super::{
    ScrollRequest, Terminal,
    config::{Config, Settings},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, atomic::Ordering},
};

struct State {
    next: u64,
    sessions: BTreeMap<String, Arc<Mutex<Terminal>>>,
}

#[derive(Clone)]
pub struct Sessions {
    state: Arc<Mutex<State>>,
    wake: crate::runtime::Wake,
    issuer: Option<Arc<dyn super::native_session::SessionIssuer>>,
}

impl Default for Sessions {
    fn default() -> Self {
        Self::new(Arc::new(|| {}), None)
    }
}

impl Sessions {
    #[cfg(test)]
    pub(super) fn fixture() -> Self {
        let sessions = Self::default();
        sessions.state.lock().unwrap().sessions.insert(
            "vt-1".into(),
            Arc::new(Mutex::new(Terminal::from_test_vt(
                80,
                24,
                b"native fixture",
            ))),
        );
        sessions.state.lock().unwrap().next = 1;
        sessions
    }
    pub fn new(
        wake: crate::runtime::Wake,
        issuer: Option<Arc<dyn super::native_session::SessionIssuer>>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                next: 0,
                sessions: BTreeMap::new(),
            })),
            wake,
            issuer,
        }
    }

    pub fn binding(&self) -> mix::evaluator::ExtFn {
        let sessions = self.clone();
        mix::sync_ext(move |args| {
            let fail = |msg| mix::MixError::RuntimeError { msg, span: None };
            let [
                mix::value::Value::String(op),
                mix::value::Value::String(handle),
                body,
            ] = args.as_slice()
            else {
                return Err(fail(
                    "viu_terminal expects operation, handle, arguments".into(),
                ));
            };
            crate::runtime::from_json(
                sessions
                    .call(op, handle, crate::runtime::to_json(body)?)
                    .map_err(fail)?,
            )
        })
    }

    pub(crate) fn get(&self, handle: &str) -> Result<Arc<Mutex<Terminal>>, String> {
        self.state
            .lock()
            .map_err(|_| "terminal owner poisoned")?
            .sessions
            .get(handle)
            .cloned()
            .ok_or("stale terminal handle".into())
    }

    pub fn call(&self, op: &str, handle: &str, a: Value) -> Result<Value, String> {
        if op == "identity" {
            // Gold's process token: OS-random seed, exact through JSON/Mix
            // numbers, independent of reusable pane counters and PID reuse.
            use std::hash::{BuildHasher, Hasher};
            static INSTANCE: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
            let instance = INSTANCE.get_or_init(|| {
                let mut hasher = std::hash::RandomState::new().build_hasher();
                hasher.write_u32(std::process::id());
                (hasher.finish() & ((1 << 53) - 1)).max(1)
            });
            return Ok(json!({"instance":instance,"pid":std::process::id()}));
        }
        if op == "create" {
            let term = a["term"].as_str().unwrap_or("xterm-256color");
            if term.is_empty() || term.len() > 128 || term.chars().any(char::is_control) {
                return Err("invalid TERM".into());
            }
            let scrollback = if a["scrollback"].is_null() {
                1000
            } else {
                crate::widgets::whole(&a["scrollback"])
                    .filter(|n| *n <= 1_000_000)
                    .ok_or("scrollback must be 0..1000000")? as usize
            };
            let cwd = if a["cwd"].is_null() {
                None
            } else {
                Some(a["cwd"].as_str().ok_or("cwd must be text")?.to_owned())
            };
            let id = {
                let mut state = self.state.lock().map_err(|_| "terminal owner poisoned")?;
                state.next = state
                    .next
                    .checked_add(1)
                    .ok_or("terminal handle space exhausted")?;
                state.next
            };
            let t = Terminal::start_session_in(
                Settings {
                    config: Config {
                        scrollback,
                        ..Default::default()
                    },
                    term: term.into(),
                },
                self.issuer.as_deref(),
                id,
                cwd,
            )?;
            t.set_wake(self.wake.clone());
            let handle = format!("vt-{id}");
            let answer = json!({"handle":handle,"pid":t.pid});
            self.state
                .lock()
                .map_err(|_| "terminal owner poisoned")?
                .sessions
                .insert(handle, Arc::new(Mutex::new(t)));
            return Ok(answer);
        }
        if op == "close" {
            let t = self
                .state
                .lock()
                .map_err(|_| "terminal owner poisoned")?
                .sessions
                .remove(handle)
                .ok_or("stale terminal handle")?;
            // Serialize with native input before any teardown. Removal makes
            // this handle permanently stale; new panes receive a new handle.
            let mut t = t.lock().map_err(|_| "terminal poisoned")?;
            t.listener.revoke_control();
            t.listener.quit.store(true, Ordering::Release);
            t.shutdown();
            return Ok(json!({"closed":true}));
        }
        let t = self.get(handle)?;
        let t = t.lock().map_err(|_| "terminal poisoned")?;
        match op {
            "state" => Ok(
                json!({"pid":t.pid,"title":t.listener.title(),"exited":t.listener.quit.load(Ordering::Acquire),"display_offset":t.display_offset(),"alternate":t.alternate_screen()}),
            ),
            "snapshot" => Ok(json!(t.snapshot_with(
                a["contents"] != false,
                crate::widgets::whole(&a["scrollback_lines"]).unwrap_or(0) as usize
            ))),
            "type" => {
                t.listener
                    .bus_text(a["text"].as_str().ok_or("text required")?)?;
                Ok(json!({"written":true}))
            }
            "paste" => {
                t.paste(a["text"].as_str().ok_or("text required")?)?;
                Ok(json!({"pasted":true}))
            }
            "selection" => Ok(json!(t.selection_text())),
            "clear-selection" => {
                t.selection_clear();
                Ok(json!({}))
            }
            "scroll" => {
                let request = match a["to"].as_str() {
                    Some("top") => ScrollRequest::Top,
                    Some("bottom") => ScrollRequest::Bottom,
                    Some("page-up") => ScrollRequest::PageUp,
                    Some("page-down") => ScrollRequest::PageDown,
                    _ => {
                        let n = a["lines"]
                            .as_f64()
                            .filter(|n| {
                                n.is_finite()
                                    && n.fract() == 0.0
                                    && n.abs() <= 9_007_199_254_740_991.0
                            })
                            .ok_or("whole scroll lines required")?;
                        ScrollRequest::Lines(n as i64)
                    }
                };
                let (offset, history, changed) = t.scroll_view_state(request);
                Ok(json!({"display_offset":offset,"history_size":history,"changed":changed}))
            }
            "resize" => {
                let cols = crate::widgets::whole(&a["cols"])
                    .filter(|n| (1..=240).contains(n))
                    .ok_or("cols must be 1..240")? as u16;
                let rows = crate::widgets::whole(&a["rows"])
                    .filter(|n| (1..=100).contains(n))
                    .ok_or("rows must be 1..100")? as u16;
                t.resize(cols, rows, 0, 0);
                Ok(json!({"cols":cols,"rows":rows}))
            }
            _ => Err("unknown terminal operation".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_invalidates_handles_and_already_borrowed_input_owners() {
        let sessions = Sessions::fixture();
        let borrowed = sessions.get("vt-1").unwrap();
        assert_eq!(
            sessions.call("close", "vt-1", json!({})).unwrap(),
            json!({"closed":true})
        );
        assert!(sessions.get("vt-1").is_err());
        assert!(
            sessions
                .call("type", "vt-1", json!({"text":"late"}))
                .is_err()
        );
        assert!(borrowed.lock().unwrap().listener.type_text("late").is_err());
        assert!(sessions.call("close", "vt-1", json!({})).is_err());
    }

    #[test]
    fn invalid_geometry_is_refused_before_mutating_the_native_grid() {
        let sessions = Sessions::fixture();
        for args in [
            json!({"cols":0,"rows":24}),
            json!({"cols":80.5,"rows":24}),
            json!({"cols":80,"rows":101}),
        ] {
            assert!(sessions.call("resize", "vt-1", args).is_err());
        }
        let owner = sessions.get("vt-1").unwrap();
        let t = owner.lock().unwrap();
        let s = t.screen(false);
        assert_eq!((s.cols, s.rows), (80, 24));
    }
}
