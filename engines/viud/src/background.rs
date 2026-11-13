// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Start the render-node desk before acquiring a VT-bound seat.
use smithay::reexports::calloop::{EventLoop, ping::make_ping};
use std::{cell::RefCell, ffi::OsString, rc::Rc};
use world::state::{Loop, StatusSession};

pub type Handles = Rc<RefCell<Option<native::wire::entry::NativeHandles>>>;

/// Configured advertised mode is known before seat acquisition. Once KMS is
/// attached the shared Output carries its last live mode across every pause.
pub fn initial_mode()
-> Result<(smithay::output::Mode, Option<smithay::output::Output>), Box<dyn std::error::Error>> {
    let config = model::environment::config::base::get();
    let discovered = kms::mode::offscreen::discover(&config.render_node, &config.scanout_node);
    if let Ok(size) = std::env::var("MIX_VIUD_OFFSCREEN_SIZE") {
        return crate::cli::parse_size(&size)
            .map(|size| {
                (
                    smithay::output::Mode {
                        size: size.into(),
                        refresh: discovered.as_ref().map_or(60_000, |(mode, _)| mode.refresh),
                    },
                    discovered.map(|(_, output)| output),
                )
            })
            .map_err(Into::into);
    }
    if let Some((mode, output)) = discovered {
        return Ok((mode, Some(output)));
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
    Ok((
        smithay::output::Mode {
            size: size.into(),
            refresh: 60_000,
        },
        None,
    ))
}

pub fn wire(
    state: &mut Loop,
    event_loop: &mut EventLoop<'static, Loop>,
    socket: OsString,
    initial: (smithay::output::Mode, Option<smithay::output::Output>),
    scale: f64,
    render_node: &str,
) -> Result<Handles, Box<dyn std::error::Error>> {
    let (mode, output) = initial;
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
    let handles = Rc::new(RefCell::new(None));
    let target = handles.clone();
    let handle = event_loop.handle();
    let (ping, source) = make_ping()?;
    let token = event_loop
        .handle()
        .insert_source(source, move |_, _, state| {
            if target.borrow().is_some() {
                return;
            }
            let Some(session) = kms::seat::session::factory::try_create() else {
                return;
            };
            let native = native::wire::entry::wire(
                state,
                socket.clone(),
                &handle,
                Some(output.clone()),
                session,
            );
            // Headless focus stays, but the human pointer/touch sources belong to KMS.
            state.inner.status_session = StatusSession::Active;
            *target.borrow_mut() = Some(native);
            state
                .state
                .schedule_redraw(protocols::redraw::schedule::RedrawReason::Resume);
        })?;
    // Only this blocking, event-driven waiter runs off-thread. No seat/device
    // or Wayland state crosses threads. Once attached, pings are ignored;
    // libseat owns subsequent pause/resume notifications.
    let _ = token;
    if std::env::var("MIX_TEST_VIUD_SEAT_INACTIVE").as_deref() == Ok("1") {
        return Ok(handles);
    }
    std::thread::Builder::new()
        .name("viud-vt-wait".into())
        .spawn(move || {
            if let Err(error) = kms::seat::session::vt_wait::watch_own_vt(|| ping.ping()) {
                warn!("VT activation source failed; desk remains offscreen: {error}");
            }
        })?;
    Ok(handles)
}
