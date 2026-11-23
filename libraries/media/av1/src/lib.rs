//! CFR AV1 Main, 8-bit BT.709 limited-range 420. No container or process I/O.
use media_frame::Yuv420;
use rav1e::prelude::*;
use std::sync::Arc;
mod decoder;
pub mod reference;
pub use decoder::{DecodedFrame, Decoder};

#[derive(Clone, Copy, Debug)]
pub enum Target {
    Quality(u8),
    Bitrate(u32),
}
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub width: usize,
    pub height: usize,
    pub fps_num: u32,
    pub fps_den: u32,
    pub speed: u8,
    pub target: Target,
    pub key_interval: u64,
    pub tile_cols: usize,
    pub tile_rows: usize,
    pub threads: usize,
}
#[derive(Debug)]
pub struct Error(pub String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}

/// Immutable prepared pixels can be reused for repeats, including across encoders
/// of the same dimensions. Each submit still runs a full CFR encode step.
#[derive(Clone)]
pub struct Prepared {
    frame: Arc<Frame<u8>>,
    width: usize,
    height: usize,
}
#[derive(Debug)]
pub struct Packet {
    pub data: Vec<u8>,
    /// Ticks of fps_den / fps_num seconds, starting at zero.
    pub pts: u64,
    pub duration: u64,
    pub key_frame: bool,
}
pub struct Encoder {
    ctx: Context<u8>,
    settings: Settings,
    flushed: bool,
    failed: Option<String>,
    submitted: u64,
    tiles: (usize, usize),
}
impl Encoder {
    pub fn new(settings: Settings) -> Result<Self, Error> {
        Self::new_inner(settings, false)
    }
    /// Narrow repeat-tracker contract: every coded frame is error resilient.
    pub fn new_repeats(settings: Settings) -> Result<Self, Error> {
        Self::new_inner(settings, true)
    }
    fn new_inner(settings: Settings, error_resilient: bool) -> Result<Self, Error> {
        let s = settings;
        if !(16..=65535).contains(&s.width) || !(16..=65535).contains(&s.height) {
            return Err(Error("dimensions must be 16..=65535".into()));
        }
        if s.speed > 10
            || s.fps_num == 0
            || s.fps_den == 0
            || s.key_interval == 0
            || s.threads == 0
            || s.threads > 64
            || s.tile_cols > 64
            || s.tile_rows > 64
            || !s.tile_cols.is_power_of_two()
            || !s.tile_rows.is_power_of_two()
        {
            return Err(Error(
                "invalid speed, rate, interval, tiles or threads".into(),
            ));
        }
        let mut e = EncoderConfig::with_speed_preset(s.speed);
        e.width = s.width;
        e.height = s.height;
        e.time_base = Rational::new(s.fps_den as u64, s.fps_num as u64);
        e.bit_depth = 8;
        e.chroma_sampling = ChromaSampling::Cs420;
        e.chroma_sample_position = ChromaSamplePosition::Unknown;
        e.pixel_range = PixelRange::Limited;
        e.color_description = Some(ColorDescription {
            color_primaries: ColorPrimaries::BT709,
            transfer_characteristics: TransferCharacteristics::BT709,
            matrix_coefficients: MatrixCoefficients::BT709,
        });
        e.low_latency = true;
        e.error_resilient = error_resilient;
        e.set_key_frame_interval(s.key_interval, s.key_interval);
        e.tile_cols = s.tile_cols;
        e.tile_rows = s.tile_rows;
        match s.target {
            Target::Quality(q) => e.quantizer = q as usize,
            Target::Bitrate(b) => {
                if b == 0 || b > i32::MAX as u32 {
                    return Err(Error("invalid bitrate".into()));
                }
                e.bitrate = b as i32;
                e.quantizer = 255;
            }
        }
        let config = Config::new().with_encoder_config(e).with_threads(s.threads);
        let tiling = config.tiling_info().map_err(|e| Error(e.to_string()))?;
        let tiles = (tiling.cols, tiling.rows);
        let ctx = config.new_context().map_err(|e| Error(e.to_string()))?;
        Ok(Self {
            ctx,
            settings,
            flushed: false,
            failed: None,
            submitted: 0,
            tiles,
        })
    }
    pub fn tiles(&self) -> (usize, usize) {
        self.tiles
    }
    pub fn prepare(&self, src: &Yuv420) -> Result<Prepared, Error> {
        src.validate().map_err(|e| Error(e.to_string()))?;
        if (src.width, src.height) != (self.settings.width, self.settings.height) {
            return Err(Error("frame size mismatch".into()));
        }
        let mut f = self.ctx.new_frame();
        for (i, data) in [&src.y, &src.u, &src.v].iter().enumerate() {
            let w = if i == 0 {
                src.width
            } else {
                src.width.div_ceil(2)
            };
            f.planes[i].copy_from_raw_u8(data, w, 1);
        }
        for plane in &mut f.planes {
            plane.pad(src.width, src.height);
        }
        Ok(Prepared {
            frame: Arc::new(f),
            width: src.width,
            height: src.height,
        })
    }
    /// Returns all currently available packets; callers should mux them promptly.
    pub fn submit(&mut self, frame: &Prepared) -> Result<Vec<Packet>, Error> {
        if let Some(error) = &self.failed {
            return Err(Error(error.clone()));
        }
        if self.flushed
            || (frame.width, frame.height) != (self.settings.width, self.settings.height)
        {
            return Err(Error("closed encoder or frame size mismatch".into()));
        }
        let result = self
            .ctx
            .send_frame(frame.frame.clone())
            .map_err(|e| Error(e.to_string()));
        self.remember_failure(result)?;
        self.submitted += 1;
        let result = self.drain(false);
        self.remember_failure(result)
    }
    /// Idempotent end-of-input, including an empty input. No further submissions
    /// are accepted. Flush/drain errors are permanent and repeated on later calls.
    /// A fatal drain error discards packets collected during that call: the
    /// context cannot complete the stream, so callers must discard that stream.
    pub fn finish(&mut self) -> Result<Vec<Packet>, Error> {
        self.finish_using(|encoder| {
            encoder
                .ctx
                .send_frame(None)
                .map_err(|e| Error(e.to_string()))?;
            encoder.drain(true)
        })
    }
    fn finish_using(
        &mut self,
        flush: impl FnOnce(&mut Self) -> Result<Vec<Packet>, Error>,
    ) -> Result<Vec<Packet>, Error> {
        if let Some(error) = &self.failed {
            return Err(Error(error.clone()));
        }
        if self.flushed {
            return Ok(Vec::new());
        }
        let result = if self.submitted == 0 {
            Ok(Vec::new())
        } else {
            flush(self)
        };
        let packets = self.remember_failure(result)?;
        self.flushed = true;
        Ok(packets)
    }
    fn remember_failure<T>(&mut self, result: Result<T, Error>) -> Result<T, Error> {
        if let Err(error) = &result {
            self.failed = Some(error.0.clone());
        }
        result
    }
    fn drain(&mut self, finishing: bool) -> Result<Vec<Packet>, Error> {
        Self::drain_using(finishing, || {
            self.ctx.receive_packet().map(|p| Packet {
                data: p.data,
                pts: p.input_frameno,
                duration: 1,
                key_frame: p.frame_type == FrameType::KEY,
            })
        })
    }
    fn drain_using(
        finishing: bool,
        mut receive: impl FnMut() -> Result<Packet, EncoderStatus>,
    ) -> Result<Vec<Packet>, Error> {
        let mut packets = Vec::new();
        loop {
            match receive() {
                Ok(p) => packets.push(p),
                Err(EncoderStatus::Encoded) => continue,
                Err(EncoderStatus::NeedMoreData) if !finishing => break,
                Err(EncoderStatus::LimitReached) if finishing => break,
                Err(e) => return Err(Error(e.to_string())),
            }
        }
        Ok(packets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn settings() -> Settings {
        Settings {
            width: 32,
            height: 32,
            fps_num: 30000,
            fps_den: 1001,
            speed: 10,
            target: Target::Quality(100),
            key_interval: 2,
            tile_cols: 1,
            tile_rows: 1,
            threads: 1,
        }
    }
    #[test]
    fn packets_repeats_and_flush() {
        let mut enc = Encoder::new(settings()).unwrap();
        let src = media_frame::Bgra::solid(32, 32, [0, 0, 0, 255])
            .unwrap()
            .to_yuv420();
        let prepared = enc.prepare(&src).unwrap();
        let mut packets = Vec::new();
        for _ in 0..5 {
            packets.extend(enc.submit(&prepared).unwrap());
        }
        packets.extend(enc.finish().unwrap());
        assert_eq!(
            packets.iter().map(|p| p.pts).collect::<Vec<_>>(),
            [0, 1, 2, 3, 4]
        );
        assert!(packets[0].key_frame);
        assert!(packets[2].key_frame);
        assert!(packets[4].key_frame);
        assert!(
            packets
                .iter()
                .all(|p| p.duration == 1 && !p.data.is_empty())
        );
        assert!(enc.finish().unwrap().is_empty());
        assert!(enc.submit(&prepared).is_err());
    }
    #[test]
    fn empty_finish() {
        let mut enc = Encoder::new(settings()).unwrap();
        assert!(enc.finish().unwrap().is_empty());
        assert!(enc.finish().unwrap().is_empty());
        assert!(enc.flushed);
    }
    #[test]
    fn finish_failures_stay_errors() {
        for stage in ["send_frame(None)", "drain after packet"] {
            let mut enc = Encoder::new(settings()).unwrap();
            enc.submitted = 1;
            let result = enc.finish_using(|_| {
                if stage == "send_frame(None)" {
                    return Err(Error(stage.into()));
                }
                let mut events = [
                    Ok(Packet {
                        data: vec![1],
                        pts: 0,
                        duration: 1,
                        key_frame: true,
                    }),
                    Err(EncoderStatus::Failure),
                ]
                .into_iter();
                Encoder::drain_using(true, || events.next().unwrap())
            });
            let error = result.unwrap_err().0;
            assert!(!enc.flushed);
            assert_eq!(enc.finish().unwrap_err().0, error);
            let prepared = enc
                .prepare(
                    &media_frame::Bgra::solid(32, 32, [0, 0, 0, 255])
                        .unwrap()
                        .to_yuv420(),
                )
                .unwrap();
            assert_eq!(enc.submit(&prepared).unwrap_err().0, error);
        }
    }
    #[test]
    fn odd_dimensions_and_size_mismatches() {
        let mut s = settings();
        s.width = 33;
        s.height = 17;
        let mut enc = Encoder::new(s).unwrap();
        let source = media_frame::Bgra::solid(33, 17, [20, 40, 60, 255])
            .unwrap()
            .to_yuv420();
        let prepared = enc.prepare(&source).unwrap();
        let mut packets = enc.submit(&prepared).unwrap();
        packets.extend(enc.submit(&prepared).unwrap());
        packets.extend(enc.finish().unwrap());
        assert_eq!(packets.iter().map(|p| p.pts).collect::<Vec<_>>(), [0, 1]);
        let mut other = Encoder::new(settings()).unwrap();
        assert!(other.prepare(&source).is_err());
        assert!(other.submit(&prepared).is_err());
    }
    #[test]
    fn dimensions_rejected_before_tiling() {
        for dimension in [0, 1, 65536] {
            for width in [true, false] {
                let mut s = settings();
                if width {
                    s.width = dimension;
                } else {
                    s.height = dimension;
                }
                assert!(Encoder::new(s).is_err());
            }
        }
    }
    #[test]
    fn actual_4k_tiles() {
        let mut s = settings();
        s.width = 3840;
        s.height = 2160;
        s.tile_cols = 2;
        s.tile_rows = 2;
        assert_eq!(Encoder::new(s).unwrap().tiles(), (2, 2));
    }
    #[test]
    fn invalid_settings() {
        let mut s = settings();
        s.speed = 11;
        assert!(Encoder::new(s).is_err());
        s = settings();
        s.tile_cols = 3;
        assert!(Encoder::new(s).is_err());
        s = settings();
        s.target = Target::Bitrate(0);
        assert!(Encoder::new(s).is_err());
        s = settings();
        s.threads = usize::MAX;
        assert!(Encoder::new(s).is_err());
        s = settings();
        s.tile_rows = 128;
        assert!(Encoder::new(s).is_err());
    }
}
