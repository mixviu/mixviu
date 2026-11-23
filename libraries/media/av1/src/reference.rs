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
        // Failure must also invalidate any previously shown reference.
        let result = self.observe_inner(packet);
        if result.is_err() { self.slot = None; }
        result
    }
    fn observe_inner(&mut self, packet: &[u8]) -> Result<bool> {
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
                // AV1 sequence_header_obu: enable_order_hint gates this field,
                // independently of seq_force_integer_mv. See normative syntax:
                // https://aomediacodec.github.io/av1-spec/#sequence-header-obu-syntax
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
                // Refuse primary-reference / short-signaling paths outright.
                // new_repeats() pins rav1e's error_resilient setting explicitly.
                if resilient == 0 { return Err(unknown()); }
                b.get(1)?;
                let screen = if self.screen == 2 {
                    b.get(1)?
                } else {
                    self.screen
                };
                if screen > 0 && self.integer == 2 {
                    b.get(1)?;
                }
                if b.get(1)? != 0 { return Err(unknown()); } // frame_size_override
                b.get(self.order_bits)?;
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
    fn bits(fields: &[(u32, usize)]) -> Vec<u8> {
        let mut data = vec![0; fields.iter().map(|(_, n)| n).sum::<usize>().div_ceil(8)];
        let mut pos = 0;
        for &(v, n) in fields {
            for i in (0..n).rev() {
                data[pos / 8] |= (((v >> i) & 1) as u8) << (7 - pos % 8);
                pos += 1;
            }
        }
        data
    }
    fn sequence(order: bool) -> Vec<u8> {
        // Exact sequence_header_obu prefix through OrderHintBits. The integer
        // MV value is zero in both cases: it does not gate order-hint bits.
        let mut f = vec![(0,5),(0,1),(0,1),(0,5),(0,12),(0,5),
            (0,4),(0,4),(0,1),(0,1),(0,1),(0,3),(0,4),(order as u32,1)];
        if order { f.push((0,2)); }
        f.extend([(0,1),(1,1),(0,1),(0,1)]);
        if order { f.push((2,3)); }
        let payload = bits(&f);
        let mut obu = vec![0x0a, payload.len() as u8];
        obu.extend(payload);
        obu
    }
    fn inter(order: bool, resilient: bool, size_override: bool, short: bool) -> Vec<u8> {
        let mut f = vec![(0,1),(1,2),(1,1),(resilient as u32,1),(0,1),(size_override as u32,1)];
        if order { f.push((0,3)); }
        if !resilient { f.push((0,3)); } // primary_ref_frame
        f.push((4,8)); // refresh_frame_flags
        // These fields follow the refresh mask in AV1, not precede it.
        if order { f.extend([(0,24),(short as u32,1)]); }
        if short { f.extend([(0,3),(1,3)]); } else { f.push((0,21)); }
        let payload = bits(&f);
        let mut obu = vec![0x32, payload.len() as u8];
        obu.extend(payload);
        obu
    }
    #[test]
    fn exact_prefix_conditions_and_refused_header_shapes() {
        for order in [false, true] {
            let mut t = ReferenceTracker::default();
            // A sequence alone fails the packet contract, but records its prefix.
            assert!(t.observe(&sequence(order)).is_err());
            assert_eq!(t.order_bits, if order { 3 } else { 0 });
            assert!(!t.observe(&inter(order, true, false, false)).unwrap());
            assert_eq!(t.repeat().unwrap(), [0x12,0,0x1a,1,0xa8]);
            for short in [false,true] {
                assert_eq!(t.observe(&inter(order,false,false,short)).unwrap_err().0, "unknown_reference_state");
                assert!(t.repeat().is_err());
            }
            assert_eq!(t.observe(&inter(order,true,true,false)).unwrap_err().0, "unknown_reference_state");
            assert!(t.repeat().is_err());
        }
    }
    #[test]
    fn emitted_headers_determine_slots_and_sequence() {
        fn packets(width: usize) -> Vec<crate::Packet> {
            let mut e = crate::Encoder::new_repeats(crate::Settings {
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
