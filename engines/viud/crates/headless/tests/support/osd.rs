// SPDX-License-Identifier: MIT OR Apache-2.0
//! OSD acceptance through the real Bus, scene host, compositor and VFL reader.
use super::*;

fn osd_helper(action: &str, root: &Path, rest: &[&str], _url: Option<&str>) -> Output {
    assert_eq!(action, "call");
    assert_eq!(rest.get(2), Some(&"osd"));
    // Named supervised citizens send on their registered connection. Shell
    // one-shot sends use an anonymous lane, even after noded_register.
    let script = root.join("osd-call.mix");
    fs::write(&script, format!(
        "send \"osd\" {} timeout=10 body={}\nprint(json_encode({{rc:$rc,body:$result}}))\nexit(0)\n",
        serde_json::to_string(rest[0]).unwrap(), serde_json::to_string(rest[1]).unwrap()
    )).unwrap();
    Command::new(binary("mixd"))
        .arg("--serve")
        .arg(script)
        .args(["--name", rest.get(3).copied().unwrap_or("headless-osd-owner")])
        .env("MIX_NODED_URL", format!("unix://{}", root.join("run/noded/bus.sock").display()))
        .env("MIX_RUN", root.join("run"))
        .env("MIX_ETC", root.join("etc"))
        .env("MIX_VAR", root.join("var"))
        .env("MIX_NODE_CONFIG", root.join("etc/node.conf.mix"))
        .output()
        .expect("named OSD caller")
}

pub(super) fn call(bus: &Bus, command: &str, args: Value) -> Value {
    call_rc(bus, command, args, 0)
}

fn call_rc(bus: &Bus, command: &str, args: Value, expected: i32) -> Value {
    let output = osd_helper(
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

pub(super) fn ready(bus: &Bus) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let output = osd_helper(
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
    // The Bus owner starts before Gpu::new. An answer alone does not mean
    // the renderer can present an entrance: wait for its empty Wayland layer.
    // This private fixture has no other layer clients at startup/restart.
    loop {
        let surfaces = bus.call("comp.props.get", json!({"path":"surfaces"}));
        if surfaces.as_object().is_some_and(|rows| {
            rows.values().any(|row| {
                row["role"] == "layer"
                    && row["output"].is_string()
                    && row["layer"]["stratum"] == "overlay"
            })
        }) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "OSD renderer did not create its layer: {surfaces}"
        );
        thread::sleep(Duration::from_millis(20));
    }
    // GPU/font first-use setup is not an entrance sample. Warm it outside
    // the measured scene, then positively observe detachment before returning.
    call(
        bus,
        "osd.show",
        json!({"id":"fixture-ready","scene":{"kind":"caption","text":"Ready","width":320,"height":72},"anchor":{"output":"headless-0","edge":"bottom-right","margin":64}}),
    );
    for mapped in [true, false] {
        if !mapped {
            call(bus, "osd.hide", json!({"id":"fixture-ready","exit":null}));
        }
        loop {
            let surfaces = bus.call("comp.props.get", json!({"path":"surfaces"}));
            let seen = surfaces.as_object().is_some_and(|rows| {
                rows.values().any(|row| {
                    row["role"] == "layer"
                        && row["layer"]["stratum"] == "overlay"
                        && row["mapped"] == true
                })
            });
            if seen == mapped {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "OSD first-use map/detach did not complete: {surfaces}"
            );
            thread::sleep(Duration::from_millis(20));
        }
    }
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
fn caption_records_motion_consumes_clicks_and_fades_to_empty() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("osd-caption");
    // Pixel/motion acceptance needs only the caption's 400x200 region. Keep
    // debug readback/encoding bounded when the gate runs GPU tests together;
    // strict 4K30 throughput has its own optimised acceptance benchmark.
    let mut process = process::spawn_osd("osd-caption", &bus.url, "640x360");
    let mut pattern = Pattern::new(connect(&mut process));
    pattern.state.size = (640, 360);
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
    // The 700 ms entrance still provides at least three moving samples at
    // 10 fps. Unoptimised copies share CPU with the gate's other test binaries;
    // the dedicated optimised native run keeps the original 30 fps workload.
    let fps = if cfg!(debug_assertions) { 10 } else { 30 };
    let record = bus.call("comp.record.start",json!({"request_id":"osd-caption","target":{"output":"headless-0"},"path":path,"fps":fps,"cursor":false,"max_seconds":10}));
    // Keep the pointer stationary as the opaque card appears beneath it.
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":width-200,"y":height-100}),
    );
    call(&bus, "osd.show", caption());
    wait(&mut pattern, Duration::from_millis(1050));
    let (_, _, shown) = ppm(&bus, &process, "osd-shown", None);
    let badge = rgb(&shown, width, 12, 10);
    assert_ne!(
        badge,
        rgb(&before, width, 12, 10),
        "the compositor must draw the provenance badge"
    );
    for (channel, expected) in badge.iter().zip([242_u8, 166, 38]) {
        assert!(
            channel.abs_diff(expected) <= 8,
            "expected the compositor's amber badge, got {badge:?}"
        );
    }
    let region = [width - 400, height - 200, 400, 200];
    let shown_bounds = bounds(&before, &shown, width, region)
        .expect("caption must draw above a maximum-z fullscreen client");
    let items = call(&bus, "osd.list", json!({}));
    assert_eq!(items["items"][0]["owner"], "local:headless-osd-owner");
    for command in ["osd.update", "osd.hide"] {
        let output = osd_helper(
            "call",
            &bus.root,
            &[command, "{\"id\":\"caption\"}", "osd", "other-osd-citizen"],
            Some(&bus.url),
        );
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let reply: Value = serde_json::from_str(stdout.lines().last().unwrap()).unwrap();
        assert_eq!(reply["rc"], 10, "{reply}");
        let body = match &reply["body"] {
            Value::String(body) => serde_json::from_str(body).unwrap(),
            body => body.clone(),
        };
        assert_eq!(body["error_code"], "NOT_OWNER");
    }
    // A peer must neither forge telemetry nor poison subsequent real presses.
    for args in [
        json!({"presses":"poison"}),
        json!({"presses":999}),
        json!({}),
    ] {
        let output = osd_helper(
            "call",
            &bus.root,
            &["osd.pointer", &args.to_string(), "osd"],
            Some(&bus.url),
        );
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let reply: Value = serde_json::from_str(stdout.lines().last().unwrap()).unwrap();
        assert_eq!(
            reply["rc"], 10,
            "peer pointer command must be refused: {reply}"
        );
        let after = call(&bus, "osd.list", json!({}));
        assert_eq!(after["pointer_presses"], items["pointer_presses"]);
        assert_eq!(after["items"][0]["scene"], items["items"][0]["scene"]);
    }
    assert_eq!(items["items"][0]["pointer_input"], true);
    let buttons = pattern.state.pointer_buttons;
    let received = call(&bus, "osd.list", json!({}))["pointer_presses"]
        .as_u64()
        .unwrap();
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"click"}),
    );
    wait(&mut pattern, Duration::from_millis(80));
    assert!(
        pattern.state.pointer_buttons == buttons,
        "default opaque OSD must consume pointer buttons"
    );
    assert!(
        call(&bus, "osd.list", json!({}))["pointer_presses"]
            .as_u64()
            .unwrap()
            > received,
        "opaque OSD must observe the button"
    );
    // Headless has an agent seat and no human device sources. Establish its
    // independent keyboard focus before testing untargeted keys after an OSD click.
    bus.call(
        "comp.input.key",
        json!({"seat":"agent","window":target(&row),"key":"a"}),
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
    let received = call(&bus, "osd.list", json!({}))["pointer_presses"]
        .as_u64()
        .unwrap();
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":width-200,"y":height-100}),
    );
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"click"}),
    );
    wait(&mut pattern, Duration::from_millis(80));
    assert_eq!(
        pattern.state.pointer_buttons, buttons,
        "opted-in OSD must consume pointer input"
    );
    assert!(
        call(&bus, "osd.list", json!({}))["pointer_presses"]
            .as_u64()
            .unwrap()
            > received,
        "host must positively observe the pointer press"
    );
    let keys = pattern.state.keys;
    bus.call("comp.input.key", json!({"seat":"agent","key":"a"}));
    wait(&mut pattern, Duration::from_millis(50));
    assert!(
        pattern.state.keys > keys,
        "OSD pointer input must preserve the client's keyboard focus"
    );
    call(&bus, "osd.hide", json!({"id":"caption","exit":"fade"}));
    wait(&mut pattern, Duration::from_millis(40));
    let buttons = pattern.state.pointer_buttons;
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"click"}),
    );
    wait(&mut pattern, Duration::from_millis(40));
    assert!(
        pattern.state.pointer_buttons > buttons,
        "exiting card must stop consuming input"
    );
    let fade = motion::presets()
        .into_iter()
        .find(|p| p.name == "fade")
        .unwrap();
    wait(
        &mut pattern,
        Duration::from_secs_f64(fade.timing.duration + 0.25),
    );
    assert!(
        call(&bus, "osd.list", json!({}))["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let (_, _, after) = ppm(&bus, &process, "osd-hidden", None);
    assert_eq!(after, before, "fade must leave no OSD pixels");
    let stopped = bus.call("comp.record.stop", json!({"session":record["session"]}));
    assert_eq!(stopped["dropped"], 0, "{stopped}");

    let mut reader = media_vfl::reader::Reader::open(&path).unwrap();
    let steady_x = shown_bounds[0] + 4;
    let steady_y = shown_bounds[1] + 36;
    assert!(steady_x < width && steady_y < height);
    // This point is below the rounded corner and left of the text inset.
    // Calibrate against the drawn card so its authored background alpha and
    // the compositor's output conversion do not become motion expectations.
    let steady_green_delta = f64::from(rgb(&before, width, steady_x, steady_y)[1])
        - f64::from(rgb(&shown, width, steady_x, steady_y)[1]);
    assert!(
        steady_green_delta > 30.0,
        "card interior must visibly darken the uniform green client"
    );
    let mut observed = Vec::new();
    let mut last = None;
    while let Some(frame) = reader.read().unwrap() {
        let pixels = decoded_rgb(frame.pixels);
        if let Some(bbox) = bounds(&before, &pixels, width, region)
            && last != Some(bbox[1])
        {
            let x = bbox[0] + 4;
            let y = bbox[1] + 36;
            assert!(
                x < width && y < height,
                "motion sample outside output: {bbox:?}"
            );
            let green_delta =
                f64::from(rgb(&before, width, x, y)[1]) - f64::from(rgb(&pixels, width, x, y)[1]);
            observed.push((
                frame.t_us as f64 / 1_000_000.0,
                bbox[1] as f64,
                green_delta / steady_green_delta,
            ));
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
        .take_while(|(_, y, _)| *y > settled + 1.0)
        .copied()
        .collect::<Vec<_>>();
    assert!(
        moving.len() >= 3,
        "VFL must contain intermediate entrance values: {observed:?}"
    );
    // The request is asynchronous relative to the recorder clock. Fit only
    // that one start offset; every decoded position must agree with the
    // canonical pure library sample within raster and one-frame tolerance.
    let (best, start) = (0..1500)
        .map(|ms| {
            let start = ms as f64 / 1000.0;
            let error = moving
                .iter()
                .map(|(t, y, _)| {
                    let expected = settled
                        + preset
                            .sample((t - start).max(0.0))
                            .get(motion::Property::Ty);
                    (y - expected).abs()
                })
                .fold(0.0_f64, f64::max);
            (error, start)
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .unwrap();
    assert!(
        best <= 5.0,
        "decoded entrance disagrees with motion library: max error {best}, samples={moving:?}"
    );
    for (t, _, opacity) in &moving {
        let expected = preset
            .sample((t - start).max(0.0))
            .get(motion::Property::Opacity);
        assert!(
            (opacity - expected).abs() <= 0.12,
            "decoded entrance opacity disagrees with the same motion start: t={t}, start={start}, actual={opacity}, expected={expected}, samples={moving:?}"
        );
    }
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
    assert_eq!(
        (reader.width as usize, reader.height as usize),
        (width, height),
        "the listed fullscreen client must retain the output dimensions"
    );
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

    // A text-only update preserves geometry, visibility and input regions.
    let mut original = caption();
    original.as_object_mut().unwrap().remove("enter");
    call(&bus, "osd.show", original);
    wait(&mut pattern, Duration::from_millis(120));
    let (_, _, old_text) = ppm(&bus, &process, "osd-old-text", None);
    let update_path = process.root.join("osd-update.vfl");
    let update_record = bus.call("comp.record.start", json!({"request_id":"osd-update","target":{"output":"headless-0"},"path":update_path,"cursor":false,"max_seconds":3}));
    call(
        &bus,
        "osd.update",
        json!({"id":"caption","scene":{"kind":"caption","text":"NEW TEXT ONLY","width":320,"height":72}}),
    );
    wait(&mut pattern, Duration::from_millis(180));
    let (_, _, new_text) = ppm(&bus, &process, "osd-new-text", None);
    assert_ne!(old_text, new_text, "text-only update must redraw");
    bus.call(
        "comp.record.stop",
        json!({"session":update_record["session"]}),
    );
    let mut reader = media_vfl::reader::Reader::open(&update_path).unwrap();
    let mut final_update = Vec::new();
    while let Some(frame) = reader.read().unwrap() {
        final_update = decoded_rgb(frame.pixels);
    }
    assert_eq!(
        final_update, new_text,
        "decoded recording must show updated text"
    );
    call(&bus, "osd.hide", json!({"id":"caption"}));
    call(
        &bus,
        "osd.show",
        json!({"id":"badge-cover","scene":{"kind":"card","text":"Cover the corner"},"anchor":{"output":"headless-0","edge":"top-left","margin":0}}),
    );
    wait(&mut pattern, Duration::from_millis(150));
    let (_, _, corner) = ppm(&bus, &process, "osd-badge-cover", None);
    assert_eq!(
        rgb(&corner, width, 12, 10),
        badge,
        "host pixels cannot cover provenance"
    );
    call(&bus, "osd.hide", json!({"id":"badge-cover"}));
    wait(&mut pattern, Duration::from_millis(100));

    // Real renderer oracle: overflowing text in a tiny card must never escape
    // its declared bounds (the compositor badge is outside this comparison).
    call(
        &bus,
        "osd.show",
        json!({"id":"clip","scene":{"kind":"card","title":"LONG TITLE".repeat(100),"text":"LONG TEXT\n".repeat(500),"width":32,"height":24},"anchor":{"output":"headless-0","edge":"top-left","margin":100}}),
    );
    wait(&mut pattern, Duration::from_millis(150));
    let (_, _, clipped) = ppm(&bus, &process, "osd-clipped", None);
    assert!(bounds(&before, &clipped, width, [100, 100, 32, 24]).is_some(), "tiny card must positively draw");
    for y in 20..height {
        for x in 0..width {
            if !(100..132).contains(&x) || !(100..124).contains(&y) {
                assert_eq!(
                    rgb(&clipped, width, x, y),
                    rgb(&before, width, x, y),
                    "text escaped card at {x},{y}"
                );
            }
        }
    }
    call(
        &bus,
        "osd.update",
        json!({"id":"clip","pointer_input":false,"backing":"none"}),
    );
    wait(&mut pattern, Duration::from_millis(80));
    let buttons = pattern.state.pointer_buttons;
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":116,"y":112}),
    );
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"click"}),
    );
    wait(&mut pattern, Duration::from_millis(80));
    assert!(
        pattern.state.pointer_buttons > buttons,
        "explicit transparent item must pass pointer input"
    );
    call(&bus, "osd.hide", json!({"id":"clip"}));
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
    let mut process = process::spawn_osd("osd-window", &bus.url, "1280x720");
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
    let stale = call_rc(
        &bus,
        "osd.show",
        json!({
            "id":"stale",
            "scene":{"kind":"card","text":"must not be installed"},
            "anchor":{"window":{"id":row["id"],"generation":row["generation"].as_u64().unwrap()+1}}
        }),
        10,
    );
    assert_eq!(stale["error_code"], "STALE_GENERATION");
    assert!(
        call(&bus, "osd.list", json!({}))["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
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
    assert_eq!(actions["actions"][0]["class"], "intent");
    assert_eq!(
        actions["actions"][0]["schema"]["required"],
        json!(["id", "scene", "anchor"])
    );
    assert_eq!(
        actions["actions"][0]["schema"]["properties"]["pointer_input"]["type"],
        "boolean"
    );
    call(
        &bus,
        "osd.execute",
        json!({"id":"osd.show","params":caption()}),
    );
    call(
        &bus,
        "osd.execute",
        json!({"id":"osd.update","params":{
            "id":"caption","scene":{"kind":"title","text":"Updated over execute"}
        }}),
    );
    assert_eq!(
        call(&bus, "osd.list", json!({}))["items"][0]["scene"]["kind"],
        "title"
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

    // Pin the rejection oracle: an accepted layer client maps and blocks
    // indefinitely until closed (testkit-layer-client's main dispatch loop).
    let accepted_log = process.root.join("accepted-layer.log");
    let mut accepted = Command::new(binary("testkit-layer-client"))
        .args([
            "--namespace",
            "ordinary-review-control",
            "--layer",
            "overlay",
        ])
        .env("XDG_RUNTIME_DIR", &process.root)
        .env("WAYLAND_DISPLAY", "wayland-headless-test")
        .stdout(fs::File::create(&accepted_log).unwrap())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !fs::read_to_string(&accepted_log)
        .unwrap()
        .contains("mapped")
    {
        assert!(accepted.try_wait().unwrap().is_none());
        assert!(Instant::now() < deadline, "accepted control must map");
        wait(&mut pattern, Duration::from_millis(10));
    }
    wait(&mut pattern, Duration::from_millis(100));
    assert!(
        accepted.try_wait().unwrap().is_none(),
        "accepted client must block, unlike a refused claimant"
    );
    accepted.kill().unwrap();
    accepted.wait().unwrap();
    call(&bus, "osd.show", caption());
    let parent = process.child.id();
    let children = fs::read_to_string(format!("/proc/{parent}/task/{parent}/children")).unwrap();
    let host_pid = children
        .split_whitespace()
        .find(|pid| {
            fs::read_link(format!("/proc/{pid}/exe"))
                .ok()
                .is_some_and(|p| p.file_name().is_some_and(|n| n == "viu"))
        })
        .expect("direct viu child")
        .parse::<i32>()
        .unwrap();
    // SAFETY: this is the dedicated fixture's positively identified child.
    assert_eq!(unsafe { libc::kill(host_pid, libc::SIGKILL) }, 0);
    // Claim during the restart gap; a dead pin must never authorise it.
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
    wait(&mut pattern, Duration::from_millis(1000));
    ready(&bus);
    assert!(
        call(&bus, "osd.list", json!({}))["items"]
            .as_array()
            .unwrap()
            .is_empty(),
        "restart must discard all items before re-showing"
    );
    call(&bus, "osd.show", caption());
    wait(&mut pattern, Duration::from_millis(1100));
    let (_, _, recovered) = ppm(&bus, &process, "osd-restarted", None);
    assert!(
        bounds(&before, &recovered, w, [880, 520, 400, 200]).is_some(),
        "re-pinned host must draw"
    );
}
