use super::*;
use std::process::Command;

fn options() -> Options {
    Options::parse(std::iter::empty()).unwrap()
}
const HEADER: &str = "YUV4MPEG2 W64 H48 F30000:1001 Ip A1:1 C420jpeg\n";

#[test]
fn refusals() {
    for header in [
        HEADER.replace("C420jpeg", "C444"),
        HEADER.replace("C420jpeg", "C420p10"),
        HEADER.replace("Ip", "It"),
        HEADER.replace("Ip", "I?"),
        HEADER.replace("F30000:1001", "F0:1"),
        HEADER.replace("W64", "W0"),
        HEADER.replace("W64", "W65536"),
        HEADER.replace("W64", "W+64"),
        HEADER.replace("F30000:1001", "F30:+1"),
        HEADER.replace("W64 H48", "W8192 H4321"),
        HEADER.replace("W64 H48", "W65535 H65535"),
        HEADER.replace("A1:1", "A1:1 A1:1"),
        HEADER.replace("A1:1", "A16:9"),
        HEADER.replace("A1:1", "XBITDEPTH=10"),
        HEADER.replace("W64", "W64 W32"),
        HEADER.replace("A1:1", "é"),
        HEADER.replace("A1:1", "XCOLORRANGE=FULL"),
    ] {
        assert!(Header::parse(header.trim()).is_err(), "{header}");
    }
    for input in [
        String::new(),
        HEADER.into(),
        format!("{HEADER}FRAME\nshort"),
        format!("{HEADER}FRAME W32\n"),
        format!("{HEADER}FRAME F25:1\n"),
        format!("{HEADER}{HEADER}"),
        "x".repeat(4097),
    ] {
        assert!(encode(&mut input.as_bytes(), &mut Vec::new(), &options()).is_err());
    }
    for args in [vec!["--qp", "256"], vec!["--threads"], vec!["--oops", "1"]] {
        assert!(Options::parse(args.into_iter().map(str::to_owned)).is_err());
    }
}

#[test]
fn ivf_fields_and_packets() {
    let h = Header::parse(HEADER.trim()).unwrap();
    let mut out = Vec::new();
    ivf_header(&mut out, h).unwrap();
    assert_eq!(out.len(), 32);
    assert_eq!(&out[..12], b"DKIF\0\0\x20\0AV01");
    assert_eq!(&out[12..16], &[64, 0, 48, 0]);
    assert_eq!(&out[16..20], &30000u32.to_le_bytes());
    assert_eq!(&out[20..24], &1001u32.to_le_bytes());
    assert_eq!(&out[24..32], &[0; 8]);
    assert_eq!(
        packets(
            &mut out,
            vec![Packet {
                data: vec![7, 8],
                pts: 42,
                duration: 1,
                key_frame: true
            }]
        )
        .unwrap(),
        (14, 1)
    );
    assert_eq!(&out[32..36], &2u32.to_le_bytes());
    assert_eq!(&out[36..44], &42u64.to_le_bytes());
    assert_eq!(&out[44..], &[7, 8]);
}

#[test]
fn round_trip_dav1d_psnr() {
    if !has_dav1d() { return; }
    let root = std::env::temp_dir().join(format!(
        "av1enc-oracle-{}-{}",
        std::process::id(),
        cfg!(feature = "asm")
    ));
    std::fs::create_dir(&root).unwrap();
    let result = std::panic::catch_unwind(|| {
        let (y4m, raw) = specimen();
        let mut ivf = Vec::new();
        let stats = encode(&mut y4m.as_slice(), &mut ivf, &options()).unwrap();
        assert_eq!(stats.frames, 5);
        assert_eq!(stats.bytes, ivf.len() as u64);
        std::fs::write(root.join("out.ivf"), ivf).unwrap();
        let decoded = Command::new("ffmpeg")
            .args(["-v", "error", "-c:v", "libdav1d", "-i"])
            .arg(root.join("out.ivf"))
            .args(["-f", "rawvideo", "-pix_fmt", "yuv420p", "pipe:1"])
            .output()
            .expect("worker requires ffmpeg with libdav1d");
        assert!(
            decoded.status.success(),
            "ffmpeg/libdav1d required: {}",
            String::from_utf8_lossy(&decoded.stderr)
        );
        assert_eq!(
            decoded.stdout.len(),
            raw.len(),
            "decoded frame count must be five"
        );
        let psnr = raw.chunks_exact(64 * 48 * 3 / 2).zip(decoded.stdout.chunks_exact(64 * 48 * 3 / 2))
            .map(|(a,b)| {
                let mse = a[..64*48].iter().zip(&b[..64*48]).map(|(a,b)| (f64::from(*a)-f64::from(*b)).powi(2)).sum::<f64>() / (64*48) as f64;
                10.0 * (255.0 * 255.0 / mse).log10()
            }).fold(f64::INFINITY, f64::min);
        eprintln!(
            "av1enc oracle: frames=5 qp=100 speed=10 min_frame_luma_PSNR={psnr:.4} dB floor=25 dB asm={}",
            cfg!(feature = "asm")
        );
        assert!(psnr > 25.0, "min frame luma PSNR {psnr} <= 25 dB");
    });
    std::fs::remove_dir_all(root).unwrap();
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }
}

fn has_dav1d() -> bool {
    let missing = |what: &str| {
        assert_ne!(std::env::var("MIX_TEST_REQUIRE_FFMPEG").as_deref(), Ok("1"), "required oracle missing: {what}");
        eprintln!("SKIP: {what}");
        false
    };
    match Command::new("ffmpeg").arg("-version").output() {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return missing("ffmpeg"),
        Err(e) => panic!("cannot probe ffmpeg: {e}"),
        Ok(o) => assert!(o.status.success(), "ffmpeg probe failed"),
    }
    let o = Command::new("ffmpeg").args(["-hide_banner", "-decoders"]).output().unwrap();
    assert!(o.status.success(), "ffmpeg decoder probe failed");
    if !String::from_utf8_lossy(&o.stdout).split_whitespace().any(|s| s == "libdav1d") {
        return missing("ffmpeg libdav1d decoder");
    }
    true
}

fn specimen() -> (Vec<u8>, Vec<u8>) {
    let mut y4m = HEADER.as_bytes().to_vec();
    let mut raw = Vec::new();
    for t in 0..5 {
        let mut pixels = Vec::new();
        for y in 0..48 {
            for x in 0..64 {
                let moving_x = (x + t * 5) % 64;
                let texture = ((moving_x * 7 + y * 11) % 31) as u8;
                let base = if (moving_x / 8 + y / 8) % 2 == 0 { 50 } else { 150 };
                pixels.extend_from_slice(&[base + texture, base, 200 - texture, 255]);
            }
        }
        let frame = media_frame::Bgra::new(64, 48, pixels).unwrap().to_yuv420();
        y4m.extend_from_slice(b"FRAME\n");
        for p in [&frame.y, &frame.u, &frame.v] { y4m.extend(p); raw.extend(p); }
    }
    (y4m, raw)
}

#[test]
fn encode_without_external_tools() {
    let (y4m, _) = specimen();
    let mut out = Vec::new();
    let stats = encode(&mut y4m.as_slice(), &mut out, &options()).unwrap();
    assert!(stats.seconds > 0.0);
    assert_eq!(stats.bytes, out.len() as u64);
    let mut offset = 32;
    for pts in 0u64..5 {
        let len = u32::from_le_bytes(out[offset..offset+4].try_into().unwrap()) as usize;
        assert!(len > 0);
        assert_eq!(&out[offset+4..offset+12], &pts.to_le_bytes());
        offset += 12 + len;
    }
    assert_eq!(offset, out.len());
    assert_eq!(stats.frames, 5);
}

#[test]
fn cap_and_options() {
    assert!(Header::parse(&HEADER.replace("W64 H48", "W8192 H4320")).is_ok());
    let h = Header::parse(&HEADER.replace("W64 H48", "W8192 H4320")).unwrap();
    // Maximum permitted header with EOF must not preallocate a full plane.
    assert!(h.frame(&mut b"FRAME\n".as_slice()).is_err());
    let o = Options::parse(["--speed", "8", "--qp", "77", "--threads", "2", "--tile-cols", "2", "--tile-rows", "4", "--keyint", "12", "--out", "out.ivf"].into_iter().map(str::to_owned)).unwrap();
    assert_eq!((o.speed,o.qp,o.threads,o.cols,o.rows,o.keyint), (8,77,2,2,4,12));
    assert_eq!(o.out.as_deref(), Some("out.ivf"));
    assert!(Options::parse(["--help".into()].into_iter()).unwrap().help);
    for (flag,value) in [("--speed","11"),("--threads","0"),("--tile-cols","3"),("--tile-rows","0"),("--keyint","0")] {
        assert!(Options::parse([flag.into(),value.into()].into_iter()).is_err());
    }
}
