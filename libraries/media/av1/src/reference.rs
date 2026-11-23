//! Bounded AV1 header reader for the encoder's low latency Main contract.
//! Reads refresh flags from the bitstream, never derives slots from input indices.
use crate::Error;
type Result<T> = std::result::Result<T, Error>;
fn unknown() -> Error {
    Error("unknown_reference_state".into())
}
struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
}
impl Bits<'_> {
    fn get(&mut self, n: usize) -> Result<u32> {
        let mut v = 0;
        for _ in 0..n {
            let b = self.data.get(self.pos / 8).ok_or_else(unknown)?;
            v = v * 2 + u32::from((b >> (7 - self.pos % 8)) & 1);
            self.pos += 1;
        }
        Ok(v)
    }
}
fn obus(data: &[u8]) -> Result<Vec<(u8, &[u8])>> {
    let mut out = Vec::new();
    let mut p = 0;
    while p < data.len() {
        let h = data[p];
        p += 1;
        if h & 0x87 != 2 {
            return Err(unknown());
        }
        let mut n = 0usize;
        let mut done = false;
        for i in 0..8 {
            let b = *data.get(p).ok_or_else(unknown)?;
            p += 1;
            n |= usize::from(b & 127)
                .checked_shl(i * 7)
                .ok_or_else(unknown)?;
            if b & 128 == 0 {
                done = true;
                break;
            }
        }
        if !done {
            return Err(unknown());
        }
        let end = p.checked_add(n).ok_or_else(unknown)?;
        out.push(((h >> 3) & 15, data.get(p..end).ok_or_else(unknown)?));
        p = end;
    }
    Ok(out)
}
#[derive(Default)]
pub struct ReferenceTracker {
    sequence: Option<Vec<u8>>,
    order_bits: usize,
    screen: u32,
    integer: u32,
    slot: Option<u8>,
}
impl ReferenceTracker {
    pub fn observe(&mut self, packet: &[u8]) -> Result<bool> {
        let mut frame = false;
        let mut key = false;
        for (kind, payload) in obus(packet)? {
            let mut b = Bits {
                data: payload,
                pos: 0,
            };
            if kind == 1 {
                if let Some(previous) = &self.sequence {
                    if previous != payload {
                        return Err(Error("sequence_header_mismatch".into()));
                    }
                    continue;
                }
                if b.get(5)? != 0
                    || b.get(1)? != 0
                    || b.get(1)? != 0
                    || b.get(5)? != 0
                    || b.get(12)? != 0
                {
                    return Err(unknown());
                }
                if b.get(5)? > 7 {
                    b.get(1)?;
                }
                let w = b.get(4)? as usize + 1;
                let h = b.get(4)? as usize + 1;
                b.get(w)?;
                b.get(h)?;
                if b.get(1)? != 0 {
                    return Err(unknown());
                } // frame IDs
                b.get(3)?;
                b.get(4)?;
                let order = b.get(1)?;
                if order != 0 {
                    b.get(2)?;
                }
                self.screen = if b.get(1)? != 0 { 2 } else { b.get(1)? };
                self.integer = if self.screen > 0 && b.get(1)? == 0 {
                    b.get(1)?
                } else {
                    2
                };
                self.order_bits = if order != 0 {
                    b.get(3)? as usize + 1
                } else {
                    0
                };
                self.sequence = Some(payload.to_vec());
            } else if kind == 3 || kind == 6 {
                if frame || self.sequence.is_none() || b.get(1)? != 0 {
                    return Err(unknown());
                }
                frame = true;
                let ty = b.get(2)?;
                if ty > 1 || b.get(1)? != 1 {
                    return Err(unknown());
                } // shown KEY or INTER only
                key = ty == 0;
                let resilient = if key { 1 } else { b.get(1)? };
                b.get(1)?;
                let screen = if self.screen == 2 {
                    b.get(1)?
                } else {
                    self.screen
                };
                if screen > 0 && self.integer == 2 {
                    b.get(1)?;
                }
                b.get(1)?; // frame_size_override
                b.get(self.order_bits)?;
                if !key && resilient == 0 {
                    b.get(3)?;
                }
                if key {
                    self.slot = None;
                } else {
                    let refresh = b.get(8)?;
                    if refresh == 0 {
                        return Err(unknown());
                    }
                    self.slot = Some(refresh.trailing_zeros() as u8);
                }
            }
        }
        if !frame {
            return Err(unknown());
        }
        Ok(key)
    }
    pub fn repeat(&self) -> Result<Vec<u8>> {
        let slot = self.slot.ok_or_else(unknown)?;
        Ok(vec![0x12, 0, 0x1a, 1, 0x88 | (slot << 4)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn emitted_headers_determine_slots_and_sequence() {
        fn packets(width: usize) -> Vec<crate::Packet> {
            let mut e = crate::Encoder::new(crate::Settings {
                width,
                height: 32,
                fps_num: 30,
                fps_den: 1,
                speed: 10,
                target: crate::Target::Quality(100),
                key_interval: 1000,
                tile_cols: 1,
                tile_rows: 1,
                threads: 1,
            })
            .unwrap();
            let f = e
                .prepare(
                    &media_frame::Bgra::solid(width, 32, [80, 90, 110, 255])
                        .unwrap()
                        .to_yuv420(),
                )
                .unwrap();
            let mut out = Vec::new();
            for _ in 0..12 {
                out.extend(e.submit(&f).unwrap());
            }
            out.extend(e.finish().unwrap());
            out
        }
        let p = packets(32);
        let mut t = ReferenceTracker::default();
        assert!(t.observe(&p[0].data).unwrap());
        assert!(t.repeat().is_err());
        for packet in &p[1..] {
            assert!(!t.observe(&packet.data).unwrap());
            assert_eq!(t.repeat().unwrap().len(), 5);
        }
        assert_eq!(
            t.observe(&packets(64)[0].data).unwrap_err().0,
            "sequence_header_mismatch"
        );
    }
    #[test]
    fn unknown_state_refused() {
        let mut t = ReferenceTracker::default();
        assert_eq!(t.repeat().unwrap_err().0, "unknown_reference_state");
        assert!(t.observe(&[0x32, 1, 0]).is_err());
        assert!(t.observe(&[0xff]).is_err());
    }
}
