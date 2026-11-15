//! CFR AV1 Main, 8-bit BT.709 limited-range 420. No container or process I/O.
use media_frame::Yuv420;
use rav1e::prelude::*;
use std::sync::Arc;

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
    tiles: (usize, usize),
}
impl Encoder {
    pub fn new(settings: Settings) -> Result<Self, Error> {
        let s = settings;
        if s.speed > 10
            || s.fps_num == 0
            || s.fps_den == 0
            || s.key_interval == 0
            || s.threads == 0
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
        if self.flushed
            || (frame.width, frame.height) != (self.settings.width, self.settings.height)
        {
            return Err(Error("closed encoder or frame size mismatch".into()));
        }
        self.ctx
            .send_frame(frame.frame.clone())
            .map_err(|e| Error(e.to_string()))?;
        self.drain()
    }
    /// Idempotent end-of-input. No further submissions are accepted.
    pub fn finish(&mut self) -> Result<Vec<Packet>, Error> {
        if self.flushed {
            return Ok(Vec::new());
        }
        self.flushed = true;
        self.ctx
            .send_frame(None)
            .map_err(|e| Error(e.to_string()))?;
        self.drain()
    }
    fn drain(&mut self) -> Result<Vec<Packet>, Error> {
        let mut packets = Vec::new();
        loop {
            match self.ctx.receive_packet() {
                Ok(p) => packets.push(Packet {
                    data: p.data,
                    pts: p.input_frameno,
                    duration: 1,
                    key_frame: p.frame_type == FrameType::KEY,
                }),
                Err(EncoderStatus::Encoded) => continue,
                Err(EncoderStatus::NeedMoreData) if !self.flushed => break,
                Err(EncoderStatus::LimitReached) if self.flushed => break,
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
    }
}
