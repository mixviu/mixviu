// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

//! Logger module — metadata-only Bus history. See ../BUS-HISTORY.md.
//!
//! Two separate broker connections by design:
//!
//! 1. **Service connection** registers as `log` and serves the `log.props.*`
//!    surface. Low-volume request/response — must deliver.
//! 2. **Observer connection** registers under a process-scoped identity,
//!    subscribes to metadata-only `noded.observe`, and receives the bounded,
//!    drop-accounted broker stream, available to every Bus caller.
//!
//! Splitting the connections gives each its own bounded outbound mpsc, so a
//! slow observer consumer (disk contention, log file rotation) can never starve
//! the props surface or trigger the symmetric route_local + broadcast_tap
//! drop pattern that crashed the box on 2026-04-28.
//!
//! Each node has its own exclusively locked history directory.

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use anyhow::Result;
use datetime::{DateTime, Utc};

use crate::log_props::{LogPropsSnapshot, LogStats};

#[derive(serde::Deserialize)]
struct ObserveEvent {
    seq: u64,
    ts: String,
    direction: String,
    outcome: String,
    message_type: String,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default)]
    command: Option<String>,
    size: usize,
    correlation_id: Option<String>,
    peer: Option<String>,
    rc: Option<i64>,
    dropped_count: u64,
}

pub async fn run(
    noded_url: &str,
    node: String,
    limits: HistoryConfig,
    shutdown: tokio::sync::oneshot::Receiver<()>,
) -> Result<()> {
    anyhow::ensure!(
        !node.is_empty() && node != "." && node != ".." && !node.contains('/'),
        "invalid history node directory"
    );
    let dir = logging::default_log_dir("noded").join(&node);
    run_in(noded_url, node, limits, shutdown, &dir).await
}

async fn run_in(
    noded_url: &str,
    node: String,
    limits: HistoryConfig,
    mut shutdown: tokio::sync::oneshot::Receiver<()>,
    log_dir: &Path,
) -> Result<()> {
    let started_instant = Instant::now();
    let started_at_iso = datetime::Utc::now().to_rfc3339_opts(datetime::SecondsFormat::Secs, true);

    let dir = log_dir.to_owned();
    let history =
        match tokio::task::spawn_blocking(move || History::open(&dir, node, limits, Utc::now()))
            .await?
        {
            Ok(history) => Arc::new(std::sync::Mutex::new(history)),
            Err(error) => {
                tracing::warn!(%error, "History disabled: cannot open or lock node directory");
                return Ok(());
            }
        };
    let bus_log_path = log_dir.join("bus.jsonl");
    let log_path_str = bus_log_path.to_string_lossy().to_string();

    let stats = LogStats::new();

    // Connection 1 — registered service for log.props.* request/response.
    let svc_client = Arc::new(bus::native_client::NodedClient::connect("log", noded_url).await?);
    tracing::info!("Logger props connection registered as 'log'");

    // Connection 2 — registered observer only. Isolating it onto its own
    // bounded outbound channel means a saturated observation queue cannot
    // block log.props.* delivery on the service connection.
    let observer_name = format!("log-observe-{}", std::process::id());
    let observer_client =
        Arc::new(bus::native_client::NodedClient::connect(&observer_name, noded_url).await?);
    observer_client
        .call(
            "noded",
            "noded.observe.start",
            serde_json::json!({
                "filter": {
                    "directions": ["local", "mesh_in", "mesh_out"],
                },
                "body": "none",
                "capacity": 1024,
            }),
        )
        .await?;
    stats.tap_subscribed.store(true, Ordering::Relaxed);
    tracing::info!("Logger observer connection subscribed to Bus traffic");

    // Spawn the props-surface task on the service connection.
    let svc_stats = stats.clone();
    let svc_log_path = log_path_str.clone();
    let svc_started_at = started_at_iso.clone();
    let svc_task = tokio::spawn(async move {
        let Some(mut rx) = svc_client.incoming_async().await else {
            tracing::error!("Logger service connection has no incoming stream");
            return;
        };
        while let Some(cmd) = rx.recv().await {
            let Some(suffix) = cmd.command.strip_prefix("log.props.") else {
                // Service connection only handles log.props.* — anything
                // else is an addressing mistake by the caller. Ignore.
                continue;
            };
            let snapshot = LogPropsSnapshot {
                log_path: svc_log_path.clone(),
                started_at: svc_started_at.clone(),
                uptime_s: started_instant.elapsed().as_secs(),
                tap_subscribed: svc_stats.tap_subscribed.load(Ordering::Relaxed),
                events_seen: svc_stats.events_seen.load(Ordering::Relaxed),
                bytes_logged: svc_stats.bytes_logged.load(Ordering::Relaxed),
            };
            let args_json = crate::log_props::parse_args(cmd.header("args"))
                .or_else(|| serde_json::from_str(&cmd.body).ok());
            let resp_inner = props::bus::dispatch_props(
                &snapshot,
                suffix,
                args_json.as_ref(),
                /* redact = */ true,
            );
            let rc_u8: u8 = resp_inner.rc.clamp(0, 255) as u8;
            if let Err(e) = svc_client.respond(&cmd, rc_u8, &resp_inner.body).await {
                tracing::warn!("log.props.{suffix} respond failed: {e}");
            }
        }
        tracing::info!("Logger service connection closed");
    });

    let _svc_guard = AbortOnDrop(svc_task);
    let result = async {
        let mut rx = observer_client
            .incoming_async()
            .await
            .ok_or_else(|| anyhow::anyhow!("Logger observer has no incoming stream"))?;
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(1));
        let mut last_warning = None;
        loop {
            tokio::select! {
                _ = &mut shutdown => break,
                _ = tick.tick() => {
                    let result = disk(&history, |h| h.tick(Utc::now())).await;
                    warn_retry(result, &mut last_warning);
                }
                cmd = rx.recv() => {
                    let Some(cmd) = cmd else { break };
                    if cmd.command != "noded.observe.event" { continue; }
                    let Ok(event) = serde_json::from_str::<ObserveEvent>(&cmd.body) else {
                        tracing::warn!("Logger received malformed noded.observe.event");
                        continue;
                    };
                    match disk(&history, move |h| h.event(event, Utc::now())).await {
                        Ok(bytes) => stats.record(bytes),
                        Err(error) => warn_retry(Err(error), &mut last_warning),
                    }
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    }
    .await;
    stats.tap_subscribed.store(false, Ordering::Relaxed);
    let closed = disk(&history, |h| {
        h.row(
            serde_json::json!({"kind": "gap", "reason": "observer_closed"}),
            Utc::now(),
        )?;
        h.flush()
    })
    .await;
    result?;
    closed?;
    Ok(())
}

struct AbortOnDrop(tokio::task::JoinHandle<()>);
impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn disk<T: Send + 'static>(
    history: &Arc<std::sync::Mutex<History>>,
    operation: impl FnOnce(&mut History) -> Result<T> + Send + 'static,
) -> Result<T> {
    let history = history.clone();
    tokio::task::spawn_blocking(move || operation(&mut history.lock().unwrap())).await?
}

fn warn_retry(result: Result<()>, last: &mut Option<Instant>) {
    if let Err(error) = result
        && last.is_none_or(|t| t.elapsed() >= std::time::Duration::from_secs(60))
    {
        tracing::warn!(%error, "History I/O failed; will retry (warnings limited to once per minute)");
        *last = Some(Instant::now());
    }
}

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// Limits in the `history` map of noded/config.conf.mix (or --mesh-config).
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HistoryConfig {
    pub retention_days: u32,
    pub max_file_bytes: u64,
    pub max_total_bytes: u64,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            retention_days: 30,
            max_file_bytes: 64 * 1024 * 1024,
            max_total_bytes: 1024 * 1024 * 1024,
        }
    }
}

impl HistoryConfig {
    pub fn load(explicit: Option<&str>) -> Option<Self> {
        #[derive(Default, serde::Deserialize)]
        struct Config {
            #[serde(default)]
            history: HistoryConfig,
        }
        let path = explicit
            .map(PathBuf::from)
            .unwrap_or_else(|| config::path(config::Dir::Etc).join("noded/config.conf.mix"));
        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) if explicit.is_none() && error.kind() == std::io::ErrorKind::NotFound => {
                return Some(Self::default());
            }
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "History disabled: config cannot be read");
                return None;
            }
        };
        let parsed = config::from_conf_mix_str::<Config>(&source)
            .map_err(anyhow::Error::from)
            .and_then(|c| {
                anyhow::ensure!(
                    c.history.retention_days > 0
                        && c.history.max_file_bytes > 0
                        && c.history.max_total_bytes > 0,
                    "history limits must be positive"
                );
                Ok(c.history)
            });
        Some(match parsed {
            Ok(limits) => limits,
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "Invalid history config: using history defaults");
                Self::default()
            }
        })
    }
}

struct History {
    dir: PathBuf,
    node: String,
    limits: HistoryConfig,
    writer: BufWriter<Box<dyn Write + Send>>,
    bytes: u64,
    total: u64,
    day: String,
    dropped: u64,
    // Fields drop in declaration order: keep the lock until the writer drops.
    _lock: File,
}

fn timestamp(now: DateTime<Utc>) -> String {
    now.to_rfc3339_opts(datetime::SecondsFormat::Millis, true)
}

fn private_file(path: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    anyhow::ensure!(
        file.metadata()?.is_file(),
        "history path is not a regular file"
    );
    file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    Ok(file)
}

fn archive_key(name: &str) -> Option<(datetime::NaiveDate, u64)> {
    let stem = name.strip_prefix("bus-")?.strip_suffix(".jsonl")?;
    let date = datetime::NaiveDate::parse_from_str(stem.get(..10)?, "%Y-%m-%d").ok()?;
    let suffix = &stem[10..];
    let index = if suffix.is_empty() {
        0
    } else {
        suffix.strip_prefix('-')?.parse().ok()?
    };
    Some((date, index))
}

impl History {
    fn open(dir: &Path, node: String, limits: HistoryConfig, now: DateTime<Utc>) -> Result<Self> {
        std::fs::create_dir_all(dir)?;
        let lock = private_file(&dir.join("bus.lock"))?;
        // SAFETY: flock borrows a valid fd and does not retain a Rust pointer.
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let path = dir.join("bus.jsonl");
        let mut file = private_file(&path)?;
        if file.metadata()?.nlink() > 1 {
            // A previous rotation linked the archive but crashed before unlinking.
            // Never truncate or append to that completed archive inode.
            std::fs::remove_file(&path)?;
            file = private_file(&path)?;
        }
        // Stream one row at a time: the last row may exceed a fixed tail window.
        file.seek(SeekFrom::Start(0))?;
        let mut reader = BufReader::new(&file);
        let mut line = Vec::new();
        let mut complete = 0;
        let mut last = None;
        loop {
            line.clear();
            let n = reader.read_until(b'\n', &mut line)?;
            if n == 0 || line.last() != Some(&b'\n') {
                break;
            }
            complete += n as u64;
            if let Ok(row) = serde_json::from_slice::<serde_json::Value>(&line) {
                last = row["ts_utc"].as_str().map(str::to_owned).or(last);
            }
        }
        drop(reader);
        // Modification time is the recording clock, not a possibly old event ts.
        let modified = file
            .metadata()?
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)?;
        let day = DateTime::from_timestamp(modified.as_secs() as i64, 0)
            .unwrap_or(now)
            .format("%Y-%m-%d")
            .to_string();
        file.set_len(complete)?;
        let bytes = file.metadata()?.len();
        let mut history = Self {
            _lock: lock,
            dir: dir.to_owned(),
            node,
            limits,
            writer: BufWriter::new(Box::new(file)),
            bytes,
            total: bytes,
            day,
            dropped: 0,
        };
        history.retain(now, 0)?;
        history.row(
            serde_json::json!({"kind": "gap", "reason": "logger_start", "since_last": last}),
            now,
        )?;
        history.flush()?;
        Ok(history)
    }

    fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    fn tick(&mut self, now: DateTime<Utc>) -> Result<()> {
        let maintained = self.maintain(now);
        let flushed = self.flush();
        maintained.and(flushed)
    }

    fn rotate(&mut self, now: DateTime<Utc>) -> Result<()> {
        self.flush()?;
        if self.bytes > 0 {
            let base = format!("bus-{}", self.day);
            // Retention can remove the base and low indices. Never fill those
            // holes: the newest archive must sort after every surviving one.
            let mut suffix = 0u64;
            for entry in std::fs::read_dir(&self.dir)? {
                let entry = entry?;
                if let Some((date, index)) = archive_key(&entry.file_name().to_string_lossy())
                    && date.to_string() == self.day
                {
                    suffix = suffix.max(
                        index
                            .checked_add(1)
                            .ok_or_else(|| anyhow::anyhow!("history archive index exhausted"))?,
                    );
                }
            }
            loop {
                let name = if suffix == 0 {
                    format!("{base}.jsonl")
                } else {
                    format!("{base}-{suffix:08}.jsonl")
                };
                let archive = self.dir.join(name);
                // link is atomic and refuses collisions; never overwrite an archive.
                match std::fs::hard_link(self.dir.join("bus.jsonl"), &archive) {
                    Ok(()) => break,
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                        suffix = suffix
                            .checked_add(1)
                            .ok_or_else(|| anyhow::anyhow!("history archive index exhausted"))?;
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            std::fs::remove_file(self.dir.join("bus.jsonl"))?;
        }
        self.writer = BufWriter::new(Box::new(private_file(&self.dir.join("bus.jsonl"))?));
        self.bytes = 0;
        self.day = now.format("%Y-%m-%d").to_string();
        self.retain(now, 0)?;
        Ok(())
    }

    fn maintain(&mut self, now: DateTime<Utc>) -> Result<()> {
        if self.day != now.format("%Y-%m-%d").to_string() {
            self.rotate(now)?;
        }
        self.retain(now, 0)
    }

    fn retain(&mut self, now: DateTime<Utc>, incoming: u64) -> Result<()> {
        let mut files = Vec::new();
        let mut total = self.bytes.saturating_add(incoming);
        for entry in std::fs::read_dir(&self.dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some((date, index)) = archive_key(&name) else {
                continue;
            };
            if !entry.file_type()?.is_file() {
                continue;
            }
            let size = entry.metadata()?.len();
            total = total.saturating_add(size);
            files.push((date, index, entry.path(), size));
        }
        files.sort_by_key(|(date, index, _, _)| (*date, *index));
        for (date, _, path, size) in files {
            if (now.date_naive() - date).num_days() >= i64::from(self.limits.retention_days)
                || total > self.limits.max_total_bytes
            {
                std::fs::remove_file(path)?;
                total = total.saturating_sub(size);
            }
        }
        self.total = total.saturating_sub(incoming);
        Ok(())
    }

    fn row(&mut self, mut row: serde_json::Value, now: DateTime<Utc>) -> Result<u64> {
        row["v"] = 1.into();
        row["node"] = self.node.clone().into();
        if row.get("ts_utc").is_none() {
            row["ts_utc"] = timestamp(now).into();
        }
        let mut line = serde_json::to_vec(&row)?;
        line.push(b'\n');
        let size = line.len() as u64;
        if size > self.limits.max_file_bytes || size > self.limits.max_total_bytes {
            if row["reason"] == "row_too_large" {
                // Even the marker cannot fit an operator's tiny positive cap.
                return Ok(0);
            }
            return self.row(
                serde_json::json!({"kind":"dropped", "reason":"row_too_large", "count":1}),
                now,
            );
        }
        if self.day != now.format("%Y-%m-%d").to_string()
            || self.bytes + size > self.limits.max_file_bytes
            || self.bytes + size > self.limits.max_total_bytes
        {
            self.rotate(now)?;
        }
        if self.total.saturating_add(size) > self.limits.max_total_bytes {
            self.retain(now, size)?;
        }
        self.writer.write_all(&line)?;
        self.bytes += size;
        self.total += size;
        Ok(size)
    }

    fn event(&mut self, event: ObserveEvent, now: DateTime<Utc>) -> Result<u64> {
        let ts = DateTime::parse_from_rfc3339(&event.ts)
            .map(|t| t.with_timezone(&Utc))
            .unwrap_or(now);
        let mut bytes = 0;
        if event.dropped_count > self.dropped {
            bytes += self.row(
                serde_json::json!({"kind": "dropped", "count": event.dropped_count - self.dropped}),
                now,
            )?;
        }
        self.dropped = event.dropped_count;
        let mut row = serde_json::json!({
            "kind": "message", "ts_utc": timestamp(ts), "seq": event.seq,
            "direction": event.direction, "outcome": event.outcome, "message_type": event.message_type,
            "from": event.from, "to": event.to, "command": event.command, "size": event.size,
            "correlation_id": event.correlation_id, "rc": event.rc, "dropped_count": event.dropped_count
        });
        if let Some(peer) = event.peer {
            row["peer"] = peer.into();
        }
        bytes += self.row(row, now)?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::sync::atomic::AtomicU64;

    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "noded-history-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn at(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }
    fn rows(dir: &Path) -> Vec<Value> {
        std::fs::read_to_string(dir.join("bus.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
    fn sample(ts: &str, dropped: u64) -> ObserveEvent {
        // Same metadata shape as observe::EventBody; unexpected body fields are never persisted.
        serde_json::from_value(json!({"seq": 42, "ts": ts, "direction": "mesh_in",
            "outcome": "delivered", "message_type": "request", "from": "ced.alpha.bus",
            "to": "dopus.beta.bus", "command": "dopus.open", "size": 123,
            "correlation_id": "example-id", "peer": "alpha", "rc": 0,
            "dropped_count": dropped, "payload": {"body": "never stored"}, "payload_omitted": null})).unwrap()
    }

    #[test]
    fn format_gaps_drops_and_utc_clock() {
        let tmp = Temp::new();
        let now = at("2026-11-25T01:02:03.456Z");
        let mut h = History::open(&tmp.0, "beta".into(), HistoryConfig::default(), now).unwrap();
        h.event(sample("2026-11-25T11:02:03.456+10:00", 2), now)
            .unwrap();
        h.event(sample("", 2), now).unwrap();
        h.event(sample("", 5), now).unwrap();
        h.flush().unwrap();
        let r = rows(&tmp.0);
        assert_eq!(
            r[0],
            json!({"v":1,"ts_utc":"2026-11-25T01:02:03.456Z","node":"beta","kind":"gap","reason":"logger_start","since_last":null})
        );
        assert_eq!(r[1]["kind"], "dropped");
        assert_eq!(r[1]["count"], 2);
        assert_eq!(
            r[2],
            json!({"v":1,"ts_utc":"2026-11-25T01:02:03.456Z","node":"beta","seq":42,"kind":"message",
            "direction":"mesh_in","outcome":"delivered","message_type":"request","from":"ced.alpha.bus",
            "to":"dopus.beta.bus","command":"dopus.open","size":123,"correlation_id":"example-id","peer":"alpha","rc":0,"dropped_count":2})
        );
        assert_eq!(r[3]["ts_utc"], "2026-11-25T01:02:03.456Z");
        assert_eq!(r[4]["count"], 3);
        assert_eq!(r.len(), 6);
        assert_eq!(
            std::fs::metadata(tmp.0.join("bus.jsonl"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        drop(h);
        let _h = History::open(&tmp.0, "beta".into(), HistoryConfig::default(), now).unwrap();
        assert_eq!(
            rows(&tmp.0).last().unwrap()["since_last"],
            "2026-11-25T01:02:03.456Z"
        );
    }

    #[test]
    fn size_and_idle_utc_day_rotation_preserve_rows() {
        let tmp = Temp::new();
        let now = at("2026-11-25T23:59:59Z");
        let mut h = History::open(
            &tmp.0,
            "beta".into(),
            HistoryConfig {
                max_file_bytes: 400,
                ..HistoryConfig::default()
            },
            now,
        )
        .unwrap();
        h.event(sample("", 0), now).unwrap();
        h.event(sample("", 0), now).unwrap();
        h.flush().unwrap();
        assert!(tmp.0.join("bus-2026-11-25.jsonl").exists());
        assert!(tmp.0.join("bus-2026-11-25-00000001.jsonl").exists());
        let tomorrow = at("2026-11-26T00:00:00Z");
        h.maintain(tomorrow).unwrap();
        assert!(rows(&tmp.0).is_empty());
        assert!(tmp.0.join("bus-2026-11-25-00000002.jsonl").exists());
        for entry in std::fs::read_dir(&tmp.0).unwrap() {
            let meta = entry.unwrap().metadata().unwrap();
            assert!(meta.len() <= 400);
            assert_eq!(meta.permissions().mode() & 0o777, 0o600);
        }
    }

    #[test]
    fn retention_by_calendar_age_and_total_bytes_oldest_first() {
        let tmp = Temp::new();
        let now = at("2026-11-25T00:00:00Z");
        let mut h = History::open(&tmp.0, "beta".into(), HistoryConfig::default(), now).unwrap();
        for name in [
            "bus-2026-10-26.jsonl",
            "bus-2026-11-24.jsonl",
            "bus-2026-11-24-00000001.jsonl",
            "bus-2026-11-25.jsonl",
        ] {
            std::fs::write(tmp.0.join(name), [b'x'; 100]).unwrap();
        }
        std::fs::write(tmp.0.join("unrelated.jsonl"), [b'x'; 100]).unwrap();
        h.retain(now, 0).unwrap();
        assert!(!tmp.0.join("bus-2026-10-26.jsonl").exists());
        assert!(tmp.0.join("bus-2026-11-24.jsonl").exists());
        h.limits.max_total_bytes = h.bytes + 150;
        h.retain(now, 0).unwrap();
        assert!(!tmp.0.join("bus-2026-11-24.jsonl").exists());
        assert!(!tmp.0.join("bus-2026-11-24-00000001.jsonl").exists());
        assert!(tmp.0.join("bus-2026-11-25.jsonl").exists());
        assert!(tmp.0.join("unrelated.jsonl").exists());
    }

    #[test]
    fn units_enable_history_and_keep_monitor_disabled() {
        for unit in [
            include_str!("../units/noded.service"),
            include_str!("../units/noded@.service"),
        ] {
            assert!(!unit.contains("--no-log"));
            assert!(unit.contains("--no-monitor"));
        }
    }

    #[test]
    fn config_defaults_and_overrides() {
        #[derive(serde::Deserialize)]
        struct Config {
            history: HistoryConfig,
        }
        let c: Config = config::from_conf_mix_str(
            "history: { retention_days: 7, max_file_bytes: 1024, max_total_bytes: 4096 }",
        )
        .unwrap();
        assert_eq!(c.history.retention_days, 7);
        assert_eq!(c.history.max_file_bytes, 1024);
        assert_eq!(c.history.max_total_bytes, 4096);
        assert_eq!(HistoryConfig::default().retention_days, 30);
    }

    #[test]
    fn directory_lock_refuses_second_writer_without_touching_first() {
        let tmp = Temp::new();
        let now = Utc::now();
        let mut first =
            History::open(&tmp.0, "beta".into(), HistoryConfig::default(), now).unwrap();
        assert!(History::open(&tmp.0, "beta".into(), HistoryConfig::default(), now).is_err());
        first.event(sample("", 0), now).unwrap();
        first.flush().unwrap();
        assert_eq!(rows(&tmp.0).len(), 2);
        drop(first);
        assert!(History::open(&tmp.0, "beta".into(), HistoryConfig::default(), now).is_ok());
    }

    #[test]
    fn oversized_row_is_marked_and_next_message_is_recorded() {
        let tmp = Temp::new();
        let now = Utc::now();
        let mut h = History::open(
            &tmp.0,
            "beta".into(),
            HistoryConfig {
                max_file_bytes: 2048,
                ..HistoryConfig::default()
            },
            now,
        )
        .unwrap();
        let mut event = sample("", 0);
        event.command = Some("x".repeat(4096));
        h.event(event, now).unwrap();
        h.event(sample("", 0), now).unwrap();
        h.flush().unwrap();
        let r = rows(&tmp.0);
        assert_eq!(r[1]["kind"], "dropped");
        assert_eq!(r[1]["reason"], "row_too_large");
        assert_eq!(r[1]["count"], 1);
        assert_eq!(r[2]["command"], "dopus.open");
    }

    #[test]
    fn ordinary_rows_do_not_scan_retention_and_maintenance_recovers() {
        let tmp = Temp::new();
        let now = Utc::now();
        let mut h = History::open(&tmp.0, "beta".into(), HistoryConfig::default(), now).unwrap();
        let old = tmp.0.join("bus-2000-01-01.jsonl");
        std::fs::write(&old, "old\n").unwrap();
        h.event(sample("", 0), now).unwrap();
        assert!(old.exists(), "ordinary rows must not scan archives");
        let saved = tmp.0.with_extension("saved");
        std::fs::rename(&tmp.0, &saved).unwrap();
        assert!(h.tick(now).is_err());
        std::fs::rename(saved, &tmp.0).unwrap();
        h.tick(now).unwrap();
        assert!(!old.exists());
        h.event(sample("", 0), now).unwrap();
        h.flush().unwrap();
        assert_eq!(rows(&tmp.0).len(), 3);
    }

    #[test]
    fn transient_flush_is_retried() {
        struct FailOnce(bool);
        impl Write for FailOnce {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                if std::mem::take(&mut self.0) {
                    return Err(std::io::Error::other("injected flush failure"));
                }
                Ok(())
            }
        }
        let tmp = Temp::new();
        let now = Utc::now();
        let mut h = History::open(&tmp.0, "beta".into(), HistoryConfig::default(), now).unwrap();
        h.writer = BufWriter::new(Box::new(FailOnce(true)));
        let mut last = None;
        warn_retry(h.tick(now), &mut last);
        let warned = last.unwrap();
        warn_retry(Err(anyhow::anyhow!("second error")), &mut last);
        assert_eq!(last.unwrap(), warned);
        h.tick(now).unwrap();
        h.event(sample("", 0), now).unwrap();
        h.flush().unwrap();
    }

    #[test]
    fn restart_after_link_preserves_completed_archive() {
        let tmp = Temp::new();
        let now = Utc::now();
        let mut h = History::open(&tmp.0, "beta".into(), HistoryConfig::default(), now).unwrap();
        h.event(sample("", 0), now).unwrap();
        h.flush().unwrap();
        drop(h);
        let active = tmp.0.join("bus.jsonl");
        let archive = tmp.0.join(format!("bus-{}.jsonl", now.format("%Y-%m-%d")));
        std::fs::hard_link(&active, &archive).unwrap();
        let original = std::fs::read(&archive).unwrap();
        let mut h = History::open(&tmp.0, "beta".into(), HistoryConfig::default(), now).unwrap();
        h.event(sample("", 0), now).unwrap();
        h.flush().unwrap();
        assert_eq!(std::fs::read(&archive).unwrap(), original);
        assert_ne!(
            std::fs::metadata(active).unwrap().ino(),
            std::fs::metadata(archive).unwrap().ino()
        );
        assert_eq!(rows(&tmp.0).len(), 2);
    }

    struct BlockingWriter {
        entered: Option<tokio::sync::oneshot::Sender<()>>,
        release: std::sync::mpsc::Receiver<()>,
    }
    impl Write for BlockingWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            if let Some(entered) = self.entered.take() {
                let _ = entered.send(());
                self.release.recv().unwrap();
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn shutdown_is_bounded_with_blocking_writer() {
        let tmp = Temp::new();
        let mut h =
            History::open(&tmp.0, "beta".into(), HistoryConfig::default(), Utc::now()).unwrap();
        let (entered, waiting) = tokio::sync::oneshot::channel();
        let (release, blocked) = std::sync::mpsc::channel();
        h.writer = BufWriter::new(Box::new(BlockingWriter {
            entered: Some(entered),
            release: blocked,
        }));
        let h = Arc::new(std::sync::Mutex::new(h));
        let task = tokio::spawn(async move {
            let _ = disk(&h, History::flush).await;
        });
        waiting.await.unwrap();
        let abort = task.abort_handle();
        let started = Instant::now();
        crate::stop_logger(task).await;
        let elapsed = started.elapsed();
        // Release even on failure, so the test runtime cannot hang in Drop.
        release.send(()).unwrap();
        assert!(elapsed >= std::time::Duration::from_secs(2));
        assert!(elapsed < std::time::Duration::from_secs(3));
        tokio::task::yield_now().await;
        assert!(abort.is_finished());
    }

    #[tokio::test]
    async fn bad_history_config_warns_and_broker_starts() {
        use tracing_subscriber::prelude::*;
        #[derive(Clone)]
        struct Capture(Arc<std::sync::Mutex<Vec<u8>>>);
        impl Write for Capture {
            fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(b);
                Ok(b.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let output = Capture(Arc::new(std::sync::Mutex::new(Vec::new())));
        let capture = output.clone();
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_writer(move || capture.clone())
            .finish();
        let _guard = subscriber.set_default();
        let tmp = Temp::new();
        let path = tmp.0.join("config.conf.mix");
        for (case, block, broker_starts) in [
            // An unclosed map invalidates the whole mesh file, so main's
            // broker loader rejects it even though history uses defaults.
            ("invalid mesh syntax", Some("history: {"), false),
            ("invalid history shape", Some("history: wrong"), true),
            ("zero limit", Some("history: { max_file_bytes: 0 }"), true),
            (
                "negative limit",
                Some("history: { retention_days: -1 }"),
                true,
            ),
            ("unknown key", Some("history: { unknown: 1 }"), true),
            (
                "wrong type",
                Some("history: { max_file_bytes: wrong }"),
                true,
            ),
            (
                "fractional limit",
                Some("history: { retention_days: 1.5 }"),
                true,
            ),
            // Missing and unreadable explicit files retain main's fatal
            // mesh-config behaviour; only history disables itself.
            ("missing explicit config", None, false),
            ("unreadable explicit config", None, false),
        ] {
            output.0.lock().unwrap().clear();
            if let Some(block) = block {
                std::fs::write(&path, format!("node_name: beta\n{block}\n")).unwrap();
                assert_eq!(
                    HistoryConfig::load(path.to_str()).unwrap().max_file_bytes,
                    HistoryConfig::default().max_file_bytes
                );
            } else {
                if case == "unreadable explicit config" {
                    std::fs::create_dir(&path).unwrap();
                } else {
                    std::fs::remove_file(&path).unwrap();
                }
                assert!(HistoryConfig::load(path.to_str()).is_none());
            }
            let warnings = String::from_utf8(output.0.lock().unwrap().clone()).unwrap();
            let expected = if block.is_some() {
                "Invalid history config: using history defaults"
            } else {
                "History disabled: config cannot be read"
            };
            assert!(warnings.contains(expected), "{case}: {warnings}");
            if case == "zero limit" {
                assert!(
                    warnings.contains("history limits must be positive"),
                    "{warnings}"
                );
            }
            let mesh_error = mesh::MeshConfig::load(path.to_str().unwrap()).err();
            assert_eq!(mesh_error.is_none(), broker_starts, "{case}");
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let listen = listener.local_addr().unwrap().to_string();
            let (ready, waiting) = tokio::sync::oneshot::channel();
            let cfg = crate::noded::RunConfig {
                session_probe: None,
                unix_socket: None,
                pending_grants_per_parent: 32,
                listen: listen.clone(),
                node: "beta".into(),
                wg_ip: "127.0.0.1".into(),
                mesh_config_path: Some(path.to_string_lossy().into_owned()),
                spec_dir: None,
                admission_mode: config::node::AdmissionMode::Off,
                frame_policy: config::node::AdmissionMode::Observe,
            };
            let broker = tokio::spawn(crate::noded::run_on(cfg, listener, ready));
            if !broker_starts {
                let error = tokio::time::timeout(std::time::Duration::from_secs(5), broker)
                    .await
                    .unwrap()
                    .unwrap()
                    .expect_err(
                        "main's broker policy rejects an unavailable or invalid mesh config",
                    );
                assert_eq!(error.to_string(), mesh_error.unwrap().to_string(), "{case}");
                assert!(waiting.await.is_err(), "broker must not signal readiness");
                continue;
            }
            tokio::time::timeout(std::time::Duration::from_secs(5), waiting)
                .await
                .unwrap()
                .unwrap();
            let client =
                bus::native_client::NodedClient::connect("probe", &format!("ws://{listen}/ws"))
                    .await
                    .unwrap();
            assert!(
                client
                    .call("noded", "noded.ping", json!({}))
                    .await
                    .unwrap()
                    .get("pong")
                    .is_some()
            );
            broker.abort();
        }
    }

    #[tokio::test]
    async fn native_observation_flushes_on_timer_and_shutdown() {
        let tmp = Temp::new();
        let broker = test_broker::Broker::start();
        let (stop, shutdown) = tokio::sync::oneshot::channel();
        let url = broker.url.clone();
        let dir = tmp.0.clone();
        let logger = tokio::spawn(async move {
            run_in(
                &url,
                "beta".into(),
                HistoryConfig::default(),
                shutdown,
                &dir,
            )
            .await
        });
        let client = bus::native_client::NodedClient::connect("example-probe", &broker.url)
            .await
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                client.call("noded", "noded.ping", json!({})).await.unwrap();
                if tmp.0.join("bus.jsonl").exists()
                    && rows(&tmp.0).iter().any(|r| r["command"] == "noded.ping")
                {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        })
        .await
        .unwrap();
        // The second instance must return before attempting a service
        // registration, leaving the first observer alive and its file intact.
        let (_second_stop, second_shutdown) = tokio::sync::oneshot::channel();
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            run_in(
                &broker.url,
                "beta".into(),
                HistoryConfig::default(),
                second_shutdown,
                &tmp.0,
            ),
        )
        .await
        .unwrap()
        .unwrap();
        client.call("noded", "noded.ping", json!({})).await.unwrap();
        stop.send(()).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), logger)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let r = rows(&tmp.0);
        assert_eq!(r.first().unwrap()["reason"], "logger_start");
        assert_eq!(r.last().unwrap()["reason"], "observer_closed");
        assert_eq!(
            r.iter().filter(|r| r["reason"] == "logger_start").count(),
            1
        );
        assert!(r.iter().all(|r| r.get("payload").is_none()));
    }

    #[test]
    fn restart_recovers_long_row_and_truncated_tail() {
        let tmp = Temp::new();
        let now = Utc::now();
        let mut h = History::open(&tmp.0, "beta".into(), HistoryConfig::default(), now).unwrap();
        h.row(json!({"kind":"message", "command":"x".repeat(70000)}), now)
            .unwrap();
        h.flush().unwrap();
        drop(h);
        OpenOptions::new()
            .append(true)
            .open(tmp.0.join("bus.jsonl"))
            .unwrap()
            .write_all(b"{\"partial\"")
            .unwrap();
        let _h = History::open(&tmp.0, "beta".into(), HistoryConfig::default(), now).unwrap();
        let r = rows(&tmp.0);
        assert_eq!(r.len(), 3);
        assert_eq!(r[2]["since_last"], timestamp(now));
    }

    #[test]
    fn total_cap_includes_current_file_and_pending_row() {
        let tmp = Temp::new();
        let now = Utc::now();
        let mut h = History::open(
            &tmp.0,
            "beta".into(),
            HistoryConfig {
                max_total_bytes: 400,
                ..HistoryConfig::default()
            },
            now,
        )
        .unwrap();
        for _ in 0..10 {
            h.event(sample("", 0), now).unwrap();
        }
        h.flush().unwrap();
        let total: u64 = std::fs::read_dir(&tmp.0)
            .unwrap()
            .map(|e| e.unwrap().metadata().unwrap().len())
            .sum();
        assert!(total <= 400);
        assert_eq!(rows(&tmp.0).len(), 1);
    }

    #[test]
    fn rotation_never_reuses_deleted_older_archive_indices() {
        let tmp = Temp::new();
        let now = at("2026-11-25T00:00:00Z");
        let mut h = History::open(&tmp.0, "beta".into(), HistoryConfig::default(), now).unwrap();
        h.rotate(now).unwrap();
        h.event(sample("", 0), now).unwrap();
        h.rotate(now).unwrap();
        std::fs::remove_file(tmp.0.join("bus-2026-11-25.jsonl")).unwrap();
        h.event(sample("", 0), now).unwrap();
        h.rotate(now).unwrap();
        assert!(!tmp.0.join("bus-2026-11-25.jsonl").exists());
        let oldest = tmp.0.join("bus-2026-11-25-00000001.jsonl");
        let newest = tmp.0.join("bus-2026-11-25-00000002.jsonl");
        assert!(oldest.exists() && newest.exists());
        h.limits.max_total_bytes = std::fs::metadata(&newest).unwrap().len();
        h.retain(now, 0).unwrap();
        assert!(!oldest.exists());
        assert!(newest.exists());
    }
}
