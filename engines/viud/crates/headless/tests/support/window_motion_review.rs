// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native regressions for lifecycle policy boundaries found in review.
use super::*;
use std::io::{BufReader, Read};

fn rgb_at(bytes: &[u8], width: usize, x: usize, y: usize) -> [u8; 3] {
    let at = (y * width + x) * 3;
    bytes[at..at + 3].try_into().unwrap()
}

fn solid_window(
    bus: &Bus,
    process: &mut process::Process,
    name: &str,
    colour: u32,
    x: i32,
    y: i32,
) -> (Pattern, Value) {
    let mut client = Pattern::named(connect(process), name, false);
    client.wait_callback(0);
    let row = bus.window_for(name, &mut [&mut client]);
    bus.call("comp.window.place", place_args(&row, json!({"x":x,"y":y})));
    client.solid_buffer(colour);
    client.commit(true);
    pump_patterns(&mut [&mut client]);
    (client, row)
}

#[test]
fn completed_map_entrance_releases_visual_properties_to_window_place() {
    entrance_releases_visuals(300);
}

#[test]
fn zero_duration_map_entrance_releases_visual_properties_to_window_place() {
    entrance_releases_visuals(0);
}

fn entrance_releases_visuals(duration_ms: u64) {
    if !process::record_available() {
        return;
    }
    let label = format!("motion-review-entrance-{duration_ms}");
    let bus = Bus::start(&label);
    let mut process = process::spawn_preferences(
        &label,
        &bus.url,
        json!({
            "window_motion":{"map":{"preset":"pop","duration_ms":duration_ms}}
        }),
    );
    bus.ready();
    let (mut client, row) = solid_window(
        &bus,
        &mut process,
        "motion-review-entrance",
        0xffffffff,
        300,
        200,
    );
    thread::sleep(Duration::from_millis(400));
    pump_patterns(&mut [&mut client]);
    let (width, _, original) = ppm(&bus, &process, "completed-entrance", None);
    assert_eq!(rgb_at(&original, width, 400, 280), [255, 255, 255]);
    bus.call(
        "comp.window.place",
        place_args(
            &row,
            json!({
                "opacity":0.3,"scale":{"factor":0.5,"anchor":{"x":100,"y":80}}
            }),
        ),
    );
    let after = bus.window_for("motion-review-entrance", &mut [&mut client]);
    assert_eq!(
        after["opacity"], 0.3,
        "completed lifecycle tracks must release opacity ownership"
    );
    assert_eq!(
        after["scale"]["factor"], 0.5,
        "completed lifecycle tracks must release scale ownership"
    );
    let (width, _, placed) = ppm(&bus, &process, "placed-after-entrance", None);
    for (actual, background) in rgb_at(&placed, width, 400, 280)
        .into_iter()
        .zip([18.0, 20.0, 25.0])
    {
        let expected = 255.0 * 0.3 + background * 0.7;
        assert!(
            (f64::from(actual) - expected).abs() <= 3.0,
            "window-place opacity must affect drawn pixels after entrance: {actual} vs {expected}"
        );
    }
    assert_eq!(
        rgb_at(&placed, width, 320, 280),
        [18, 20, 25],
        "window-place scale must shrink the completed entrance's client image"
    );
}

#[test]
fn geometry_only_replacement_freezes_sampled_visuals_and_releases_their_tracks() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("motion-review-cancel");
    let mut process = process::spawn(1, None, "motion-review-cancel", Some(&bus.url));
    bus.ready();
    let (mut client, row) = solid_window(
        &bus,
        &mut process,
        "motion-review-cancel",
        0xffffffff,
        300,
        200,
    );
    bus.call(
        "comp.window.place",
        place_args(
            &row,
            json!({"scale":{"factor":1,"anchor":{"x":100,"y":80}}}),
        ),
    );
    thread::scope(|scope| {
        let first = scope.spawn(|| {
            bus.call(
                "comp.motion.run",
                json!({"targets":[{"window":{
                    "id":row["id"],"generation":row["generation"],"duration_ms":800,
                    "tracks":[
                        {"property":"scale","keyframes":[
                            {"offset":0,"value":0.4,"easing":{"kind":"linear"}},
                            {"offset":1,"value":0.8,"easing":{"kind":"linear"}}
                        ]},
                        {"property":"opacity","keyframes":[
                            {"offset":0,"value":1,"easing":{"kind":"linear"}},
                            {"offset":1,"value":0.6,"easing":{"kind":"linear"}}
                        ]}
                    ]
                }}]}),
            )
        });
        // Observe an actual intermediate sampled state before replacing it;
        // the wait starts when the Bus call has installed its tracks.
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let sampled = bus.window_for("motion-review-cancel", &mut [&mut client]);
            if sampled["scale"]["factor"].as_f64().unwrap() < 0.7 {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "first visual motion must begin before cancellation"
            );
            client.pump();
        }
        thread::sleep(Duration::from_millis(250));
        let replacement = bus.call(
            "comp.motion.run",
            json!({"targets":[{"window":{
                "id":row["id"],"generation":row["generation"],"to":{"x":300},"duration_ms":0
            }}]}),
        );
        assert_eq!(first.join().unwrap()["status"], "superseded");
        assert_eq!(replacement["status"], "completed");
    });
    let frozen = bus.window_for("motion-review-cancel", &mut [&mut client]);
    let scale = frozen["scale"]["factor"].as_f64().unwrap();
    let opacity = frozen["opacity"].as_f64().unwrap();
    assert!(
        (0.4..0.75).contains(&scale),
        "cancellation freezes its sampled scale rather than settling at 0.8: {frozen}"
    );
    assert!(
        (0.65..1.0).contains(&opacity),
        "cancellation freezes its sampled opacity rather than settling at 0.6: {frozen}"
    );
    let (width, _, pixels) = ppm(&bus, &process, "just-cancelled", None);
    for (actual, background) in rgb_at(&pixels, width, 400, 280)
        .into_iter()
        .zip([18.0, 20.0, 25.0])
    {
        let expected = 255.0 * opacity + background * (1.0 - opacity);
        assert!(
            (f64::from(actual) - expected).abs() <= 3.0,
            "frozen compositor state must match the sampled pixel opacity"
        );
    }
    thread::sleep(Duration::from_millis(600));
    let (_, _, later) = ppm(&bus, &process, "still-cancelled", None);
    assert!(
        pixels == later,
        "cancelled visual tracks must not keep progressing after geometry replacement"
    );
    let later_row = bus.window_for("motion-review-cancel", &mut [&mut client]);
    assert_eq!(later_row["scale"], frozen["scale"]);
    assert_eq!(later_row["opacity"], frozen["opacity"]);
    bus.call(
        "comp.window.place",
        place_args(
            &row,
            json!({
                "opacity":0.25,"scale":{"factor":0.9,"anchor":{"x":100,"y":80}}
            }),
        ),
    );
    let placed = bus.window_for("motion-review-cancel", &mut [&mut client]);
    assert_eq!(placed["opacity"], 0.25);
    assert_eq!(placed["scale"]["factor"], 0.9);
    let (width, _, changed) = ppm(&bus, &process, "place-after-cancel", None);
    for point in [(400, 280), (320, 280)] {
        for (actual, background) in rgb_at(&changed, width, point.0, point.1)
            .into_iter()
            .zip([18.0, 20.0, 25.0])
        {
            let expected = 255.0 * 0.25 + background * 0.75;
            assert!(
                (f64::from(actual) - expected).abs() <= 3.0,
                "window-place visuals must take ownership after cancellation at {point:?}"
            );
        }
    }
}

#[test]
fn explicit_hide_is_immediate_when_workspace_transitions_are_enabled() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("motion-review-hide");
    let mut process = process::spawn_preferences(
        "motion-review-hide",
        &bus.url,
        json!({
            "window_motion":{"workspace":{"preset":"fade","duration_ms":1000}}
        }),
    );
    bus.ready();
    let (mut client, row) = solid_window(
        &bus,
        &mut process,
        "motion-review-hide",
        0xffffffff,
        300,
        200,
    );
    let (width, _, shown) = ppm(&bus, &process, "before-explicit-hide", None);
    assert_eq!(rgb_at(&shown, width, 400, 280), [255, 255, 255]);
    bus.call(
        "comp.window.place",
        place_args(&row, json!({"visible":false})),
    );
    let hidden = bus.window_for("motion-review-hide", &mut [&mut client]);
    assert_eq!(hidden["requested_visible"], false);
    assert_eq!(hidden["visible"], false);
    let (width, _, pixels) = ppm(&bus, &process, "immediately-hidden", None);
    assert_eq!(
        rgb_at(&pixels, width, 400, 280),
        [18, 20, 25],
        "explicit hide must not create a workspace exit ghost"
    );
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":400,"y":280}),
    );
    let refused = bus.call_rc(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"click"}),
        10,
    );
    assert_eq!(refused["error"], "no_pointer_target");
}

fn exported_frames(path: &Path, mut visit: impl FnMut(usize, usize, &[u8])) {
    let manifest: Value =
        serde_json::from_slice(&fs::read(path.join("manifest.json")).unwrap()).unwrap();
    let complete: Value =
        serde_json::from_slice(&fs::read(path.join("complete.json")).unwrap()).unwrap();
    let width = manifest["width"].as_u64().unwrap() as usize;
    let frame_bytes = width * manifest["height"].as_u64().unwrap() as usize * 4;
    let exported = path.with_extension("bgra");
    let result = Command::new(binary("mixd"))
        .args(["vfl", "export"])
        .arg(path)
        .args(["--format", "bgra"])
        .stdout(fs::File::create(&exported).unwrap())
        .status()
        .unwrap();
    assert!(result.success(), "production VFL export must succeed");
    let mut reader = BufReader::new(fs::File::open(exported).unwrap());
    let mut pixels = vec![0; frame_bytes];
    for frame in 0..complete["frames"].as_u64().unwrap() as usize {
        reader.read_exact(&mut pixels).unwrap();
        visit(frame, width, &pixels);
    }
    assert_eq!(reader.read(&mut [0]).unwrap(), 0);
}

#[test]
fn placing_last_track_resolves_reply_promptly() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("placed-last-track");
    let mut process = process::spawn(1, None, "placed-last-track", Some(&bus.url));
    bus.ready();
    let (mut client, row) = solid_window(
        &bus,
        &mut process,
        "placed-last-track",
        0xffffffff,
        300,
        200,
    );
    thread::scope(|scope| {
        let running = scope.spawn(|| {
            bus.call("comp.motion.run", json!({"targets":[{"window":{
            "id":row["id"],"generation":row["generation"],"preset":"fade","duration_ms":10000
        }}]}))
        });
        let deadline = Instant::now() + Duration::from_secs(3);
        while bus.window_for("placed-last-track", &mut [&mut client])["opacity"]
            .as_f64()
            .unwrap()
            == 1.0
        {
            assert!(Instant::now() < deadline);
        }
        let placed = Instant::now();
        bus.call(
            "comp.window.place",
            place_args(&row, json!({"opacity":0.7})),
        );
        let answer = running.join().unwrap();
        assert_eq!(answer["status"], "superseded");
        assert_eq!(answer["by"], "place");
        assert!(
            placed.elapsed() < Duration::from_millis(500),
            "reply must not wait for the ten-second run"
        );
    });
}

#[test]
fn minimised_then_destroyed_never_resurrects_pixels() {
    unseen_destroy(false, true);
}

#[test]
fn workspace_switch_and_destroy_in_one_tick_never_resurrects_pixels() {
    unseen_destroy(true, true);
    unseen_destroy(true, false);
}

fn unseen_destroy(workspace: bool, already_hidden: bool) {
    if !process::record_available() {
        return;
    }
    let label = if workspace && already_hidden {
        "unseen-workspace"
    } else if workspace {
        "leaving-workspace"
    } else {
        "unseen-minimise"
    };
    let bus = Bus::start(label);
    let mut process = process::spawn_preferences(
        label,
        &bus.url,
        json!({"window_motion":{"unmap":{"preset":"fade","duration_ms":1500}}}),
    );
    bus.ready();
    let (mut client, row) = solid_window(&bus, &mut process, label, 0xffffffff, 300, 200);
    if workspace {
        bus.call(
            "comp.workspaces.set_count",
            json!({"count":2,"persist":false}),
        );
        if already_hidden {
            bus.call(
                "comp.window.send_to_workspace",
                place_args(&row, json!({"index":2,"follow":false})),
            );
        }
    } else {
        bus.call("comp.window.minimize", target(&row));
    }
    let (width, _, pixels) = ppm(&bus, &process, "unseen-before-destroy", None);
    assert_eq!(
        rgb_at(&pixels, width, 400, 280),
        if already_hidden {
            [18, 20, 25]
        } else {
            [255, 255, 255]
        }
    );
    let pause = process
        .root
        .join(format!("pause-lifecycle-{}", process.child.id()));
    let ack = process
        .root
        .join(format!("pause-lifecycle-{}.ack", process.child.id()));
    fs::write(&pause, b"pause").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !ack.exists() {
        bus.call("comp.windows.list", json!({}));
        assert!(Instant::now() < deadline);
    }
    if workspace {
        bus.call("comp.workspace.switch", json!({"index":2}));
    }
    client.commit_then_destroy();
    pump_patterns(&mut [&mut client]);
    let rows = bus.call("comp.windows.list", json!({}));
    assert!(
        rows["windows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|w| w["id"] != row["id"])
    );
    let path = process.root.join("unseen.vfl");
    let start = bus.call("comp.record.start",json!({"request_id":"unseen","target":{"output":"headless-0"},"path":path,"fps":30,"cursor":false,"max_seconds":10}));
    fs::remove_file(pause).unwrap();
    for pass in 0..2 {
        for _ in 0..50 {
            client.pump();
            thread::sleep(Duration::from_millis(5));
        }
        if workspace && pass == 0 {
            bus.call("comp.workspace.switch", json!({"index":1}));
        }
    }
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["dropped"], 0);
    let mut frames = 0;
    exported_frames(&path, |frame, width, pixels| {
        frames += 1;
        assert_eq!(
            pixel(pixels, width, 400, 280),
            [18, 20, 25],
            "unseen pixels in frame {frame}"
        );
    });
    assert!(frames >= 6);
}

#[test]
fn place_during_active_run_removes_only_supplied_tracks_and_finish_preserves_them() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("motion-review-active-place");
    let mut process = process::spawn(1, None, "motion-review-active-place", Some(&bus.url));
    bus.ready();
    let (mut client, row) = solid_window(&bus, &mut process, "active-place", 0xffffffff, 300, 200);
    thread::scope(|scope| {
        let running = scope.spawn(|| bus.call("comp.motion.run", json!({"targets":[{"window":{
            "id":row["id"],"generation":row["generation"],"duration_ms":1200,
            "tracks":[
                {"property":"scale","keyframes":[{"offset":0,"value":0.4,"easing":{"kind":"linear"}},{"offset":1,"value":0.8}]},
                {"property":"opacity","keyframes":[{"offset":0,"value":0.4,"easing":{"kind":"linear"}},{"offset":1,"value":0.6}]}
            ]
        }}]})));
        let deadline = Instant::now() + Duration::from_secs(3);
        let sampled = loop {
            let sampled = bus.window_for("active-place", &mut [&mut client]);
            if sampled["scale"]["factor"].as_f64().unwrap() < 0.7 {
                break sampled;
            }
            assert!(Instant::now() < deadline, "tracks did not start");
        };
        // Equals the stored base: it must still remove the active opacity track.
        bus.call("comp.window.place", place_args(&row, json!({"opacity":1})));
        let placed = bus.window_for("active-place", &mut [&mut client]);
        assert_eq!(placed["opacity"], 1.0);
        assert!(placed["scale"]["factor"].as_f64().unwrap() < 0.8);
        let (width, _, pixels) = ppm(&bus, &process, "placed-during-run", None);
        assert_eq!(rgb_at(&pixels, width, 320, 220), [255, 255, 255]);
        assert_eq!(running.join().unwrap()["status"], "completed");
        let final_row = bus.window_for("active-place", &mut [&mut client]);
        assert_eq!(final_row["opacity"], 1.0);
        assert_eq!(final_row["scale"]["factor"], 0.8);
        assert!(
            final_row["scale"]["factor"].as_f64().unwrap()
                > sampled["scale"]["factor"].as_f64().unwrap()
        );
        let (width, _, pixels) = ppm(&bus, &process, "placed-after-finish", None);
        assert_eq!(rgb_at(&pixels, width, 320, 220), [255, 255, 255]);
    });
}

#[test]
fn hide_then_destroy_before_lifecycle_tick_has_no_ghost_in_decoded_frames() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("motion-review-hide-destroy");
    let mut process = process::spawn_preferences(
        "motion-review-hide-destroy",
        &bus.url,
        json!({"window_motion":{"unmap":{"preset":"fade","duration_ms":1500}}}),
    );
    bus.ready();
    let (mut closing, row) = solid_window(&bus, &mut process, "hide-destroy", 0xffffffff, 300, 200);
    let (mut sentinel, _) = solid_window(
        &bus,
        &mut process,
        "hide-destroy-sentinel",
        0xffff0000,
        800,
        200,
    );
    let (width, _, pixels) = ppm(&bus, &process, "hide-destroy-shown", None);
    assert_eq!(rgb_at(&pixels, width, 400, 280), [255, 255, 255]);
    // Freeze lifecycle observation, not dispatch. Both changes happen before
    // its next tick, deterministically preserving the stale shown facts.
    fs::write(
        process
            .root
            .join(format!("pause-lifecycle-{}", process.child.id())),
        b"pause",
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !process
        .root
        .join(format!("pause-lifecycle-{}.ack", process.child.id()))
        .exists()
    {
        bus.call("comp.windows.list", json!({}));
        assert!(
            Instant::now() < deadline,
            "lifecycle barrier not acknowledged"
        );
    }
    let path = process.root.join("hide-destroy.vfl");
    let start = bus.call("comp.record.start", json!({"request_id":"hide-destroy","target":{"output":"headless-0"},"path":path,"fps":30,"cursor":false,"max_seconds":10}));
    bus.call(
        "comp.window.place",
        place_args(&row, json!({"visible":false})),
    );
    closing.commit_then_destroy();
    pump_patterns(&mut [&mut closing]);
    // A list reply witnesses destruction before the barrier is released.
    let windows = bus.call("comp.windows.list", json!({}));
    assert!(
        !windows["windows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w["id"] == row["id"])
    );
    fs::remove_file(
        process
            .root
            .join(format!("pause-lifecycle-{}", process.child.id())),
    )
    .unwrap();
    sentinel.solid_buffer(0xff20d040);
    sentinel.commit(true);
    for _ in 0..100 {
        sentinel.pump();
        thread::sleep(Duration::from_millis(5));
    }
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["dropped"], 0);
    let mut witnessed = 0;
    exported_frames(&path, |frame, width, pixels| {
        if pixel(pixels, width, 850, 280) == [0x20, 0xd0, 0x40] {
            witnessed += 1;
            assert_eq!(
                pixel(pixels, width, 400, 280),
                [18, 20, 25],
                "frame {frame}: explicit hide followed by destruction resurrected pixels"
            );
        }
    });
    assert!(witnessed >= 3, "need several frames after hide/destroy");
}

#[test]
fn close_ghost_cannot_leak_into_a_different_workspace_in_recorded_frames() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("motion-review-workspace");
    let mut process = process::spawn_preferences(
        "motion-review-workspace",
        &bus.url,
        json!({
            "window_motion":{"unmap":{"preset":"fade","duration_ms":1000}}
        }),
    );
    bus.ready();
    bus.call(
        "comp.workspaces.set_count",
        json!({"count":2,"persist":false}),
    );
    let (mut closing, _closing_row) = solid_window(
        &bus,
        &mut process,
        "workspace-one-private",
        0xffffffff,
        300,
        200,
    );
    let (mut sentinel, sentinel_row) = solid_window(
        &bus,
        &mut process,
        "workspace-two-sentinel",
        0xff20d040,
        800,
        200,
    );
    bus.call(
        "comp.window.send_to_workspace",
        place_args(&sentinel_row, json!({"index":2,"follow":false})),
    );
    pump_patterns(&mut [&mut closing, &mut sentinel]);
    let path = process.root.join("workspace-boundary.vfl");
    let start=bus.call("comp.record.start",json!({"request_id":"workspace-boundary","target":{"output":"headless-0"},"path":path,"fps":30,"cursor":false,"max_seconds":10}));
    // The client is destroyed while the exit image remains eligible for 1s.
    // Switching immediately must invalidate that old workspace's presentation.
    drop(closing);
    bus.call("comp.workspace.switch", json!({"index":2}));
    for _ in 0..100 {
        sentinel.pump();
        thread::sleep(Duration::from_millis(5));
    }
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["dropped"], 0, "{stop}");
    let mut saw_old = false;
    let mut saw_new = false;
    let mut new_frames = 0;
    exported_frames(&path, |frame, width, pixels| {
        let old = pixel(pixels, width, 400, 280);
        let sentinel = pixel(pixels, width, 850, 280);
        if !saw_new && old == [255, 255, 255] {
            saw_old = true;
        }
        // The green sentinel is an in-band witness of the new workspace's
        // rendered frame; this avoids guessing when a Bus reply was presented.
        if sentinel == [0x20, 0xd0, 0x40] {
            saw_new = true;
        }
        if saw_new {
            new_frames += 1;
            assert_eq!(
                old,
                [18, 20, 25],
                "frame {frame}: a closing workspace-one image leaked into workspace two"
            );
        }
    });
    assert!(saw_old, "recording must include the original white window");
    assert!(
        saw_new && new_frames >= 3,
        "recording must include multiple presented workspace-two frames"
    );
}

#[test]
fn never_placed_pop_uses_window_center() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("center-pop");
    let mut process = process::spawn_preferences(
        "center-pop",
        &bus.url,
        json!({"window_motion":{"map":{"preset":"pop","duration_ms":300}}}),
    );
    bus.ready();
    let mut client = Pattern::named(connect(&mut process), "center-pop", false);
    client.wait_callback(0);
    let row = bus.window_for("center-pop", &mut [&mut client]);
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let mapped = bus.window_for("center-pop", &mut [&mut client]);
        if mapped["scale"]["anchor"]["x"].as_f64() == Some(100.0) {
            assert_eq!(mapped["scale"]["anchor"]["y"].as_f64(), Some(80.0));
            break;
        }
        assert!(
            Instant::now() < deadline,
            "unplaced map preset must use its centre"
        );
    }
    bus.call("comp.motion.run", json!({"targets":[{"window":{"id":row["id"],"generation":row["generation"],"preset":"pop","anchor":"center","duration_ms":300}}]}));
    let after = bus.window_for("center-pop", &mut [&mut client]);
    assert_eq!(after["scale"]["factor"], 1.0);
    assert_eq!(after["scale"]["anchor"]["x"].as_f64(), Some(100.0));
    assert_eq!(after["scale"]["anchor"]["y"].as_f64(), Some(80.0));
}

#[test]
fn dim_exit_never_brightens_and_lock_draws_no_ghost() {
    if !process::record_available() {
        return;
    }
    for locked in [false, true] {
        let label = if locked { "locked-ghost" } else { "dim-ghost" };
        let bus = Bus::start(label);
        let mut process = process::spawn_preferences(
            label,
            &bus.url,
            json!({"window_motion":{"unmap":{"preset":"fade","duration_ms":1500}}}),
        );
        bus.ready();
        let (mut client, row) = solid_window(&bus, &mut process, label, 0xffffffff, 300, 200);
        bus.call("comp.motion.run", json!({"targets":[{"window":{"id":row["id"],"generation":row["generation"],"duration_ms":50,"tracks":[{"property":"opacity","keyframes":[{"offset":0,"value":1},{"offset":1,"value":0.05}]}]}}]}));
        let _ = ppm(&bus, &process, "dim-shown", None);
        client.commit_then_destroy();
        pump_patterns(&mut [&mut client]);
        let mut locker = Pattern::named(connect(&mut process), "locker", false);
        let _lock = if locked {
            Some(locker.lock_session())
        } else {
            None
        };
        let path = process.root.join("exit.vfl");
        let start = bus.call("comp.record.start", json!({"request_id":label,"target":{"output":"headless-0"},"path":path,"fps":30,"cursor":false,"max_seconds":10}));
        thread::sleep(Duration::from_millis(300));
        bus.call("comp.record.stop", json!({"session":start["session"]}));
        let mut frames = 0;
        exported_frames(&path, |_, width, pixels| {
            frames += 1;
            let p = pixel(pixels, width, 400, 280);
            if locked {
                assert_eq!(p, [0, 0, 0]);
            } else {
                for (value, background) in p.into_iter().zip([18.0, 20.0, 25.0]) {
                    assert!(f64::from(value) <= background + (255.0 - background) * 0.05 + 2.0);
                }
            }
        });
        assert!(frames >= 3);
    }
}

#[test]
fn exiting_window_never_reuses_old_output_or_pane_pixels() {
    if !process::record_available() {
        return;
    }
    for panes in [false, true] {
        for minimise in [false, true] {
            let label = format!("scope-exit-{panes}-{minimise}");
            let bus = Bus::start(&label);
            let mut prefs = json!({"window_motion":{"unmap":{"preset":"fade","duration_ms":1500},"minimise":{"preset":"fade","duration_ms":1500}}});
            prefs[if panes {
                "_test_two_panes"
            } else {
                "_test_two_outputs"
            }] = json!(true);
            let mut process = process::spawn_preferences(&label, &bus.url, prefs);
            bus.ready();
            let (mut client, row) = solid_window(
                &bus,
                &mut process,
                &label,
                0xffffffff,
                if panes { 300 } else { 1600 },
                200,
            );
            let source_output = if panes { "headless-0" } else { "headless-1" };
            let path = process.root.join("source.vfl");
            let start = bus.call("comp.record.start",json!({"request_id":"source","target":{"output":source_output},"path":path,"fps":30,"cursor":false,"max_seconds":10}));
            thread::sleep(Duration::from_millis(150));
            if panes {
                bus.call("comp.window.place", place_args(&row, json!({"x":900})));
            } else {
                bus.call("comp.window.place", place_args(&row, json!({"x":300})));
            }
            thread::sleep(Duration::from_millis(150));
            let (width, _, destination) = ppm(&bus, &process, "destination-shown", None);
            assert_eq!(
                rgb_at(&destination, width, if panes { 1000 } else { 400 }, 280),
                [255, 255, 255]
            );
            if minimise {
                bus.call("comp.window.minimize", target(&row));
            } else {
                client.commit_then_destroy();
                pump_patterns(&mut [&mut client]);
            }
            thread::sleep(Duration::from_millis(300));
            bus.call("comp.record.stop", json!({"session":start["session"]}));
            let mut shown = false;
            let mut cleared = false;
            let mut after = 0;
            exported_frames(&path, |frame, width, pixels| {
                let p = pixel(pixels, width, 400, 280);
                if !cleared && p == [255, 255, 255] {
                    shown = true;
                }
                if shown && p == [18, 20, 25] {
                    cleared = true;
                }
                if cleared {
                    after += 1;
                    assert_eq!(p, [18, 20, 25], "stale source scope in frame {frame}");
                }
            });
            assert!(
                shown && cleared && after >= 5,
                "must witness source pixels, move and exit: panes={panes} minimise={minimise} shown={shown} cleared={cleared} after={after}"
            );
        }
    }
}
