use media_audio::{Audio, WavFormat, timeline::Timeline};
use serde_json::json;

#[test]
fn sources_are_confined_regular_bounded_and_trimmed() {
    use std::os::unix::fs::symlink;
    let dir = std::env::temp_dir().join(format!("audio-security-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let base = dir.join("base");
    std::fs::create_dir(&base).unwrap();
    let mut bytes = std::io::Cursor::new(Vec::new());
    Audio::new(48000, 2, vec![0.1; 4800])
        .unwrap()
        .write_wav(&mut bytes, WavFormat::Float32)
        .unwrap();
    std::fs::write(dir.join("outside.wav"), bytes.get_ref()).unwrap();
    std::fs::write(base.join("inside.wav"), bytes.get_ref()).unwrap();
    symlink("../outside.wav", base.join("escape.wav")).unwrap();
    symlink("inside.wav", base.join("allowed.wav")).unwrap();
    symlink("/dev/zero", base.join("device.wav")).unwrap();
    let mut v = json!({"version":1,"sample_rate":48000,"channels":2,"duration_s":0.01,"sources":[{"path":"allowed.wav","at_s":0}],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}});
    let mix = |v: serde_json::Value| serde_json::from_value::<Timeline>(v).unwrap().mix(&base);
    assert!(mix(v.clone()).is_ok());
    for (path, message) in [
        ("escape.wav", "escapes timeline directory"),
        ("device.wav", "not a regular file"),
        (".", "not a regular file"),
    ] {
        v["sources"][0]["path"] = json!(path);
        let e = mix(v.clone()).unwrap_err().to_string();
        assert!(e.contains("sources[0]") && e.contains(message), "{e}");
    }
    // A FIFO with no writer must be rejected immediately, not block in open.
    let fifo =
        std::ffi::CString::new(base.join("fifo.wav").as_os_str().as_encoded_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    v["sources"][0]["path"] = json!("fifo.wav");
    assert!(
        mix(v.clone())
            .unwrap_err()
            .to_string()
            .contains("not a regular file")
    );
    let large = std::fs::File::create(base.join("large.wav")).unwrap();
    large.set_len(10800 * 48000 * 8 * 4 + 65536 + 1).unwrap();
    v["sources"][0]["path"] = json!("large.wav");
    assert!(
        mix(v.clone())
            .unwrap_err()
            .to_string()
            .contains("byte input limit")
    );
    // A valid header with an excessive rate-amplified duration is rejected
    // without allocating its declared decoded sample buffer.
    v["sources"][0]["path"] = json!("long.wav");
    let mut long = std::io::Cursor::new(Vec::new());
    Audio::new(1, 1, vec![0.1; 10801])
        .unwrap()
        .write_wav(&mut long, WavFormat::Float32)
        .unwrap();
    std::fs::write(base.join("long.wav"), long.into_inner()).unwrap();
    assert!(
        mix(v.clone())
            .unwrap_err()
            .to_string()
            .contains("frame limit")
    );
    // The invalid float after the trim must never be decoded or validated.
    let mut trimmed = bytes.into_inner();
    let end = trimmed.len();
    trimmed[end - 4..].copy_from_slice(&f32::NAN.to_le_bytes());
    std::fs::write(base.join("trim.wav"), trimmed).unwrap();
    v["sources"] = json!([{"path":"trim.wav","at_s":0,"from_s":0.01,"to_s":0.02}]);
    assert_eq!(mix(v.clone()).unwrap().frames(), 480);
    v["sources"] = json!(vec![json!({"path":"inside.wav","at_s":0}); 16]);
    assert!(mix(v.clone()).is_ok());
    v["sources"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"inside.wav","at_s":0}));
    assert!(
        mix(v)
            .unwrap_err()
            .to_string()
            .contains("16x output frames")
    );
    std::fs::remove_dir_all(dir).unwrap();
}
