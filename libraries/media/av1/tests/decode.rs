use media_av1::{Decoder, Encoder, Settings, Target};
use media_webm::{Demuxer, Video, Writer, av1_config};
use std::{io::Cursor, process::Command};
fn ff(args: &[&str]) -> Vec<u8> {
    let o = Command::new("ffmpeg")
        .args(["-v", "error", "-nostdin"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    o.stdout
}
fn available() -> bool {
    if Command::new("ffmpeg").arg("-version").output().is_ok() {
        return true;
    }
    assert_ne!(
        std::env::var("MIX_TEST_REQUIRE_FFMPEG").as_deref(),
        Ok("1"),
        "ffmpeg required"
    );
    false
}
fn compare(bytes: &[u8], label: &str, width: usize, height: usize) {
    let temp = tempfile::tempdir().unwrap();
    let webm = temp.path().join("input.webm");
    std::fs::write(&webm, bytes).unwrap();
    let raw = temp.path().join("ours.yuv");
    let d = Demuxer::new(bytes).unwrap();
    let mut decoder = Decoder::new(2, 500).unwrap();
    let mut pixels = Vec::new();
    let mut count = 0;
    let mut emit = |f: media_av1::DecodedFrame| {
        assert_eq!((f.width, f.height), (width, height));
        for p in f.planes {
            pixels.extend(p);
        }
        count += 1;
        Ok(())
    };
    for p in d.packets() {
        decoder.submit(p.data, p.timestamp, &mut emit).unwrap();
    }
    decoder.finish(&mut emit).unwrap();
    std::fs::write(&raw, pixels).unwrap();
    let ours = ff(&[
        "-f",
        "rawvideo",
        "-pixel_format",
        "yuv420p",
        "-video_size",
        &format!("{width}x{height}"),
        "-i",
        raw.to_str().unwrap(),
        "-f",
        "framemd5",
        "-",
    ]);
    let oracle = ff(&[
        "-c:v",
        "libdav1d",
        "-threads",
        "2",
        "-i",
        webm.to_str().unwrap(),
        "-map",
        "0:v",
        "-fps_mode",
        "passthrough",
        "-f",
        "framemd5",
        "-",
    ]);
    let hashes = |b: &[u8]| -> Vec<String> {
        String::from_utf8_lossy(b)
            .lines()
            .filter(|l| !l.starts_with('#'))
            .map(|l| l.rsplit(',').next().unwrap().trim().to_owned())
            .collect()
    };
    assert_eq!(hashes(&ours), hashes(&oracle), "{label}");
    println!("CHECKSUM {label} {width}x{height} frames={count} MD5=all-equal");
}
#[test]
fn own_encodes_and_show_existing() {
    if !available() {
        return;
    }
    for (width, height) in [(32, 32), (33, 17), (128, 72)] {
        let mut e = Encoder::new(Settings {
            width,
            height,
            fps_num: 25,
            fps_den: 1,
            speed: 10,
            target: Target::Quality(110),
            key_interval: 10,
            tile_cols: 1,
            tile_rows: 1,
            threads: 2,
        })
        .unwrap();
        let f = e
            .prepare(
                &media_frame::Bgra::solid(width, height, [30, 100, 200, 255])
                    .unwrap()
                    .to_yuv420(),
            )
            .unwrap();
        let mut packets = Vec::new();
        for _ in 0..5 {
            packets.extend(e.submit(&f).unwrap());
        }
        packets.extend(e.finish().unwrap());
        for repeats in [false, true] {
            let mut w = Writer::new(
                Cursor::new(Vec::new()),
                Video {
                    width: width as u32,
                    height: height as u32,
                    codec_private: av1_config(&packets[0].data).unwrap(),
                },
                None,
            )
            .unwrap();
            if repeats {
                w.packet(1, 0, 40, true, &packets[0].data).unwrap();
                for i in 1..5 {
                    w.packet(1, i * 40, 40, false, &[0x12, 0, 0x1a, 1, 0x88])
                        .unwrap();
                }
            } else {
                for p in &packets {
                    w.packet(1, p.pts * 40, 40, p.key_frame, &p.data).unwrap();
                }
            }
            compare(
                &w.finish().unwrap().into_inner(),
                if repeats { "show-existing" } else { "rav1e" },
                width,
                height,
            );
        }
    }
}
#[test]
fn independent_encoders() {
    if !available() {
        return;
    }
    for codec in ["libaom-av1", "libsvtav1"] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("independent.webm");
        ff(&[
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=128x128:rate=25:duration=0.4",
            "-c:v",
            codec,
            "-threads",
            "2",
            "-pix_fmt",
            "yuv420p",
            path.to_str().unwrap(),
        ]);
        compare(&std::fs::read(path).unwrap(), codec, 128, 128);
    }
}
#[test]
fn malformed_and_limits() {
    assert!(Decoder::new(0, 1).is_err());
    let mut d = Decoder::new(1, 1).unwrap();
    assert!(d.submit(&[0xff; 128], 0, |_| Ok(())).is_err());
    assert!(d.submit(&[0], 0, |_| Ok(())).is_err());
}
