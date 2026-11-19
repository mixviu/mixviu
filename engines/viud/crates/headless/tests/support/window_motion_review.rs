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
    if !process::record_available() { return; }
    let bus = Bus::start("motion-review-entrance");
    let mut process = process::spawn_preferences("motion-review-entrance", &bus.url, json!({
        "window_motion":{"map":{"preset":"pop","duration_ms":300}}
    }));
    bus.ready();
    let (mut client, row) = solid_window(&bus, &mut process, "motion-review-entrance", 0xffffffff, 300, 200);
    thread::sleep(Duration::from_millis(400));
    pump_patterns(&mut [&mut client]);
    let (width, _, original) = ppm(&bus, &process, "completed-entrance", None);
    assert_eq!(rgb_at(&original, width, 400, 280), [255,255,255]);
    bus.call("comp.window.place", place_args(&row, json!({
        "opacity":0.3,"scale":{"factor":0.5,"anchor":{"x":100,"y":80}}
    })));
    let after = bus.window_for("motion-review-entrance", &mut [&mut client]);
    assert_eq!(after["opacity"], 0.3, "completed lifecycle tracks must release opacity ownership");
    assert_eq!(after["scale"]["factor"], 0.5, "completed lifecycle tracks must release scale ownership");
    let (width, _, placed) = ppm(&bus, &process, "placed-after-entrance", None);
    for (actual, background) in rgb_at(&placed, width, 400, 280).into_iter().zip([18.0,20.0,25.0]) {
        let expected = 255.0*0.3 + background*0.7;
        assert!((f64::from(actual)-expected).abs()<=3.0, "window-place opacity must affect drawn pixels after entrance: {actual} vs {expected}");
    }
    assert_eq!(rgb_at(&placed,width,320,280), [18,20,25], "window-place scale must shrink the completed entrance's client image");
}

#[test]
fn explicit_hide_is_immediate_when_workspace_transitions_are_enabled() {
    if !process::record_available() { return; }
    let bus = Bus::start("motion-review-hide");
    let mut process = process::spawn_preferences("motion-review-hide", &bus.url, json!({
        "window_motion":{"workspace":{"preset":"fade","duration_ms":1000}}
    }));
    bus.ready();
    let (mut client, row) = solid_window(&bus, &mut process, "motion-review-hide", 0xffffffff, 300, 200);
    let (width, _, shown) = ppm(&bus, &process, "before-explicit-hide", None);
    assert_eq!(rgb_at(&shown,width,400,280),[255,255,255]);
    bus.call("comp.window.place", place_args(&row,json!({"visible":false})));
    let hidden = bus.window_for("motion-review-hide", &mut [&mut client]);
    assert_eq!(hidden["requested_visible"],false);
    assert_eq!(hidden["visible"],false);
    let (width, _, pixels) = ppm(&bus, &process, "immediately-hidden", None);
    assert_eq!(rgb_at(&pixels,width,400,280),[18,20,25],"explicit hide must not create a workspace exit ghost");
    bus.call("comp.input.pointer.move",json!({"seat":"agent","output":"headless-0","x":400,"y":280}));
    let refused=bus.call_rc("comp.input.pointer.button",json!({"seat":"agent","button":"left","action":"click"}),10);
    assert_eq!(refused["error"],"no_pointer_target");
}

fn exported_frames(path: &Path, mut visit: impl FnMut(usize,usize,&[u8])) {
    let manifest: Value=serde_json::from_slice(&fs::read(path.join("manifest.json")).unwrap()).unwrap();
    let complete: Value=serde_json::from_slice(&fs::read(path.join("complete.json")).unwrap()).unwrap();
    let width=manifest["width"].as_u64().unwrap() as usize;
    let frame_bytes=width*manifest["height"].as_u64().unwrap() as usize*4;
    let exported=path.with_extension("bgra");
    let result=Command::new(binary("mixd")).args(["vfl","export"]).arg(path)
        .args(["--format","bgra"]).stdout(fs::File::create(&exported).unwrap()).status().unwrap();
    assert!(result.success(),"production VFL export must succeed");
    let mut reader=BufReader::new(fs::File::open(exported).unwrap());
    let mut pixels=vec![0;frame_bytes];
    for frame in 0..complete["frames"].as_u64().unwrap() as usize {
        reader.read_exact(&mut pixels).unwrap();
        visit(frame,width,&pixels);
    }
    assert_eq!(reader.read(&mut [0]).unwrap(),0);
}

#[test]
fn close_ghost_cannot_leak_into_a_different_workspace_in_recorded_frames() {
    if !process::record_available() { return; }
    let bus=Bus::start("motion-review-workspace");
    let mut process=process::spawn_preferences("motion-review-workspace",&bus.url,json!({
        "window_motion":{"unmap":{"preset":"fade","duration_ms":1000}}
    }));
    bus.ready();
    bus.call("comp.workspaces.set_count",json!({"count":2,"persist":false}));
    let (mut closing,_closing_row)=solid_window(&bus,&mut process,"workspace-one-private",0xffffffff,300,200);
    let (mut sentinel,sentinel_row)=solid_window(&bus,&mut process,"workspace-two-sentinel",0xff20d040,800,200);
    bus.call("comp.window.send_to_workspace",place_args(&sentinel_row,json!({"index":2,"follow":false})));
    pump_patterns(&mut [&mut closing,&mut sentinel]);
    let path=process.root.join("workspace-boundary.vfl");
    let start=bus.call("comp.record.start",json!({"request_id":"workspace-boundary","target":{"output":"headless-0"},"path":path,"fps":30,"cursor":false,"max_seconds":10}));
    // The client is destroyed while the exit image remains eligible for 1s.
    // Switching immediately must invalidate that old workspace's presentation.
    drop(closing);
    bus.call("comp.workspace.switch",json!({"index":2}));
    for _ in 0..100 {
        sentinel.pump();
        thread::sleep(Duration::from_millis(5));
    }
    let stop=bus.call("comp.record.stop",json!({"session":start["session"]}));
    assert_eq!(stop["dropped"],0,"{stop}");
    let mut saw_old=false;
    let mut saw_new=false;
    let mut new_frames=0;
    exported_frames(&path,|frame,width,pixels| {
        let old=pixel(pixels,width,400,280);
        let sentinel=pixel(pixels,width,850,280);
        if !saw_new && old==[255,255,255] { saw_old=true; }
        // The green sentinel is an in-band witness of the new workspace's
        // rendered frame; this avoids guessing when a Bus reply was presented.
        if sentinel==[0x20,0xd0,0x40] { saw_new=true; }
        if saw_new {
            new_frames+=1;
            assert_eq!(old,[18,20,25],"frame {frame}: a closing workspace-one image leaked into workspace two");
        }
    });
    assert!(saw_old,"recording must include the original white window");
    assert!(saw_new && new_frames>=3,"recording must include multiple presented workspace-two frames");
}
