// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg(feature = "sqlite")]
#[path = "support/temp_dir.rs"]
mod temp_dir;
use mix::{
    error::MixError,
    evaluator::{Evaluator, SharedBuf},
    lexer::Lexer,
    parser::Parser,
};
use rusqlite::Connection;
use temp_dir::TestDir;

async fn run(src: &str) -> Result<String, MixError> {
    let tokens = Lexer::new(src).tokenize()?;
    let stmts = Parser::new(tokens, src).parse_program()?;
    let out = SharedBuf::new();
    let mut eval = Evaluator::with_output(Box::new(out.clone()), Box::new(SharedBuf::new()));
    eval.execute(&stmts).await?;
    Ok(out.to_string_lossy())
}

async fn code(src: &str) -> String {
    run(&format!("try\n{src}\ncatch $m, $e\nprint($e.code)\nend"))
        .await
        .unwrap()
        .trim()
        .to_owned()
}

#[tokio::test]
async fn live_wal_writer_committed_snapshot_and_contract() {
    let dir = TestDir::new("backup-wal");
    let src = dir.path("source.db");
    let dst = dir.path("copy.db");
    let writer = Connection::open(&src).unwrap();
    writer.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0; CREATE TABLE t(x); INSERT INTO t VALUES(1),(2); BEGIN IMMEDIATE; INSERT INTO t VALUES(3);").unwrap();
    let result = run(&format!("$r = sqlbackup({:?}, {:?})\nprint($r.ok)\nprint($r.integrity)\nprint($r.pages > 0)\nprint($r.bytes > 0)\nprint($r.duration_ms >= 0)\nprint($r.source_pages == $r.pages)", src, dst)).await.unwrap();
    assert_eq!(result, "true\nok\ntrue\ntrue\ntrue\ntrue\n");
    let copy = Connection::open(&dst).unwrap();
    assert_eq!(
        copy.query_row("SELECT sum(x) FROM t", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        copy.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        copy.query_row("PRAGMA journal_mode", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "delete"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&dst).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    writer.execute_batch("ROLLBACK").unwrap();
}

#[tokio::test]
async fn refuses_existing_and_can_replace_offline_copy_with_mode() {
    let dir = TestDir::new("backup-overwrite");
    let src = dir.path("source.db");
    let dst = dir.path("copy.db");
    Connection::open(&src)
        .unwrap()
        .execute_batch("CREATE TABLE t(x)")
        .unwrap();
    std::fs::write(&dst, "keep").unwrap();
    assert_eq!(
        code(&format!("sqlbackup({src:?}, {dst:?})")).await,
        "SQL_DEST_EXISTS"
    );
    assert_eq!(std::fs::read(&dst).unwrap(), b"keep");
    run(&format!(
        "sqlbackup({src:?}, {dst:?}, {{overwrite: true, mode: 0o640}})"
    ))
    .await
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&dst).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }
    assert_eq!(
        code(&format!("sqlbackup({src:?}, {src:?}, {{overwrite: true}})")).await,
        "OPTION_INVALID"
    );
    std::fs::write(dir.path("copy.db-wal"), "sidecar").unwrap();
    assert_eq!(
        code(&format!("sqlbackup({src:?}, {dst:?}, {{overwrite: true}})")).await,
        "OPTION_INVALID"
    );
}

#[tokio::test]
async fn errors_preserve_destination_and_leave_no_staging_files() {
    let dir = TestDir::new("backup-errors");
    let src = dir.path("source.db");
    let dst = dir.path("copy.db");
    assert_eq!(
        code(&format!("sqlbackup({src:?}, {dst:?})")).await,
        "SQL_NO_DATABASE"
    );
    assert!(!src.exists());
    std::fs::write(&src, "not sqlite").unwrap();
    std::fs::write(&dst, "keep").unwrap();
    assert_eq!(
        code(&format!("sqlbackup({src:?}, {dst:?}, {{overwrite: true}})")).await,
        "SQL_ERROR"
    );
    assert_eq!(std::fs::read(&dst).unwrap(), b"keep");
    assert_eq!(
        code(&format!("sqlbackup({src:?}, {dst:?}, nil)")).await,
        "TYPE_MISMATCH"
    );
    for opts in [
        "{mode: -1}",
        "{mode: 512}",
        "{retry_ms: 60001}",
        "{retry_ms: 0.5}",
        "{overwrite: 1}",
        "{unknown: true}",
    ] {
        assert_eq!(
            code(&format!("sqlbackup({src:?}, {dst:?}, {opts})")).await,
            "OPTION_INVALID"
        );
    }
    std::fs::remove_file(&src).unwrap();
    Connection::open(&src)
        .unwrap()
        .execute_batch("CREATE TABLE t(x)")
        .unwrap();
    assert_eq!(
        code(&format!(
            "sqlbackup({src:?}, {:?})",
            dir.path("absent/copy.db")
        ))
        .await,
        "SQL_IO"
    );
    assert_eq!(std::fs::read_dir(dir.path("")).unwrap().count(), 2);
}

#[tokio::test]
async fn exclusive_writer_exhausts_bounded_busy_budget() {
    let dir = TestDir::new("backup-busy");
    let src = dir.path("source.db");
    let dst = dir.path("copy.db");
    let writer = Connection::open(&src).unwrap();
    writer
        .execute_batch("CREATE TABLE t(x); BEGIN EXCLUSIVE; INSERT INTO t VALUES(1)")
        .unwrap();
    let start = std::time::Instant::now();
    assert_eq!(
        code(&format!("sqlbackup({src:?}, {dst:?}, {{retry_ms: 20}})")).await,
        "SQL_BUSY"
    );
    assert!(start.elapsed() < std::time::Duration::from_secs(2));
    assert!(!dst.exists());
    writer.execute_batch("ROLLBACK").unwrap();
}

#[tokio::test]
async fn retries_until_writer_releases_lock() {
    let dir = TestDir::new("backup-retry");
    let src = dir.path("source.db");
    let dst = dir.path("copy.db");
    let writer = Connection::open(&src).unwrap();
    writer
        .execute_batch("CREATE TABLE t(x); BEGIN EXCLUSIVE; INSERT INTO t VALUES(7)")
        .unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        writer.execute_batch("COMMIT").unwrap();
    });
    run(&format!("sqlbackup({src:?}, {dst:?}, {{retry_ms: 2000}})"))
        .await
        .unwrap();
    release.join().unwrap();
    assert_eq!(
        Connection::open(dst)
            .unwrap()
            .query_row("SELECT x FROM t", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        7
    );
}

#[cfg(unix)]
#[tokio::test]
async fn refuses_symlink_destination_without_touching_target() {
    let dir = TestDir::new("backup-symlink");
    let src = dir.path("source.db");
    let dst = dir.path("copy.db");
    let target = dir.path("target");
    Connection::open(&src)
        .unwrap()
        .execute_batch("CREATE TABLE t(x)")
        .unwrap();
    std::fs::write(&target, "keep").unwrap();
    std::os::unix::fs::symlink(&target, &dst).unwrap();
    assert_eq!(
        code(&format!("sqlbackup({src:?}, {dst:?}, {{overwrite: true}})")).await,
        "OPTION_INVALID"
    );
    assert_eq!(std::fs::read(target).unwrap(), b"keep");
}

#[cfg(unix)]
#[tokio::test]
async fn refuses_source_hardlink_without_replacing_it() {
    use std::os::unix::fs::MetadataExt;
    let dir = TestDir::new("backup-hardlink");
    let src = dir.path("source.db");
    let dst = dir.path("copy.db");
    Connection::open(&src)
        .unwrap()
        .execute_batch("CREATE TABLE t(x)")
        .unwrap();
    std::fs::hard_link(&src, &dst).unwrap();
    let before = std::fs::metadata(&dst).unwrap();
    assert_eq!(
        code(&format!("sqlbackup({src:?}, {dst:?}, {{overwrite: true}})")).await,
        "OPTION_INVALID"
    );
    assert_eq!(std::fs::metadata(&dst).unwrap().ino(), before.ino());
}

#[tokio::test]
async fn registry_rejects_nonstring_paths_before_filesystem_access() {
    assert_eq!(code("sqlbackup(nil, \"unused\")").await, "TYPE_MISMATCH");
    assert_eq!(code("sqlbackup(\"unused\", false)").await, "TYPE_MISMATCH");
}

#[cfg(unix)]
#[tokio::test]
async fn wal_without_creatable_shm_returns_sql_io() {
    use std::os::unix::{fs::PermissionsExt, process::CommandExt};
    if let Some(src) = std::env::var_os("MIX_TEST_BACKUP_READONLY_SOURCE") {
        let dst = std::env::var_os("MIX_TEST_BACKUP_READONLY_DEST").unwrap();
        assert_eq!(
            code(&format!("sqlbackup({src:?}, {dst:?})")).await,
            "SQL_IO"
        );
        assert!(!std::path::Path::new(&dst).exists());
        return;
    }
    let dir = TestDir::new("backup-readonly-wal");
    std::fs::set_permissions(dir.path(""), std::fs::Permissions::from_mode(0o755)).unwrap();
    let writer_path = dir.path("writer.db");
    let writer = Connection::open(&writer_path).unwrap();
    writer.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0; CREATE TABLE t(x); INSERT INTO t VALUES(1)").unwrap();
    let source_dir = dir.path("readonly");
    let output_dir = dir.path("output");
    std::fs::create_dir(&source_dir).unwrap();
    std::fs::create_dir(&output_dir).unwrap();
    std::fs::set_permissions(&output_dir, std::fs::Permissions::from_mode(0o777)).unwrap();
    let src = source_dir.join("source.db");
    std::fs::copy(&writer_path, &src).unwrap();
    std::fs::copy(dir.path("writer.db-wal"), source_dir.join("source.db-wal")).unwrap();
    for file in [&src, &source_dir.join("source.db-wal")] {
        std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o444)).unwrap();
    }
    std::fs::set_permissions(&source_dir, std::fs::Permissions::from_mode(0o555)).unwrap();
    let dst = output_dir.join("copy.db");
    let mut child = std::process::Command::new(std::env::current_exe().unwrap());
    child
        .args([
            "--exact",
            "wal_without_creatable_shm_returns_sql_io",
            "--nocapture",
        ])
        .env("MIX_TEST_BACKUP_READONLY_SOURCE", &src)
        .env("MIX_TEST_BACKUP_READONLY_DEST", &dst);
    // CBC runs as root; drop only the child so permissions are actually tested.
    if unsafe { libc::geteuid() } == 0 {
        child.gid(65534).uid(65534);
    }
    let result = child.output().unwrap();
    std::fs::set_permissions(&source_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("1 passed"));
    assert!(!source_dir.join("source.db-shm").exists());
    assert!(!dst.exists());
}
