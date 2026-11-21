// SPDX-License-Identifier: MIT OR Apache-2.0
//! Host-owned, report-only design validation. No Mix evaluator runs here.
use std::{collections::{BTreeMap, BTreeSet}, sync::{Arc, mpsc}};
use design::{DesignContext, DesignSourceDocument};
use serde_json::{Value, json};

type Task = (String, Arc<DesignSourceDocument>, DesignContext);

#[derive(Default)]
pub(crate) struct Validation {
    seen: BTreeSet<(String, DesignContext)>,
    pending: Vec<Task>,
    sender: Option<mpsc::Sender<Task>>,
    receiver: Option<mpsc::Receiver<(String, Option<String>)>>,
    errors: BTreeMap<String, String>,
    pub notice: String,
    outstanding: usize,
}

impl Validation {
    pub fn observe(&mut self, theme: &toolkit::Theme) {
        if let Some(error) = theme.load_error() {
            self.refused(theme.load_source_digest().unwrap_or("unreadable-source"), error);
        }
        let key = (theme.source_digest().to_owned(), theme.context().clone());
        if self.seen.insert(key.clone()) {
            self.pending.push((key.0, theme.source_document(), key.1));
        }
    }

    pub fn observe_document(&mut self, document: Arc<DesignSourceDocument>, context: DesignContext) {
        let digest = design::document_digest(&document);
        if self.seen.insert((digest.clone(), context.clone())) {
            self.pending.push((digest, document, context));
        }
    }

    /// Called only after a ready frame. Exactly one native worker per host;
    /// dropping the host closes its queue and never joins on the UI thread.
    pub fn after_frame(&mut self) {
        if self.sender.is_none() {
            let (tx, tasks) = mpsc::channel::<Task>();
            let (answers, rx) = mpsc::channel();
            std::thread::Builder::new().name("design-report".into()).spawn(move || {
                for (digest, document, context) in tasks {
                    let error = match design::compile_design(&document, context) {
                        design::DesignCompileResult::Success(_) => None,
                        design::DesignCompileResult::Fatal(failure) => Some(format!("{:?}", failure.diagnostics)),
                    };
                    if answers.send((digest, error)).is_err() { break; }
                }
            }).expect("start design report worker");
            self.sender = Some(tx);
            self.receiver = Some(rx);
        }
        for task in self.pending.drain(..) {
            if self.sender.as_ref().unwrap().send(task).is_ok() { self.outstanding += 1; }
        }
    }

    pub fn poll(&mut self) {
        if let Some(receiver) = &self.receiver {
            while let Ok((digest, error)) = receiver.try_recv() {
                self.outstanding = self.outstanding.saturating_sub(1);
                if let Some(error) = error {
                    tracing::warn!(source_digest=%digest, %error, "design background report");
                    self.errors.insert(digest, error);
                }
            }
        }
    }

    pub fn refused(&mut self, digest: &str, error: &str) {
        if self.notice != error {
            tracing::warn!(source_digest=digest, error, "design context refused");
        }
        self.notice = error.into();
        self.errors.insert(digest.into(), error.into());
    }

    pub fn info(&self) -> Value {
        json!({"pending":self.outstanding + self.pending.len(), "errors":self.errors,"notice":self.notice})
    }
}

// Sources actually installed by session and local appearance adapters. The
// host drains these after drawing; this carries source inputs, never artifacts
// or validation results, and is local to the rendering context.
pub(crate) fn observe(ctx: &egui::Context, theme: &toolkit::Theme) {
    ctx.data_mut(|data| {
        let key = egui::Id::unique("viu-design-report-inputs");
        let inputs = data.get_temp_mut_or_default::<BTreeMap<String, toolkit::Theme>>(key);
        inputs.insert(theme.source_digest().to_owned(), theme.clone());
    });
}

pub(crate) fn drain(ctx: &egui::Context, validation: &mut Validation) {
    let inputs = ctx.data_mut(|data| data.remove_temp::<BTreeMap<String, toolkit::Theme>>(egui::Id::unique("viu-design-report-inputs"))).unwrap_or_default();
    for theme in inputs.values() { validation.observe(theme); }
}
