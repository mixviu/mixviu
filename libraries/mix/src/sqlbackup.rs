// SPDX-License-Identifier: MIT OR Apache-2.0
//! Online SQLite backup, staged so a failed copy never replaces a good backup.
use super::*;
use rusqlite::{Connection, ErrorCode, OpenFlags, backup::StepResult, ffi};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

// rusqlite 0.41's Backup exposes no finish(), and Drop discards its result.
// Own the same native handle here so finish errors are checked exactly once.
struct CheckedBackup<'a> {
    raw: Option<std::ptr::NonNull<ffi::sqlite3_backup>>,
    _source: &'a Connection,
    _destination: &'a mut Connection,
}

fn sqlite_result(rc: i32) -> rusqlite::Result<()> {
    if rc == ffi::SQLITE_OK { Ok(()) } else {
        Err(rusqlite::Error::SqliteFailure(ffi::Error::new(rc), None))
    }
}

impl<'a> CheckedBackup<'a> {
    fn new(source: &'a Connection, destination: &'a mut Connection) -> rusqlite::Result<Self> {
        // SAFETY: both connections outlive this handle. The exclusive destination
        // borrow prevents other SQLite operations until finish/drop releases it.
        let raw = unsafe { ffi::sqlite3_backup_init(destination.handle(), c"main".as_ptr(), source.handle(), c"main".as_ptr()) };
        let raw = std::ptr::NonNull::new(raw).ok_or_else(|| {
            rusqlite::Error::SqliteFailure(ffi::Error::new(unsafe { ffi::sqlite3_errcode(destination.handle()) }), None)
        })?;
        Ok(Self { raw: Some(raw), _source: source, _destination: destination })
    }

    fn step(&self, pages: i32) -> rusqlite::Result<StepResult> {
        // SAFETY: raw remains live until finish/drop, with connections borrowed.
        match unsafe { ffi::sqlite3_backup_step(self.raw.unwrap().as_ptr(), pages) } {
            ffi::SQLITE_DONE => Ok(StepResult::Done),
            ffi::SQLITE_OK => Ok(StepResult::More),
            ffi::SQLITE_BUSY => Ok(StepResult::Busy),
            ffi::SQLITE_LOCKED => Ok(StepResult::Locked),
            rc => sqlite_result(rc).map(|()| StepResult::More),
        }
    }

    fn finish(mut self) -> rusqlite::Result<()> {
        // SAFETY: take ensures Drop cannot finish the handle a second time.
        sqlite_result(unsafe { ffi::sqlite3_backup_finish(self.raw.take().unwrap().as_ptr()) })
    }
}

impl Drop for CheckedBackup<'_> {
    fn drop(&mut self) {
        if let Some(raw) = self.raw.take() {
            // SAFETY: fallback for an early error; no further use of the handle.
            unsafe { ffi::sqlite3_backup_finish(raw.as_ptr()); }
        }
    }
}

fn verify_page_counts(source: i64, destination: i64) -> MixResult<()> {
    if source != destination {
        return Err(MixError::structured("SQL_INTEGRITY", format!("sqlbackup: source_pages={source}, destination_pages={destination}")));
    }
    Ok(())
}

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
    backup_with_destination(args, |_| Ok(()))
}

fn backup_with_destination(args: Vec<Value>, configure: impl FnOnce(&Connection) -> MixResult<()>) -> MixResult<Option<Value>> {
    expect_args_between("sqlbackup", &args, 2, 3)?;
    let path_arg = |i: usize| match &args[i] {
        Value::String(s) if !s.is_empty() && s != ":memory:" && !s.contains('\0') => Ok(s.as_str()),
        Value::String(_) => Err(opt_invalid(
            "sqlbackup()",
            "paths must be non-empty filesystem strings",
        )),
        other => Err(MixError::structured("TYPE_MISMATCH", format!("sqlbackup: path must be a string, got {}", other.type_name()))),
    };
    let src = Path::new(path_arg(0)?);
    let dst = Path::new(path_arg(1)?);
    let mut overwrite = false;
    let mut mode = 0o600;
    let mut retry_ms = 5000;
    if let Some(opts) = args.get(2) {
        let Value::Map(opts) = opts else {
            return Err(MixError::structured("TYPE_MISMATCH", "sqlbackup: opts must be a map"));
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
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if m.dev() == metadata.dev() && m.ino() == metadata.ino() {
                    return Err(opt_invalid("sqlbackup()", "source and destination are the same inode"));
                }
            }
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
    let source_pages: i64 = source.query_row("PRAGMA page_count", [], |r| r.get(0)).map_err(sql_error)?;
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
    configure(&destination)?;
    destination
        .busy_timeout(Duration::ZERO)
        .map_err(sql_error)?;
    {
        let backup = CheckedBackup::new(&source, &mut destination).map_err(sql_error)?;
        loop {
            match backup.step(128).map_err(sql_error)? {
                StepResult::Done => {
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
        backup.finish().map_err(sql_error)?;
    }
    let pages: i64 = destination.query_row("PRAGMA page_count", [], |r| r.get(0)).map_err(sql_error)?;
    verify_page_counts(source_pages, pages)?;
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
            ("source_pages".into(), Value::Number(source_pages as f64)),
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

#[cfg(test)]
#[path = "../tests/support/temp_dir.rs"]
mod test_dir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_full_never_publishes_and_preserves_existing_copy() {
        let dir = test_dir::TestDir::new("backup-full");
        let src = dir.path("source.db");
        let dst = dir.path("copy.db");
        Connection::open(&src).unwrap().execute_batch("CREATE TABLE t(x); INSERT INTO t VALUES(zeroblob(100000))").unwrap();
        for overwrite in [false, true] {
            if overwrite { std::fs::write(&dst, "keep").unwrap(); }
            let err = backup_with_destination(vec![
                Value::String(src.to_str().unwrap().into()),
                Value::String(dst.to_str().unwrap().into()),
                Value::map([("overwrite".into(), Value::Bool(overwrite))].into_iter().collect()),
            ], |destination| destination.execute_batch("PRAGMA max_page_count=1").map_err(sql_error)).unwrap_err();
            assert_eq!(err.info().unwrap().code, "SQL_IO");
            if overwrite { assert_eq!(std::fs::read(&dst).unwrap(), b"keep"); }
            else { assert!(!dst.exists()); }
            assert!(std::fs::read_dir(dir.path("")).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().starts_with(".mix-sqlbackup-")));
        }
    }

    #[test]
    fn explicit_finish_returns_native_destination_error() {
        let source = Connection::open_in_memory().unwrap();
        source.execute_batch("CREATE TABLE t(x); INSERT INTO t VALUES(zeroblob(100000))").unwrap();
        let mut destination = Connection::open_in_memory().unwrap();
        destination.execute_batch("PRAGMA max_page_count=1").unwrap();
        let backup = CheckedBackup::new(&source, &mut destination).unwrap();
        assert_eq!(backup.step(-1).unwrap_err().sqlite_error_code(), Some(ErrorCode::DiskFull));
        assert_eq!(backup.finish().unwrap_err().sqlite_error_code(), Some(ErrorCode::DiskFull));
    }

    #[test]
    fn mismatched_page_counts_report_both_and_refuse_empty_or_partial_copy() {
        for destination in [0, 1] {
            let err = verify_page_counts(20, destination).unwrap_err();
            assert_eq!(err.info().unwrap().code, "SQL_INTEGRITY");
            assert!(err.to_string().contains(&format!("source_pages=20, destination_pages={destination}")));
        }
        verify_page_counts(20, 20).unwrap();
    }

    #[test]
    fn direct_dispatch_top_level_types_match_registry_errors() {
        for args in [
            vec![Value::Nil, Value::String("dst".into())],
            vec![Value::String("src".into()), Value::Bool(false)],
            vec![Value::String("src".into()), Value::String("dst".into()), Value::Nil],
        ] {
            assert_eq!(builtin_sqlbackup(args).unwrap_err().info().unwrap().code, "TYPE_MISMATCH");
        }
        assert_eq!(builtin_sqlbackup(vec![Value::String("".into()), Value::String("dst".into())]).unwrap_err().info().unwrap().code, "OPTION_INVALID");
    }
}
