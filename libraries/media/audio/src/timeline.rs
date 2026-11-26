//! Flat, absolute-time episode audio. Mix owns scene and join policy.
use crate::limiter::{TARGET_TOLERANCE_LU, cpu_seconds, target_matches};
use crate::{Audio, LimiterConfig, Loudness, WavFormat};
use media_opus::{Channels, Decoder, Encoder, OpusDecoder, OpusEncoder, OpusHead, Packet};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufWriter, Write},
    path::{Component, Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Timeline {
    pub version: u32,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_s: f64,
    pub sources: Vec<Source>,
    pub master: Master,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub path: PathBuf,
    pub at_s: f64,
    #[serde(default)]
    pub from_s: f64,
    pub to_s: Option<f64>,
    #[serde(default)]
    pub gain_db: f64,
    #[serde(default)]
    pub fade_in_s: f64,
    #[serde(default)]
    pub fade_out_s: f64,
}
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Master {
    pub integrated_lufs: f64,
    pub true_peak_dbtp: f64,
    /// Explicit per-timeline permission; never enabled by a default.
    #[serde(default)]
    pub accept_ceiling_limited: bool,
}
fn frames(s: f64) -> usize {
    (s * 48000.0).round() as usize
}
fn valid_time(s: f64) -> bool {
    s.is_finite() && (0.0..=10800.0).contains(&s)
}
fn refusal(name: &str) -> Box<dyn std::error::Error> {
    format!("timeline: {name}").into()
}
/// Conservative offline peak: twelve stereo f64 buffers, detector envelope,
/// packet/index storage, and the largest native decode/resampling window.
/// Headers only; no PCM allocation. Also used while video encoding overlaps.
pub fn peak_memory_bytes(duration_s: f64, sources: &[Source], base: &Path) -> Result<u64> {
    let output = frames(duration_s);
    let mut largest = 0;
    for s in sources {
        let source = crate::source::wav(crate::source::open(base, &s.path)?)?;
        let from = frames(s.from_s);
        let to = s.to_s.map(frames).unwrap_or(source.frames);
        if from > to || to > source.frames || frames(s.at_s) > output {
            return Err(refusal("source_memory_range"));
        }
        let placed = (to - from).min(output - frames(s.at_s));
        largest = largest.max(source.decoded_samples(from..from + placed)?);
    }
    (output as u64)
        .checked_mul(224)
        .and_then(|n| largest.checked_mul(24)?.checked_add(n))
        .and_then(|n| n.checked_add(32 * 1024 * 1024))
        .ok_or_else(|| refusal("audio_memory_budget"))
}
impl Timeline {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            return Err(refusal("version"));
        }
        if self.sample_rate != 48000 {
            return Err(refusal("sample_rate"));
        }
        if self.channels != 2 {
            return Err(refusal("channels"));
        }
        if !valid_time(self.duration_s) {
            return Err(refusal("duration_s"));
        }
        if self.sources.len() > 10000 {
            return Err(refusal("sources: more than 10000"));
        }
        if !self.master.integrated_lufs.is_finite()
            || !(-70.0..=0.0).contains(&self.master.integrated_lufs)
        {
            return Err(refusal("master.integrated_lufs"));
        }
        if !self.master.true_peak_dbtp.is_finite()
            || !(-60.0..=0.0).contains(&self.master.true_peak_dbtp)
        {
            return Err(refusal("master.true_peak_dbtp"));
        }
        for (i, s) in self.sources.iter().enumerate() {
            let fail = |field: &str| refusal(&format!("sources[{i}].{field}"));
            for (name, v) in [
                ("at_s", s.at_s),
                ("from_s", s.from_s),
                ("fade_in_s", s.fade_in_s),
                ("fade_out_s", s.fade_out_s),
            ] {
                if !valid_time(v) {
                    return Err(fail(name));
                }
            }
            if s.at_s > self.duration_s {
                return Err(fail("at_s: past duration_s"));
            }
            if s.path.as_os_str().is_empty() {
                return Err(fail("path"));
            }
            if s.path.is_absolute()
                || s.path.components().any(|c| {
                    matches!(
                        c,
                        Component::ParentDir | Component::RootDir | Component::Prefix(_)
                    )
                })
            {
                return Err(fail("path: must be relative without .. components"));
            }
            if !s.gain_db.is_finite() || !(-120.0..=60.0).contains(&s.gain_db) {
                return Err(fail("gain_db"));
            }
            if let Some(to) = s.to_s {
                if !valid_time(to) || to < s.from_s {
                    return Err(fail("to_s"));
                }
                if frames(s.fade_in_s) > frames(to) - frames(s.from_s)
                    || frames(s.fade_out_s) > frames(to) - frames(s.from_s)
                {
                    return Err(fail("fade: longer than source"));
                }
            }
        }
        Ok(())
    }
    /// f32 accumulation with no normalisation. One source is decoded at a time.
    /// The offline L5/L5b API consumes f64; see README for peak memory budgeting.
    pub fn mix(&self, base: &Path) -> Result<Audio> {
        self.validate()?;
        let base = std::fs::canonicalize(if base.as_os_str().is_empty() {
            Path::new(".")
        } else {
            base
        })?;
        let output_frames = frames(self.duration_s);
        let mut placed_frames = 0usize;
        // Sixteen stereo layers at the highest admitted input rate. Count the
        // exact native windows, including sinc support, before decoding.
        let decoded_budget = output_frames as u64 * 2 * 8 * 16;
        let mut decoded_samples = 0u64;
        let mut out = Vec::<f32>::new();
        let n = frames(self.duration_s) * 2;
        out.try_reserve_exact(n)
            .map_err(|_| crate::Error::SizeOverflow)?;
        out.resize(n, 0.0);
        for (index, s) in self.sources.iter().enumerate() {
            let fail = |e| refusal(&format!("sources[{index}].{e}"));
            let file = crate::source::open(&base, &s.path).map_err(|e| fail(e.to_string()))?;
            let source = crate::source::wav(file).map_err(|e| fail(e.to_string()))?;
            let from = frames(s.from_s);
            let to = s.to_s.map(frames).unwrap_or(source.frames);
            if from > to || to > source.frames {
                return Err(refusal(&format!("sources[{index}].trim: outside WAV")));
            }
            let len = to - from;
            if frames(s.fade_in_s) > len || frames(s.fade_out_s) > len {
                return Err(refusal(&format!(
                    "sources[{index}].fade: longer than source"
                )));
            }
            let start = frames(s.at_s);
            let placed = len.min(output_frames - start);
            placed_frames = placed_frames
                .checked_add(placed)
                .ok_or(crate::Error::SizeOverflow)?;
            if placed_frames > output_frames * 16 {
                return Err(refusal(&format!(
                    "sources[{index}].work: summed placed frames exceed 16x output frames"
                )));
            }
            decoded_samples = decoded_samples
                .checked_add(
                    source
                        .decoded_samples(from..from + placed)
                        .map_err(|e| fail(e.to_string()))?,
                )
                .ok_or(crate::Error::SizeOverflow)?;
            if decoded_samples > decoded_budget {
                return Err(refusal(&format!(
                    "sources[{index}].work: decoded input samples exceed 16x output frames x 2 channels x 8 rate budget"
                )));
            }
            let clip = source
                .decode(from..from + placed)
                .map_err(|e| fail(e.to_string()))?;
            let gain = 10.0_f64.powf(s.gain_db / 20.0)
                * if clip.channels() == 1 {
                    std::f64::consts::FRAC_1_SQRT_2
                } else {
                    1.0
                };
            for k in 0..placed {
                let fade = ramp(k, frames(s.fade_in_s)) * ramp(len - 1 - k, frames(s.fade_out_s));
                for c in 0..2 {
                    let src =
                        k * clip.channels() as usize + if clip.channels() == 1 { 0 } else { c };
                    let dst = &mut out[(start + k) * 2 + c];
                    *dst += (clip.samples()[src] * gain * fade) as f32;
                    if !dst.is_finite() {
                        return Err(refusal(&format!("sources[{index}].mix: non-finite sum")));
                    }
                }
            }
        }
        let mut programme = Vec::new();
        programme
            .try_reserve_exact(out.len())
            .map_err(|_| crate::Error::SizeOverflow)?;
        programme.extend(out.into_iter().map(f64::from));
        Ok(Audio::new(48000, 2, programme)?)
    }
}
/// N-frame inclusive linear ramp: first 0, last 1. A one-frame fade is 0.
fn ramp(k: usize, n: usize) -> f64 {
    if n == 0 || k >= n {
        1.0
    } else if n == 1 {
        0.0
    } else {
        k as f64 / (n - 1) as f64
    }
}
#[derive(Serialize)]
pub struct Meter {
    pub integrated_lufs: f64,
    pub true_peak_dbtp: f64,
    lra_lu: f64,
}
impl From<Loudness> for Meter {
    fn from(m: Loudness) -> Self {
        Self {
            integrated_lufs: m.integrated_lufs,
            true_peak_dbtp: m.true_peak_dbtp,
            lra_lu: m.range_lu,
        }
    }
}
#[derive(Serialize)]
pub struct MasteringCost {
    pub limiter_passes: u8,
    pub limiter_cpu_s: Vec<f64>,
    pub mastering_cpu_s: f64,
    pub codec_cpu_s: f64,
    pub mastering_wall_s: f64,
    pub codec_wall_s: f64,
}

/// At most six fresh masters and six encode/decode/meter attempts.
pub const MAX_CODEC_ATTEMPTS: u8 = 6;

#[derive(Serialize)]
pub struct Receipt {
    pub before: Meter,
    pub after: Meter,
    pub gain_db: f64,
    pub max_gain_reduction_db: f64,
    pub shortfall_lu: f64,
    pub target_reached: bool,
    pub refused: bool,
    pub passes: u8,
    pub sample_count: usize,
    pub packets: usize,
    pub bytes: u64,
    pub seconds: f64,
    pub encoded: Meter,
    pub codec_trim_db: f64,
    /// Number of corrections (attempts minus one).
    pub codec_passes: u8,
    pub codec_attempts: u8,
    pub mastering_cost: Vec<MasteringCost>,
}
pub struct Encoded {
    pub audio: Audio,
    pub head: OpusHead,
    pub packets: Vec<Packet>,
    pub receipt: Receipt,
}
/// Mix once, then bounded deterministic mastering from the original mix.
pub fn prepare(t: &Timeline, base: &Path) -> Result<Encoded> {
    prepare_with_encoder(t, base, encode_checked)
}

// Private injection point for deterministic codec-budget refusal coverage.
// Production always supplies the real encode/decode/meter implementation.
fn prepare_with_encoder(
    t: &Timeline,
    base: &Path,
    mut encode: impl FnMut(&Audio) -> Result<(OpusHead, Vec<Packet>, Loudness)>,
) -> Result<Encoded> {
    let original = t.mix(base)?;
    let mut pcm_target = t.master.integrated_lufs;
    let mut codec_trim_db = 0.0;
    let mut codec_passes = 0;
    let mut mastering_cost = Vec::new();
    let mut total_passes = 0;
    let mut max_gain_reduction_db: f64 = 0.0;
    let (audio, r, head, packets, encoded, reached) = loop {
        let mut audio = original.try_clone()?;
        let wall = std::time::Instant::now();
        let start = cpu_seconds();
        let r = audio.normalise_to_target(
            pcm_target,
            t.master.true_peak_dbtp + codec_trim_db,
            LimiterConfig::default(),
        )?;
        let mastering_cpu_s = cpu_seconds() - start;
        let mastering_wall_s = wall.elapsed().as_secs_f64();
        total_passes += r.passes;
        max_gain_reduction_db = max_gain_reduction_db.max(r.max_gain_reduction_db);
        let start = cpu_seconds();
        let wall = std::time::Instant::now();
        let (head, packets, encoded) = encode(&audio)?;
        mastering_cost.push(MasteringCost {
            limiter_passes: r.passes,
            limiter_cpu_s: r.pass_cpu_s[..usize::from(r.passes)].to_vec(),
            mastering_cpu_s,
            codec_cpu_s: cpu_seconds() - start,
            mastering_wall_s,
            codec_wall_s: wall.elapsed().as_secs_f64(),
        });
        let reached = !r.refused
            && target_matches(
                t.master.integrated_lufs,
                encoded.integrated_lufs,
                TARGET_TOLERANCE_LU,
            )
            && encoded.true_peak_dbtp <= t.master.true_peak_dbtp;
        if reached || r.refused || codec_passes + 1 == MAX_CODEC_ATTEMPTS {
            break (audio, r, head, packets, encoded, reached);
        }
        // Lower the PCM ceiling for reconstruction overshoot and compensate
        // measured codec loudness loss. Re-limit the ORIGINAL, never an already
        // limited signal. No random state, adaptive stopping based on time, or AGC.
        codec_trim_db += (t.master.true_peak_dbtp - encoded.true_peak_dbtp - 0.02).min(0.0);
        pcm_target += t.master.integrated_lufs - encoded.integrated_lufs;
        codec_passes += 1;
    };
    let after = audio.measure()?;
    let count = audio.frames();
    let receipt = Receipt {
        before: r.before.into(),
        after: after.into(),
        gain_db: r.gain_db,
        max_gain_reduction_db,
        shortfall_lu: (t.master.integrated_lufs - encoded.integrated_lufs).max(0.0),
        target_reached: reached,
        // Only a PCM ceiling/reduction-limited shortfall qualifies. A codec
        // correction budget miss alone does not grant this exception.
        refused: !reached
            && !(t.master.accept_ceiling_limited
                && r.refused
                && r.shortfall_lu > TARGET_TOLERANCE_LU
                && encoded.integrated_lufs.is_finite()
                && encoded.integrated_lufs < t.master.integrated_lufs - TARGET_TOLERANCE_LU
                && encoded.true_peak_dbtp <= t.master.true_peak_dbtp),
        passes: total_passes,
        sample_count: count,
        packets: packets.len(),
        bytes: packets.iter().map(|p| p.data.len() as u64).sum(),
        seconds: count as f64 / 48000.0,
        encoded: encoded.into(),
        codec_trim_db,
        codec_passes,
        codec_attempts: codec_passes + 1,
        mastering_cost,
    };
    Ok(Encoded {
        audio,
        head,
        packets,
        receipt,
    })
}

pub fn render(t: &Timeline, base: &Path, out: &Path, wav: Option<&Path>) -> Result<Receipt> {
    render_prepared(prepare(t, base)?, out, wav)
}

pub(crate) fn render_prepared(
    prepared: Encoded,
    out: &Path,
    wav: Option<&Path>,
) -> Result<Receipt> {
    let Encoded {
        audio,
        head,
        packets,
        mut receipt,
    } = prepared;
    // Declare the primary first so rollback drops the optional WAV before
    // removing the WebM, preserving the publication ordering during cleanup.
    let mut output = crate::pending::Pending::new(out)?;
    let mut wav_output = wav.map(crate::pending::Pending::new).transpose()?;
    if let Some(output) = &mut wav_output {
        let mut buffer = BufWriter::new(&mut output.file);
        audio.write_wav(&mut buffer, WavFormat::Float32)?;
        buffer.flush()?;
    }
    let mut writer = media_webm::Writer::audio(
        BufWriter::new(&mut output.file),
        media_webm::Opus {
            channels: 2,
            pre_skip: head.pre_skip,
            input_sample_rate: 48000,
        },
    )?;
    let count = audio.frames();
    for (k, p) in packets.iter().enumerate() {
        writer.opus_packet(k as u64 * 20, &p.data, p.discard_padding)?;
    }
    writer.set_duration_samples(count as u64);
    writer.finish()?.flush()?;
    output.file.sync_all()?;
    if let Some(wav) = &wav_output {
        wav.file.sync_all()?;
    }
    receipt.bytes = output.file.metadata()?.len();
    if !receipt.refused {
        output.publish()?;
        if let Some(wav) = &mut wav_output {
            wav.publish()?;
        }
        if let Some(wav) = &mut wav_output {
            wav.commit();
        }
        output.commit();
    }
    Ok(receipt)
}

/// Decode only packets produced by our own encoder, never untrusted input.
fn encode_checked(audio: &Audio) -> Result<(OpusHead, Vec<Packet>, Loudness)> {
    let mut enc = OpusEncoder::new(Channels::Stereo, 128000)?;
    let head = enc.head().clone();
    let count = audio.frames();
    let blocks = count.div_ceil(960);
    let padding = blocks * 960 - count;
    // Flush delay plus final partial-frame padding may exceed one frame. Encode
    // only ceil((length+lookahead)/960) packets and put all excess on the last.
    let needed = (count + usize::from(enc.head().pre_skip)).div_ceil(960);
    let mut packets = Vec::new();
    packets
        .try_reserve_exact(needed)
        .map_err(|_| crate::Error::SizeOverflow)?;
    for k in 0..blocks {
        let start = k * 1920;
        let mut pcm = [0f32; 1920];
        for (d, s) in pcm
            .iter_mut()
            .zip(&audio.samples()[start..audio.samples().len().min(start + 1920)])
        {
            *d = *s as f32;
        }
        let mut p = enc.encode(&pcm)?;
        let discard = if k + 1 == needed {
            padding - usize::from(enc.head().pre_skip)
        } else {
            0
        };
        p.discard_padding = discard as u16;
        packets.push(p);
    }
    if needed > blocks
        && let Some(mut p) = enc.finish()?
    {
        p.discard_padding += padding as u16;
        packets.push(p);
    }
    let mut decoder = OpusDecoder::new(Channels::Stereo)?;
    let mut decoded = Vec::new();
    decoded
        .try_reserve_exact((count + 1920) * 2)
        .map_err(|_| crate::Error::SizeOverflow)?;
    let mut skip = usize::from(head.pre_skip) * 2;
    for p in &packets {
        let frame = decoder.decode(Some(&p.data))?;
        let start = skip.min(frame.pcm.len());
        skip -= start;
        let end = frame.pcm.len() - usize::from(p.discard_padding) * 2;
        if end < start || decoded.len() + end - start > count * 2 {
            return Err(refusal("Opus: decoded sample count"));
        }
        decoded.extend(frame.pcm[start..end].iter().copied().map(f64::from));
    }
    if decoded.len() != count * 2 {
        return Err(refusal("Opus: decoded sample count"));
    }
    let measured = Audio::new(48000, 2, decoded)?.measure()?;
    Ok((head, packets, measured))
}

#[cfg(test)]
mod refusal_tests {
    use super::*;

    #[test]
    fn persistent_codec_error_exhausts_budget_and_rolls_back_both_outputs() {
        let dir = std::env::temp_dir().join(format!("codec-exhaustion-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("tone.wav");
        Audio::new(
            48000,
            2,
            (0..48000)
                .flat_map(|n| {
                    let s = 0.05 * (std::f64::consts::TAU * 1000.0 * n as f64 / 48000.0).sin();
                    [s, s]
                })
                .collect(),
        )
        .unwrap()
        .write_wav(std::fs::File::create(&source).unwrap(), WavFormat::Float32)
        .unwrap();
        let t: Timeline = serde_json::from_value(serde_json::json!({
            "version":1,"sample_rate":48000,"channels":2,"duration_s":1,
            "sources":[{"path":"tone.wav","at_s":0}],
            "master":{"integrated_lufs":-14,"true_peak_dbtp":-1}
        }))
        .unwrap();
        let mut attempts = 0;
        let prepared = prepare_with_encoder(&t, &dir, |audio| {
            attempts += 1;
            let (head, packets, mut measured) = encode_checked(audio)?;
            // Model a codec that keeps missing both limits despite corrections.
            // Real packets exercise staging, but these injected measurements
            // deliberately prevent convergence without relying on codec quirks.
            measured.integrated_lufs = -14.3;
            measured.true_peak_dbtp = -0.8;
            Ok((head, packets, measured))
        })
        .unwrap();
        assert_eq!(attempts, MAX_CODEC_ATTEMPTS);
        let out = dir.join("refused.webm");
        let wav = dir.join("refused.wav");
        let receipt = render_prepared(prepared, &out, Some(&wav)).unwrap();
        assert!(receipt.refused && !receipt.target_reached);
        assert_eq!(receipt.codec_passes, MAX_CODEC_ATTEMPTS - 1);
        assert_eq!(
            receipt.mastering_cost.len(),
            usize::from(MAX_CODEC_ATTEMPTS)
        );
        assert_eq!(receipt.passes, 0, "fixture stays on the static PCM path");
        assert_eq!(receipt.encoded.integrated_lufs, -14.3);
        assert_eq!(receipt.encoded.true_peak_dbtp, -0.8);
        assert!((receipt.shortfall_lu - 0.3).abs() < 1e-12);
        assert!(!out.exists() && !wav.exists());
        let remaining: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(
            remaining,
            vec![source],
            "no partial output or staging files"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
