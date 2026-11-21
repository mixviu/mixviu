//! AV1 Main 8-bit 4:2:0 + optional Opus WebM. Timestamps are container ticks.
mod reader;
mod writer;
pub use reader::{Cue, Demuxer, Packet, Track};
pub use writer::{Opus, Video, Writer, av1_config};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Truncated,
    SizeOverflow,
    UnsupportedLacing,
    Invalid(&'static str),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
type Result<T> = std::result::Result<T, Error>;

fn vint(b: &[u8], p: &mut usize, id: bool) -> Result<(u64, bool)> {
    let first = *b.get(*p).ok_or(Error::Truncated)?;
    let n = first.leading_zeros() as usize + 1;
    if n > if id { 4 } else { 8 } {
        return Err(Error::SizeOverflow);
    }
    let end = p.checked_add(n).ok_or(Error::SizeOverflow)?;
    let bytes = b.get(*p..end).ok_or(Error::Truncated)?;
    let mut v = if id {
        first as u64
    } else {
        (first as u64) & (0xffu64 >> n)
    };
    for byte in &bytes[1..] {
        v = (v << 8) | *byte as u64;
    }
    *p = end;
    Ok((v, !id && v == (1u64 << (7 * n)) - 1))
}
struct Element<'a> {
    id: u64,
    data: &'a [u8],
    start: usize,
}
fn element<'a>(b: &'a [u8], p: &mut usize) -> Result<Element<'a>> {
    let start = *p;
    let (id, _) = vint(b, p, true)?;
    let (size, unknown) = vint(b, p, false)?;
    let body = *p;
    if unknown && id != 0x18538067 {
        return Err(Error::Invalid("unknown size only supported for Segment"));
    }
    let end = if unknown {
        b.len()
    } else {
        body.checked_add(usize::try_from(size).map_err(|_| Error::SizeOverflow)?)
            .ok_or(Error::SizeOverflow)?
    };
    let data = b.get(body..end).ok_or(Error::Truncated)?;
    *p = end;
    Ok(Element { id, data, start })
}
fn uint(b: &[u8]) -> Result<u64> {
    if b.len() > 8 {
        return Err(Error::SizeOverflow);
    }
    Ok(b.iter().fold(0, |v, b| (v << 8) | *b as u64))
}
fn children(b: &[u8]) -> impl Iterator<Item = Result<Element<'_>>> {
    let mut p = 0;
    std::iter::from_fn(move || {
        if p == b.len() {
            return None;
        }
        let r = element(b, &mut p);
        if r.is_err() {
            p = b.len();
        }
        Some(r)
    })
}
