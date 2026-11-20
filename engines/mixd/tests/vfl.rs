use media_vfl::{Frame, Pixels, Writer};
use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Command, Stdio},
};

// More output than a pipe can hold, so closing the reader necessarily reaches
// the writer while it still has output left (without sleeps or timing races).
fn pipe_take(name: &str, side: u32) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("mixd-vfl-{name}-{}", std::process::id()));
    let mut w = Writer::new(p.clone(), side, side, 30).unwrap();
    w.origin(0).unwrap();
    w.frame(Frame {
        slot: 0,
        pixels: Pixels::Bgra(vec![0; (side * side * 4) as usize].into()),
    })
    .unwrap();
    w.finish(128, 0, 0, None, true).unwrap();
    // Keep marker generation independent of the recorder's marker policy.
    let mut markers = std::fs::File::create(p.join("markers.jsonl")).unwrap();
    for seq in 0..16384 {
        writeln!(
            markers,
            "{{\"seq\":{seq},\"t_us\":0,\"frame\":0,\"label\":\"test\",\"data\":null}}"
        )
        .unwrap();
    }
    p
}

#[test]
fn streaming_stdout_closed_early_is_quiet() {
    let p = pipe_take("closed-pipe", 128);
    for args in [
        vec!["export", "--format", "bgra"],
        vec!["export", "--format", "y4m"],
        vec!["markers"],
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_mixd"))
            .args(["vfl", args[0]])
            .arg(&p)
            .args(&args[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut reader = BufReader::new(child.stdout.take().unwrap());
        if args[0] == "markers" {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(line.contains("test"));
        } else {
            let size = if args.last() == Some(&"y4m") {
                let mut header = String::new();
                reader.read_line(&mut header).unwrap();
                assert!(header.starts_with("YUV4MPEG2"));
                reader.read_line(&mut String::new()).unwrap();
                128 * 128 * 3 / 2
            } else {
                128 * 128 * 4
            };
            reader.read_exact(&mut vec![0; size]).unwrap();
        }
        drop(reader);
        let o = child.wait_with_output().unwrap();
        assert_eq!(o.status.code(), Some(0), "{args:?}: {o:?}");
        assert!(o.stderr.is_empty(), "{args:?}: {o:?}");
    }
    std::fs::remove_dir_all(p).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn streaming_stdout_other_errors_still_fail() {
    // A small frame also exercises errors from the final buffered flush.
    let p = pipe_take("full-output", 4);
    for command in ["export", "markers"] {
        let full = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap();
        let o = Command::new(env!("CARGO_BIN_EXE_mixd"))
            .args(["vfl", command])
            .arg(&p)
            .stdout(full)
            .output()
            .unwrap();
        assert!(!o.status.success(), "{command}: {o:?}");
        assert!(!o.stderr.is_empty(), "{command}: {o:?}");
    }
    std::fs::remove_dir_all(p).unwrap();
}
fn ffmpeg(input: &[u8], args: &[&str]) -> Vec<u8> {
    let mut c = Command::new("ffmpeg")
        .args(["-v", "error"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.take().unwrap().write_all(input).unwrap();
    let o = c.wait_with_output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    o.stdout
}
#[test]
fn export_bgra_framemd5_y4m_and_ranges() {
    let p = std::env::temp_dir().join(format!("mixd-vfl-test-{}", std::process::id()));
    let mut w = Writer::new(p.clone(), 4, 4, 30).unwrap();
    w.origin(0).unwrap();
    let a: Vec<u8> = (0..64).map(|i| i * 3).collect();
    let b: Vec<u8> = (0..64).map(|i| 255 - i * 3).collect();
    w.frame(Frame {
        slot: 0,
        pixels: Pixels::Bgra(a.clone().into()),
    })
    .unwrap();
    w.frame(Frame {
        slot: 2,
        pixels: Pixels::Bgra(b.clone().into()),
    })
    .unwrap();
    w.finish(3, 1, 0, None, true).unwrap();
    let export = |args: &[&str]| {
        let o = Command::new(env!("CARGO_BIN_EXE_mixd"))
            .args(["vfl", "export"])
            .arg(&p)
            .args(args)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        o.stdout
    };
    let raw = export(&[]);
    let expected = [a.clone(), a.clone(), b.clone()].concat();
    assert_eq!(raw, expected);
    let md5args = [
        "-f",
        "rawvideo",
        "-pixel_format",
        "bgra",
        "-video_size",
        "4x4",
        "-framerate",
        "30",
        "-i",
        "pipe:0",
        "-f",
        "framemd5",
        "pipe:1",
    ];
    assert_eq!(ffmpeg(&raw, &md5args), ffmpeg(&expected, &md5args));
    assert_eq!(export(&["--from", "1", "--to", "1"]), a);
    let y4m = export(&["--format", "y4m"]);
    let decoded = ffmpeg(
        &y4m,
        &[
            "-i", "pipe:0", "-pix_fmt", "yuv420p", "-f", "rawvideo", "pipe:1",
        ],
    );
    let mut yuv = Vec::new();
    for p in [a.clone(), a, b] {
        let y = media_frame::Bgra::new(4, 4, p).unwrap().to_yuv420();
        yuv.extend(y.y);
        yuv.extend(y.u);
        yuv.extend(y.v);
    }
    assert_eq!(decoded, yuv);
    let o = Command::new(env!("CARGO_BIN_EXE_mixd"))
        .args(["vfl", "verify"])
        .arg(&p)
        .output()
        .unwrap();
    assert!(o.status.success());
    assert!(String::from_utf8(o.stdout).unwrap().contains("\"ok\":true"));
    for fps in ["0", "61"] {
        let o = Command::new(env!("CARGO_BIN_EXE_mixd"))
            .args(["vfl", "export"])
            .arg(&p)
            .args(["--fps", fps])
            .output()
            .unwrap();
        assert!(!o.status.success());
        assert!(String::from_utf8(o.stderr).unwrap().contains("1-60"));
    }
    std::fs::remove_file(p.join("complete.json")).unwrap();
    std::fs::remove_file(p.join("markers.jsonl")).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_mixd"))
        .args(["vfl", "markers"])
        .arg(&p)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(o.stdout.is_empty());
    std::fs::remove_dir_all(p).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_mixd"))
        .args(["vfl", "verify", "/nonexistent-vfl-demo-take"])
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(
        String::from_utf8(o.stdout)
            .unwrap()
            .contains("\"ok\":false")
    );
}
