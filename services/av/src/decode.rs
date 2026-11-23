use media_opus::{Channels, Decoder as _, OpusDecoder};
use media_webm::Demuxer;
use serde_json::json;
use std::{
    ffi::OsString,
    fs::File,
    io::{BufWriter, Read, Write},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn run(probe: bool, mut args: impl Iterator<Item = OsString>) -> Result<()> {
    let input = args.next().ok_or("expected WebM input")?;
    let mut y4m = None;
    let mut wav = None;
    while let Some(arg) = args.next() {
        if probe {
            return Err("probe takes only one input".into());
        }
        if arg == "--y4m" && y4m.is_none() {
            y4m = Some(args.next().ok_or("missing Y4M path")?);
        } else if arg == "--wav" && wav.is_none() {
            wav = Some(args.next().ok_or("missing WAV path")?);
        } else {
            return Err("usage: av decode FILE.webm --y4m OUT.y4m [--wav OUT.wav]".into());
        }
    }
    let mut bytes = Vec::new();
    File::open(&input)?
        .take(256 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    let d = Demuxer::new(&bytes)?;
    if probe {
        let tracks: Vec<_> = d.tracks.iter().map(|t| json!({
            "number": t.number, "codec":t.codec, "width":t.width, "height":t.height,
            "fps": if t.kind == 1 { video_duration(&d,t).map(|ns| 1e9 / ns as f64) } else { None },
            "sample_rate": t.sample_rate, "channels":t.channels,
            "packets": d.packets().iter().filter(|p| p.track == t.number).count()
        })).collect();
        let mut frames = 0;
        if let Some(t) = d.tracks.iter().find(|t| t.codec == "V_AV1") {
            let mut decoder = media_av1::Decoder::new(1, 500_000)?;
            for p in d.packets().iter().filter(|p| p.track == t.number) {
                decoder.submit(p.data, 0, |_| {
                    frames += 1;
                    Ok(())
                })?;
            }
            decoder.finish(|_| {
                frames += 1;
                Ok(())
            })?;
        }
        let duration = d.duration.map(|v| v * d.scale_ns as f64 / 1e9).or_else(|| {
            d.packets()
                .iter()
                .map(|p| {
                    (p.timestamp as f64 + p.duration.unwrap_or(0) as f64) * d.scale_ns as f64 / 1e9
                })
                .reduce(f64::max)
        });
        println!(
            "{}",
            json!({"duration":duration, "tracks":tracks,
            "frames":frames, "packets":d.packets().len(),
            "keyframes":d.packets().iter().filter(|p| p.key_frame && d.tracks.iter().any(|t| t.number == p.track && t.kind == 1)).count(), "cues":d.cues.len()})
        );
        return Ok(());
    }
    let out = y4m.ok_or("--y4m is required")?;
    if out == input || wav.as_ref().is_some_and(|w| w == &input || w == &out) {
        return Err("input and output paths must differ".into());
    }
    let t = d
        .tracks
        .iter()
        .find(|t| t.codec == "V_AV1")
        .ok_or("no AV1 track")?;
    let mut video = BufWriter::new(File::create(out)?);
    let mut decoder = media_av1::Decoder::new(4, 500_000)?;
    let mut size = None;
    let mut emit = |f: media_av1::DecodedFrame| {
        let write = (|| -> std::io::Result<()> {
            if size.is_none() {
                let duration = video_duration(&d, t).unwrap_or(1_000_000_000);
                // VFR timing is preserved in per-frame XPTS tags (nanoseconds).
                let rate = if duration > 0 {
                    format!("1000000000:{duration}")
                } else {
                    "0:0".into()
                };
                writeln!(
                    video,
                    "YUV4MPEG2 W{} H{} F{} Ip A1:1 C420jpeg",
                    f.width, f.height, rate
                )?;
                size = Some((f.width, f.height));
            }
            if size != Some((f.width, f.height)) {
                return Err(std::io::Error::other("Y4M size changed"));
            }
            writeln!(video, "FRAME XPTS={}", f.timestamp_ns)?;
            for plane in &f.planes {
                video.write_all(plane)?;
            }
            Ok(())
        })();
        write.map_err(|e| media_av1::Error(e.to_string()))
    };
    for p in d.packets().iter().filter(|p| p.track == t.number) {
        let ns = (p.timestamp as i128 * d.scale_ns as i128).try_into()?;
        decoder.submit(p.data, ns, &mut emit)?;
    }
    decoder.finish(&mut emit)?;
    video.flush()?;
    if let Some(path) = wav {
        audio(&d, path)?;
    }
    Ok(())
}

fn video_duration(d: &Demuxer<'_>, t: &media_webm::Track<'_>) -> Option<u64> {
    if t.default_duration_ns > 0 {
        return Some(t.default_duration_ns);
    }
    let mut previous = None;
    for p in d.packets().iter().filter(|p| p.track == t.number) {
        if let Some(duration) = p.duration.filter(|n| *n > 0) {
            return duration.checked_mul(d.scale_ns);
        }
        if let Some(last) = previous {
            if p.timestamp > last {
                return (p.timestamp - last)
                    .try_into()
                    .ok()
                    .and_then(|n: u64| n.checked_mul(d.scale_ns));
            }
        }
        previous = Some(p.timestamp);
    }
    None
}

fn audio(d: &Demuxer<'_>, path: OsString) -> Result<()> {
    let t = d
        .tracks
        .iter()
        .find(|t| t.codec == "A_OPUS")
        .ok_or("no Opus track")?;
    let h = t.codec_private;
    if h.len() != 19 || &h[..8] != b"OpusHead" || h[8] > 15 || h[18] != 0 {
        return Err("unsupported OpusHead".into());
    }
    let channels = match h[9] {
        1 => Channels::Mono,
        2 => Channels::Stereo,
        _ => return Err("unsupported Opus channels".into()),
    };
    let n = channels.count();
    let mut skip = usize::from(u16::from_le_bytes([h[10], h[11]]));
    let gain = 10f32.powf(i16::from_le_bytes([h[16], h[17]]) as f32 / (256.0 * 20.0));
    let mut decoder = OpusDecoder::new(channels)?;
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: n as u16,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )?;
    for p in d.packets().iter().filter(|p| p.track == t.number) {
        let pcm = decoder.decode(Some(p.data))?.pcm;
        let padding = ((p.discard_padding_ns.unsigned_abs() as u128 * 48_000 + 500_000_000)
            / 1_000_000_000) as usize;
        let len = pcm.len() / n;
        if padding > len {
            return Err("Opus discard padding exceeds packet".into());
        }
        let start_pad = if p.discard_padding_ns < 0 { padding } else { 0 };
        let end_pad = if p.discard_padding_ns > 0 { padding } else { 0 };
        let available = len - start_pad - end_pad;
        let skipped = skip.min(available);
        skip -= skipped;
        for sample in &pcm[(start_pad + skipped) * n..(len - end_pad) * n] {
            writer.write_sample(*sample * gain)?;
        }
    }
    writer.finalize()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use media_opus::{Encoder as _, OpusEncoder};
    use std::{io::Cursor, process::Command};
    fn ff(args: &[&str]) -> Vec<u8> {
        let o = Command::new("ffmpeg")
            .args(["-v", "error", "-nostdin"])
            .args(args)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        o.stdout
    }
    #[test]
    fn opus_sample_count_and_pcm_oracle() {
        if Command::new("ffmpeg").arg("-version").output().is_err() {
            assert_ne!(std::env::var("MIX_TEST_REQUIRE_FFMPEG").as_deref(), Ok("1"));
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        for channels in [Channels::Mono, Channels::Stereo] {
            let mut enc = OpusEncoder::new(channels, 96000).unwrap();
            let head = enc.head().clone();
            let mut w = media_webm::Writer::audio(
                Cursor::new(Vec::new()),
                media_webm::Opus {
                    channels: head.channels,
                    pre_skip: head.pre_skip,
                    input_sample_rate: 48000,
                },
            )
            .unwrap();
            for i in 0..10 {
                let pcm: Vec<_> = (0..960)
                    .flat_map(|j| {
                        let sample =
                            ((i * 960 + j) as f32 * std::f32::consts::TAU * 440.0 / 48000.0).sin()
                                * 0.1;
                        vec![sample; channels.count()]
                    })
                    .collect();
                let p = enc.encode(&pcm).unwrap();
                w.opus_packet(i * 20, &p.data, 0).unwrap();
            }
            if let Some(p) = enc.finish().unwrap() {
                w.opus_packet(200, &p.data, p.discard_padding).unwrap();
            }
            let own = w.finish().unwrap().into_inner();
            let path = dir.path().join("ff.webm");
            ff(&[
                "-y",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=48000:duration=0.200354167",
                "-ac",
                &channels.count().to_string(),
                "-c:a",
                "libopus",
                path.to_str().unwrap(),
            ]);
            let foreign = std::fs::read(&path).unwrap();
            for (label, bytes) in [("own", own), ("ffmpeg", foreign)] {
                let input = dir.path().join("input.webm");
                std::fs::write(&input, &bytes).unwrap();
                let wav = dir.path().join("out.wav");
                audio(&Demuxer::new(&bytes).unwrap(), wav.clone().into_os_string()).unwrap();
                let ours: Vec<f32> = hound::WavReader::open(wav)
                    .unwrap()
                    .samples()
                    .map(|s| s.unwrap())
                    .collect();
                let reference = ff(&[
                    "-c:a",
                    "libopus",
                    "-i",
                    input.to_str().unwrap(),
                    "-f",
                    "f32le",
                    "-",
                ]);
                let reference: Vec<f32> = reference
                    .chunks_exact(4)
                    .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                    .collect();
                assert_eq!(ours.len(), reference.len(), "{label} sample count");
                let rms = (ours
                    .iter()
                    .zip(&reference)
                    .map(|(a, b)| f64::from(a - b).powi(2))
                    .sum::<f64>()
                    / ours.len() as f64)
                    .sqrt();
                assert!(rms <= 0.001, "{label} RMS {rms}");
                println!(
                    "OPUS {label} channels={} samples={} RMS_dBFS={:.2}",
                    channels.count(),
                    ours.len() / channels.count(),
                    20.0 * rms.log10()
                );
            }
        }
    }
}
