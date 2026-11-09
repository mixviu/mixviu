// SPDX-License-Identifier: MIT OR Apache-2.0
//! Run alone on CBC in release mode; no display, Bus or host icon theme.
use super::*;

#[test]
#[ignore = "release benchmark; mutates XDG environment, run alone"]
fn panel_icon_frames() {
    let root = tempfile::tempdir().unwrap();
    let installed = std::env::var("ICON_BENCH_INSTALLED").as_deref() == Ok("1");
    let inventory = std::env::var("ICON_BENCH_TREE").unwrap_or_else(|_| "small".into());
    // A realistic directory fanout, with no dependency on worker packages.
    for theme in ["Fixture", "hicolor", "Adwaita", "AdwaitaLegacy"] {
        for size in [16, 22, 24, 32, 48, 64, 96, 128, 256, 512] {
            for context in [
                "apps",
                "actions",
                "status",
                "devices",
                "places",
                "categories",
            ] {
                std::fs::create_dir_all(
                    root.path()
                        .join(format!("icons/{theme}/{size}x{size}/{context}")),
                )
                .unwrap();
            }
        }
    }
    if inventory != "small" {
        let dir = root.path().join("icons/Fixture/32x32/apps");
        // Papirus-scale regular-file count, beyond the old work limit. Hard
        // links keep fixture setup small without changing directory entries.
        let seed = dir.join("seed.svg");
        std::fs::write(&seed, "svg").unwrap();
        for i in 0..70_000 {
            std::fs::hard_link(&seed, dir.join(format!("fixture-{i}.svg"))).unwrap();
        }
        if inventory == "overflow" {
            // Also exceed the tracked-directory bound without making themed
            // lookup itself enumerate these unrelated directories.
            for i in 0..4200 {
                std::fs::create_dir_all(root.path().join(format!("icons/overflow/{i}"))).unwrap();
            }
        }
    }
    if installed {
        let dir = root.path().join("icons/Fixture/32x32/status");
        for name in [
            "dev.mixos",
            "preferences-system-notifications-symbolic",
            "audio-volume-high-symbolic",
            "network-wired-symbolic",
        ] {
            std::fs::copy(
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/embedded-icon.svg"
                ),
                dir.join(format!("{name}.svg")),
            )
            .unwrap();
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
    for external in [false, true] {
        let mut content = super::tests::content(source, None, false);
        if external {
            let tree = &mut Arc::get_mut(&mut content).unwrap().tree;
            for (id, source) in [
                ("launcher_icon", "dev.mixos"),
                ("st_notes_i", "preferences-system-notifications-symbolic"),
                ("st_vol_i", "audio-volume-high-symbolic"),
                ("st_net_i", "network-wired-symbolic"),
            ] {
                tree.nodes
                    .get_mut(id)
                    .unwrap()
                    .ports
                    .insert("src".into(), serde_json::json!(source));
                tree.nodes
                    .get_mut(id)
                    .unwrap()
                    .ports
                    .insert("hidden".into(), serde_json::json!(false));
            }
            tree.nodes
                .get_mut("st_vol")
                .unwrap()
                .ports
                .insert("hidden".into(), serde_json::json!(false));
        }
        let mut view = SceneUi::new(content.clone());
        let ctx = egui::Context::default();
        view.setup(&ctx);
        let mut samples = Vec::new();
        for frame in 0..110 {
            let start = std::time::Instant::now();
            view.update(SceneMessage::Replace(content.clone()));
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1920.0, 52.0),
                    )),
                    ..Default::default()
                },
                |ui| view.show(ui, &mut Vec::new()),
            );
            output.textures_delta.clear();
            if frame >= 10 {
                samples.push(start.elapsed().as_secs_f64() * 1000.0);
            }
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "ICON_PANEL tree={inventory} installed={installed} external={external} n={} median_ms={:.4} p95_ms={:.4}",
            samples.len(),
            samples[50],
            samples[95]
        );
    }
}
