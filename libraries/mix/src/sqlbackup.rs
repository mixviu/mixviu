// SPDX-License-Identifier: MIT OR Apache-2.0
//! Online SQLite backup, staged so a failed copy never replaces a good backup.
use super::*;
use rusqlite::{
    Connection, ErrorCode, OpenFlags,
    backup::{Backup, StepResult},
};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn sql_error(e: rusqlite::Error) -> MixError {
    let code = match e.sqlite_error_code() {
        Some(ErrorCode::DatabaseBusy) => "SQL_BUSY",
        Some(ErrorCode::DatabaseLocked) => "SQL_LOCKED",
        Some(
            ErrorCode::SystemIoFailure
            | ErrorCode::CannotOpen
            | ErrorCode::DiskFull
            | ErrorCode::ReadOnly,
        ) => "SQL_IO",
        _ => "SQL_ERROR",
    };
    MixError::structured(code, format!("sqlbackup: {e}"))
}

fn io_error(e: std::io::Error) -> MixError {
    MixError::structured("SQL_IO", format!("sqlbackup: {e}"))
}

struct Staging(PathBuf);
impl Drop for Staging {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub(super) fn builtin_sqlbackup(args: Vec<Value>) -> MixResult<Option<Value>> {
    expect_args_between("sqlbackup", &args, 2, 3)?;
    let path_arg = |i: usize| match &args[i] {
        Value::String(s) if !s.is_empty() && s != ":memory:" && !s.contains('\0') => Ok(s.as_str()),
        _ => Err(opt_invalid(
            "sqlbackup()",
            "paths must be non-empty filesystem strings",
        )),
    };
    let src = Path::new(path_arg(0)?);
    let dst = Path::new(path_arg(1)?);
    let mut overwrite = false;
    let mut mode = 0o600;
    let mut retry_ms = 5000;
    if let Some(opts) = args.get(2) {
        let Value::Map(opts) = opts else {
            return Err(opt_invalid("sqlbackup()", "opts must be a map"));
        };
        for (key, value) in opts.iter() {
            match (key.as_str(), value) {
                ("overwrite", Value::Bool(v)) => overwrite = *v,
                ("mode", Value::Number(v))
                    if v.is_finite() && v.fract() == 0.0 && (0.0..=511.0).contains(v) =>
                {
                    mode = *v as u32
                }
                ("retry_ms", Value::Number(v))
                    if v.is_finite() && v.fract() == 0.0 && (0.0..=60000.0).contains(v) =>
                {
                    retry_ms = *v as u64
                }
                _ => return Err(opt_invalid("sqlbackup()", format!("invalid option {key}"))),
            }
        }
    }
    let started = Instant::now();
    let metadata = std::fs::metadata(src).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            MixError::structured("SQL_NO_DATABASE", "sqlbackup: source does not exist")
        } else {
            io_error(e)
        }
    })?;
    if !metadata.is_file() {
        return Err(MixError::structured(
            "SQL_IO",
            "sqlbackup: source is not a regular file",
        ));
    }
    match std::fs::symlink_metadata(dst) {
        Ok(_) if !overwrite => {
            return Err(MixError::structured(
                "SQL_DEST_EXISTS",
                "sqlbackup: destination exists",
            ));
        }
        Ok(m) => {
            // Replacing a symlink or live database risks modifying unrelated data.
            if !m.is_file()
                || std::fs::canonicalize(src).map_err(io_error)?
                    == std::fs::canonicalize(dst).map_err(io_error)?
            {
                return Err(opt_invalid(
                    "sqlbackup()",
                    "destination must be a separate regular file",
                ));
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(io_error(e)),
    }
    for suffix in ["-wal", "-shm", "-journal"] {
        let mut sidecar = dst.as_os_str().to_os_string();
        sidecar.push(suffix);
        if std::fs::symlink_metadata(sidecar).is_ok() {
            return Err(opt_invalid(
                "sqlbackup()",
                "destination has SQLite sidecars; stop its writer first",
            ));
        }
    }
    let source =
        Connection::open_with_flags(src, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(sql_error)?;
    source.busy_timeout(Duration::ZERO).map_err(sql_error)?;
    // Pin the read snapshot so a constantly changing WAL cannot restart the copy forever.
    source.execute_batch("BEGIN").map_err(sql_error)?;
    let deadline = Instant::now() + Duration::from_millis(retry_ms);
    loop {
        match source.query_row("SELECT count(*) FROM sqlite_schema", [], |r| {
            r.get::<_, i64>(0)
        }) {
            Ok(_) => break,
            Err(e)
                if matches!(
                    e.sqlite_error_code(),
                    Some(ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked)
                ) && Instant::now() < deadline =>
            {
                std::thread::sleep(
                    Duration::from_millis(10)
                        .min(deadline.saturating_duration_since(Instant::now())),
                );
            }
            Err(e) => return Err(sql_error(e)),
        }
    }
    let parent = dst
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let (staging, file) = loop {
        let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = parent.join(format!(".mix-sqlbackup-{}-{id}", std::process::id()));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(file) => break (Staging(path), file),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(io_error(e)),
        }
    };
    let mut destination = Connection::open(&staging.0).map_err(sql_error)?;
    destination
        .busy_timeout(Duration::ZERO)
        .map_err(sql_error)?;
    let pages;
    {
        let backup = Backup::new(&source, &mut destination).map_err(sql_error)?;
        loop {
            match backup.step(128).map_err(sql_error)? {
                StepResult::Done => {
                    pages = backup.progress().pagecount;
                    break;
                }
                StepResult::More => {}
                status => {
                    if Instant::now() >= deadline {
                        let code = if matches!(status, StepResult::Locked) {
                            "SQL_LOCKED"
                        } else {
                            "SQL_BUSY"
                        };
                        return Err(MixError::structured(
                            code,
                            "sqlbackup: retry budget exhausted",
                        ));
                    }
                    std::thread::sleep(
                        Duration::from_millis(10)
                            .min(deadline.saturating_duration_since(Instant::now())),
                    );
                }
            }
        }
    }
    // A WAL source copies its journal setting too. Publish a standalone file.
    destination
        .execute_batch("PRAGMA journal_mode=DELETE")
        .map_err(sql_error)?;
    let mut check = destination
        .prepare("PRAGMA integrity_check")
        .map_err(sql_error)?;
    let integrity: Vec<String> = check
        .query_map([], |r| r.get(0))
        .map_err(sql_error)?
        .collect::<Result<_, _>>()
        .map_err(sql_error)?;
    if integrity != ["ok"] {
        return Err(MixError::structured(
            "SQL_INTEGRITY",
            format!("sqlbackup: {}", integrity.join("; ")),
        ));
    }
    drop(check);
    destination.close().map_err(|(_, e)| sql_error(e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(mode))
            .map_err(io_error)?;
    }
    #[cfg(not(unix))]
    let _ = mode;
    file.sync_all().map_err(io_error)?;
    let bytes = file.metadata().map_err(io_error)?.len();
    if overwrite {
        std::fs::rename(&staging.0, dst).map_err(io_error)?;
    } else {
        std::fs::hard_link(&staging.0, dst).map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                MixError::structured("SQL_DEST_EXISTS", "sqlbackup: destination exists")
            } else {
                io_error(e)
            }
        })?;
    }
    Ok(Some(Value::map(
        [
            ("ok".into(), Value::Bool(true)),
            ("pages".into(), Value::Number(pages as f64)),
            ("bytes".into(), Value::Number(bytes as f64)),
            ("integrity".into(), Value::String("ok".into())),
            (
                "duration_ms".into(),
                Value::Number(started.elapsed().as_secs_f64() * 1000.0),
            ),
        ]
        .into_iter()
        .collect(),
    )))
}
