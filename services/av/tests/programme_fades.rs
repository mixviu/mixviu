use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

fn run(mut command: Command) -> Vec<u8> {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn raw_y4m(path: &Path) -> Vec<u8> {
    let bytes = fs::read(path).unwrap();
    let mut offset = bytes.iter().position(|b| *b == b'\n').unwrap() + 1;
    let mut out = Vec::new();
    while offset < bytes.len() {
        assert!(bytes[offset..].starts_with(b"FRAME"));
        offset += bytes[offset..].iter().position(|b| *b == b'\n').unwrap() + 1;
        out.extend_from_slice(&bytes[offset..offset + 1536]);
        offset += 1536;
    }
    assert_eq!(out.len(), 120 * 1536);
    out
}

#[test]
fn programme_envelopes_cross_still_and_vfl_cuts_against_ffmpeg() {
    if Command::new("ffmpeg").arg("-version").output().is_err()
        && std::env::var_os("MIX_TEST_REQUIRE_FFMPEG").is_none()
    {
        eprintln!("ffmpeg unavailable");
        return;
    }
    let root = tempfile::tempdir().unwrap();
    for (name, rgb) in [("a.png", [180, 80, 30]), ("b.png", [30, 150, 90])] {
        let mut e = png::Encoder::new(fs::File::create(root.path().join(name)).unwrap(), 32, 32);
        e.set_color(png::ColorType::Rgb);
        e.write_header()
            .unwrap()
            .write_image_data(&rgb.repeat(1024))
            .unwrap();
    }
    let mut w = media_vfl::Writer::new(root.path().join("take.vfl"), 32, 32, 30).unwrap();
    w.origin(0).unwrap();
    for (slot, bgra) in [(0, [200, 40, 80, 255]), (12, [40, 180, 210, 255])] {
        w.frame(media_vfl::Frame {
            slot,
            pixels: media_vfl::Pixels::Bgra(bgra.repeat(1024).into()),
        })
        .unwrap();
    }
    w.finish(120, 0, 0, None, true).unwrap();
    media_audio::Audio::new(
        48000,
        1,
        (0..192000)
            .map(|n| 0.1 * (n as f64 * 440.0 * std::f64::consts::TAU / 48000.0).sin())
            .collect(),
    )
    .unwrap()
    .write_wav(
        fs::File::create(root.path().join("tone.wav")).unwrap(),
        media_audio::WavFormat::Float32,
    )
    .unwrap();
    let mut t = json!({"version":1,"duration_s":4,
        "video":{"width":32,"height":32,"fps":30,"segments":[
            {"still":"a.png","at_s":0,"duration_s":0.5},
            {"still":"b.png","at_s":0.5,"duration_s":0.03333333333333333},
            {"path":"take.vfl","from_slot":10,"to_slot":80,"at_s":0.5333333333333333},
            {"still":"a.png","at_s":3.1,"duration_s":0.4},
            {"still":"b.png","at_s":3.5,"duration_s":0.3},
            {"path":"take.vfl","from_slot":10,"to_slot":16,"at_s":3.8}]},
        "audio":{"sample_rate":48000,"channels":2,"sources":[{"path":"tone.wav","at_s":0.1,"from_s":0.2,"to_s":3.9}],
            "master":{"integrated_lufs":-14,"true_peak_dbtp":-1}},
        "encode":{"speed":10,"qp":1,"tile_cols":1,"tile_rows":1,"threads":1,"chunks":2,"keyint_s":1}});
    let path = root.path().join("timeline.json");
    let render = |t: &Value, name: &str| {
        fs::write(&path, serde_json::to_vec(t).unwrap()).unwrap();
        let mut c = Command::new(env!("CARGO_BIN_EXE_av"));
        c.arg("render")
            .arg(&path)
            .arg("--out")
            .arg(root.path().join(name));
        serde_json::from_slice::<Value>(&run(c)).unwrap()
    };
    render(&t, "plain.webm");
    av::render::reference(
        &path,
        fs::File::create(root.path().join("plain.y4m")).unwrap(),
    )
    .unwrap();
    let plain = raw_y4m(&root.path().join("plain.y4m"));
    t["video"]["fade_in_s"] = json!(0);
    t["video"]["fade_out_s"] = json!(0);
    render(&t, "zero.webm");
    assert_eq!(
        fs::read(root.path().join("plain.webm")).unwrap(),
        fs::read(root.path().join("zero.webm")).unwrap()
    );
    // The nonzero VFL trim is honoured: source slot 12 arrives two frames after the cut.
    assert_eq!(&plain[16 * 1536..17 * 1536], &plain[17 * 1536..18 * 1536]);
    assert_ne!(&plain[17 * 1536..18 * 1536], &plain[18 * 1536..19 * 1536]);
    for (name, incoming, outgoing, filter) in [
        ("in", 0.6, 0.0, "fade=t=in:s=0:n=17"),
        ("out", 0.0, 1.0, "fade=t=out:s=90:n=29"),
        ("both", 0.6, 1.0, "fade=t=in:s=0:n=17,fade=t=out:s=90:n=29"),
    ] {
        t["video"]["fade_in_s"] = json!(incoming);
        t["video"]["fade_out_s"] = json!(outgoing);
        let webm = format!("{name}.webm");
        let receipt = render(&t, &webm);
        assert_eq!(receipt["frames_out"], 120);
        assert!(
            receipt["frames_encoded"].as_u64().unwrap()
                >= u64::from(incoming > 0.0) * 18 + u64::from(outgoing > 0.0) * 30
        );
        let decoded_path = root.path().join(format!("{name}.y4m"));
        let mut c = Command::new(env!("CARGO_BIN_EXE_av"));
        c.arg("decode")
            .arg(root.path().join(&webm))
            .arg("--y4m")
            .arg(&decoded_path);
        run(c);
        let decoded = raw_y4m(&decoded_path);
        let mut c = Command::new("ffmpeg");
        c.args(["-v", "error", "-i"])
            .arg(root.path().join("plain.y4m"))
            .args(["-vf", filter, "-f", "rawvideo", "-pix_fmt", "yuv420p", "-"]);
        let oracle = run(c);
        assert_eq!(decoded.len(), oracle.len());
        let mut maxima = [0u8; 3];
        for (i, (a, b)) in decoded.iter().zip(&oracle).enumerate() {
            let plane = match i % 1536 {
                0..1024 => 0,
                1024..1280 => 1,
                _ => 2,
            };
            maxima[plane] = maxima[plane].max(a.abs_diff(*b));
        }
        println!("{name}: decoded ffmpeg fade per-plane max error {maxima:?}");
        assert!(maxima.iter().all(|v| *v <= 3), "{name}: {maxima:?}");
        // Programme picture changes cannot alter the mastered Opus stream or timestamps.
        let audio = |name: &str| {
            let bytes = fs::read(root.path().join(name)).unwrap();
            media_webm::Demuxer::new(&bytes)
                .unwrap()
                .packets()
                .iter()
                .filter(|p| p.track == 2)
                .map(|p| (p.timestamp, p.data.to_vec()))
                .collect::<Vec<_>>()
        };
        assert_eq!(audio(&webm), audio("plain.webm"));
    }
}
