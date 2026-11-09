// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! What a surface's numbers mean (its `wp_color_management_v1` image
//! description), what the output can show, and the decode uniforms that join
//! the two. The protocol layer writes descriptions; the composite reads them.

use super::math::{self, BT709, BT2020, Chromaticity};
use smithay::backend::renderer::element::Id;
use smithay::backend::renderer::gles::{Uniform, UniformValue};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{Resource, Weak};
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

/// A transfer function the composite can decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transfer {
    /// sRGB, decoded as pure gamma 2.2 (what SDR displays do; KWin's choice).
    Srgb,
    Gamma22,
    /// BT.1886, decoded as gamma 2.4.
    Bt1886,
    Gamma28,
    /// Linear, extended range (`ext_linear`, scRGB-style).
    Linear,
    /// SMPTE ST 2084.
    Pq,
    /// ARIB STD-B67 / BT.2100 HLG.
    Hlg,
}

impl Transfer {
    /// The `u_tf` code the decode shader branches on.
    pub fn code(self) -> i32 {
        match self {
            Transfer::Pq => 1,
            Transfer::Hlg => 2,
            Transfer::Linear => 3,
            Transfer::Bt1886 => 4,
            Transfer::Gamma28 => 5,
            Transfer::Srgb | Transfer::Gamma22 => 0,
        }
    }

    pub fn is_hdr(self) -> bool {
        matches!(self, Transfer::Pq | Transfer::Hlg)
    }

    /// The protocol's default luminances for a named transfer function:
    /// (min, max, reference) in cd/m².
    pub fn default_luminances(self) -> (f64, f64, f64) {
        match self {
            Transfer::Pq => (0.005, 10000.0, 203.0),
            Transfer::Hlg => (0.005, 1000.0, 203.0),
            _ => (0.2, 80.0, 80.0),
        }
    }
}

/// A complete parametric image description.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Description {
    pub transfer: Transfer,
    pub primaries: Chromaticity,
    /// Primary colour volume luminances, cd/m².
    pub lum_min: f64,
    pub lum_max: f64,
    pub lum_ref: f64,
    /// Mastering display (target colour volume), when the client gave one.
    pub mastering_primaries: Option<Chromaticity>,
    pub mastering_min: Option<f64>,
    pub mastering_max: Option<f64>,
    /// Content light levels, cd/m².
    pub max_cll: Option<f64>,
    pub max_fall: Option<f64>,
}

impl Description {
    pub fn new(transfer: Transfer, primaries: Chromaticity) -> Self {
        let (lum_min, lum_max, lum_ref) = transfer.default_luminances();
        Self {
            transfer,
            primaries,
            lum_min,
            lum_max,
            lum_ref,
            mastering_primaries: None,
            mastering_min: None,
            mastering_max: None,
            max_cll: None,
            max_fall: None,
        }
    }

    pub fn srgb() -> Self {
        Self::new(Transfer::Srgb, BT709)
    }

    /// True when decoding would change nothing: content already in the
    /// composite's blending space.
    pub fn is_identity(&self) -> bool {
        matches!(self.transfer, Transfer::Srgb | Transfer::Gamma22)
            && self.primaries == BT709
            && (self.lum_max - self.lum_ref).abs() < 1e-6
    }

    /// The brightest the content claims to get, cd/m²: MaxCLL, else the
    /// mastering peak, else the primary volume's maximum.
    pub fn content_peak(&self) -> f64 {
        self.max_cll
            .filter(|v| *v > 0.0)
            .or(self.mastering_max.filter(|v| *v > 0.0))
            .unwrap_or(self.lum_max)
    }
}

/// What an HDR output shows, in cd/m².
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutputColor {
    /// Where SDR white (and every surface's reference white) lands.
    pub sdr_white: f64,
    /// The display's peak (EDID, or an override).
    pub max: f64,
    /// The display's black.
    pub min: f64,
    /// The display's own primaries (EDID), for the output's image description.
    pub display: Chromaticity,
}

impl OutputColor {
    /// An SDR output as a decode target: white is the peak, so HDR content is
    /// tone-mapped into SDR white, and the 8/10-bit target clips the gamut to
    /// BT.709. The absolute level only scales the curve; 203 is BT.2408's.
    pub fn sdr() -> Self {
        Self {
            sdr_white: 203.0,
            max: 203.0,
            min: 0.2,
            display: BT709,
        }
    }

    /// Defaults per BT.2408 when the EDID is silent: 203 cd/m² reference white
    /// and a conservative 400 cd/m² (HDR400) peak.
    pub fn from_edid(max: Option<f64>, min: Option<f64>, display: Option<Chromaticity>) -> Self {
        let max = std::env::var("MIX_VIUD_HDR_MAX_NITS")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|v| (80.0..=10000.0).contains(v))
            .or(max.filter(|v| *v > 0.0))
            .unwrap_or(400.0);
        let sdr_white = std::env::var("MIX_VIUD_HDR_SDR_WHITE")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|v| (40.0..=10000.0).contains(v))
            .unwrap_or(203.0)
            .min(max);
        Self {
            sdr_white,
            max,
            min: min.filter(|v| *v >= 0.0).unwrap_or(0.005),
            display: display.unwrap_or(BT2020),
        }
    }

    /// The encode stage's uniforms.
    pub fn encode_uniforms(&self) -> Vec<Uniform<'static>> {
        let to2020 = math::rgb_to_rgb(&BT709, &BT2020).unwrap_or(math::IDENTITY);
        vec![
            Uniform::new(
                "u_to2020",
                UniformValue::Matrix3x3 {
                    matrices: vec![math::to_gl(&to2020)],
                    transpose: false,
                },
            ),
            Uniform::new("u_sdr_white", self.sdr_white as f32),
            Uniform::new("u_out_max", self.max as f32),
        ]
    }

    /// The decode stage's uniforms for content described by `d`.
    pub fn decode_uniforms(&self, d: &Description) -> Vec<Uniform<'static>> {
        let prim = math::rgb_to_rgb(&d.primaries, &BT709).unwrap_or(math::IDENTITY);
        let hlg = d.transfer == Transfer::Hlg;
        // HLG is scene-referred: BT.2100's OOTF renders it for the DISPLAY's peak
        // Lw (system gamma from Lw too), giving absolute display light directly.
        // So for HLG the scale is this output's peak and the result is taken as
        // nits as is (reference = SDR white), with no further tone mapping.
        let lum_max = if hlg { self.max } else { d.lum_max };
        let lum_ref = if hlg {
            self.sdr_white
        } else if d.lum_ref > 0.0 {
            d.lum_ref
        } else {
            80.0
        };
        let hlg_gamma = 1.2 + 0.42 * (lum_max.max(1.0) / 1000.0).log10();
        // Tone map on the composite's absolute output luminance: the content's
        // peak lands at peak / ref * sdr_white; compress it into the display.
        let peak_out = d.content_peak() / lum_ref * self.sdr_white;
        let tm = if !hlg && (d.transfer.is_hdr() || d.transfer == Transfer::Linear) {
            if peak_out > self.max {
                (
                    1.0,
                    math::pq_inv_eotf(peak_out) as f32,
                    math::pq_inv_eotf(self.max) as f32,
                )
            } else {
                (0.0, 0.0, 0.0)
            }
        } else {
            (0.0, 0.0, 0.0)
        };
        vec![
            Uniform::new("u_tf", d.transfer.code()),
            Uniform::new(
                "u_prim",
                UniformValue::Matrix3x3 {
                    matrices: vec![math::to_gl(&prim)],
                    transpose: false,
                },
            ),
            Uniform::new("u_lum_max", lum_max as f32),
            Uniform::new("u_inv_ref", (1.0 / lum_ref) as f32),
            Uniform::new("u_hlg_gamma", hlg_gamma as f32),
            Uniform::new("u_sdr_white", self.sdr_white as f32),
            Uniform::new("u_tm", tm),
        ]
    }
}

/// Each output's HDR state, by output name (absent: SDR). The protocol layer
/// reads it to answer "what does this output want" per `wl_output`.
static OUTPUTS: RwLock<Option<HashMap<String, OutputColor>>> = RwLock::new(None);

/// Record (or, with `None`, clear) one output's HDR state. Other outputs keep
/// theirs. Returns whether anything changed, so callers notify only then.
pub fn set_output(name: &str, color: Option<OutputColor>) -> bool {
    let mut guard = OUTPUTS.write().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    let before = map.get(name).copied();
    match color {
        Some(c) => {
            map.insert(name.to_string(), c);
        }
        None => {
            map.remove(name);
        }
    }
    before != color
}

/// One output's HDR state, `None` when it is SDR (or unknown).
pub fn output(name: &str) -> Option<OutputColor> {
    OUTPUTS
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .and_then(|m| m.get(name).copied())
}

/// The HDR state of some HDR output, for answers not tied to one output (a
/// surface's preferred description). The brightest, so content is never
/// mastered below what any HDR output can show. `None` when all are SDR.
pub fn any_hdr_output() -> Option<OutputColor> {
    OUTPUTS
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .and_then(|m| m.values().copied().max_by(|a, b| a.max.total_cmp(&b.max)))
}

/// A tagged surface: its description and the generation it was set at. The
/// generation changes on every set or unset, so the composite can tell a
/// description change from an unchanged surface even when no buffer damage
/// came with it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tagged {
    pub description: Description,
    pub generation: u64,
}

type Registry = HashMap<Id, (Weak<WlSurface>, Tagged)>;
static SURFACES: Mutex<Option<Registry>> = Mutex::new(None);
static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
/// Most live tagged surfaces kept; beyond it new tags draw undecoded.
const SURFACES_MAX: usize = 16384;

/// Record (or, with `None`, forget) a surface's image description, keyed by
/// the render element id smithay gives that surface.
pub fn set_surface(surface: &WlSurface, description: Option<Description>) {
    let id = Id::from_wayland_resource(surface);
    let generation = GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut guard = SURFACES.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    // Dead surfaces go now, not only at the next render, so a client cannot
    // grow this by tagging and destroying surfaces between frames; and it
    // never exceeds a hard cap of live tagged surfaces.
    map.retain(|_, (weak, _)| weak.upgrade().is_ok());
    match description {
        Some(_) if map.len() >= SURFACES_MAX && !map.contains_key(&id) => {}
        Some(description) => {
            map.insert(
                id,
                (
                    surface.downgrade(),
                    Tagged {
                        description,
                        generation,
                    },
                ),
            );
        }
        None => {
            map.remove(&id);
        }
    }
}

/// The live tagged surfaces, by render element id. Dead surfaces are dropped
/// here, so the registry never outgrows the surfaces that exist.
pub fn surfaces() -> HashMap<Id, Tagged> {
    let mut guard = SURFACES.lock().unwrap_or_else(|e| e.into_inner());
    let Some(map) = guard.as_mut() else {
        return HashMap::new();
    };
    map.retain(|_, (weak, _)| weak.upgrade().is_ok());
    map.iter().map(|(id, (_, t))| (id.clone(), *t)).collect()
}

/// Whether any live surface carries a description that needs decoding: an SDR
/// output must then composite through the decode stage too.
pub fn any_decoding() -> bool {
    surfaces().values().any(|t| !t.description.is_identity())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(us: &[Uniform<'static>], name: &str) -> UniformValue {
        us.iter().find(|u| u.name == name).unwrap().value.clone()
    }

    #[test]
    fn srgb_is_the_identity_and_pq_is_not() {
        assert!(Description::srgb().is_identity());
        assert!(!Description::new(Transfer::Pq, BT2020).is_identity());
        assert!(!Description::new(Transfer::Srgb, math::DISPLAY_P3).is_identity());
    }

    #[test]
    fn pq_content_brighter_than_the_display_is_tone_mapped() {
        let out = OutputColor {
            sdr_white: 203.0,
            max: 400.0,
            min: 0.005,
            display: BT2020,
        };
        let mut d = Description::new(Transfer::Pq, BT2020);
        d.max_cll = Some(1000.0);
        match value(&out.decode_uniforms(&d), "u_tm") {
            UniformValue::_3f(on, src, dst) => {
                assert_eq!(on, 1.0);
                assert!((src as f64 - math::pq_inv_eotf(1000.0)).abs() < 1e-6);
                assert!((dst as f64 - math::pq_inv_eotf(400.0)).abs() < 1e-6);
            }
            v => panic!("{v:?}"),
        }
        d.max_cll = Some(300.0);
        assert!(matches!(
            value(&out.decode_uniforms(&d), "u_tm"),
            UniformValue::_3f(on, _, _) if on == 0.0
        ));
    }

    #[test]
    fn hlg_is_rendered_for_the_display_peak() {
        let out = OutputColor {
            sdr_white: 203.0,
            max: 400.0,
            min: 0.005,
            display: BT2020,
        };
        let us = out.decode_uniforms(&Description::new(Transfer::Hlg, BT2020));
        // Scale and system gamma from the display (400), not the content (1000).
        assert!(matches!(value(&us, "u_lum_max"), UniformValue::_1f(v) if v == 400.0));
        let want_gamma = (1.2 + 0.42 * (400.0f64 / 1000.0).log10()) as f32;
        assert!(
            matches!(value(&us, "u_hlg_gamma"), UniformValue::_1f(v) if (v - want_gamma).abs() < 1e-6)
        );
        // Absolute display light: no further tone mapping.
        assert!(matches!(value(&us, "u_tm"), UniformValue::_3f(on, _, _) if on == 0.0));
    }

    #[test]
    fn sdr_white_never_exceeds_the_peak() {
        let out = OutputColor::from_edid(Some(150.0), None, None);
        assert!(out.sdr_white <= out.max);
    }
}
