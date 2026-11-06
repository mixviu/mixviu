// SPDX-License-Identifier: MIT OR Apache-2.0
//! Independent Rust Dopus driver. Only this comparator calls dopus-core.
//! Composition and the rendered control set are shared with the Mix driver.
use dopus_core::{
    ConfigFile, ConfirmAnswer, CoreEvent, DopusCore, DropAction, PaneId, PromptKind, SortColumn,
};
use dopus_core::{config::Sidebar, properties::Properties};
use eframe::egui;
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::mpsc,
    thread::JoinHandle,
    time::{Instant, SystemTime},
};
use viu::runtime::Wake;

pub struct Native {
    core: Option<DopusCore>,
    rx: Option<mpsc::Receiver<CoreEvent>>,
    relay: Option<JoinHandle<()>>,
    contract: Value,
    default_keys: Value,
    appearance_choice: Value,
    appearance_session: Value,
    window: Value,
    window_choice: Value,
    pub appearance: Value,
    form: Value,
    form_kind: String,
    token: u64,
    form_instance: u64,
    form_queue: VecDeque<(Value, String, u64, Option<Value>)>,
    drop_request: Option<Value>,
    locations: [String; 2],
    pub serial: u64,
    pub focus: Option<Value>,
    info: String,
    pub effects: VecDeque<Value>,
}
fn closed() -> Value {
    json!({"open":false,"title":"","body":"","field":null,"choices":[],"instance":0})
}
fn string<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key].as_str().ok_or_else(|| format!("{key} must be text"))
}
fn path_value(path: &Path) -> Value {
    viu::file_data::encode_path(path).unwrap_or_else(|error| {
        eprintln!("Dopus path encoding refused: {error}");
        Value::Null
    })
}
fn path_text(path: &Path) -> String {
    path_value(path).as_str().unwrap_or("").to_owned()
}
fn paths_value<P: AsRef<Path>>(paths: impl IntoIterator<Item = P>) -> Value {
    Value::Array(paths.into_iter().map(|p| path_value(p.as_ref())).collect())
}
fn native_path(text: &str) -> Result<PathBuf, String> {
    viu::file_data::decode_path(text)
}
fn pane(i: usize) -> PaneId {
    if i == 0 { PaneId::Left } else { PaneId::Right }
}
fn date(t: Option<SystemTime>) -> String {
    t.map(dopus_core::format_modified_at)
        .unwrap_or_else(|| "—".into())
}
fn icon(path: &Path, dir: bool, expanded: bool) -> &'static str {
    if dir {
        return if expanded { "folder-open" } else { "folder" };
    }
    match path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "avif" => "file-image",
        "mp3" | "flac" | "wav" | "ogg" => "file-music",
        "mp4" | "m4v" | "mkv" | "webm" => "file-video-camera",
        "zip" | "gz" | "tar" | "7z" => "file-archive",
        "rs" | "mix" | "c" | "h" | "toml" | "yaml" | "yml" | "json" => "file-code",
        "txt" | "log" | "conf" | "md" | "html" | "htm" | "css" | "csv" | "xml" | "pdf" => {
            "file-text"
        }
        _ => "file",
    }
}
impl Native {
    pub fn new(profile: &Path, contract: Value, wake: Wake) -> Self {
        let (config, file) = ConfigFile::load(profile);
        let locations = [path_text(&config.left.path), path_text(&config.right.path)];
        let (core, source) = DopusCore::new(config, Some(file));
        let (tx, rx) = mpsc::sync_channel(64);
        // Preserve the original core's worker delivery and wake the UI exactly
        // when an event arrives. No periodic idle polling is introduced.
        let relay = std::thread::Builder::new()
            .name("dopus-core-events".into())
            .spawn(move || {
                while let Ok(event) = source.recv() {
                    if tx.send(event).is_err() {
                        break;
                    }
                    wake();
                }
            })
            .expect("core event relay");
        Self {
            core: Some(core),
            rx: Some(rx),
            relay: Some(relay),
            default_keys: contract["actions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| (a["id"].as_str().unwrap().to_owned(), a["keys"].clone()))
                .collect::<serde_json::Map<_, _>>()
                .into(),
            contract,
            appearance_choice: json!({}),
            appearance_session: json!({"mode":"dark","scheme":"studio","style":"default"}),
            window: json!({}),
            window_choice: json!({}),
            appearance: json!({"mode":"dark","scheme":"studio","style":"default"}),
            form_instance: 0,
            form_queue: VecDeque::new(),
            form: closed(),
            form_kind: String::new(),
            token: 0,
            drop_request: None,
            locations,
            serial: 0,
            focus: None,
            info: "Select a file or folder".into(),
            effects: VecDeque::new(),
        }
    }
    fn core(&self) -> &DopusCore {
        self.core.as_ref().unwrap()
    }
    fn core_mut(&mut self) -> &mut DopusCore {
        self.core.as_mut().unwrap()
    }
    fn enqueue_form(
        &mut self,
        kind: &str,
        token: u64,
        mut form: Value,
        drop_request: Option<Value>,
    ) -> Result<(), String> {
        if self.form_queue.len() >= 16 {
            return Err("Dialog queue is full".into());
        }
        self.form_instance += 1;
        form["instance"] = json!(self.form_instance);
        if self.form["open"] == true {
            self.form_queue
                .push_back((form, kind.into(), token, drop_request));
        } else {
            self.form = form;
            self.form_kind = kind.into();
            self.token = token;
            self.drop_request = drop_request;
        }
        Ok(())
    }
    fn advance_form(&mut self) {
        if let Some((form, kind, token, drop_request)) = self.form_queue.pop_front() {
            self.form = form;
            self.form_kind = kind;
            self.token = token;
            self.drop_request = drop_request;
        } else {
            self.form = closed();
            self.form_kind.clear();
            self.token = 0;
            self.drop_request = None;
        }
    }
    fn absorb(&mut self, events: Vec<CoreEvent>) {
        for event in events {
            match event {
                CoreEvent::PromptRequested {
                    token,
                    kind,
                    initial,
                } => {
                    let form_kind = match kind {
                        PromptKind::NewFolder => "mkdir",
                        PromptKind::Rename => "rename",
                    };
                    let form = json!({"open":true,"title":if kind==PromptKind::NewFolder{"New folder"}else{"Rename"},"body":"","field":initial,"error":"","choices":[{"id":"cancel","label":"Cancel","enabled":true,"cancel":true},{"id":"ok","label":"OK","enabled":true,"default":true}]});
                    if let Err(error) = self.enqueue_form(form_kind, token, form, None) {
                        self.core_mut().prompt_text(token, None);
                        self.info = error;
                    }
                }
                CoreEvent::ConfirmRequested { token, message } => {
                    let form = json!({"open":true,"title":"Delete permanently","body":message,"field":null,"error":"","choices":[{"id":"cancel","label":"Cancel","enabled":true,"cancel":true},{"id":"ok","label":"Delete","enabled":true,"default":true}]});
                    if let Err(error) = self.enqueue_form("delete", token, form, None) {
                        self.core_mut().confirm(token, ConfirmAnswer::No);
                        self.info = error;
                    }
                }
                CoreEvent::Status { kind, text, .. } => {
                    self.info = if kind == dopus_core::StatusKind::Message {
                        text
                    } else {
                        self.core().info().into()
                    }
                }
                CoreEvent::OpenFile(path) => self
                    .effects
                    .push_back(json!({"kind":"open","path":path_value(&path)})),
                _ => {}
            }
        }
    }
    pub fn poll(&mut self, ctx: &egui::Context) -> bool {
        let mut changed = false;
        while let Ok(event) = self.rx.as_ref().unwrap().try_recv() {
            let derived = self.core_mut().on_event(event);
            self.absorb(derived);
            changed = true;
        }
        let events = self.core_mut().tick(Instant::now());
        changed |= !events.is_empty();
        self.absorb(events);
        if let Some(deadline) = self.core().next_deadline() {
            ctx.request_repaint_after(deadline.saturating_duration_since(Instant::now()));
        }
        for i in 0..2 {
            let actual = path_text(&self.core().pane(pane(i)).path);
            if self.locations[i].is_empty() {
                self.locations[i] = actual;
            }
        }
        changed
    }
    pub fn enabled(&self, id: &str) -> bool {
        let a = self.core().availability();
        match id {
            "file.open" => a.has_selection,
            "file.mail" => a.selection_all_files && a.has_selection,
            "file.rename" => a.selection_count == 1 && !a.operation_running,
            "file.copy-other-pane" | "file.move-other-pane" | "file.delete" => {
                a.has_selection && !a.operation_running
            }
            "file.new-folder" => !a.operation_running,
            id if id.starts_with("selection.") => a.rows_available,
            _ => true,
        }
    }
    fn command(&self, id: &str) -> Value {
        let mut a = self.contract["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == id)
            .cloned()
            .unwrap_or_else(|| json!({"id":id,"label":id,"keys":[]}));
        a["enabled"] = json!(self.enabled(id));
        a["shortcut"] = json!(
            a["keys"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" / ")
        );
        let p = self.core().pane(self.core().active());
        a["checked"] = json!(match id {
            "view.toggle-hidden" => p.show_hidden,
            "view.toggle-previews" => p.previews,
            "view.toggle-places" => self.core().sidebar(Sidebar::Places).open,
            "view.toggle-properties" => self.core().sidebar(Sidebar::Properties).open,
            "theme.scheme-session" | "theme.mode-session" | "theme.style-session" => {
                let key = id
                    .strip_prefix("theme.")
                    .unwrap()
                    .strip_suffix("-session")
                    .unwrap();
                self.appearance_choice.get(key).is_none()
            }
            id if id.starts_with("theme.decorations-") =>
                self.window_choice["window.decorations"]
                    .as_str()
                    .unwrap_or("auto")
                    == &id[18..],
            id if id.starts_with("theme.captions-") =>
                self.window_choice["window.caption_side"]
                    .as_str()
                    .unwrap_or("auto")
                    == &id[15..],
            id if id.starts_with("theme.scheme-") =>
                self.appearance_choice["scheme"].as_str() == Some(&id[13..]),
            id if id.starts_with("theme.style-") =>
                self.appearance_choice["style"].as_str() == Some(&id[12..]),
            id if id.starts_with("theme.mode-") =>
                self.appearance_choice["mode"].as_str() == Some(&id[11..]),
            _ => false,
        });
        a
    }
    pub fn actions(&self) -> Value {
        json!({"actions":self.contract["actions"].as_array().unwrap().iter().map(|a|self.command(a["id"].as_str().unwrap())).collect::<Vec<_>>()})
    }
    pub fn effect_result(&mut self, rc: u8, body: &Value) {
        self.info = body["message"]
            .as_str()
            .unwrap_or(if rc == 0 {
                "Completed"
            } else {
                "Application effect failed"
            })
            .to_owned();
    }
    fn search_choices(&self) -> Value {
        let query = self.form["field"].as_str().unwrap_or("").to_lowercase();
        let mut choices = self.contract["actions"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|a| {
                a["id"] != "search"
                    && (a["label"]
                        .as_str()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&query)
                        || a["id"].as_str().unwrap_or("").contains(&query))
            })
            .map(|a| self.command(a["id"].as_str().unwrap()))
            .collect::<Vec<_>>();
        if let Some(first) = choices.first_mut() {
            first["default"] = json!(true);
        }
        choices.push(json!({"id":"cancel","label":"Cancel","enabled":true,"cancel":true}));
        json!(choices)
    }
    fn menus(&self) -> Value {
        let mut menus = self.contract["menus"].clone();
        for m in menus.as_array_mut().unwrap() {
            for a in m["children"].as_array_mut().unwrap() {
                *a = self.toolbar_item(a.clone());
            }
        }
        menus
    }
    fn toolbar_item(&self, mut item: Value) -> Value {
        if let Some(children) = item["children"].as_array() {
            item["children"] = json!(
                children
                    .iter()
                    .cloned()
                    .map(|v| self.toolbar_item(v))
                    .collect::<Vec<_>>()
            );
        } else {
            let command = self.command(item["id"].as_str().unwrap_or(""));
            for key in ["enabled", "checked", "shortcut"] {
                item[key] = command[key].clone();
            }
        }
        item
    }
    fn toolbar(&self) -> Value {
        json!(
            self.contract["toolbar"]
                .as_array()
                .into_iter()
                .flatten()
                .cloned()
                .map(|item| self.toolbar_item(item))
                .collect::<Vec<_>>()
        )
    }
    pub fn action(&mut self, id: &str, params: &Value) -> Result<Value, String> {
        let empty = json!({});
        let params = if params.is_null() { &empty } else { params };
        let fields = params.as_object().ok_or("Action params must be a map")?;
        if fields.keys().any(|k| {
            !["pane", "ctrl", "shift", "path"].contains(&k.as_str())
                || (k == "path" && id != "place.open")
        }) {
            return Err("Unknown action parameter".into());
        }
        if ["ctrl", "shift"].iter().any(|key| {
            params
                .get(*key)
                .is_some_and(|v| !v.is_null() && !v.is_boolean())
        }) {
            return Err("Selection modifier must be bool".into());
        }
        if id != "place.open"
            && !self.contract["actions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|a| a["id"] == id)
        {
            return Err("UNKNOWN_ACTION".into());
        }
        if id == "place.open" && string(params, "path")?.is_empty() {
            return Err("Path must be non-empty text".into());
        }
        let previous = self.core().active();
        if let Some(target) = params.get("pane").filter(|v| !v.is_null()) {
            let p = match (target.as_str(), target.as_u64()) {
                (Some("left"), _) | (_, Some(0)) => PaneId::Left,
                (Some("right"), _) | (_, Some(1)) => PaneId::Right,
                (Some("active"), _) => self.core().active(),
                _ => return Err("Unknown pane".into()),
            };
            self.core_mut().set_active_pane(p);
        }
        if !self.enabled(id) {
            self.core_mut().set_active_pane(previous);
            return Err("UNAVAILABLE".into());
        }
        let p = self.core().active();
        let i = p.index();
        self.focus = None;
        match id {
            "nav.switch-pane" => self.core_mut().switch_pane(),
            "nav.back" => self.core_mut().go_back(),
            "nav.forward" => self.core_mut().go_forward(),
            "nav.parent" => self.core_mut().go_parent(),
            "nav.home" => self.core_mut().go_home(),
            "place.open" => self
                .core_mut()
                .navigate(p, native_path(string(params, "path")?)?),
            "location.focus" => {
                self.focus = Some(
                    json!({"id":format!("{}-location",if i==0{"left"}else{"right"}),"selection":[0,self.locations[i].chars().count()]}),
                );
            }
            "view.toggle-places" => self.core_mut().toggle_sidebar(Sidebar::Places),
            "view.toggle-properties" => self.core_mut().toggle_sidebar(Sidebar::Properties),
            "view.refresh" => self.core_mut().refresh(),
            "view.toggle-hidden" => self.core_mut().toggle_hidden(),
            "view.toggle-previews" => self.core_mut().toggle_previews(),
            id if id.starts_with("view.sort-") => {
                let col = match &id[10..] {
                    "name" => SortColumn::Name,
                    "size" => SortColumn::Size,
                    "modified" => SortColumn::Modified,
                    _ => return Err("Unknown sort column".into()),
                };
                let old = self.core().pane(p);
                let ascending = old.sort != col || !old.ascending;
                self.core_mut().set_sort(col, ascending);
            }
            "selection.next" | "selection.previous" => self.core_mut().select_relative_modified(
                p,
                if id == "selection.next" { 1 } else { -1 },
                params["ctrl"] == true,
                params["shift"] == true,
            ),
            "selection.first" | "selection.last" => self.core_mut().select_edge_modified(
                p,
                id == "selection.last",
                params["ctrl"] == true,
                params["shift"] == true,
            ),
            "file.open" => self.core_mut().open_selection(),
            "file.new-folder" => self.core_mut().begin_new_folder(),
            "file.rename" => self.core_mut().begin_rename(),
            "file.delete" => self.core_mut().delete_selection(),
            "file.copy-other-pane" => self.core_mut().copy_selection_to_other_pane(),
            "file.move-other-pane" => self.core_mut().move_selection_to_other_pane(),
            "file.mail" => {
                let paths = self.core().operation_sources(p);
                self.effects
                    .push_back(json!({"kind":"mail","paths":paths_value(paths)}));
            }
            "theme.scheme-session" | "theme.mode-session" | "theme.style-session" => {
                let key = id
                    .strip_prefix("theme.")
                    .unwrap()
                    .strip_suffix("-session")
                    .unwrap();
                self.appearance_choice.as_object_mut().unwrap().remove(key);
                self.appearance[key] = self.appearance_session[key].clone();
            }
            id if id.starts_with("theme.decorations-") => {
                self.window_choice["window.decorations"] = json!(&id[18..])
            }
            id if id.starts_with("theme.captions-") => {
                self.window_choice["window.caption_side"] = json!(&id[15..])
            }
            "theme.mode-toggle" => {
                self.appearance["mode"] = json!(if self.appearance["mode"] == "dark" {
                    "light"
                } else {
                    "dark"
                })
            }
            id if id.starts_with("theme.scheme-") => self.appearance["scheme"] = json!(&id[13..]),
            id if id.starts_with("theme.style-") => self.appearance["style"] = json!(&id[12..]),
            id if id.starts_with("theme.mode-") => self.appearance["mode"] = json!(&id[11..]),
            "help.macros" => return Ok(json!({"panel":"macros"})),
            "help.about" | "help.shortcuts" => {
                let body = if id == "help.about" {
                    "Dopus — Rust behaviour with shared native Viu widgets.\nLab comparison implementation.".into()
                } else {
                    self.contract["actions"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|a| {
                            format!(
                                "{}  {}",
                                a["keys"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .filter_map(Value::as_str)
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                a["label"].as_str().unwrap()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                };
                let form = json!({"open":true,"legacy_kind":if id=="help.about"{"about"}else{"shortcuts"},"title":if id=="help.about"{"About Dopus"}else{"Keyboard shortcuts"},"body":body,"field":null,"choices":[{"id":"cancel","label":"Cancel","enabled":true,"cancel":true},{"id":"ok","label":"OK","enabled":true,"default":true}]});
                self.enqueue_form("message", 0, form, None)?;
            }
            "search" => {
                if self.form["open"] == true {
                    return Err("Answer the current dialog first".into());
                }
                self.enqueue_form("search",0,json!({"open":true,"title":"Search commands","body":"","field":"","choice_layout":"list","choices":[]}),None)?;
                self.form["choices"] = self.search_choices();
            }
            "app.quit" => {
                self.core_mut().flush_config();
            }
            _ => return Err("UNKNOWN_ACTION".into()),
        }
        if id.starts_with("theme.") && !id.ends_with("-session") {
            let key = if id.starts_with("theme.scheme-") {
                Some("scheme")
            } else if id.starts_with("theme.style-") {
                Some("style")
            } else if id.starts_with("theme.mode-") {
                Some("mode")
            } else {
                None
            };
            if let Some(key) = key {
                self.appearance_choice[key] = self.appearance[key].clone();
            }
        }
        for i in 0..2 {
            self.locations[i] = path_text(&self.core().pane(pane(i)).path);
        }
        let events = self.core_mut().tick(Instant::now());
        self.absorb(events);
        self.serial += 1;
        Ok(json!({"ok":true}))
    }
    pub fn answer(&mut self, choice: &str, text: Option<&str>) -> Result<Value, String> {
        if self.form["open"] != true {
            return Err("No dialog is open".into());
        }
        if let Some(text) = text {
            self.form["field"] = json!(text);
        }
        if choice != "cancel" && self.form_kind == "search" {
            if !self.form["choices"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["id"] == choice && c["enabled"] == true)
            {
                return Err("Command is not available".into());
            }
            self.advance_form();
            return self.action(choice, &json!({}));
        }
        if choice != "cancel" && self.form_kind == "drop" {
            let action = match choice {
                "copy" => DropAction::Copy,
                "move" => DropAction::Move,
                _ => return Err("Unknown choice".into()),
            };
            let request = self.drop_request.as_ref().unwrap();
            let sources = request["keys"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_str)
                .map(native_path)
                .collect::<Result<Vec<_>, _>>()?;
            let destination = native_path(string(request, "destination")?)?;
            let source_pane = if request["source"]
                .as_str()
                .is_some_and(|s| s.starts_with("right-"))
            {
                PaneId::Right
            } else {
                PaneId::Left
            };
            if let Err(error) =
                self.core_mut()
                    .transfer_paths(source_pane, sources, destination, action)
            {
                self.advance_form();
                self.info = error.clone();
                return Err(error);
            }
        } else if self.form_kind == "delete" {
            let token = self.token;
            self.core_mut().confirm(
                token,
                if choice == "ok" {
                    ConfirmAnswer::Yes
                } else {
                    ConfirmAnswer::No
                },
            );
        } else if ["mkdir", "rename"].contains(&self.form_kind.as_str()) {
            let value = if choice == "cancel" {
                None
            } else if choice == "ok" {
                let value = string(&self.form, "field")?;
                if let Err(error) = dopus_core::validate_filename(value) {
                    self.form["error"] = json!(error.clone());
                    return Err(error);
                }
                Some(value.to_owned())
            } else {
                return Err("Unknown choice".into());
            };
            let token = self.token;
            self.core_mut().prompt_text(token, value);
        } else if choice != "cancel" && choice != "ok" {
            return Err("Unknown choice".into());
        }
        self.advance_form();
        let events = self.core_mut().tick(Instant::now());
        self.absorb(events);
        Ok(json!({"ok":true}))
    }
    pub fn intent(&mut self, a: &Value) -> Result<Value, String> {
        self.focus = None;
        let id = string(a, "id")?;
        let action = string(a, "action")?;
        let v = &a["value"];
        if id == "menu" || id == "toolbar" {
            return self.action(string(v, "command")?, &json!({}));
        }
        if id == "form" {
            return match action {
                "edit" => {
                    self.form["field"] = json!(v.as_str().ok_or("form edit must be text")?);
                    if self.form_kind == "search" {
                        self.form["choices"] = self.search_choices();
                    }
                    Ok(json!({"ok":true}))
                }
                "answer" => self.answer(string(v, "choice")?, None),
                _ => Err("Unknown form action".into()),
            };
        }
        if id == "panes" {
            self.core_mut()
                .set_split_ratio(v.as_f64().ok_or("split must be numeric")? as f32);
            return Ok(json!({"ok":true}));
        }
        if id == "root" {
            let sidebar = match string(v, "child")? {
                "places-pane" => Sidebar::Places,
                "properties-pane" => Sidebar::Properties,
                _ => return Err("Unknown sidebar".into()),
            };
            self.core_mut().set_sidebar_width(
                sidebar,
                v["fraction"].as_f64().ok_or("fraction must be numeric")? as f32,
            );
            return Ok(json!({"ok":true}));
        }
        if id == "places" {
            if action == "activate-pane" {
                return Ok(json!({"ok":true}));
            }
            let index = string(v, "key")?
                .strip_prefix("place.")
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or("Unknown place")?;
            let path = self
                .core()
                .places()
                .get(index)
                .ok_or("Unknown place")?
                .1
                .clone();
            let p = self.core().active();
            self.core_mut().navigate(p, path);
            return Ok(json!({"ok":true}));
        }
        let p = if id.starts_with("left-") {
            PaneId::Left
        } else if id.starts_with("right-") {
            PaneId::Right
        } else {
            return Err("Unknown widget".into());
        };
        self.core_mut().set_active_pane(p);
        if id.ends_with("-location") {
            if matches!(action, "cancel" | "blur") {
                self.locations[p.index()] = path_text(&self.core().pane(p).path);
                return Ok(json!({"ok":true}));
            }
            let path = v
                .as_str()
                .filter(|s| s.len() <= 4096)
                .ok_or("location must be text")?;
            if action == "submit" {
                self.core_mut().navigate(p, native_path(path)?);
            } else if action != "set" {
                return Err("Unknown location action".into());
            }
            self.locations[p.index()] = path.into();
        } else if id.ends_with("-tree") {
            match action {
                "select" | "activate" | "context" => {
                    let path = native_path(string(v, "key")?)?;
                    if !self
                        .core()
                        .visible_rows(p)
                        .iter()
                        .any(|r| r.entry.path == path)
                    {
                        return Err("Unknown row".into());
                    }
                    if action != "context" || !self.core().pane(p).selected_paths.contains(&path) {
                        self.core_mut().select_modified(
                            p,
                            path.clone(),
                            v["ctrl"] == true,
                            v["shift"] == true,
                        );
                    }
                    if action == "activate" {
                        let folder = self
                            .core()
                            .visible_rows(p)
                            .iter()
                            .any(|r| r.entry.path == path && r.entry.is_dir);
                        if folder {
                            self.core_mut().toggle_expand(p, &path);
                        } else {
                            self.core_mut().open_selection();
                        }
                    }
                }
                "expand" => {
                    let path = native_path(string(v, "key")?)?;
                    if !self
                        .core()
                        .visible_rows(p)
                        .iter()
                        .any(|r| r.entry.path == path && r.entry.is_dir)
                    {
                        return Err("Unknown folder row".into());
                    }
                    self.core_mut().toggle_expand(p, &path);
                }
                "sort" => {
                    return self.action(&format!("view.sort-{}", string(v, "column")?), &json!({}));
                }
                "command" => {
                    if let Some(key) = v["key"].as_str() {
                        let path = native_path(key)?;
                        if !self.core().pane(p).selected_paths.contains(&path) {
                            self.core_mut().select_path(p, Some(path));
                        }
                    }
                    return self.action(string(v, "command")?, &json!({}));
                }
                "activate-pane" => {}
                "navigate" => {
                    let key = string(v, "key")?;
                    match key {
                        "ArrowUp" | "ArrowDown" => self.core_mut().select_relative_modified(
                            p,
                            if key == "ArrowDown" { 1 } else { -1 },
                            v["ctrl"] == true,
                            v["shift"] == true,
                        ),
                        "Home" | "End" => self.core_mut().select_edge_modified(
                            p,
                            key == "End",
                            v["ctrl"] == true,
                            v["shift"] == true,
                        ),
                        "ArrowLeft" | "ArrowRight" => {
                            if let Some(path) = self.core().pane(p).selected.clone() {
                                let expanded = self.core().pane(p).expanded.contains(&path);
                                if (key == "ArrowRight" && !expanded)
                                    || (key == "ArrowLeft" && expanded)
                                {
                                    self.core_mut().toggle_expand(p, &path);
                                }
                            }
                        }
                        _ => return Err("Unknown navigation key".into()),
                    }
                }
                "drop" => {
                    let keys = v["keys"]
                        .as_array()
                        .ok_or("drop sources required")?
                        .iter()
                        .map(|p| native_path(p.as_str().ok_or("drop source must be text")?))
                        .collect::<Result<Vec<_>, _>>()?;
                    let sources = keys
                        .iter()
                        .filter(|path| {
                            !keys
                                .iter()
                                .any(|parent| parent != *path && path.starts_with(parent))
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    let destination = native_path(string(v, "destination")?)?;
                    if !dopus_core::model::file_drop_actions_batch(
                        &sources,
                        &destination,
                        self.core().availability().operation_running,
                    )
                    .contains(DropAction::Copy)
                    {
                        return Err("This destination cannot receive the dragged items".into());
                    }
                    let source_pane = if v["source"]
                        .as_str()
                        .is_some_and(|s| s.starts_with("right-"))
                    {
                        PaneId::Right
                    } else {
                        PaneId::Left
                    };
                    if v["ctrl"] == true || v["shift"] == true {
                        self.core_mut().transfer_paths(
                            source_pane,
                            sources,
                            destination,
                            if v["ctrl"] == true {
                                DropAction::Copy
                            } else {
                                DropAction::Move
                            },
                        )?;
                        return Ok(json!({"ok":true}));
                    }
                    if self.form["open"] == true {
                        return Err("Answer the current dialog first".into());
                    }
                    let mut request = v.clone();
                    request["keys"] = paths_value(sources);
                    let form = json!({"open":true,"title":"Transfer items","body":"Move or copy the dragged items?","field":null,"choices":[{"id":"cancel","label":"Cancel","enabled":true,"cancel":true},{"id":"move","label":"Move","enabled":true},{"id":"copy","label":"Copy","enabled":true}]});
                    self.enqueue_form("drop", 0, form, Some(request))?;
                }
                _ => return Err("Unknown tree action".into()),
            }
        } else {
            return Err("Unknown widget".into());
        }
        for i in 0..2 {
            if id.ends_with("-tree") {
                self.locations[i] = path_text(&self.core().pane(pane(i)).path);
            }
        }
        let events = self.core_mut().tick(Instant::now());
        self.absorb(events);
        self.serial += 1;
        Ok(json!({"ok":true}))
    }
    pub fn call(&mut self, command: &str, args: &Value) -> Result<Value, String> {
        self.focus = None;
        match command {
            "dopus.execute" => self.action(string(args, "id")?, &args["params"]),
            "dopus.intent" => self.intent(args),
            "dopus.dialog" => {
                let object = args
                    .as_object()
                    .ok_or("Dialog arguments must be an object")?;
                if object
                    .keys()
                    .any(|k| !["open", "answer", "name"].contains(&k.as_str()))
                    || (object.contains_key("open") && object.len() != 1)
                    || (object.contains_key("name") && !object.contains_key("answer"))
                {
                    return Err("Invalid dialog arguments".into());
                }
                if let Some(open) = object.get("open") {
                    if open.is_null() {
                        if self.form["open"] == true {
                            self.answer("cancel", None)?;
                        }
                        return Ok(json!({"dialog":self.legacy_view()}));
                    }
                    let kind = open
                        .as_str()
                        .filter(|s| ["about", "shortcuts"].contains(s))
                        .ok_or("Unknown dialog kind")?;
                    if self.form["open"] == true {
                        return Err("DIALOG_OPEN: answer the open dialog first".into());
                    }
                    self.action(&format!("help.{kind}"), &json!({}))?;
                    return Ok(json!({"dialog":self.legacy_view()}));
                }
                if let Some(answer) = object.get("answer") {
                    if self.form["open"] != true {
                        return Err("NO_DIALOG: no dialog is open".into());
                    }
                    let answer = answer.as_str().ok_or("Answer must be text")?;
                    let choice = match self.form_kind.as_str() {
                        "delete" => match answer {
                            "yes" => "ok",
                            "no" => "cancel",
                            _ => return Err("Unknown delete choice".into()),
                        },
                        "message" => {
                            if answer == "done" {
                                "ok"
                            } else {
                                return Err("Expected done".into());
                            }
                        }
                        _ => {
                            if !self.form["choices"].as_array().is_some_and(|a| {
                                a.iter().any(|c| c["id"] == answer && c["enabled"] == true)
                            }) {
                                return Err("Unknown dialog choice".into());
                            }
                            answer
                        }
                    };
                    let name = if let Some(name) = object.get("name") {
                        if !["mkdir", "rename"].contains(&self.form_kind.as_str()) {
                            return Err("Dialog has no name field".into());
                        }
                        Some(name.as_str().ok_or("Name must be text")?)
                    } else {
                        None
                    };
                    self.answer(choice, name)?;
                    return Ok(json!({"dialog":self.legacy_view()}));
                }
                Ok(json!({"dialog":self.legacy_view()}))
            }
            "dopus.dialog.answer" => {
                if args.get("instance") != self.form.get("instance") {
                    return Err(
                        "STALE_FORM: read the current dialog instance before answering".into(),
                    );
                }
                if args.get("text").is_some_and(|v| !v.is_string()) {
                    return Err("Dialog text must be text".into());
                }
                self.answer(string(args, "choice")?, args["text"].as_str())
            }
            "dopus.open" => {
                let paths = args["paths"]
                    .as_array()
                    .filter(|p| !p.is_empty())
                    .ok_or("paths must contain a location")?;
                if paths.iter().any(|p| p.as_str().is_none_or(str::is_empty)) {
                    return Err("paths must contain non-empty text".into());
                }
                for (i, path) in paths.iter().take(2).enumerate() {
                    self.core_mut().navigate(
                        pane(i),
                        native_path(path.as_str().ok_or("path must be text")?)?,
                    );
                    self.locations[i] = path_text(&self.core().pane(pane(i)).path);
                }
                Ok(json!({"ok":true}))
            }
            "dopus.theme.set" => {
                if let Some(mode) = args.get("mode") {
                    if ![json!("dark"), json!("light")].contains(mode) {
                        return Err("Unknown theme mode".into());
                    }
                }
                if let Some(scheme) = args.get("scheme") {
                    if self.contract["schemes"]
                        .get(scheme.as_str().ok_or("scheme must be text")?)
                        .is_none()
                    {
                        return Err("Unknown colour scheme".into());
                    }
                }
                if let Some(style) = args.get("style") {
                    if !self.contract["styles"].as_array().unwrap().contains(style) {
                        return Err("Unknown widget style".into());
                    }
                }
                if let Some(mode) = args.get("mode") {
                    self.appearance["mode"] = mode.clone();
                }
                if let Some(scheme) = args.get("scheme") {
                    self.appearance["scheme"] = scheme.clone();
                }
                if let Some(style) = args.get("style") {
                    self.appearance["style"] = style.clone();
                }
                let mut answer =
                    json!({"scheme":self.appearance["scheme"],"mode":self.appearance["mode"]});
                if args.get("style").is_some() {
                    answer["style"] = self.appearance["style"].clone();
                }
                for key in ["mode", "scheme", "style"] {
                    if args.get(key).is_some() {
                        self.appearance_choice[key] = self.appearance[key].clone();
                    }
                }
                Ok(answer)
            }
            "dopus.session" => {
                self.window = args
                    .get("window")
                    .filter(|v| v.is_object())
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let mut appearance = json!({"mode":"dark","scheme":"studio","style":"default"});
                for (key, value) in args["appearance"]
                    .as_object()
                    .ok_or("session appearance must be an object")?
                {
                    appearance[key] = value.clone();
                }
                self.appearance_session = appearance.clone();
                for (key, value) in self.appearance_choice.as_object().unwrap() {
                    appearance[key] = value.clone();
                }
                viu::appearance::resolve(&appearance)?;
                self.appearance = appearance;
                for action in self.contract["actions"].as_array_mut().unwrap() {
                    let id = action["id"].as_str().unwrap();
                    action["keys"] = args["keymap"]
                        .get(id)
                        .unwrap_or(&self.default_keys[id])
                        .clone();
                }
                self.serial += 1;
                Ok(json!({"ok":true}))
            }
            "dopus.quit" | "dopus.persist" => {
                self.core_mut().flush_config();
                Ok(json!({"ok":true}))
            }
            _ => Err("UNKNOWN_COMMAND".into()),
        }
    }
    fn properties(&self) -> Value {
        match self.core().properties(self.core().active()) {
            Properties::Folder { path, summary } => {
                json!([{"label":"Folder","value":path},{"label":"Contents","value":summary}])
            }
            Properties::Entry {
                entry,
                metadata,
                count_pending,
            } => {
                let metadata = match metadata {
                    Some(Ok(metadata)) => metadata,
                    value => {
                        return json!([{"label":"Name","value":dopus_core::sanitise_display_text(&entry.name)},if let Some(Err(error))=value{json!({"label":"Error","value":error})}else{json!({"label":"Kind","value":"…"})},{"label":"Path","value":dopus_core::sanitise_display_path(&entry.path)}]);
                    }
                };
                let mut fields = vec![
                    json!({"label":"Name","value":dopus_core::sanitise_display_text(&entry.name)}),
                    json!({"label":"Kind","value":metadata.kind}),
                    json!({"label":"Size","value":if entry.is_dir{if count_pending{"…".into()}else{dopus_core::format_child_count(entry.child_count)}}else{dopus_core::format_size(metadata.size)}}),
                    json!({"label":"Modified","value":date(metadata.modified)}),
                    json!({"label":"Created","value":date(metadata.created)}),
                    json!({"label":"Accessed","value":date(metadata.accessed)}),
                    json!({"label":"Permissions","value":metadata.permissions}),
                    json!({"label":"Owner and group","value":metadata.owner_group}),
                ];
                if let Some(target) = metadata.symlink_target {
                    fields.push(json!({"label":"Link target","value":target}));
                }
                fields.push(
                    json!({"label":"Path","value":dopus_core::sanitise_display_path(&entry.path)}),
                );
                json!(fields)
            }
        }
    }
    fn legacy_view(&self) -> Value {
        if self.form["open"] != true {
            return Value::Null;
        }
        let kind = match self.form_kind.as_str() {
            "mkdir" | "rename" => "name",
            "message" => self.form["legacy_kind"].as_str().unwrap_or("message"),
            other => other,
        };
        let buttons=match self.form_kind.as_str(){
            "delete"=>json!([{"id":"yes","label":"Yes","role":"default"},{"id":"no","label":"No","role":"cancel"}]),
            "message"=>json!([{"id":"done","label":"Done","role":"default"}]),
            _=>json!(self.form["choices"].as_array().into_iter().flatten().map(|c|json!({"id":c["id"],"label":c["label"],"role":if c["cancel"]==true{"cancel"}else if c["default"]==true{"default"}else{"other"}})).collect::<Vec<_>>())
        };
        let mut value =
            json!({"kind":kind,"title":self.form["title"],"buttons":buttons,"status":self.info});
        if self.form["field"].is_string() {
            value["name"] = self.form["field"].clone();
        } else {
            value["body"] = self.form["body"].clone();
        }
        if self.form_kind == "drop"
            && let Some(request) = &self.drop_request
        {
            value["paths"] = request["keys"].clone();
            value["path"] = request["destination"].clone();
        }
        value
    }
    fn window_settings(&self) -> Value {
        let mut settings = self.window.clone();
        for (key, value) in self.window_choice.as_object().unwrap() {
            if value != "auto" {
                settings[key] = value.clone();
            }
        }
        settings
    }
    pub fn model(&self) -> Value {
        let window = self.window_settings();
        let side = |key: &str, fallback: &str| {
            window[key]
                .as_str()
                .filter(|v| ["left", "center", "right"].contains(v))
                .unwrap_or(fallback)
                .to_owned()
        };
        let title_align = side(
            "window.title_align",
            if window["window.style"] == "win11" {
                "left"
            } else {
                "center"
            },
        );
        let caption_side = side("window.caption_side", "right");
        let compact = [json!("button"), json!("titlebar")].contains(&window["window.menu_style"]);
        let mut header_groups = json!({"menu":{"preferred":if compact{36}else{180},"minimum":if compact{36}else{120},"align":side("window.menu_side","left"),"shrink":1,"order":0},"toolbar":{"preferred":280,"minimum":80,"align":side("window.actions_align","right"),"shrink":1,"order":2},"caption-title":{"preferred":"content","minimum":0,"align":title_align,"shrink":0,"order":1},"window":{"preferred":"window.captions","minimum":"window.captions","align":caption_side,"shrink":null,"order":3}});
        if window["viu.hosted-menu"] == true {
            header_groups["menu"]["preferred"] = json!(0);
            header_groups["menu"]["minimum"] = json!(0);
        }
        if window["viu.hosted-actions"] == true {
            header_groups["toolbar"]["preferred"] = json!(0);
            header_groups["toolbar"]["minimum"] = json!(0);
        }
        let caption_control = json!({"id":"caption-title","kind":"label","label":"","initial":"","properties":{"align":title_align,"emphasis":true,"truncate":true}});
        let core = self.core();
        let config = core.config_snapshot();
        let mut values = json!({"caption-title":dopus_core::sanitise_display_path(&core.pane(core.active()).path),"window":{"side":caption_side,"enabled":true},"menu":self.menus(),"toolbar":self.toolbar(),"status":self.info,"properties":self.properties(),"form":self.form});
        values["form"]["dialog"] = self.legacy_view();
        let mut panes = Vec::new();
        let context = self.contract["context"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| self.command(id.as_str().unwrap()))
            .collect::<Vec<_>>();
        let background = self.contract["background_context"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| self.command(id.as_str().unwrap()))
            .collect::<Vec<_>>();
        for i in 0..2 {
            let p = core.pane(pane(i));
            let side = if i == 0 { "left" } else { "right" };
            let rows=core.visible_rows(pane(i)).iter().map(|r|{let e=&r.entry;let expanded=p.expanded.contains(&e.path);json!({"key":path_value(&e.path),"cells":[dopus_core::sanitise_display_text(&e.name),if e.is_dir{dopus_core::format_child_count(e.child_count)}else{e.size.map(dopus_core::format_size).unwrap_or_else(||"—".into())},date(e.modified)],"depth":r.depth,"branch":e.is_dir,"expanded":expanded,"selected":false,"focused":false,"icon":icon(&e.path,e.is_dir,expanded)})}).collect::<Vec<_>>();
            values[format!("{side}-location")] = json!(self.locations[i]);
            values[format!("{side}-summary")] = json!(p.footer_summary());
            values[format!("{side}-tree")] = json!({"key":path_value(&p.path),"revision":p.listing_revision,"active":core.active()==pane(i),"columns":self.contract["columns"],"sort":p.sort,"ascending":p.ascending,"rows":rows,"selected_keys":paths_value(&p.selected_paths),"focused_key":p.selected.as_deref().map(path_value),"context":context,"background_context":background});
            let mut expanded = p.expanded.iter().collect::<Vec<_>>();
            expanded.sort();
            panes.push(json!({"path":path_value(&p.path),"generation":p.generation,"show_hidden":p.show_hidden,"previews":p.previews,"sort":p.sort,"ascending":p.ascending,"selected":p.selected.as_deref().map(path_value),"selected_paths":paths_value(core.selected_paths(pane(i))),"expanded":paths_value(expanded),"summary":p.footer_summary(),"rows":rows.len(),"error":null}));
        }
        values["places"] = json!({"key":"places","revision":0,"active":false,"columns":[{"id":"name","label":"Places"}],"rows":core.places().iter().enumerate().map(|(i,(name,path))|json!({"key":format!("place.{i}"),"path":path_value(path),"cells":[name],"depth":0,"branch":false,"expanded":false,"selected":core.pane(core.active()).path==*path,"focused":false,"icon":match name.as_str(){"Home"=>"house","Filesystem"=>"hard-drive","Documents"=>"file-text","Downloads"=>"download","Music"=>"music","Pictures"=>"image","Videos"=>"film",_=>"folder"}})).collect::<Vec<_>>(),"context":[],"background_context":[]});
        values["preview"] = match core.preview(core.active()) {
            Some(dopus_core::thumbs::Preview::Ready(path)) => {
                json!({"phase":"ready","path":path_value(&path)})
            }
            Some(dopus_core::thumbs::Preview::Pending) => json!({"phase":"pending"}),
            _ => json!({"phase":"none"}),
        };
        json!({"window_settings":window,"topbar_hidden":window["viu.hosted-menu"]==true && window["viu.hosted-actions"]==true,"actions":self.actions()["actions"],"values":values,"panes":panes,"active":core.active().index(),"split":config.split_ratio,"header_groups":header_groups,"caption_control":caption_control,"pane_outlines":{"left":if core.active()==PaneId::Left{json!("design.accent")}else{Value::Null},"right":if core.active()==PaneId::Right{json!("design.accent")}else{Value::Null}},"layout":{"form":{"edge":"top","size":0},"topbar":{"edge":"top","size":34},"status-pane":{"edge":"bottom","size":32},"places-pane":{"edge":"left","fraction":config.places.width,"resizable":true,"min_fraction":0.1,"max_fraction":0.3,"minimum_window_width":640},"properties-pane":{"edge":"right","fraction":config.properties.width,"resizable":true,"min_fraction":0.1,"max_fraction":0.3,"minimum_window_width":640},"center-pane":{"edge":"fill"}},"places_hidden":!config.places.open,"properties_hidden":!config.properties.open,"appearance":self.appearance,"busy":core.availability().operation_running,"serial":self.serial,"focus":self.focus,"info":self.info,"ready":!core.pane(PaneId::Left).listing&&!core.pane(PaneId::Right).listing})
    }
}
impl Drop for Native {
    fn drop(&mut self) {
        self.core.take();
        self.rx.take();
        if let Some(thread) = self.relay.take() {
            let _ = thread.join();
        }
    }
}
