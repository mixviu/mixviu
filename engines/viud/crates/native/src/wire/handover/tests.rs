// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
use super::*;

fn fixture() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("handover-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    TRAIL.with_borrow_mut(|t| *t = Some(Trail::new(dir.join("handover-wayland-example.jsonl"), "wayland-example".into())));
    dir
}
fn path(dir: &std::path::Path) -> PathBuf { dir.join("handover-wayland-example.jsonl") }
fn rows(dir: &std::path::Path) -> Vec<Value> {
    std::fs::read_to_string(path(dir)).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect()
}
#[test]
fn encoding_and_activate_start_is_synced_before_commit() {
    let dir = fixture();
    let result = run(vec![row("ReclaimModeset", Some(12), Some(34), Some("1920x1080@60".into()))], || {
        let r = rows(&dir);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0]["phase"], "start");
        // run returns from emit only after write_all + sync_data succeed.
        TRAIL.with_borrow(|t| assert!(!t.as_ref().unwrap().disabled));
        Err::<(), _>("example error\nquoted: \"".into())
    });
    assert!(result.is_err());
    let r = rows(&dir);
    assert_eq!(r[1]["ok"], false);
    assert_eq!(r[1]["error"], "example error\nquoted: \"");
    assert_eq!(r[1]["connector"], 12);
    assert_eq!(r[1]["crtc"], 34);
    assert_eq!(r[1]["socket"], "wayland-example");
    assert!(r[1]["elapsed_ms"].is_number());
    let ts = r[1]["ts"].as_str().unwrap();
    assert_eq!(ts.len(), 24);
    assert!(ts.ends_with('Z'));
    assert!(r[1]["boot_ns"].as_u64().unwrap() > 0);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn pause_defers_every_row_until_display_and_input_finish() {
    let dir = fixture();
    pause(|| {
        run(vec![row("CrtcClear", None, Some(34), None)], || Ok(())).unwrap();
        point("DisplayPaused");
        point("InputSuspended");
        assert!(!path(&dir).exists());
    });
    let r = rows(&dir);
    assert_eq!(r.len(), 5);
    assert_eq!(r[0]["event"], "PauseSession");
    assert_eq!(r[4]["event"], "InputSuspended");
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn rotation_keeps_one_previous_file() {
    let dir = fixture();
    let mut r = row("Example",None,None,None);
    r["data"] = json!("x".repeat(400_000));
    for _ in 0..5 { emit(r.clone()); }
    assert_eq!(rows(&dir).len(), 1);
    let previous = path(&dir).with_extension("jsonl.1");
    assert_eq!(std::fs::read_to_string(previous).unwrap().lines().count(), 2);
    assert!(std::fs::metadata(path(&dir)).unwrap().len() <= LIMIT);
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn unwritable_directory_warns_once_and_never_blocks_session() {
    let dir = fixture();
    // A regular file as parent deterministically refuses writes, even as root.
    let blocker = dir.join("not-a-directory");
    std::fs::write(&blocker, "fixture").unwrap();
    TRAIL.with_borrow_mut(|t| t.as_mut().unwrap().path = blocker.join("trail.jsonl"));
    let mut paused = false;
    pause(|| { point("DisplayPaused"); paused = true; });
    assert!(paused);
    let mut activated = false;
    run(vec![row("ReclaimModeset",None,None,None)], || { activated = true; Ok(()) }).unwrap();
    assert!(activated);
    TRAIL.with_borrow(|t| assert_eq!(t.as_ref().unwrap().warnings, 1));
    std::fs::remove_dir_all(dir).unwrap();
}
