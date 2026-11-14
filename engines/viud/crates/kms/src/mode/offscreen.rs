// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Read-only mode discovery: no seat, DRM master, modeset or forced probe.
use smithay::output::Mode;
use smithay::reexports::drm::control::Device;
use std::{
    os::fd::{AsFd, BorrowedFd},
    path::Path,
};

struct Probe(std::fs::File);
impl AsFd for Probe {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}
impl smithay::reexports::drm::Device for Probe {}
impl Device for Probe {}

/// Follow the same connector/profile policy as KMS, when its card is readable.
pub fn discover(render_node: &str, scanout_node: &str) -> Option<(Mode, smithay::output::Output)> {
    let path = if scanout_node.is_empty() {
        crate::device::node::render_node(Path::new(render_node))
            .and_then(crate::device::node::primary_node)?
            .dev_path()?
    } else {
        scanout_node.into()
    };
    let probe = Probe(std::fs::File::open(path).ok()?);
    let resources = probe.resource_handles().ok()?;
    let connectors = resources
        .connectors()
        .iter()
        .filter_map(|c| probe.get_connector(*c, false).ok())
        .collect();
    let profiles = render_gles::preference::output::profile::get();
    let connector = crate::connector::select::select(&probe, connectors, &profiles)?;
    let raw = crate::edid::parse::read(&probe, &connector);
    let parsed = raw.as_ref().and_then(crate::edid::parse::parse);
    let key = crate::edid::identity::connector_key(parsed.as_ref(), &connector);
    let profile =
        model::environment::preference::identity::find(&profiles, &key, |p| p.identity.as_deref())
            .or_else(|| profiles.iter().find(|p| p.identity.is_none()));
    let default = render_gles::preference::output::profile::default_mode().map(|mode| {
        render_gles::preference::output::profile::OutputProfile {
            mode: Some(mode),
            ..Default::default()
        }
    });
    let mode =
        crate::mode::select::select(&connector, profile.or(default.as_ref())).map(Mode::from)?;
    let identity = crate::edid::identity::for_connector(parsed.as_ref(), &connector);
    let output = crate::output::physical::create(&connector, &identity);
    Some((mode, output))
}
