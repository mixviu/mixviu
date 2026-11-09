// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Raw-DRM CONNECTOR PROPERTY pass: `Colorspace`, `HDR_OUTPUT_METADATA` and
//! `max bpc`. smithay's `DrmCompositor` owns the per-frame page-flip commit but
//! exposes none of these, so we set them ourselves; they are sticky atomic state
//! that persists across smithay's flips AND across compositors — which is the
//! whole reason this pass must run for SDR pipes too, not just HDR ones. See
//! `apply_connector_props` for what each value is set to and why.
//!
//! SAFETY: every commit is preceded by a `TEST_ONLY` atomic commit. If the test
//! fails (malformed blob, unsupported property, wrong enum) we return an error
//! and the caller stays SDR — a bad blob can never blank the display.

use crate::context::render::OutputPipe;
use kms::edid::parse::HdrInfo;
use render_gles::color::hdr::describe::{self, OutputColor};
use render_gles::color::hdr::math::Chromaticity;
use render_gles::color::hdr::pass::HdrPass;
use scenegraph::scene::element::SceneElement;
use smithay::backend::drm::DrmDeviceFd;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::utils::Scale;

/// What this display shows in HDR, from its EDID (with the
/// `MIX_VIUD_HDR_SDR_WHITE` / `MIX_VIUD_HDR_MAX_NITS` overrides).
pub fn output_color(caps: &HdrInfo) -> OutputColor {
    let p = &caps.primaries;
    let f = |c: (f32, f32)| (c.0 as f64, c.1 as f64);
    let display =
        (p.red.0 > 0.0 && p.green.0 > 0.0 && p.blue.0 > 0.0 && p.white.0 > 0.0).then(|| {
            Chromaticity {
                r: f(p.red),
                g: f(p.green),
                b: f(p.blue),
                w: f(p.white),
            }
        });
    OutputColor::from_edid(
        caps.hdr.max_luminance.map(f64::from),
        caps.hdr.min_luminance.map(f64::from),
        display,
    )
}

/// Tell the protocol layer what output `name` wants (`None`: SDR), log it, and
/// notify the clients that asked when it changed.
pub fn publish(name: &str, color: Option<OutputColor>) {
    if let Some(c) = &color {
        info!(
            "HDR output {name}: PQ BT.2020, SDR white {} cd/m², peak {} cd/m², black {} cd/m²",
            c.sdr_white, c.max, c.min
        );
    }
    if describe::set_output(name, color) {
        dispatcher::wire::color::output_changed(name);
    }
}

/// Take a pipe off HDR after a failure: back to SDR signalling (the connector
/// pass runs again and resets `Colorspace`/metadata) and an SDR composite.
pub fn fall_back(pipe: &mut OutputPipe, why: &str) {
    warn!("HDR off for this session: {why}");
    pipe.hdr_active = false;
    pipe.hdr_pass = None;
    pipe.props_applied = false;
    publish(&pipe.output.name(), None);
    let c = &pipe.hdr_caps;
    model::stats::registry::base::set_hdr_info(
        false,
        c.hdr_capable(),
        "SDR",
        c.hdr.max_luminance.unwrap_or(0.0),
        c.colorimetry.bt2020_rgb,
        "8-bit sRGB",
    );
}

/// Compose a pipe's frame through the colour pass when it needs one: always on
/// an HDR pipe (PQ encode), and on an SDR pipe while some surface carries a
/// colour description that needs decoding (tone-mapped to SDR white, gamut
/// clipped by the target). Returns the one element the scanout compositor
/// should draw, or `None` to draw the scene directly (an SDR pipe with nothing
/// to decode, or after a failure).
pub fn compose(
    pipe: &mut OutputPipe,
    renderer: &mut GlesRenderer,
    elements: &[SceneElement<GlesRenderer>],
    scale: f64,
) -> Option<SceneElement<GlesRenderer>> {
    let hdr = pipe.hdr_active;
    if !hdr && !describe::any_decoding() {
        // Nothing to decode on SDR: free the intermediate.
        pipe.hdr_pass = None;
        return None;
    }
    let color = if hdr {
        output_color(&pipe.hdr_caps)
    } else {
        OutputColor::sdr()
    };
    let size = pipe
        .output
        .current_transform()
        .transform_size(pipe.mode.size);
    let scale = Scale::from(scale);
    if !pipe.hdr_pass.as_ref().is_some_and(|p| p.fits(size, scale)) {
        match HdrPass::new(renderer, size, scale) {
            Ok(pass) => {
                info!(
                    "colour pass ({}): {}x{} FP16 intermediate",
                    if hdr { "HDR" } else { "SDR decode" },
                    size.w,
                    size.h
                );
                pipe.hdr_pass = Some(pass);
                if hdr {
                    publish(&pipe.output.name(), Some(color));
                }
            }
            Err(e) => {
                if hdr {
                    fall_back(pipe, &e);
                } else {
                    sdr_pass_failed(&e);
                }
                return None;
            }
        }
    }
    let pass = pipe.hdr_pass.as_mut()?;
    let tagged = describe::surfaces();
    if let Err(e) = pass.render(renderer, elements, &tagged, &color) {
        if hdr {
            fall_back(pipe, &e);
        } else {
            pipe.hdr_pass = None;
            sdr_pass_failed(&e);
        }
        return None;
    }
    Some(SceneElement::HdrEncode(scenegraph::scene::hdr::Encode(
        pass.encode_element(&color, hdr),
    )))
}

/// An SDR pipe could not decode tagged surfaces: they draw undecoded. Logged
/// once, not every frame.
fn sdr_pass_failed(why: &str) {
    static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        warn!("SDR colour pass unavailable, tagged surfaces draw undecoded: {why}");
    }
}
use smithay::reexports::drm::control::atomic::AtomicModeReq;
use smithay::reexports::drm::control::{
    AtomicCommitFlags, Device as ControlDevice, ResourceHandle, connector, crtc, property,
};

/// Kernel `struct hdr_metadata_infoframe` (CTA-861 static metadata type 1).
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct InfoframeXy {
    x: u16,
    y: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct HdrMetadataInfoframe {
    /// `enum hdmi_eotf`: 0 SDR, 1 HDR gamma, 2 ST 2084 (PQ), 3 HLG.
    eotf: u8,
    /// Static metadata descriptor id (0 = type 1).
    metadata_type: u8,
    display_primaries: [InfoframeXy; 3],
    white_point: InfoframeXy,
    /// cd/m² (1-nit units).
    max_display_mastering_luminance: u16,
    /// 0.0001 cd/m² units.
    min_display_mastering_luminance: u16,
    /// cd/m².
    max_cll: u16,
    /// cd/m².
    max_fall: u16,
}

/// Kernel `struct hdr_output_metadata`. The blob the connector property takes.
#[repr(C)]
#[derive(Clone, Copy)]
struct HdrOutputMetadata {
    /// 0 = HDMI_STATIC_METADATA_TYPE1.
    metadata_type: u32,
    infoframe: HdrMetadataInfoframe,
}

/// EOTF code for SMPTE ST 2084 (PQ).
const EOTF_PQ: u8 = 2;
const EOTF_HLG: u8 = 3;

/// CIE xy → kernel 0.00002-unit fixed point (value = coord * 50000).
fn xy(coord: (f32, f32)) -> InfoframeXy {
    InfoframeXy {
        x: (coord.0 * 50000.0).round().clamp(0.0, 65535.0) as u16,
        y: (coord.1 * 50000.0).round().clamp(0.0, 65535.0) as u16,
    }
}

/// BT.2020 primaries (used when the EDID chromaticity is absent/zero).
const BT2020: ([(f32, f32); 3], (f32, f32)) = (
    [(0.708, 0.292), (0.170, 0.797), (0.131, 0.046)],
    (0.3127, 0.3290),
);

fn build_metadata(caps: &HdrInfo) -> HdrOutputMetadata {
    let p = &caps.primaries;
    // Use EDID primaries when present (non-zero), else fall back to BT.2020.
    let have_edid = p.red.0 > 0.0 && p.green.0 > 0.0 && p.blue.0 > 0.0 && p.white.0 > 0.0;
    let (prim, white) = if have_edid {
        ([p.red, p.green, p.blue], p.white)
    } else {
        BT2020
    };
    let max = caps.hdr.max_luminance.unwrap_or(0.0);
    let min = caps.hdr.min_luminance.unwrap_or(0.0);
    let fall = caps.hdr.max_frame_avg_luminance.unwrap_or(0.0);
    let eotf = if caps.hdr.eotf_pq { EOTF_PQ } else { EOTF_HLG };
    HdrOutputMetadata {
        metadata_type: 0,
        infoframe: HdrMetadataInfoframe {
            eotf,
            metadata_type: 0,
            display_primaries: [xy(prim[0]), xy(prim[1]), xy(prim[2])],
            white_point: xy(white),
            max_display_mastering_luminance: max.round().clamp(0.0, 65535.0) as u16,
            min_display_mastering_luminance: (min * 10000.0).round().clamp(0.0, 65535.0) as u16,
            max_cll: max.round().clamp(0.0, 65535.0) as u16,
            max_fall: fall.round().clamp(0.0, 65535.0) as u16,
        },
    }
}

/// Look up a property handle by name on a connector, CRTC or any DRM object.
fn prop_handle<H: ResourceHandle>(
    drm: &DrmDeviceFd,
    conn: H,
    name: &str,
) -> Result<property::Handle, String> {
    let set = drm
        .get_properties(conn)
        .map_err(|e| format!("get_properties: {e}"))?;
    for h in set.as_props_and_values().0 {
        if let Ok(info) = drm.get_property(*h)
            && info.name().to_str() == Ok(name)
        {
            return Ok(*h);
        }
    }
    Err(format!("object lacks property {name}"))
}

/// Raw value of an enum property's entry named `name`.
fn enum_value(drm: &DrmDeviceFd, handle: property::Handle, name: &str) -> Result<u64, String> {
    let info = drm
        .get_property(handle)
        .map_err(|e| format!("get_property: {e}"))?;
    if let property::ValueType::Enum(values) = info.value_type() {
        let (raws, enums) = values.values();
        for (raw, ev) in raws.iter().zip(enums.iter()) {
            if ev.name().to_str() == Ok(name) {
                return Ok(*raw);
            }
        }
        return Err(format!("enum has no entry {name}"));
    }
    Err("not an enum property".into())
}

/// A connector property handle by name, or `None` when the connector lacks it.
/// Absence is normal (not every connector exposes `Colorspace` or `max bpc`) and
/// must not fail the whole pass.
fn opt_prop<H: ResourceHandle>(drm: &DrmDeviceFd, conn: H, name: &str) -> Option<property::Handle> {
    prop_handle(drm, conn, name).ok()
}

/// The connector's CURRENT raw value for `handle` — what the previous owner of
/// this connector left behind. Logged so a stale value is visible as a fact.
fn current_raw<H: ResourceHandle>(
    drm: &DrmDeviceFd,
    conn: H,
    handle: property::Handle,
) -> Option<u64> {
    let set = drm.get_properties(conn).ok()?;
    let (handles, values) = set.as_props_and_values();
    handles
        .iter()
        .zip(values.iter())
        .find(|(h, _)| **h == handle)
        .map(|(_, v)| *v)
}

/// Clamp `want` into an unsigned-range property's advertised bounds.
fn clamp_range(drm: &DrmDeviceFd, handle: property::Handle, want: u64) -> u64 {
    match drm.get_property(handle).map(|i| i.value_type()) {
        Ok(property::ValueType::UnsignedRange(lo, hi)) => want.clamp(lo, hi),
        _ => want,
    }
}

/// What the pass actually did, per property, for the log line.
pub struct PropOutcome {
    pub colorspace: String,
    pub metadata: String,
    pub max_bpc: String,
    /// The CRTC colour pipeline (DEGAMMA_LUT, CTM, GAMMA_LUT) as found, and reset.
    pub crtc_color: String,
}

/// Bring the connector's colorimetry and bit-depth properties in line with what
/// this pipe is ACTUALLY driving.
///
/// These are sticky atomic connector properties: they survive our own page flips,
/// and they survive US — whatever ran on another VT leaves its values behind for
/// the next compositor to inherit. The previous version of this code only ever
/// SET BT.2020 + PQ, and only when HDR was active, so an SDR pipe that inherited
/// BT.2020 (from an earlier HDR session, from a pipe that fell back to SDR after a
/// signalling failure, or from another compositor on another VT) went on
/// displaying sRGB content as BT.2020 — heavily oversaturated and red-shifted,
/// with no code path anywhere that would ever put it back. So the SDR case is now
/// written EXPLICITLY rather than left alone.
///
/// `max bpc` is READ but deliberately NOT written.
///
/// Writing it was tried and REVERTED. It governs the LINK depth, so raising a
/// connector from 8 to 10 bpc costs ~25% more link bandwidth. On a shared display
/// engine that is enough to push a SECOND pipe's modeset over the limit: every
/// tiled/CCS format then fails the atomic test, smithay reacts by forcing implicit
/// modifiers across the whole device, and the Vulkan renderer cannot create images
/// for implicit-modifier buffers — so the machine goes fully black rather than
/// merely losing a bit of colour depth. It is also a STICKY property, so the
/// damage outlives the process that did it.
///
/// If 10-bit link depth is wanted, it has to be negotiated as part of the modeset
/// (validated against the whole device's bandwidth with every pipe present), not
/// poked in afterwards one connector at a time.
///
/// The CRTC colour pipeline is reset too: `DEGAMMA_LUT`, `CTM` and `GAMMA_LUT`
/// are cleared (blob 0, identity). They are just as sticky, and a compositor that
/// offloads its colour pipeline leaves them loaded: KWin in HDR keeps gamma-encoded
/// SDR in its framebuffer and lets `GAMMA_LUT` produce PQ, so viud on the next VT
/// sent its own PQ pixels through that LUT a second time. The double encoding
/// lifted every grey into a grey film (desk4 and the LG, 2026-11-09). viud does
/// all colour work in its shaders, so the pipeline must be identity.
///
/// Missing properties are skipped, not fatal. Every commit is preceded by a
/// `TEST_ONLY` commit, so a rejected request can never blank the display.
pub fn apply_connector_props(
    drm: &DrmDeviceFd,
    conn: connector::Handle,
    crtc: crtc::Handle,
    caps: &HdrInfo,
    hdr_active: bool,
    want_bpc: u64,
) -> Result<PropOutcome, String> {
    let cs_prop = opt_prop(drm, conn, "Colorspace");
    let meta_prop = opt_prop(drm, conn, "HDR_OUTPUT_METADATA");
    let bpc_prop = opt_prop(drm, conn, "max bpc");

    let mut outcome = PropOutcome {
        colorspace: "absent".into(),
        metadata: "absent".into(),
        max_bpc: "absent".into(),
        crtc_color: String::new(),
    };

    // CRTC colour pipeline: clear whatever the previous owner loaded.
    let mut luts: Vec<property::Handle> = Vec::new();
    for name in ["DEGAMMA_LUT", "CTM", "GAMMA_LUT"] {
        if let Some(h) = opt_prop(drm, crtc, name) {
            let was = current_raw(drm, crtc, h).unwrap_or(0);
            let state = if was == 0 {
                "none".to_string()
            } else {
                format!("blob({was})->cleared")
            };
            outcome.crtc_color.push_str(&format!("{name}={state} "));
            luts.push(h);
        }
    }

    // Target colorspace: BT.2020 only while HDR is actually driving this pipe,
    // Default otherwise — the reset half that never existed before.
    let cs_target = if hdr_active { "BT2020_RGB" } else { "Default" };
    let cs = match cs_prop {
        Some(h) => match enum_value(drm, h, cs_target) {
            Ok(v) => {
                let was = current_raw(drm, conn, h);
                outcome.colorspace = format!("{was:?} -> {cs_target}({v})");
                Some((h, v))
            }
            Err(e) => {
                outcome.colorspace = format!("no {cs_target} entry: {e}");
                None
            }
        },
        None => None,
    };

    // HDR metadata blob: the PQ infoframe while HDR is active, explicitly CLEARED
    // (blob 0) otherwise, so a stale infoframe cannot outlive the HDR session.
    let mut blob_raw: Option<u64> = None;
    let meta = match meta_prop {
        Some(h) => {
            let was = current_raw(drm, conn, h);
            if hdr_active {
                let metadata = build_metadata(caps);
                let blob = drm
                    .create_property_blob(&metadata)
                    .map_err(|e| format!("create_property_blob: {e}"))?;
                let raw: u64 = blob.into();
                blob_raw = Some(raw);
                outcome.metadata = format!("{was:?} -> PQ blob({raw})");
                Some((h, raw))
            } else {
                outcome.metadata = format!("{was:?} -> cleared(0)");
                Some((h, 0u64))
            }
        }
        None => None,
    };

    // OBSERVE ONLY — deliberately NOT written. See the note on `want_bpc` above.
    // Reading it is still worth doing: a stale value left by another compositor is
    // exactly the kind of thing we want visible in a log.
    if let Some(h) = bpc_prop {
        let was = current_raw(drm, conn, h);
        let would = clamp_range(drm, h, want_bpc);
        outcome.max_bpc = format!(
            "{was:?} (observed only, NOT written; would have been {would} for depth request {want_bpc})"
        );
    }
    let bpc: Option<(property::Handle, u64)> = None;

    if cs.is_none() && meta.is_none() && bpc.is_none() && luts.is_empty() {
        return Ok(outcome);
    }

    let build = || {
        let mut req = AtomicModeReq::new();
        if let Some((h, v)) = cs {
            req.add_property(conn, h, property::Value::UnsignedRange(v));
        }
        if let Some((h, v)) = meta {
            req.add_property(conn, h, property::Value::Blob(v));
        }
        if let Some((h, v)) = bpc {
            req.add_property(conn, h, property::Value::UnsignedRange(v));
        }
        for h in &luts {
            req.add_property(crtc, *h, property::Value::Blob(0));
        }
        req
    };

    // Validate before committing — a bad blob fails here, never on screen.
    if let Err(e) = drm.atomic_commit(
        AtomicCommitFlags::TEST_ONLY | AtomicCommitFlags::ALLOW_MODESET,
        build(),
    ) {
        if let Some(b) = blob_raw {
            let _ = drm.destroy_property_blob(b);
        }
        return Err(format!(
            "connector-props TEST commit rejected: {e} [{}|{}|{}|{}]",
            outcome.colorspace, outcome.metadata, outcome.max_bpc, outcome.crtc_color
        ));
    }
    if let Err(e) = drm.atomic_commit(AtomicCommitFlags::ALLOW_MODESET, build()) {
        if let Some(b) = blob_raw {
            let _ = drm.destroy_property_blob(b);
        }
        return Err(format!(
            "connector-props commit failed: {e} [{}|{}|{}|{}]",
            outcome.colorspace, outcome.metadata, outcome.max_bpc, outcome.crtc_color
        ));
    }
    // The HDR blob is intentionally leaked for as long as the property references
    // it; it is replaced (or cleared) by the next pass on this connector.
    Ok(outcome)
}
