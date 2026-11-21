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
        14
    );
    assert_eq!(&out[32..36], &2u32.to_le_bytes());
    assert_eq!(&out[36..44], &42u64.to_le_bytes());
    assert_eq!(&out[44..], &[7, 8]);
}

#[test]
fn round_trip_dav1d_psnr() {
    let root = std::env::temp_dir().join(format!(
        "av1enc-oracle-{}-{}",
        std::process::id(),
        cfg!(feature = "asm")
    ));
    std::fs::create_dir(&root).unwrap();
    let result = std::panic::catch_unwind(|| {
        let mut y4m = HEADER.as_bytes().to_vec();
        let mut raw = Vec::new();
        for t in 0..5 {
            let mut bgra = media_frame::Bgra::solid(64, 48, [40 + t * 4, 80, 120, 255])
                .unwrap()
                .to_yuv420();
            for (i, y) in bgra.y.iter_mut().enumerate() {
                *y += (i % 11) as u8;
            }
            y4m.extend_from_slice(b"FRAME\n");
            for plane in [&bgra.y, &bgra.u, &bgra.v] {
                y4m.extend_from_slice(plane);
                raw.extend_from_slice(plane);
            }
        }
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
        let mse = raw
            .iter()
            .zip(&decoded.stdout)
            .map(|(a, b)| (f64::from(*a) - f64::from(*b)).powi(2))
            .sum::<f64>()
            / raw.len() as f64;
        let psnr = 10.0 * (255.0 * 255.0 / mse).log10();
        eprintln!(
            "av1enc oracle: frames=5 qp=100 speed=10 PSNR={psnr:.4} dB floor=35 dB asm={}",
            cfg!(feature = "asm")
        );
        assert!(psnr > 35.0, "PSNR {psnr} <= 35 dB");
    });
    std::fs::remove_dir_all(root).unwrap();
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }
}
