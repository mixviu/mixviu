use crate::{Error, Result};
use std::io::{Seek, SeekFrom, Write};

fn size(n: u64) -> Result<[u8; 8]> {
    if n >= (1 << 56) - 1 {
        return Err(Error::SizeOverflow);
    }
    Ok((n | (1 << 56)).to_be_bytes())
}
fn elem(id: u32, data: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    let i = id.to_be_bytes();
    b.extend_from_slice(&i[i.iter().position(|x| *x != 0).unwrap_or(3)..]);
    // All metadata and packets have usize lengths, bounded below 2^56.
    b.extend_from_slice(&size(data.len() as u64).expect("element length below 2^56"));
    b.extend_from_slice(data);
    b
}
fn u(id: u32, n: u64) -> Vec<u8> {
    elem(id, &n.to_be_bytes())
}
fn master(id: u32, items: &[Vec<u8>]) -> Vec<u8> {
    elem(id, &items.concat())
}
pub struct Video {
    pub width: u32,
    pub height: u32,
    pub codec_private: Vec<u8>,
}
pub struct Opus {
    pub channels: u8,
    pub pre_skip: u16,
    pub input_sample_rate: u32,
}
impl Opus {
    pub fn head(&self) -> Vec<u8> {
        let mut b = b"OpusHead".to_vec();
        b.extend([1, self.channels]);
        b.extend(self.pre_skip.to_le_bytes());
        b.extend(self.input_sample_rate.to_le_bytes());
        b.extend([0, 0, 0]);
        b
    }
}
/// Requires an empty seekable target. Memory is O(video keyframes), never O(packets).
/// Non-seekable outputs must spool to a file; there is no silent live-file fallback.
pub struct Writer<W> {
    out: W,
    segment: u64,
    segment_size: u64,
    seek: u64,
    info: u64,
    tracks: u64,
    duration: u64,
    cluster: Option<(u64, u64, u64)>,
    cues: Vec<(u64, u64)>,
    end: u64,
    last: u64,
    audio: bool,
    video: bool,
    discard_ns: u64,
    duration_ms: Option<f64>,
    failed: bool,
}
impl<W: Write + Seek> Writer<W> {
    pub fn new(out: W, video: Video, audio: Option<Opus>) -> Result<Self> {
        Self::configured(out, Some(video), audio)
    }
    pub fn audio(out: W, audio: Opus) -> Result<Self> {
        Self::configured(out, None, Some(audio))
    }
    fn configured(mut out: W, video: Option<Video>, audio: Option<Opus>) -> Result<Self> {
        if out.stream_position()? != 0
            || out.seek(SeekFrom::End(0))? != 0
            || video.as_ref().is_some_and(|v| {
                v.width == 0
                    || v.height == 0
                    || v.codec_private.len() < 4
                    || v.codec_private[0] != 0x81
            })
            || audio
                .as_ref()
                .is_some_and(|a| !(1..=2).contains(&a.channels))
        {
            return Err(Error::Invalid("invalid target or track configuration"));
        }
        out.write_all(&master(
            0x1a45dfa3,
            &[
                u(0x4286, 1),
                u(0x42f7, 1),
                u(0x42f2, 4),
                u(0x42f3, 8),
                elem(0x4282, b"webm"),
                u(0x4287, 4),
                u(0x4285, 2),
            ],
        ))?;
        out.write_all(&0x18538067u32.to_be_bytes())?;
        let segment_size = out.stream_position()?;
        out.write_all(&size(0)?)?;
        let segment = out.stream_position()?;
        let seek = segment;
        out.write_all(&vec![0; 256])?;
        let info = out.stream_position()?;
        let prefix = [
            u(0x2ad7b1, 1_000_000),
            elem(0x4d80, b"media-webm"),
            elem(0x5741, b"media-webm"),
        ]
        .concat();
        let inf = elem(
            0x1549a966,
            &[prefix.clone(), elem(0x4489, &0f64.to_be_bytes())].concat(),
        );
        let duration = info + 12 + prefix.len() as u64 + 10;
        out.write_all(&inf)?;
        let colour = master(
            0x55b0,
            &[
                u(0x55b1, 1),
                u(0x55b2, 8),
                u(0x55b9, 1),
                u(0x55ba, 1),
                u(0x55bb, 1),
            ],
        );
        let mut entries = Vec::new();
        if let Some(video) = &video {
            entries.push(master(
                0xae,
                &[
                    u(0xd7, 1),
                    u(0x73c5, 1),
                    u(0x83, 1),
                    u(0x9c, 0),
                    elem(0x86, b"V_AV1"),
                    elem(0x63a2, &video.codec_private),
                    master(
                        0xe0,
                        &[
                            u(0xb0, video.width as u64),
                            u(0xba, video.height as u64),
                            colour,
                        ],
                    ),
                ],
            ));
        }
        if let Some(a) = &audio {
            entries.push(master(
                0xae,
                &[
                    u(0xd7, 2),
                    u(0x73c5, 2),
                    u(0x83, 2),
                    u(0x9c, 0),
                    elem(0x86, b"A_OPUS"),
                    elem(0x63a2, &a.head()),
                    u(0x56aa, a.pre_skip as u64 * 1_000_000_000 / 48000),
                    u(0x56bb, 80_000_000),
                    master(
                        0xe1,
                        &[
                            elem(0xb5, &48000f64.to_be_bytes()),
                            u(0x9f, a.channels as u64),
                        ],
                    ),
                ],
            ));
        }
        let tracks = out.stream_position()?;
        out.write_all(&master(0x1654ae6b, &entries))?;
        Ok(Self {
            out,
            segment,
            segment_size,
            seek,
            info,
            tracks,
            duration,
            cluster: None,
            cues: Vec::new(),
            end: 0,
            last: 0,
            audio: audio.is_some(),
            video: video.is_some(),
            discard_ns: 0,
            duration_ms: None,
            failed: false,
        })
    }
    /// Audio packet with terminal padding in 48 kHz samples per channel.
    pub fn opus_packet(
        &mut self,
        timestamp_ms: u64,
        data: &[u8],
        discard_samples: u16,
    ) -> Result<()> {
        self.discard_ns = (u64::from(discard_samples) * 1_000_000_000 + 24000) / 48000;
        self.packet(2, timestamp_ms, 20, true, data)
    }
    pub fn set_duration_samples(&mut self, samples: u64) {
        self.duration_ms = Some(samples as f64 / 48.0);
    }
    fn close_cluster(&mut self) -> Result<()> {
        if let Some((at, body, _)) = self.cluster.take() {
            let end = self.out.stream_position()?;
            self.out.seek(SeekFrom::Start(at + 4))?;
            self.out.write_all(&size(end - body)?)?;
            self.out.seek(SeekFrom::Start(end))?;
        }
        Ok(())
    }
    /// Interleave in nondecreasing millisecond timestamps. Duration is in ms.
    pub fn packet(
        &mut self,
        track: u8,
        timestamp: u64,
        duration: u64,
        key: bool,
        data: &[u8],
    ) -> Result<()> {
        if self.failed {
            return Err(Error::Invalid("writer failed; discard output"));
        }
        let result = self.write_packet(track, timestamp, duration, key, data);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn write_packet(&mut self, track: u8, t: u64, d: u64, key: bool, data: &[u8]) -> Result<()> {
        if !(track == 1 && self.video || track == 2 && self.audio)
            || t > i64::MAX as u64
            || track == 1 && self.cues.is_empty() && !key
            || t < self.last
            || data.is_empty()
            || data.len() as u64 >= (1 << 56) - 16
        {
            return Err(Error::Invalid("invalid packet or timestamp order"));
        }
        let end = t.checked_add(d).ok_or(Error::SizeOverflow)?;
        if self.cluster.is_none_or(|(_, _, base)| t - base >= 5000) || track == 1 && key {
            self.close_cluster()?;
            let at = self.out.stream_position()?;
            self.out.write_all(&0x1f43b675u32.to_be_bytes())?;
            self.out.write_all(&size(0)?)?;
            let body = self.out.stream_position()?;
            self.out.write_all(&u(0xe7, t))?;
            self.cluster = Some((at, body, t));
        }
        let (at, _, base) = self.cluster.ok_or(Error::Invalid("missing cluster"))?;
        if track == 1 && key || !self.video && base == t {
            self.cues.push((t, at - self.segment));
        }
        if self.discard_ns != 0 || track == 1 {
            let block = [
                &[0x80 | track][..],
                &((t - base) as i16).to_be_bytes(),
                &[0],
                data,
            ]
            .concat();
            let mut items = vec![elem(0xa1, &block), u(0x9b, d)];
            if track == 1 && !key {
                // RFC 9559 §5.1.3.5.5: zero means dependent, exact reference
                // blocks unknown. The packet API does not expose codec refs.
                items.push(elem(0xfb, &0i64.to_be_bytes()));
            }
            if self.discard_ns != 0 {
                items.push(elem(0x75a2, &(self.discard_ns as i64).to_be_bytes()));
            }
            self.out.write_all(&master(0xa0, &items))?;
            self.discard_ns = 0;
        } else {
            self.out.write_all(&[0xa3])?;
            self.out.write_all(&size(data.len() as u64 + 4)?)?;
            self.out.write_all(&[0x80 | track])?;
            self.out.write_all(&((t - base) as i16).to_be_bytes())?;
            self.out.write_all(&[if key { 0x80 } else { 0 }])?;
            self.out.write_all(data)?;
        }
        self.last = t;
        self.end = self.end.max(end);
        Ok(())
    }
    pub fn finish(mut self) -> Result<W> {
        if self.cues.is_empty() {
            return Err(Error::Invalid("video keyframe required"));
        }
        if self.failed {
            return Err(Error::Invalid("writer failed; discard output"));
        }
        self.close_cluster()?;
        let cues = self.out.stream_position()?;
        // Emit each CuePoint directly, retaining only compact keyframe positions.
        self.out.write_all(&0x1c53bb6bu32.to_be_bytes())?;
        let size_at = self.out.stream_position()?;
        self.out.write_all(&size(0)?)?;
        let body = self.out.stream_position()?;
        for &(time, pos) in &self.cues {
            self.out.write_all(&master(
                0xbb,
                &[
                    u(0xb3, time),
                    master(
                        0xb7,
                        &[u(0xf7, if self.video { 1 } else { 2 }), u(0xf1, pos)],
                    ),
                ],
            ))?;
        }
        let end = self.out.stream_position()?;
        self.out.seek(SeekFrom::Start(size_at))?;
        self.out.write_all(&size(end - body)?)?;
        self.out.seek(SeekFrom::Start(self.duration))?;
        self.out
            .write_all(&self.duration_ms.unwrap_or(self.end as f64).to_be_bytes())?;
        let entries = [
            (0x1549a966, self.info),
            (0x1654ae6b, self.tracks),
            (0x1c53bb6b, cues),
        ]
        .map(|(id, pos)| {
            master(
                0x4dbb,
                &[
                    elem(0x53ab, &(id as u32).to_be_bytes()),
                    u(0x53ac, pos - self.segment),
                ],
            )
        });
        let head = master(0x114d9b74, &entries);
        self.out.seek(SeekFrom::Start(self.seek))?;
        self.out.write_all(&head)?;
        self.out
            .write_all(&elem(0xec, &vec![0; 256 - head.len() - 9]))?;
        self.out.seek(SeekFrom::Start(self.segment_size))?;
        self.out.write_all(&size(end - self.segment)?)?;
        self.out.seek(SeekFrom::Start(end))?;
        self.out.flush()?;
        Ok(self.out)
    }
}

struct Bits<'a> {
    b: &'a [u8],
    p: usize,
}
impl Bits<'_> {
    fn get(&mut self, n: usize) -> Result<u32> {
        let mut v = 0;
        for _ in 0..n {
            let byte = self.b.get(self.p / 8).ok_or(Error::Truncated)?;
            v = (v << 1) | ((byte >> (7 - self.p % 8)) & 1) as u32;
            self.p += 1;
        }
        Ok(v)
    }
}
/// Build av1C from a low-overhead temporal unit's sequence header. This bounded
/// subset accepts the media-av1 contract: Main, 8-bit non-monochrome 4:2:0.
/// Timing/decoder-model streams are explicitly refused rather than guessed.
pub fn av1_config(packet: &[u8]) -> Result<Vec<u8>> {
    let mut p = 0;
    while p < packet.len() {
        let start = p;
        let h = packet[p];
        p += 1;
        if h & 0x83 != 2 {
            return Err(Error::Invalid(
                "OBU must have size, no forbidden/reserved bits",
            ));
        }
        if h & 4 != 0 {
            p = p.checked_add(1).ok_or(Error::SizeOverflow)?;
        }
        let mut len = 0u64;
        let mut done = false;
        for i in 0..8 {
            let b = *packet.get(p).ok_or(Error::Truncated)?;
            p += 1;
            len |= ((b & 127) as u64) << (7 * i);
            if b & 128 == 0 {
                done = true;
                break;
            }
        }
        if !done {
            return Err(Error::SizeOverflow);
        }
        let end = p
            .checked_add(usize::try_from(len).map_err(|_| Error::SizeOverflow)?)
            .ok_or(Error::SizeOverflow)?;
        let payload = packet.get(p..end).ok_or(Error::Truncated)?;
        if (h >> 3) & 15 == 1 {
            let mut bits = Bits { b: payload, p: 0 };
            let profile = bits.get(3)?;
            bits.get(1)?;
            let reduced = bits.get(1)?;
            if profile != 0 {
                return Err(Error::Invalid("only AV1 Main supported"));
            }
            let mut level = 0;
            let mut tier = 0;
            if reduced != 0 {
                level = bits.get(5)?;
            } else {
                if bits.get(1)? != 0 {
                    return Err(Error::Invalid("sequence timing not supported"));
                }
                let delay = bits.get(1)?;
                let ops = bits.get(5)? + 1;
                for i in 0..ops {
                    bits.get(12)?;
                    let l = bits.get(5)?;
                    let t = if l > 7 { bits.get(1)? } else { 0 };
                    if i == 0 {
                        level = l;
                        tier = t;
                    }
                    if delay != 0 && bits.get(1)? != 0 {
                        bits.get(4)?;
                    }
                }
            }
            let wb = bits.get(4)? + 1;
            let hb = bits.get(4)? + 1;
            bits.get(wb as usize)?;
            bits.get(hb as usize)?;
            if reduced == 0 && bits.get(1)? != 0 {
                bits.get(7)?;
            }
            bits.get(3)?;
            if reduced == 0 {
                bits.get(4)?;
                let order = bits.get(1)?;
                if order != 0 {
                    bits.get(2)?;
                }
                let screen = if bits.get(1)? != 0 { 2 } else { bits.get(1)? };
                if screen > 0 && bits.get(1)? == 0 {
                    bits.get(1)?;
                }
                if order != 0 {
                    bits.get(3)?;
                }
            }
            bits.get(3)?;
            if bits.get(1)? != 0 || bits.get(1)? != 0 {
                return Err(Error::Invalid("requires 8-bit non-monochrome"));
            }
            if bits.get(1)? != 0 && bits.get(24)? == 0x01_0d_00 {
                return Err(Error::Invalid("sRGB identity colour is not Main 4:2:0"));
            }
            bits.get(1)?;
            let chroma = bits.get(2)?;
            let mut config = vec![0x81, level as u8, (tier as u8) << 7 | 12 | chroma as u8, 0];
            config.extend_from_slice(&packet[start..end]);
            return Ok(config);
        }
        p = end;
    }
    Err(Error::Invalid("missing sequence header OBU"))
}
