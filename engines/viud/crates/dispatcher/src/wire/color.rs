// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! `wp_color_management_v1`: the vocabulary (what the manager advertises), the
//! conversions between protocol values and `render_gles::color::hdr`
//! descriptions, and the image-description info events. The request handlers
//! are in `wire::colorsurf`.

use render_gles::color::hdr::describe::{self, Description, Transfer};
use render_gles::color::hdr::math::{self, BT709, BT2020, Chromaticity, D65, DISPLAY_P3};
use smithay::reexports::wayland_protocols::wp::color_management::v1::server::{
    wp_color_management_output_v1::WpColorManagementOutputV1,
    wp_color_management_surface_feedback_v1::WpColorManagementSurfaceFeedbackV1,
    wp_color_manager_v1::{Feature, Primaries, RenderIntent, TransferFunction, WpColorManagerV1},
    wp_image_description_info_v1::WpImageDescriptionInfoV1,
    wp_image_description_v1::WpImageDescriptionV1,
};
use smithay::reexports::wayland_server::Weak;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::{DisplayHandle, GlobalDispatch, Resource};
use smithay::wayland::compositor::with_states;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

pub static IDENTITY: AtomicU32 = AtomicU32::new(1);

/// An image description object's data: the description, or `None` for one
/// that was sent `failed` (any use of it is then a protocol error the client
/// already knows about; it is treated as untagged).
#[derive(Debug, Clone, Copy, Default)]
pub struct ImageDescData(pub Option<Description>);

/// A parametric creator's accumulated requests.
#[derive(Debug, Default)]
pub struct ParamsState {
    pub transfer: Option<Transfer>,
    pub primaries: Option<Chromaticity>,
    /// (min, max, reference) in cd/m².
    pub luminances: Option<(f64, f64, f64)>,
    pub mastering_primaries: Option<Chromaticity>,
    /// (min, max) in cd/m².
    pub mastering_luminance: Option<(f64, f64)>,
    pub max_cll: Option<f64>,
    pub max_fall: Option<f64>,
    /// Set by a request the compositor cannot honour; `create` then fails.
    pub unsupported: Option<&'static str>,
}

impl ParamsState {
    /// The finished description, or why it cannot be made.
    pub fn build(&self) -> Result<Description, &'static str> {
        if let Some(why) = self.unsupported {
            return Err(why);
        }
        let (Some(transfer), Some(primaries)) = (self.transfer, self.primaries) else {
            return Err("transfer function and primaries are both required");
        };
        let mut d = Description::new(transfer, primaries);
        if let Some((min, max, reference)) = self.luminances {
            if !(min < reference && min < max) {
                return Err("luminances: min must be below reference and max");
            }
            d.lum_min = min;
            // PQ's range is fixed by the transfer function itself.
            if transfer != Transfer::Pq {
                d.lum_max = max;
            }
            d.lum_ref = reference;
        }
        d.mastering_primaries = self.mastering_primaries;
        if let Some((min, max)) = self.mastering_luminance {
            d.mastering_min = Some(min);
            d.mastering_max = Some(max);
        }
        d.max_cll = self.max_cll;
        d.max_fall = self.max_fall;
        Ok(d)
    }
}

pub fn transfer_from(tf: TransferFunction) -> Option<Transfer> {
    Some(match tf {
        TransferFunction::Srgb => Transfer::Srgb,
        TransferFunction::Gamma22 => Transfer::Gamma22,
        TransferFunction::Gamma28 => Transfer::Gamma28,
        TransferFunction::Bt1886 => Transfer::Bt1886,
        TransferFunction::ExtLinear => Transfer::Linear,
        TransferFunction::St2084Pq => Transfer::Pq,
        TransferFunction::Hlg => Transfer::Hlg,
        _ => return None,
    })
}

pub fn transfer_to(tf: Transfer) -> TransferFunction {
    match tf {
        Transfer::Srgb => TransferFunction::Srgb,
        Transfer::Gamma22 => TransferFunction::Gamma22,
        Transfer::Gamma28 => TransferFunction::Gamma28,
        Transfer::Bt1886 => TransferFunction::Bt1886,
        Transfer::Linear => TransferFunction::ExtLinear,
        Transfer::Pq => TransferFunction::St2084Pq,
        Transfer::Hlg => TransferFunction::Hlg,
    }
}

const NAMED_TF: [TransferFunction; 7] = [
    TransferFunction::Srgb,
    TransferFunction::Gamma22,
    TransferFunction::Gamma28,
    TransferFunction::Bt1886,
    TransferFunction::ExtLinear,
    TransferFunction::St2084Pq,
    TransferFunction::Hlg,
];

/// The named primaries the compositor knows the chromaticities of.
pub fn primaries_from(p: Primaries) -> Option<Chromaticity> {
    Some(match p {
        Primaries::Srgb => BT709,
        Primaries::Bt2020 => BT2020,
        Primaries::DisplayP3 => DISPLAY_P3,
        Primaries::DciP3 => Chromaticity {
            w: (0.314, 0.351),
            ..DISPLAY_P3
        },
        // BT.601 625-line (EBU).
        Primaries::Pal => Chromaticity {
            r: (0.640, 0.330),
            g: (0.290, 0.600),
            b: (0.150, 0.060),
            w: D65,
        },
        // SMPTE 170M / BT.601 525-line.
        Primaries::Ntsc => Chromaticity {
            r: (0.630, 0.340),
            g: (0.310, 0.595),
            b: (0.155, 0.070),
            w: D65,
        },
        Primaries::AdobeRgb => Chromaticity {
            r: (0.640, 0.330),
            g: (0.210, 0.710),
            b: (0.150, 0.060),
            w: D65,
        },
        _ => return None,
    })
}

const NAMED_PRIMARIES: [Primaries; 7] = [
    Primaries::Srgb,
    Primaries::Bt2020,
    Primaries::DisplayP3,
    Primaries::DciP3,
    Primaries::Pal,
    Primaries::Ntsc,
    Primaries::AdobeRgb,
];

fn named_primaries_of(c: &Chromaticity) -> Option<Primaries> {
    NAMED_PRIMARIES
        .into_iter()
        .find(|p| primaries_from(*p).as_ref() == Some(c))
}

/// Protocol chromaticity (×1 000 000 integers) to xy.
#[allow(clippy::too_many_arguments)]
pub fn chromaticity(
    rx: i32,
    ry: i32,
    gx: i32,
    gy: i32,
    bx: i32,
    by: i32,
    wx: i32,
    wy: i32,
) -> Option<Chromaticity> {
    let f = |v: i32| v as f64 / 1_000_000.0;
    let c = Chromaticity {
        r: (f(rx), f(ry)),
        g: (f(gx), f(gy)),
        b: (f(bx), f(by)),
        w: (f(wx), f(wy)),
    };
    // Degenerate primaries (collinear, or a zero y) cannot form a matrix.
    if [c.r.1, c.g.1, c.b.1, c.w.1].iter().any(|y| *y <= 0.0) {
        return None;
    }
    math::rgb_to_xyz(&c)?;
    Some(c)
}

fn xy6(c: &Chromaticity) -> [i32; 8] {
    let i = |v: f64| (v * 1_000_000.0).round() as i32;
    [
        i(c.r.0),
        i(c.r.1),
        i(c.g.0),
        i(c.g.1),
        i(c.b.0),
        i(c.b.1),
        i(c.w.0),
        i(c.w.1),
    ]
}

/// An output's description: for an HDR output PQ, BT.2020, reference white at
/// the SDR white and the display's volume as the target; plain sRGB otherwise.
pub fn output_description(output: &str) -> Description {
    describe_output(describe::output(output))
}

/// What a surface should be in, not tied to one output: the brightest HDR
/// output's description when there is one, else sRGB.
pub fn preferred() -> Description {
    describe_output(describe::any_hdr_output())
}

fn describe_output(out: Option<describe::OutputColor>) -> Description {
    match out {
        Some(out) => {
            let mut d = Description::new(Transfer::Pq, BT2020);
            d.lum_ref = out.sdr_white;
            d.mastering_primaries = Some(out.display);
            d.mastering_min = Some(out.min);
            d.mastering_max = Some(out.max);
            d.max_cll = Some(out.max);
            d
        }
        None => Description::srgb(),
    }
}

/// Live `wp_color_management_output_v1` objects (with their output's name) and
/// surface feedback objects, so a change in an output's HDR state reaches the
/// clients that asked.
static OUTPUT_OBJECTS: Mutex<Vec<(String, Weak<WpColorManagementOutputV1>)>> =
    Mutex::new(Vec::new());
static FEEDBACK_OBJECTS: Mutex<Vec<Weak<WpColorManagementSurfaceFeedbackV1>>> =
    Mutex::new(Vec::new());

pub fn track_output(output: &str, obj: &WpColorManagementOutputV1) {
    let mut v = OUTPUT_OBJECTS.lock().unwrap();
    v.retain(|(_, w)| w.upgrade().is_ok());
    v.push((output.to_string(), obj.downgrade()));
}

pub fn track_feedback(obj: &WpColorManagementSurfaceFeedbackV1) {
    let mut v = FEEDBACK_OBJECTS.lock().unwrap();
    v.retain(|w| w.upgrade().is_ok());
    v.push(obj.downgrade());
}

/// An output's HDR state changed: tell that output's objects, and every surface
/// feedback object (their preferred description follows the HDR outputs).
pub fn output_changed(output: &str) {
    for (name, w) in OUTPUT_OBJECTS.lock().unwrap().iter() {
        if name == output
            && let Ok(o) = w.upgrade()
        {
            o.image_description_changed();
        }
    }
    let identity = IDENTITY.fetch_add(1, Ordering::Relaxed);
    for w in FEEDBACK_OBJECTS.lock().unwrap().iter() {
        if let Ok(fb) = w.upgrade() {
            if fb.version() >= 2 {
                fb.preferred_changed2(0, identity);
            } else {
                fb.preferred_changed(identity);
            }
        }
    }
}

/// A surface's colour description as double-buffered surface state: set by
/// `set_image_description` / `unset_image_description` (pending), applied by
/// `wl_surface.commit` with the buffer it describes, and through smithay's
/// cache for synchronised subsurfaces. `serial` changes on every set or unset.
#[derive(Debug, Clone, Copy, Default)]
pub struct ColorCached {
    pub description: Option<Description>,
    pub serial: u64,
}

impl smithay::wayland::compositor::Cacheable for ColorCached {
    fn commit(&mut self, _dh: &DisplayHandle) -> Self {
        *self
    }
    fn merge_into(self, into: &mut Self, _dh: &DisplayHandle) {
        *into = self;
    }
}

static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// The serial last handed to the composite, per surface.
#[derive(Default)]
struct AppliedSerial(std::sync::atomic::AtomicU64);

/// Record a pending description (`None`: unset); it takes effect on commit.
pub fn set_pending_color(surface: &WlSurface, description: Option<Description>) {
    let serial = SERIAL.fetch_add(1, Ordering::Relaxed) + 1;
    with_states(surface, |states| {
        let mut cached = states.cached_state.get::<ColorCached>();
        let pending = cached.pending();
        pending.description = description;
        pending.serial = serial;
    });
}

/// `wl_surface.commit`: hand the composite every committed description change
/// in this surface's tree (a parent commit applies its synchronised
/// subsurfaces' cached state too).
pub fn on_commit(surface: &WlSurface) {
    use smithay::wayland::compositor::{TraversalAction, with_surface_tree_downward};
    // The global exists only with `hdr` set; without it no client can have set
    // a description, so skip the walk on every commit.
    if !model::environment::config::base::try_get().is_some_and(|e| e.hdr) {
        return;
    }
    let mut changed: Vec<(WlSurface, Option<Description>)> = Vec::new();
    with_surface_tree_downward(
        surface,
        (),
        |_, _, _| TraversalAction::DoChildren(()),
        |s, states, _| {
            let current = *states.cached_state.get::<ColorCached>().current();
            states.data_map.insert_if_missing_threadsafe(AppliedSerial::default);
            if let Some(applied) = states.data_map.get::<AppliedSerial>()
                && applied.0.swap(current.serial, Ordering::Relaxed) != current.serial
            {
                changed.push((s.clone(), current.description));
            }
        },
        |_, _, _| true,
    );
    for (s, d) in changed {
        describe::set_surface(&s, d);
    }
}

pub fn send_ready(desc: &WpImageDescriptionV1) {
    let id = IDENTITY.fetch_add(1, Ordering::Relaxed);
    if desc.version() >= 2 {
        desc.ready2(0, id);
    } else {
        desc.ready(id);
    }
}

/// The info events for `d`. NOT `done`: it is a destructor event, so the caller
/// queues it for after dispatch (`Dispatch::pending_info_done`).
pub fn send_info(info: &WpImageDescriptionInfoV1, d: &Description) {
    match named_primaries_of(&d.primaries) {
        Some(p) => info.primaries_named(p),
        None => {
            let [rx, ry, gx, gy, bx, by, wx, wy] = xy6(&d.primaries);
            info.primaries(rx, ry, gx, gy, bx, by, wx, wy);
        }
    }
    info.tf_named(transfer_to(d.transfer));
    info.luminances(
        (d.lum_min * 10000.0).round() as u32,
        d.lum_max.round() as u32,
        d.lum_ref.round() as u32,
    );
    let target = d.mastering_primaries.unwrap_or(d.primaries);
    let [rx, ry, gx, gy, bx, by, wx, wy] = xy6(&target);
    info.target_primaries(rx, ry, gx, gy, bx, by, wx, wy);
    info.target_luminance(
        (d.mastering_min.unwrap_or(d.lum_min) * 10000.0).round() as u32,
        d.mastering_max.unwrap_or(d.lum_max).round() as u32,
    );
    if let Some(cll) = d.max_cll {
        info.target_max_cll(cll.round() as u32);
    }
    if let Some(fall) = d.max_fall {
        info.target_max_fall(fall.round() as u32);
    }
}

pub fn wenum<T: TryFrom<u32>>(w: smithay::reexports::wayland_server::WEnum<T>) -> Option<T> {
    w.into_result().ok()
}

pub fn create_global<W: GlobalDispatch<WpColorManagerV1, ()> + 'static>(dh: &DisplayHandle) {
    if !model::environment::config::base::get().hdr {
        return;
    }
    dh.create_global::<W, WpColorManagerV1, ()>(1, ());
}

pub fn bind_color_manager(mgr: WpColorManagerV1) {
    mgr.supported_intent(RenderIntent::Perceptual);
    for feature in [
        Feature::Parametric,
        Feature::SetPrimaries,
        Feature::SetLuminances,
        Feature::SetMasteringDisplayPrimaries,
        Feature::ExtendedTargetVolume,
        Feature::WindowsScrgb,
    ] {
        mgr.supported_feature(feature);
    }
    for tf in NAMED_TF {
        mgr.supported_tf_named(tf);
    }
    for p in NAMED_PRIMARIES {
        mgr.supported_primaries_named(p);
    }
    mgr.done();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_advertised_name_converts() {
        for tf in NAMED_TF {
            assert_eq!(transfer_to(transfer_from(tf).unwrap()), tf);
        }
        for p in NAMED_PRIMARIES {
            let c = primaries_from(p).unwrap();
            assert_eq!(named_primaries_of(&c), Some(p));
            assert!(math::rgb_to_xyz(&c).is_some());
        }
    }

    #[test]
    fn a_creator_needs_both_names_and_sane_luminances() {
        let mut s = ParamsState::default();
        assert!(s.build().is_err());
        s.transfer = Some(Transfer::Pq);
        s.primaries = Some(BT2020);
        let d = s.build().unwrap();
        assert_eq!((d.lum_max, d.lum_ref), (10000.0, 203.0));
        s.luminances = Some((0.01, 400.0, 300.0));
        let d = s.build().unwrap();
        // PQ keeps its fixed range; the reference moves.
        assert_eq!((d.lum_max, d.lum_ref), (10000.0, 300.0));
        s.luminances = Some((500.0, 400.0, 300.0));
        assert!(s.build().is_err());
    }

    #[test]
    fn protocol_chromaticity_round_trips() {
        let [rx, ry, gx, gy, bx, by, wx, wy] = xy6(&BT2020);
        assert_eq!(chromaticity(rx, ry, gx, gy, bx, by, wx, wy), Some(BT2020));
        assert_eq!(chromaticity(0, 0, 0, 0, 0, 0, 0, 0), None);
    }
}
