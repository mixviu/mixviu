// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exit-only ownership, immutable hidden captures and buffer replacement.
use super::*;

fn rgb_at(bytes: &[u8], width: usize, x: usize, y: usize) -> [u8; 3] {
    bytes[(y * width + x) * 3..(y * width + x) * 3 + 3]
        .try_into()
        .unwrap()
}

fn window(
    bus: &Bus,
    process: &mut process::Process,
    name: &str,
    x: usize,
    y: usize,
) -> (Pattern, Value) {
    let mut client = Pattern::named(connect(process), name, false);
    client.wait_callback(0);
    let row = bus.window_for(name, &mut [&mut client]);
    bus.call("comp.window.place", place_args(&row, json!({"x":x,"y":y})));
    client.solid_buffer(0xffffffff);
    let previous = client.state.callbacks;
    client.commit(true);
    client.wait_callback(previous);
    (client, row)
}

fn metrics(process: &process::Process) -> (usize, u64) {
    let log = fs::read_to_string(process.root.join("viud.log")).unwrap();
    let line = log
        .lines()
        .rev()
        .find(|line| line.contains("window_exit_buffers buffers="))
        .expect("exit ownership metrics");
    let field = |name: &str| {
        line.split_whitespace()
            .find_map(|word| word.strip_prefix(name))
            .unwrap()
            .parse::<u64>()
            .unwrap()
    };
    (field("buffers=") as usize, field("pixel_bytes="))
}

fn rss(process: &process::Process) -> u64 {
    fs::read_to_string(format!("/proc/{}/status", process.child.id()))
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn hidden_capture_cannot_mutate_an_outgoing_image() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("exit-hidden-copy");
    let mut process = process::spawn_preferences(
        "exit-hidden-copy",
        &bus.url,
        json!({"window_motion":{"minimise":{"preset":"fade","duration_ms":3000}}}),
    );
    bus.ready();
    let (mut client, row) = window(&bus, &mut process, "exit-hidden-copy", 300, 200);
    bus.call("comp.window.minimize", target(&row));
    let (width, _, before) = ppm(&bus, &process, "exit-before-mutation", None);
    assert!(rgb_at(&before, width, 400, 280)[1] > 150);
    client.solid_buffer(0xffff0000);
    client.commit(true);
    pump_patterns(&mut [&mut client]);
    let (width, height, hidden) = ppm(&bus, &process, "hidden-new-buffer", Some(target(&row)));
    assert_eq!(
        rgb_at(&hidden, width, width / 2, height / 2),
        [255, 0, 0],
        "hidden capture actually imports the new buffer"
    );
    let (width, _, outgoing) = ppm(&bus, &process, "outgoing-after-hidden-import", None);
    let pixel = rgb_at(&outgoing, width, 400, 280);
    assert!(
        pixel[1] > 100 && pixel[0].abs_diff(pixel[1]) <= 4,
        "outgoing image must remain white after hidden red capture: {pixel:?}"
    );
}

#[test]
fn commit_b_then_destroy_captures_the_current_buffer_before_renderer_reset() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("exit-commit-destroy");
    let mut process = process::spawn_preferences(
        "exit-commit-destroy",
        &bus.url,
        json!({"window_motion":{"unmap":{"preset":"fade","duration_ms":1500}}}),
    );
    bus.ready();
    let (mut client, _row) = window(&bus, &mut process, "exit-commit-destroy", 300, 200);
    client.solid_buffer(0xffff0000);
    client.commit_then_destroy();
    pump_patterns(&mut [&mut client]);
    let (width, _, pixels) = ppm(&bus, &process, "destroyed-buffer-b", None);
    let pixel = rgb_at(&pixels, width, 400, 280);
    assert!(
        pixel[0] > 150 && pixel[1] < 50 && pixel[2] < 55,
        "exit must freeze committed red B, not previous white A: {pixel:?}"
    );
    assert_eq!(metrics(&process).0, 1);
}

#[test]
fn eight_live_windows_release_buffers_and_exit_ownership_returns_to_zero() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("exit-eight-memory");
    let mut process = process::spawn_preferences(
        "exit-eight-memory",
        &bus.url,
        json!({"window_motion":{"minimise":{"preset":"fade","duration_ms":3000}}}),
    );
    bus.ready();
    let before = rss(&process);
    let mut clients = Vec::new();
    let mut rows = Vec::new();
    for index in 0..8 {
        let (client, row) = window(
            &bus,
            &mut process,
            &format!("exit-eight-{index}"),
            40 + (index % 4) * 300,
            80 + (index / 4) * 300,
        );
        clients.push(client);
        rows.push(row);
    }
    for client in &mut clients {
        let releases = client.state.buffer_releases;
        for _ in 0..3 {
            client.solid_buffer(0xffffffff);
            let callback = client.state.callbacks;
            client.commit(true);
            client.wait_callback(callback);
        }
        assert!(
            client.state.buffer_releases >= releases + 2,
            "live snapshot metadata must not pin replaced wl_buffers"
        );
    }
    ppm(&bus, &process, "eight-shown", None);
    assert_eq!(metrics(&process), (0, 0));
    let shown = rss(&process);
    for row in &rows {
        bus.call("comp.window.minimize", target(row));
    }
    ppm(&bus, &process, "eight-exiting", None);
    let held = metrics(&process);
    assert_eq!(held, (8, 8 * 200 * 160 * 4));
    let exiting = rss(&process);
    thread::sleep(Duration::from_millis(3300));
    ppm(&bus, &process, "eight-ended", None);
    assert_eq!(metrics(&process), (0, 0));
    println!(
        "EXIT_MEMORY windows=8 live_leases=0 exit_leases={} exit_pixel_bytes={} ended_leases=0 rss_before_kib={before} rss_shown_kib={shown} rss_exit_kib={exiting} rss_ended_kib={}",
        held.0,
        held.1,
        rss(&process)
    );
}
