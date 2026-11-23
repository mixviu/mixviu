use crate::{Error, Result, children, element, uint, vint};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct Track<'a> {
    pub number: u64,
    pub kind: u64,
    pub codec: &'a str,
    pub codec_private: &'a [u8],
    pub codec_delay_ns: u64,
    pub seek_pre_roll_ns: u64,
    pub width: u64,
    pub height: u64,
    pub default_duration_ns: u64,
    pub sample_rate: f64,
    pub channels: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet<'a> {
    pub track: u64,
    /// Signed container ticks, before subtracting CodecDelay. Multiply by scale_ns.
    pub timestamp: i64,
    pub key_frame: bool,
    pub data: &'a [u8],
    pub duration: Option<u64>,
    pub discard_padding_ns: i64,
}
#[derive(Debug)]
pub struct Cue {
    pub timestamp: u64,
    pub track: u64,
    pub cluster_position: u64,
}
/// Zero-copy finite-file reader. Metadata/index storage is bounded by input
/// length. Unknown IDs are skipped without recursion; unsupported lacing fails.
pub struct Demuxer<'a> {
    pub tracks: Vec<Track<'a>>,
    pub cues: Vec<Cue>,
    pub scale_ns: u64,
    pub duration: Option<f64>,
    packets: Vec<Packet<'a>>,
    keyframes: HashMap<(u64, u64, u64), usize>,
    cursor: usize,
}
impl<'a> Demuxer<'a> {
    pub fn new(input: &'a [u8]) -> Result<Self> {
        // Zero-copy input plus capped indexes: at most 256 MiB of input and
        // approximately 64 MiB of metadata. Parsing has fixed depth (<= 6),
        // never recursively traverses unknown masters.
        if input.len() > 256 * 1024 * 1024 {
            return Err(Error::Invalid("input allocation limit"));
        }
        validate_depth(input, 0, &mut 0)?;
        let mut p = 0;
        let header = element(input, &mut p)?;
        if header.id != 0x1a45dfa3 {
            return Err(Error::Invalid("missing EBML header"));
        }
        let mut webm = false;
        for e in children(header.data) {
            let e = e?;
            if e.id == 0x4282 {
                webm = e.data == b"webm";
            }
        }
        if !webm {
            return Err(Error::Invalid("not WebM"));
        }
        let mut segment = element(input, &mut p)?;
        while segment.id == 0xec {
            segment = element(input, &mut p)?;
        }
        if segment.id != 0x18538067 || p != input.len() {
            return Err(Error::Invalid("expected one Segment"));
        }
        let mut d = Self {
            tracks: Vec::new(),
            cues: Vec::new(),
            scale_ns: 1_000_000,
            duration: None,
            packets: Vec::new(),
            keyframes: HashMap::new(),
            cursor: 0,
        };
        let mut track_numbers = HashSet::new();
        for e in children(segment.data) {
            let e = e?;
            match e.id {
                0x1549a966 => {
                    for c in children(e.data) {
                        let c = c?;
                        match c.id {
                            0x2ad7b1 => {
                                d.scale_ns = uint(c.data)?;
                                if d.scale_ns == 0 {
                                    return Err(Error::Invalid("zero scale"));
                                }
                            }
                            0x4489 => {
                                let v = match c.data.len() {
                                    4 => f32::from_be_bytes(
                                        c.data.try_into().map_err(|_| Error::Truncated)?,
                                    ) as f64,
                                    8 => f64::from_be_bytes(
                                        c.data.try_into().map_err(|_| Error::Truncated)?,
                                    ),
                                    _ => return Err(Error::Invalid("float size")),
                                };
                                if !v.is_finite() || v < 0.0 {
                                    return Err(Error::Invalid("duration"));
                                }
                                d.duration = Some(v);
                            }
                            _ => {}
                        }
                    }
                }
                0x1654ae6b => {
                    for c in children(e.data) {
                        let c = c?;
                        if c.id != 0xae {
                            continue;
                        }
                        let mut t = Track::default();
                        for f in children(c.data) {
                            let f = f?;
                            match f.id {
                                0xd7 => t.number = uint(f.data)?,
                                0x83 => t.kind = uint(f.data)?,
                                0x86 => {
                                    t.codec = std::str::from_utf8(f.data)
                                        .map_err(|_| Error::Invalid("codec UTF-8"))?
                                }
                                0x63a2 => t.codec_private = f.data,
                                0x56aa => t.codec_delay_ns = uint(f.data)?,
                                0x56bb => t.seek_pre_roll_ns = uint(f.data)?,
                                0x23e383 => t.default_duration_ns = uint(f.data)?,
                                0xe0 | 0xe1 => {
                                    for v in children(f.data) {
                                        let v = v?;
                                        match v.id {
                                            0xb0 => t.width = uint(v.data)?,
                                            0xba => t.height = uint(v.data)?,
                                            0x9f => t.channels = uint(v.data)?,
                                            0xb5 => t.sample_rate = float(v.data)?,
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        if t.number == 0 || t.codec.is_empty() || !track_numbers.insert(t.number) {
                            return Err(Error::Invalid("invalid track"));
                        }
                        match t.codec {
                            "V_AV1" if t.kind == 1 => {}
                            "A_OPUS" if t.kind == 2 => {
                                if t.channels == 0 {
                                    t.channels = 1;
                                }
                                if t.sample_rate == 0.0 {
                                    t.sample_rate = 48_000.0;
                                }
                            }
                            _ => return Err(Error::UnsupportedCodec(t.codec.to_owned())),
                        }
                        if d.tracks.len() >= 64 {
                            return Err(Error::Invalid("track limit"));
                        }
                        d.tracks.push(t);
                    }
                }
                0x1f43b675 => {
                    let mut base = None;
                    // Timestamp may occur after a block in a valid master.
                    for c in children(e.data) {
                        let c = c?;
                        if c.id == 0xe7 {
                            base = Some(
                                i64::try_from(uint(c.data)?).map_err(|_| Error::SizeOverflow)?,
                            );
                        }
                    }
                    let base = base.ok_or(Error::Invalid("Cluster missing Timestamp"))?;
                    for c in children(e.data) {
                        let c = c?;
                        match c.id {
                            0xa3 => d.block(c.data, base, None, e.start as u64)?,
                            0xa0 => {
                                let mut block = None;
                                let mut reference = false;
                                let mut duration = None;
                                let mut padding = 0;
                                for f in children(c.data) {
                                    let f = f?;
                                    match f.id {
                                        0xa1 => block = Some(f.data),
                                        0xfb => reference = true,
                                        0x9b => duration = Some(uint(f.data)?),
                                        0x75a2 => padding = signed(f.data)?,
                                        _ => {}
                                    }
                                }
                                d.block(
                                    block.ok_or(Error::Invalid("BlockGroup missing Block"))?,
                                    base,
                                    Some(!reference),
                                    e.start as u64,
                                )?;
                                let packet = d.packets.last_mut().ok_or(Error::Truncated)?;
                                packet.duration = duration;
                                packet.discard_padding_ns = padding;
                            }
                            _ => {}
                        }
                    }
                }
                0x1c53bb6b => {
                    for c in children(e.data) {
                        let c = c?;
                        if c.id != 0xbb {
                            continue;
                        }
                        let mut time = None;
                        let mut positions = Vec::new();
                        for f in children(c.data) {
                            let f = f?;
                            match f.id {
                                0xb3 => time = Some(uint(f.data)?),
                                0xb7 => {
                                    let mut track = 0;
                                    let mut pos = None;
                                    for v in children(f.data) {
                                        let v = v?;
                                        match v.id {
                                            0xf7 => track = uint(v.data)?,
                                            0xf1 => pos = Some(uint(v.data)?),
                                            _ => {}
                                        }
                                    }
                                    positions
                                        .push((track, pos.ok_or(Error::Invalid("cue position"))?));
                                    if positions.len() > 64 {
                                        return Err(Error::Invalid("cue positions limit"));
                                    }
                                }
                                _ => {}
                            }
                        }
                        for (track, pos) in positions {
                            if d.cues.len() >= 100_000 {
                                return Err(Error::Invalid("cue limit"));
                            }
                            d.cues.push(Cue {
                                timestamp: time.ok_or(Error::Invalid("cue time"))?,
                                track,
                                cluster_position: pos,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        if d.tracks.is_empty() || d.packets.iter().any(|p| !track_numbers.contains(&p.track)) {
            return Err(Error::Invalid("missing packet track"));
        }
        // Validate cue targets now; seek never trusts an arbitrary byte offset.
        d.cues.retain(|cue| {
            d.keyframes
                .contains_key(&(cue.cluster_position, cue.track, cue.timestamp))
        });
        Ok(d)
    }
    fn block(&mut self, b: &'a [u8], base: i64, key: Option<bool>, cluster: u64) -> Result<()> {
        if self.packets.len() >= 500_000 {
            return Err(Error::Invalid("packet limit"));
        }
        let mut p = 0;
        let (track, unknown) = vint(b, &mut p, false)?;
        if unknown || track == 0 {
            return Err(Error::Invalid("block track"));
        }
        let hdr = b.get(p..p + 3).ok_or(Error::Truncated)?;
        if hdr[2] & 6 != 0 {
            return Err(Error::UnsupportedLacing);
        }
        let timestamp = base
            .checked_add(i16::from_be_bytes([hdr[0], hdr[1]]) as i64)
            .ok_or(Error::SizeOverflow)?;
        let key_frame = key.unwrap_or(hdr[2] & 128 != 0);
        if key_frame && timestamp >= 0 {
            self.keyframes
                .entry((cluster, track, timestamp as u64))
                .or_insert(self.packets.len());
        }
        self.packets.push(Packet {
            track,
            timestamp,
            key_frame,
            data: &b[p + 3..],
            duration: None,
            discard_padding_ns: 0,
        });
        Ok(())
    }
    pub fn packets(&self) -> &[Packet<'a>] {
        &self.packets
    }
    /// Presentation ticks after CodecDelay, rounded to the nearest container
    /// tick (halves up), matching ffmpeg's packet time base conversion.
    pub fn presentation_timestamp(&self, packet: &Packet<'_>) -> Result<i64> {
        let track = self
            .tracks
            .iter()
            .find(|t| t.number == packet.track)
            .ok_or(Error::Invalid("packet track"))?;
        let delay = (u128::from(track.codec_delay_ns) + u128::from(self.scale_ns) / 2)
            / u128::from(self.scale_ns);
        packet
            .timestamp
            .checked_sub(i64::try_from(delay).map_err(|_| Error::SizeOverflow)?)
            .ok_or(Error::SizeOverflow)
    }
    pub fn next_packet(&mut self) -> Option<Packet<'a>> {
        let p = self.packets.get(self.cursor)?.clone();
        self.cursor += 1;
        Some(p)
    }
    /// Position at the video keyframe at or before a container tick. None before
    /// the first cue, or if the file has no video cue. Audio preroll is the caller's job.
    pub fn seek(&mut self, timestamp: u64) -> Option<u64> {
        let video = self.tracks.iter().find(|t| t.kind == 1)?.number;
        let cue = self
            .cues
            .iter()
            .filter(|c| c.track == video && c.timestamp <= timestamp)
            .max_by_key(|c| c.timestamp)?;
        self.cursor = *self
            .keyframes
            .get(&(cue.cluster_position, video, cue.timestamp))?;
        Some(cue.timestamp)
    }
}

fn float(b: &[u8]) -> Result<f64> {
    let v = match b.len() {
        4 => f32::from_be_bytes(b.try_into().map_err(|_| Error::Truncated)?) as f64,
        8 => f64::from_be_bytes(b.try_into().map_err(|_| Error::Truncated)?),
        _ => return Err(Error::Invalid("float size")),
    };
    if !v.is_finite() || v < 0.0 {
        return Err(Error::Invalid("invalid float"));
    }
    Ok(v)
}
fn signed(b: &[u8]) -> Result<i64> {
    if b.is_empty() || b.len() > 8 {
        return Err(Error::Invalid("signed integer size"));
    }
    let mut bytes = [if b[0] & 128 != 0 { 255 } else { 0 }; 8];
    bytes[8 - b.len()..].copy_from_slice(b);
    Ok(i64::from_be_bytes(bytes))
}

fn validate_depth(b: &[u8], depth: usize, elements: &mut usize) -> Result<()> {
    if depth > 8 {
        return Err(Error::Invalid("nesting depth limit"));
    }
    for e in children(b) {
        let e = e?;
        *elements += 1;
        if *elements > 2_000_000 {
            return Err(Error::Invalid("element count limit"));
        }
        if matches!(
            e.id,
            0x1a45dfa3
                | 0x18538067
                | 0x1549a966
                | 0x1654ae6b
                | 0xae
                | 0xe0
                | 0xe1
                | 0x1f43b675
                | 0xa0
                | 0x1c53bb6b
                | 0xbb
                | 0xb7
        ) {
            validate_depth(e.data, depth + 1, elements)?;
        }
    }
    Ok(())
}
