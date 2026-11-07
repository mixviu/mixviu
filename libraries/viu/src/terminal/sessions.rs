// SPDX-License-Identifier: MIT OR Apache-2.0
//! In-process ownership of native terminal sessions. Mix chooses when to
//! create/close and how to compose them; PTY bytes never traverse the model.
use super::{
    ScrollRequest, Terminal,
    config::{Config, Settings},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex, atomic::Ordering},
};

#[derive(Clone, Copy, PartialEq)]
struct Placement {
    rect: [f32; 4],
    cell: [f32; 2],
    scale: f32,
    font_px: f32,
}

struct State {
    next: u64,
    sessions: BTreeMap<String, Arc<Mutex<Terminal>>>,
    placements: BTreeMap<String, Placement>,
    pane_ids: BTreeMap<String, u64>,
}

#[derive(Default)]
struct NativeOwner {
    supervisor: Option<super::session_actor::Supervisor>,
    attempted: bool,
    error: Option<String>,
}

#[derive(Clone)]
pub struct Sessions {
    state: Arc<Mutex<State>>,
    wake: crate::runtime::Wake,
    issuer: Option<Arc<dyn super::native_session::SessionIssuer>>,
    changed: Arc<Mutex<BTreeSet<String>>>,
    native_enabled: bool,
    native: Arc<Mutex<NativeOwner>>,
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
                placements: BTreeMap::new(),
                pane_ids: BTreeMap::new(),
            })),
            wake,
            issuer,
            changed: Arc::default(),
            native_enabled: false,
            native: Arc::default(),
        }
    }

    /// Lazy verified sessions for a host. Only the first terminal creation
    /// starts the actor, on the behaviour thread, with gold's one startup wait.
    pub fn with_native(wake: crate::runtime::Wake) -> Self {
        let mut sessions = Self::new(wake, None);
        sessions.native_enabled = true;
        sessions
    }

    fn native_handle(&self) -> Option<super::session_actor::NativeSession> {
        if !self.native_enabled {
            return None;
        }
        let mut native = self.native.lock().unwrap();
        if !native.attempted {
            native.attempted = true;
            if native.supervisor.is_none() {
                match super::session_actor::Supervisor::start() {
                    Ok(supervisor) => native.supervisor = Some(supervisor),
                    Err(error) => {
                        eprintln!("terminal native session: {error}");
                        native.error = Some(error);
                    }
                }
            }
            if let Some(supervisor) = native.supervisor.as_mut() {
                supervisor.wait_startup();
            }
        }
        native.supervisor.as_ref().map(|s| s.handle.clone())
    }

    pub(crate) fn activity(&self) {
        if let Some(supervisor) = &self.native.lock().unwrap().supervisor {
            supervisor.handle.activity();
        }
    }

    pub(crate) fn take_changes(&self) -> BTreeSet<String> {
        std::mem::take(&mut *self.changed.lock().unwrap())
    }

    pub(crate) fn retry_changes(&self, handles: BTreeSet<String>) {
        self.changed.lock().unwrap().extend(handles);
        (self.wake)();
    }

    pub(crate) fn placed(
        &self,
        handle: &str,
        rect: egui::Rect,
        cell: (u32, u32),
        scale: f32,
        font_px: f32,
    ) {
        let placement = Placement {
            rect: [rect.min.x, rect.min.y, rect.width(), rect.height()],
            cell: [cell.0 as f32 / scale, cell.1 as f32 / scale],
            scale,
            font_px,
        };
        let mut state = self.state.lock().unwrap();
        if state.placements.get(handle) != Some(&placement) {
            state.placements.insert(handle.into(), placement);
            self.changed.lock().unwrap().insert(handle.into());
            (self.wake)();
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
        if op == "session" {
            return Ok(self.native.lock().unwrap().supervisor.as_ref().map_or_else(
                || json!({"diagnostic":"native identity unavailable; panes are graphics-only"}),
                |s| s.handle.status(),
            ));
        }
        if op == "font-size" {
            let base = a["base"].as_f64().ok_or("font base required")? as f32;
            if !super::config::valid_font(base) {
                return Err("invalid font base".into());
            }
            let steps = a["steps"]
                .as_f64()
                .filter(|n| n.is_finite() && n.fract() == 0.0 && n.abs() <= 128.0)
                .ok_or("bounded whole font steps required")? as i32;
            let mut font = super::font::FontSize::new(base);
            font.step_by(steps);
            return Ok(json!(font.current()));
        }
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
            let (id, pane_id) = {
                let mut state = self.state.lock().map_err(|_| "terminal owner poisoned")?;
                let id = state
                    .next
                    .checked_add(1)
                    .ok_or("terminal handle space exhausted")?;
                let pane_id = if a["pane_id"].is_null() {
                    id
                } else {
                    crate::widgets::whole(&a["pane_id"])
                        .filter(|id| *id > 0)
                        .ok_or("positive native pane id required")?
                };
                if state.pane_ids.values().any(|existing| *existing == pane_id) {
                    return Err("native pane id already in use".into());
                }
                state.next = id;
                state.pane_ids.insert(format!("vt-{id}"), pane_id);
                (id, pane_id)
            };
            let native = self.native_handle();
            let issuer = self.issuer.as_deref().or_else(|| {
                native
                    .as_ref()
                    .map(|h| h as &dyn super::native_session::SessionIssuer)
            });
            let started = Terminal::start_session_in(
                Settings {
                    config: Config {
                        scrollback,
                        ..Default::default()
                    },
                    term: term.into(),
                },
                issuer,
                pane_id,
                cwd,
            );
            let t = match started {
                Ok(t) => t,
                Err(error) => {
                    self.state
                        .lock()
                        .unwrap()
                        .pane_ids
                        .remove(&format!("vt-{id}"));
                    return Err(error);
                }
            };
            let handle = format!("vt-{id}");
            let changed = self.changed.clone();
            let wake = self.wake.clone();
            let owner = handle.clone();
            t.set_wake(Arc::new(move || {
                changed.lock().unwrap().insert(owner.clone());
                wake();
            }));
            let binding = native.as_ref().map_or("unavailable", |n| {
                if n.launched(pane_id) {
                    "granted"
                } else {
                    "graphics-only"
                }
            });
            let answer = json!({"handle":handle,"pid":t.pid,"binding":binding,"native_id":pane_id});
            self.state
                .lock()
                .map_err(|_| "terminal owner poisoned")?
                .sessions
                .insert(handle, Arc::new(Mutex::new(t)));
            return Ok(answer);
        }
        if op == "close" {
            let t = {
                let mut state = self.state.lock().map_err(|_| "terminal owner poisoned")?;
                let t = state
                    .sessions
                    .remove(handle)
                    .ok_or("stale terminal handle")?;
                state.placements.remove(handle);
                state.pane_ids.remove(handle);
                t
            };
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
            "state" => {
                let (cols, rows) = t.dimensions();
                let (placement, pane_id) = {
                    let state = self.state.lock().map_err(|_| "terminal owner poisoned")?;
                    (
                        state.placements.get(handle).copied(),
                        state.pane_ids.get(handle).copied(),
                    )
                };
                let binding =
                    self.native
                        .lock()
                        .unwrap()
                        .supervisor
                        .as_ref()
                        .map_or("unavailable", |s| {
                            if pane_id.is_some_and(|id| s.handle.launched(id)) {
                                "granted"
                            } else {
                                "graphics-only"
                            }
                        });
                let g = placement.map_or([0.0; 4], |p| p.rect);
                Ok(
                    json!({"pid":t.pid,"title":t.listener.title(),"cols":cols,"rows":rows,
                    "geometry":{"x":g[0],"y":g[1],"w":g[2],"h":g[3]},
                    "cell":placement.map(|p|p.cell),"scale":placement.map(|p|p.scale),"font_px":placement.map(|p|p.font_px),
                    "exited":t.listener.quit.load(Ordering::Acquire),"display_offset":t.display_offset(),"alternate":t.alternate_screen(),"binding":binding,"native_id":pane_id}),
                )
            }
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
                    _ if a["page"].is_number() => ScrollRequest::Pages(
                        a["page"]
                            .as_f64()
                            .filter(|n| {
                                n.is_finite()
                                    && n.fract() == 0.0
                                    && n.abs() <= 9_007_199_254_740_991.0
                            })
                            .ok_or("whole scroll pages required")? as i64,
                    ),
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
    fn lazy_owner_issues_the_first_real_mix_shell_and_revokes_on_close() {
        const CHILD: &str = "VIU_TERM_OWNER_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let broker = test_broker::Broker::start();
            let home = tempfile::tempdir().unwrap();
            let mut entry = std::mem::MaybeUninit::<libc::passwd>::uninit();
            let mut result = std::ptr::null_mut();
            let mut buffer = vec![0u8; 65536];
            assert_eq!(
                unsafe {
                    libc::getpwuid_r(
                        libc::geteuid(),
                        entry.as_mut_ptr(),
                        buffer.as_mut_ptr().cast(),
                        buffer.len(),
                        &mut result,
                    )
                },
                0
            );
            assert!(!result.is_null());
            let entry = unsafe { entry.assume_init() };
            let account = unsafe { std::ffi::CStr::from_ptr(entry.pw_name) }
                .to_str()
                .unwrap();
            let config = home.path().join("node.conf.mix");
            std::fs::write(
                &config,
                format!(
                    "noded: {{ unix_socket: {}, account: {} }}\n",
                    serde_json::to_string(&broker.endpoint).unwrap(),
                    serde_json::to_string(account).unwrap()
                ),
            )
            .unwrap();
            std::fs::write(
                home.path().join(".mixrc"),
                "fn prompt()\nreturn \"VIU_OWNER> \"\nend\n",
            )
            .unwrap();
            // The embedded broker runs as this test's user, unlike the installed
            // noded account. Isolate its configuration in a subprocess; neither
            // the test runner nor the real PTY launcher changes global env.
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "terminal::sessions::tests::lazy_owner_issues_the_first_real_mix_shell_and_revokes_on_close",
                    "--nocapture",
                ])
                .env(CHILD, "1")
                .env_remove(super::super::session_fd::MARKER)
                .env("HOME", home.path())
                .env("MIXOS", home.path())
                .env("MIXOS_SRC", home.path())
                .env("MIXOS_PREFIX", "/opt/mixos")
                .env("MIXOS_NODE_CONFIG", &config)
                .env("MIXOS_BROKER_ACCOUNT", account)
                .env("MIXOS_BUS_SOCKET", &broker.endpoint)
                .env("MIXOS_RUN", home.path().join("run"))
                .env("MIXOS_ETC", home.path().join("etc"))
                .env("MIXOS_VAR", home.path().join("data"))
                .env("MIXOS_CACHE", home.path().join("cache"))
                .env("XDG_CONFIG_HOME", home.path().join("config"))
                .env("XDG_DATA_HOME", home.path().join("data"))
                .env("XDG_CACHE_HOME", home.path().join("cache"))
                .env("XDG_RUNTIME_DIR", home.path().join("runtime"))
                .env("MIX_STATS", "off")
                .env("MIX_EDITOR", "owned")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "isolated owner test failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        let sessions = Sessions::with_native(Arc::new(|| {}));
        assert!(!sessions.native.lock().unwrap().attempted);
        let home = tempfile::tempdir().unwrap();
        let created = sessions
            .call("create", "", json!({"pane_id":1,"cwd":home.path()}))
            .unwrap();
        assert_eq!(created["binding"], "granted");
        let native = sessions.native_handle().unwrap();
        let handle = created["handle"].as_str().unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            if let Some(generation) = native.pane_generation(1) {
                let snapshot = sessions.call("snapshot", handle, json!({})).unwrap();
                if native.child_binding(1, generation).is_some()
                    && snapshot.as_str().unwrap().contains("VIU_OWNER>")
                {
                    break;
                }
            }
            assert!(
                std::time::Instant::now() < deadline,
                "Mix did not enrol from sealed handoff: {}\n{}",
                native.status(),
                sessions.call("snapshot", handle, json!({})).unwrap()
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert!(sessions.native.lock().unwrap().attempted);
        sessions.call("close", handle, json!({})).unwrap();
        assert!(native.pane_generation(1).is_none());
        let pid = created["pid"].as_i64().unwrap() as libc::pid_t;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while unsafe { libc::kill(pid, 0) } == 0 {
            assert!(
                std::time::Instant::now() < deadline,
                "closed PTY child remained alive"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    #[test]
    fn explicit_pane_identity_is_independent_of_native_handle_counter() {
        let sessions = Sessions::default();
        let home = tempfile::tempdir().unwrap();
        let created = sessions
            .call("create", "", json!({"pane_id":41,"cwd":home.path()}))
            .unwrap();
        assert_eq!(created["handle"], "vt-1");
        assert_eq!(created["native_id"], 41);
        assert!(sessions.call("create", "", json!({"pane_id":41})).is_err());
        assert_eq!(
            sessions.call("state", "vt-1", json!({})).unwrap()["native_id"],
            41
        );
        sessions.call("close", "vt-1", json!({})).unwrap();
        assert!(sessions.state.lock().unwrap().pane_ids.is_empty());
    }

    #[test]
    fn font_steps_use_gold_f32_arithmetic_for_settings_round_trips() {
        let sessions = Sessions::default();
        let mut font = super::super::font::FontSize::new(14.667);
        font.increase();
        assert_eq!(
            sessions
                .call("font-size", "", json!({"base":14.667,"steps":1}))
                .unwrap(),
            json!(font.current())
        );
        for args in [
            json!({"base":5,"steps":1}),
            json!({"base":14,"steps":0.5}),
            json!({"base":14,"steps":1000}),
        ] {
            assert!(sessions.call("font-size", "", args).is_err());
        }
    }

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
