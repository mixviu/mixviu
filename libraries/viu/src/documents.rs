// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reusable in-process document/mirror facility. It performs no transport,
//! storage, tab or session policy. Mix explicitly drains and delivers effects.
use editor::{Doc, EditCommand, Event, Selection, View};
use editor_model::{
    highlight::Highlight,
    mirror::{LaneArg, Mirror, Phase, ServerOp, Step},
    model::{EditCfg, EditorModel, line_comment_for},
    types::{Intent, OpIdGen, Outgoing},
};
use serde_json::{Value, json};
mod tools;
use std::{
    collections::{BTreeMap, VecDeque},
    hash::{BuildHasher, Hasher},
    sync::{Arc, Mutex},
};

struct Document {
    preview: Option<(u64, Arc<toolkit::markdown::Document>)>,
    diagnostics: editor_model::diag::Diagnostics,
    matches: Vec<std::ops::Range<usize>>,
    resyncs: u64,
    mirror: Mirror,
    model: EditorModel,
    highlight: Highlight,
    identity: u64,
    out: VecDeque<Outgoing>,
    notices: Vec<String>,
    layout: editor::LayoutReport,
}
struct State {
    counters: BTreeMap<&'static str, u64>,
    documents: BTreeMap<String, Document>,
    ids: OpIdGen,
    next: u64,
}
impl Default for State {
    fn default() -> Self {
        let run = std::collections::hash_map::RandomState::new()
            .build_hasher()
            .finish() as u32;
        Self {
            counters: [
                "events",
                "history_recoveries",
                "snapshot_recoveries",
                "conflicts",
                "retries",
                "uncertain",
            ]
            .into_iter()
            .map(|k| (k, 0))
            .collect(),
            documents: BTreeMap::new(),
            ids: OpIdGen::new(run),
            next: 0,
        }
    }
}
#[derive(Clone, Default)]
pub struct Documents(Arc<Mutex<State>>);

/// Mix numbers arrive as JSON floats. Admit exact, safe whole numbers only.
fn normalize(value: Value) -> Result<Value, String> {
    Ok(match value {
        Value::Number(n) if n.is_f64() => {
            let f = n.as_f64().unwrap();
            if f.fract() == 0.0 {
                if f.abs() > 9_007_199_254_740_991.0 {
                    return Err("integer exceeds Mix's exact range".into());
                }
                if f < 0.0 {
                    json!(f as i64)
                } else {
                    json!(f as u64)
                }
            } else {
                Value::Number(n)
            }
        }
        Value::Array(a) => Value::Array(a.into_iter().map(normalize).collect::<Result<_, _>>()?),
        Value::Object(a) => Value::Object(
            a.into_iter()
                .map(|(k, v)| Ok((k, normalize(v)?)))
                .collect::<Result<_, String>>()?,
        ),
        v => v,
    })
}
fn text_arg<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key]
        .as_str()
        .ok_or_else(|| format!("{key} requires text"))
}
fn decode<T: serde::de::DeserializeOwned>(v: &Value) -> Result<T, String> {
    serde_json::from_value(normalize(v.clone())?).map_err(|e| e.to_string())
}
fn point(text: &edit::text::Text, v: &Value) -> Result<usize, String> {
    if v == "start" {
        return Ok(0);
    }
    if v == "end" {
        return Ok(text.len());
    }
    let offset = if let Some(n) = crate::widgets::whole(v) {
        n as usize
    } else {
        let line = crate::widgets::whole(&v["line"]).ok_or("line required")? as usize;
        let col = crate::widgets::whole(&v["col"]).unwrap_or(1) as usize;
        let range = text.line_range(line).ok_or("line out of range")?;
        let mut s = String::new();
        text.read(range.clone(), &mut s);
        let content = s.strip_suffix('\n').unwrap_or(&s);
        if col == 0 || col > content.chars().count() + 1 {
            return Err("column out of range".into());
        }
        range.start
            + content
                .char_indices()
                .nth(col - 1)
                .map_or(content.len(), |(i, _)| i)
    };
    if offset > text.len() || !text.is_char_boundary(offset) {
        return Err("offset is outside text or a UTF-8 boundary".into());
    }
    Ok(offset)
}
fn scalar_offset(text: &edit::text::Text, mut n: usize) -> usize {
    let mut offset = 0;
    while offset < text.len() {
        let chunk = std::str::from_utf8(text.chunk_at(offset)).expect("UTF-8 text chunks");
        for (i, _) in chunk.char_indices() {
            if n == 0 {
                return offset + i;
            }
            n -= 1;
        }
        offset += chunk.len();
    }
    text.len()
}
impl Document {
    fn step(&mut self, step: Step) {
        for delta in step.deltas {
            if delta.kind == editor_model::types::DeltaKind::Resync {
                self.resyncs += 1;
            }
            self.diagnostics.apply_delta(&delta);
            self.matches.clear();
            self.model.apply_delta(&delta);
            self.highlight.apply_delta(self.mirror.text(), &delta);
        }
        self.model.clamp(self.mirror.text());
        self.model.remote = self
            .mirror
            .remote_cursors()
            .iter()
            .filter_map(|(origin, sels)| origin.parse().ok().map(|origin| (origin, sels.clone())))
            .collect();
        self.out.extend(step.out);
        self.notices
            .extend(step.notices.into_iter().map(|n| format!("{n:?}")));
        if self.notices.len() > 64 {
            self.notices.drain(..self.notices.len() - 64);
        }
    }
    fn state(&self, key: &str) -> Value {
        let m = &self.mirror;
        json!({"document":key,"buffer":m.buffer(),"epoch":m.epoch(),"rev":m.rev(),"view_gen":m.view_gen(),
            "phase":match m.phase(){Phase::Live=>"live",Phase::Bootstrapping{..}=>"bootstrapping",Phase::Recovering{..}=>"recovering",Phase::Detached{..}=>"detached"},
            "reattach_needed":matches!(m.phase(),Phase::Detached{reason:editor_model::mirror::DetachReason::EpochChanged}),
            "bytes":m.text().len(),"lines":m.text().line_count(),"path":m.meta().path,"language":m.meta().language,
            "dirty":m.meta().dirty,"pending":m.pending(),"inflight":m.inflight().is_some(),"idle":m.is_idle(),"resyncs":self.resyncs,"has_remote":m.last_remote().is_some(),
            "recovered":m.meta().recovered,"recovery_id":m.meta().recovery_id,"saved_rev":m.meta().saved_rev,"disk":m.meta().disk,"bom":m.meta().bom,"eol":m.meta().eol,
            "selection":{"anchor":m.text().point(self.model.sel.anchor),"head":m.text().point(self.model.sel.head)},
            "overwrite":self.model.overwrite,"first_line":self.model.scroll.first_line,"detached_copy":m.detached_copy().is_some(),"conflicts":m.conflicts().len(),"layout":self.layout,"notices":self.notices})
    }
    fn command(
        &mut self,
        c: EditCommand,
        view: &View,
        ids: &mut OpIdGen,
        intent: Intent,
    ) -> Result<(), String> {
        let cfg = EditCfg {
            measure: view.measure(),
            insert_spaces: !matches!(self.mirror.meta().language.as_str(), "go" | "makefile"),
            eol: if self.mirror.meta().eol == edit::buffer::Eol::Crlf {
                "\r\n"
            } else {
                "\n"
            },
            line_comment: line_comment_for(&self.mirror.meta().language),
        };
        if let Some(edit) = self.model.command(self.mirror.text(), &cfg, c) {
            let caret = edit.caret_after;
            let step = self
                .mirror
                .local_edit(edit, intent, ids)
                .map_err(|e| format!("{e:?}"))?;
            self.step(step);
            self.model.sel = caret;
        }
        Ok(())
    }
}
impl Documents {
    pub fn binding(&self) -> mix::evaluator::ExtFn {
        let docs = self.clone();
        mix::sync_ext(move |args| {
            let fail = |msg| mix::MixError::RuntimeError { msg, span: None };
            let [
                mix::value::Value::String(op),
                mix::value::Value::String(key),
                body,
            ] = args.as_slice()
            else {
                return Err(fail(
                    "viu_document expects operation, document id, arguments".into(),
                ));
            };
            crate::runtime::from_json(
                docs.call(op, key, crate::runtime::to_json(body)?)
                    .map_err(fail)?,
            )
        })
    }
    pub fn call(&self, op: &str, key: &str, a: Value) -> Result<Value, String> {
        if key.is_empty() || key.len() > 128 {
            return Err("invalid document id".into());
        }
        if op == "lex" {
            let request = {
                let mut state = self.0.lock().map_err(|_| "document owner poisoned")?;
                let Some(doc) = state.documents.get_mut(key) else {
                    return Ok(Value::Null);
                };
                let m = &doc.mirror;
                let tag = editor_model::highlight::ResultTag {
                    epoch: m.epoch().into(),
                    buffer: m.buffer().into(),
                    view_gen: m.view_gen(),
                    language: m.meta().language.clone(),
                    cfg: 0,
                };
                doc.highlight.mix_request(m.text(), tag)
            };
            if let Some((tag, source)) = request {
                let spans = editor_model::highlight::run_mix(&tag.language, &source);
                let mut state = self.0.lock().map_err(|_| "document owner poisoned")?;
                if let Some(doc) = state.documents.get_mut(key) {
                    doc.highlight.mix_result(tag, spans);
                }
            }
            return Ok(Value::Null);
        }
        if op == "preview" {
            let (text, generation) = {
                let state = self.0.lock().map_err(|_| "document owner poisoned")?;
                let doc = state.documents.get(key).ok_or("unknown document")?;
                let mut text = String::new();
                doc.mirror
                    .text()
                    .read(0..doc.mirror.text().len(), &mut text);
                (text, doc.mirror.view_gen())
            };
            let preview = Arc::new(toolkit::markdown::Document::parse(&text));
            let mut state = self.0.lock().map_err(|_| "document owner poisoned")?;
            let doc = state
                .documents
                .get_mut(key)
                .ok_or("document closed while parsing")?;
            if doc.mirror.view_gen() != generation {
                return Err("document changed while parsing".into());
            }
            doc.preview = Some((generation, preview));
            return Ok(json!({"view_gen":generation}));
        }
        let mut state = self.0.lock().map_err(|_| "document owner poisoned")?;
        if op == "stats" {
            return Ok(json!(state.counters));
        }
        if let Some(counter) = match op {
            "event" => Some("events"),
            "retry" => Some("retries"),
            "deadline" => Some("uncertain"),
            _ => None,
        } {
            *state.counters.entry(counter).or_default() += 1;
        }
        if op == "open" || op == "reset" {
            if op == "open" && state.documents.contains_key(key) {
                return Err("document already open".into());
            }
            let open: edit::wire::OpenReply = decode(&a)?;
            let (mirror, step) = Mirror::bootstrap(&open).map_err(|e| format!("{e:?}"))?;
            state.next += 1;
            let mut doc = Document {
                preview: None,
                diagnostics: Default::default(),
                matches: Vec::new(),
                resyncs: 0,
                mirror,
                model: EditorModel::default(),
                highlight: Highlight::for_language(
                    &open.language,
                    open.path.as_deref().map(std::path::Path::new),
                ),
                identity: state.next,
                out: VecDeque::new(),
                notices: Vec::new(),
                layout: Default::default(),
            };
            doc.step(step);
            let value = doc.state(key);
            state.documents.insert(key.into(), doc);
            return Ok(value);
        }
        if op == "close" {
            state.documents.remove(key).ok_or("unknown document")?;
            return Ok(json!({"closed":true}));
        }
        let State { documents, ids, .. } = &mut *state;
        let doc = documents.get_mut(key).ok_or("unknown document")?;
        let previous_phase = doc.mirror.phase().clone();
        let previous_conflicts = doc.mirror.conflicts().len();
        match op {
            "state" => {}
            "find" => return tools::find(doc, &a, ids),
            "diagnostics" => return tools::diagnostics(doc, &a),
            "problems" => return Ok(tools::problems(doc)),
            "selected" => {
                let lo = doc.model.sel.anchor.min(doc.model.sel.head);
                let hi = doc.model.sel.anchor.max(doc.model.sel.head);
                let mut text = String::new();
                doc.mirror.text().read(lo..hi, &mut text);
                return Ok(json!({"text":if lo==hi{None}else{Some(text)}}));
            }
            "clear-markers" => doc.model.clear_markers(),
            "conflict" => {
                let rev = crate::widgets::whole(&a["rev"]).ok_or("conflict revision required")?;
                let conflict = doc
                    .mirror
                    .conflicts()
                    .iter()
                    .find(|c| c.rev == rev)
                    .ok_or("conflict no longer exists")?;
                if a["reinsert"] == true {
                    let text = conflict.texts.concat();
                    doc.command(
                        EditCommand::Insert(text),
                        &View::default(),
                        ids,
                        Intent::ui(doc.identity),
                    )?;
                }
                doc.mirror.dismiss_conflict(rev);
            }
            "overwrite" => doc.model.overwrite = !doc.model.overwrite,
            "inspect" => {
                let m = &doc.mirror;
                if a["text"] == true && m.text().len() > 4 * 1024 * 1024 {
                    return Err("text exceeds ced.state 4 MiB inline limit".into());
                }
                let mut hasher = blake3::Hasher::new();
                let mut offset = 0;
                let mut text = String::new();
                while offset < m.text().len() {
                    let chunk = m.text().chunk_at(offset);
                    hasher.update(chunk);
                    if a["text"] == true {
                        text.push_str(std::str::from_utf8(chunk).unwrap());
                    }
                    offset += chunk.len();
                }
                let mut out = doc.state(key);
                out["text_hash"] = json!(hasher.finalize().to_hex().to_string());
                out["text"] = if a["text"] == true {
                    json!(text)
                } else {
                    Value::Null
                };
                out["last_remote"] = m
                    .last_remote()
                    .map(|r| json!({"origin":r.origin,"lane":r.lane,"rev":r.rev,"kind":r.kind}))
                    .unwrap_or(Value::Null);
                out["conflicts"]=json!(m.conflicts().iter().map(|c|json!({"rev":c.rev,"remote_origin":c.remote_origin,"lines":[c.lines.0,c.lines.1],"texts":c.texts})).collect::<Vec<_>>());
                return Ok(out);
            }
            "read" => {
                if a.get("view_gen")
                    .is_some_and(|g| crate::widgets::whole(g) != Some(doc.mirror.view_gen()))
                {
                    return Err("document changed during paged read".into());
                }
                let start = point(doc.mirror.text(), a.get("start").unwrap_or(&json!(0)))?;
                let limit = crate::widgets::whole(&a["max_bytes"])
                    .unwrap_or(1024 * 1024)
                    .min(4 * 1024 * 1024) as usize;
                let mut end = (start + limit).min(doc.mirror.text().len());
                while !doc.mirror.text().is_char_boundary(end) {
                    end -= 1;
                }
                let mut text = String::new();
                doc.mirror.text().read(start..end, &mut text);
                return Ok(
                    json!({"text":text,"start":start,"next":if end<doc.mirror.text().len(){Some(end)}else{None},"view_gen":doc.mirror.view_gen(),"rev":doc.mirror.rev()}),
                );
            }
            "hash" => {
                let mut hasher = blake3::Hasher::new();
                let mut offset = 0;
                while offset < doc.mirror.text().len() {
                    let chunk = doc.mirror.text().chunk_at(offset);
                    hasher.update(chunk);
                    offset += chunk.len();
                }
                return Ok(json!({"hash":hasher.finalize().to_hex().to_string()}));
            }
            "effects" => {
                while let Some(out) = doc.mirror.next_outgoing() {
                    doc.out.push_back(out);
                }
                let out:Vec<_>=doc.out.drain(..).map(|o|json!({"command":o.command,"body":o.body,"op_id":o.op_id,"deadline_ms":o.deadline_ms})).collect();
                let retry = doc.mirror.take_retry_timer();
                let echo = doc.mirror.take_echo_timer();
                let outcomes: Vec<_> = doc
                    .mirror
                    .take_outcomes()
                    .into_iter()
                    .map(|(id, o)| match o {
                        editor_model::mirror::Outcome::Done => json!({"op_id":id,"ok":true}),
                        editor_model::mirror::Outcome::Refused(r) => {
                            json!({"op_id":id,"ok":false,"error":r})
                        }
                    })
                    .collect();
                return Ok(
                    json!({"state":doc.state(key),"out":out,"retry_ms":retry,"echo":echo,"outcomes":outcomes}),
                );
            }
            "reply" => {
                let id = text_arg(&a, "op_id")?;
                let step = if a["rc"].as_f64() == Some(0.0) {
                    doc.mirror.on_reply(id, Ok(normalize(a["body"].clone())?))
                } else {
                    doc.mirror.on_reply(id, Err(decode(&a["body"])?))
                };
                doc.step(step);
            }
            "event" => {
                let event: edit::wire::Event = decode(&a)?;
                let step = doc.mirror.on_event(&event);
                doc.step(step);
            }
            "suspect" => {
                let step = doc.mirror.suspect();
                doc.step(step);
            }
            "epoch" => {
                let step = doc.mirror.epoch_changed();
                doc.step(step);
            }
            "reattach" => {
                let open = decode(&a)?;
                let step = doc.mirror.reattach(&open);
                doc.step(step);
            }
            "saved" => {
                let list: edit::wire::ListReply = decode(&a)?;
                if list.epoch == doc.mirror.epoch() {
                    if let Some(row) = list
                        .buffers
                        .iter()
                        .find(|b| b.buffer == doc.mirror.buffer())
                    {
                        doc.mirror.note_saved(row.saved_rev, row.recovered);
                    }
                }
            }
            "keep-mine" => {
                let step = doc.mirror.keep_mine(Intent::ui(doc.identity), ids);
                doc.step(step);
            }
            "take-theirs" => {
                let step = doc.mirror.take_theirs();
                doc.step(step);
            }
            "deadline" => {
                let step = doc.mirror.on_deadline(text_arg(&a, "op_id")?);
                doc.step(step);
            }
            "retry" => {
                let step = doc.mirror.on_retry();
                doc.step(step);
            }
            "echo" => {
                let step = doc.mirror.on_echo_timeout(text_arg(&a, "op_id")?);
                doc.step(step);
            }
            "select" => {
                doc.model.sel = Selection {
                    anchor: point(doc.mirror.text(), &a["anchor"])?,
                    head: point(doc.mirror.text(), &a["head"])?,
                };
                doc.model.preferred_cells = None;
            }
            "restore-view" => {
                let caret = crate::widgets::whole(&a["caret"]).unwrap_or(0) as usize;
                let mut caret = caret.min(doc.mirror.text().len());
                while !doc.mirror.text().is_char_boundary(caret) {
                    caret -= 1;
                }
                doc.model.sel = Selection {
                    anchor: caret,
                    head: caret,
                };
                doc.model.scroll.first_line = (crate::widgets::whole(&a["first_line"]).unwrap_or(1)
                    as usize)
                    .max(1)
                    .min(doc.mirror.text().line_count());
            }
            "command" => {
                let name = text_arg(&a, "name")?;
                let intent = a["caller"].as_str().map_or_else(
                    || Intent::ui(doc.identity),
                    |caller| Intent::bus(doc.identity, caller),
                );
                let command = match name {
                    "insert" => Some(EditCommand::Insert(text_arg(&a, "text")?.into())),
                    "select-all" => Some(EditCommand::SelectAll),
                    "delete" => Some(EditCommand::Delete),
                    "backspace" => Some(EditCommand::Backspace),
                    "delete-word-left" => Some(EditCommand::DeleteWordLeft),
                    "delete-word-right" => Some(EditCommand::DeleteWordRight),
                    "duplicate-line" => Some(EditCommand::DuplicateLine),
                    "delete-line" => Some(EditCommand::DeleteLine),
                    "move-line-up" => Some(EditCommand::MoveLineUp),
                    "move-line-down" => Some(EditCommand::MoveLineDown),
                    "toggle-comment" => Some(EditCommand::ToggleComment),
                    "indent" => Some(EditCommand::Tab),
                    "outdent" => Some(EditCommand::Outdent),
                    _ => None,
                };
                if let Some(c) = command {
                    doc.command(c, &View::default(), ids, intent)?;
                } else {
                    let server = match name {
                        "undo" => ServerOp::Undo {
                            lane: if a["lane"] == "any" {
                                LaneArg::Any
                            } else if let Some(l) = a["lane"].as_str() {
                                LaneArg::Lane(l.to_owned())
                            } else {
                                LaneArg::Own
                            },
                        },
                        "redo" => ServerOp::Redo { lane: LaneArg::Own },
                        "save" => ServerOp::Save {
                            path: a["path"].as_str().map(str::to_owned),
                            force: a["force"] == true,
                        },
                        "reload" => ServerOp::Reload {
                            force: a["force"] == true,
                        },
                        _ => return Err("unknown editor command".into()),
                    };
                    let (step, op_id) = doc.mirror.server_op(server, intent, ids);
                    doc.step(step);
                    return Ok(json!({"state":doc.state(key),"op_id":op_id}));
                }
            }
            _ => return Err("unknown document operation".into()),
        }
        let value = doc.state(key);
        let next_phase = doc.mirror.phase().clone();
        let new_conflicts = doc
            .mirror
            .conflicts()
            .len()
            .saturating_sub(previous_conflicts);
        *state.counters.entry("conflicts").or_default() += new_conflicts as u64;
        if next_phase != previous_phase {
            if let Phase::Recovering { how, .. } = next_phase {
                let counter = match how {
                    editor_model::mirror::RecoverHow::History => "history_recoveries",
                    editor_model::mirror::RecoverHow::Snapshot => "snapshot_recoveries",
                };
                *state.counters.entry(counter).or_default() += 1;
            }
        }
        Ok(value)
    }
    pub fn draw(
        &self,
        ui: &mut egui::Ui,
        widget: &crate::widgets::Widget,
        value: &Value,
        selection: Option<[usize; 2]>,
        events: &mut Vec<crate::widgets::Intent>,
    ) -> egui::Response {
        let Some(key) = value["document"].as_str() else {
            return ui.colored_label(egui::Color32::RED, "missing document handle");
        };
        let Ok(mut state) = self.0.lock() else {
            return ui.colored_label(egui::Color32::RED, "document owner poisoned");
        };
        let State { documents, ids, .. } = &mut *state;
        let Some(doc) = documents.get_mut(key) else {
            return ui.colored_label(egui::Color32::RED, "unknown document handle");
        };
        let before = doc.state(key);
        if let Some([start, end]) = selection {
            doc.model.sel = Selection {
                anchor: scalar_offset(doc.mirror.text(), start),
                head: scalar_offset(doc.mirror.text(), end),
            };
        }
        let mut view = crate::editor::native_view(ui, widget);
        view.matches = doc.matches.clone();
        let diagnostics = tools::render_diagnostics(doc);
        let palette = crate::editor::native_palette(ui, widget);
        let padding = widget.properties["padding"].as_f64().unwrap_or(0.0).round() as i8;
        let out = egui::Frame::new()
            .fill(palette.background)
            .inner_margin(egui::Margin::same(padding))
            .show(ui, |ui| {
                editor::show(
                    ui,
                    (&widget.id, key),
                    &Doc {
                        text: doc.mirror.text(),
                        identity: doc.identity,
                        revision: doc.mirror.view_gen(),
                        model: &doc.model,
                        highlight: Some(&doc.highlight),
                        diagnostics: &diagnostics,
                    },
                    &palette,
                    &view,
                )
            })
            .inner;
        if out.response.has_focus() {
            ui.ctx()
                .data_mut(|d| d.insert_temp(egui::Id::unique("viu-editor-focused"), true));
        }
        for event in out.events {
            match event {
                Event::Command(c) => {
                    if let Err(e) = doc.command(c, &view, ids, Intent::ui(doc.identity)) {
                        doc.notices.push(e);
                    }
                }
                Event::Scrolled(s) => doc.model.scroll = s,
                Event::Preedit(p) => doc.model.set_preedit(!p.is_empty()),
                Event::Undo | Event::Redo => {
                    let op = if matches!(event, Event::Undo) {
                        ServerOp::Undo { lane: LaneArg::Own }
                    } else {
                        ServerOp::Redo { lane: LaneArg::Own }
                    };
                    let (step, _) = doc.mirror.server_op(op, Intent::ui(doc.identity), ids);
                    doc.step(step);
                }
                Event::Layout(l) => doc.layout = l,
                _ => {}
            }
        }
        let after = doc.state(key);
        if before != after {
            events.push(crate::widgets::Intent::set(&widget.id, after));
        }
        out.response
    }

    pub fn show_preview(
        &self,
        ui: &mut egui::Ui,
        widget: &crate::widgets::Widget,
        value: &Value,
    ) -> egui::Response {
        let preview = {
            let state = self.0.lock().unwrap();
            value["document"]
                .as_str()
                .and_then(|key| state.documents.get(key))
                .and_then(|d| {
                    d.preview
                        .as_ref()
                        .filter(|(generation, _)| *generation == d.mirror.view_gen())
                        .map(|(_, preview)| preview.clone())
                })
        };
        let palette = crate::editor::native_palette(ui, widget);
        let view = crate::editor::native_view(ui, widget);
        egui::Frame::new()
            .fill(palette.background)
            .inner_margin(widget.properties["padding"].as_f64().unwrap_or(8.0) as i8)
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                for style in [egui::TextStyle::Body, egui::TextStyle::Monospace] {
                    if let Some(font) = ui.style_mut().text_styles.get_mut(&style) {
                        font.size = view.font_size.unwrap_or(13.0);
                    }
                }
                egui::ScrollArea::both()
                    .id_salt((&widget.id, "markdown"))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if let Some(document) = preview {
                            toolkit::markdown::show(ui, &document);
                        } else {
                            ui.weak("Preparing preview…");
                        }
                    });
            })
            .response
    }
}
