//! Packed straight-alpha BGRA operations and centred BT.709 limited-range 420.
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bgra {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Yuv420 {
    pub width: usize,
    pub height: usize,
    pub y: Vec<u8>,
    pub u: Vec<u8>,
    pub v: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidFrame;
impl std::fmt::Display for InvalidFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid frame dimensions or storage")
    }
}
impl std::error::Error for InvalidFrame {}

fn size(w: usize, h: usize) -> Result<usize, InvalidFrame> {
    if w == 0 || h == 0 {
        return Err(InvalidFrame);
    }
    w.checked_mul(h)
        .and_then(|n| n.checked_mul(4))
        .ok_or(InvalidFrame)
}

impl Bgra {
    pub fn new(width: usize, height: usize, pixels: Vec<u8>) -> Result<Self, InvalidFrame> {
        if pixels.len() != size(width, height)? {
            return Err(InvalidFrame);
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }
    pub fn solid(width: usize, height: usize, colour: [u8; 4]) -> Result<Self, InvalidFrame> {
        let mut pixels = vec![0; size(width, height)?];
        for p in pixels.chunks_exact_mut(4) {
            p.copy_from_slice(&colour);
        }
        Self::new(width, height, pixels)
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
    /// Nearest-neighbour, top-left aligned integer mapping. Preserves UI colours.
    pub fn scale(&self, width: usize, height: usize) -> Result<Self, InvalidFrame> {
        let mut out = Self::solid(width, height, [0; 4])?;
        for y in 0..height {
            for x in 0..width {
                let src = ((y as u128 * self.height as u128 / height as u128) as usize
                    * self.width
                    + (x as u128 * self.width as u128 / width as u128) as usize)
                    * 4;
                out.pixels[(y * width + x) * 4..(y * width + x + 1) * 4]
                    .copy_from_slice(&self.pixels[src..src + 4]);
            }
        }
        Ok(out)
    }
    /// Separable Catmull-Rom bicubic (a=-0.5), pixel-centre aligned, clamped
    /// edges. Filters encoded/gamma-space BGRA channels, matching the usual
    /// video/UI scaling contract. Downsampling widens the kernel to antialias.
    /// Straight alpha is filtered independently; composite before scaling when
    /// transparent colour fringes are unwanted. Quantisation occurs only once.
    pub fn resample(&self, width: usize, height: usize) -> Result<Self, InvalidFrame> {
        size(width, height)?;
        if (width, height) == (self.width, self.height) { return Ok(self.clone()); }
        fn cubic(x: f64) -> f64 {
            let x = x.abs();
            if x < 1.0 { (1.5*x-2.5)*x*x+1.0 }
            else if x < 2.0 { ((-0.5*x+2.5)*x-4.0)*x+2.0 }
            else { 0.0 }
        }
        fn weights(src: usize, dst: usize) -> Vec<Vec<(usize,f32)>> {
            let scale = src as f64 / dst as f64;
            let stretch = scale.max(1.0);
            (0..dst).map(|i| {
                let centre = (i as f64+0.5)*scale-0.5;
                let lo = (centre-2.0*stretch).ceil() as isize;
                let hi = (centre+2.0*stretch).floor() as isize;
                let mut taps: Vec<_> = (lo..=hi).map(|j| ((j.clamp(0,src as isize-1)) as usize, cubic((j as f64-centre)/stretch) as f32)).collect();
                let sum: f32 = taps.iter().map(|(_,w)| w).sum();
                for (_, w) in &mut taps { *w /= sum; }
                taps
            }).collect()
        }
        let wx = weights(self.width,width);
        let wy = weights(self.height,height);
        let n = width.checked_mul(self.height).and_then(|n|n.checked_mul(4)).ok_or(InvalidFrame)?;
        let mut horizontal = Vec::new();
        horizontal.try_reserve_exact(n).map_err(|_|InvalidFrame)?;
        horizontal.resize(n,0.0f32);
        for y in 0..self.height {
            for (x,taps) in wx.iter().enumerate() {
                let out = &mut horizontal[(y*width+x)*4..(y*width+x+1)*4];
                for &(sx,w) in taps {
                    for (d,s) in out.iter_mut().zip(&self.pixels[(y*self.width+sx)*4..(y*self.width+sx+1)*4]) { *d += f32::from(*s)*w; }
                }
            }
        }
        let mut out = Self::solid(width,height,[0;4])?;
        let mut row = vec![0.0f32;width*4];
        for (y,taps) in wy.iter().enumerate() {
            row.fill(0.0);
            for &(sy,w) in taps {
                for (d,s) in row.iter_mut().zip(&horizontal[sy*width*4..(sy+1)*width*4]) { *d += s*w; }
            }
            for (d,s) in out.pixels[y*width*4..(y+1)*width*4].iter_mut().zip(&row) { *d = s.round().clamp(0.0,255.0) as u8; }
        }
        Ok(out)
    }
    pub fn pad(
        &self,
        width: usize,
        height: usize,
        x: usize,
        y: usize,
        colour: [u8; 4],
    ) -> Result<Self, InvalidFrame> {
        if x.checked_add(self.width).is_none_or(|v| v > width)
            || y.checked_add(self.height).is_none_or(|v| v > height)
        {
            return Err(InvalidFrame);
        }
        let mut out = Self::solid(width, height, colour)?;
        for row in 0..self.height {
            let dst = ((y + row) * width + x) * 4;
            out.pixels[dst..dst + self.width * 4]
                .copy_from_slice(&self.pixels[row * self.width * 4..(row + 1) * self.width * 4]);
        }
        Ok(out)
    }
    /// Straight-alpha source-over, same-sized frames; rounded integer arithmetic.
    pub fn blend(&self, foreground: &Self) -> Result<Self, InvalidFrame> {
        if (self.width, self.height) != (foreground.width, foreground.height) {
            return Err(InvalidFrame);
        }
        let mut out = self.clone();
        for ((b, f), o) in self
            .pixels
            .chunks_exact(4)
            .zip(foreground.pixels.chunks_exact(4))
            .zip(out.pixels.chunks_exact_mut(4))
        {
            let a = u32::from(f[3]);
            let ba = u32::from(b[3]);
            let den = a * 255 + ba * (255 - a);
            for c in 0..3 {
                o[c] = (u32::from(f[c]) * a * 255 + u32::from(b[c]) * ba * (255 - a) + den / 2)
                    .checked_div(den)
                    .unwrap_or(0) as u8;
            }
            o[3] = ((den + 127) / 255) as u8;
        }
        Ok(out)
    }
    /// Linear interpolation in encoded BGRA, weight 0..=255 (not linear light).
    pub fn crossfade(&self, other: &Self, weight: u8) -> Result<Self, InvalidFrame> {
        if (self.width, self.height) != (other.width, other.height) {
            return Err(InvalidFrame);
        }
        let mut out = self.clone();
        let w = u32::from(weight);
        for ((a, b), o) in self.pixels.iter().zip(&other.pixels).zip(&mut out.pixels) {
            *o = ((u32::from(*a) * (255 - w) + u32::from(*b) * w + 127) / 255) as u8;
        }
        Ok(out)
    }
    /// Alpha is ignored: composite onto the intended background before conversion.
    /// Chroma is a 2x2 box at (0.5,0.5); odd edges replicate the last pixel.
    pub fn to_yuv420(&self) -> Yuv420 {
        let mut out = Yuv420 {
            width: self.width,
            height: self.height,
            y: Vec::with_capacity(self.width * self.height),
            u: Vec::new(),
            v: Vec::new(),
        };
        fn matrix(r: f64, g: f64, b: f64) -> [f64; 3] {
            [
                16.0 + (0.2126 * r + 0.7152 * g + 0.0722 * b) * 219.0 / 255.0,
                128.0 + (-0.114572106 * r - 0.385427894 * g + 0.5 * b) * 224.0 / 255.0,
                128.0 + (0.5 * r - 0.454152908 * g - 0.045847092 * b) * 224.0 / 255.0,
            ]
        }
        for p in self.pixels.chunks_exact(4) {
            out.y
                .push(matrix(p[2] as f64, p[1] as f64, p[0] as f64)[0].round() as u8);
        }
        for y in (0..self.height).step_by(2) {
            for x in (0..self.width).step_by(2) {
                let mut rgb = [0.0; 3];
                for dy in 0..2 {
                    for dx in 0..2 {
                        let i = (((y + dy).min(self.height - 1)) * self.width
                            + (x + dx).min(self.width - 1))
                            * 4;
                        for (c, value) in rgb.iter_mut().enumerate() {
                            *value += self.pixels[i + 2 - c] as f64 / 4.0;
                        }
                    }
                }
                let m = matrix(rgb[0], rgb[1], rgb[2]);
                out.u.push(m[1].round() as u8);
                out.v.push(m[2].round() as u8);
            }
        }
        out
    }
}

impl Yuv420 {
    pub fn validate(&self) -> Result<(), InvalidFrame> {
        size(self.width, self.height)?;
        let c = self.width.div_ceil(2) * self.height.div_ceil(2);
        if self.y.len() != self.width * self.height || self.u.len() != c || self.v.len() != c {
            return Err(InvalidFrame);
        }
        Ok(())
    }
}

/// A hold shares immutable storage; duration/timestamps belong to the caller.
pub fn still_hold(frame: Arc<Bgra>, count: usize) -> impl Iterator<Item = Arc<Bgra>> {
    std::iter::repeat_n(frame, count)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bicubic_constants_identity_ramps_and_antialias() {
        let flat = Bgra::solid(7,5,[17,91,203,255]).unwrap();
        assert_eq!(flat.resample(7,5).unwrap(),flat);
        for (w,h) in [(13,11),(1,1),(3,2)] { assert_eq!(flat.resample(w,h).unwrap(),Bgra::solid(w,h,[17,91,203,255]).unwrap()); }
        let ramp = Bgra::new(4,1,vec![[0,0,0,255],[64,64,64,255],[128,128,128,255],[192,192,192,255]].into_iter().flatten().collect()).unwrap();
        let smooth = ramp.resample(8,1).unwrap();
        assert!(smooth.pixels()[4] > 0 && smooth.pixels()[4] < 64);
        let board = Bgra::new(64,1,(0..64).flat_map(|i| [if i%2==0 {0} else {255};4]).collect()).unwrap().resample(1,1).unwrap();
        assert!((i16::from(board.pixels()[0])-128).abs() <= 1);
        assert!(flat.resample(0,2).is_err());
    }
    #[test]
    fn primaries_and_odd_edges() {
        for (bgra, y, u, v) in [
            ([0, 0, 0, 255], 16, 128, 128),
            ([255, 255, 255, 255], 235, 128, 128),
            ([0, 0, 255, 255], 63, 102, 240),
            ([0, 255, 0, 255], 173, 42, 26),
            ([255, 0, 0, 255], 32, 240, 118),
        ] {
            let f = Bgra::solid(3, 1, bgra).unwrap().to_yuv420();
            assert_eq!(f.y, vec![y; 3]);
            assert_eq!(f.u, vec![u; 2]);
            assert_eq!(f.v, vec![v; 2]);
            f.validate().unwrap();
        }
    }
    #[test]
    fn odd_edge_replicates_the_last_pixel() {
        // black, black, red: the second chroma sample is red alone, not red
        // averaged with a wrapped or zero neighbour.
        let f = Bgra::new(3, 1, vec![0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 255, 255])
            .unwrap()
            .to_yuv420();
        assert_eq!(f.y, [16, 16, 63]);
        assert_eq!(f.u, [128, 102]);
        assert_eq!(f.v, [128, 240]);
        // The same column stood on end pins the last row for odd heights.
        let g = Bgra::new(1, 3, vec![0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 255, 255])
            .unwrap()
            .to_yuv420();
        assert_eq!(g.y, [16, 16, 63]);
        assert_eq!(g.u, [128, 102]);
        assert_eq!(g.v, [128, 240]);
    }
    #[test]
    fn box_filter() {
        let f = Bgra::new(
            2,
            2,
            vec![
                0, 0, 255, 255, 0, 255, 0, 255, 255, 0, 0, 255, 255, 255, 255, 255,
            ],
        )
        .unwrap()
        .to_yuv420();
        assert_eq!(f.y, [63, 173, 32, 235]);
        assert_eq!(f.u, [128]);
        assert_eq!(f.v, [128]);
    }
    #[test]
    fn operations() {
        let b = Bgra::solid(1, 1, [0, 0, 0, 255]).unwrap();
        let f = Bgra::solid(1, 1, [255, 0, 0, 128]).unwrap();
        assert_eq!(b.blend(&f).unwrap().pixels(), [128, 0, 0, 255]);
        assert_eq!(b.crossfade(&f, 255).unwrap(), f);
        assert_eq!(b.crossfade(&f, 0).unwrap(), b);
        assert_eq!(b.crossfade(&f, 128).unwrap().pixels(), [128, 0, 0, 191]);
        assert_eq!(
            f.scale(2, 1).unwrap().pixels(),
            [255, 0, 0, 128, 255, 0, 0, 128]
        );
        assert_eq!(
            f.pad(2, 1, 1, 0, [0; 4]).unwrap().pixels(),
            [0, 0, 0, 0, 255, 0, 0, 128]
        );
        let a = Arc::new(f);
        assert!(still_hold(a.clone(), 2).all(|f| Arc::ptr_eq(&a, &f)));
        assert!(b.pad(1, 1, 1, 0, [0; 4]).is_err());
        assert!(Bgra::new(0, 1, vec![]).is_err());
    }
    #[test]
    fn nonuniform_geometry_and_transparency() {
        let f = Bgra::new(
            2,
            2,
            vec![1, 0, 0, 255, 2, 0, 0, 255, 3, 0, 0, 255, 4, 0, 0, 255],
        )
        .unwrap();
        let scaled = f.scale(3, 3).unwrap();
        assert_eq!(
            scaled
                .pixels()
                .chunks_exact(4)
                .map(|p| p[0])
                .collect::<Vec<_>>(),
            [1, 1, 2, 1, 1, 2, 3, 3, 4]
        );
        let padded = f.pad(3, 3, 1, 1, [0; 4]).unwrap();
        assert_eq!(
            padded
                .pixels()
                .chunks_exact(4)
                .map(|p| p[0])
                .collect::<Vec<_>>(),
            [0, 0, 0, 0, 1, 2, 0, 3, 4]
        );
        let clear = Bgra::solid(1, 1, [99, 88, 77, 0]).unwrap();
        assert_eq!(clear.blend(&clear).unwrap().pixels(), [0; 4]);
        let a = Bgra::solid(1, 1, [0, 0, 0, 128]).unwrap();
        let b = Bgra::solid(1, 1, [255, 0, 0, 128]).unwrap();
        assert_eq!(a.blend(&b).unwrap().pixels(), [170, 0, 0, 192]);
    }
}
