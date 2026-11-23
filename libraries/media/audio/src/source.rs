//! Descriptor-bound Linux source confinement and seeked WAV decoding.
use crate::{Audio, Error, MAX_RESAMPLED_FRAMES, validate_wav_header};
use std::{fs::File, io::{BufReader, Read, Seek, SeekFrom}, path::Path};

// Three hours, 48 kHz, eight channels, four bytes/sample, plus 64 KiB
// of RIFF headers. Standard RIFF's 32-bit length is a tighter bound today.
pub(crate) const MAX_SOURCE_BYTES: u64 = 10800 * 48000 * 8 * 4 + 65536;
const MAX_SOURCE_SAMPLES: u64 = 10800 * 48000 * 8;

#[cfg(target_os = "linux")]
pub(crate) fn open(base: &Path, path: &Path) -> Result<File, Box<dyn std::error::Error>> {
    use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};
    // O_NONBLOCK prevents opening a FIFO from waiting for a writer. It does
    // not change regular-file reads. Inspect this descriptor before reading.
    let file = File::options().read(true).custom_flags(libc::O_NONBLOCK).open(base.join(path))?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err("path: source is not a regular file".into());
    }
    let resolved = std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))?;
    if !resolved.starts_with(base) {
        return Err("path: opened source escapes timeline directory".into());
    }
    if metadata.len() > MAX_SOURCE_BYTES {
        return Err(Error::InputTooLarge { max_bytes: MAX_SOURCE_BYTES }.into());
    }
    Ok(file)
}
#[cfg(not(target_os = "linux"))]
pub(crate) fn open(_: &Path, _: &Path) -> Result<File, Box<dyn std::error::Error>> {
    Err("path: descriptor-bound source confinement requires Linux /proc/self/fd".into())
}

pub(crate) struct Wav<R> {
    reader: hound::WavReader<R>,
    pub frames: usize,
}
impl<R: Read + Seek> Wav<R> {
    pub fn new(mut input: R, len: u64) -> Result<Self, Error> {
        if len > MAX_SOURCE_BYTES {
            return Err(Error::InputTooLarge { max_bytes: MAX_SOURCE_BYTES });
        }
        validate_wav_header(&mut input, len)?;
        input.seek(SeekFrom::Start(0))?;
        let reader = hound::WavReader::new(input)?;
        // len() counts interleaved values; duration() counts channel frames.
        if u64::from(reader.len()) > MAX_SOURCE_SAMPLES {
            return Err(Error::InputTooLarge { max_bytes: MAX_SOURCE_BYTES });
        }
        let frames = (u64::from(reader.duration()) * 48000)
            .div_ceil(u64::from(reader.spec().sample_rate));
        if frames > MAX_RESAMPLED_FRAMES as u64 {
            return Err(Error::OutputTooLong { max_frames: MAX_RESAMPLED_FRAMES });
        }
        Ok(Self { reader, frames: frames as usize })
    }
    pub fn decode(mut self, range: std::ops::Range<usize>) -> Result<Audio, Error> {
        let spec = self.reader.spec();
        if range.start > range.end || range.end > self.frames {
            return Err(Error::InvalidAudio);
        }
        if range.is_empty() {
            return Audio::silence(48000, spec.channels, 0);
        }
        let (start, end) = if spec.sample_rate == 48000 {
            (range.start, range.end)
        } else {
            // Keep both sides of the 512-tap sinc without decoding the rest
            // of the WAV. Absolute output positions preserve resampler phase.
            let start = (range.start as u64 * u64::from(spec.sample_rate) / 48000)
                .saturating_sub(512);
            let end = (range.end as u64 * u64::from(spec.sample_rate)).div_ceil(48000) + 512;
            (start as usize, end.min(u64::from(self.reader.duration())) as usize)
        };
        self.reader.seek(start as u32)?;
        let count = (end - start).checked_mul(spec.channels as usize).ok_or(Error::SizeOverflow)?;
        let mut samples = Vec::new();
        samples.try_reserve_exact(count).map_err(|_| Error::SizeOverflow)?;
        match (spec.sample_format, spec.bits_per_sample) {
            (hound::SampleFormat::Float, 32) => {
                for s in self.reader.samples::<f32>().take(count) {
                    samples.push(f64::from(s?));
                }
            }
            (hound::SampleFormat::Int, b @ (16 | 24 | 32)) => {
                let scale = 2.0_f64.powi(b as i32 - 1);
                for s in self.reader.samples::<i32>().take(count) {
                    samples.push(s? as f64 / scale);
                }
            }
            _ => return Err(Error::UnsupportedFormat),
        }
        if samples.len() != count {
            return Err(Error::InvalidHeader);
        }
        let audio = Audio::new(spec.sample_rate, spec.channels, samples)?;
        if spec.sample_rate == 48000 { Ok(audio) } else { audio.resample_window(start, range) }
    }
}
pub(crate) fn wav(file: File) -> Result<Wav<BufReader<File>>, Error> {
    let len = file.metadata()?.len();
    Wav::new(BufReader::new(file), len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    struct Counted {
        inner: Cursor<Vec<u8>>,
        read: usize,
    }
    impl Read for Counted {
        fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
            let n = self.inner.read(b)?;
            self.read += n;
            Ok(n)
        }
    }
    impl Seek for Counted {
        fn seek(&mut self, p: SeekFrom) -> std::io::Result<u64> { self.inner.seek(p) }
    }
    #[test]
    fn seeked_trim_matches_full_resampling_with_bounded_reads() {
        for rate in [22050, 44100, 48000, 96000] {
            let original = Audio::new(rate, 2, (0..rate * 2).map(|i| (i as f64 * 0.021).sin() * 0.1).collect()).unwrap();
            let mut bytes = Cursor::new(Vec::new());
            original.write_wav(&mut bytes, crate::WavFormat::Float32).unwrap();
            let expected = Audio::read_wav(Cursor::new(bytes.get_ref())).unwrap().resample_48k().unwrap();
            let len = bytes.get_ref().len() as u64;
            let mut input = Counted { inner: bytes, read: 0 };
            let actual = Wav::new(&mut input, len).unwrap().decode(24000..24100).unwrap();
            assert_eq!(actual.frames(), 100);
            assert!(actual.samples().iter().zip(&expected.samples()[48000..48200]).all(|(a,b)| (a-b).abs() < 1e-12), "rate {rate}");
            assert!(input.read < 12000, "only the trim, filter support and headers are read: {}", input.read);
        }
    }
    #[test]
    fn opened_descriptor_survives_path_replacement() {
        let dir = std::env::temp_dir().join(format!("audio-descriptor-{}", std::process::id()));
        std::fs::create_dir(&dir).unwrap();
        let base = std::fs::canonicalize(&dir).unwrap();
        std::fs::write(base.join("source"), b"original").unwrap();
        let mut file = open(&base, Path::new("source")).unwrap();
        std::fs::rename(base.join("source"), base.join("moved")).unwrap();
        std::fs::write(base.join("source"), b"replacement").unwrap();
        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();
        assert_eq!(text, "original");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
