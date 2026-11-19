// SPDX-License-Identifier: MIT OR Apache-2.0
//! OSD acceptance through the real Bus, scene host, compositor and VFL reader.
use super::*;

fn call(bus: &Bus, command: &str, args: Value) -> Value {
    call_rc(bus, command, args, 0)
}

fn call_rc(bus: &Bus, command: &str, args: Value, expected: i32) -> Value {
    let output = helper(
        "call",
        &bus.root,
        &[command, &args.to_string(), "osd"],
        Some(&bus.url),
    );
    assert!(output.status.success(), "OSD helper: {output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let reply: Value = serde_json::from_str(stdout.lines().last().expect("OSD reply")).unwrap();
    assert_eq!(reply["rc"], expected, "{command}: {reply}");
    match &reply["body"] {
        Value::String(body) => serde_json::from_str(body).unwrap(),
        body => body.clone(),
    }
}

fn ready(bus: &Bus) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let output = helper(
            "call",
            &bus.root,
            &["osd.list", "{}", "osd"],
            Some(&bus.url),
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().last()
            && let Ok(reply) = serde_json::from_str::<Value>(line)
            && reply["rc"] == 0
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "OSD host did not register: {output:?}"
        );
        thread::sleep(Duration::from_millis(50));
    }
    assert!(
        call(bus, "osd.list", json!({}))["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

fn wait(pattern: &mut Pattern, duration: Duration) {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        pattern.pump();
        thread::sleep(Duration::from_millis(5));
    }
}

fn caption() -> Value {
    json!({"id":"caption","scene":{"kind":"caption","text":"An OSD caption above client windows","width":320,"height":72},"anchor":{"output":"headless-0","edge":"bottom-right","margin":64},"enter":"up"})
}

fn rgb(pixels: &[u8], width: usize, x: usize, y: usize) -> &[u8] {
    &pixels[(y * width + x) * 3..(y * width + x + 1) * 3]
}

/// Bounding box of changes within an output-local region, excluding the cursor.
fn bounds(before: &[u8], after: &[u8], width: usize, region: [usize; 4]) -> Option<[usize; 4]> {
    let [x, y, w, h] = region;
    let mut result = [x + w, y + h, 0, 0];
    for py in y..y + h {
        for px in x..x + w {
            if rgb(before, width, px, py)
                .iter()
                .zip(rgb(after, width, px, py))
                .any(|(a, b)| a.abs_diff(*b) > 4)
            {
                result[0] = result[0].min(px);
                result[1] = result[1].min(py);
                result[2] = result[2].max(px + 1);
                result[3] = result[3].max(py + 1);
            }
        }
    }
    (result[2] > result[0] && result[3] > result[1]).then_some(result)
}

fn decoded_rgb(frame: &[u8]) -> Vec<u8> {
    frame
        .chunks_exact(4)
        .flat_map(|p| [p[2], p[1], p[0]])
        .collect()
}

#[test]
fn caption_records_motion_clicks_through_and_fades_to_empty() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("osd-caption");
    let mut process = process::spawn_osd("osd-caption", &bus.url);
    let mut pattern = Pattern::new(connect(&mut process));
    pattern.solid_buffer(0xff20d040);
    pattern.commit(true);
    pattern.wait_callback(0);
    bus.ready();
    ready(&bus);
    let row = bus.window(&mut pattern);
    bus.call(
        "comp.window.place",
        place_args(&row, json!({"z":2147483647})),
    );
    wait(&mut pattern, Duration::from_millis(150));
    let (width, height, before) = ppm(&bus, &process, "osd-before", None);
    let path = process.root.join("osd-caption.vfl");
    let record = bus.call("comp.record.start",json!({"request_id":"osd-caption","target":{"output":"headless-0"},"path":path,"fps":30,"cursor":false,"max_seconds":10}));
    call(&bus, "osd.show", caption());
    wait(&mut pattern, Duration::from_millis(1050));
    let (_, _, shown) = ppm(&bus, &process, "osd-shown", None);
    let region = [width - 400, height - 200, 400, 200];
    let shown_bounds = bounds(&before, &shown, width, region)
        .expect("caption must draw above a maximum-z fullscreen client");
    let items = call(&bus, "osd.list", json!({}));
    assert_eq!(items["items"][0]["pointer_input"], false);
    let buttons = pattern.state.pointer_buttons;
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":width-200,"y":height-100}),
    );
    bus.call(
        "comp.input.pointer.button",
        json!({"button":"left","action":"click"}),
    );
    wait(&mut pattern, Duration::from_millis(80));
    assert!(
        pattern.state.pointer_buttons > buttons,
        "passive OSD must pass pointer buttons to the client"
    );
    call(
        &bus,
        "osd.update",
        json!({"id":"caption","pointer_input":true}),
    );
    wait(&mut pattern, Duration::from_millis(100));
    assert_eq!(
        call(&bus, "osd.list", json!({}))["items"][0]["pointer_input"],
        true
    );
    let buttons = pattern.state.pointer_buttons;
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":width-200,"y":height-100}),
    );
    bus.call(
        "comp.input.pointer.button",
        json!({"button":"left","action":"click"}),
    );
    wait(&mut pattern, Duration::from_millis(80));
    assert_eq!(
        pattern.state.pointer_buttons, buttons,
        "opted-in OSD must receive its own pointer input"
    );
    let keys = pattern.state.keys;
    bus.call("comp.input.key", json!({"key":"a"}));
    wait(&mut pattern, Duration::from_millis(50));
    assert!(
        pattern.state.keys > keys,
        "OSD pointer input must preserve the client's keyboard focus"
    );
    call(&bus, "osd.hide", json!({"id":"caption","exit":"fade"}));
    wait(&mut pattern, Duration::from_millis(850));
    assert!(
        call(&bus, "osd.list", json!({}))["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let (_, _, after) = ppm(&bus, &process, "osd-hidden", None);
    assert_eq!(after, before, "fade must leave no OSD pixels");
    let stopped = bus.call("comp.record.stop", json!({"session":record["session"]}));
    assert_eq!(stopped["dropped"], 0);

    let mut reader = media_vfl::reader::Reader::open(&path).unwrap();
    let mut observed = Vec::new();
    let mut last = None;
    while let Some(frame) = reader.read().unwrap() {
        let pixels = decoded_rgb(frame.pixels);
        if let Some(bbox) = bounds(&before, &pixels, width, region)
            && last != Some(bbox[1])
        {
            observed.push((frame.t_us as f64 / 1_000_000.0, bbox[1] as f64));
            last = Some(bbox[1]);
        }
    }
    let preset = motion::presets()
        .into_iter()
        .find(|p| p.name == "up")
        .unwrap();
    let settled = shown_bounds[1] as f64;
    let moving = observed
        .iter()
        .take_while(|(_, y)| *y > settled + 1.0)
        .copied()
        .collect::<Vec<_>>();
    assert!(
        moving.len() >= 3,
        "VFL must contain intermediate entrance values: {observed:?}"
    );
    // The request is asynchronous relative to the recorder clock. Fit only
    // that one start offset; every decoded position must agree with the
    // canonical pure library sample within raster and one-frame tolerance.
    let best = (0..1500)
        .map(|ms| {
            let start = ms as f64 / 1000.0;
            moving
                .iter()
                .map(|(t, y)| {
                    let expected = settled
                        + preset
                            .sample((t - start).max(0.0))
                            .get(motion::Property::Ty);
                    (y - expected).abs()
                })
                .fold(0.0_f64, f64::max)
        })
        .fold(f64::INFINITY, f64::min);
    assert!(
        best <= 5.0,
        "decoded entrance disagrees with motion library: max error {best}, samples={moving:?}"
    );
    let mut reader = media_vfl::reader::Reader::open(&path).unwrap();
    let mut final_pixels = Vec::new();
    while let Some(frame) = reader.read().unwrap() {
        final_pixels = decoded_rgb(frame.pixels);
    }
    assert_eq!(
        final_pixels, before,
        "the final VFL frame must contain no OSD pixels"
    );

    let mut show = caption();
    show.as_object_mut().unwrap().remove("enter");
    call(&bus, "osd.show", show);
    wait(&mut pattern, Duration::from_millis(120));
    let listed_path = process.root.join("osd-listed.vfl");
    let listed=bus.call("comp.record.start",json!({"request_id":"osd-listed","target":{"windows":[target(&row)]},"path":listed_path,"cursor":false,"max_seconds":3}));
    wait(&mut pattern, Duration::from_millis(150));
    bus.call("comp.record.stop", json!({"session":listed["session"]}));
    let mut reader = media_vfl::reader::Reader::open(&listed_path).unwrap();
    let mut frames = 0;
    while let Some(frame) = reader.read().unwrap() {
        assert_eq!(
            decoded_rgb(frame.pixels),
            before,
            "listed-window recording must exclude OSD"
        );
        frames += 1;
    }
    assert!(frames > 0);
    call(&bus, "osd.hide", json!({"id":"caption"}));
    wait(&mut pattern, Duration::from_millis(100));

    let idle=bus.call("comp.record.start",json!({"request_id":"osd-idle","target":{"output":"headless-0"},"path":process.root.join("osd-idle.vfl"),"cursor":false,"max_seconds":3}));
    let a = bus.call("comp.record.status", json!({"session":idle["session"]}));
    wait(&mut pattern, Duration::from_millis(400));
    let b = bus.call("comp.record.stop", json!({"session":idle["session"]}));
    assert_eq!(
        a["readbacks"], b["readbacks"],
        "empty OSD must leave idle readbacks unchanged"
    );
}

#[test]
fn window_card_follows_hides_and_osd_refusals_are_typed() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("osd-window");
    let mut process = process::spawn_osd("osd-window", &bus.url);
    let mut pattern = Pattern::named(connect(&mut process), "osd-anchor", false);
    pattern.wait_callback(0);
    bus.ready();
    ready(&bus);
    let row = bus.window_for("osd-anchor", &mut [&mut pattern]);
    bus.call(
        "comp.window.place",
        place_args(&row, json!({"output":"headless-0","x":100,"y":100})),
    );
    wait(&mut pattern, Duration::from_millis(150));
    let (w, _, before) = ppm(&bus, &process, "osd-card-before", None);
    call(
        &bus,
        "osd.show",
        json!({"id":"card","scene":{"kind":"card","title":"Window","text":"Follows its window","width":180,"height":80},"anchor":{"window":target(&row),"offset":{"x":230,"y":0}}}),
    );
    wait(&mut pattern, Duration::from_millis(150));
    let (_, _, shown) = ppm(&bus, &process, "osd-card-shown", None);
    let first = bounds(&before, &shown, w, [320, 80, 220, 180]).expect("window card pixels");
    bus.call(
        "comp.window.place",
        place_args(&row, json!({"x":500,"y":200})),
    );
    wait(&mut pattern, Duration::from_millis(200));
    let (_, _, moved) = ppm(&bus, &process, "osd-card-moved", None);
    let second =
        bounds(&before, &moved, w, [720, 180, 220, 180]).expect("moved window card pixels");
    assert!((second[0] as i64 - first[0] as i64 - 400).abs() <= 2);
    assert!((second[1] as i64 - first[1] as i64 - 100).abs() <= 2);
    bus.call("comp.window.minimize", target(&row));
    wait(&mut pattern, Duration::from_millis(200));
    let (_, _, hidden) = ppm(&bus, &process, "osd-card-minimized", None);
    assert!(
        bounds(&before, &hidden, w, [720, 180, 220, 180]).is_none(),
        "minimized anchor must suppress OSD pixels"
    );
    call(&bus, "osd.hide", json!({"id":"card"}));
    for (command, args) in [
        (
            "osd.show",
            json!({"id":"bad","scene":{"kind":"caption","text":"bad"},"anchor":{"output":"headless-0","edge":"bottom"},"enter":"unknown"}),
        ),
        ("osd.update", json!({"id":"missing","pointer_input":true})),
        ("osd.hide", json!({"id":"missing"})),
        (
            "osd.show",
            json!({"id":"bad","scene":{"kind":"caption","text":"bad"},"anchor":{"window":{"id":row["id"],"generation":0}}}),
        ),
    ] {
        let refused = call_rc(&bus, command, args, 10);
        assert!(
            refused["error_code"]
                .as_str()
                .is_some_and(|s| !s.is_empty()),
            "typed refusal: {refused}"
        );
    }
    let actions = call(&bus, "osd.actions", json!({}));
    assert!(
        actions.is_object() || actions.is_array(),
        "discoverable OSD actions"
    );
    call(
        &bus,
        "osd.execute",
        json!({"id":"osd.show","params":caption()}),
    );
    call(
        &bus,
        "osd.execute",
        json!({"id":"osd.hide","params":{"id":"caption"}}),
    );
    let refused = bus.call_rc(
        "comp.window.place",
        place_args(&row, json!({"band":"osd"})),
        10,
    );
    assert!(
        refused["error_code"].is_string(),
        "apps cannot request privileged bands: {refused}"
    );

    let mut imposter = Command::new(binary("testkit-layer-client"))
        .args([
            "--namespace",
            "viu-osd",
            "--layer",
            "overlay",
            "--keyboard",
            "exclusive",
        ])
        .env("XDG_RUNTIME_DIR", &process.root)
        .env("WAYLAND_DISPLAY", "wayland-headless-test")
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if let Some(status) = imposter.try_wait().unwrap() {
            assert!(status.success(), "closed imposter: {status}");
            break;
        }
        if Instant::now() >= deadline {
            let _ = imposter.kill();
            let _ = imposter.wait();
            panic!("client-spoofed OSD namespace was not closed");
        }
        wait(&mut pattern, Duration::from_millis(10));
    }
}
