use media_opus::{Channels, Decoder, Encoder, FRAME_SAMPLES, OpusDecoder, OpusEncoder, Packet};
use std::{fs, path::Path, process::Command};
mod support;

fn oracle_available() -> bool {
    let version = Command::new("ffmpeg").arg("-version").output();
    if !support::available(version.is_ok_and(|o| o.status.success()), "ffmpeg is missing or cannot execute") { return false; }
    for kind in ["-encoders", "-decoders"] {
        let probe = Command::new("ffmpeg").args(["-hide_banner", kind]).output();
        if !support::available(probe.is_ok_and(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).lines().any(|line| line.split_whitespace().nth(1) == Some("libopus"))), &format!("ffmpeg libopus {kind} unavailable")) { return false; }
    }
    true
}

fn ffmpeg(args: &[&str]) {
    let output = Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-y"])
        .args(args)
        .output()
        .expect("worker requires ffmpeg with libopus");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn page(packet: &[u8], sequence: u32, granule: u64, flags: u8, out: &mut Vec<u8>) {
    let mut bytes = Vec::from(&b"OggS\0"[..]);
    bytes.push(flags);
    bytes.extend_from_slice(&granule.to_le_bytes());
    bytes.extend_from_slice(&7u32.to_le_bytes());
    bytes.extend_from_slice(&sequence.to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.push((packet.len() / 255 + 1) as u8);
    bytes.extend(std::iter::repeat_n(255, packet.len() / 255));
    bytes.push((packet.len() % 255) as u8);
    bytes.extend_from_slice(packet);
    let mut crc = 0u32;
    for &byte in &bytes {
        crc ^= u32::from(byte) << 24;
        for _ in 0..8 {
            crc = if crc & 0x8000_0000 != 0 {
                (crc << 1) ^ 0x04c1_1db7
            } else {
                crc << 1
            };
        }
    }
    bytes[22..26].copy_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&bytes);
}
fn ogg(head: &[u8], packets: &[Packet]) -> Vec<u8> {
    let mut bytes = Vec::new();
    page(head, 0, 0, 2, &mut bytes);
    page(b"OpusTags\0\0\0\0\0\0\0\0", 1, 0, 0, &mut bytes);
    for (i, packet) in packets.iter().enumerate() {
        page(
            &packet.data,
            i as u32 + 2,
            (i as u64 + 1) * FRAME_SAMPLES as u64 - u64::from(packet.discard_padding),
            if i + 1 == packets.len() { 4 } else { 0 },
            &mut bytes,
        );
    }
    bytes
}
fn demux(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut packets = Vec::new();
    let mut pending = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        assert_eq!(&bytes[offset..offset + 4], b"OggS");
        let segments = bytes[offset + 26] as usize;
        let mut payload = offset + 27 + segments;
        for &size in &bytes[offset + 27..payload] {
            let end = payload + usize::from(size);
            pending.extend_from_slice(&bytes[payload..end]);
            payload = end;
            if size < 255 {
                if !pending.starts_with(b"OpusHead") && !pending.starts_with(b"OpusTags") {
                    packets.push(std::mem::take(&mut pending));
                }
                pending.clear();
            }
        }
        offset = payload;
    }
    packets
}
fn read_pcm(path: &Path) -> Vec<f32> {
    fs::read(path)
        .unwrap()
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
        .collect()
}
fn snr(a: &[f32], b: &[f32]) -> f64 {
    let signal: f64 = a.iter().map(|&v| f64::from(v).powi(2)).sum();
    let error: f64 = a
        .iter()
        .zip(b)
        .map(|(&a, &b)| (f64::from(a) - f64::from(b)).powi(2))
        .sum();
    10. * (signal / error.max(1e-30)).log10()
}

#[test]
fn libopus_interop_and_container_timing() {
    if !oracle_available() { return; }
    let directory = support::TempDir::new("interop");
    let work = directory.path();
    let source_path = work.join("source.f32");
    let candidate_path = work.join("candidate.ogg");
    let oracle_path = work.join("oracle.ogg");
    let decoded_path = work.join("decoded.f32");
    let oracle_pcm = work.join("oracle.f32");
    for channels in [Channels::Mono, Channels::Stereo] {
        let count = channels.count();
        let source: Vec<f32> = (0..48_000)
            .flat_map(|i| {
                (0..count).map(move |c| {
                    let t = i as f32 / 48_000.;
                    0.35 * (std::f32::consts::TAU * (440. + c as f32 * 73.) * t).sin()
                })
            })
            .collect();
        let input: Vec<u8> = source.iter().flat_map(|v| v.to_le_bytes()).collect();
        fs::write(&source_path, input).unwrap();
        for bitrate in [64_000, 96_000, 128_000] {
            let mut encoder = OpusEncoder::new(channels, bitrate).unwrap();
            let mut packets = Vec::new();
            for frame in source.chunks_exact(FRAME_SAMPLES * count) {
                packets.push(encoder.encode(frame).unwrap());
            }
            if let Some(tail) = encoder.finish().unwrap() {
                packets.push(tail);
            }
            fs::write(&candidate_path, ogg(&encoder.head().bytes(), &packets)).unwrap();
            ffmpeg(&[
                "-c:a",
                "libopus",
                "-i",
                candidate_path.to_str().unwrap(),
                "-f",
                "f32le",
                decoded_path.to_str().unwrap(),
            ]);
            let forward = read_pcm(&decoded_path);
            assert_eq!(
                forward.len(),
                source.len(),
                "pre-skip and discard padding must preserve duration"
            );
            ffmpeg(&[
                "-f",
                "f32le",
                "-ar",
                "48000",
                "-ac",
                &count.to_string(),
                "-i",
                source_path.to_str().unwrap(),
                "-c:a",
                "libopus",
                "-b:a",
                &bitrate.to_string(),
                "-frame_duration",
                "20",
                oracle_path.to_str().unwrap(),
            ]);
            ffmpeg(&[
                "-c:a",
                "libopus",
                "-i",
                oracle_path.to_str().unwrap(),
                "-f",
                "f32le",
                oracle_pcm.to_str().unwrap(),
            ]);
            let reference = read_pcm(&oracle_pcm);
            assert_eq!(reference.len(), source.len());
            let quality = snr(&source, &forward);
            let oracle_quality = snr(&source, &reference);
            println!(
                "{channels:?} {bitrate}: source SNR candidate={quality} libopus={oracle_quality}"
            );
            assert!(
                quality >= oracle_quality - 3.,
                "encoder quality outside spike band"
            );
            let encoded = fs::read(&oracle_path).unwrap();
            let skip = u16::from_le_bytes(encoded[38..40].try_into().unwrap()) as usize;
            let mut decoder = OpusDecoder::new(channels).unwrap();
            let mut reverse = Vec::new();
            for packet in demux(&encoded) {
                let output = decoder.decode(Some(&packet)).unwrap();
                assert!(!output.packet_lost);
                reverse.extend(output.pcm);
            }
            let aligned = &reverse[skip * count..skip * count + reference.len()];
            assert!(
                snr(&reference, aligned) > 50.,
                "decoder mismatch against libopus"
            );
        }
    }
}
