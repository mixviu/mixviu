// SPDX-License-Identifier: MIT OR Apache-2.0
//! Host-owned, report-only design validation. No Mix evaluator runs here.
use design::{DesignContext, DesignSourceDocument};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, mpsc},
};

type Task = (
    String,
    Arc<DesignSourceDocument>,
    DesignContext,
    Option<std::path::PathBuf>,
);

#[derive(Default)]
pub(crate) struct Validation {
    seen: BTreeSet<String>,
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
            self.refused(
                theme.load_source_digest().unwrap_or("unreadable-source"),
                error,
            );
        }
        let digest = theme.source_digest().to_owned();
        if self.seen.insert(digest.clone()) {
            self.pending.push((
                digest,
                theme.source_document(),
                theme.context().clone(),
                theme.source_path().map(std::path::Path::to_owned),
            ));
        }
    }

    /// Called only after a ready frame. Exactly one native worker per host;
    /// dropping the host closes its queue and never joins on the UI thread.
    pub fn after_frame(&mut self) {
        if self.sender.is_none() {
            let (tx, tasks) = mpsc::channel::<Task>();
            let (answers, rx) = mpsc::channel();
            std::thread::Builder::new()
                .name("design-report".into())
                .spawn(move || {
                    let mut files = BTreeMap::<std::path::PathBuf, String>::new();
                    loop {
                        match tasks.recv_timeout(std::time::Duration::from_millis(500)) {
                            Ok((digest, document, context, path)) => {
                                if let Some(path) = path {
                                    files.entry(path).or_default();
                                } else {
                                    let error = match design::compile_design(&document, context) {
                                        design::DesignCompileResult::Success(_) => None,
                                        design::DesignCompileResult::Fatal(failure) => {
                                            Some(format!("{:?}", failure.diagnostics))
                                        }
                                    };
                                    if answers.send((digest, error)).is_err() {
                                        break;
                                    }
                                    continue;
                                }
                            }
                            Err(mpsc::RecvTimeoutError::Timeout) => {}
                            Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        }
                        for (path, previous) in &mut files {
                            let read = std::fs::read_to_string(path);
                            let digest = match &read {
                                Ok(source) => design::source_digest(source),
                                Err(error) => {
                                    design::source_digest(&format!("{}:{error}", path.display()))
                                }
                            };
                            if *previous == digest {
                                continue;
                            }
                            *previous = digest.clone();
                            let error = read
                                .map_err(|e| e.to_string())
                                .and_then(|source| {
                                    let theme =
                                        toolkit::Theme::from_source("design-report", &source)
                                            .map_err(|e| e.to_string())?;
                                    match design::compile_design(
                                        &theme.source_document(),
                                        theme.context().clone(),
                                    ) {
                                        design::DesignCompileResult::Success(_) => Ok(()),
                                        design::DesignCompileResult::Fatal(failure) => {
                                            Err(format!("{:?}", failure.diagnostics))
                                        }
                                    }
                                })
                                .err();
                            if answers.send((digest, error)).is_err() {
                                return;
                            }
                        }
                    }
                })
                .expect("start design report worker");
            self.sender = Some(tx);
            self.receiver = Some(rx);
        }
        for task in self.pending.drain(..) {
            if self.sender.as_ref().unwrap().send(task).is_ok() {
                self.outstanding += 1;
            }
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
            tracing::warn!(source_digest = digest, error, "design context refused");
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
    let inputs = ctx
        .data_mut(|data| {
            data.remove_temp::<BTreeMap<String, toolkit::Theme>>(egui::Id::unique(
                "viu-design-report-inputs",
            ))
        })
        .unwrap_or_default();
    for theme in inputs.values() {
        validation.observe(theme);
    }
}
