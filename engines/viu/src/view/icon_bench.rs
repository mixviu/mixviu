// SPDX-License-Identifier: MIT OR Apache-2.0
//! Run alone on CBC in release mode; no display, Bus or host icon theme.
use super::*;

#[test]
#[ignore = "release benchmark; mutates XDG environment, run alone"]
fn panel_icon_frames() {
    let root = tempfile::tempdir().unwrap();
    let installed = std::env::var("ICON_BENCH_INSTALLED").as_deref() == Ok("1");
    // A realistic directory fanout, with no dependency on worker packages.
    for theme in ["Fixture", "hicolor", "Adwaita", "AdwaitaLegacy"] {
        for size in [16, 22, 24, 32, 48, 64, 96, 128, 256, 512] {
            for context in ["apps", "actions", "status", "devices", "places", "categories"] {
                std::fs::create_dir_all(root.path().join(format!("icons/{theme}/{size}x{size}/{context}"))).unwrap();
            }
        }
    }
    if installed {
        let dir = root.path().join("icons/Fixture/32x32/status");
        for name in ["dev.mixos", "preferences-system-notifications-symbolic", "audio-volume-high-symbolic", "network-wired-symbolic"] {
            std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/embedded-icon.svg"), dir.join(format!("{name}.svg"))).unwrap();
        }
    }
    // This test is invoked in its own process with exactly one test thread.
    unsafe {
        std::env::set_var("HOME", root.path());
        std::env::set_var("XDG_DATA_HOME", root.path());
        std::env::set_var("XDG_DATA_DIRS", root.path());
        std::env::set_var("XDG_CONFIG_HOME", root.path());
        std::env::set_var("XDG_ICON_THEME", "Fixture");
    }
    let source = include_str!("../../../../share/scenes/panel/scene.mix");
    let content = super::tests::content(source, None, false);
    let mut view = SceneUi::new(content.clone());
    let ctx = egui::Context::default();
    view.setup(&ctx);
    let mut samples = Vec::new();
    for frame in 0..110 {
        let start = std::time::Instant::now();
        view.update(SceneMessage::Replace(content.clone()));
        let mut output = ctx.run_ui(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1920.0, 52.0))),
            ..Default::default()
        }, |ui| view.show(ui, &mut Vec::new()));
        output.textures_delta.clear();
        if frame >= 10 { samples.push(start.elapsed().as_secs_f64() * 1000.0); }
    }
    samples.sort_by(f64::total_cmp);
    println!("ICON_PANEL installed={installed} n={} median_ms={:.4} p95_ms={:.4}", samples.len(), samples[50], samples[95]);
}
