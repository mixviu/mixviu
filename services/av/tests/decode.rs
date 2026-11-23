use std::process::Command;
#[test]
fn probe_decode_and_error_conventions() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.webm");
    std::fs::write(
        &input,
        include_bytes!("../../../libraries/media/webm/tests/fixtures/rav1e.webm"),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("probe")
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout.iter().filter(|b| **b == b'\n').count(), 1);
    let j: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(j["frames"], 10);
    assert_eq!(j["tracks"][0]["codec"], "V_AV1");
    assert_eq!(j["tracks"][0]["width"], 64);
    let y4m = dir.path().join("out.y4m");
    let out = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("decode")
        .arg(&input)
        .arg("--y4m")
        .arg(&y4m)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let pixels = std::fs::read(y4m).unwrap();
    assert!(pixels.starts_with(b"YUV4MPEG2 W64 H48"));
    assert_eq!(pixels.windows(5).filter(|w| *w == b"FRAME").count(), 10);
    for args in [
        vec!["decode"],
        vec!["probe"],
        vec!["decode", "missing.webm", "--unknown"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_av"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stderr.starts_with(b"av: "));
    }
}
