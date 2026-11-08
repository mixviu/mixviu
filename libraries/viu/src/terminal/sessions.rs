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
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
};
#[path = "clipboard_requests.rs"]
mod clipboard_requests;
pub(crate) use clipboard_requests::{PaneClipboardReply, WindowClipboardRequest};

// Exact decimal text is the in-process bridge for integers outside Mix's f64
// range. The application's wire policy still rejects JSON string arguments.
fn scroll_integer(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| {
            let text = value.as_str()?;
            let n = text.parse::<i64>().ok()?;
            (n.to_string() == text).then_some(n)
        })
        .or_else(|| {
            value
                .as_f64()
                .filter(|n| n.is_finite() && n.fract() == 0.0 && n.abs() <= 9_007_199_254_740_991.0)
                .map(|n| n as i64)
        })
}

#[cfg(test)]
#[test]
fn exact_scroll_bridge_keeps_i64_limits_and_refuses_noncanonical_text() {
    assert_eq!(
        scroll_integer(&json!("9223372036854775807")),
        Some(i64::MAX)
    );
    assert_eq!(
        scroll_integer(&json!("-9223372036854775808")),
        Some(i64::MIN)
    );
    for text in [
        "9223372036854775808",
        "-9223372036854775809",
        "1.0",
        "+1",
        "01",
        "-0",
    ] {
        assert_eq!(scroll_integer(&json!(text)), None);
    }
    assert_eq!(scroll_integer(&json!(1)), Some(1));
    assert_eq!(scroll_integer(&json!(1.5)), None);
}

#[derive(Clone, Copy, PartialEq)]
struct Placement {
    rect: [f32; 4],
    cell: [f32; 2],
    scale: f32,
    font_px: f32,
}

// Gold's Removed/Cleanup lifecycle: retired terminals keep their admission
// slot until acknowledgement and bounded PTY cleanup finish off the callback.
// The owner is lazy so startup config/version cannot create a cleanup thread.
#[derive(Default)]
struct Cleanup {
    sender: Option<mpsc::SyncSender<Arc<Mutex<Terminal>>>>,
    worker: Option<std::thread::JoinHandle<()>>,
    pending: Arc<AtomicUsize>,
}
impl Cleanup {
    fn start(&mut self) -> Result<(), String> {
        if self.sender.is_some() {
            return Ok(());
        }
        let (sender, receiver) = mpsc::sync_channel::<Arc<Mutex<Terminal>>>(32);
        let pending = self.pending.clone();
        let worker = std::thread::Builder::new()
            .name("viu-terminal-cleanup".into())
            .spawn(move || {
                for terminal in receiver {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        terminal
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .shutdown();
                    }));
                    if result.is_err() {
                        eprintln!("terminal cleanup panicked; retiring its admission slot");
                    }
                    pending.fetch_sub(1, Ordering::AcqRel);
                }
            })
            .map_err(|e| format!("terminal cleanup worker: {e}"))?;
        self.sender = Some(sender);
        self.worker = Some(worker);
        Ok(())
    }
    fn reserve(&self) {
        self.pending.fetch_add(1, Ordering::AcqRel);
    }
    fn submit(&self, terminal: Arc<Mutex<Terminal>>) {
        // Active + pending panes never exceeds 32; this queue cannot fill
        // with nonempty removals, including while the worker is stalled.
        self.sender
            .as_ref()
            .expect("cleanup started before removal")
            .send(terminal)
            .expect("terminal cleanup owner alive");
    }
}
impl Drop for Cleanup {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

struct State {
    next: u64,
    sessions: BTreeMap<String, Arc<Mutex<Terminal>>>,
    placements: BTreeMap<String, Placement>,
    pane_ids: BTreeMap<String, u64>,
    metadata: super::metadata::Metadata,
    clipboard_cursor: usize,
    cleanup: Cleanup,
    closing: bool,
}
impl Drop for State {
    fn drop(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        if self.cleanup.start().is_err() {
            return;
        } // Terminal Drop still owns fallback cleanup.
        for (_, terminal) in std::mem::take(&mut self.sessions) {
            terminal.lock().unwrap_or_else(|e| e.into_inner()).retire();
            self.cleanup.reserve();
            self.cleanup.submit(terminal);
        }
        // Cleanup's field destructor drains before Sessions drops its native
        // actor; no detached cleanup or unacknowledged owner is abandoned.
    }
}

#[derive(Default)]
struct NativeOwner {
    supervisor: Option<super::session_actor::Supervisor>,
    control: Option<Arc<dyn super::session_actor::ControlLane>>,
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
    layout: super::layout_control::Bridge,
    osc52_read: Arc<std::sync::atomic::AtomicBool>,
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
                metadata: super::metadata::Metadata::default(),
                clipboard_cursor: 0,
                cleanup: Cleanup::default(),
                closing: false,
            })),
            layout: super::layout_control::Bridge::new(wake.clone()),
            osc52_read: Arc::default(),
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

    /// Called by the host after stopping its evaluator, before releasing the
    /// native actor. Drain outside State's lock so actor metadata lookup and
    /// transient Weak upgrades cannot make the actor join its own cleanup.
    pub(crate) fn shutdown(&self) {
        let removed = {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.closing = true;
            if !state.sessions.is_empty() && state.cleanup.start().is_err() {
                return;
            }
            let removed = std::mem::take(&mut state.sessions);
            state.placements.clear();
            state.pane_ids.clear();
            for _ in &removed {
                state.cleanup.reserve();
            }
            removed
        };
        for (_, terminal) in removed {
            terminal.lock().unwrap_or_else(|e| e.into_inner()).retire();
            self.state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .cleanup
                .submit(terminal);
        }
        let worker = {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            state.cleanup.sender.take();
            state.cleanup.worker.take()
        };
        if let Some(worker) = worker {
            let _ = worker.join();
        }
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
        let handle = native.supervisor.as_ref().map(|s| s.handle.clone());
        if native.control.is_none() {
            if let Some(handle) = &handle {
                let state = Arc::downgrade(&self.state);
                let pane = Arc::new(move |id| {
                    let state = state.upgrade()?;
                    let state = state.lock().ok()?;
                    let (key, _) = state
                        .pane_ids
                        .iter()
                        .find(|(_, existing)| **existing == id)?;
                    state.sessions.get(key).cloned()
                });
                let state = Arc::downgrade(&self.state);
                let metadata =
                    Arc::new(move || Some(state.upgrade()?.lock().ok()?.metadata.clone()));
                let control: Arc<dyn super::session_actor::ControlLane> =
                    Arc::new(super::recipient::Recipient::new(
                        handle.clone(),
                        pane,
                        metadata,
                        self.layout.clone(),
                    ));
                handle.install_control(&control);
                native.control = Some(control);
            }
        }
        handle
    }

    pub(crate) fn activity(&self) {
        if let Some(supervisor) = &self.native.lock().unwrap().supervisor {
            supervisor.handle.activity();
        }
    }

    pub(crate) fn take_changes(&self) -> BTreeSet<String> {
        std::mem::take(&mut *self.changed.lock().unwrap())
    }
    pub(crate) fn layout_bridge(&self) -> super::layout_control::Bridge {
        self.layout.clone()
    }

    pub(crate) fn retry_changes(&self, handles: BTreeSet<String>) {
        self.changed.lock().unwrap().extend(handles);
        (self.wake)();
    }
    pub(crate) fn clear_other_selections(&self, selected: &str) {
        let panes = self
            .state
            .lock()
            .unwrap()
            .sessions
            .iter()
            .filter(|(handle, _)| handle.as_str() != selected)
            .map(|(_, pane)| pane.clone())
            .collect::<Vec<_>>();
        for pane in panes {
            pane.lock().unwrap().selection_clear();
        }
    }
    #[cfg(test)]
    pub(super) fn fixture_pane(&self, handle: &str, bytes: &[u8]) {
        self.state.lock().unwrap().sessions.insert(
            handle.into(),
            Arc::new(Mutex::new(Terminal::from_test_vt(80, 24, bytes))),
        );
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
        if op == "session.quarantine" {
            super::session_fd::quarantine_inherited();
            return Ok(Value::Null);
        }
        if op == "config" {
            // The terminal's defaults and bounded terminfo probe do not open
            // a PTY or start native session ownership.
            super::session_fd::quarantine_inherited();
            let text = serde_json::to_string(&Settings {
                config: Config::default(),
                term: super::config::selected_term().into(),
            })
            .map_err(|e| e.to_string())?;
            // Serialize the f32 directly before crossing JSON/Mix's f64
            // boundary; Value::from(f32) would expose its widened binary tail.
            return serde_json::from_str(&text).map_err(|e| e.to_string());
        }
        if op == "font.parse" {
            let text = a["text"].as_str().ok_or("font text required")?;
            return match super::config::parse_font(text) {
                None => Ok(Value::Null),
                Some(px) => {
                    serde_json::from_str(&serde_json::to_string(&px).map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())
                }
            };
        }
        if op == "clipboard.configure" {
            self.configure_clipboard(
                a["read_allowed"]
                    .as_bool()
                    .ok_or("read_allowed must be boolean")?,
            );
            return Ok(json!({"configured":true}));
        }
        if op.starts_with("control.") {
            return self.layout.call(op, a);
        }
        if op == "metadata" {
            let metadata = super::metadata::Metadata::decode(&a)?;
            let mut state = self.state.lock().map_err(|_| "terminal owner poisoned")?;
            if metadata
                .tabs
                .iter()
                .flat_map(|t| &t.panes)
                .any(|id| !state.pane_ids.values().any(|p| p == id))
            {
                return Err("metadata references a missing native pane".into());
            }
            state.metadata = metadata;
            return Ok(json!({"published":true}));
        }
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
                if state.closing {
                    return Err("terminal owner closing".into());
                }
                if state.sessions.len().max(state.pane_ids.len())
                    + state.cleanup.pending.load(Ordering::Acquire)
                    >= 32
                {
                    return Err("terminal limit (32 active or closing panes) reached".into());
                }
                state.cleanup.start()?;
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
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            t.set_osc52_read(self.osc52_read.load(Ordering::Acquire));
            state.sessions.insert(handle, Arc::new(Mutex::new(t)));
            return Ok(answer);
        }
        if op == "close" {
            let t = {
                let mut state = self.state.lock().map_err(|_| "terminal owner poisoned")?;
                state.cleanup.start()?;
                let t = state
                    .sessions
                    .remove(handle)
                    .ok_or("stale terminal handle")?;
                state.placements.remove(handle);
                state.pane_ids.remove(handle);
                state.cleanup.reserve();
                t
            };
            // Serialize with native input before any teardown. Removal makes
            // this handle permanently stale; new panes receive a new handle.
            t.lock().unwrap_or_else(|e| e.into_inner()).retire();
            self.state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .cleanup
                .submit(t);
            return Ok(json!({"closed":true}));
        }
        let t = self.get(handle)?;
        let t = t.lock().map_err(|_| "terminal poisoned")?;
        match op {
            // Compound-widget policy decides when focus/geometry invalidates
            // foreground input. Revoke under the same lock as native writes.
            "invalidate-control" => {
                t.listener.revoke_control();
                Ok(json!({"foreground_generation":t.listener.foreground_generation()}))
            }
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
            "search" => {
                let flag = |key: &str, default| match a.get(key) {
                    None => Ok(default),
                    Some(value) => value
                        .as_bool()
                        .ok_or_else(|| format!("{key} must be a boolean")),
                };
                let mut pattern = super::model::SearchPattern::new(
                    a["query"].as_str().ok_or("search query required")?,
                    flag("regex", false)?,
                    flag("case_sensitive", true)?,
                )?;
                let origin = a
                    .get("origin")
                    .map(|value| {
                        serde_json::from_value::<super::model::SearchPoint>(value.clone()).map_err(
                            |_| "search origin requires signed row and non-negative col".to_owned(),
                        )
                    })
                    .transpose()?;
                let max_lines = match a.get("max_lines") {
                    None => 10000,
                    Some(value) => crate::widgets::whole(value)
                        .filter(|n| (1..=10000).contains(n))
                        .ok_or("search line budget must be 1..10000")?
                        as usize,
                };
                serde_json::to_value(t.search(
                    &mut pattern,
                    origin,
                    flag("backwards", false)?,
                    max_lines,
                )?)
                .map_err(|error| error.to_string())
            }
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
                    _ if a.get("page").is_some() => ScrollRequest::Pages(
                        scroll_integer(&a["page"]).ok_or("whole scroll pages required")?,
                    ),
                    _ => ScrollRequest::Lines(
                        scroll_integer(&a["lines"]).ok_or("whole scroll lines required")?,
                    ),
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
    fn startup_config_and_font_parse_never_open_a_session_or_start_the_native_owner() {
        let sessions = Sessions::with_native(Arc::new(|| {}));
        let config = sessions.call("config", "", json!({})).unwrap();
        assert_eq!(
            config["font_px"].to_string(),
            serde_json::to_string(&Config::default().font_px).unwrap()
        );
        assert_eq!(config["scrollback"], 1000);
        assert_eq!(config["cursor"], "underline");
        assert!(matches!(
            config["TERM"].as_str(),
            Some("xterm-rio" | "xterm-256color")
        ));
        assert_eq!(
            sessions
                .call("font.parse", "", json!({"text":"5.9999999"}))
                .unwrap(),
            json!(6.0)
        );
        assert_eq!(
            sessions
                .call("font.parse", "", json!({"text":" 18.5"}))
                .unwrap(),
            Value::Null
        );
        assert_eq!(
            sessions
                .call("font.parse", "", json!({"text":"14.667"}))
                .unwrap()
                .to_string(),
            "14.667"
        );
        assert!(sessions.state.lock().unwrap().sessions.is_empty());
        assert_eq!(sessions.state.lock().unwrap().next, 0);
        assert!(!sessions.native.lock().unwrap().attempted);
        assert!(sessions.native.lock().unwrap().supervisor.is_none());
    }

    #[test]
    fn search_bridge_reads_the_owned_grid_and_refuses_malformed_options() {
        let sessions = Sessions::default();
        sessions.state.lock().unwrap().sessions.insert(
            "fixture".into(),
            Arc::new(Mutex::new(Terminal::from_test_vt(12, 3, b"needle [x]"))),
        );
        let result = sessions
            .call("search", "fixture", json!({"query":"[x]"}))
            .unwrap();
        assert_eq!(result["matched"]["start"], json!({"row":0,"col":7}));
        assert_eq!(result["matched"]["end"], json!({"row":0,"col":9}));
        assert_eq!(result["limited"], false);
        assert_eq!(
            sessions.call("selection", "fixture", json!({})).unwrap(),
            json!(null)
        );
        for options in [
            json!({"query":"needle","regex":"true"}),
            json!({"query":"needle","max_lines":0}),
            json!({"query":"needle","origin":{"row":0,"col":12}}),
            json!({"query":"needle","origin":{"row":0.5,"col":0}}),
            json!({"query":"[","regex":true}),
        ] {
            assert!(sessions.call("search", "fixture", options).is_err());
        }
        assert!(
            sessions
                .call("search", "absent", json!({"query":"needle"}))
                .is_err()
        );
    }

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
            for mode in ["close", "exit", "burst", "delay"] {
                let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "terminal::sessions::tests::lazy_owner_issues_the_first_real_mix_shell_and_revokes_on_close",
                    "--nocapture",
                ])
                .env(CHILD, mode)
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
                    "isolated owner {mode} test failed:\n{}\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
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
        let bound = loop {
            if let Some(generation) = native.pane_generation(1) {
                let snapshot = sessions.call("snapshot", handle, json!({})).unwrap();
                if native.child_binding(1, generation).is_some()
                    && snapshot.as_str().unwrap().contains("VIU_OWNER>")
                {
                    break native.child_binding(1, generation).unwrap();
                }
            }
            assert!(
                std::time::Instant::now() < deadline,
                "Mix did not enrol from sealed handoff: {}\n{}",
                native.status(),
                sessions.call("snapshot", handle, json!({})).unwrap()
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        sessions.call("metadata","",json!({"tabs":[{"id":17,"title":"Mix metadata","active":true,"selected":1,"panes":[1]}]})).unwrap();
        assert_native_controls(&sessions, &bound);
        if std::env::var(CHILD).unwrap() == "burst" {
            assert_native_read_burst(&sessions, &bound);
        }
        assert!(sessions.native.lock().unwrap().attempted);
        let natural_exit = std::env::var(CHILD).unwrap() == "exit";
        if natural_exit {
            sessions
                .call("type", handle, json!({"text":"exit\n"}))
                .unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
            loop {
                let state = sessions.call("state", handle, json!({})).unwrap();
                if state["exited"] == true && native.pane_generation(1).is_none() {
                    break;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "real Mix exit did not invalidate its still-owned terminal"
                );
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        } else {
            sessions.call("close", handle, json!({})).unwrap();
        }
        assert!(native.pane_generation(1).is_none());
        // Observe the real broker before dropping/closing a naturally exited
        // Terminal. Otherwise explicit close could make this proof vacuous.
        let (options, url) = bus::client_helpers::BrokerEnvironment::capture()
            .resolve()
            .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            use bus::native_client::{NodedClient, UnixConnectOutcome};
            let UnixConnectOutcome::VerifiedUnix(observer) =
                NodedClient::connect_unix("", &url, &options, None)
                    .await
                    .unwrap()
            else {
                panic!("verified native observer required");
            };
            tokio::time::timeout(std::time::Duration::from_secs(15), async {
                loop {
                    let record = observer.session_self(bound.record_id).await.unwrap().record;
                    if record.state == bus::native_session::BindingState::Revoked {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                }
            })
            .await
            .expect("real shell close/exit must revoke at the broker");
            observer.client().close().await;
        });
        if natural_exit {
            sessions.call("close", handle, json!({})).unwrap();
        }
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

    fn assert_native_read_burst(sessions: &Sessions, child: &bus::native_session::SessionRecord) {
        use bus::native_client::{NodedClient, UnixConnectOutcome};
        use bus::native_session::Role;
        let (options, url) = bus::client_helpers::BrokerEnvironment::capture()
            .resolve()
            .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let pane = sessions.get("vt-1").unwrap();
        let (entered, held) = std::sync::mpsc::channel();
        let (release, resumed) = std::sync::mpsc::channel();
        let holder = std::thread::spawn(move || {
            let _pane = pane.lock().unwrap();
            entered.send(()).unwrap();
            resumed
                .recv_timeout(std::time::Duration::from_secs(3))
                .unwrap();
        });
        held.recv_timeout(std::time::Duration::from_secs(1))
            .unwrap();
        runtime.block_on(async {
            let UnixConnectOutcome::VerifiedUnix(caller) =
                NodedClient::connect_unix("", &url, &options, None)
                    .await
                    .unwrap()
            else {
                panic!("verified burst caller required")
            };
            let parent = caller
                .session_list()
                .await
                .unwrap()
                .records
                .into_iter()
                .find(|r| r.role == Role::Term && Some(r.instance_id) == child.parent_instance)
                .unwrap();
            let body = json!({"target":{"instance_id":parent.instance_id,
                "incarnation":parent.incarnation,"pane_id":child.pane_id.unwrap(),
                "pane_generation":child.pane_generation.unwrap()}})
            .to_string();
            // Keep the lock held until the reply: the native actor must remain
            // schedulable for cleanup acknowledgements and lease maintenance.
            // The old blocking lock waits for the holder's three-second guard.
            let busy = tokio::time::timeout(
                std::time::Duration::from_millis(400),
                caller.client().call_with_headers_raw(
                    &parent.name,
                    "term.session",
                    &Default::default(),
                    &body,
                ),
            )
            .await
            .expect("a busy pane must not park the native actor")
            .unwrap();
            assert_eq!(busy.0, 10);
            assert_eq!(serde_json::from_str::<Value>(&busy.1).unwrap()["error_code"], "RESOURCE_LIMIT");
            // Use the actual verified transport and bounded recipient queue.
            // A fresh isolated owner keeps this pressure test independent of
            // the ordinary input/replay/expiry proof's connection lifecycle.
            let barrier = Arc::new(tokio::sync::Barrier::new(129));
            let caller = Arc::new(caller);
            let mut calls = tokio::task::JoinSet::new();
            for _ in 0..128 {
                let caller = caller.clone();
                let destination = parent.name.clone();
                let body = body.clone();
                let barrier = barrier.clone();
                calls.spawn(async move {
                    barrier.wait().await;
                    tokio::time::timeout(
                        std::time::Duration::from_secs(10),
                        caller.client().call_with_headers_raw(
                            &destination,
                            "term.session",
                            &Default::default(),
                            &body,
                        ),
                    )
                    .await
                    .expect("burst request must receive a bounded reply")
                    .expect("burst transport must retain its caller")
                });
            }
            barrier.wait().await;
            // A controlled pane read stall lets the burst reach the actual
            // native transport/recipient bounds before the first reply.
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            release.send(()).unwrap();
            let mut accepted = 0;
            let mut refused = 0;
            while let Some(result) = calls.join_next().await {
                let (rc, body, _) = result.unwrap();
                let body: Value = serde_json::from_str(&body).unwrap();
                if rc == 0 {
                    assert!(body["panes"].as_array().is_some_and(|p| !p.is_empty()));
                    accepted += 1;
                } else {
                    assert_eq!(rc, 10, "{body}");
                    assert!(
                        matches!(
                            body["error_code"].as_str(),
                            Some("RESOURCE_LIMIT" | "FORBIDDEN")
                        ),
                        "{body}"
                    );
                    refused += 1;
                }
            }
            assert_eq!(accepted + refused, 128);
            assert!(refused > 0, "the burst must exercise bounded refusal");
            println!("NATIVE_READ_BURST accepted={accepted} refused={refused} completed=128");
        });
        holder.join().unwrap();
    }

    fn assert_native_controls(sessions: &Sessions, child: &bus::native_session::SessionRecord) {
        use bus::native_client::{NodedClient, UnixConnectOutcome};
        use bus::native_session::Role;
        let (options, url) = bus::client_helpers::BrokerEnvironment::capture()
            .resolve()
            .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let UnixConnectOutcome::VerifiedUnix(caller)=NodedClient::connect_unix("",&url,&options,None).await.unwrap() else {panic!("verified control caller required")};
            let parent=caller.session_list().await.unwrap().records.into_iter().find(|r|r.role==Role::Term && Some(r.instance_id)==child.parent_instance).unwrap();
            let target=json!({"instance_id":parent.instance_id,"incarnation":parent.incarnation,"pane_id":child.pane_id.unwrap(),"pane_generation":child.pane_generation.unwrap()});
            let invoke=async |command: &str,body: Value| {
                let (rc,body,_)=tokio::time::timeout(std::time::Duration::from_secs(6),caller.client().call_with_headers_raw(&parent.name,command,&Default::default(),&body.to_string())).await.unwrap().unwrap();
                (rc,serde_json::from_str::<Value>(&body).unwrap())
            };
            // Keep the poison flag through actual verified readback. Both the
            // target lock and the metadata walk must preserve the serial lane.
            let poisoned=sessions.get("vt-1").unwrap();
            let poison=poisoned.clone();
            assert!(std::thread::spawn(move || {
                let _pane=poison.lock().unwrap();
                panic!("controlled protected-readback poison fixture");
            }).join().is_err());
            assert!(poisoned.is_poisoned());
            let (rc,session)=invoke("term.session",json!({"target":target})).await;
            assert_eq!(rc,0,"{session}");
            assert_eq!(session["request_high_water"],"0");
            assert_eq!(session["tabs"][0]["id"],17);
            assert_eq!(session["tabs"][0]["title"],"Mix metadata");
            assert_eq!(session["panes"][0]["tab_id"],17);
            assert_eq!(session["panes"][0]["target"],target);
            for command in ["term.list","term.tabs","term.panes"] {
                assert_eq!(invoke(command,json!({"target":target})).await,(0,session.clone()));
            }
            assert_eq!(invoke("term.props.get",json!({"target":target,"property":"state"})).await,(0,session.clone()));
            sessions.call("metadata","",json!({"tabs":[{"id":17,"title":"renamed by Mix","active":true,"selected":1,"panes":[1]}]})).unwrap();
            let (rc,renamed)=invoke("term.session",json!({"target":target})).await;
            assert_eq!(rc,0,"{renamed}");
            assert_eq!(renamed["tabs"][0]["title"],"renamed by Mix");
            assert_eq!(renamed["panes"][0]["child_pid"],session["panes"][0]["child_pid"]);
            let (rc,snapshot)=invoke("term.snapshot",json!({"target":target})).await;
            assert_eq!(rc,0,"{snapshot}");
            assert!(poisoned.is_poisoned(),"readback must not require clearing poison");
            poisoned.clear_poison();
            let generation=snapshot["foreground_generation"].as_u64().unwrap().to_string();
            let text="print(\"PROTECTED_\" .. \"OWNER\")\n";
            let request=json!({"target":target,"request_id":"1","foreground_generation":generation,"text":text});
            let (rc,accepted)=invoke("term.type",request.clone()).await;
            assert_eq!(rc,0,"{accepted}");
            assert_eq!(accepted["status"],"accepted");
            assert_eq!(accepted["operation_id"],"1");
            assert_eq!(invoke("term.type",request.clone()).await,(0,accepted.clone()));
            let mut conflict=request.clone();conflict["text"]=json!("different");
            assert_eq!(invoke("term.type",conflict).await.1["error_code"],"CONFLICT");
            tokio::time::timeout(std::time::Duration::from_secs(8),async {
                loop {
                    let (rc,screen)=invoke("term.snapshot",json!({"target":target,"contents":true})).await;
                    assert_eq!(rc,0,"{screen}");
                    if screen["text"].as_str().unwrap().contains("PROTECTED_OWNER") {break;}
                    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                }
            }).await.expect("protected input must execute in the real Mix shell");
            let mut stale=target.clone();stale["pane_generation"]=json!((child.pane_generation.unwrap().0+1).to_string());
            assert_eq!(invoke("term.snapshot",json!({"target":stale})).await,(10,json!({"error_code":"FORBIDDEN"})));
            let stale_foreground=json!({"target":target,"request_id":"2","foreground_generation":"0","text":"must not write"});
            assert_eq!(invoke("term.type",stale_foreground).await.1["error_code"],"STALE_GENERATION");
            tokio::time::timeout(std::time::Duration::from_secs(10),async {
                loop {
                    let (rc,outcome)=invoke("term.operation",json!({"target":target,"operation_id":"1"})).await;
                    assert_eq!(rc,0,"{outcome}");
                    if outcome["boundary"]=="pty_write" {
                        assert_eq!(outcome["status"],"completed");
                        assert_eq!(outcome["delivered_bytes_lower_bound"].as_u64().unwrap(),text.len() as u64);
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                }
            }).await.expect("retained input outcome must settle at the PTY boundary");
            assert_eq!(invoke("term.type",request).await,(0,accepted));
            let child_target=json!({"version":1,"target":{"broker_epoch":child.broker_epoch,"record":child.reference(),"instance_id":child.instance_id,"pane_id":child.pane_id,"pane_generation":child.pane_generation}});
            let generation=tokio::time::timeout(std::time::Duration::from_secs(20),async {
                loop {
                    let status=caller.client().call(&child.name,"shell.status",child_target.clone()).await.unwrap();
                    if status["status"]["snapshot"]["phase"]=="prompt-ready" && status["status"]["snapshot"]["continuation"]==false {
                        break status["status"]["snapshot"]["prompt_generation"].as_str().unwrap().to_owned();
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                }
            }).await.expect("child prompt must be ready for forwarded evaluation");
            // The real verified connection supplies this epoch, never test JSON.
            let (rc,current)=invoke("term.session",json!({"target":target})).await;
            assert_eq!(rc,0,"{current}");
            assert_eq!(current["request_high_water"],"2");
            let epoch=current["request_epoch"].clone();
            assert_eq!(epoch,json!(caller.session_context().await.unwrap().connection_id));
            let evaluation=json!({"target":target,"request_id":"3","request_epoch":epoch,"prompt_generation":generation,"source":"print(\"FORWARDED_\" .. \"EVAL\")"});
            let (rc,evaluated)=invoke("term.execute",evaluation.clone()).await;
            assert_eq!(rc,0,"{evaluated}");
            assert_eq!(evaluated["status"],"accepted");
            assert_eq!(evaluated["target"],target);
            let operation=evaluated["operation_id"].clone();
            tokio::time::timeout(std::time::Duration::from_secs(20),async {
                loop {
                    let (rc,result)=invoke("term.exec.result",json!({"target":target,"operation_id":operation})).await;
                    assert_eq!(rc,0,"{result}");
                    if result["state"]=="finished" {
                        assert_eq!(result["result"]["outcome"],"completed");break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                }
            }).await.expect("forwarded evaluation must finish in the real shell");
            assert_eq!(invoke("term.execute",evaluation).await,(0,evaluated));
            let missing_cwd=json!({"target":target,"request_id":"4","request_epoch":epoch,"source":"40 + 2","timeout_ms":"20000"});
            assert_eq!(invoke("term.task.submit",missing_cwd).await.1["error_code"],"INVALID_ARGUMENT");
            let submission=json!({"target":target,"request_id":"5","request_epoch":epoch,"source":"40 + 2","cwd":std::env::var("HOME").unwrap(),"timeout_ms":"20000"});
            let (rc,task)=invoke("term.task.submit",submission.clone()).await;
            assert_eq!(rc,0,"{task}");
            assert_eq!(task["status"],"accepted");
            assert_eq!(task["target"],target);
            let operation=task["operation_id"].clone();
            tokio::time::timeout(std::time::Duration::from_secs(30),async {
                loop {
                    let (rc,result)=invoke("term.task.result",json!({"target":target,"operation_id":operation})).await;
                    assert_eq!(rc,0,"{result}");
                    if result["state"]=="settled" {
                        assert_eq!(result["report"]["outcome"]["kind"],"exited");
                        assert_eq!(result["report"]["outcome"]["code"],0);
                        assert!(result["report"]["result"]["data"].as_str().unwrap().contains("42"));break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                }
            }).await.expect("forwarded isolated task must return its real typed result");
            assert_eq!(invoke("term.task.submit",submission).await,(0,task));
            for command in ["term.exec.result","term.exec.cancel"] {
                assert_eq!(invoke(command,json!({"target":target,"operation_id":operation})).await.1["error_code"],"UNKNOWN_OUTCOME");
            }
            assert!(sessions.call("state","vt-1",json!({})).is_ok());
            // Exercise the real verified recipient -> native mailbox scope ->
            // authored Mix reducer -> retained reply path. The fixture owns
            // only the pump; layout policy is loaded from the real app module.
            let stopped=Arc::new(std::sync::atomic::AtomicBool::new(false));
            let stop=stopped.clone();let owned=sessions.clone();
            let delayed = std::env::var("VIU_TERM_OWNER_CHILD").unwrap() == "delay";
            let pump=std::thread::spawn(move || {
                let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/term");
                let source=r#"$m=require(MODEL)
$policy=require(POLICY)
$native=fn($op,$handle,$args)=viu_terminal($op,$handle,$args)
$s={layout:$m.open_tab($m.initial(),{handle:"vt-1",pid:1},nil),look:{values:{}},closed:false}
$slow=DELAY_FIRST
on viu.terminal.control
  $request=$native("control.inspect","",{})
  if $slow then $slow=false;sleep(3) end
  $out=$policy.apply($s,$request,$native);$s=$out.state
  $published=$native("metadata","",$m.metadata($s.layout))
  $finished=$native("control.finish","",{rc:$out.rc,body:$out.body})
end
"#.replace("MODEL",&serde_json::to_string(&root.join("model.mix")).unwrap())
    .replace("POLICY",&serde_json::to_string(&root.join("protected-layout.mix")).unwrap())
    .replace("DELAY_FIRST", if delayed {"true"} else {"false"});
                let bridge=owned.layout_bridge();
                let mut app=crate::runtime::Runtime::with_services(8,Arc::new(||{}),None,crate::runtime::Services {terminals:Some(owned),..Default::default()});
                app.load(crate::runtime::Program {path:root.join("protected-test.mix"),source,scene:"test".into(),citizen:"term".into()}).unwrap();
                while !stop.load(Ordering::Acquire) {
                    while let Some(done)=app.try_completion() {assert!(done.error.is_none(),"{:?}",done.error);}
                    while let Some(job)=bridge.take() {app.native_event("viu.terminal.control".into(),json!({}),job.context()).unwrap();}
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            });
            let before_layout=invoke("term.snapshot",json!({"target":target})).await.1["foreground_generation"].as_u64().unwrap();
            let selection=json!({"target":target,"request_id":"6","request_epoch":epoch});
            if delayed {
                let before=invoke("term.session",json!({"target":target})).await.1;
                let refused=invoke("term.tab.select",selection.clone()).await;
                assert_eq!(refused.0,10,"{refused:?}");
                assert_eq!(refused.1["error_code"],"FORBIDDEN");
                let after=invoke("term.session",json!({"target":target})).await.1;
                assert_eq!(after["request_high_water"],before["request_high_water"],"expired callback must not reserve its request id");
                assert_eq!(after["foreground_generation"],before["foreground_generation"],"expired callback must not mutate layout");
                let owners=|state:&Value| state["panes"].as_array().unwrap().iter().map(|pane|json!({"target":pane["target"],"pid":pane["child_pid"],"cols":pane["cols"],"rows":pane["rows"]})).collect::<Vec<_>>();
                assert_eq!(owners(&after),owners(&before),"expired callback must retain pane owners and geometry");
            }
            let selected=invoke("term.tab.select",selection.clone()).await;
            assert_eq!(selected,(0,json!({"selected":target,"operation_id":"6"})));
            assert_eq!(invoke("term.tab.select",selection).await,selected);
            assert_eq!(invoke("term.snapshot",json!({"target":target})).await.1["foreground_generation"],before_layout+1,"same-tab selection invalidates once; replay never invalidates again");
            let invalid=json!({"target":target,"request_id":"7","request_epoch":epoch,"dir":"invalid"});
            assert_eq!(invoke("term.pane.split",invalid.clone()).await.1["error_code"],"INVALID_ARGUMENT");
            assert_eq!(invoke("term.pane.split",invalid).await.1["error_code"],"INVALID_ARGUMENT");
            let split=json!({"target":target,"request_id":"8","request_epoch":epoch,"dir":"v"});
            let split_reply=invoke("term.pane.split",split.clone()).await;
            assert_eq!(split_reply,(0,json!({"pane_id":2,"operation_id":"8"})));
            assert_eq!(invoke("term.pane.split",split).await,split_reply);
            assert_eq!(invoke("term.snapshot",json!({"target":target})).await.1["foreground_generation"],before_layout+3,"protected split selects then invalidates the original tab");
            // A sibling now changes geometry too. Omitted authority refuses
            // before reservation and a stale sibling generation also refuses.
            let denied=json!({"target":target,"request_id":"9","request_epoch":epoch});
            assert_eq!(invoke("term.tab.select",denied.clone()).await.1["error_code"],"FORBIDDEN");
            let mut denied_stale=denied;denied_stale["affected"]=json!([{"instance_id":parent.instance_id,"incarnation":parent.incarnation,"pane_id":"2","pane_generation":"0"}]);
            assert_eq!(invoke("term.tab.select",denied_stale).await.1["error_code"],"FORBIDDEN");
            assert_eq!(invoke("term.session",json!({"target":target})).await.1["request_high_water"],"8");
            let ready_pane=async |id:u64| {
                tokio::time::timeout(std::time::Duration::from_secs(20),async {
                    loop {
                        let (rc,state)=invoke("term.session",json!({"target":target})).await;
                        assert_eq!(rc,0,"{state}");
                        if let Some(pane)=state["panes"].as_array().unwrap().iter().find(|pane|pane["target"]["pane_id"]==id.to_string()) {
                            return pane["target"].clone();
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                }).await.expect("new protected pane must enrol and publish native metadata")
            };
            let second=ready_pane(2).await;
            let second_pid=sessions.call("state","vt-2",json!({})).unwrap()["pid"].as_u64().unwrap();
            if std::env::var("VIU_TERM_OWNER_CHILD").unwrap()=="burst" {
                // Metadata reads of pane1 must also refuse promptly when a
                // different enumerated pane is held by cleanup or rendering.
                let pane=sessions.get("vt-2").unwrap();
                let (entered,held)=std::sync::mpsc::channel();
                let (release,resumed)=std::sync::mpsc::channel();
                let holder=std::thread::spawn(move || {
                    let _pane=pane.lock().unwrap();
                    entered.send(()).unwrap();
                    resumed.recv_timeout(std::time::Duration::from_secs(3)).unwrap();
                });
                held.recv_timeout(std::time::Duration::from_secs(1)).unwrap();
                let busy=tokio::time::timeout(std::time::Duration::from_millis(400),invoke("term.session",json!({"target":target}))).await;
                release.send(()).unwrap();holder.join().unwrap();
                let busy=busy.expect("metadata enumeration must not park the native actor");
                assert_eq!(busy.0,10);assert_eq!(busy.1["error_code"],"RESOURCE_LIMIT");
            }
            let new_tab=json!({"target":target,"affected":[second],"request_id":"9","request_epoch":epoch});
            assert_eq!(invoke("term.tab.new",new_tab.clone()).await,(0,json!({"tab_id":2,"operation_id":"9"})));
            assert_eq!(invoke("term.tab.new",new_tab).await,(0,json!({"tab_id":2,"operation_id":"9"})));
            let third=ready_pane(3).await;
            let third_pid=sessions.call("state","vt-3",json!({})).unwrap()["pid"].as_u64().unwrap();
            let fourth_tab=json!({"target":target,"affected":[second,third],"request_id":"10","request_epoch":epoch});
            assert_eq!(invoke("term.tab.new",fourth_tab).await,(0,json!({"tab_id":3,"operation_id":"10"})));
            let fourth=ready_pane(4).await;
            let fourth_pid=sessions.call("state","vt-4",json!({})).unwrap()["pid"].as_u64().unwrap();
            // Closing an inactive singleton first selects it, then its following
            // replacement. This must not silently close the former active pane.
            let close_third=json!({"target":third,"affected":[fourth],"request_id":"11","request_epoch":epoch});
            let closed_third=invoke("term.pane.close",close_third.clone()).await;
            assert_eq!(closed_third,(0,json!({"closed":third,"operation_id":"11"})));
            assert_eq!(invoke("term.pane.close",close_third).await,closed_third);
            assert!(sessions.call("state","vt-3",json!({})).is_err());
            assert!(sessions.call("state","vt-4",json!({})).is_ok());
            let (rc,state)=invoke("term.session",json!({"target":target})).await;
            assert_eq!(rc,0,"{state}");
            assert_eq!(state["tabs"].as_array().unwrap().iter().find(|tab|tab["active"]==true).unwrap()["id"],3);
            // Closing the last-position active tab needs every leaf of its
            // previous replacement tab. A partial set cannot consume the id.
            let partial=json!({"target":fourth,"affected":[target],"request_id":"12","request_epoch":epoch});
            assert_eq!(invoke("term.tab.close",partial).await.1["error_code"],"FORBIDDEN");
            let close_fourth=json!({"target":fourth,"affected":[target,second],"request_id":"12","request_epoch":epoch});
            let closed_fourth=invoke("term.tab.close",close_fourth.clone()).await;
            assert_eq!(closed_fourth,(0,json!({"closed":fourth,"operation_id":"12"})));
            assert_eq!(invoke("term.tab.close",close_fourth).await,closed_fourth);
            let close_second=json!({"target":second,"affected":[target],"request_id":"13","request_epoch":epoch});
            let closed_second=invoke("term.pane.close",close_second.clone()).await;
            assert_eq!(closed_second,(0,json!({"closed":second,"operation_id":"13"})));
            assert_eq!(invoke("term.pane.close",close_second).await,closed_second);
            assert_eq!(invoke("term.operation",json!({"target":second,"operation_id":"13"})).await,closed_second);
            let (rc,state)=invoke("term.session",json!({"target":target})).await;
            assert_eq!(rc,0,"{state}");assert_eq!(state["request_high_water"],"13");
            assert_eq!(state["panes"].as_array().unwrap().len(),1);
            assert_eq!(state["tabs"].as_array().unwrap().len(),1);
            assert_eq!(state["tabs"][0]["id"],1);assert_eq!(state["tabs"][0]["active"],true);
            assert_eq!(state["foreground_generation"],before_layout+7,"open, replacement-tab focus and sibling close preserve gold invalidation counts; refusals and retries do not revoke");
            let stale_layout=json!({"target":target,"request_id":"14","request_epoch":epoch,"foreground_generation":before_layout.to_string(),"text":"LAYOUT_STALE_MUST_NOT_WRITE"});
            assert_eq!(invoke("term.type",stale_layout).await.1["error_code"],"STALE_GENERATION","layout changes invalidate previously usable foreground input tokens");
            assert!(!invoke("term.snapshot",json!({"target":target,"contents":true})).await.1["text"].as_str().unwrap().contains("LAYOUT_STALE_MUST_NOT_WRITE"));
            for handle in ["vt-2","vt-3","vt-4"] {assert!(sessions.call("state",handle,json!({})).is_err());}
            let retired_native=sessions.native.lock().unwrap().supervisor.as_ref().unwrap().handle.clone();
            for id in [2,3,4] {assert!(retired_native.pane_generation(id).is_none(),"closed pane retained local input authority");}
            // A retained close reply means local handles/authority are gone;
            // the owned cleanup queue completes acknowledgement and reaping
            // afterwards. Keep the real process proof, bounded as below for
            // the final pane, without draining the surviving owner's session.
            tokio::time::timeout(std::time::Duration::from_secs(5),async {
                while [second_pid,third_pid,fourth_pid].iter().any(|pid|std::path::Path::new(&format!("/proc/{pid}")).exists()) {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }).await.expect("protected closes must reap every owned child within the cleanup deadline");
            for pid in [second_pid,third_pid,fourth_pid] {
                assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists(),"protected cleanup left child {pid}");
            }
            stopped.store(true,Ordering::Release);pump.join().unwrap();
            // Keep the real broker, enrolled shell and final PTY write path.
            // Blocking only the writer seam makes expiry observable before a
            // human key could revoke the same permit and hide a broken timer.
            let terminal=sessions.state.lock().unwrap().sessions["vt-1"].clone();
            let listener=terminal.lock().unwrap().listener.clone();
            let input_written=|| terminal.lock().unwrap().stats.lock().unwrap().input_written;
            let before=input_written();
            listener.block_control_writes(true);
            let expiry=json!({"target":target,"request_id":"15","request_epoch":epoch,"foreground_generation":listener.foreground_generation().to_string(),"text":"EXPIRED_INPUT_MUST_NOT_WRITE"});
            let expiry_started=std::time::Instant::now();
            let (rc,accepted_expiry)=invoke("term.type",expiry.clone()).await;
            assert_eq!(rc,0,"{accepted_expiry}");
            let expired=tokio::time::timeout(std::time::Duration::from_secs(10),async {
                loop {
                    let (rc,outcome)=invoke("term.operation",json!({"target":target,"operation_id":accepted_expiry["operation_id"]})).await;
                    assert_eq!(rc,0,"{outcome}");
                    if outcome["reason"]=="input_lease_ended" {break outcome;}
                    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                }
            }).await.expect("queued permit must expire without human revocation");
            assert!(expiry_started.elapsed()>=std::time::Duration::from_secs(2),"the permit must age out on its own two-second deadline");
            assert_eq!(expired["status"],"unknown");
            assert_eq!(expired["boundary"],"pty_write");
            assert_eq!(expired["delivered_bytes_lower_bound"],0);
            assert_eq!(input_written(),before,"expiry must be proved before any human key");
            assert_eq!(invoke("term.type",expiry).await,(0,accepted_expiry));

            let revoked=json!({"target":target,"request_id":"16","request_epoch":epoch,"foreground_generation":listener.foreground_generation().to_string(),"text":"REVOKED_INPUT_MUST_NOT_WRITE"});
            assert_eq!(invoke("term.type",revoked.clone()).await.0,0);
            listener.key(super::super::Key::Char(' '),std::time::Instant::now()).unwrap();
            listener.block_control_writes(false);
            listener.key(super::super::Key::Interrupt,std::time::Instant::now()).unwrap();
            let notice=tokio::time::timeout(std::time::Duration::from_secs(5),async {
                loop {
                    let event=caller.recv_shared().await.expect("verified caller remains live");
                    if event.command().command!="term.input.revoked" {continue;}
                    let body:Value=serde_json::from_str(&event.command().body).unwrap();
                    if body["request_id"]=="16" {break (event,body);}
                }
            }).await.expect("private revocation must reach the original verified caller");
            assert_eq!(notice.0.trusted_context().unwrap().session.as_ref().unwrap().record_id,parent.record_id);
            assert_eq!(notice.1["target"],target);
            assert_eq!(notice.1["outcome"],"partial_or_unknown");
            assert_eq!(notice.1["delivered_bytes_lower_bound"],0);
            tokio::time::timeout(std::time::Duration::from_secs(5),async {
                while input_written()-before<2 {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }).await.expect("human keys must reach the real PTY after the writer unblocks");
            assert_eq!(input_written()-before,2,"only the human space and interrupt reach the PTY");
            let (rc,screen)=invoke("term.snapshot",json!({"target":target,"contents":true})).await;
            assert_eq!(rc,0,"{screen}");
            for marker in ["EXPIRED_INPUT_MUST_NOT_WRITE","REVOKED_INPUT_MUST_NOT_WRITE"] {
                assert!(!screen["text"].as_str().unwrap().contains(marker));
            }
            let fresh=json!({"target":target,"request_id":"17","request_epoch":epoch,"foreground_generation":listener.foreground_generation().to_string(),"text":""});
            assert_eq!(invoke("term.type",fresh).await.0,0,"fresh admission survives expired and revoked writers");
            assert_eq!(invoke("term.type",revoked).await.0,0,"retained revoked acceptance survives human invalidation");
            caller.client().close().await;
        });
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
        sessions.call("metadata","",json!({"tabs":[{"id":7,"title":"application","active":true,"selected":41,"panes":[41]}]})).unwrap();
        assert!(sessions.call("metadata","",json!({"tabs":[{"id":7,"title":"missing","active":true,"selected":42,"panes":[42]}]})).is_err());
        assert_eq!(
            sessions.state.lock().unwrap().metadata.tabs[0].title,
            "application"
        );
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
    fn poisoned_terminal_and_cleanup_panic_do_not_poison_the_owner_or_lose_slots() {
        let sessions = Sessions::fixture();
        sessions.fixture_pane("vt-2", b"surviving cleanup");
        let borrowed = sessions.get("vt-1").unwrap();
        let poison = borrowed.clone();
        assert!(
            std::thread::spawn(move || {
                let _pane = poison.lock().unwrap();
                panic!("controlled terminal poison fixture");
            })
            .join()
            .is_err()
        );
        borrowed
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .before_pty_cleanup = Some(Box::new(|| panic!("controlled cleanup fixture")));
        sessions.call("close", "vt-1", json!({})).unwrap();
        sessions.call("close", "vt-2", json!({})).unwrap();
        let pending = sessions.state.lock().unwrap().cleanup.pending.clone();
        sessions.shutdown();
        assert_eq!(pending.load(Ordering::Acquire), 0);
        assert!(sessions.state.lock().unwrap().sessions.is_empty());
        assert!(
            borrowed
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .listener
                .type_text("late")
                .is_err()
        );
    }

    #[test]
    fn slow_cleanup_keeps_its_slot_without_blocking_close_or_borrowed_input_revocation() {
        let sessions = Sessions::fixture();
        for id in 2..=32 {
            sessions.fixture_pane(&format!("vt-{id}"), b"owned cleanup fixture");
        }
        let borrowed = sessions.get("vt-1").unwrap();
        let (release, blocked) = mpsc::sync_channel(1);
        borrowed.lock().unwrap().before_pty_cleanup = Some(Box::new(move || {
            blocked
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
        }));
        let pending = sessions.state.lock().unwrap().cleanup.pending.clone();
        let start = std::time::Instant::now();
        sessions.call("close", "vt-1", json!({})).unwrap();
        let elapsed = start.elapsed();
        // Do not lock the Terminal here: the cleanup worker deliberately owns
        // its mutex while stalled. The borrowed Listener remains invalidated.
        let admission = sessions.call("create", "", json!({"pane_id":33}));
        assert!(admission.unwrap_err().contains("32 active or closing"));
        assert!(sessions.get("vt-1").is_err());
        assert!(!sessions.native.lock().unwrap().attempted);
        assert_eq!(pending.load(Ordering::Acquire), 1);
        release.send(()).unwrap();
        assert!(
            elapsed < std::time::Duration::from_secs(1),
            "close waited for cleanup: {elapsed:?}"
        );
        assert!(borrowed.lock().unwrap().listener.type_text("late").is_err());
        // Owner teardown drains queued cleanup, while the native actor is
        // still retained. No detached job survives the final Sessions owner.
        let retained = sessions.clone();
        sessions.shutdown();
        assert_eq!(pending.load(Ordering::Acquire), 0);
        assert!(
            retained
                .call("create", "", json!({}))
                .unwrap_err()
                .contains("owner closing")
        );
        drop(sessions);
        drop(retained);
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
