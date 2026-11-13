// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Start the render-node desk before acquiring a VT-bound seat.
use smithay::reexports::calloop::EventLoop;
mod activation;
mod test_seat;
pub(crate) use activation::watcher_failed;
use std::{cell::RefCell, ffi::OsString, rc::Rc};
use world::state::{Loop, StatusSession};

pub type Handles = Rc<RefCell<Option<native::wire::entry::NativeHandles>>>;

pub struct InitialOutput {
    mode: smithay::output::Mode,
    output: Option<smithay::output::Output>,
}

/// Configured advertised mode is known before seat acquisition. Once KMS is
/// attached the shared Output carries its last live mode across every pause.
pub fn initial_mode() -> Result<InitialOutput, Box<dyn std::error::Error>> {
    let config = model::environment::config::base::get();
    let discovered = kms::mode::offscreen::discover(&config.render_node, &config.scanout_node);
    match std::env::var("MIX_VIUD_OFFSCREEN_SIZE") {
        Ok(value) => {
            return Ok(InitialOutput {
                mode: smithay::output::Mode {
                    size: override_size(&value).into(),
                    refresh: discovered.as_ref().map_or(60_000, |(mode, _)| mode.refresh),
                },
                output: discovered.map(|(_, output)| output),
            });
        }
        Err(std::env::VarError::NotUnicode(_)) => {
            warn!("MIX_VIUD_OFFSCREEN_SIZE is not UTF-8; using 3840x2160");
            return Ok(InitialOutput {
                mode: smithay::output::Mode {
                    size: (3840, 2160).into(),
                    refresh: 60_000,
                },
                output: discovered.map(|(_, output)| output),
            });
        }
        Err(std::env::VarError::NotPresent) => {}
    }
    if let Some((mode, output)) = discovered {
        return Ok(InitialOutput {
            mode,
            output: Some(output),
        });
    }
    use render_gles::preference::output::profile::{self, ModeRequest};
    let mode = profile::get()
        .into_iter()
        .find(|p| p.active)
        .and_then(|p| p.mode)
        .or_else(profile::default_mode);
    let size = match mode {
        Some(
            ModeRequest::Advertised { width, height, .. } | ModeRequest::Cvt { width, height, .. },
        ) => (i32::from(width), i32::from(height)),
        _ => (3840, 2160),
    };
    Ok(InitialOutput {
        mode: smithay::output::Mode {
            size: size.into(),
            refresh: 60_000,
        },
        output: None,
    })
}

pub fn wire(
    state: &mut Loop,
    event_loop: &mut EventLoop<'static, Loop>,
    socket: OsString,
    initial: InitialOutput,
    scale: f64,
    render_node: &str,
) -> Result<Handles, Box<dyn std::error::Error>> {
    let InitialOutput { mode, output } = initial;
    state.inner.status_session = StatusSession::Paused;
    let output = headless::wire::install(
        state,
        &event_loop.handle(),
        headless::wire::Options {
            size: mode.size,
            scale,
            render_node,
        },
        Some(mode),
        output,
    )?;
    headless::wire::vt_session(state);
    match std::env::var("MIX_TEST_VIUD_SEAT_INACTIVE").ok().as_deref() {
        Some("1") => Ok(Rc::new(RefCell::new(None))),
        Some(value) if value.starts_with("fd:") => {
            test_seat::install(state, output, render_node, &value[3..])?;
            Ok(Rc::new(RefCell::new(None)))
        }
        _ => activation::install(state, socket, output),
    }
}

fn override_size(value: &str) -> (i32, i32) {
    crate::cli::parse_size(value).unwrap_or_else(|error| {
        warn!("invalid MIX_VIUD_OFFSCREEN_SIZE ({error}); using 3840x2160");
        (3840, 2160)
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn malformed_offscreen_sizes_use_the_4k_default() {
        for size in ["", "garbage", "0x2160", "3840x0", "9999999999x2160"] {
            assert_eq!(super::override_size(size), (3840, 2160));
        }
        assert_eq!(super::override_size("1920x1080"), (1920, 1080));
    }
}
