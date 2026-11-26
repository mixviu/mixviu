use media_audio::pending::Pending;
use media_opus::{Channels, Decoder as _, OpusDecoder};
use media_webm::Demuxer;
use md5::{Digest, Md5};
use serde_json::json;
use std::{
    ffi::OsString,
    fs::File,
    io::{self, BufWriter, Cursor, Read, Write},
    path::Path,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const DEFAULT_MAX_OUTPUT_BYTES: u64 = 64 * 1024 * 1024 * 1024;
const FRAME_MARGIN: u64 = 2;
const MAX_PROBE_RATE: i128 = 1000;
const MAX_PROBE_CANDIDATES: i128 = 64;

struct OutputBudget {
    limit: u64,
    used: u64,
}
impl OutputBudget {
    fn charge(&mut self, bytes: u64) -> Result<()> {
        self.used = self
            .used
            .checked_add(bytes)
            .filter(|n| *n <= self.limit)
            .ok_or("OUTPUT_LIMIT_EXCEEDED: total output byte cap")?;
        Ok(())
    }
}

pub fn run(probe: bool, mut args: impl Iterator<Item = OsString>) -> Result<u8> {
    let input = args.next().ok_or("expected WebM input")?;
    let mut y4m = None;
    let mut framemd5 = None;
    let mut wav = None;
    let mut max_output_bytes = None;
    while let Some(arg) = args.next() {
        if probe {
            return Err("probe takes only one input".into());
        }
        if arg == "--y4m" && y4m.is_none() {
            y4m = Some(args.next().ok_or("missing Y4M path")?);
        } else if arg == "--framemd5" && framemd5.is_none() {
            framemd5 = Some(args.next().ok_or("missing framemd5 path")?);
        } else if arg == "--wav" && wav.is_none() {
            wav = Some(args.next().ok_or("missing WAV path")?);
        } else if arg == "--max-output-bytes" && max_output_bytes.is_none() {
            let value = args.next().ok_or("missing output byte cap")?;
            let limit: u64 = value
                .to_str()
                .ok_or("INVALID_OUTPUT_LIMIT")?
                .parse()
                .map_err(|_| "INVALID_OUTPUT_LIMIT: expected a positive integer")?;
            if limit == 0 {
                return Err("INVALID_OUTPUT_LIMIT: expected a positive integer".into());
            }
            max_output_bytes = Some(limit);
        } else {
            return Err(
                "usage: av decode FILE.webm [--y4m OUT.y4m] [--framemd5 OUT.txt] [--wav OUT.wav] [--max-output-bytes N]"
                    .into(),
            );
        }
    }
    let mut bytes = Vec::new();
    File::open(&input)?
        .take(256 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    let d = Demuxer::new(&bytes)?;
    if probe {
        let mut tracks = Vec::new();
        for t in &d.tracks {
            tracks.push(json!({
                "number": t.number, "codec":t.codec, "width":t.width, "height":t.height,
                "fps": if t.kind == 1 { probe_video_rate(t.default_duration_ns, d.scale_ns,
                    d.packets().iter().filter(|p| p.track == t.number)) } else { None },
                "sample_rate": t.sample_rate, "channels":t.channels,
                "packets": d.packets().iter().filter(|p| p.track == t.number).count()
            }));
        }
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
        let receipt = json!({"duration":duration, "tracks":tracks,
            "frames":frames, "packets":d.packets().len(),
            "keyframes":d.packets().iter().filter(|p| p.key_frame && d.tracks.iter().any(|t| t.number == p.track && t.kind == 1)).count(), "cues":d.cues.len()});
        match receipt_line(&serde_json::to_string(&receipt)?) {
            Err(e) if e.kind() != io::ErrorKind::BrokenPipe => return Err(e.into()),
            _ => return Ok(0),
        }
    }
    if y4m.is_none() && framemd5.is_none() {
        return Err("VIDEO_OUTPUT_REQUIRED: --y4m or --framemd5 is required".into());
    }
    let paths: Vec<_> = [&y4m, &framemd5, &wav].into_iter().flatten().collect();
    for (i, path) in paths.iter().enumerate() {
        if **path == input || paths[..i].contains(path) {
            return Err("OUTPUT_PATH_CONFLICT: input and output paths must differ".into());
        }
    }
    let t = d
        .tracks
        .iter()
        .find(|t| t.codec == "V_AV1")
        .ok_or("no AV1 track")?;
    let frame_duration = video_duration(&d, t)?;
    let max_frames = duration_frame_limit(&d, t, frame_duration)?;
    let inferred_rate = probe_video_rate(t.default_duration_ns, d.scale_ns,
        d.packets().iter().filter(|p| p.track == t.number));
    let mut budget = OutputBudget {
        limit: max_output_bytes.unwrap_or(DEFAULT_MAX_OUTPUT_BYTES),
        used: 0,
    };
    let mut output = y4m.as_ref().map(|p| Pending::new(Path::new(p))).transpose()?;
    let mut md5_output = framemd5.as_ref().map(|p| Pending::new(Path::new(p))).transpose()?;
    let mut wav_output = wav
        .as_ref()
        .map(|path| Pending::new(Path::new(path)))
        .transpose()?;
    let mut video = output.as_mut().map(|p| BufWriter::new(&mut p.file));
    let mut hashes = md5_output.as_mut().map(|p| BufWriter::new(&mut p.file));
    let mut decoder = media_av1::Decoder::new(4, 500_000)?;
    let mut size = None;
    let mut frames = 0u64;
    let mut emit = |f: media_av1::DecodedFrame| {
        let write = (|| -> Result<()> {
            let next = frames.checked_add(1).ok_or("FRAME_COUNT_LIMIT_EXCEEDED")?;
            if next > max_frames {
                return Err("FRAME_COUNT_LIMIT_EXCEEDED: container duration and frame rate".into());
            }
            let frame_bytes = f
                .planes
                .iter()
                .try_fold(0u64, |n, p| n.checked_add(p.len() as u64))
                .ok_or("OUTPUT_LIMIT_EXCEEDED: frame size overflow")?;
            if video.is_some() && next
                .checked_mul(frame_bytes)
                .is_none_or(|n| n > budget.limit)
            {
                return Err("OUTPUT_LIMIT_EXCEEDED: frames times frame bytes".into());
            }
            let mut header = String::new();
            if size.is_none() {
                // VFR timing is preserved in per-frame XPTS tags (nanoseconds).
                let rate = header_rate(inferred_rate, frame_duration);
                header = format!(
                    "YUV4MPEG2 W{} H{} F{} Ip A1:1 C420jpeg\n",
                    f.width, f.height, rate
                );
                size = Some((f.width, f.height));
            }
            if size != Some((f.width, f.height)) {
                return Err("Y4M size changed".into());
            }
            let tag = format!("FRAME XPTS={}\n", f.timestamp_ns);
            if let Some(video) = &mut video {
                budget.charge(
                frame_bytes
                    .checked_add((header.len() + tag.len()) as u64)
                    .ok_or("OUTPUT_LIMIT_EXCEEDED")?,
            )?;
            video.write_all(header.as_bytes())?;
            video.write_all(tag.as_bytes())?;
            for plane in &f.planes {
                video.write_all(plane)?;
            }
            }
            if let Some(hashes) = &mut hashes {
                let mut text = String::new();
                if frames == 0 {
                    text = format!("#av-framemd5 v1 W{} H{} yuv420p\n", f.width, f.height);
                }
                let mut digest = Md5::new();
                for plane in &f.planes {
                    digest.update(plane);
                }
                text.push_str(&format!("{} {:x} {} {}\n", frames, digest.finalize(), frame_bytes, f.timestamp_ns));
                budget.charge(text.len() as u64)?;
                hashes.write_all(text.as_bytes())?;
            }
            frames = next;
            Ok(())
        })();
        write.map_err(|e| media_av1::Error(e.to_string()))
    };
    for p in d.packets().iter().filter(|p| p.track == t.number) {
        let ns = (p.timestamp as i128 * d.scale_ns as i128)
            .try_into()
            .map_err(|_| "TIMESTAMP_OVERFLOW")?;
        decoder.submit(p.data, ns, &mut emit)?;
    }
    decoder.finish(&mut emit)?;
    if frames == 0 {
        return Err("NO_VIDEO_FRAMES: AV1 stream emitted no video frames".into());
    }
    if let Some(video) = &mut video { video.flush()?; }
    if let Some(hashes) = &mut hashes { hashes.flush()?; }
    drop(video);
    drop(hashes);
    if let Some(wav) = &mut wav_output {
        audio(&d, &mut wav.file, &mut budget)?;
    }
    let receipt = serde_json::to_string(
        &json!({"published":true,"frames":frames,"output_bytes":budget.used,
            "framemd5":framemd5.is_some(), "framemd5_lines":if framemd5.is_some() { frames } else { 0 }}),
    )?;
    if let Some(output) = &mut output { output.publish()?; }
    if let Some(output) = &mut md5_output { output.publish()?; }
    if let Some(wav) = &mut wav_output {
        wav.publish()?;
    }
    if let Some(wav) = &mut wav_output {
        wav.commit();
    }
    if let Some(output) = &mut output { output.commit(); }
    if let Some(output) = &mut md5_output { output.commit(); }
    if let Err(e) = receipt_line(&receipt)
        && e.kind() != io::ErrorKind::BrokenPipe
    {
        let _ =
            media_audio::cli::diagnostic(format_args!("published but receipt not written: {e}"));
        return Ok(4);
    }
    Ok(0)
}

fn header_rate(rate: Option<f64>, duration: Option<u64>) -> String {
    if let Some(rate) = rate {
        if (rate - rate.round()).abs() < 0.00001 {
            return format!("{}:1", rate.round() as u64);
        }
        let numerator = (rate * 1001.0).round();
        if (rate - numerator / 1001.0).abs() < 0.00001 {
            return format!("{}:1001", numerator as u64);
        }
        // Exact uniform intervals and DefaultDuration need not be standard grids.
        return format!("1000000000:{}", (1e9 / rate).round() as u64);
    }
    format!("1000000000:{}", duration.unwrap_or(1_000_000_000))
}

fn receipt_line(receipt: &str) -> io::Result<()> {
    let mut out = io::stdout().lock();
    writeln!(out, "{receipt}").and_then(|()| out.flush())
}

fn duration_frame_limit(
    d: &Demuxer<'_>,
    track: &media_webm::Track<'_>,
    interval: Option<u64>,
) -> Result<u64> {
    if let (Some(duration), Some(interval)) = (d.duration, interval) {
        if track.default_duration_ns == 0 {
            // A single packet delta does not establish the frame rate of VFR.
            // Only infer a count ceiling when every declared duration/delta agrees.
            let mut previous = None;
            for packet in d.packets().iter().filter(|p| p.track == track.number) {
                if let Some(ticks) = packet.duration {
                    let ns = ticks.checked_mul(d.scale_ns).ok_or("TIMESTAMP_OVERFLOW")?;
                    if ns != interval {
                        return Ok(500_000);
                    }
                }
                if let Some(last) = previous {
                    let ticks = packet
                        .timestamp
                        .checked_sub(last)
                        .ok_or("TIMESTAMP_OVERFLOW")?;
                    let Ok(ticks) = u64::try_from(ticks) else {
                        return Ok(500_000);
                    };
                    let ns = ticks.checked_mul(d.scale_ns).ok_or("TIMESTAMP_OVERFLOW")?;
                    if ns != interval {
                        return Ok(500_000);
                    }
                }
                previous = Some(packet.timestamp);
            }
        }
        let frames = (duration * d.scale_ns as f64 / interval as f64).ceil();
        if !frames.is_finite() {
            return Err("TIMESTAMP_OVERFLOW: container duration".into());
        }
        return Ok((frames as u64).saturating_add(FRAME_MARGIN).min(500_000));
    }
    // Without Segment Duration or a nominal frame interval there is no
    // independent duration/rate bound. The byte budget and 500k ceiling remain.
    Ok(500_000)
}

/// Probe timing only; Segment Duration and decoded frame count are independent.
/// DefaultDuration wins, then an exact uniform tick interval (>= 3 packets).
/// Otherwise test endpoint-plausible N Hz and N*1000/1001 Hz grids in the
/// inclusive 1..=1000 fps range. Preflight both windows before scanning any
/// grid: more than 64 candidate slots in total (before deduplication) gives
/// unknown. DefaultDuration and exact uniform timing need no candidate search
/// and retain their full range. With P container packets, work is O(C*P), C <= 64,
/// plus O(P) preflight passes, and O(1) extra space. P is already bounded by
/// Demuxer's 256 MiB input / 128 MiB index limits.
/// Test every remaining grid,
/// returning a rate only for one match. Fractional fps is returned as f64
/// (e.g. 30000/1001), not rounded to 29.97 or a neighbouring integer.
/// A grid needs residual spread strictly below one TimestampScale tick and
/// consistent block durations. Symmetric +/- half-tick rounding is deliberately
/// rejected at equality: conservative unknown, never a relaxed extra match.
///
/// Horizon: periods P and Q accumulate one tick T of relative drift after
/// ceil(T/abs(P-Q)) intervals. With unknown quantisation phase, a sufficient
/// phase-independent distinguishing horizon is ceil(2*T/abs(P-Q)) + 1 frames;
/// before that, enumerate both hypotheses rather than trusting the average.
/// For N vs N*1000/1001, abs(P-Q)=1e9/(1000*N) ns: at T=1 ms, 30 vs
/// 30000/1001 needs 61 frames for this bound. Actual grids can separate sooner.
fn probe_video_rate<'a>(
    default_ns: u64,
    scale_ns: u64,
    packets: impl Iterator<Item = &'a media_webm::Packet<'a>> + Clone,
) -> Option<f64> {
    probe_video_rate_with_work(default_ns, scale_ns, packets, || {})
}

// The observer lets tests assert the actual number of grid scans, independent
// of machine speed. The no-op production observer is optimised away.
fn probe_video_rate_with_work<'a>(
    default_ns: u64,
    scale_ns: u64,
    packets: impl Iterator<Item = &'a media_webm::Packet<'a>> + Clone,
    mut tested_candidate: impl FnMut(),
) -> Option<f64> {
    if default_ns > 0 {
        return Some(1e9 / default_ns as f64);
    }
    let count = packets.clone().count();
    if count < 3 || scale_ns == 0 {
        return None;
    }
    let first = packets.clone().next()?.timestamp;
    let delta = i128::from(packets.clone().nth(1)?.timestamp) - i128::from(first);
    if delta <= 0 {
        return None;
    }
    let mut uniform = true;
    let mut previous = first;
    for (i, packet) in packets.clone().enumerate() {
        if i > 0 && packet.timestamp <= previous {
            return None;
        }
        previous = packet.timestamp;
        uniform &= i128::from(packet.timestamp) - i128::from(first) == i as i128 * delta;
    }
    if uniform {
        if packets
            .clone()
            .any(|p| p.duration.is_some_and(|d| i128::from(d) != delta))
        {
            return None;
        }
        return Some(1e9 / delta.checked_mul(i128::from(scale_ns))? as f64);
    }

    let tick = i128::from(scale_ns);
    let span = (i128::from(previous) - i128::from(first)).checked_mul(tick)?;
    let mut windows = [(1, 1, 0, -1), (1000, 1001, 0, -1)];
    let mut candidates = 0;
    for (multiplier, denominator, lower, upper) in &mut windows {
        let period_numerator = 1_000_000_000 * *denominator;
        let elapsed = (count - 1) as i128 * period_numerator;
        // Endpoint residual must be strictly within one tick. These open
        // bounds enumerate every plausible N, not just the rounded mean.
        let lower_divisor = span.checked_add(tick)?.checked_mul(*multiplier)?;
        let upper_divisor = span.checked_sub(tick)?.checked_mul(*multiplier)?;
        // 1 <= N*multiplier/denominator <= MAX_PROBE_RATE, exactly.
        *lower = (elapsed / lower_divisor + 1).max((*denominator + *multiplier - 1) / *multiplier);
        *upper = ((elapsed - 1) / upper_divisor).min(MAX_PROBE_RATE * *denominator / *multiplier);
        candidates += (*upper - *lower + 1).max(0);
        if candidates > MAX_PROBE_CANDIDATES {
            return None;
        }
    }
    let mut matched = None;
    for (multiplier, denominator, lower, upper) in windows {
        let period_numerator = 1_000_000_000 * denominator;
        for n in lower..=upper {
            // 1001*k * 1000/1001 duplicates integer 1000*k, already tested.
            if denominator == 1001 && n % 1001 == 0 {
                continue;
            }
            tested_candidate();
            let numerator = n * multiplier;
            let quantum = tick.checked_mul(numerator)?;
            let (mut low, mut high) = (0, 0);
            let mut fits = true;
            for (i, packet) in packets.clone().enumerate() {
                let elapsed =
                    (i128::from(packet.timestamp) - i128::from(first)).checked_mul(quantum)?;
                let residual = elapsed.checked_sub(i as i128 * period_numerator)?;
                low = low.min(residual);
                high = high.max(residual);
                if high.checked_sub(low)? >= quantum {
                    fits = false;
                    break;
                }
                if let Some(duration) = packet.duration {
                    let residual = i128::from(duration).checked_mul(quantum)? - period_numerator;
                    if residual.checked_abs()? >= quantum {
                        fits = false;
                        break;
                    }
                }
            }
            if fits {
                if matched.is_some() {
                    return None;
                }
                matched = Some(numerator as f64 / denominator as f64);
            }
        }
    }
    matched
}

// Decode keeps its independent interval and duration/count ceiling.
fn video_duration(d: &Demuxer<'_>, t: &media_webm::Track<'_>) -> Result<Option<u64>> {
    if t.default_duration_ns > 0 {
        return Ok(Some(t.default_duration_ns));
    }
    let mut previous = None;
    for p in d.packets().iter().filter(|p| p.track == t.number) {
        if let Some(duration) = p.duration.filter(|n| *n > 0) {
            return Ok(Some(
                duration
                    .checked_mul(d.scale_ns)
                    .ok_or("TIMESTAMP_OVERFLOW")?,
            ));
        }
        if let Some(last) = previous
            && p.timestamp > last
        {
            let delta = p
                .timestamp
                .checked_sub(last)
                .ok_or("TIMESTAMP_OVERFLOW: packet delta")?;
            let delta = delta
                .checked_abs()
                .ok_or("TIMESTAMP_OVERFLOW: packet delta")? as u64;
            return Ok(Some(
                delta.checked_mul(d.scale_ns).ok_or("TIMESTAMP_OVERFLOW")?,
            ));
        }
        previous = Some(p.timestamp);
    }
    Ok(None)
}

fn audio(d: &Demuxer<'_>, output: &mut File, budget: &mut OutputBudget) -> Result<()> {
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
    let spec = hound::WavSpec {
        channels: n as u16,
        sample_rate: 48_000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    // Ask hound for its exact float-WAV header size, including extensions.
    let mut header = Cursor::new(Vec::new());
    hound::WavWriter::new(&mut header, spec)?.finalize()?;
    budget.charge(header.get_ref().len() as u64)?;
    let mut buffered = BufWriter::new(output);
    let mut writer = hound::WavWriter::new(&mut buffered, spec)?;
    for p in d.packets().iter().filter(|p| p.track == t.number) {
        let pcm = decoder.decode(Some(p.data))?.pcm;
        let absolute_padding = p
            .discard_padding_ns
            .checked_abs()
            .ok_or("TIMESTAMP_OVERFLOW: discard padding")?;
        let padding = ((absolute_padding as u128 * 48_000 + 500_000_000) / 1_000_000_000) as usize;
        let len = pcm.len() / n;
        if padding > len {
            return Err("Opus discard padding exceeds packet".into());
        }
        let start_pad = if p.discard_padding_ns < 0 { padding } else { 0 };
        let end_pad = if p.discard_padding_ns > 0 { padding } else { 0 };
        let available = len - start_pad - end_pad;
        let skipped = skip.min(available);
        skip -= skipped;
        let samples = &pcm[(start_pad + skipped) * n..(len - end_pad) * n];
        budget.charge(
            (samples.len() as u64)
                .checked_mul(4)
                .ok_or("OUTPUT_LIMIT_EXCEEDED")?,
        )?;
        for sample in samples {
            writer.write_sample(*sample * gain)?;
        }
    }
    writer.finalize()?;
    buffered.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use media_opus::{Encoder as _, OpusEncoder};
    use std::{io::Cursor, process::Command};

    fn timing_packets(timestamps: &[i64], duration: Option<u64>) -> Vec<media_webm::Packet<'_>> {
        timestamps
            .iter()
            .map(|&timestamp| media_webm::Packet {
                track: 1,
                timestamp,
                key_frame: false,
                data: &[],
                duration,
                discard_padding_ns: 0,
            })
            .collect()
    }

    #[test]
    fn probe_cfr30_quantised_timestamps() {
        // Both truncating and rounding muxers; offset must not affect the rate.
        for rounding in [0, 15] {
            let timestamps: Vec<_> = (0..18_247)
                .map(|i| 700 + (i * 1000 + rounding) / 30)
                .collect();
            for duration in [None, Some(33)] {
                let packets = timing_packets(&timestamps, duration);
                assert_eq!(probe_video_rate(0, 1_000_000, packets.iter()), Some(30.0));
            }
        }
    }

    #[test]
    fn probe_declared_default_interval() {
        let packets = timing_packets(&[0], None);
        assert_eq!(
            probe_video_rate(40_000_000, 1_000_000, packets.iter()),
            Some(25.0)
        );
        assert_eq!(
            probe_video_rate(33_366_667, 1_000_000, packets.iter()),
            Some(1e9 / 33_366_667.0)
        );
    }

    #[test]
    fn probe_insufficient_or_irregular_timing_is_unknown() {
        for timestamps in [
            &[][..],
            &[0],
            &[0, 33],
            &[0, 33, 100, 133, 200],
            // Average is 30 fps, but the interior timestamp breaks the grid.
            &[0, 20, 80, 100],
            // Same 33/34 delta histogram as CFR30, with accumulated drift.
            &[0, 33, 66, 99, 133, 167, 200],
            &[0, 33, 33, 100],
            &[0, 67, 33, 100],
        ] {
            let packets = timing_packets(timestamps, None);
            assert_eq!(
                probe_video_rate(0, 1_000_000, packets.iter()),
                None,
                "{timestamps:?}"
            );
        }
        let packets = timing_packets(&[0, 40, 80, 120], Some(20));
        assert_eq!(probe_video_rate(0, 1_000_000, packets.iter()), None);
    }

    #[test]
    fn probe_exact_fractional_tick_interval() {
        let packets = timing_packets(
            &[0, 33, 66, 99, 132, 165, 198, 231, 264, 297, 330, 363, 396],
            Some(33),
        );
        assert_eq!(
            probe_video_rate(0, 1_000_000, packets.iter()),
            Some(1e9 / 33_000_000.0)
        );
    }

    #[test]
    fn probe_exact_short_interval_precedes_neighbouring_grids() {
        let packets = timing_packets(&[0, 33, 66], Some(33));
        assert_eq!(
            probe_video_rate(0, 1_000_000, packets.iter()),
            Some(1e9 / 33_000_000.0)
        );
    }

    #[test]
    fn probe_ambiguous_neighbours_are_unknown() {
        let packets = timing_packets(&[0, 17, 33], None);
        assert_eq!(probe_video_rate(0, 1_000_000, packets.iter()), None);
        let packets = timing_packets(
            &[0, 33, 66, 100, 133, 166, 200, 233, 266, 300, 333],
            Some(33),
        );
        assert_eq!(probe_video_rate(0, 1_000_000, packets.iter()), None);
        for rounding in [0, 15_000] {
            // Seven frames are ambiguous for either quantisation phase;
            // rounded timestamps can distinguish the grids sooner than floor.
            for (count, expected) in [(7, None), (301, Some(30_000.0 / 1001.0))] {
                let timestamps: Vec<_> = (0..count)
                    .map(|i| (i * 1_001_000 + rounding) / 30_000)
                    .collect();
                let packets = timing_packets(&timestamps, Some(33));
                assert_eq!(probe_video_rate(0, 1_000_000, packets.iter()), expected);
            }
        }
        let timestamps: Vec<_> = (0..301).map(|i| i * 1000 / 30).collect();
        let packets = timing_packets(&timestamps, None);
        assert_eq!(probe_video_rate(0, 1_000_000, packets.iter()), Some(30.0));
    }

    #[test]
    fn probe_symmetric_half_tick_rounding_is_conservatively_unknown() {
        // Ties to even creates both -0.5 and +0.5 tick residuals for 16 Hz.
        let timestamps: Vec<_> = (0..161)
            .map(|i| i / 4 * 250 + [0, 62, 125, 188][(i % 4) as usize])
            .collect();
        let packets = timing_packets(&timestamps, None);
        assert_eq!(probe_video_rate(0, 1_000_000, packets.iter()), None);
    }

    #[test]
    fn probe_candidate_search_has_a_preflight_budget() {
        for (scale, timestamps) in [
            // Formerly hundreds of millions of endpoint-plausible rates.
            (1, [0, 1, 3]),
            // Hundreds of candidates remain even inside the plausible range.
            (1, [0, 1_000_000, 3_000_000]),
            (1_000_000, [0, 1, 3]),
        ] {
            let packets = timing_packets(&timestamps, None);
            let mut tested = 0;
            assert_eq!(
                probe_video_rate_with_work(0, scale, packets.iter(), || tested += 1),
                None
            );
            assert_eq!(
                tested, 0,
                "over-budget/out-of-range input must not scan grids"
            );
        }
        let timestamps: Vec<_> = (0..301).map(|i| i * 1000 / 30).collect();
        let packets = timing_packets(&timestamps, None);
        let mut tested = 0;
        assert_eq!(
            probe_video_rate_with_work(0, 1_000_000, packets.iter(), || tested += 1),
            Some(30.0)
        );
        assert!(tested > 0 && tested <= MAX_PROBE_CANDIDATES);
    }

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
    fn framemd5_dav1d_oracle_and_transactional_outputs() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("changing.webm");
        ff(&["-f", "lavfi", "-i", "testsrc2=size=64x48:rate=30", "-frames:v", "90",
            "-c:v", "libaom-av1", "-cpu-used", "8", input.to_str().unwrap()]);
        let y4m = dir.path().join("out.y4m");
        let hashes = dir.path().join("out.txt");
        let invoke = |flags: &[&str]| {
            run(false, std::iter::once(input.clone().into_os_string())
                .chain(flags.iter().map(OsString::from)))
        };
        assert!(invoke(&[]).unwrap_err().to_string().contains("VIDEO_OUTPUT_REQUIRED"));
        assert!(invoke(&["--y4m", hashes.to_str().unwrap(), "--framemd5", hashes.to_str().unwrap()])
            .unwrap_err().to_string().contains("OUTPUT_PATH_CONFLICT"));
        let h = hashes.to_str().unwrap();
        let y = y4m.to_str().unwrap();
        invoke(&["--y4m", y, "--framemd5", h]).unwrap();
        let text = std::fs::read_to_string(&hashes).unwrap();
        let encoded = std::fs::read(&input).unwrap();
        let demux = Demuxer::new(&encoded).unwrap();
        assert_eq!(text.lines().next(), Some("#av-framemd5 v1 W64 H48 yuv420p"));
        let ours: Vec<_> = text.lines().skip(1).enumerate().map(|(i, l)| {
            let columns: Vec<_> = l.split_whitespace().collect();
            assert_eq!(columns.len(), 4);
            assert_eq!(columns[0], i.to_string());
            assert_eq!(columns[2], "4608");
            assert_eq!(columns[3].parse::<i64>().unwrap(), demux.packets()[i].timestamp * demux.scale_ns as i64);
            columns[1]
        }).collect();
        let oracle = ff(&["-c:v", "libdav1d", "-i", input.to_str().unwrap(),
            "-map", "0:v:0", "-fps_mode", "passthrough", "-f", "framemd5", "-"]);
        let oracle = String::from_utf8(oracle).unwrap();
        let reference: Vec<_> = oracle.lines().filter(|l| !l.starts_with('#') && !l.is_empty())
            .map(|l| l.rsplit(',').next().unwrap().trim()).collect();
        assert_eq!(ours.len(), 90);
        assert!(ours.windows(2).any(|p| p[0] != p[1]));
        assert_eq!(ours, reference);
        let raw = std::fs::read(&y4m).unwrap();
        let mut at = raw.iter().position(|b| *b == b'\n').unwrap() + 1;
        assert!(std::str::from_utf8(&raw[..at]).unwrap().contains(" F30:1 "));
        for digest in &ours {
            at += raw[at..].iter().position(|b| *b == b'\n').unwrap() + 1;
            assert_eq!(*digest, format!("{:x}", Md5::digest(&raw[at..at + 4608])));
            at += 4608;
        }
        assert_eq!(at, raw.len());
        std::fs::remove_file(&y4m).unwrap();
        std::fs::remove_file(&hashes).unwrap();
        invoke(&["--framemd5", h, "--max-output-bytes", "10000"]).unwrap();
        assert_eq!(std::fs::read_to_string(&hashes).unwrap(), text);
        std::fs::remove_file(&hashes).unwrap();
        let wav = dir.path().join("out.wav");
        for flags in [vec!["--y4m", y, "--framemd5", h, "--max-output-bytes", "10000"],
            vec!["--framemd5", h, "--max-output-bytes", "100"],
            vec!["--y4m", y, "--framemd5", h, "--wav", wav.to_str().unwrap()]] {
            assert!(invoke(&flags).is_err());
            assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
        }
    }

    #[test]
    fn y4m_rate_grids_and_vfr_fallback() {
        assert_eq!(header_rate(Some(30.0), Some(33_000_000)), "30:1");
        assert_eq!(header_rate(Some(30000.0 / 1001.0), None), "30000:1001");
        assert_eq!(header_rate(Some(25.0), None), "25:1");
        assert_eq!(header_rate(None, Some(33_000_000)), "1000000000:33000000");
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
                audio(
                    &Demuxer::new(&bytes).unwrap(),
                    &mut File::create(&wav).unwrap(),
                    &mut OutputBudget {
                        limit: DEFAULT_MAX_OUTPUT_BYTES,
                        used: 0,
                    },
                )
                .unwrap();
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
