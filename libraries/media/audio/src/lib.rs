//! In-memory narration primitives. Offsets and lengths are frames, never scalar samples.
//! Finite f64 interleaved samples preserve PCM32 precision and unclipped mix headroom.
use rubato::{
    WindowFunction,
    interpolation::{get_nearest_times_4, interp_cubic},
    sinc_interpolator::{SincInterpolator, make_interpolator},
};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
pub mod cli;
mod limiter;
pub mod pending;
pub mod progress;
mod source;
pub mod timeline;
pub use limiter::{LimitReport, LimiterConfig, TargetReport};

/// Standard RIFF maximum, enough for over three hours of stereo float32 at 48 kHz.
pub const MAX_WAV_BYTES: u64 = u32::MAX as u64 + 8;
/// Three hours at 48 kHz: 50% headroom over a two-hour narration.
pub const MAX_RESAMPLED_FRAMES: usize = 48000 * 60 * 60 * 3;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Wav(hound::Error),
    InvalidHeader,
    UnsupportedFormat,
    InvalidAudio,
    InputSampleRate { rate: u32 },
    SizeOverflow,
    FormatMismatch,
    Measurement(ebur128::Error),
    InputTooLarge { max_bytes: u64 },
    OutputTooLong { max_frames: usize },
    InterpolationBounds,
    InvalidWritePosition,
    Unmeasurable,
    ReductionLimit { required_db: f64 },
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "audio I/O: {e}"),
            Self::Wav(e) => write!(f, "WAV decoding or encoding: {e}"),
            Self::InvalidHeader => f.write_str("invalid or truncated RIFF/WAVE header"),
            Self::UnsupportedFormat => {
                f.write_str("unsupported WAV encoding, channels or bit depth")
            }
            Self::InvalidAudio => f.write_str("invalid audio rate, channels or sample value"),
            Self::InputSampleRate { rate } => {
                write!(f, "WAV sample_rate {rate} outside 8000..384000 Hz")
            }
            Self::SizeOverflow => f.write_str("audio buffer size overflows or cannot be allocated"),
            Self::FormatMismatch => {
                f.write_str("clip rate or channel count differs from destination")
            }
            Self::Measurement(e) => write!(f, "loudness measurement: {e}"),
            Self::InputTooLarge { max_bytes } => {
                write!(f, "WAV exceeds {max_bytes} byte input limit")
            }
            Self::OutputTooLong { max_frames } => {
                write!(f, "resampled audio exceeds {max_frames} frame limit")
            }
            Self::InterpolationBounds => {
                f.write_str("rubato interpolation index or phase outside its contract")
            }
            Self::InvalidWritePosition => {
                f.write_str("WAV output must be an empty stream positioned at zero")
            }
            Self::Unmeasurable => f.write_str("audio has no measurable integrated loudness"),
            Self::ReductionLimit { required_db } => write!(
                f,
                "limiter needs {required_db:.3} dB reduction, exceeding 6 dB"
            ),
        }
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<hound::Error> for Error {
    fn from(e: hound::Error) -> Self {
        match e {
            hound::Error::Unsupported => Self::UnsupportedFormat,
            e => Self::Wav(e),
        }
    }
}
impl From<ebur128::Error> for Error {
    fn from(e: ebur128::Error) -> Self {
        Self::Measurement(e)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Audio {
    rate: u32,
    channels: u16,
    samples: Vec<f64>,
}
#[derive(Clone, Copy, Debug)]
pub enum WavFormat {
    Pcm16,
    Pcm24,
    Pcm32,
    Float32,
}
#[derive(Clone, Copy, Debug)]
pub struct Loudness {
    /// Negative infinity for silence or fewer than 400 ms of gated audio.
    pub integrated_lufs: f64,
    pub range_lu: f64,
    pub sample_peak_dbfs: f64,
    pub true_peak_dbtp: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct GainReport {
    pub before: Loudness,
    pub after: Loudness,
    pub gain_db: f64,
    pub shortfall_lu: f64,
    pub peak_limited: bool,
}
impl Audio {
    pub fn new(rate: u32, channels: u16, samples: Vec<f64>) -> Result<Self, Error> {
        if rate == 0
            || !(1..=2).contains(&channels)
            || !samples.len().is_multiple_of(channels as usize)
            || samples.iter().any(|x| !x.is_finite())
        {
            return Err(Error::InvalidAudio);
        }
        Ok(Self {
            rate,
            channels,
            samples,
        })
    }
    /// Fallible copy for whole-programme offline operations.
    pub fn try_clone(&self) -> Result<Self, Error> {
        let mut samples = Vec::new();
        samples
            .try_reserve_exact(self.samples.len())
            .map_err(|_| Error::SizeOverflow)?;
        samples.extend_from_slice(&self.samples);
        Ok(Self {
            rate: self.rate,
            channels: self.channels,
            samples,
        })
    }
    pub fn rate(&self) -> u32 {
        self.rate
    }
    pub fn channels(&self) -> u16 {
        self.channels
    }
    pub fn samples(&self) -> &[f64] {
        &self.samples
    }
    pub fn frames(&self) -> usize {
        self.samples.len() / self.channels as usize
    }
    pub fn silence(rate: u32, channels: u16, frames: usize) -> Result<Self, Error> {
        if rate == 0 || !(1..=2).contains(&channels) {
            return Err(Error::InvalidAudio);
        }
        let n = frames
            .checked_mul(channels as usize)
            .ok_or(Error::SizeOverflow)?;
        let mut samples = Vec::new();
        samples
            .try_reserve_exact(n)
            .map_err(|_| Error::SizeOverflow)?;
        samples.resize(n, 0.0);
        Self::new(rate, channels, samples)
    }
    pub fn map_channels(&self, channels: u16) -> Result<Self, Error> {
        let mut samples = Vec::new();
        if !(1..=2).contains(&channels) {
            return Err(Error::InvalidAudio);
        }
        samples
            .try_reserve_exact(
                self.frames()
                    .checked_mul(channels as usize)
                    .ok_or(Error::SizeOverflow)?,
            )
            .map_err(|_| Error::SizeOverflow)?;
        match (self.channels, channels) {
            (a, b) if a == b => samples.extend_from_slice(&self.samples),
            (1, 2) => samples.extend(self.samples.iter().flat_map(|&s| [s, s])),
            (2, 1) => samples.extend(
                self.samples
                    .chunks_exact(2)
                    .map(|s| s[0] * 0.5 + s[1] * 0.5),
            ),
            _ => return Err(Error::InvalidAudio),
        }
        Self::new(self.rate, channels, samples)
    }
    /// Pad with silence or trim the end. Trimming a range belongs to the timeline caller.
    pub fn resize(&mut self, frames: usize) -> Result<(), Error> {
        let n = frames
            .checked_mul(self.channels as usize)
            .ok_or(Error::SizeOverflow)?;
        self.samples
            .try_reserve(n.saturating_sub(self.samples.len()))
            .map_err(|_| Error::SizeOverflow)?;
        self.samples.resize(n, 0.0);
        Ok(())
    }
    /// Unnormalised sum at a frame offset, extending the destination if necessary.
    /// Rejects non-finite sums before modifying any samples.
    pub fn place(&mut self, clip: &Self, offset: usize) -> Result<(), Error> {
        if (self.rate, self.channels) != (clip.rate, clip.channels) {
            return Err(Error::FormatMismatch);
        }
        if clip.frames() == 0 {
            return Ok(());
        }
        let end = offset
            .checked_add(clip.frames())
            .ok_or(Error::SizeOverflow)?;
        end.checked_mul(self.channels as usize)
            .ok_or(Error::SizeOverflow)?;
        let start = offset
            .checked_mul(self.channels as usize)
            .ok_or(Error::SizeOverflow)?;
        for (i, x) in clip.samples.iter().enumerate() {
            if !(self.samples.get(start + i).copied().unwrap_or(0.0) + x).is_finite() {
                return Err(Error::InvalidAudio);
            }
        }
        if end > self.frames() {
            self.resize(end)?;
        }
        for (d, s) in self.samples[start..].iter_mut().zip(&clip.samples) {
            *d += s;
        }
        Ok(())
    }
    /// 512-tap squared Blackman-Harris sinc, automatic anti-alias cutoff,
    /// 256 table phases, cubic interpolation, f64. Positional interpolation
    /// samples exactly n * input_rate / 48000; zero padding handles both edges.
    pub fn resample_48k(&self) -> Result<Self, Error> {
        let frames = usize::try_from((self.frames() as u128 * 48000).div_ceil(self.rate as u128))
            .map_err(|_| Error::SizeOverflow)?;
        if frames > MAX_RESAMPLED_FRAMES {
            return Err(Error::OutputTooLong {
                max_frames: MAX_RESAMPLED_FRAMES,
            });
        }
        if self.rate == 48000 {
            return self.try_clone();
        }
        if self.frames() == 0 {
            return Self::silence(48000, self.channels, 0);
        }
        self.resample_window(0, 0..frames)
    }
    // Input is a seeked window with at least 512 frames of filter support on
    // either side (or the actual file boundary). Keep the original time origin.
    fn resample_window(&self, origin: usize, range: std::ops::Range<usize>) -> Result<Self, Error> {
        let frames = range.len();
        let ratio = 48000.0 / self.rate as f64;
        let sinc = make_interpolator::<f64>(512, ratio, None, 256, WindowFunction::BlackmanHarris2);
        let mut out = Self::silence(48000, self.channels, frames)?;
        let pad = sinc.nbr_points();
        let len = self
            .frames()
            .checked_add(2 * pad)
            .ok_or(Error::SizeOverflow)?;
        let mut wave = Vec::new();
        wave.try_reserve_exact(len)
            .map_err(|_| Error::SizeOverflow)?;
        wave.resize(len, 0.0);
        for c in 0..self.channels as usize {
            for (n, frame) in self
                .samples
                .chunks_exact(self.channels as usize)
                .enumerate()
            {
                wave[n + pad] = frame[c];
            }
            for n in 0..frames {
                let pos = (range.start + n) as f64 * self.rate as f64 / 48000.0 - origin as f64
                    + pad as f64;
                let t = pos - (pad / 2) as f64 + 1.0 - 1.0 / 256.0;
                let mut nearest = [(0isize, 0isize); 4];
                get_nearest_times_4(t, 256, &mut nearest);
                let mut points = [0.0; 4];
                for (p, (i, s)) in points.iter_mut().zip(nearest) {
                    // rubato 5.1.0 explicitly asserts index + nbr_points < len,
                    // even though the mathematical dot-product slice could end at len.
                    if i < 0
                        || (i as usize)
                            .checked_add(pad)
                            .is_none_or(|end| end >= wave.len())
                        || !(0..256).contains(&s)
                    {
                        return Err(Error::InterpolationBounds);
                    }
                    *p = sinc.get_sinc_interpolated(&wave, i as usize, s as usize);
                }
                out.samples[n * self.channels as usize + c] =
                    interp_cubic((t * 256.0).fract(), &points);
            }
        }
        if out.samples.iter().any(|x| !x.is_finite()) {
            return Err(Error::InvalidAudio);
        }
        Ok(out)
    }
    /// BS.1770 integrated gating and Tech 3342 LRA. True peak uses ebur128's
    /// polyphase FIR (4x below 96 kHz, 2x below 192 kHz). Measure at 48 kHz
    /// after resampling when comparing the narration output chain.
    pub fn measure(&self) -> Result<Loudness, Error> {
        use ebur128::{EbuR128, Mode};
        let mut m = EbuR128::new(
            self.channels as u32,
            self.rate,
            Mode::I | Mode::LRA | Mode::TRUE_PEAK,
        )?;
        m.add_frames_f64(&self.samples)?;
        let integrated_lufs = m.loudness_global()?;
        let range_lu = m.loudness_range()?;
        let mut sample_peak: f64 = 0.0;
        let mut true_peak: f64 = 0.0;
        // Flush FIR history separately: silence must not add loudness gating blocks.
        for c in 0..self.channels as u32 {
            sample_peak = sample_peak.max(m.sample_peak(c)?);
            true_peak = true_peak.max(m.true_peak(c)?);
        }
        m.add_frames_f64(&[0.0; 128][..64 * self.channels as usize])?;
        for c in 0..self.channels as u32 {
            true_peak = true_peak.max(m.true_peak(c)?);
        }
        Ok(Loudness {
            integrated_lufs,
            range_lu,
            sample_peak_dbfs: 20.0 * sample_peak.log10(),
            true_peak_dbtp: 20.0 * true_peak.log10(),
        })
    }
    /// Two-pass static gain, capped by measured true peak. No dynamics or clipping.
    /// Absolute gating can change after gain: `after` is remeasured, never inferred.
    pub fn normalise(&mut self, target_lufs: f64, ceiling_dbtp: f64) -> Result<GainReport, Error> {
        if !target_lufs.is_finite() || !ceiling_dbtp.is_finite() || ceiling_dbtp > 0.0 {
            return Err(Error::InvalidAudio);
        }
        let before = self.measure()?;
        if !before.integrated_lufs.is_finite() {
            return Err(Error::Unmeasurable);
        }
        let requested = target_lufs - before.integrated_lufs;
        let gain_db = requested.min(ceiling_dbtp - before.true_peak_dbtp);
        let gain = 10.0_f64.powf(gain_db / 20.0);
        if !gain.is_finite() || self.samples.iter().any(|x| !(x * gain).is_finite()) {
            return Err(Error::InvalidAudio);
        }
        for x in &mut self.samples {
            *x *= gain;
        }
        let after = self.measure()?;
        Ok(GainReport {
            before,
            after,
            gain_db,
            shortfall_lu: (target_lufs - after.integrated_lufs).max(0.0),
            peak_limited: gain_db < requested,
        })
    }
    pub fn read_wav(r: impl Read) -> Result<Self, Error> {
        Self::read_wav_limited(r, MAX_WAV_BYTES)
    }
    /// Reads at most min(max_bytes, MAX_WAV_BYTES) + 1 bytes (one sentinel).
    /// The default cap fits a two-hour stereo 48 kHz float32 narration with room.
    pub fn read_wav_limited(r: impl Read, max_bytes: u64) -> Result<Self, Error> {
        let max_bytes = max_bytes.min(MAX_WAV_BYTES);
        let mut bytes = Vec::new();
        r.take(max_bytes + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > max_bytes {
            return Err(Error::InputTooLarge { max_bytes });
        }
        validate_riff(&bytes)?;
        let mut reader = hound::WavReader::new(Cursor::new(bytes))?;
        let spec = reader.spec();
        let mut samples = Vec::new();
        samples
            .try_reserve_exact(reader.duration() as usize * spec.channels as usize)
            .map_err(|_| Error::SizeOverflow)?;
        match (spec.sample_format, spec.bits_per_sample) {
            (hound::SampleFormat::Float, 32) => {
                for sample in reader.samples::<f32>() {
                    samples.push(f64::from(sample?));
                }
            }
            (hound::SampleFormat::Int, b @ (16 | 24 | 32)) => {
                let scale = 2.0_f64.powi(b as i32 - 1);
                for sample in reader.samples::<i32>() {
                    samples.push(sample? as f64 / scale);
                }
            }
            _ => return Err(Error::UnsupportedFormat),
        };
        Self::new(spec.sample_rate, spec.channels, samples)
    }
    /// Integer output is rounded; out-of-range samples are refused, never clipped.
    /// Requires an empty stream at offset zero: hound also seeks relative to zero.
    pub fn write_wav(&self, mut w: impl Write + Seek, format: WavFormat) -> Result<(), Error> {
        let origin = w.stream_position()?;
        let end = w.seek(SeekFrom::End(0))?;
        w.seek(SeekFrom::Start(origin))?;
        if origin != 0 || end != 0 {
            return Err(Error::InvalidWritePosition);
        }
        let (sample_format, bits) = match format {
            WavFormat::Pcm16 => (hound::SampleFormat::Int, 16),
            WavFormat::Pcm24 => (hound::SampleFormat::Int, 24),
            WavFormat::Pcm32 => (hound::SampleFormat::Int, 32),
            WavFormat::Float32 => (hound::SampleFormat::Float, 32),
        };
        let scale = 2.0_f64.powi(bits - 1);
        let block_align = self.channels as u32 * (bits as u32 / 8);
        self.rate
            .checked_mul(block_align)
            .ok_or(Error::SizeOverflow)?;
        let data_bytes = self.frames() as u64 * block_align as u64;
        if data_bytes > u32::MAX as u64 - 128 {
            return Err(Error::SizeOverflow);
        }
        if self.samples.iter().any(|&s| {
            if sample_format == hound::SampleFormat::Float {
                !(s as f32).is_finite()
            } else {
                s < -1.0 || s > 1.0 - 1.0 / scale
            }
        }) {
            return Err(Error::InvalidAudio);
        }
        let mut writer = hound::WavWriter::new(
            &mut w,
            hound::WavSpec {
                channels: self.channels,
                sample_rate: self.rate,
                bits_per_sample: bits as u16,
                sample_format,
            },
        )?;
        for &s in &self.samples {
            if sample_format == hound::SampleFormat::Float {
                writer.write_sample(s as f32)?;
            } else {
                writer.write_sample((s * scale).round() as i32)?;
            }
        }
        writer.finalize()?;
        // hound does not append RIFF's word-alignment byte for odd PCM24 data.
        if !data_bytes.is_multiple_of(2) {
            w.seek(SeekFrom::End(0))?;
            w.write_all(&[0])?;
            let end = w.stream_position()?;
            w.seek(SeekFrom::Start(4))?;
            w.write_all(&((end - 8) as u32).to_le_bytes())?;
            w.seek(SeekFrom::Start(end))?;
        }
        Ok(())
    }
}

// Preflight all RIFF chunk boundaries before hound sees them, including divisors
// used in its header parser. Unknown chunks and canonical 44-byte LINEAR16 work.
fn validate_riff(b: &[u8]) -> Result<(), Error> {
    validate_wav_header(&mut Cursor::new(b), b.len() as u64)
}
fn validate_wav_header(r: &mut (impl Read + Seek), len: u64) -> Result<(), Error> {
    fn u16at(b: &[u8], n: usize) -> u16 {
        u16::from_le_bytes([b[n], b[n + 1]])
    }
    fn u32at(b: &[u8], n: usize) -> u32 {
        u32::from_le_bytes([b[n], b[n + 1], b[n + 2], b[n + 3]])
    }
    if len < 12 {
        return Err(Error::InvalidHeader);
    }
    let mut header = [0; 12];
    r.seek(SeekFrom::Start(0))?;
    r.read_exact(&mut header)?;
    if &header[..4] != b"RIFF" || &header[8..12] != b"WAVE" || u32at(&header, 4) as u64 + 8 != len {
        return Err(Error::InvalidHeader);
    }
    let mut p = 12u64;
    let mut align = None;
    let mut data = None;
    let mut metadata_bytes = 12u64;
    while p < len {
        if len - p < 8 {
            return Err(Error::InvalidHeader);
        }
        r.seek(SeekFrom::Start(p))?;
        let mut chunk = [0; 8];
        r.read_exact(&mut chunk)?;
        let n = u32at(&chunk, 4) as u64;
        let start = p + 8;
        let end = start
            .checked_add(n)
            .filter(|&v| v <= len)
            .ok_or(Error::InvalidHeader)?;
        metadata_bytes = metadata_bytes
            .checked_add(8 + n % 2 + if &chunk[..4] == b"data" { 0 } else { n })
            .filter(|&v| v <= 65536)
            .ok_or(Error::InvalidHeader)?;
        match &chunk[..4] {
            b"fmt " => {
                if n < 16 || align.is_some() {
                    return Err(Error::InvalidHeader);
                }
                let mut format = [0; 40];
                r.read_exact(&mut format[..n.min(40) as usize])?;
                let b = &format;
                let start = 0;
                let c = u16at(b, start + 2);
                let rate = u32at(b, start + 4);
                let bits = u16at(b, start + 14);
                if !(1..=2).contains(&c) || rate == 0 || ![16, 24, 32].contains(&bits) {
                    return Err(Error::UnsupportedFormat);
                }
                match u16at(b, start) {
                    1 => {}
                    3 if bits == 32 => {}
                    0xfffe => {
                        if n < 40 || u16at(b, start + 16) != 22 {
                            return Err(Error::InvalidHeader);
                        }
                        // Deliberately refuse narrower valid bits: hound's exposed
                        // valid width does not imply correct left-aligned decoding.
                        if u16at(b, start + 18) != bits {
                            return Err(Error::UnsupportedFormat);
                        }
                        let guid = &b[start + 24..start + 40];
                        let pcm = [1, 0, 0, 0, 0, 0, 16, 0, 128, 0, 0, 170, 0, 56, 155, 113];
                        let float = [3, 0, 0, 0, 0, 0, 16, 0, 128, 0, 0, 170, 0, 56, 155, 113];
                        if guid != pcm && !(bits == 32 && guid == float) {
                            return Err(Error::UnsupportedFormat);
                        }
                    }
                    _ => return Err(Error::UnsupportedFormat),
                }
                let a = c * (bits / 8);
                if u16at(b, start + 12) != a
                    || rate.checked_mul(a as u32) != Some(u32at(b, start + 8))
                {
                    return Err(Error::InvalidHeader);
                }
                align = Some(a as u64);
            }
            b"data" => {
                if data.is_some() || align.is_none() {
                    return Err(Error::InvalidHeader);
                }
                data = Some(n);
            }
            _ => {}
        }
        p = end
            .checked_add(n % 2)
            .filter(|&v| v <= len)
            .ok_or(Error::InvalidHeader)?;
    }
    match (align, data) {
        (Some(a), Some(n)) if n % a == 0 => Ok(()),
        _ => Err(Error::InvalidHeader),
    }
}
