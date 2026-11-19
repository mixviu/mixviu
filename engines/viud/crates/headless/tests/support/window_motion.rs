// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native presentation tests: owned headless compositor, Wayland buffers, Bus,
//! and the production VFL exporter. No application or desktop session is used.
use super::*;
use std::io::{BufReader, Read};

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
    let mut pattern = Pattern::named(connect(process), name, false);
    pattern.wait_callback(0);
    let row = bus.window_for(name, &mut [&mut pattern]);
    bus.call(
        "comp.window.place",
        place_args(
            &row,
            json!({
                "x":300,"y":200,"scale":{"factor":1,"anchor":{"x":100,"y":80}}
            }),
        ),
    );
    pattern.solid_buffer(0xffffffff);
    pattern.commit(true);
    pump_patterns(&mut [&mut pattern]);
    (pattern, row)
}

// The fixture is white on the headless background [18,20,25]. Sampling the
// centre avoids chrome and texture-filtered edges. The run length of the
// centre colour measures the scaled client width independently of metadata.
fn visual(pixels: &[u8], width: usize) -> (f64, f64) {
    let at = |x| &pixels[(280 * width + x) * 4..(280 * width + x) * 4 + 3];
    let centre = at(400);
    let opacity = (f64::from(centre[2]) - 18.0) / (255.0 - 18.0);
    if opacity < 0.04 {
        return (opacity, 0.0);
    }
    let same = |x| at(x).iter().zip(centre).all(|(a, b)| a.abs_diff(*b) <= 2);
    let left = (100..400).rev().take_while(|x| same(*x)).count();
    let right = (400..700).take_while(|x| same(*x)).count();
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

#[test]
fn record_window_pop_intermediate_pixels_and_idle_repeats() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("window-pop");
    let mut process = process::spawn(1, None, "window-pop", Some(&bus.url));
    bus.ready();
    let (mut pattern, row) = white_window(&bus, &mut process, "window-pop");
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
    visit_export(&path, |frame, pixels| {
        let (opacity, scale) = visual(pixels, 1280);
        if !(0.04..0.98).contains(&opacity) {
            return;
        }
        intermediate += 1;
        // Pixel rounding and alpha blend quantisation: <=2.5% opacity and
        // <=2.5% scale (five logical pixels across this 200px fixture).
        // This checks pixels against the shared library on the recording's
        // absolute frame clock, within the bounded hold/readback interval.
        let slot =
            (t0 as f64 + frame as f64 * 1_000_000.0 / 30.0 - motion_start as f64) / 1_000_000.0;
        let earliest = (slot - source_age).max(0.0);
        let latest = (slot + 1.0 / 60.0).min(0.3);
        let matched = earliest <= latest
            && (0..=1000).any(|tick| {
                let t = earliest + (latest - earliest) * f64::from(tick) / 1000.0;
                let values = preset.sample(t);
                (values.get(motion::Property::Opacity) - opacity).abs() <= 0.025
                    && (values.get(motion::Property::Scale) - scale).abs() <= 0.025
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
}

fn opacity_track(from: f64, to: f64) -> Value {
    json!([{"property":"opacity","keyframes":[
        {"offset":0,"value":from,"easing":{"kind":"linear"}},
        {"offset":1,"value":to,"easing":{"kind":"linear"}}
    ]}])
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
        pixel(&pixels, 1280, 250, 280),
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
    let mut process = process::spawn_preferences(
        "window-close",
        &bus.url,
        json!({
            "window_motion":{"unmap":{"preset":"fade","duration_ms":300}}
        }),
    );
    bus.ready();
    let (pattern, _row) = white_window(&bus, &mut process, "window-close");
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
        json!({"seat":"agent","output":"headless-0","x":400,"y":280}),
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
        let (opacity, _) = visual(pixels, 1280);
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
    if !process::record_available() { return; }
    let bus=Bus::start("close-rounded-stack");
    let mut process=process::spawn_preferences("close-rounded-stack",&bus.url,json!({
        "decorations_ssd":true,
        "window_motion":{"unmap":{"preset":"fade","duration_ms":600}}
    }));
    bus.ready();
    let mut rear=Pattern::decorated(connect(&mut process),"rounded-rear");
    rear.wait_callback(0);
    let rear_row=bus.window_for("rounded-rear",&mut [&mut rear]);
    bus.call("comp.window.place",place_args(&rear_row,json!({"x":300,"y":200,"z":0})));
    rear.solid_buffer(0xffffffff);
    rear.commit(true);
    let mut front=Pattern::named(connect(&mut process),"opaque-front",false);
    front.wait_callback(0);
    let front_row=bus.window_for("opaque-front",&mut [&mut rear,&mut front]);
    bus.call("comp.window.place",place_args(&front_row,json!({"x":400,"y":260,"z":10})));
    front.solid_buffer(0xff20d040);
    front.commit(true);
    pump_patterns(&mut [&mut rear,&mut front]);
    let (width,height,before)=ppm(&bus,&process,"rounded-stack-before",None);
    assert_eq!((width,height),(1280,720));
    assert_eq!(pixel(&before,width,350,280),[255,255,255]);
    assert_eq!(pixel(&before,width,450,300),[0x20,0xd0,0x40]);
    // Find a shader-cut pixel INSIDE the rectangular client buffer, at its
    // bottom-left corner. This proves the fixture has rounded client clipping;
    // a rectangular snapshot would paint white here as soon as it faded.
    let (corner_x,corner_y,corner_red)=(300..308).flat_map(|x| (352..360).map(move |y| (x,y)))
        .map(|(x,y)| (x,y,pixel(&before,width,x,y)[0]))
        .min_by_key(|(_,_,red)| *red).unwrap();
    assert!(corner_red<60,"server-decorated client must have a rounded bottom corner; found {corner_red}");
    let path=process.root.join("rounded-stack.vfl");
    let start=bus.call("comp.record.start",json!({"request_id":"rounded-stack","target":{"output":"headless-0"},"path":path,"max_seconds":10,"fps":30,"cursor":false}));
    drop(rear);
    for _ in 0..160 {
        front.pump();
        thread::sleep(Duration::from_millis(5));
    }
    let stop=bus.call("comp.record.stop",json!({"session":start["session"]}));
    assert_eq!(stop["dropped"],0,"{stop}");
    let mut fading_frames=0;
    let mut final_rear=255;
    visit_export(&path,|frame,pixels| {
        let at=|x,y| {
            let p=&pixels[(y*width+x)*4..(y*width+x)*4+4];
            [p[2],p[1],p[0]]
        };
        assert_eq!(at(450,300),[0x20,0xd0,0x40],"frame {frame}: rear ghost must stay behind the opaque front window");
        let rear_red=at(350,280)[0];
        if (30..245).contains(&rear_red) {
            fading_frames+=1;
            assert!(at(corner_x,corner_y)[0]<=corner_red+8,"frame {frame}: retained client texture must keep its rounded shader cut");
        }
        final_rear=rear_red;
    });
    assert!(fading_frames>=3,"close must show intermediate retained-buffer frames");
    assert_eq!(final_rear,18,"rear window disappears when its exit completes");
}

#[test]
fn record_window_retarget_supersedes_without_returning_to_first_keyframe() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("window-retarget");
    let mut process = process::spawn(1, None, "window-retarget", Some(&bus.url));
    bus.ready();
    let (mut pattern, row) = white_window(&bus, &mut process, "window-retarget");
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
            "id":row["id"],"generation":row["generation"],"tracks":first_tracks,"duration_ms":800
        }}]}))
        });
        thread::sleep(Duration::from_millis(250));
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
    visit_export(&path, |_, pixels| values.push(visual(pixels, 1280)));
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
        "a replacement omitting scale must not drop the running scale track: {values:?}"
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
