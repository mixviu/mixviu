use std::process::Command;
#[test]
fn av_errors_are_clean() {
    for args in [
        vec![],
        vec!["audio"],
        vec!["unknown", "x", "--out", "y"],
        vec!["audio", "missing.json", "--out", "x", "--out", "y"],
    ] {
        let o = Command::new(env!("CARGO_BIN_EXE_av"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(2));
        assert!(o.stdout.is_empty());
        let error = String::from_utf8(o.stderr).unwrap();
        assert!(error.starts_with("av: "), "{error}");
        assert!(!error.contains("panicked"), "{error}");
    }
}
