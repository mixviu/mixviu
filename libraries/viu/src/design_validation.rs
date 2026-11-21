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
    sender: Option<mpsc::SyncSender<Task>>,
    receiver: Option<mpsc::Receiver<(String, Option<String>)>>,
    errors: BTreeMap<String, String>,
    pub notice: String,
    outstanding: usize,
    worker_error: Option<String>,
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
        if self.seen.len() >= 128 { self.seen.clear(); }
        if self.seen.insert(digest.clone()) {
            if self.pending.len() >= 32 {
                let retired = self.pending.remove(0);
                self.seen.remove(&retired.0);
            }
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
        if self.worker_error.is_some() { return; }
        if self.sender.is_none() {
            let (tx, tasks) = mpsc::sync_channel::<Task>(4);
            let (answers, rx) = mpsc::channel();
            let worker = std::thread::Builder::new()
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
                });
            if let Err(error) = worker {
                tracing::warn!(%error, "design report worker unavailable");
                self.worker_error = Some(error.to_string());
                return;
            }
            self.sender = Some(tx);
            self.receiver = Some(rx);
        }
        for task in std::mem::take(&mut self.pending) {
            match self.sender.as_ref().unwrap().try_send(task) {
                Ok(()) => self.outstanding += 1,
                Err(mpsc::TrySendError::Full(task)) => self.pending.push(task),
                Err(mpsc::TrySendError::Disconnected(_)) => {
                    self.worker_error = Some("design report worker stopped".into());
                }
            }
        }
    }

    pub fn poll(&mut self) {
        if let Some(receiver) = &self.receiver {
            while let Ok((digest, error)) = receiver.try_recv() {
                self.outstanding = self.outstanding.saturating_sub(1);
                if let Some(error) = error {
                    tracing::warn!(source_digest=%digest, %error, "design background report");
                    if self.errors.len() >= 64 { self.errors.pop_first(); }
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
        if self.errors.len() >= 64 { self.errors.pop_first(); }
        self.errors.insert(digest.into(), error.into());
    }

    pub fn info(&self) -> Value {
        json!({"pending":self.outstanding + self.pending.len(), "errors":self.errors,"notice":self.notice,"worker_error":self.worker_error})
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
