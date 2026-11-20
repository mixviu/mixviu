use media_vfl::{
    Frame, Pixels, Writer,
    reader::{Reader, verify},
};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
struct Take(PathBuf);
impl Take {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
                "vfl-reader-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )))
    }
}
impl Drop for Take {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn json_edit(p: &Path, f: impl FnOnce(&mut Value)) {
    let mut v = serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap();
    f(&mut v);
    std::fs::write(p, v.to_string()).unwrap();
}
fn take(complete: bool) -> (Take, Vec<Vec<u8>>) {
    let t = Take::new();
    let mut w = Writer::new(t.0.clone(), 4, 4, 1).unwrap();
    w.origin(123).unwrap();
    let a = vec![1; 64];
    let mut b = a.clone();
    b[20..28].fill(3);
    b[36..44].fill(9);
    let c = vec![42; 64];
    let frames = vec![
        a.clone(),
        a.clone(),
        b.clone(),
        c.clone(),
        c.clone(),
        c.clone(),
        c.clone(),
        c.clone(),
        c.clone(),
        c.clone(),
        c.clone(),
        a.clone(),
    ];
    for (slot, p) in [(0, a.clone()), (2, b), (3, c), (11, a)] {
        w.frame(Frame {
            slot,
            pixels: Pixels::Bgra(p.into()),
        })
        .unwrap();
    }
    w.frame(Frame {
        slot: 12,
        pixels: Pixels::Marker(json!({"seq":0,"label":"demo","data":{"x":1},"t_us":100,"frame":0})),
    })
    .unwrap();
    w.finish(12, 2, 123, None, complete).unwrap();
    (t, frames)
}
#[test]
fn roundtrip_all_kinds_dropped_interrupted_and_seek() {
    for complete in [true, false] {
        let (t, expected) = take(complete);
        let mut r = Reader::open(&t.0).unwrap();
        assert_eq!(r.complete(), complete);
        for (i, p) in expected.iter().enumerate() {
            let f = r.read().unwrap().unwrap();
            assert_eq!(f.slot, i as u64);
            assert_eq!(f.t_us, i as u64 * 1_000_000);
            assert_eq!(f.pixels, p);
        }
        assert!(r.read().unwrap().is_none());
        for i in [11, 0, 10, 3, 2, 1] {
            assert_eq!(r.seek(i).unwrap().pixels, expected[i as usize]);
        }
        let v = verify(&t.0);
        assert_eq!(v["ok"], true, "{v}");
        assert_eq!(v["frames"], 12);
        assert_eq!(v["markers"], 1);
        assert_eq!(v["complete"], complete);
        assert_eq!(v["dropped"], if complete { json!(2) } else { Value::Null });
    }
}
#[test]
fn corruptions_are_rejected_without_panics() {
    for case in 0..10 {
        let (t, _) = take(true);
        let chunk = t.0.join("chunk-000000.bin");
        match case {
            0 => {
                let mut b = std::fs::read(&chunk).unwrap();
                b[18] ^= 1;
                std::fs::write(&chunk, b).unwrap();
            }
            1 => {
                let mut b = std::fs::read(&chunk).unwrap();
                b.pop();
                std::fs::write(&chunk, b).unwrap();
            }
            2 => json_edit(&t.0.join("manifest.json"), |v| v["version"] = json!(1)),
            3 => {
                std::fs::rename(&chunk, t.0.join("chunk-000001.bin")).unwrap();
            }
            4 => json_edit(&t.0.join("complete.json"), |v| v["frames"] = json!(99)),
            5 => {
                let mut b = std::fs::read(&chunk).unwrap();
                b[..4].copy_from_slice(&u32::MAX.to_le_bytes());
                std::fs::write(&chunk, b).unwrap();
            }
            6 => json_edit(&t.0.join("chunk-000000.json"), |v| v["crc32"] = json!(0)),
            7 => {
                std::fs::write(t.0.join("frames.jsonl"), "{}").unwrap();
            }
            8 => {
                std::fs::write(t.0.join("markers.jsonl"), "{\"seq\":1}\n").unwrap();
            }
            _ => json_edit(&t.0.join("manifest.json"), |v| v["width"] = json!(u64::MAX)),
        }
        assert_eq!(verify(&t.0)["ok"], false, "case {case}");
    }
}
#[test]
fn v1_without_rectangles_and_missing_recovery_sidecars() {
    let t = Take::new();
    let mut w = Writer::new(t.0.clone(), 2, 2, 30).unwrap();
    w.origin(1).unwrap();
    w.frame(Frame {
        slot: 0,
        pixels: Pixels::Bgra(Arc::from([4; 16])),
    })
    .unwrap();
    w.finish(2, 0, 1, None, true).unwrap();
    for p in ["manifest.json", "complete.json"] {
        json_edit(&t.0.join(p), |v| v["version"] = json!(1));
    }
    assert_eq!(verify(&t.0)["ok"], true);
    std::fs::remove_file(t.0.join("complete.json")).unwrap();
    for p in [
        "origin.json",
        "frames.jsonl",
        "markers.jsonl",
        "chunk-000000.json",
    ] {
        std::fs::remove_file(t.0.join(p)).unwrap();
    }
    let mut r = Reader::open(&t.0).unwrap();
    assert!(!r.complete());
    assert_eq!(r.read().unwrap().unwrap().pixels, [4; 16]);
    assert!(r.read().unwrap().is_some());
    assert!(r.read().unwrap().is_none());
}
#[test]
fn chunk_order_receipts_and_seek_across_chunks() {
    let (t, expected) = take(true);
    let p = t.0.join("chunk-000000.bin");
    let b = std::fs::read(&p).unwrap();
    let cut = 17 + u32::from_le_bytes(b[..4].try_into().unwrap()) as usize;
    std::fs::write(&p, &b[..cut]).unwrap();
    std::fs::write(t.0.join("chunk-000001.bin"), &b[cut..]).unwrap();
    for (n, bytes, next) in [(0, &b[..cut], 1), (1, &b[cut..], 12)] {
        std::fs::write(
            t.0.join(format!("chunk-{n:06}.json")),
            json!({"bytes":bytes.len(),"crc32":crc32fast::hash(bytes),"next_frame":next})
                .to_string(),
        )
        .unwrap();
    }
    json_edit(&t.0.join("complete.json"), |v| v["chunks"] = json!(2));
    assert_eq!(verify(&t.0)["ok"], true);
    let mut r = Reader::open(&t.0).unwrap();
    assert_eq!(r.seek(11).unwrap().pixels, expected[11]);
    assert!(r.read().unwrap().is_none());
    for target in (0..expected.len()).rev().chain(0..expected.len()) {
        assert_eq!(r.seek(target as u64).unwrap().pixels, expected[target]);
    }
    let receipt = t.0.join("chunk-000000.json");
    let intact = std::fs::read(&receipt).unwrap();
    std::fs::remove_file(t.0.join("complete.json")).unwrap();
    tear(&receipt);
    assert_eq!(verify(&t.0)["ok"], false);
    assert_eq!(assert_seek_matches(&t.0, &expected), 1);
    std::fs::write(&receipt, intact).unwrap();
    std::fs::rename(t.0.join("chunk-000001.bin"), t.0.join("chunk-000002.bin")).unwrap();
    assert_eq!(verify(&t.0)["ok"], false);
    let mut r = Reader::open(&t.0).unwrap();
    assert_eq!(r.read().unwrap().unwrap().pixels, expected[0]);
    assert!(r.read().is_err());
}
#[test]
fn forged_rectangle_and_torn_recovery_metadata() {
    let (t, _) = take(false);
    std::fs::write(t.0.join("interrupted.json"), "{").unwrap();
    std::fs::write(t.0.join("origin.json"), "{}").unwrap();
    assert!(Reader::open(&t.0).is_ok());
    let p = t.0.join("chunk-000000.bin");
    let mut b = std::fs::read(&p).unwrap();
    let mut at = 0;
    while b[at + 16] != media_vfl::RECT_DELTA {
        at += 17 + u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize;
    }
    let end = at + 17 + u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize;
    b[at + 25..at + 29].copy_from_slice(&u32::MAX.to_le_bytes());
    let crc = crc32fast::hash(&b[at + 8..end]);
    b[at + 4..at + 8].copy_from_slice(&crc.to_le_bytes());
    std::fs::write(p, b).unwrap();
    let mut r = Reader::open(&t.0).unwrap();
    r.read().unwrap();
    r.read().unwrap();
    assert_eq!(
        r.read().err().unwrap().kind,
        media_vfl::reader::ErrorKind::Limit
    );
}
fn decoded(r: &mut Reader) -> (Vec<Vec<u8>>, Option<media_vfl::reader::ErrorKind>) {
    let mut frames = Vec::new();
    loop {
        match r.read() {
            Ok(Some(f)) => frames.push(f.pixels.to_vec()),
            Ok(None) => return (frames, None),
            Err(e) => return (frames, Some(e.kind)),
        }
    }
}
fn assert_seek_matches(path: &Path, expected: &[Vec<u8>]) -> usize {
    let mut r = Reader::open(path).unwrap();
    let (frames, failure) = decoded(&mut r);
    assert_eq!(frames, &expected[..frames.len()]);
    for target in 0..frames.len() {
        let mut r = Reader::open(path).unwrap();
        assert_eq!(r.seek(target as u64).unwrap().pixels, frames[target]);
        let (tail, error) = decoded(&mut r);
        assert_eq!(tail, frames[target + 1..]);
        assert_eq!(error, failure, "target {target}");
    }
    if failure.is_some() {
        assert!(
            Reader::open(path)
                .unwrap()
                .seek(frames.len() as u64)
                .is_err()
        );
    }
    frames.len()
}
fn tear(path: &Path) {
    let mut b = std::fs::read(path).unwrap();
    b.truncate(b.len() - 3);
    std::fs::write(path, b).unwrap();
}
#[test]
fn interrupted_torn_final_frame_line_preserves_last_frame() {
    let (t, expected) = take(false);
    tear(&t.0.join("frames.jsonl"));
    assert_eq!(assert_seek_matches(&t.0, &expected), 12);
    let v = verify(&t.0);
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["complete"], false);
    assert_eq!(v["frames"], 12);
}
#[test]
fn interrupted_torn_final_marker_line_is_advisory() {
    let (t, expected) = take(false);
    tear(&t.0.join("markers.jsonl"));
    assert_eq!(assert_seek_matches(&t.0, &expected), 12);
    let v = verify(&t.0);
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["complete"], false);
    assert_eq!(v["markers"], 0);
    assert!(
        media_vfl::reader::Markers::open(&t.0)
            .unwrap()
            .read()
            .unwrap()
            .is_none()
    );
}
#[test]
fn interrupted_torn_final_receipt_is_advisory() {
    let (t, expected) = take(false);
    tear(&t.0.join("chunk-000000.json"));
    assert_eq!(assert_seek_matches(&t.0, &expected), 12);
    let v = verify(&t.0);
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["complete"], false);
    assert_eq!(v["dropped"], Value::Null);
}
#[test]
fn advisory_receipts_never_hide_extra_frame_rows() {
    use std::io::Write;
    for receipt in ["chunk-000000.json", "interrupted.json"] {
        for malformed in ["{", "{}"] {
            for tail in [
                None,
                Some("{garbage}\n".to_string()),
                Some("x".repeat(1024 * 1024 + 1)),
                Some("{\"frame\":12,\"t_us\":12000000}\n".to_string()),
            ] {
                let (t, expected) = take(false);
                std::fs::write(t.0.join(receipt), malformed).unwrap();
                if let Some(tail) = &tail {
                    std::fs::OpenOptions::new()
                        .append(true)
                        .open(t.0.join("frames.jsonl"))
                        .unwrap()
                        .write_all(tail.as_bytes())
                        .unwrap();
                }
                let mut r = Reader::open(&t.0).unwrap();
                let (frames, failure) = decoded(&mut r);
                assert_eq!(frames, expected);
                assert_eq!(failure.is_some(), tail.is_some(), "{receipt} {malformed}");
                assert_eq!(assert_seek_matches(&t.0, &expected), 12);
                let v = verify(&t.0);
                assert_eq!(v["ok"], tail.is_none(), "{receipt} {malformed}: {v}");
                assert_eq!(v["complete"], false);
                assert_eq!(v["dropped"], Value::Null);
                if tail.is_some() {
                    assert!(
                        v["errors"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|e| e["file"].as_str().unwrap().contains("frames.jsonl"))
                    );
                }
            }
        }
    }
}
#[test]
fn malformed_drop_counts_are_advisory_only_during_recovery() {
    for complete in [false, true] {
        for dropped in [
            None,
            Some(json!("bad")),
            Some(json!(-1)),
            Some(Value::Null),
            Some(json!(1.5)),
        ] {
            let (t, expected) = take(complete);
            let receipt = if complete {
                "complete.json"
            } else {
                "interrupted.json"
            };
            json_edit(&t.0.join(receipt), |v| {
                if let Some(dropped) = dropped {
                    v["dropped"] = dropped;
                } else {
                    v.as_object_mut().unwrap().remove("dropped");
                }
            });
            let mut r = Reader::open(&t.0).unwrap();
            let (frames, failure) = decoded(&mut r);
            assert_eq!(frames, expected);
            assert_eq!(failure.is_some(), complete);
            assert_eq!(assert_seek_matches(&t.0, &expected), 12);
            let v = verify(&t.0);
            assert_eq!(v["ok"], !complete, "{receipt}: {v}");
            assert_eq!(v["complete"], false);
            assert_eq!(v["dropped"], Value::Null);
        }
    }
}
#[test]
fn interrupted_torn_binary_tail_stops_at_valid_prefix() {
    let (t, expected) = take(false);
    tear(&t.0.join("chunk-000000.bin"));
    std::fs::remove_file(t.0.join("chunk-000000.json")).unwrap();
    let p = t.0.join("frames.jsonl");
    let rows = std::fs::read_to_string(&p).unwrap();
    std::fs::write(
        p,
        rows.lines().take(11).collect::<Vec<_>>().join("\n") + "\n",
    )
    .unwrap();
    assert_eq!(assert_seek_matches(&t.0, &expected), 11);
    let v = verify(&t.0);
    assert_eq!(v["ok"], true, "{v}");
    assert_eq!(v["recoverable"], true);
    assert_eq!(v["complete"], false);
    assert_eq!(v["valid_frames"], 11);
    assert_eq!(v["tail"]["chunk"], 0);
    assert!(v["tail"]["offset"].as_u64().unwrap() > 0);
}
#[test]
fn durable_final_receipt_refuses_torn_binary() {
    for receipt in [None, Some("{"), Some("{}"), Some("numeric")] {
        let (t, _) = take(false);
        tear(&t.0.join("chunk-000000.bin"));
        let p = t.0.join("chunk-000000.json");
        match receipt {
            None => {}
            Some("numeric") => json_edit(&p, |v| v["bytes"] = json!(0)),
            Some(s) => std::fs::write(&p, s).unwrap(),
        }
        let rows = std::fs::read_to_string(t.0.join("frames.jsonl")).unwrap();
        std::fs::write(
            t.0.join("frames.jsonl"),
            rows.lines().take(11).collect::<Vec<_>>().join("\n") + "\n",
        )
        .unwrap();
        let v = verify(&t.0);
        assert_eq!(v["ok"], matches!(receipt, Some("{") | Some("{}")), "{v}");
        if receipt.is_none() || receipt == Some("numeric") {
            assert_eq!(v["errors"][0]["kind"], "crc");
            assert!(v["tail"].is_null());
        }
    }
}
#[test]
fn frame_count_preserves_position_and_does_not_poison_on_scan_failure() {
    let (t, expected) = take(true);
    let mut r = Reader::open(&t.0).unwrap();
    r.read().unwrap();
    assert_eq!(r.frame_count().unwrap(), 12);
    assert_eq!(r.frame_count().unwrap(), 12);
    assert_eq!(r.read().unwrap().unwrap().pixels, expected[1]);
    assert_eq!(r.seek(11).unwrap().pixels, expected[11]);
    let (t, _) = take(false);
    tear(&t.0.join("chunk-000000.bin"));
    let mut r = Reader::open(&t.0).unwrap();
    assert!(r.frame_count().is_err());
    assert_eq!(r.read().unwrap().unwrap().slot, 0);
}
#[test]
fn non_utf8_paths_encode_errors_without_panicking() {
    use std::os::unix::ffi::OsStringExt;
    let path = std::ffi::OsString::from_vec(b"/nonexistent-vfl-\xff".to_vec());
    let v = verify(Path::new(&path));
    assert_eq!(v["ok"], false);
    assert!(
        v["errors"][0]["file"]
            .as_str()
            .unwrap()
            .contains('\u{fffd}')
    );
}
#[test]
fn extra_frame_rows_are_errors_at_clean_and_torn_ends() {
    for torn in [false, true] {
        let (t, _) = take(false);
        std::fs::remove_file(t.0.join("chunk-000000.json")).unwrap();
        if torn {
            tear(&t.0.join("chunk-000000.bin"));
        } else {
            use std::io::Write;
            std::fs::OpenOptions::new()
                .append(true)
                .open(t.0.join("frames.jsonl"))
                .unwrap()
                .write_all(b"{\"frame\":12,\"t_us\":12000000}\n")
                .unwrap();
        }
        let v = verify(&t.0);
        assert_eq!(v["ok"], false, "{v}");
        assert_eq!(v["errors"][0]["kind"], "structure");
    }
}
#[test]
fn key_index_is_capped_but_linear_verification_and_count_are_not() {
    let (t, _) = take(false);
    let p = t.0.join("chunk-000000.bin");
    let b = std::fs::read(&p).unwrap();
    let end = 17 + u32::from_le_bytes(b[..4].try_into().unwrap()) as usize;
    let mut record = b[..end].to_vec();
    let mut bytes = Vec::new();
    for slot in 0..65_537u64 {
        record[8..16].copy_from_slice(&slot.to_le_bytes());
        let crc = crc32fast::hash(&record[8..]);
        record[4..8].copy_from_slice(&crc.to_le_bytes());
        bytes.extend_from_slice(&record);
    }
    std::fs::write(p, bytes).unwrap();
    for name in ["chunk-000000.json", "frames.jsonl", "markers.jsonl"] {
        std::fs::remove_file(t.0.join(name)).unwrap();
    }
    let mut r = Reader::open(&t.0).unwrap();
    assert_eq!(r.frame_count().unwrap(), 65_537);
    assert_eq!(
        r.seek(65_536).err().unwrap().kind,
        media_vfl::reader::ErrorKind::Limit
    );
    assert_eq!(verify(&t.0)["valid_frames"], 65_537);
    assert_eq!(verify(&t.0)["ok"], true);
}
#[test]
fn writer_record_write_boundaries_recover_only_final_eof() {
    // Five writer writes: length, CRC, slot, kind, payload. Include a
    // partial payload as well as the boundary after the complete payload.
    for boundary in [4, 8, 16, 17, 18, 0] {
        let (t, expected) = take(false);
        let p = t.0.join("chunk-000000.bin");
        let mut b = std::fs::read(&p).unwrap();
        let mut at = 0;
        while b[at + 16] != media_vfl::RECT_DELTA {
            at += 17 + u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize;
        }
        let end = at + 17 + u32::from_le_bytes(b[at..at + 4].try_into().unwrap()) as usize;
        b.truncate(if boundary == 0 { end } else { at + boundary });
        std::fs::write(&p, b).unwrap();
        std::fs::remove_file(t.0.join("chunk-000000.json")).unwrap();
        // Model the writer's ordering: binary record precedes sidecar append.
        let rows = std::fs::read_to_string(t.0.join("frames.jsonl")).unwrap();
        let n = if boundary == 0 { 3 } else { 2 };
        std::fs::write(
            t.0.join("frames.jsonl"),
            rows.lines().take(n).collect::<Vec<_>>().join("\n") + "\n",
        )
        .unwrap();
        let v = verify(&t.0);
        assert_eq!(v["ok"], true, "{boundary}: {v}");
        assert_eq!(v["recoverable"], boundary != 0, "{v}");
        assert_eq!(v["valid_frames"], n);
        let mut r = Reader::open(&t.0).unwrap();
        assert_eq!(decoded(&mut r).0, expected[..n]);
        assert_eq!(
            media_vfl::reader::verify_with_options(&t.0, true)["ok"],
            false
        );
    }
}
#[test]
fn completion_read_failures_are_fatal_but_parse_and_false_are_recoverable() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    if let Some(p) = std::env::var_os("VFL_PERMISSION_TEST_TAKE") {
        let p = PathBuf::from(p);
        let e = Reader::open(&p).err().unwrap();
        assert_eq!(e.kind, media_vfl::reader::ErrorKind::Io);
        assert_eq!(e.file, p.join("complete.json"));
        assert!(media_vfl::reader::Markers::open(&p).is_err());
        return;
    }
    for case in 0..6 {
        let (t, _) = take(true);
        let p = t.0.join("complete.json");
        match case {
            0 => std::fs::write(&p, "{").unwrap(),
            1 => json_edit(&p, |v| v["complete"] = json!(false)),
            2 => std::fs::write(&p, vec![b' '; 1024 * 1024 + 1]).unwrap(),
            3 => {
                std::fs::remove_file(&p).unwrap();
                symlink("absent", &p).unwrap();
            }
            4 => {
                std::fs::remove_file(&p).unwrap();
                std::fs::create_dir(&p).unwrap();
            }
            _ => std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o000)).unwrap(),
        }
        // Root bypasses file mode bits; exercise the real open failure in an
        // unprivileged subprocess instead of silently skipping the case.
        // SAFETY: geteuid has no preconditions.
        if case == 5 && unsafe { libc::geteuid() } == 0 {
            use std::os::unix::process::CommandExt;
            std::fs::set_permissions(&t.0, std::fs::Permissions::from_mode(0o755)).unwrap();
            let manifest = t.0.join("manifest.json");
            std::fs::set_permissions(manifest, std::fs::Permissions::from_mode(0o644)).unwrap();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "completion_read_failures_are_fatal_but_parse_and_false_are_recoverable",
                ])
                .env("VFL_PERMISSION_TEST_TAKE", &t.0)
                .uid(65534)
                .gid(65534)
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
            continue;
        }
        assert_eq!(Reader::open(&t.0).is_ok(), case < 2, "case {case}");
        assert_eq!(media_vfl::reader::Markers::open(&t.0).is_ok(), case < 2);
        assert_eq!(
            media_vfl::reader::verify_with_options(&t.0, true)["ok"],
            false
        );
    }
}
#[test]
fn failed_seek_poisons_reads_and_future_seeks() {
    let (t, _) = take(true);
    let mut r = Reader::open(&t.0).unwrap();
    assert!(r.seek(100).is_err());
    assert!(r.read().is_err());
    assert!(r.seek(0).is_err());
}
#[test]
fn binary_tear_does_not_hide_other_damage_or_apply_to_earlier_chunks() {
    for complete in [false, true] {
        let (t, _) = take(complete);
        tear(&t.0.join("chunk-000000.bin"));
        use std::io::Write;
        std::fs::OpenOptions::new()
            .append(true)
            .open(t.0.join("frames.jsonl"))
            .unwrap()
            .write_all(b"{bad}\n")
            .unwrap();
        let v = verify(&t.0);
        assert_eq!(v["ok"], false);
        assert_eq!(v["recoverable"], false);
    }
    let (t, _) = take(false);
    tear(&t.0.join("chunk-000000.bin"));
    std::fs::write(t.0.join("chunk-000001.bin"), []).unwrap();
    let v = verify(&t.0);
    assert_eq!(v["ok"], false);
    assert_eq!(v["recoverable"], false);
}
#[test]
fn marker_errors_always_name_row_start() {
    for tail in [
        r#"{"seq":1,"frame":0,"label":"x","data":null}"#,
        r#"{"seq":5,"t_us":0,"frame":0,"label":"x","data":null}"#,
    ] {
        let (t, _) = take(true);
        let p = t.0.join("markers.jsonl");
        let first = std::fs::read_to_string(&p).unwrap();
        std::fs::write(&p, format!("{first}{tail}\n")).unwrap();
        let mut m = media_vfl::reader::Markers::open(&t.0).unwrap();
        m.read().unwrap();
        assert_eq!(m.read().unwrap_err().offset, first.len() as u64);
    }
}
#[test]
fn complete_torn_sidecars_are_errors() {
    for sidecar in ["frames.jsonl", "markers.jsonl", "chunk-000000.json"] {
        let (t, expected) = take(true);
        tear(&t.0.join(sidecar));
        let n = assert_seek_matches(&t.0, &expected);
        assert_eq!(n, if sidecar == "frames.jsonl" { 11 } else { 12 });
        let v = verify(&t.0);
        assert_eq!(v["ok"], false, "{sidecar}: {v}");
    }
}
#[test]
fn malformed_and_oversized_interior_rows_never_disappear() {
    for complete in [true, false] {
        for replacement in ["{broken}\n".to_string(), "x".repeat(1024 * 1024 + 1)] {
            let (t, expected) = take(complete);
            let p = t.0.join("frames.jsonl");
            let old = std::fs::read_to_string(&p).unwrap();
            let rows: Vec<_> = old.lines().collect();
            std::fs::write(
                &p,
                format!("{}\n{}{}\n", rows[0], replacement, rows[2..].join("\n")),
            )
            .unwrap();
            assert_eq!(assert_seek_matches(&t.0, &expected), 1);
            assert_eq!(verify(&t.0)["ok"], false);
            assert!(Reader::open(&t.0).unwrap().seek(11).is_err());
        }
        let (t, _) = take(complete);
        let p = t.0.join("markers.jsonl");
        let old = std::fs::read_to_string(&p).unwrap();
        std::fs::write(p, format!("{{broken}}\n{old}")).unwrap();
        assert_eq!(verify(&t.0)["ok"], false);
    }
}
#[test]
fn reader_manifest_caps_are_absolute() {
    for (width, height, fps) in [(30000, 30000, 30), (4, 4, 0), (0, 4, 30), (4, 4, 61)] {
        let (t, _) = take(false);
        json_edit(&t.0.join("manifest.json"), |v| {
            v["width"] = json!(width);
            v["height"] = json!(height);
            v["fps"] = json!(fps);
        });
        let e = Reader::open(&t.0).err().unwrap();
        assert_eq!(e.kind, media_vfl::reader::ErrorKind::Limit);
        assert_eq!(e.file, t.0.join("manifest.json"));
    }
}
#[test]
fn recovery_rejects_terminated_and_oversized_final_rows() {
    use std::io::Write;
    for sidecar in ["frames.jsonl", "markers.jsonl"] {
        for tail in ["{broken}\n".to_string(), "x".repeat(1024 * 1024 + 1)] {
            let (t, expected) = take(false);
            std::fs::OpenOptions::new()
                .append(true)
                .open(t.0.join(sidecar))
                .unwrap()
                .write_all(tail.as_bytes())
                .unwrap();
            assert_eq!(verify(&t.0)["ok"], false);
            assert_eq!(assert_seek_matches(&t.0, &expected), 12);
        }
    }
}
#[test]
#[ignore = "60 second generated 4K30 export measurement on CBC"]
fn bench_export() {
    for busy in [false, true] {
        let t = Take::new();
        let mut w = Writer::new(t.0.clone(), 3840, 2160, 30).unwrap();
        w.origin(0).unwrap();
        let mut pixels = vec![0; 3840 * 2160 * 4];
        for n in 0..1800 {
            if busy {
                for (i, p) in pixels.iter_mut().enumerate() {
                    *p = ((i * 37 + i / 31 + n * 13) % 256) as u8;
                }
            } else {
                pixels[n % 1024] = (n % 256) as u8;
            }
            w.frame(Frame {
                slot: n as u64,
                pixels: Pixels::Bgra(pixels.clone().into()),
            })
            .unwrap();
        }
        w.finish(1800, 0, 0, None, true).unwrap();
        drop(w);
        for _ in 0..3 {
            let out = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--ignored", "--exact", "bench_child", "--nocapture"])
                .env("VFL_BENCH_TAKE", &t.0)
                .stdout(std::process::Stdio::null())
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            println!(
                "VFL_BENCH busy={busy}\n{}\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
}
#[test]
#[ignore = "child process of bench_export"]
fn bench_child() {
    let Some(path) = std::env::var_os("VFL_BENCH_TAKE") else {
        return;
    };
    let now = std::time::Instant::now();
    media_vfl::cli::run(["export".into(), path, "--format".into(), "bgra".into()]).unwrap();
    eprintln!(
        "frames=1800 seconds={} fps={}",
        now.elapsed().as_secs_f64(),
        1800.0 / now.elapsed().as_secs_f64()
    );
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
    // SAFETY: getrusage writes the supplied valid storage and retains no pointer.
    assert_eq!(
        unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) },
        0
    );
    let usage = unsafe { usage.assume_init() };
    // VmHWM belongs to this executable's address space. ru_maxrss can retain
    // the pre-exec parent's high-water mark on Linux.
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    let peak = status
        .lines()
        .find_map(|l| l.strip_prefix("VmHWM:"))
        .unwrap();
    eprintln!(
        "peak_rss={} user_s={} system_s={}",
        peak.trim(),
        usage.ru_utime.tv_sec as f64 + usage.ru_utime.tv_usec as f64 / 1_000_000.0,
        usage.ru_stime.tv_sec as f64 + usage.ru_stime.tv_usec as f64 / 1_000_000.0
    );
}
