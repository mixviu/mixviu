// SPDX-License-Identifier: MIT OR Apache-2.0
//! Host-owned, report-only design validation. No Mix evaluator runs here.
use design::{DesignContext, DesignSourceDocument};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::PathBuf,
    sync::{Arc, mpsc},
};

type Observation = (String, Option<PathBuf>);
type Task = (Observation, Arc<DesignSourceDocument>, DesignContext);
struct Report {
    solicited: bool,
    digest: String,
    retired: Option<String>,
    error: Option<String>,
}

// One current file per host. Replacing it unregisters the old path, including
// when the current source becomes embedded. Queues and histories are bounded.
const PENDING_LIMIT: usize = 32;
const SEEN_LIMIT: usize = 128;
const ERROR_LIMIT: usize = 64;

#[derive(Default)]
pub(crate) struct Validation {
    seen: BTreeSet<Observation>,
    pending: VecDeque<Task>,
    sender: Option<mpsc::SyncSender<Task>>,
    receiver: Option<mpsc::Receiver<Report>>,
    errors: BTreeMap<String, String>,
    pub notice: String,
    notice_digest: Option<String>,
    outstanding: usize,
    worker_error: Option<String>,
    skipped: usize,
    evicted: usize,
    current_path: Option<PathBuf>,
}

fn check(document: &DesignSourceDocument, context: DesignContext) -> Option<String> {
    match design::compile_design(document, context) {
        design::DesignCompileResult::Success(_) => None,
        design::DesignCompileResult::Fatal(failure) => Some(format!("{:?}", failure.diagnostics)),
    }
}

fn scan(path: &std::path::Path, previous: &mut Option<String>, solicited: bool) -> Option<Report> {
    let source = std::fs::read_to_string(path);
    let (digest, error) = match source {
        Ok(source) => {
            let digest = design::source_digest(&source);
            if previous.as_ref() == Some(&digest) {
                // Ack only: no new validation result to erase a prior failure.
                return solicited.then(|| Report {
                    solicited,
                    digest: String::new(),
                    retired: None,
                    error: None,
                });
            }
            let error = toolkit::Theme::from_source("design-report", &source)
                .map_err(|e| e.to_string())
                .and_then(|theme| {
                    check(&theme.source_document(), theme.context().clone()).map_or(Ok(()), Err)
                })
                .err();
            (digest, error)
        }
        // Removing the file restores the embedded theme, just like Theme::read.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            (design::source_digest(design::EMBEDDED_DEFAULT_SOURCE), None)
        }
        Err(error) => (
            design::source_digest(&format!("{}:{error}", path.display())),
            Some(error.to_string()),
        ),
    };
    if previous.as_ref() == Some(&digest) && !solicited {
        return None;
    }
    let retired = previous
        .replace(digest.clone())
        .filter(|old| old != &digest);
    Some(Report {
        solicited,
        digest,
        retired,
        error,
    })
}

fn process_task(task: Task, file: &mut Option<(PathBuf, Option<String>)>) -> Option<Report> {
    let ((digest, path), document, context) = task;
    if let Some(path) = path {
        let retired = if file.as_ref().map(|f| &f.0) != Some(&path) {
            let retired = file.take().and_then(|f| f.1);
            *file = Some((path, None));
            retired
        } else {
            None
        };
        let (path, previous) = file.as_mut().unwrap();
        // Every received task acknowledges exactly once, even
        // if a prior spontaneous scan already saw these bytes.
        let mut report = scan(path, previous, true);
        if let Some(report) = &mut report
            && retired.is_some()
        {
            report.retired = retired;
        }
        report
    } else {
        let retired = file.take().and_then(|f| f.1);
        Some(Report {
            solicited: true,
            digest,
            retired,
            error: check(&document, context),
        })
    }
}

impl Validation {
    pub fn observe(&mut self, theme: &toolkit::Theme) {
        if let Some(error) = theme.load_error() {
            self.refused(
                theme.load_source_digest().unwrap_or("unreadable-source"),
                error,
            );
        }
        if self.worker_error.is_some() {
            self.skipped += 1;
            return;
        }
        let path = theme.source_path().map(std::path::Path::to_owned);
        let changed_path = path != self.current_path;
        self.current_path = path.clone();
        let key = (theme.source_digest().to_owned(), path);
        if !changed_path && self.seen.contains(&key) {
            return;
        }
        if self.seen.len() >= SEEN_LIMIT && !self.seen.contains(&key) {
            self.seen.pop_first();
            self.evicted += 1;
        }
        self.seen.insert(key.clone());
        if self.pending.len() >= PENDING_LIMIT {
            let retired = self.pending.pop_front().unwrap();
            self.seen.remove(&retired.0);
            self.skipped += 1;
            tracing::warn!("design report queue full; retired source counted in info");
        }
        self.pending
            .push_back((key, theme.source_document(), theme.context().clone()));
    }

    /// Called only after a ready frame. The worker may block on its bounded
    /// reply queue; the rendering thread only ever uses try_send/try_recv.
    pub fn after_frame(&mut self) {
        if self.worker_error.is_some() {
            return;
        }
        if self.sender.is_none() {
            let (tx, tasks) = mpsc::sync_channel::<Task>(4);
            let (answers, rx) = mpsc::sync_channel(8);
            let worker = std::thread::Builder::new()
                .name("design-report".into())
                .spawn(move || {
                    let mut file: Option<(PathBuf, Option<String>)> = None;
                    loop {
                        let report = match tasks.recv_timeout(std::time::Duration::from_millis(500))
                        {
                            Ok(task) => process_task(task, &mut file),
                            Err(mpsc::RecvTimeoutError::Timeout) => file
                                .as_mut()
                                .and_then(|(path, previous)| scan(path, previous, false)),
                            Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        };
                        if let Some(report) = report
                            && answers.send(report).is_err()
                        {
                            break;
                        }
                    }
                });
            if let Err(error) = worker {
                self.worker_error = Some(error.to_string());
                self.skipped += self.pending.len();
                self.pending.clear();
                return;
            }
            self.sender = Some(tx);
            self.receiver = Some(rx);
        }
        while let Some(task) = self.pending.pop_front() {
            match self.sender.as_ref().unwrap().try_send(task) {
                Ok(()) => self.outstanding += 1,
                Err(mpsc::TrySendError::Full(task)) => {
                    self.pending.push_front(task);
                    break;
                }
                Err(mpsc::TrySendError::Disconnected(_)) => {
                    self.skipped += 1; // the popped task is no longer in pending
                    self.disconnected();
                    break;
                }
            }
        }
    }

    fn disconnected(&mut self) {
        self.worker_error = Some("design report worker stopped".into());
        self.skipped += self.outstanding + self.pending.len();
        self.outstanding = 0;
        self.pending.clear();
    }

    pub fn poll(&mut self) {
        loop {
            let Some(receiver) = &self.receiver else {
                break;
            };
            match receiver.try_recv() {
                Ok(report) => {
                    if report.solicited {
                        self.outstanding = self.outstanding.saturating_sub(1);
                    }
                    if let Some(retired) = report.retired {
                        self.clear_error(&retired);
                    }
                    if let Some(error) = report.error {
                        tracing::warn!(source_digest=%report.digest, %error, "design background report");
                        self.record(report.digest, error);
                    } else {
                        self.clear_error(&report.digest);
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.disconnected();
                    break;
                }
            }
        }
    }

    fn clear_error(&mut self, digest: &str) {
        self.errors.remove(digest);
        if self.notice_digest.as_deref() == Some(digest) {
            self.dismiss();
        }
    }

    pub fn dismiss(&mut self) {
        self.notice.clear();
        self.notice_digest = None;
    }

    fn record(&mut self, digest: String, error: String) {
        if self.errors.len() >= ERROR_LIMIT && !self.errors.contains_key(&digest) {
            self.errors.pop_first();
            self.evicted += 1;
        }
        self.errors.insert(digest, error);
    }
    pub fn refused(&mut self, digest: &str, error: &str) {
        if self.notice != error {
            tracing::warn!(source_digest = digest, error, "design context refused");
        }
        self.notice = error.into();
        self.notice_digest = Some(digest.into());
        self.record(digest.into(), error.into());
    }
    pub fn info(&self) -> Value {
        json!({"pending":self.outstanding + self.pending.len(), "errors":self.errors,"notice":self.notice,"worker_error":self.worker_error,"skipped":self.skipped,"evicted":self.evicted})
    }
}

pub(crate) fn observe(ctx: &egui::Context, theme: &toolkit::Theme) {
    ctx.data_mut(|data| {
        // Installation order matters: the last installation is the current
        // source, including a change from a file back to embedded defaults.
        data.insert_temp(
            egui::Id::unique("viu-design-report-input"),
            Some(theme.clone()),
        );
    });
}
pub(crate) fn drain(ctx: &egui::Context, validation: &mut Validation) {
    if let Some(theme) = ctx
        .data_mut(|data| {
            data.remove_temp::<Option<toolkit::Theme>>(egui::Id::unique("viu-design-report-input"))
        })
        .flatten()
    {
        validation.observe(&theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_before_first_frame_and_full_channel_keeps_latest_registration() {
        let dir = tempfile::tempdir().unwrap();
        let old_path = dir.path().join("theme.mix");
        let new_path = dir.path().join("other.mix");
        for path in [&old_path, &new_path] {
            std::fs::write(path, "mode: dark").unwrap();
        }
        let old = toolkit::Theme::read(&old_path).unwrap().unwrap();
        for latest in [
            toolkit::Theme::embedded(),
            toolkit::Theme::read(&new_path).unwrap().unwrap(),
        ] {
            for full in [false, true] {
                // Hold the worker receiver to make Full deterministic, including
                // before the first frame. Use the real worker task processor below.
                let (tx, tasks) = mpsc::sync_channel(4);
                let (answers, rx) = mpsc::sync_channel(8);
                let initial = if full { 4 } else { 0 };
                let mut v = Validation {
                    sender: Some(tx.clone()),
                    receiver: Some(rx),
                    outstanding: initial,
                    ..Default::default()
                };
                for _ in 0..initial {
                    tx.send((
                        (old.source_digest().into(), Some(old_path.clone())),
                        old.source_document(),
                        old.context().clone(),
                    ))
                    .unwrap();
                }
                v.observe(&old);
                v.observe(&latest);
                v.after_frame();
                let mut file = None;
                if full {
                    assert_eq!(
                        v.pending.len(),
                        2,
                        "Full must return without spinning or losing work"
                    );
                    for _ in 0..initial {
                        answers
                            .send(process_task(tasks.try_recv().unwrap(), &mut file).unwrap())
                            .unwrap();
                    }
                    v.poll();
                    v.after_frame();
                }
                for _ in 0..2 {
                    answers
                        .send(process_task(tasks.try_recv().unwrap(), &mut file).unwrap())
                        .unwrap();
                }
                v.poll();
                assert_eq!(
                    file.as_ref().map(|f| f.0.as_path()),
                    latest.source_path(),
                    "worker must retain the latest registration"
                );
                assert_eq!(v.info()["pending"], 0);
            }
        }
    }

    #[test]
    fn repair_clears_notice_by_digest_after_different_background_diagnostic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("theme.mix");
        std::fs::write(&path, "broken").unwrap();
        let rejected = toolkit::Theme::load_from(&path);
        let mut v = Validation::default();
        v.refused(
            rejected.load_source_digest().unwrap(),
            rejected.load_error().unwrap(),
        );
        let display = v.notice.clone();
        let (answers, rx) = mpsc::sync_channel(8);
        v.receiver = Some(rx);
        let mut previous = None;
        let report = scan(&path, &mut previous, false).unwrap();
        assert_ne!(
            report.error.as_ref().unwrap(),
            &display,
            "fixture must exercise differing source labels/formatting"
        );
        answers.send(report).unwrap();
        v.poll();
        assert_eq!(v.notice, display);
        std::fs::write(&path, "mode: dark").unwrap();
        answers
            .send(scan(&path, &mut previous, false).unwrap())
            .unwrap();
        v.poll();
        assert!(v.errors.is_empty());
        assert!(
            v.notice.is_empty(),
            "repair must clear the refusal notice by digest"
        );
        assert!(v.notice_digest.is_none());
    }

    #[test]
    fn disconnected_send_counts_popped_task_and_spawn_failure_never_queues() {
        let theme = toolkit::Theme::embedded();
        let (tx, tasks) = mpsc::sync_channel(4);
        drop(tasks);
        let mut v = Validation {
            sender: Some(tx),
            outstanding: 3,
            ..Default::default()
        };
        v.observe(&theme);
        v.after_frame();
        assert_eq!(
            v.skipped, 4,
            "include the popped task as well as in-flight work"
        );
        assert_eq!(v.info()["pending"], 0);
        let mut failed = Validation {
            worker_error: Some("spawn failed".into()),
            ..Default::default()
        };
        for _ in 0..5 {
            failed.observe(&theme);
        }
        failed.after_frame();
        failed.poll();
        assert_eq!(failed.skipped, 5);
        assert_eq!(failed.info()["pending"], 0);
        assert!(failed.receiver.is_none());
    }
    fn settle(validation: &mut Validation) {
        let until = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            validation.after_frame();
            validation.poll();
            if validation.info()["pending"] == 0 {
                return;
            }
            assert!(std::time::Instant::now() < until, "{}", validation.info());
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    #[test]
    fn file_registration_repair_deletion_and_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("theme.mix");
        let other = dir.path().join("other.mix");
        let source = design::EMBEDDED_DEFAULT_SOURCE;
        let mut v = Validation::default();
        // Same bytes first pathless, then file-backed: must register the path.
        v.observe(&toolkit::Theme::from_source("pathless", source).unwrap());
        settle(&mut v);
        std::fs::write(&path, source).unwrap();
        v.observe(&toolkit::Theme::read(&path).unwrap().unwrap());
        settle(&mut v);
        std::fs::write(&path, "broken").unwrap();
        let until = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while v.errors.is_empty() {
            v.poll();
            assert!(std::time::Instant::now() < until);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(v.errors.contains_key(&design::source_digest("broken")));
        assert_eq!(
            v.info()["pending"],
            0,
            "spontaneous report has no task credit"
        );
        // Re-observe bytes already polled: task still gets exactly one ack.
        v.observe(&toolkit::Theme::load_from(&path));
        settle(&mut v);
        assert!(
            v.errors.contains_key(&design::source_digest("broken")),
            "ack must not clear an unvalidated error"
        );
        std::fs::write(&path, source).unwrap();
        let until = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while !v.errors.is_empty() {
            v.poll();
            assert!(std::time::Instant::now() < until);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        std::fs::remove_file(&path).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(550));
        v.poll();
        assert!(
            v.errors.is_empty(),
            "missing theme means embedded, not a design error"
        );
        // Another path replaces the registration rather than extending a map.
        std::fs::write(&other, source).unwrap();
        v.observe(&toolkit::Theme::read(&other).unwrap().unwrap());
        settle(&mut v);
        std::fs::write(&path, "broken old path").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(550));
        v.poll();
        assert!(v.errors.is_empty());
        v.observe(&toolkit::Theme::embedded());
        settle(&mut v);
        std::fs::write(&other, "broken unregistered path").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(550));
        v.poll();
        assert!(v.errors.is_empty());
        assert_eq!(v.info()["pending"], 0);
    }

    #[test]
    fn accounting_distinguishes_unsolicited_ack_and_disconnect() {
        let (answers, rx) = mpsc::sync_channel(8);
        let mut v = Validation {
            receiver: Some(rx),
            outstanding: 2,
            ..Default::default()
        };
        v.refused("repaired", "bad");
        answers
            .send(Report {
                solicited: false,
                digest: "repaired".into(),
                retired: None,
                error: None,
            })
            .unwrap();
        v.poll();
        assert!(v.errors.is_empty());
        assert_eq!(v.outstanding, 2);
        answers
            .send(Report {
                solicited: true,
                digest: String::new(),
                retired: None,
                error: None,
            })
            .unwrap();
        v.poll();
        assert_eq!(v.outstanding, 1);
        drop(answers);
        v.poll();
        assert_eq!(v.info()["pending"], 0);
        assert_eq!(v.skipped, 1);
        assert!(v.worker_error.is_some());
    }

    #[test]
    fn queue_and_history_degradation_is_bounded_and_counted() {
        let dir = tempfile::tempdir().unwrap();
        let mut v = Validation::default();
        for i in 0..40 {
            let path = dir.path().join(format!("source-{i}"));
            std::fs::write(&path, "mode: dark").unwrap();
            v.observe(&toolkit::Theme::read(&path).unwrap().unwrap());
        }
        assert_eq!(v.pending.len(), PENDING_LIMIT);
        assert_eq!(v.skipped, 8);
        for i in 40..200 {
            v.pending.clear(); // completed submission between observations
            let path = dir.path().join(format!("source-{i}"));
            std::fs::write(&path, "mode: dark").unwrap();
            v.observe(&toolkit::Theme::read(&path).unwrap().unwrap());
        }
        assert!(v.evicted > 0);
        assert!(v.seen.len() <= SEEN_LIMIT);
        for i in 0..80 {
            v.refused(&format!("bad-{i}"), "failure");
        }
        assert_eq!(v.errors.len(), ERROR_LIMIT);
    }
}
