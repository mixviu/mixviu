//! 48 kHz mono/stereo Opus packets. Encoding uses 20 ms frames; decoding accepts
//! every legal Opus packet duration, including those in the RFC vector sets.
#[cfg(feature = "bench-cranpose")]
use opus_bench as codec;
#[cfg(not(feature = "bench-cranpose"))]
use opus_pure as codec;

pub const SAMPLE_RATE: u32 = 48_000;
pub const FRAME_SAMPLES: usize = 960;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Channels {
    Mono,
    Stereo,
}
impl Channels {
    pub fn count(self) -> usize {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
        }
    }
}
#[derive(Debug)]
pub struct Error(pub String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}

/// Mapping family 0: one stream, coupled only for stereo. No mapping table is
/// serialised in the 19-byte OpusHead. Rates and pre-skip use 48 kHz units.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpusHead {
    pub channels: u8,
    pub pre_skip: u16,
    pub input_sample_rate: u32,
    pub output_gain: i16,
    pub mapping_family: u8,
    pub stream_count: u8,
    pub coupled_count: u8,
}
impl OpusHead {
    pub fn bytes(&self) -> [u8; 19] {
        let mut b = [0; 19];
        b[..8].copy_from_slice(b"OpusHead");
        b[8] = 1;
        b[9] = self.channels;
        b[10..12].copy_from_slice(&self.pre_skip.to_le_bytes());
        b[12..16].copy_from_slice(&self.input_sample_rate.to_le_bytes());
        b[16..18].copy_from_slice(&self.output_gain.to_le_bytes());
        b[18] = self.mapping_family;
        b
    }
}
#[derive(Debug)]
pub struct Packet {
    pub data: Vec<u8>,
    /// Samples per channel to discard at the end, for container discard padding.
    pub discard_padding: u16,
}
pub trait Encoder {
    fn head(&self) -> &OpusHead;
    fn encode(&mut self, pcm: &[f32]) -> Result<Packet, Error>;
    /// Flushes the codec lookahead. Idempotent, including for empty input.
    fn finish(&mut self) -> Result<Option<Packet>, Error>;
}
#[derive(Debug)]
pub struct Decoded {
    pub pcm: Vec<f32>,
    pub channels: Channels,
    /// True only for an explicit missing packet. Invalid packets return errors.
    pub packet_lost: bool,
}
pub trait Decoder {
    /// None reports a lost 20 ms frame and requests concealment. Some(empty)
    /// is invalid input, not an implicit request for concealment.
    fn decode(&mut self, packet: Option<&[u8]>) -> Result<Decoded, Error>;
}
pub struct OpusEncoder {
    inner: codec::OpusEncoder,
    channels: Channels,
    head: OpusHead,
    submitted: bool,
    closed: bool,
}
impl OpusEncoder {
    pub fn new(channels: Channels, bitrate: u32) -> Result<Self, Error> {
        if !(6_000..=512_000).contains(&bitrate) {
            return Err(Error("bitrate must be 6000..=512000 bits/second".into()));
        }
        let mut inner = codec::OpusEncoder::new(
            SAMPLE_RATE as i32, channels.count(), codec::Application::Audio,
        ).map_err(|e| Error(e.to_string()))?;
        inner.bitrate_bps = bitrate as i32;
        let pre_skip = u16::try_from(inner.lookahead())
            .map_err(|_| Error("encoder lookahead exceeds OpusHead".into()))?;
        if usize::from(pre_skip) > FRAME_SAMPLES {
            return Err(Error("encoder lookahead exceeds one frame".into()));
        }
        Ok(Self {
            inner, channels,
            head: OpusHead { channels: channels.count() as u8, pre_skip,
                input_sample_rate: SAMPLE_RATE, output_gain: 0, mapping_family: 0,
                stream_count: 1, coupled_count: u8::from(channels == Channels::Stereo) },
            submitted: false, closed: false,
        })
    }
    fn frame(&mut self, pcm: &[f32]) -> Result<Packet, Error> {
        let mut data = vec![0; 4000];
        let n = self.inner.encode(pcm, FRAME_SAMPLES, &mut data)
            .map_err(|e| Error(e.to_string()))?;
        data.truncate(n);
        Ok(Packet { data, discard_padding: 0 })
    }
}
impl Encoder for OpusEncoder {
    fn head(&self) -> &OpusHead { &self.head }
    fn encode(&mut self, pcm: &[f32]) -> Result<Packet, Error> {
        if self.closed { return Err(Error("encoder is closed".into())); }
        if pcm.len() != FRAME_SAMPLES * self.channels.count() || pcm.iter().any(|v| !v.is_finite()) {
            return Err(Error("expected one finite 20 ms interleaved PCM frame".into()));
        }
        let result = self.frame(pcm);
        if result.is_err() { self.closed = true; } else { self.submitted = true; }
        result
    }
    fn finish(&mut self) -> Result<Option<Packet>, Error> {
        if self.closed { return Ok(None); }
        self.closed = true;
        if !self.submitted || self.head.pre_skip == 0 { return Ok(None); }
        let mut packet = self.frame(&vec![0.; FRAME_SAMPLES * self.channels.count()])?;
        packet.discard_padding = (FRAME_SAMPLES - usize::from(self.head.pre_skip)) as u16;
        Ok(Some(packet))
    }
}
pub struct OpusDecoder {
    inner: codec::OpusDecoder,
    channels: Channels,
}
impl OpusDecoder {
    pub fn new(channels: Channels) -> Result<Self, Error> {
        let inner = codec::OpusDecoder::new(SAMPLE_RATE as i32, channels.count())
            .map_err(|e| Error(e.to_string()))?;
        Ok(Self { inner, channels })
    }
}
impl Decoder for OpusDecoder {
    fn decode(&mut self, packet: Option<&[u8]>) -> Result<Decoded, Error> {
        if packet.is_some_and(|p| p.is_empty() || p.len() > 65_535) {
            return Err(Error("empty or oversized Opus packet".into()));
        }
        let capacity = if packet.is_none() { FRAME_SAMPLES } else { 5760 };
        let mut pcm = vec![0.; capacity * self.channels.count()];
        let n = self.inner.decode(packet.unwrap_or_default(), capacity, &mut pcm)
            .map_err(|e| Error(e.to_string()))?;
        pcm.truncate(n * self.channels.count());
        if pcm.iter().any(|v| !v.is_finite()) { return Err(Error("non-finite decoder output".into())); }
        Ok(Decoded { pcm, channels: self.channels, packet_lost: packet.is_none() })
    }
}
