//! Minimal streaming Y4M reader: only 8-bit 420, preserving odd dimensions.
use media_frame::Yuv420;
use std::io::{self, BufRead};
pub struct Reader<R> { input: R, width: usize, height: usize }
fn bad() -> io::Error { io::Error::new(io::ErrorKind::InvalidData, "invalid or unsupported Y4M (need 8-bit 420)") }
fn line(r: &mut impl BufRead) -> io::Result<String> {
    let mut b = Vec::new();
    r.take(4097).read_until(b'\n', &mut b)?;
    if b.len() > 4096 || (!b.is_empty() && b.last() != Some(&b'\n')) { return Err(bad()); }
    String::from_utf8(b).map_err(|_| bad())
}
impl<R: BufRead> Reader<R> {
    pub fn new(mut input: R) -> io::Result<Self> {
        let header = line(&mut input)?;
        let mut fields = header.split_whitespace();
        if fields.next() != Some("YUV4MPEG2") { return Err(bad()); }
        let (mut width, mut height) = (0usize,0usize);
        for field in fields {
            if let Some(v) = field.strip_prefix('W') { width = v.parse().map_err(|_| bad())?; }
            if let Some(v) = field.strip_prefix('H') { height = v.parse().map_err(|_| bad())?; }
            if field.starts_with('C') && !["C420", "C420jpeg", "C420mpeg2", "C420paldv"].contains(&field) { return Err(bad()); }
            if field.starts_with('I') && field != "Ip" && field != "I?" { return Err(bad()); }
        }
        if width == 0 || height == 0 || width.checked_mul(height).and_then(|n| n.checked_mul(4)).is_none() { return Err(bad()); }
        Ok(Self { input,width,height })
    }
    pub fn next_frame(&mut self) -> io::Result<Option<Yuv420>> {
        let header = line(&mut self.input)?;
        if header.is_empty() { return Ok(None); }
        if header.split_whitespace().next() != Some("FRAME") { return Err(bad()); }
        let mut plane = |n| -> io::Result<Vec<u8>> {
            let mut v = Vec::new();
            v.try_reserve_exact(n).map_err(|_| bad())?;
            v.resize(n,0);
            self.input.read_exact(&mut v)?;
            Ok(v)
        };
        let c = self.width.div_ceil(2)*self.height.div_ceil(2);
        Ok(Some(Yuv420 { width:self.width,height:self.height,y:plane(self.width*self.height)?,u:plane(c)?,v:plane(c)? }))
    }
}
