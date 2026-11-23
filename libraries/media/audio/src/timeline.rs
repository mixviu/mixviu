//! Flat, absolute-time episode audio. Mix owns scene and join policy.
use crate::{Audio, LimiterConfig, Loudness, WavFormat};
use media_opus::{Channels, Decoder, Encoder, OpusDecoder, OpusEncoder, OpusHead, Packet};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
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
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Master {
    pub integrated_lufs: f64,
    pub true_peak_dbtp: f64,
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
        let mut out = Vec::<f32>::new();
        let n = frames(self.duration_s) * 2;
        out.try_reserve_exact(n)
            .map_err(|_| crate::Error::SizeOverflow)?;
        out.resize(n, 0.0);
        for (index, s) in self.sources.iter().enumerate() {
            let clip =
                Audio::read_wav(BufReader::new(File::open(base.join(&s.path))?))?.resample_48k()?;
            let from = frames(s.from_s);
            let to = s.to_s.map(frames).unwrap_or(clip.frames());
            if from > to || to > clip.frames() {
                return Err(refusal(&format!("sources[{index}].trim: outside WAV")));
            }
            let len = to - from;
            if frames(s.fade_in_s) > len || frames(s.fade_out_s) > len {
                return Err(refusal(&format!(
                    "sources[{index}].fade: longer than source"
                )));
            }
            let start = frames(s.at_s);
            let gain = 10.0_f64.powf(s.gain_db / 20.0)
                * if clip.channels() == 1 {
                    std::f64::consts::FRAC_1_SQRT_2
                } else {
                    1.0
                };
            for k in 0..len.min(frames(self.duration_s) - start) {
                let fade = ramp(k, frames(s.fade_in_s)) * ramp(len - 1 - k, frames(s.fade_out_s));
                for c in 0..2 {
                    let src = (from + k) * clip.channels() as usize
                        + if clip.channels() == 1 { 0 } else { c };
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
    true_peak_dbtp: f64,
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
    pub codec_passes: u8,
}
pub struct Encoded {
    pub audio: Audio,
    pub head: OpusHead,
    pub packets: Vec<Packet>,
    pub receipt: Receipt,
}
pub fn prepare(t: &Timeline, base: &Path) -> Result<Encoded> {
    let mut audio = t.mix(base)?;
    let r = audio.normalise_to_target(
        t.master.integrated_lufs,
        t.master.true_peak_dbtp,
        LimiterConfig::default(),
    )?;
    let mut codec_trim_db = 0.0;
    let mut codec_passes = 0;
    let (head, packets, encoded) = loop {
        let (head, packets, encoded) = encode_checked(&audio)?;
        if encoded.true_peak_dbtp <= t.master.true_peak_dbtp {
            break (head, packets, encoded);
        }
        if codec_passes == 2 {
            return Err(refusal(
                "Opus: peak ceiling unreachable after two corrections",
            ));
        }
        // Opus reconstruction can overshoot even ceiling-compliant PCM.
        // Only attenuate, with a 0.02 dB guard; every attempt is remeasured.
        let trim = t.master.true_peak_dbtp - encoded.true_peak_dbtp - 0.02;
        let gain = 10.0_f64.powf(trim / 20.0);
        for sample in &mut audio.samples {
            *sample *= gain;
        }
        codec_trim_db += trim;
        codec_passes += 1;
    };
    let after = audio.measure()?;
    let count = audio.frames();
    let receipt = Receipt {
        before: r.before.into(),
        after: after.into(),
        gain_db: r.gain_db + codec_trim_db,
        max_gain_reduction_db: r.max_gain_reduction_db,
        shortfall_lu: (t.master.integrated_lufs - encoded.integrated_lufs).max(0.0),
        target_reached: (t.master.integrated_lufs - encoded.integrated_lufs).abs() <= 0.5,
        refused: r.refused,
        passes: r.passes,
        sample_count: count,
        packets: packets.len(),
        bytes: packets.iter().map(|p| p.data.len() as u64).sum(),
        seconds: count as f64 / 48000.0,
        encoded: encoded.into(),
        codec_trim_db,
        codec_passes,
    };
    Ok(Encoded {
        audio,
        head,
        packets,
        receipt,
    })
}

pub fn render(t: &Timeline, base: &Path, out: &Path, wav: Option<&Path>) -> Result<Receipt> {
    let Encoded {
        audio,
        head,
        packets,
        mut receipt,
    } = prepare(t, base)?;
    let mut wav_output = wav.map(crate::pending::Pending::new).transpose()?;
    if let Some(output) = &mut wav_output {
        audio.write_wav(BufWriter::new(&mut output.file), WavFormat::Float32)?;
    }
    let mut output = crate::pending::Pending::new(out)?;
    let mut writer = media_webm::Writer::audio(
        BufWriter::new(&mut output.file),
        media_webm::Opus {
            channels: 2,
            pre_skip: head.pre_skip,
            input_sample_rate: 48000,
        },
    )?;
    for (k, p) in packets.iter().enumerate() {
        writer.opus_packet(k as u64 * 20, &p.data, p.discard_padding)?;
    }
    writer.set_duration_samples(audio.frames() as u64);
    drop(writer.finish()?);
    receipt.bytes = output.file.metadata()?.len();
    if !receipt.refused && receipt.target_reached {
        if let Some(wav) = &mut wav_output {
            wav.publish()?;
        }
        output.publish()?;
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
