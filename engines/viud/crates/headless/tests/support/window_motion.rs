// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native presentation tests: owned headless compositor, Wayland buffers, Bus,
//! and the production VFL exporter. No application or desktop session is used.
use super::*;
use std::io::{BufReader, Read};

fn ppm_pixel(bytes: &[u8], width: usize, x: usize, y: usize) -> [u8; 3] {
    let at = (y * width + x) * 3;
    bytes[at..at + 3].try_into().unwrap()
}

fn visit_export(path: &Path, mut visit: impl FnMut(usize, &[u8])) {
    let manifest: Value =
        serde_json::from_slice(&fs::read(path.join("manifest.json")).unwrap()).unwrap();
    let complete: Value =
        serde_json::from_slice(&fs::read(path.join("complete.json")).unwrap()).unwrap();
    let frame_bytes = manifest["width"].as_u64().unwrap() as usize
        * manifest["height"].as_u64().unwrap() as usize
        * 4;
    let exported = path.with_extension("bgra");
    let output = fs::File::create(&exported).unwrap();
    let status = Command::new(binary("mixd"))
        .args(["vfl", "export"])
        .arg(path)
        .args(["--format", "bgra"])
        .stdout(output)
        .status()
        .unwrap();
    assert!(status.success(), "production VFL export failed: {path:?}");
    let mut reader = BufReader::new(fs::File::open(&exported).unwrap());
    let mut frame = vec![0; frame_bytes];
    for index in 0..complete["frames"].as_u64().unwrap() as usize {
        reader.read_exact(&mut frame).unwrap();
        visit(index, &frame);
    }
    assert_eq!(
        reader.read(&mut [0]).unwrap(),
        0,
        "export has exactly the recorded frames"
    );
}

fn white_window(bus: &Bus, process: &mut process::Process, name: &str) -> (Pattern, Value) {
    white_window_at(bus, process, name, 300, 200)
}

fn white_window_at(
    bus: &Bus,
    process: &mut process::Process,
    name: &str,
    x: i32,
    y: i32,
) -> (Pattern, Value) {
    white_window_sized(bus, process, name, x, y, (200, 160))
}

fn white_window_sized(
    bus: &Bus,
    process: &mut process::Process,
    name: &str,
    x: i32,
    y: i32,
    size: (i32, i32),
) -> (Pattern, Value) {
    let mut pattern = Pattern::sized(connect(process), name, size, false);
    pattern.wait_callback(0);
    let row = bus.window_for(name, &mut [&mut pattern]);
    bus.call(
        "comp.window.place",
        place_args(
            &row,
            json!({
                "x":x,"y":y,"scale":{"factor":1,"anchor":{"x":100,"y":80}}
            }),
        ),
    );
    pattern.solid_buffer(0xffffffff);
    pattern.commit(true);
    pump_patterns(&mut [&mut pattern]);
    (pattern, row)
}

// Keep functional recordings small enough for the workspace's debug compositor
// under its parallel pool. The 200x160 client, 30Hz samples and pixel oracles
// stay unchanged; the separate throughput acceptance still records 4K30.
const MOTION_SIZE: &str = "300x220";
const MOTION_WIDTH: usize = 300;
// Close tests need only colour, corner and stacking probes, not the pop's
// 200px scale ruler. Bound their per-frame debug encoder work further.
const CLOSE_SIZE: &str = "240x180";
const CLOSE_WIDTH: usize = 240;

fn opacity_at(pixels: &[u8], width: usize, x: usize, y: usize) -> f64 {
    (f64::from(pixels[(y * width + x) * 4 + 2]) - 18.0) / (255.0 - 18.0)
}

// The fixture is white on the headless background [18,20,25]. Sampling the
// centre avoids chrome and texture-filtered edges. The run length of the
// centre colour measures the scaled client width independently of metadata.
fn visual(pixels: &[u8], width: usize) -> (f64, f64) {
    let at = |x| &pixels[(110 * width + x) * 4..(110 * width + x) * 4 + 3];
    let centre = at(150);
    let opacity = (f64::from(centre[2]) - 18.0) / (255.0 - 18.0);
    if opacity < 0.04 {
        return (opacity, 0.0);
    }
    let same = |x| at(x).iter().zip(centre).all(|(a, b)| a.abs_diff(*b) <= 2);
    let left = (0..150).rev().take_while(|x| same(*x)).count();
    let right = (150..width).take_while(|x| same(*x)).count();
    (opacity, (left + right) as f64 / 200.0)
}

fn trace_field(line: &str, key: &str) -> u64 {
    line.split_whitespace()
        .find_map(|s| s.strip_prefix(key)?.parse().ok())
        .unwrap()
}

fn frame_cpu_samples(process: &process::Process, begin_us: u64, end_us: u64) -> Vec<u64> {
    let field = |line: &str, key: &str| -> u64 {
        line.split_whitespace()
            .find_map(|s| s.strip_prefix(key)?.parse().ok())
            .unwrap()
    };
    let mut samples: Vec<_> = fs::read_to_string(process.root.join("frames.log"))
        .unwrap()
        .lines()
        .filter(|s| s.contains("stage=headless_frame_cpu "))
        .filter(|s| (begin_us..end_us).contains(&field(s, "start_us=")))
        .map(|s| field(s, "duration_us="))
        .collect();
    samples.sort_unstable();
    samples
}

fn redraw_requests(process: &process::Process, begin: u64, end: u64) -> usize {
    fs::read_to_string(process.root.join("frames.log"))
        .unwrap()
        .lines()
        .filter(|line| line.contains("stage=comp_redraw_request "))
        .filter(|line| (begin..end).contains(&trace_field(line, "start_us=")))
        .count()
}

#[test]
fn motion_redraws_follow_output_frames_and_settle_without_requests() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("motion-redraw-clock");
    let mut process = process::spawn_size("motion-redraw-clock", &bus.url, MOTION_SIZE);
    bus.ready();
    let (mut pattern, row) = white_window_at(&bus, &mut process, "motion-redraw-clock", 50, 30);
    let _ = ppm(&bus, &process, "ready", None);
    let begin = monotonic_ns() / 1000;
    thread::scope(|scope| {
        let motion = scope.spawn(|| {
            bus.call(
                "comp.motion.run",
                json!({"targets":[{"window":{
                    "id":row["id"], "generation":row["generation"],
                    "tracks":opacity_track(0.2,1.0),"duration_ms":1200
                }}]}),
            )
        });
        // Keep unrelated protocol and Bus events arriving between output ticks.
        while !motion.is_finished() {
            pattern.pump();
            bus.call("comp.windows.list", json!({}));
        }
        assert_eq!(motion.join().unwrap()["status"], "completed");
    });
    let end = monotonic_ns() / 1000;
    let requests = redraw_requests(&process, begin, end);
    let nominal_frames = ((end - begin) * 60).div_ceil(1_000_000) as usize;
    println!("MOTION_REDRAWS output=headless-0 frames={nominal_frames} requests={requests}");
    assert!(
        requests <= 3 * nominal_frames + 8,
        "animation wakes must follow output frames: {requests} requests for {nominal_frames} frames"
    );
    assert!(!frame_cpu_samples(&process, begin, end).is_empty());
    // Drain the endpoint's already owed frame before checking a settled scene.
    let _ = ppm(&bus, &process, "settled", None);
    pump_patterns(&mut [&mut pattern]);
    thread::sleep(Duration::from_millis(100));
    let idle_begin = monotonic_ns() / 1000;
    thread::sleep(Duration::from_millis(350));
    let idle_end = monotonic_ns() / 1000;
    let idle = redraw_requests(&process, idle_begin, idle_end);
    println!("MOTION_REDRAWS output=headless-0 idle_requests={idle}");
    assert_eq!(idle, 0, "settled scene must make zero requests");
    assert!(frame_cpu_samples(&process, idle_begin, idle_end).is_empty());
}

#[test]
fn record_window_pop_intermediate_pixels_and_idle_repeats() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("window-pop");
    let mut process = process::spawn_size("window-pop", &bus.url, MOTION_SIZE);
    bus.ready();
    let (mut pattern, row) = white_window_at(&bus, &mut process, "window-pop", 50, 30);
    let path = process.root.join("pop.vfl");
    let start = bus.call("comp.record.start",json!({"request_id":"pop","target":{"output":"headless-0"},"path":path,"max_seconds":10,"fps":30,"cursor":false}));
    let reply = bus.call(
        "comp.motion.run",
        json!({"targets":[{"window":{
            "id":row["id"],"generation":row["generation"],"preset":"pop","duration_ms":300
        }}]}),
    );
    assert_eq!(reply["status"], "completed", "{reply}");
    pump_patterns(&mut [&mut pattern]);
    let before = bus.call("comp.record.status", json!({"session":start["session"]}));
    let idle_begin = monotonic_ns() / 1000;
    thread::sleep(Duration::from_millis(350));
    let idle_end = monotonic_ns() / 1000;
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["dropped"], 0, "{stop}");
    assert_eq!(
        stop["readbacks"], before["readbacks"],
        "completed motion must repeat without readbacks: {stop}"
    );
    assert!(stop["repeated"].as_u64().unwrap() > before["repeated"].as_u64().unwrap());
    assert!(
        frame_cpu_samples(&process, idle_begin, idle_end).is_empty(),
        "completed animation must not schedule frames"
    );
    let mut preset = motion::presets()
        .into_iter()
        .find(|p| p.name == "pop")
        .unwrap();
    preset.timing.duration = 0.3;
    let trace = fs::read_to_string(process.root.join("frames.log")).unwrap();
    let motion_start = trace
        .lines()
        .find(|line| {
            line.contains("stage=window_motion_start ")
                && trace_field(line, "subject=") == row["id"].as_u64().unwrap()
        })
        .map(|line| trace_field(line, "start_us="))
        .expect("frame trace must expose the motion clock origin");
    // Recording holds the latest submitted scene until the next 30Hz sample.
    // Its origin is capture completion. Bound that source age by one recorder
    // period plus the measured maximum readback latency; allow one output
    // period for a frame assembled just after the nominal recording slot.
    let source_age = 1.0 / 30.0 + stop["readback_max_us"].as_u64().unwrap() as f64 / 1_000_000.0;
    let t0 = start["t0_us"].as_u64().unwrap();
    let mut intermediate = 0;
    let mut overshoot = 0;
    visit_export(&path, |frame, pixels| {
        let (opacity, scale) = visual(pixels, MOTION_WIDTH);
        let slot =
            (t0 as f64 + frame as f64 * 1_000_000.0 / 30.0 - motion_start as f64) / 1_000_000.0;
        // Slots can hold a source scene from before command admission. That
        // source is the known identity window, not the preset's first keyframe.
        // Every slot after admission still has a bounded-source-age oracle.
        if slot < 0.0 {
            return;
        }
        if (0.04..0.98).contains(&opacity) {
            intermediate += 1;
        }
        if scale > 1.025 {
            overshoot += 1;
        }
        // Pixel rounding and alpha blend quantisation: <=2.5% opacity and
        // <=2.5% scale (five logical pixels across this 200px fixture).
        // This checks pixels against the shared library on the recording's
        // absolute frame clock, within the bounded hold/readback interval.
        let earliest = (slot - source_age).clamp(0.0, 0.3);
        let latest = (slot + 1.0 / 60.0).clamp(0.0, 0.3);
        let held_initial =
            slot <= source_age && (opacity - 1.0).abs() <= 0.025 && (scale - 1.0).abs() <= 0.025;
        let matched = held_initial
            || earliest <= latest
                && (0..=1000).any(|tick| {
                    let t = earliest + (latest - earliest) * f64::from(tick) / 1000.0;
                    let values = preset.sample(t);
                    (values.get(motion::Property::Opacity) - opacity).abs() <= 0.025
                        && (opacity < 0.04
                            || (values.get(motion::Property::Scale) - scale).abs() <= 0.025)
                });
        assert!(
            matched,
            "frame={frame} slot={slot} allowed={earliest}..{latest} opacity={opacity} scale={scale} is outside the shared pop trajectory"
        );
        println!(
            "VFL_POP_SAMPLE frame={frame} slot_s={slot:.6} source_age_limit_s={source_age:.6} opacity={opacity:.5} scale={scale:.5}"
        );
    });
    assert!(
        intermediate >= 2,
        "300ms at 30Hz must expose intermediate opacity and scale"
    );
    assert!(
        overshoot > 0,
        "pop must expose its opaque scale overshoot, then return to identity"
    );
}

fn opacity_track(from: f64, to: f64) -> Value {
    json!([{"property":"opacity","keyframes":[
        {"offset":0,"value":from,"easing":{"kind":"linear"}},
        {"offset":1,"value":to,"easing":{"kind":"linear"}}
    ]}])
}

#[test]
fn tiny_animated_scale_suppresses_retained_grab() {
    let bus = Bus::start("tiny-grab");
    let mut process = process::spawn(1, None, "tiny-grab", Some(&bus.url));
    bus.ready();
    let (mut pattern, row) = white_window(&bus, &mut process, "tiny-grab");
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":400,"y":280}),
    );
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"press"}),
    );
    pump_patterns(&mut [&mut pattern]);
    let before = pattern.state.pointer_positions.len();
    bus.call("comp.motion.run", json!({"targets":[{"window":{"id":row["id"],"generation":row["generation"],"duration_ms":0,"tracks":[{"property":"scale","keyframes":[{"offset":0,"value":1},{"offset":1,"value":1e-310}]}]}}]}));
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":450,"y":280}),
    );
    pump_patterns(&mut [&mut pattern]);
    assert_eq!(
        pattern.state.pointer_positions.len(),
        before,
        "tiny scale cancels positional grab routing"
    );
    assert!(
        pattern
            .state
            .pointer_positions
            .iter()
            .all(|(x, y)| x.is_finite() && y.is_finite())
    );
}

#[test]
fn scaled_window_input_follows_pixels_and_uses_client_coordinates() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("scaled-input");
    let mut process = process::spawn(1, None, "scaled-input", Some(&bus.url));
    bus.ready();
    let (mut pattern, row) = white_window(&bus, &mut process, "scaled-input");
    bus.call(
        "comp.window.place",
        place_args(
            &row,
            json!({"scale":{"factor":2,"anchor":{"x":100,"y":80}}}),
        ),
    );
    let (_, _, pixels) = ppm(&bus, &process, "scaled-input", None);
    assert_eq!(
        ppm_pixel(&pixels, 1280, 250, 280),
        [255, 255, 255],
        "scaled pixels extend outside the configured rectangle"
    );
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":250,"y":280}),
    );
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"click"}),
    );
    pump_patterns(&mut [&mut pattern]);
    assert!(
        pattern.state.pointer_buttons >= 2,
        "input follows enlarged pixels"
    );
    let &(x, y) = pattern
        .state
        .pointer_positions
        .last()
        .expect("surface-local pointer coordinates");
    assert!(
        (x - 25.0).abs() <= 0.5 && (y - 80.0).abs() <= 0.5,
        "inverse drawn transform must give (25,80), got ({x},{y})"
    );
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"press"}),
    );
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":650,"y":280}),
    );
    pump_patterns(&mut [&mut pattern]);
    let &(x, y) = pattern
        .state
        .pointer_positions
        .last()
        .expect("grabbed surface-local motion");
    assert!(
        (x - 225.0).abs() <= 0.5 && (y - 80.0).abs() <= 0.5,
        "implicit grab outside the drawn window must retain inverse mapping, got ({x},{y})"
    );
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"release"}),
    );
    let after = bus.window_for("scaled-input", &mut [&mut pattern]);
    assert_eq!(
        after["window_width"], row["window_width"],
        "composition scale never configures a larger client"
    );
    assert_eq!(after["window_height"], row["window_height"]);
}

#[test]
fn record_close_retains_pixels_without_retaining_client_or_input() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("window-close");
    let mut process = process::spawn_preferences_size(
        "window-close",
        &bus.url,
        CLOSE_SIZE,
        json!({
            "window_motion":{"unmap":{"preset":"fade","duration_ms":300}}
        }),
    );
    bus.ready();
    let (pattern, _row) = white_window_sized(&bus, &mut process, "window-close", 50, 40, (100, 80));
    let path = process.root.join("close.vfl");
    let start=bus.call("comp.record.start",json!({"request_id":"close","target":{"output":"headless-0"},"path":path,"max_seconds":10,"fps":30,"cursor":false}));
    // Drop the entire Wayland connection. A ghost is only presentation data;
    // it must not hold the role or pretend the destroyed client is still live.
    drop(pattern);
    let begin = Instant::now();
    loop {
        let rows = bus.call("comp.windows.list", json!({"app_id":"window-close"}));
        if rows["windows"].as_array().unwrap().is_empty() {
            break;
        }
        assert!(
            begin.elapsed() < Duration::from_millis(250),
            "client destruction must not wait for the 300ms animation"
        );
    }
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":100,"y":80}),
    );
    let refused = bus.call_rc(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"click"}),
        10,
    );
    assert_eq!(
        refused["error"], "no_pointer_target",
        "a closing ghost never receives input: {refused}"
    );
    thread::sleep(Duration::from_millis(400));
    let before = bus.call("comp.record.status", json!({"session":start["session"]}));
    thread::sleep(Duration::from_millis(200));
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(
        stop["readbacks"], before["readbacks"],
        "completed close animation repeats"
    );
    assert_eq!(stop["dropped"], 0, "{stop}");
    let mut intermediate = 0;
    let mut last = 1.0;
    visit_export(&path, |_, pixels| {
        let opacity = opacity_at(pixels, CLOSE_WIDTH, 100, 80);
        if (0.04..0.98).contains(&opacity) {
            intermediate += 1;
        }
        last = opacity;
    });
    assert!(
        intermediate >= 2,
        "destroyed window's last white buffer must remain visible while fading"
    );
    assert!(
        last < 0.01,
        "last buffer must be dropped after the close animation"
    );
}

#[test]
fn record_close_retains_rounded_corners_and_rear_stacking() {
    rounded_close_stack(false);
}

#[test]
fn record_close_retains_rounded_corners_and_front_stacking() {
    rounded_close_stack(true);
}

fn rounded_close_stack(closing_front: bool) {
    if !process::record_available() {
        return;
    }
    let label = if closing_front {
        "close-rounded-front"
    } else {
        "close-rounded-rear"
    };
    let bus = Bus::start(label);
    let mut process = process::spawn_preferences_size(
        label,
        &bus.url,
        CLOSE_SIZE,
        json!({
            "decorations_ssd":true,
            "window_motion":{"unmap":{"preset":"fade","duration_ms":600}}
        }),
    );
    bus.ready();
    let mut rear = Pattern::sized(connect(&mut process), "rounded-rear", (100, 80), true);
    rear.wait_callback(0);
    let rear_row = bus.window_for("rounded-rear", &mut [&mut rear]);
    bus.call(
        "comp.window.place",
        place_args(
            &rear_row,
            json!({"x":50,"y":40,"z":if closing_front {20} else {0}}),
        ),
    );
    rear.solid_buffer(0xffffffff);
    rear.commit(true);
    let mut front = Pattern::sized(connect(&mut process), "opaque-front", (100, 80), false);
    front.wait_callback(0);
    let front_row = bus.window_for("opaque-front", &mut [&mut rear, &mut front]);
    bus.call(
        "comp.window.place",
        place_args(&front_row, json!({"x":100,"y":70,"z":10})),
    );
    front.solid_buffer(0xff20d040);
    front.commit(true);
    pump_patterns(&mut [&mut rear, &mut front]);
    assert_eq!(rear.state.size, (100, 80));
    assert_eq!(front.state.size, (100, 80));
    let (width, height, before) = ppm(&bus, &process, "rounded-stack-before", None);
    assert_eq!((width, height), (CLOSE_WIDTH, 180));
    assert_eq!(ppm_pixel(&before, width, 75, 80), [255, 255, 255]);
    assert_eq!(
        ppm_pixel(&before, width, 125, 90),
        if closing_front {
            [255, 255, 255]
        } else {
            [0x20, 0xd0, 0x40]
        }
    );
    // Find a shader-cut pixel INSIDE the rectangular client buffer, at its
    // bottom-left corner. This proves the fixture has rounded client clipping;
    // a rectangular snapshot would paint white here as soon as it faded.
    let (corner_x, corner_y, corner_red) = (50..58)
        .flat_map(|x| (112..120).map(move |y| (x, y)))
        .map(|(x, y)| (x, y, ppm_pixel(&before, width, x, y)[0]))
        .min_by_key(|(_, _, red)| *red)
        .unwrap();
    assert!(
        corner_red < 60,
        "server-decorated client must have a rounded bottom corner; found {corner_red}"
    );
    // A screenshot can render before the normal presentation path. Exits
    // deliberately require evidence from the preceding presented frame.
    let presented = rear.state.callbacks;
    rear.commit(true);
    rear.wait_callback(presented);
    let path = process.root.join("rounded-stack.vfl");
    let start=bus.call("comp.record.start",json!({"request_id":"rounded-stack","target":{"output":"headless-0"},"path":path,"max_seconds":10,"fps":30,"cursor":false}));
    drop(rear);
    for _ in 0..160 {
        front.pump();
        thread::sleep(Duration::from_millis(5));
    }
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["dropped"], 0, "{stop}");
    let mut fading_frames = 0;
    let mut final_rear = 255;
    visit_export(&path, |frame, pixels| {
        let at = |x, y| {
            let p = &pixels[(y * width + x) * 4..(y * width + x) * 4 + 4];
            [p[2], p[1], p[0]]
        };
        let rear_red = at(75, 80)[0];
        if closing_front {
            // SSD retains its black shadow behind the fading white client.
            // Thus the underlay's transmission is (1-client_alpha) *
            // (1-shadow_alpha), not simply 1-client_alpha. Both probes lie
            // inside the shadow's constant centre tile. Infer transmission
            // from green-minus-red, which cancels the white foreground, then
            // independently check blue and the white-over-background probe.
            let over = at(125, 90);
            let transmission = (f64::from(over[1]) - f64::from(over[0])) / (208.0 - 32.0);
            let foreground = f64::from(over[0]) - 32.0 * transmission;
            assert!((-0.025..=1.025).contains(&transmission));
            assert!(foreground >= -4.0 && foreground <= 255.0 * (1.0 - transmission) + 4.0);
            assert!(
                (f64::from(rear_red) - (foreground + 18.0 * transmission)).abs() <= 4.0,
                "frame {frame}: retained white client and shadow must composite above both underlays"
            );
            for (actual, under) in over.into_iter().zip([0x20, 0xd0, 0x40]) {
                let expected = foreground + f64::from(under) * transmission;
                assert!(
                    (f64::from(actual) - expected).abs() <= 4.0,
                    "frame {frame}: top closing ghost must blend above the surviving green window, got {actual}, expected {expected}"
                );
            }
        } else {
            assert_eq!(
                at(125, 90),
                [0x20, 0xd0, 0x40],
                "frame {frame}: rear ghost must stay behind the opaque front window"
            );
        }
        if (30..245).contains(&rear_red) {
            fading_frames += 1;
            assert!(
                at(corner_x, corner_y)[0] <= corner_red + 8,
                "frame {frame}: retained client texture must keep its rounded shader cut"
            );
        }
        final_rear = rear_red;
    });
    assert!(
        fading_frames >= 3,
        "close must show intermediate retained-buffer frames"
    );
    assert_eq!(
        final_rear, 18,
        "rear window disappears when its exit completes"
    );
}

#[test]
fn record_window_retarget_supersedes_without_returning_to_first_keyframe() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("window-retarget");
    let mut process = process::spawn_size("window-retarget", &bus.url, MOTION_SIZE);
    bus.ready();
    let (mut pattern, row) = white_window_at(&bus, &mut process, "window-retarget", 50, 30);
    // Establish the initial pixels and warm the renderer's readback allocation
    // before measuring motion. A cold first copy under the workspace pool took
    // 17.6ms and incurred recorder debt before retargeting even began.
    let (width, height, pixels) = ppm(&bus, &process, "retarget-ready", None);
    assert_eq!((width, height), (MOTION_WIDTH, 220));
    assert_eq!(ppm_pixel(&pixels, width, 150, 110), [255, 255, 255]);
    let path = process.root.join("retarget.vfl");
    let start = bus.call("comp.record.start",json!({"request_id":"retarget","target":{"output":"headless-0"},"path":path,"max_seconds":10,"fps":30,"cursor":false}));
    let mut first_tracks = opacity_track(1.0, 0.2);
    first_tracks
        .as_array_mut()
        .unwrap()
        .push(json!({"property":"scale","keyframes":[
            {"offset":0,"value":0.4,"easing":{"kind":"linear"}},
            {"offset":1,"value":0.8,"easing":{"kind":"linear"}}
        ]}));
    thread::scope(|scope| {
        let first = scope.spawn(|| {
            bus.call("comp.motion.run",json!({"targets":[{"window":{
            "id":row["id"],"generation":row["generation"],"tracks":first_tracks,"duration_ms":4000
        }}]}))
        });
        thread::sleep(Duration::from_millis(800));
        let second = bus.call("comp.motion.run",json!({"targets":[{"window":{
            "id":row["id"],"generation":row["generation"],"tracks":opacity_track(0.0,1.0),"duration_ms":500
        }}]}));
        assert_eq!(first.join().unwrap()["status"], "superseded");
        assert_eq!(second["status"], "completed");
    });
    pump_patterns(&mut [&mut pattern]);
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["dropped"], 0, "{stop}");
    let mut values = Vec::new();
    visit_export(&path, |_, pixels| values.push(visual(pixels, MOTION_WIDTH)));
    assert!(
        values.iter().any(|v| v.0 < 0.9),
        "the first run must actually change pixels"
    );
    assert!(
        values.iter().all(|v| v.0 > 0.4),
        "retarget must not jump to the second run's explicit zero endpoint: {values:?}"
    );
    assert!(
        values.windows(2).all(|v| (v[1].0 - v[0].0).abs() < 0.06),
        "adjacent 30Hz frames stay within the linear velocity bound plus blend quantisation: {values:?}"
    );
    assert!(
        values
            .windows(2)
            .filter(|v| v[0].0 < 0.9 && v[1].0 < 0.9)
            .all(|v| (v[1].1 - v[0].1).abs() < 0.08),
        "a replacement omitting scale must be continuous: {values:?}"
    );
    assert!(
        (values.last().unwrap().1 - 0.8).abs() <= 0.025,
        "omitted scale continues to its original endpoint: {values:?}"
    );
}

#[test]
#[ignore = "native render-node frame-time benchmark"]
fn benchmark_window_motion_1_8_32() {
    assert!(process::record_available());
    for count in [1, 8, 32] {
        let bus = Bus::start(&format!("window-bench-{count}"));
        let mut process = process::spawn(1, None, &format!("window-bench-{count}"), Some(&bus.url));
        process.extend_watchdog(Duration::from_secs(120));
        bus.ready();
        let mut patterns = Vec::new();
        let mut rows = Vec::new();
        for index in 0..count {
            let (pattern, row) = white_window(&bus, &mut process, &format!("motion-bench-{index}"));
            // A visible 8x4 grid avoids occluding the work being measured.
            bus.call("comp.window.place",place_args(&row,json!({"x":(index%8)*150,"y":(index/8)*170,"scale":{"factor":0.65,"anchor":{"x":0,"y":0}}})));
            patterns.push(pattern);
            rows.push(row);
        }
        let mut clients: Vec<_> = patterns.iter_mut().collect();
        pump_patterns(&mut clients);
        drop(clients);
        let (width, height, visible) = ppm(&bus, &process, "benchmark-visible-grid", None);
        assert_eq!((width, height), (1280, 720));
        for index in 0..count {
            let x = (index % 8) * 150 + 50;
            let y = (index / 8) * 170 + 50;
            assert_eq!(
                ppm_pixel(&visible, width, x, y),
                [255, 255, 255],
                "benchmark must draw all {count} scaled windows: window {index} at ({x},{y})"
            );
        }
        let begin = monotonic_ns() / 1000;
        thread::scope(|scope| {
            let bus_ref = &bus;
            let handles: Vec<_> = rows.iter().map(|row| scope.spawn(move || {
                bus_ref.call("comp.motion.run",json!({"targets":[{"window":{
                    "id":row["id"],"generation":row["generation"],"tracks":opacity_track(0.2,1.0),"duration_ms":2000
                }}]}))
            })).collect();
            while handles.iter().any(|h| !h.is_finished()) {
                for pattern in &mut patterns {
                    pattern.pump();
                }
                thread::sleep(Duration::from_millis(2));
            }
            for handle in handles {
                assert_eq!(handle.join().unwrap()["status"], "completed");
            }
        });
        let end = monotonic_ns() / 1000;
        thread::sleep(Duration::from_millis(50));
        let samples = frame_cpu_samples(&process, begin, end);
        assert!(samples.len() > 20, "benchmark must render animated frames");
        println!(
            "WINDOW_MOTION count={count} frames={} median_us={} p95_us={} max_us={} output=1280x720 recording=false",
            samples.len(),
            samples[samples.len() / 2],
            samples[samples.len() * 95 / 100],
            samples.last().unwrap()
        );
    }
}
