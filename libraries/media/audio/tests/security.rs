use media_audio::{Audio, WavFormat, timeline::Timeline};
use serde_json::json;

// Standard PCM16 header with sparse data: malformed rates need no decoding.
fn sparse_wav(path: &std::path::Path, rate: u32, channels: u16, frames: u32) {
    use std::io::Write;
    let data = frames * u32::from(channels) * 2;
    let mut header = Vec::new();
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&(36 + data).to_le_bytes());
    header.extend_from_slice(b"WAVEfmt ");
    header.extend_from_slice(&16u32.to_le_bytes());
    header.extend_from_slice(&1u16.to_le_bytes());
    header.extend_from_slice(&channels.to_le_bytes());
    header.extend_from_slice(&rate.to_le_bytes());
    header.extend_from_slice(&(rate * u32::from(channels) * 2).to_le_bytes());
    header.extend_from_slice(&(channels * 2).to_le_bytes());
    header.extend_from_slice(&16u16.to_le_bytes());
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data.to_le_bytes());
    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(&header).unwrap();
    file.set_len(44 + u64::from(data)).unwrap();
}

#[test]
fn sample_rates_and_native_decode_budget() {
    let dir = std::env::temp_dir().join(format!("audio-native-budget-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let mut v = json!({"version":1,"sample_rate":48000,"channels":2,"duration_s":4.0/48000.0,"sources":[{"path":"source.wav","at_s":0}],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}});
    let mix = |v: serde_json::Value| serde_json::from_value::<Timeline>(v).unwrap().mix(&dir);
    for rate in [7999, 384001] {
        sparse_wav(&dir.join("source.wav"), rate, 1, 1);
        let error = mix(v.clone()).unwrap_err().to_string();
        assert!(
            error.contains("sources[0]") && error.contains("sample_rate"),
            "{error}"
        );
    }
    for rate in [8000, 384000] {
        sparse_wav(&dir.join("source.wav"), rate, 1, 1);
        assert!(mix(v.clone()).is_ok(), "inclusive rate {rate}");
    }
    // Four output frames have a 1024-input-sample budget. At 384 kHz,
    // each mono window reads 4*8+512=544 samples including sinc support.
    sparse_wav(&dir.join("source.wav"), 384000, 1, 1024);
    let source = json!({"path":"source.wav","at_s":0,"to_s":4.0/48000.0});
    v["sources"] = json!([source.clone()]);
    assert!(mix(v.clone()).is_ok());
    v["sources"] = json!([source.clone(), source.clone()]);
    let error = mix(v.clone()).unwrap_err().to_string();
    assert!(
        error.contains("sources[1].work: decoded input samples"),
        "{error}"
    );
    // A shorter stereo file clips each native window to 256*2 samples:
    // two windows exactly reach the budget; one more is refused.
    sparse_wav(&dir.join("source.wav"), 384000, 2, 256);
    assert!(mix(v.clone()).is_ok());
    v["sources"] = json!([source.clone(), source.clone(), source]);
    let error = mix(v).unwrap_err().to_string();
    assert!(
        error.contains("sources[2].work: decoded input samples"),
        "{error}"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

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
    // Unsupported sample rates are refused before sample decoding.
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
            .contains("sample_rate")
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
