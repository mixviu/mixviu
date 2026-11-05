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
    pub appearance: Value,
    form: Value,
    form_kind: String,
    token: u64,
    drop_request: Option<Value>,
    locations: [String; 2],
    pub serial: u64,
    pub focus: Option<String>,
    info: String,
    pub effects: VecDeque<Value>,
}
fn closed() -> Value {
    json!({"open":false,"title":"","body":"","field":null,"choices":[],"instance":0})
}
fn string<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key].as_str().ok_or_else(|| format!("{key} must be text"))
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
        let locations = [
            config.left.path.to_string_lossy().into_owned(),
            config.right.path.to_string_lossy().into_owned(),
        ];
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
            contract,
            appearance: json!({"mode":"dark","scheme":"ocean","style":"studio","body":14.666667}),
            form: closed(),
            form_kind: String::new(),
            token: 0,
            drop_request: None,
            locations,
            serial: 0,
            focus: None,
            info: "Ready".into(),
            effects: VecDeque::new(),
        }
    }
    fn core(&self) -> &DopusCore {
        self.core.as_ref().unwrap()
    }
    fn core_mut(&mut self) -> &mut DopusCore {
        self.core.as_mut().unwrap()
    }
    fn absorb(&mut self, events: Vec<CoreEvent>) {
        for event in events {
            match event {
                CoreEvent::PromptRequested {
                    token,
                    kind,
                    initial,
                } => {
                    self.token = token;
                    self.form_kind = match kind {
                        PromptKind::NewFolder => "mkdir",
                        PromptKind::Rename => "rename",
                    }
                    .into();
                    self.form = json!({"open":true,"title":if kind==PromptKind::NewFolder{"New folder"}else{"Rename"},"body":"","field":initial,"instance":token,"error":"","choices":[{"id":"cancel","label":"Cancel","enabled":true,"cancel":true},{"id":"ok","label":"OK","enabled":true,"default":true}]});
                }
                CoreEvent::ConfirmRequested { token, message } => {
                    self.token = token;
                    self.form_kind = "delete".into();
                    self.form = json!({"open":true,"title":"Delete permanently","body":message,"field":null,"instance":token,"error":"","choices":[{"id":"cancel","label":"Cancel","enabled":true,"cancel":true},{"id":"ok","label":"Delete","enabled":true,"default":true}]});
                }
                CoreEvent::Status {
                    kind: dopus_core::StatusKind::Message,
                    text,
                    ..
                } => self.info = text,
                CoreEvent::OpenFile(path) => {
                    self.effects.push_back(json!({"kind":"open","path":path}))
                }
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
            let actual = self
                .core()
                .pane(pane(i))
                .path
                .to_string_lossy()
                .into_owned();
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
            _ => false,
        });
        a
    }
    pub fn actions(&self) -> Value {
        json!({"actions":self.contract["actions"].as_array().unwrap().iter().map(|a|self.command(a["id"].as_str().unwrap())).collect::<Vec<_>>()})
    }
    fn menus(&self) -> Value {
        let mut menus = self.contract["menus"].clone();
        for m in menus.as_array_mut().unwrap() {
            for a in m["children"].as_array_mut().unwrap() {
                *a = self.command(a["id"].as_str().unwrap());
            }
        }
        menus
    }
    pub fn action(&mut self, id: &str, params: &Value) -> Result<Value, String> {
        if let Some(target) = params.get("pane") {
            let p = match target.as_str() {
                Some("left") => PaneId::Left,
                Some("right") => PaneId::Right,
                Some("active") => self.core().active(),
                _ => return Err("Unknown pane".into()),
            };
            self.core_mut().set_active_pane(p);
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
        if !self.enabled(id) {
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
                .navigate(p, PathBuf::from(string(params, "path")?)),
            "location.focus" => {
                self.focus = Some(format!(
                    "{}-location",
                    if i == 0 { "left" } else { "right" }
                ))
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
                self.effects.push_back(json!({"kind":"mail","paths":paths}));
            }
            "theme.mode-toggle" => {
                self.appearance["mode"] = json!(if self.appearance["mode"] == "dark" {
                    "light"
                } else {
                    "dark"
                })
            }
            id if id.starts_with("theme.scheme-") => self.appearance["scheme"] = json!(&id[13..]),
            "help.about" | "help.shortcuts" => {
                self.form_kind = "message".into();
                self.token += 1;
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
                self.form = json!({"open":true,"title":if id=="help.about"{"About Dopus"}else{"Keyboard shortcuts"},"body":body,"field":null,"instance":self.token,"choices":[{"id":"cancel","label":"Cancel","enabled":true,"cancel":true},{"id":"ok","label":"OK","enabled":true,"default":true}]});
            }
            "search" => {
                self.focus = Some(format!(
                    "{}-location",
                    if i == 0 { "left" } else { "right" }
                ))
            }
            "app.quit" => {
                self.core_mut().flush_config();
            }
            _ => return Err("UNKNOWN_ACTION".into()),
        }
        for i in 0..2 {
            self.locations[i] = self
                .core()
                .pane(pane(i))
                .path
                .to_string_lossy()
                .into_owned();
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
                .map(PathBuf::from)
                .collect();
            let destination = PathBuf::from(string(request, "destination")?);
            let source_pane = self.core().active();
            self.core_mut()
                .transfer_paths(source_pane, sources, destination, action)?;
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
        self.form = closed();
        self.drop_request = None;
        let events = self.core_mut().tick(Instant::now());
        self.absorb(events);
        Ok(json!({"ok":true}))
    }
    pub fn intent(&mut self, a: &Value) -> Result<Value, String> {
        let id = string(a, "id")?;
        let action = string(a, "action")?;
        let v = &a["value"];
        if id == "menu" {
            return self.action(string(v, "command")?, &json!({}));
        }
        if id == "form" {
            return match action {
                "edit" => {
                    self.form["field"] = json!(v.as_str().ok_or("form edit must be text")?);
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
                "places" => Sidebar::Places,
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
            let path = v
                .as_str()
                .filter(|s| s.len() <= 4096)
                .ok_or("location must be text")?;
            if action == "submit" {
                self.core_mut().navigate(p, path.into());
            } else if action != "set" {
                return Err("Unknown location action".into());
            }
            self.locations[p.index()] = path.into();
        } else if id.ends_with("-tree") {
            match action {
                "select" | "activate" | "context" => {
                    let path = PathBuf::from(string(v, "key")?);
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
                            path,
                            v["ctrl"] == true,
                            v["shift"] == true,
                        );
                    }
                    if action == "activate" {
                        self.core_mut().open_selection();
                    }
                }
                "expand" => {
                    let path = PathBuf::from(string(v, "key")?);
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
                        let path = PathBuf::from(key);
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
                    self.drop_request = Some(v.clone());
                    self.form_kind = "drop".into();
                    self.token += 1;
                    self.form = json!({"open":true,"title":"Transfer items","body":"Move or copy the dragged items?","field":null,"instance":self.token,"choices":[{"id":"cancel","label":"Cancel","enabled":true,"cancel":true},{"id":"move","label":"Move","enabled":true},{"id":"copy","label":"Copy","enabled":true}]});
                }
                _ => return Err("Unknown tree action".into()),
            }
        } else {
            return Err("Unknown widget".into());
        }
        for i in 0..2 {
            if id.ends_with("-tree") {
                self.locations[i] = self
                    .core()
                    .pane(pane(i))
                    .path
                    .to_string_lossy()
                    .into_owned();
            }
        }
        let events = self.core_mut().tick(Instant::now());
        self.absorb(events);
        self.serial += 1;
        Ok(json!({"ok":true}))
    }
    pub fn call(&mut self, command: &str, args: &Value) -> Result<Value, String> {
        match command {
            "dopus.execute" => self.action(string(args, "id")?, &args["params"]),
            "dopus.intent" => self.intent(args),
            "dopus.dialog.answer" => self.answer(string(args, "choice")?, args["text"].as_str()),
            "dopus.open" => {
                let paths = args["paths"]
                    .as_array()
                    .filter(|p| !p.is_empty())
                    .ok_or("paths must contain a location")?;
                for (i, path) in paths.iter().take(2).enumerate() {
                    self.core_mut().navigate(
                        pane(i),
                        PathBuf::from(path.as_str().ok_or("path must be text")?),
                    );
                    self.locations[i] = self
                        .core()
                        .pane(pane(i))
                        .path
                        .to_string_lossy()
                        .into_owned();
                }
                Ok(json!({"ok":true}))
            }
            "dopus.theme.set" => {
                if let Some(mode) = args.get("mode") {
                    if ![json!("dark"), json!("light")].contains(mode) {
                        return Err("Unknown theme mode".into());
                    }
                    self.appearance["mode"] = mode.clone();
                }
                if let Some(scheme) = args.get("scheme") {
                    if self.contract["schemes"]
                        .get(scheme.as_str().ok_or("scheme must be text")?)
                        .is_none()
                    {
                        return Err("Unknown colour scheme".into());
                    }
                    self.appearance["scheme"] = scheme.clone();
                }
                Ok(self.appearance.clone())
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
                entry, metadata, ..
            } => {
                let Some(Ok(metadata)) = metadata else {
                    return json!([{"label":"Name","value":dopus_core::sanitise_display_text(&entry.name)},{"label":"Metadata","value":"Loading…"}]);
                };
                let mut fields = vec![
                    json!({"label":"Name","value":dopus_core::sanitise_display_text(&entry.name)}),
                    json!({"label":"Type","value":metadata.kind}),
                    json!({"label":"Location","value":dopus_core::sanitise_display_path(entry.path.parent().unwrap_or(Path::new("/")))}),
                    json!({"label":"Size","value":dopus_core::format_size(metadata.size)}),
                    json!({"label":"Modified","value":date(metadata.modified)}),
                    json!({"label":"Created","value":date(metadata.created)}),
                    json!({"label":"Accessed","value":date(metadata.accessed)}),
                    json!({"label":"Permissions","value":metadata.permissions}),
                    json!({"label":"Owner","value":metadata.owner_group}),
                ];
                if let Some(target) = metadata.symlink_target {
                    fields.push(json!({"label":"Target","value":target}));
                }
                json!(fields)
            }
        }
    }
    pub fn model(&self) -> Value {
        let core = self.core();
        let config = core.config_snapshot();
        let mut values = json!({"menu":self.menus(),"status":self.info,"properties":self.properties(),"form":self.form});
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
            let rows=core.visible_rows(pane(i)).iter().map(|r|{let e=&r.entry;let expanded=p.expanded.contains(&e.path);json!({"key":e.path,"cells":[dopus_core::sanitise_display_text(&e.name),if e.is_dir{dopus_core::format_child_count(e.child_count)}else{e.size.map(dopus_core::format_size).unwrap_or_else(||"—".into())},date(e.modified)],"depth":r.depth,"branch":e.is_dir,"expanded":expanded,"selected":p.selected_paths.contains(&e.path),"focused":p.selected.as_ref()==Some(&e.path),"icon":icon(&e.path,e.is_dir,expanded)})}).collect::<Vec<_>>();
            values[format!("{side}-location")] = json!(self.locations[i]);
            values[format!("{side}-summary")] = json!(p.footer_summary());
            values[format!("{side}-tree")] = json!({"key":p.path,"revision":p.listing_revision,"active":core.active()==pane(i),"columns":self.contract["columns"],"sort":p.sort,"ascending":p.ascending,"rows":rows,"context":context,"background_context":background});
            let mut expanded = p.expanded.iter().collect::<Vec<_>>();
            expanded.sort();
            panes.push(json!({"path":p.path,"generation":p.generation,"show_hidden":p.show_hidden,"previews":p.previews,"sort":p.sort,"ascending":p.ascending,"selected":p.selected,"selected_paths":core.selected_paths(pane(i)),"expanded":expanded,"summary":p.footer_summary(),"rows":rows.len(),"error":null}));
        }
        values["places"] = json!({"key":"places","revision":0,"active":false,"columns":[{"id":"name","label":"Places"}],"rows":core.places().iter().enumerate().map(|(i,(name,path))|json!({"key":format!("place.{i}"),"path":path,"cells":[name],"depth":0,"branch":false,"expanded":false,"selected":core.pane(core.active()).path==*path,"focused":false,"icon":match name.as_str(){"Home"=>"house","Filesystem"=>"hard-drive","Documents"=>"file-text","Downloads"=>"download","Music"=>"music","Pictures"=>"image","Videos"=>"film",_=>"folder"}})).collect::<Vec<_>>(),"context":[],"background_context":[]});
        values["preview"] = match core.preview(core.active()) {
            Some(dopus_core::thumbs::Preview::Ready(path)) => json!({"phase":"ready","path":path}),
            Some(dopus_core::thumbs::Preview::Pending) => json!({"phase":"pending"}),
            _ => json!({"phase":"none"}),
        };
        json!({"values":values,"panes":panes,"active":core.active().index(),"split":config.split_ratio,"layout":{"form":{"edge":"top","size":0},"menu":{"edge":"top","size":28},"status":{"edge":"bottom","size":24},"places":{"edge":"left","fraction":config.places.width,"resizable":true,"minimum":0.1,"maximum":0.4},"properties-pane":{"edge":"right","fraction":config.properties.width,"resizable":true,"minimum":0.1,"maximum":0.4,"minimum_window_width":700},"panes":{"edge":"fill"}},"places_hidden":!config.places.open,"properties_hidden":!config.properties.open,"appearance":self.appearance,"busy":core.availability().operation_running,"serial":self.serial,"focus":self.focus,"info":self.info,"ready":!core.pane(PaneId::Left).listing&&!core.pane(PaneId::Right).listing})
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
