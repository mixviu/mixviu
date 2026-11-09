// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The colour maths the HDR shaders run, in Rust: the transfer functions,
//! primaries matrices and the BT.2390 tone-mapping curve. The GLSL in
//! [`super::glsl`] is a transcription of these; the tests here are the oracle
//! for both, so a shader change is checked against the same numbers.

/// SMPTE ST 2084 (PQ) constants.
const M1: f64 = 2610.0 / 16384.0;
const M2: f64 = 2523.0 / 4096.0 * 128.0;
const C1: f64 = 3424.0 / 4096.0;
const C2: f64 = 2413.0 / 4096.0 * 32.0;
const C3: f64 = 2392.0 / 4096.0 * 32.0;

/// PQ code value (0..1) to absolute luminance in cd/m².
pub fn pq_eotf(e: f64) -> f64 {
    let e = e.clamp(0.0, 1.0);
    let p = e.powf(1.0 / M2);
    let num = (p - C1).max(0.0);
    let den = C2 - C3 * p;
    10000.0 * (num / den).powf(1.0 / M1)
}

/// Absolute luminance in cd/m² to a PQ code value (0..1).
pub fn pq_inv_eotf(nits: f64) -> f64 {
    let y = (nits / 10000.0).clamp(0.0, 1.0);
    let p = y.powf(M1);
    ((C1 + C2 * p) / (1.0 + C3 * p)).powf(M2)
}

/// ARIB STD-B67 (HLG) constants.
const HLG_A: f64 = 0.178_832_77;
const HLG_B: f64 = 0.284_668_92;
const HLG_C: f64 = 0.559_910_73;

/// HLG signal (0..1) to normalised scene light (0..1): the inverse OETF.
pub fn hlg_inv_oetf(e: f64) -> f64 {
    let e = e.clamp(0.0, 1.0);
    if e <= 0.5 {
        e * e / 3.0
    } else {
        (((e - HLG_C) / HLG_A).exp() + HLG_B) / 12.0
    }
}

/// Sign-preserving power, so extended-range (negative, out-of-gamut) values
/// survive a transfer function the way scRGB and KWin's blending space need.
pub fn spow(x: f64, p: f64) -> f64 {
    x.signum() * x.abs().powf(p)
}

/// A colour space's primaries and white point, CIE 1931 xy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chromaticity {
    pub r: (f64, f64),
    pub g: (f64, f64),
    pub b: (f64, f64),
    pub w: (f64, f64),
}

pub const D65: (f64, f64) = (0.3127, 0.3290);

pub const BT709: Chromaticity = Chromaticity {
    r: (0.640, 0.330),
    g: (0.300, 0.600),
    b: (0.150, 0.060),
    w: D65,
};

pub const DISPLAY_P3: Chromaticity = Chromaticity {
    r: (0.680, 0.320),
    g: (0.265, 0.690),
    b: (0.150, 0.060),
    w: D65,
};

pub const BT2020: Chromaticity = Chromaticity {
    r: (0.708, 0.292),
    g: (0.170, 0.797),
    b: (0.131, 0.046),
    w: D65,
};

/// A row-major 3x3.
pub type Mat3 = [[f64; 3]; 3];

pub const IDENTITY: Mat3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

fn xyz((x, y): (f64, f64)) -> [f64; 3] {
    [x / y, 1.0, (1.0 - x - y) / y]
}

pub fn mul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut out = [[0.0; 3]; 3];
    for (r, row) in out.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            *cell = (0..3).map(|k| a[r][k] * b[k][c]).sum();
        }
    }
    out
}

pub fn apply(m: &Mat3, v: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|r| m[r][0] * v[0] + m[r][1] * v[1] + m[r][2] * v[2])
}

pub fn invert(m: &Mat3) -> Option<Mat3> {
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if det.abs() < 1e-12 {
        return None;
    }
    let mut out = [[0.0; 3]; 3];
    for (r, row) in out.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            // Adjugate: the transposed cofactor matrix.
            let rows: Vec<usize> = (0..3).filter(|&i| i != c).collect();
            let cols: Vec<usize> = (0..3).filter(|&i| i != r).collect();
            let minor = m[rows[0]][cols[0]] * m[rows[1]][cols[1]]
                - m[rows[0]][cols[1]] * m[rows[1]][cols[0]];
            let sign = if (r + c) % 2 == 0 { 1.0 } else { -1.0 };
            *cell = sign * minor / det;
        }
    }
    Some(out)
}

/// Linear RGB in `c` to CIE XYZ (Y = 1 at white).
pub fn rgb_to_xyz(c: &Chromaticity) -> Option<Mat3> {
    let (r, g, b) = (xyz(c.r), xyz(c.g), xyz(c.b));
    let m = [[r[0], g[0], b[0]], [r[1], g[1], b[1]], [r[2], g[2], b[2]]];
    let s = apply(&invert(&m)?, xyz(c.w));
    Some([
        [m[0][0] * s[0], m[0][1] * s[1], m[0][2] * s[2]],
        [m[1][0] * s[0], m[1][1] * s[1], m[1][2] * s[2]],
        [m[2][0] * s[0], m[2][1] * s[1], m[2][2] * s[2]],
    ])
}

/// Bradford chromatic adaptation from white `from` to white `to` (XYZ).
fn bradford(from: (f64, f64), to: (f64, f64)) -> Option<Mat3> {
    const B: Mat3 = [
        [0.8951, 0.2664, -0.1614],
        [-0.7502, 1.7135, 0.0367],
        [0.0389, -0.0685, 1.0296],
    ];
    let (s, d) = (apply(&B, xyz(from)), apply(&B, xyz(to)));
    let scale = [
        [d[0] / s[0], 0.0, 0.0],
        [0.0, d[1] / s[1], 0.0],
        [0.0, 0.0, d[2] / s[2]],
    ];
    Some(mul(&invert(&B)?, &mul(&scale, &B)))
}

/// Linear RGB in `src` to linear RGB in `dst`, adapting the white point when
/// they differ. `None` for degenerate primaries.
pub fn rgb_to_rgb(src: &Chromaticity, dst: &Chromaticity) -> Option<Mat3> {
    let to_xyz = rgb_to_xyz(src)?;
    let from_xyz = invert(&rgb_to_xyz(dst)?)?;
    let adapt = if src.w == dst.w {
        IDENTITY
    } else {
        bradford(src.w, dst.w)?
    };
    let m = mul(&from_xyz, &mul(&adapt, &to_xyz));
    // Client-supplied primaries reach here; nothing non-finite may go on to a
    // GL uniform.
    // Real gamut matrices have entries of order 1; anything near f32's range
    // would overflow to inf when cast for the GL uniform.
    m.iter()
        .flatten()
        .all(|v| v.is_finite() && v.abs() <= 1.0e6)
        .then_some(m)
}

/// A row-major matrix as the column-major `[f32; 9]` GLSL's `mat3` takes.
pub fn to_gl(m: &Mat3) -> [f32; 9] {
    let mut out = [0.0f32; 9];
    for c in 0..3 {
        for r in 0..3 {
            out[c * 3 + r] = m[r][c] as f32;
        }
    }
    out
}

/// ITU-R BT.2390 EETF, the tone-mapping knee, in the PQ domain: maps a PQ
/// value from a source range whose peak is `src_max` to a target whose peak
/// is `dst_max` (both PQ code values, black at 0). Linear below the knee, a
/// Hermite roll-off above it that lands exactly on `dst_max`. Identity when
/// the source already fits.
pub fn bt2390(e: f64, src_max: f64, dst_max: f64) -> f64 {
    if src_max <= dst_max || src_max <= 0.0 {
        return e.min(src_max.max(dst_max));
    }
    let e1 = (e / src_max).clamp(0.0, 1.0);
    let max_lum = dst_max / src_max;
    let ks = 1.5 * max_lum - 0.5;
    let e2 = if e1 < ks || ks >= 1.0 {
        e1
    } else {
        let t = (e1 - ks) / (1.0 - ks);
        let (t2, t3) = (t * t, t * t * t);
        (2.0 * t3 - 3.0 * t2 + 1.0) * ks
            + (t3 - 2.0 * t2 + t) * (1.0 - ks)
            + (-2.0 * t3 + 3.0 * t2) * max_lum
    };
    e2 * src_max
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn pq_round_trips_and_hits_the_reference_points() {
        // BT.2408: 203 cd/m² reference white is PQ 0.58 (58%).
        assert!(close(pq_inv_eotf(203.0), 0.5807, 1e-3));
        assert!(close(pq_inv_eotf(10000.0), 1.0, 1e-9));
        assert!(close(pq_inv_eotf(100.0), 0.5081, 1e-3));
        for nits in [0.005, 1.0, 80.0, 203.0, 400.0, 1000.0, 4000.0] {
            assert!(close(pq_eotf(pq_inv_eotf(nits)), nits, nits * 1e-6 + 1e-9));
        }
    }

    #[test]
    fn hlg_is_continuous_and_reaches_one() {
        assert!(close(hlg_inv_oetf(0.5), 1.0 / 12.0, 1e-9));
        assert!(close(hlg_inv_oetf(0.5 + 1e-9), 1.0 / 12.0, 1e-6));
        assert!(close(hlg_inv_oetf(1.0), 1.0, 1e-6));
    }

    #[test]
    fn bt709_to_bt2020_matches_bt2087() {
        // ITU-R BT.2087 Table 2 (709 -> 2020, linear).
        let m = rgb_to_rgb(&BT709, &BT2020).unwrap();
        let want = [
            [0.6274, 0.3293, 0.0433],
            [0.0691, 0.9195, 0.0114],
            [0.0164, 0.0880, 0.8956],
        ];
        for r in 0..3 {
            for c in 0..3 {
                assert!(close(m[r][c], want[r][c], 2e-4), "m[{r}][{c}]={}", m[r][c]);
            }
        }
        // White stays white; rows sum to one.
        for row in m {
            assert!(close(row.iter().sum::<f64>(), 1.0, 1e-9));
        }
        let back = mul(&rgb_to_rgb(&BT2020, &BT709).unwrap(), &m);
        for r in 0..3 {
            for c in 0..3 {
                assert!(close(back[r][c], IDENTITY[r][c], 1e-9));
            }
        }
    }

    #[test]
    fn a_white_point_change_is_adapted() {
        let d50 = Chromaticity {
            w: (0.3457, 0.3585),
            ..BT709
        };
        let m = rgb_to_rgb(&d50, &BT709).unwrap();
        let white = apply(&m, [1.0, 1.0, 1.0]);
        for v in white {
            assert!(close(v, 1.0, 1e-3), "{white:?}");
        }
    }

    #[test]
    fn bt2390_is_identity_below_the_knee_and_lands_on_the_peak() {
        let (src, dst) = (pq_inv_eotf(4000.0), pq_inv_eotf(400.0));
        assert!(close(bt2390(src, src, dst), dst, 1e-9));
        let low = pq_inv_eotf(50.0);
        assert!(close(bt2390(low, src, dst), low, 1e-9));
        // Monotone through the knee.
        let mut prev = 0.0;
        for i in 0..=100 {
            let e = src * i as f64 / 100.0;
            let out = bt2390(e, src, dst);
            assert!(out + 1e-12 >= prev && out <= dst + 1e-9);
            prev = out;
        }
        // Nothing to do when the source fits.
        assert!(close(bt2390(0.3, 0.5, 0.7), 0.3, 1e-12));
    }

    #[test]
    fn degenerate_white_points_are_refused() {
        let bad = Chromaticity {
            // y = 0 divides by zero: the matrix would be NaN/inf.
            w: (0.3127, 0.0),
            ..BT709
        };
        assert!(rgb_to_rgb(&bad, &BT709).is_none());
        // Finite in f64 but far outside any real gamut: refused too.
        let thin = Chromaticity {
            r: (0.3127, 0.3290),
            g: (0.3127 + 1.0e-12, 0.3290),
            b: (0.3127, 0.3290 + 1.0e-12),
            w: D65,
        };
        assert!(rgb_to_rgb(&thin, &BT709).is_none());
    }

    #[test]
    fn to_gl_is_column_major() {
        let m = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        assert_eq!(to_gl(&m), [1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0]);
    }
}
