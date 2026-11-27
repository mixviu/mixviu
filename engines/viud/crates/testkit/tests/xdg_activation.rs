// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Real xdg wire requests, production host policy and production lifecycle focus.
use dispatcher::wire::trait_::wire_trait::WireTrait;
use protocols::window::ident;
use smithay::reexports::calloop::EventLoop;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use testkit::Harness;
use world::state::{Loader, Loop, Orchestrator};

fn policy_loop(h: &Harness) -> Loop {
    let event_loop: EventLoop<'static, Loop> = EventLoop::try_new().unwrap();
    let kernel = Default::default();
    let worlds = world::world::manager::WorldManager::new(
        world::kind::build::base::spatial(
            world::world::manager::MAIN_WORLD,
            "main",
            vec![],
            &kernel,
        ),
        world::kind::build::base::overlay(world::world::manager::KERNEL, "kernel", vec![], &kernel),
        &kernel,
    );
    let dh = h.wire.state.output.display_handle.clone();
    let inner = Orchestrator::new(
        world::environment::type_::base::Environment {
            gpu: String::new(),
            desktop_name: "testkit".into(),
        },
        false,
        Loader {
            socket_name: "testkit-policy".into(),
            display_handle: dh.clone(),
            loop_signal: event_loop.get_signal(),
        },
        kernel,
        worlds,
    );
    let mut lp = Loop::new(inner, &dh, None, event_loop.handle());
    lp.inner
        .host_space_mut()
        .state
        .map_output(&h.wire.inner.output, (0, 0));
    for window in h.wire.inner.space.state.elements() {
        lp.inner.host_space_mut().state.map_element(
            window.clone(),
            h.wire.inner.space.state.element_location(window).unwrap(),
            false,
        );
    }
    lp.inner.host_space_mut().state.refresh();
    lp
}

fn service(h: &mut Harness, lp: &mut Loop) {
    std::mem::swap(&mut h.wire.inner.comp, &mut lp.inner.comp);
    policy_host::control::apply_client_requests(lp);
    frames::hook::window::interface::drain_lifecycle(lp);
    std::mem::swap(&mut h.wire.inner.comp, &mut lp.inner.comp);
}

fn focused(lp: &Loop) -> Option<WlSurface> {
    lp.state.seat.seat.get_keyboard().unwrap().current_focus()
}

#[test]
fn focused_creator_can_raise_another_client_but_background_unknown_and_reuse_cannot() {
    let mut h = Harness::new();
    let (a, _, _) = h.mapped_toplevel(64, 48);
    let b_client = h.add_client();
    let b = h.extra_clients[b_client].create_surface();
    let (_xdg, _top) = h.extra_clients[b_client].toplevel(&b);
    h.extra_clients[b_client].commit(&b);
    h.roundtrip_client(b_client);
    h.extra_clients[b_client].attach(&b, 64, 48);
    h.roundtrip_client(b_client);
    h.tick_frame();
    let server_a = h
        .wire
        .inner
        .space
        .state
        .elements()
        .find_map(|w| {
            let s = ident::surface(w)?;
            (h.handle_of(&a) == dispatcher::wire::trait_::surface_event::SurfaceHandle::wl(&s))
                .then_some(s)
        })
        .unwrap();
    let server_b = h
        .wire
        .inner
        .space
        .state
        .elements()
        .find_map(|w| {
            let s = ident::surface(w)?;
            (h.handle_of_client(b_client, &b)
                == dispatcher::wire::trait_::surface_event::SurfaceHandle::wl(&s))
            .then_some(s)
        })
        .unwrap();
    let mut lp = policy_loop(&h);
    h.focus_keyboard(&a);
    let serial = h
        .wire
        .state
        .seat
        .seat
        .get_keyboard()
        .unwrap()
        .last_enter()
        .unwrap();
    h.client.activation_token(Some(serial.into()), Some(&a));
    h.roundtrip();
    let token = h.client.state.activation_tokens.last().unwrap().clone();
    h.extra_clients[b_client]
        .activation()
        .activate(token.clone(), &b);
    h.roundtrip_client(b_client);
    service(&mut h, &mut lp);
    assert_eq!(focused(&lp), Some(server_b.clone()));
    assert_eq!(
        lp.inner
            .host_space()
            .state
            .elements()
            .last()
            .and_then(ident::surface),
        Some(server_b.clone()),
        "B raised"
    );
    assert!(
        protocols::xdg::activation::request::activations(&server_b)
            .iter()
            .any(|e| e.token.to_string() == token)
    );

    // Give A focus through the real keyboard before each negative request.
    let keyboard = lp.state.seat.seat.get_keyboard().unwrap();
    keyboard.set_focus(
        &mut lp.state,
        Some(server_a.clone()),
        smithay::utils::SERIAL_COUNTER.next_serial(),
    );
    h.extra_clients[b_client]
        .activation()
        .activate(token.clone(), &b);
    h.roundtrip_client(b_client);
    service(&mut h, &mut lp);
    assert_eq!(
        focused(&lp),
        Some(server_a.clone()),
        "spent token cannot steal focus"
    );
    assert_eq!(
        protocols::xdg::activation::request::activations(&server_b).len(),
        1,
        "reused token remains logged, deduplicated"
    );

    h.extra_clients[b_client].activation_token(Some(serial.into()), Some(&b));
    h.roundtrip_client(b_client);
    let background = h.extra_clients[b_client]
        .state
        .activation_tokens
        .last()
        .unwrap()
        .clone();
    h.extra_clients[b_client]
        .activation()
        .activate(background.clone(), &b);
    h.roundtrip_client(b_client);
    service(&mut h, &mut lp);
    assert_eq!(
        focused(&lp),
        Some(server_a.clone()),
        "background creator cannot self-activate"
    );
    assert!(
        protocols::xdg::activation::request::activations(&server_b)
            .iter()
            .any(|e| e.token.to_string() == background)
    );
    let before = protocols::xdg::activation::request::activations(&server_b).len();
    h.extra_clients[b_client]
        .activation()
        .activate("unknown-token".into(), &b);
    h.roundtrip_client(b_client);
    service(&mut h, &mut lp);
    assert_eq!(focused(&lp), Some(server_a));
    assert_eq!(
        protocols::xdg::activation::request::activations(&server_b).len(),
        before,
        "unknown token is dropped by Smithay (addendum 1)"
    );
}
