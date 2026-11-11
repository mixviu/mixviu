// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Panel-only blanking, after plane blending. Scene buffers, damage, capture
//! and client pacing are deliberately outside this module.

use super::hdr::{current_raw, opt_prop};
use crate::context::render::NativeRenderContext;
use smithay::backend::drm::DrmDeviceFd;
use smithay::reexports::drm::control::{
    AtomicCommitFlags, Device as _, atomic::AtomicModeReq, crtc, property,
};
use std::{cell::RefCell, os::fd::AsFd, rc::Rc};
use world::state::Loop;

/// The property and payload chosen from the CRTC's advertised colour pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Plan {
    Ctm,
    Gamma(usize),
}

fn plan(ctm: bool, gamma_size: Option<u64>) -> Option<Plan> {
    if ctm {
        Some(Plan::Ctm)
    } else {
        gamma_size
            .filter(|n| *n > 0 && *n <= 1_048_576)
            .map(|n| Plan::Gamma(n as usize))
    }
}

trait Surface {
    fn property(&self, name: &str) -> Option<u32>;
    fn value(&self, handle: u32) -> Option<u64>;
    fn blob(&self, bytes: &mut [u8]) -> Result<u64, String>;
    fn commit(&self, props: &[(u32, u64)], test: bool) -> Result<(), String>;
    fn destroy(&self, blob: u64);
}

struct DrmSurface<'a> {
    drm: &'a DrmDeviceFd,
    crtc: crtc::Handle,
}
fn drm_handle(raw: u32) -> property::Handle {
    // Raw handles originate exclusively from opt_prop's NonZeroU32 handles.
    std::num::NonZeroU32::new(raw)
        .expect("DRM property handle is nonzero")
        .into()
}
impl Surface for DrmSurface<'_> {
    fn property(&self, name: &str) -> Option<u32> {
        opt_prop(self.drm, self.crtc, name).map(u32::from)
    }
    fn value(&self, handle: u32) -> Option<u64> {
        current_raw(self.drm, self.crtc, drm_handle(handle))
    }
    fn blob(&self, bytes: &mut [u8]) -> Result<u64, String> {
        drm_ffi::mode::create_property_blob(self.drm.as_fd(), bytes)
            .map(|b| u64::from(b.blob_id))
            .map_err(|e| format!("blank blob: {e}"))
    }
    fn commit(&self, props: &[(u32, u64)], test: bool) -> Result<(), String> {
        let mut req = AtomicModeReq::new();
        for (handle, value) in props {
            req.add_property(
                self.crtc,
                drm_handle(*handle),
                property::Value::Blob(*value),
            );
        }
        let flags = AtomicCommitFlags::ALLOW_MODESET
            | if test {
                AtomicCommitFlags::TEST_ONLY
            } else {
                AtomicCommitFlags::empty()
            };
        self.drm
            .atomic_commit(flags, req)
            .map_err(|e| format!("blank {}commit: {e}", if test { "TEST_ONLY " } else { "" }))
    }
    fn destroy(&self, blob: u64) {
        let _ = self.drm.destroy_property_blob(blob);
    }
}

fn selection(surface: &impl Surface) -> Option<(u32, Plan)> {
    let ctm = surface.property("CTM");
    let gamma = surface.property("GAMMA_LUT");
    let size = gamma
        .and_then(|_| surface.property("GAMMA_LUT_SIZE"))
        .and_then(|h| surface.value(h));
    match plan(ctm.is_some(), size)? {
        Plan::Ctm => Some((ctm?, Plan::Ctm)),
        p @ Plan::Gamma(_) => Some((gamma?, p)),
    }
}

fn tested(surface: &impl Surface, props: &[(u32, u64)]) -> Result<(), String> {
    surface.commit(props, true)?;
    surface.commit(props, false)
}

fn restore(surface: &impl Surface, prop: u32, blob: u64) -> Result<(), String> {
    if surface.value(prop) == Some(blob) {
        return Ok(());
    }
    tested(surface, &[(prop, blob)])
}

fn unload(surface: &impl Surface, prop: u32, blob: u64) -> Result<(), String> {
    tested(surface, &[(prop, 0)])?;
    surface.destroy(blob);
    Ok(())
}

fn color_plan(surface: &impl Surface, owned: Option<(u32, u64)>) -> Vec<(u32, u64)> {
    ["DEGAMMA_LUT", "CTM", "GAMMA_LUT"]
        .into_iter()
        .filter_map(|name| surface.property(name))
        .filter_map(|h| {
            let target = owned
                .filter(|(prop, _)| *prop == h)
                .map_or(0, |(_, blob)| blob);
            (surface.value(h).unwrap_or(0) != target).then_some((h, target))
        })
        .collect()
}

/// The connector pass uses this same plan for SDR and HDR. Blanking owns the
/// selected property; every other stale colour property goes back to identity.
pub(super) fn connector_color_plan(
    drm: &DrmDeviceFd,
    crtc: crtc::Handle,
    owned: Option<(property::Handle, u64)>,
) -> Vec<(property::Handle, u64)> {
    color_plan(&DrmSurface { drm, crtc }, owned.map(|(h, b)| (h.into(), b)))
        .into_iter()
        .map(|(h, b)| (drm_handle(h), b))
        .collect()
}

fn prepare(surface: &impl Surface) -> Result<(u32, u64, Vec<(u32, u64)>), String> {
    let (handle, plan) = selection(surface).ok_or("no CTM or usable GAMMA_LUT")?;
    // drm_color_ctm: nine u64 coefficients. drm_color_lut: RGB + reserved,
    // four u16 values per advertised entry. All-zero bytes encode both exactly.
    let bytes = match plan {
        Plan::Ctm => 9 * 8,
        Plan::Gamma(n) => n * 8,
    };
    let blob = surface.blob(&mut vec![0; bytes])?;
    let mut props: Vec<_> = ["DEGAMMA_LUT", "CTM", "GAMMA_LUT"]
        .into_iter()
        .filter_map(|n| surface.property(n))
        .map(|h| (h, 0))
        .collect();
    for (h, v) in &mut props {
        if *h == handle {
            *v = blob;
        }
    }
    Ok((handle, blob, props))
}

fn load(surface: &impl Surface) -> Result<(u32, u64), String> {
    let (handle, blob, props) = prepare(surface)?;
    if let Err(e) = tested(surface, &props) {
        surface.destroy(blob);
        return Err(e);
    }
    Ok((handle, blob))
}

struct Change {
    crtc: crtc::Handle,
    props: Vec<(u32, u64)>,
}

trait AtomicDevice {
    fn commit(&self, changes: &[Change], test: bool) -> Result<(), String>;
}

impl AtomicDevice for DrmDeviceFd {
    fn commit(&self, changes: &[Change], test: bool) -> Result<(), String> {
        let mut req = AtomicModeReq::new();
        for change in changes {
            for (h, blob) in &change.props {
                req.add_property(change.crtc, drm_handle(*h), property::Value::Blob(*blob));
            }
        }
        let flags = AtomicCommitFlags::ALLOW_MODESET
            | if test {
                AtomicCommitFlags::TEST_ONLY
            } else {
                AtomicCommitFlags::empty()
            };
        self.atomic_commit(flags, req)
            .map_err(|e| format!("blank {}commit: {e}", if test { "TEST_ONLY " } else { "" }))
    }
}

fn transaction(device: &impl AtomicDevice, changes: &[Change]) -> Result<(), String> {
    if changes.is_empty() {
        return Ok(());
    }
    device.commit(changes, true)?;
    device.commit(changes, false)
}

/// Owns the sticky property and its userspace blob reference. Dropping a pipe
/// clears the property before freeing its CRTC, including on clean shutdown.
pub struct Blank {
    loaded: Option<Loaded>,
    pub failed: bool,
    pub pending_check: bool,
}
impl Default for Blank {
    fn default() -> Self {
        Self {
            loaded: None,
            failed: false,
            pending_check: true,
        }
    }
}
struct Loaded {
    drm: DrmDeviceFd,
    crtc: crtc::Handle,
    prop: u32,
    blob: u64,
}
impl Drop for Loaded {
    fn drop(&mut self) {
        if self.blob == 0 {
            return;
        }
        let surface = DrmSurface {
            drm: &self.drm,
            crtc: self.crtc,
        };
        if let Err(e) = tested(&surface, &[(self.prop, 0)]) {
            warn!("blank cleanup crtc {:?}: {e}", self.crtc);
        }
        surface.destroy(self.blob);
    }
}
impl Blank {
    pub fn preserved(&self) -> Option<(property::Handle, u64)> {
        self.loaded.as_ref().map(|l| (drm_handle(l.prop), l.blob))
    }
    fn clear(&mut self) -> Result<(), String> {
        if let Some(l) = self.loaded.as_ref() {
            unload(
                &DrmSurface {
                    drm: &l.drm,
                    crtc: l.crtc,
                },
                l.prop,
                l.blob,
            )?;
        }
        // The second clear in Loaded::drop is harmless but unnecessary.
        if let Some(mut l) = self.loaded.take() {
            l.blob = 0;
        }
        Ok(())
    }
    fn apply(&mut self, drm: &DrmDeviceFd, crtc: crtc::Handle) -> Result<(), String> {
        if let Some(l) = &self.loaded {
            return restore(&DrmSurface { drm, crtc }, l.prop, l.blob);
        }
        let (prop, blob) = load(&DrmSurface { drm, crtc })?;
        self.loaded = Some(Loaded {
            drm: drm.clone(),
            crtc,
            prop,
            blob,
        });
        Ok(())
    }
}

pub fn supported(ctx: &NativeRenderContext) -> bool {
    let mut live = ctx
        .outputs
        .iter()
        .filter(|p| p.drm_output.is_some())
        .peekable();
    live.peek().is_some()
        && live.all(|p| {
            !p.blank.failed
                && selection(&DrmSurface {
                    drm: &ctx.drm_fd,
                    crtc: p.crtc,
                })
                .is_some()
        })
}

pub fn apply(ctx: &mut NativeRenderContext, on: bool) -> Result<(), String> {
    // Prepare every blob first, then one device-wide TEST/real transaction.
    // No output can be left half transitioned if another CRTC refuses.
    let mut changes = Vec::new();
    let mut allocated = Vec::new();
    let mut names = Vec::new();
    for (idx, p) in ctx.outputs.iter_mut().enumerate() {
        if on && p.drm_output.is_none() {
            continue;
        }
        if on {
            let surface = DrmSurface {
                drm: &ctx.drm_fd,
                crtc: p.crtc,
            };
            let props = if let Some(l) = &p.blank.loaded {
                color_plan(&surface, Some((l.prop, l.blob)))
            } else {
                match prepare(&surface) {
                    Ok((prop, blob, props)) => {
                        allocated.push((idx, prop, blob));
                        props
                    }
                    Err(e) => {
                        p.blank.failed = true;
                        for (_, _, blob) in allocated {
                            let _ = ctx.drm_fd.destroy_property_blob(blob);
                        }
                        return Err(format!("{}: {e}", p.output.name()));
                    }
                }
            };
            if !props.is_empty() {
                changes.push(Change {
                    crtc: p.crtc,
                    props,
                });
            }
            names.push(p.output.name());
        } else if let Some(l) = &p.blank.loaded {
            changes.push(Change {
                crtc: p.crtc,
                props: vec![(l.prop, 0)],
            });
            names.push(p.output.name());
        }
    }
    if on && names.is_empty() {
        return Err("no active output supports blanking".into());
    }
    if let Err(e) = transaction(&ctx.drm_fd, &changes) {
        for (_, _, blob) in allocated {
            let _ = ctx.drm_fd.destroy_property_blob(blob);
        }
        if on {
            for p in &mut ctx.outputs {
                if names.contains(&p.output.name()) {
                    p.blank.failed = true;
                }
            }
        }
        return Err(format!("{}: {e}", names.join(", ")));
    }
    if on {
        for (idx, prop, blob) in allocated {
            ctx.outputs[idx].blank.loaded = Some(Loaded {
                drm: ctx.drm_fd.clone(),
                crtc: ctx.outputs[idx].crtc,
                prop,
                blob,
            });
        }
    } else {
        for p in &mut ctx.outputs {
            if let Some(mut l) = p.blank.loaded.take() {
                let _ = l.drm.destroy_property_blob(l.blob);
                l.blob = 0; // property already cleared by the atomic transaction
            }
        }
    }
    for p in &mut ctx.outputs {
        p.blank.failed = false;
    }
    ctx.blank_requested = on;
    Ok(())
}

/// Pause / DPMS cleanup preserves the logical request for resume.
pub fn release(ctx: &mut NativeRenderContext) {
    let requested = ctx.blank_requested;
    if let Err(e) = apply(ctx, false) {
        warn!("blank release: {e}");
    }
    ctx.blank_requested = requested;
    for p in &mut ctx.outputs {
        p.blank.pending_check = true;
    }
}

pub fn retire(pipe: &mut crate::context::render::OutputPipe) {
    if let Err(e) = pipe.blank.clear() {
        warn!("blank retire {}: {e}", pipe.output.name());
    }
    pipe.blank.failed = false;
}

/// Called at pipe rebuild/resume and before connector-property resets. A failed
/// output is parked until a new explicit action/rebuild; never retried per frame.
pub fn refresh(state: &mut Loop, ctx: &mut NativeRenderContext) {
    if ctx.blank_requested {
        for p in &mut ctx.outputs {
            // Fresh smithay outputs may not yet have committed their first
            // modeset. Wait for their first flip rather than recording an
            // inactive-CRTC rejection as permanent lack of support.
            let active = ctx
                .drm_fd
                .get_crtc(p.crtc)
                .ok()
                .and_then(|c| c.mode())
                .is_some();
            if p.drm_output.is_some()
                && active
                && !p.blank.failed
                && let Err(e) = p.blank.apply(&ctx.drm_fd, p.crtc)
            {
                p.blank.failed = true;
                warn!("blank unavailable on {}: {e}", p.output.name());
            }
        }
    }
    world::comp::blank::set_supported(state, supported(ctx));
}

/// One check after a modeset/resume/hotplug's first completed flip. Normal
/// vblanks perform no ioctls or property work here.
pub fn after_flip(state: &mut Loop, ctx: &mut NativeRenderContext, crtc: crtc::Handle) {
    let Some(idx) = ctx.outputs.iter().position(|p| p.crtc == crtc) else {
        return;
    };
    if !ctx.outputs[idx].blank.pending_check {
        return;
    }
    refresh(state, ctx);
    ctx.outputs[idx].blank.pending_check = false;
}

pub fn install(state: &mut Loop, ctx: &Rc<RefCell<NativeRenderContext>>) {
    world::comp::blank::set_supported(state, supported(&ctx.borrow()));
    let weak = Rc::downgrade(ctx);
    state.inner.comp.blank.backend = Some(Rc::new(move |on| {
        let ctx = weak.upgrade().ok_or("native display stopped")?;
        let result = apply(&mut ctx.borrow_mut(), on);
        result
    }));
}

pub fn shutdown(ctx: &Rc<RefCell<NativeRenderContext>>) {
    release(&mut ctx.borrow_mut());
}

#[cfg(test)]
mod tests;
