//! 8-bit planar 4:2:0 quality metrics matching FFmpeg n9.1 vf_ssim/vf_psnr.
//! No conversion, range expansion or chroma resampling is performed.
use media_frame::Yuv420;
pub mod y4m;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    SizeMismatch,
    InvalidFormat,
    /// FFmpeg divides by zero for planes without an 8x8 window. We refuse them.
    NoSsimWindow,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SizeMismatch => "frame dimensions differ",
            Self::InvalidFormat => "invalid 8-bit YUV420 frame storage",
            Self::NoSsimWindow => "SSIM needs at least 8x8 in every plane",
        })
    }
}
impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ssim { pub y: f64, pub u: f64, pub v: f64, pub all: f64 }

/// Positive infinity is explicit, including when only one plane is identical.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Decibels { Finite(f64), Infinite }
impl std::fmt::Display for Decibels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Self::Finite(v) => write!(f, "{v:.6}"), Self::Infinite => f.write_str("inf") }
    }
}
fn db(mse: f64) -> Decibels {
    if mse == 0.0 { Decibels::Infinite } else { Decibels::Finite(10.0 * (65025.0 / mse).log10()) }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Psnr {
    pub y: Decibels, pub u: Decibels, pub v: Decibels, pub all: Decibels,
    pub mse_y: f64, pub mse_u: f64, pub mse_v: f64, pub mse_all: f64,
}
fn from_mse(m: [f64; 4]) -> Psnr {
    Psnr { y: db(m[0]), u: db(m[1]), v: db(m[2]), all: db(m[3]),
        mse_y: m[0], mse_u: m[1], mse_v: m[2], mse_all: m[3] }
}
fn validate(a: &Yuv420, b: &Yuv420) -> Result<(), Error> {
    if (a.width, a.height) != (b.width, b.height) { return Err(Error::SizeMismatch); }
    // validate() checks overflow before computing plane sizes.
    a.validate().map_err(|_| Error::InvalidFormat)?;
    b.validate().map_err(|_| Error::InvalidFormat)?;
    Ok(())
}
fn weighted(f: &Yuv420, p: [f64; 3]) -> f64 {
    let total = f.y.len() as f64 + f.u.len() as f64 + f.v.len() as f64;
    p[0] * (f.y.len() as f64 / total) + p[1] * (f.u.len() as f64 / total)
        + p[2] * (f.v.len() as f64 / total)
}
pub fn psnr(a: &Yuv420, b: &Yuv420) -> Result<Psnr, Error> {
    validate(a, b)?;
    let mut m = [0.0; 4];
    for (i, (x, y)) in [(&a.y, &b.y), (&a.u, &b.u), (&a.v, &b.v)].into_iter().enumerate() {
        // u128 prevents overflow even for the largest addressable slice.
        let sum: u128 = x.iter().zip(y).map(|(&x, &y)| { let d = i32::from(x) - i32::from(y); (d*d) as u128 }).sum();
        m[i] = sum as f64 / x.len() as f64;
    }
    m[3] = weighted(a, [m[0], m[1], m[2]]);
    Ok(from_mse(m))
}

fn plane_ssim(a: &[u8], b: &[u8], w: usize, h: usize) -> f64 {
    let cols = w / 4;
    let rows = h / 4;
    let mut prev = vec![[0i32; 4]; cols];
    let mut curr = prev.clone();
    let mut score = 0.0;
    for by in 0..rows {
        for (bx, s) in curr.iter_mut().enumerate() {
            *s = [0; 4];
            for dy in 0..4 {
                let at = (by * 4 + dy) * w + bx * 4;
                for dx in 0..4 {
                    let x = i32::from(a[at + dx]);
                    let y = i32::from(b[at + dx]);
                    s[0] += x; s[1] += y; s[2] += x*x + y*y; s[3] += x*y;
                }
            }
        }
        if by > 0 {
            for bx in 0..cols-1 {
                let s: [i32; 4] = std::array::from_fn(|i| prev[bx][i] + prev[bx+1][i] + curr[bx][i] + curr[bx+1][i]);
                let [x,y,ss,xy] = s;
                let vars = ss*64 - x*x - y*y;
                let covar = xy*64 - x*y;
                // FFmpeg's rounded constants and f32 intermediate products.
                score += ((2*x*y + 416) as f32 * (2*covar + 235963) as f32
                    / ((x*x + y*y + 416) as f32 * (vars + 235963) as f32)) as f64;
            }
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    score / ((cols-1)*(rows-1)) as f64
}
pub fn ssim(a: &Yuv420, b: &Yuv420) -> Result<Ssim, Error> {
    validate(a, b)?;
    let (cw,ch) = (a.width.div_ceil(2), a.height.div_ceil(2));
    if cw < 8 || ch < 8 { return Err(Error::NoSsimWindow); }
    let y = plane_ssim(&a.y, &b.y, a.width, a.height);
    let u = plane_ssim(&a.u, &b.u, cw, ch);
    let v = plane_ssim(&a.v, &b.v, cw, ch);
    Ok(Ssim { y,u,v,all: weighted(a, [y,u,v]) })
}

/// Constant storage sequence accumulator. Failed updates leave it unchanged.
/// PSNR mean is computed from mean MSE, as in FFmpeg, not mean dB.
#[derive(Default, Debug)]
pub struct Sequence { count: u64, s: [f64;4], m: [f64;4], min_s: Option<Ssim>, min_p: Option<Psnr> }
#[derive(Clone, Copy, Debug)]
pub struct Summary { pub frames: u64, pub mean_ssim: Ssim, pub min_ssim: Ssim, pub mean_psnr: Psnr, pub min_psnr: Psnr }
impl Sequence {
    pub fn update(&mut self, a: &Yuv420, b: &Yuv420) -> Result<(Ssim,Psnr), Error> {
        let s = ssim(a,b)?;
        let p = psnr(a,b)?;
        self.count = self.count.saturating_add(1);
        let n = self.count as f64;
        for (i,v) in [s.y,s.u,s.v,s.all].into_iter().enumerate() { self.s[i] += (v-self.s[i])/n; }
        for (i,v) in [p.mse_y,p.mse_u,p.mse_v,p.mse_all].into_iter().enumerate() { self.m[i] += (v-self.m[i])/n; }
        self.min_s = Some(match self.min_s { None => s, Some(m) => Ssim { y:m.y.min(s.y), u:m.u.min(s.u), v:m.v.min(s.v), all:m.all.min(s.all) } });
        self.min_p = Some(match self.min_p { None => p, Some(m) => from_mse([m.mse_y.max(p.mse_y), m.mse_u.max(p.mse_u), m.mse_v.max(p.mse_v), m.mse_all.max(p.mse_all)]) });
        Ok((s,p))
    }
    pub fn summary(&self) -> Option<Summary> {
        Some(Summary { frames:self.count, mean_ssim:Ssim { y:self.s[0],u:self.s[1],v:self.s[2],all:self.s[3] }, min_ssim:self.min_s?, mean_psnr:from_mse(self.m), min_psnr:self.min_p? })
    }
}
