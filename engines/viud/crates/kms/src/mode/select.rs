// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Mode selection policy (ex wire.rs): area -> refresh -> PREFERRED,
//! lexicographic. Consumes preference values when a profile requests an
//! advertised mode; otherwise unchanged default policy.

use render_gles::preference::output::profile::{ModeRequest, OutputProfile};
use smithay::reexports::drm::control::{Mode as DrmMode, ModeFlags, ModeTypeFlags, connector};

/// Whether `m` is interlaced. An interlaced mode must never be chosen over a
/// progressive one: the scanout planes reject tiled/CCS buffers on an interlaced
/// CRTC, so every explicit-modifier format fails the atomic test. smithay reacts
/// to that by falling the WHOLE DEVICE back to implicit modifiers — and the
/// Vulkan renderer cannot create images for implicit-modifier buffers at all, so
/// the result is not a degraded pipe but a blank screen on every output.
/// Observed live: a 1280x1024 panel advertising `1920x1080i` took the whole
/// compositor dark once it was plugged in as a second monitor.
pub fn is_interlaced(m: &DrmMode) -> bool {
    m.flags().contains(ModeFlags::INTERLACE)
}

/// Default policy: progressive first, then area -> refresh -> PREFERRED.
///
/// `!is_interlaced` is the HIGHEST-priority key, ahead of area — an interlaced
/// mode is only ever selected when the connector advertises nothing else.
pub fn select_default(info: &connector::Info) -> Option<DrmMode> {
    // HDR: among same-size, same-refresh timings prefer the one closest to the
    // nominal rate, i.e. the CTA-861 video timing (4K60: VIC 97, 594 MHz,
    // 60.000 Hz) over an EDID reduced-blanking one (533.25 MHz, 59.997 Hz).
    // That is the timing KWin drives this LG with in HDR; on the reduced-blanking
    // timing the same PQ signal showed a grey film (desk4, 2026-11-09).
    let hdr = model::environment::config::base::try_get().is_some_and(|e| e.hdr);
    info.modes()
        .iter()
        .max_by_key(|m| {
            let (w, h) = m.size();
            let area = (w as u64) * (h as u64);
            let refresh = m.vrefresh();
            let nominal = if hdr { exact_refresh_mhz(m) } else { 0 };
            let is_preferred = m.mode_type().contains(ModeTypeFlags::PREFERRED);
            (!is_interlaced(m), area, refresh, nominal, is_preferred)
        })
        .copied()
}

/// The mode's exact refresh in mHz from its pixel clock and totals.
pub fn exact_refresh_mhz(m: &DrmMode) -> u64 {
    let (htotal, vtotal) = (m.hsync().2 as u64, m.vsync().2 as u64);
    if htotal == 0 || vtotal == 0 {
        return 0;
    }
    (m.clock() as u64) * 1_000_000 / (htotal * vtotal)
}

/// Preference-aware selection: an `Advertised` request narrows the list; any
/// synthesis request is NOT handled here (that is `mode.synthesize`, Law 7).
pub fn select(info: &connector::Info, profile: Option<&OutputProfile>) -> Option<DrmMode> {
    if let Some(OutputProfile {
        mode:
            Some(ModeRequest::Advertised {
                width,
                height,
                refresh_mhz,
            }),
        ..
    }) = profile
    {
        // A requested WxH@R can be advertised both progressive and interlaced
        // (1920x1080 vs 1920x1080i). Match the progressive one first — see
        // `is_interlaced` for why taking the interlaced variant blanks the device.
        let matches = |m: &&DrmMode| {
            let (w, h) = m.size();
            w == *width && h == *height && m.vrefresh() * 1000 == *refresh_mhz
        };
        // HDR: several timings can share WxH@R (see `select_default`); take the
        // one closest to nominal (first among equals). SDR keeps the first.
        let hdr = model::environment::config::base::try_get().is_some_and(|e| e.hdr);
        let hit = if hdr {
            info.modes()
                .iter()
                .filter(|m| matches(m) && !is_interlaced(m))
                .rev()
                .max_by_key(|m| exact_refresh_mhz(m))
        } else {
            info.modes().iter().find(|m| matches(m) && !is_interlaced(m))
        }
        .or_else(|| info.modes().iter().find(matches));
        if let Some(m) = hit {
            return Some(*m);
        }
        warn!("requested advertised mode not found; falling back to default policy");
    }
    select_default(info)
}

pub fn log_selected(mode: &DrmMode) {
    info!(
        "selected mode: {}x{} @ {}Hz, type: {:?}",
        mode.size().0,
        mode.size().1,
        mode.vrefresh(),
        mode.mode_type(),
    );
}
