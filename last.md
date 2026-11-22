# PR #231 round 2: host wake lock order

PR: https://github.com/mixviu/mixviu/pull/231
Validated code head: cd5320babe5f14ea9109d3c6cd5275724e53dff1.
PR head: the receipt commit containing this file; code is unchanged from the validated head.
Merged main: f33a6596 (#232) and 5779ecb0 (#233), merge c41bbe50. No conflicts; round-1 receipt preserved.
Round 2 adds an explicit editor fake write-rejection assertion at editor_app_test.mix:44-49.
Base prerequisite: #220 merged as e8e9da41729efa4dba3fff1663c50f9f49cd6c79 before work began.

## Round 2 validation — green

Code head: cd5320babe5f14ea9109d3c6cd5275724e53dff1.

- 20261122-102136-605796a9-a1: toolkit, all targets and doc tests, four configurations (default; no-default; default+layer; no-default+layer), each in the normal 8-wide pool; CI-form scoped clippy in each configuration. BUILD_RC=0. Unit counts respectively 399, 381, 434, 416 passed, two ignored each. All four macros reap tests passed; no known-flake panic occurred or was suppressed. 24 binaries per pool.
- 20261122-102218-3b2815f1-a1: freshly built Mix ran the editor read-only write-admission negative control and the full editor script. BUILD_RC=0. Script 852 ms standalone. No new environment variable.
- 20261122-102135-0d36c056-a1: lib-viu, all 56 targets per configuration in the normal 8-wide pool, doc tests and CI-form scoped clippy: BUILD_RC=0. Configurations: default; no-default; default+test-support; no-default+test-support. Unit counts respectively 434, 434, 436, 436 passed, seven ignored each. app_mix passed in all four pools (33.60, 44.79, 36.24, 35.93 s aggregate binary times). All new positive and negative controls passed in all four configurations. The fixed decorations timing test passed in all four.
- 20261122-102137-1f688be5-a1: fmt, env registry, mix-services, lint, native store refusal and engine clippy: BUILD_RC=0. Env registry: 278 names, zero problems. Mix lint: zero regressions. Service checks: 1,062 passed plus scene-loader boot/upgrade. Native store parent test passed in 1.71 s (it runs the isolated child containing the write:true refusal).

Workshop coverage is now real: the standalone test-support binary ran nine tests, all passed in 17.70 s. The serial lane also ran nine tests each with default+test-support (15.02 s) and no-default+test-support (16.03 s). Zero-test selections without the opt-in feature are not counted as workshop coverage.

| Control | Location | Result |
| --- | --- | --- |
| Wake releases registry before Context write | app_host.rs:3689 | Passed in all four lib-viu configurations |
| Restored held-guard loop times out, then joins both threads | app_host.rs:3694 | Passed in all four configurations |
| File-tile helper completes with a writer confirmed queued after a barrier | file_tiles.rs:414 | Passed in all four configurations |
| Restored nested viewport read times out in reaped subprocess | file_tiles.rs:418 | Passed in all four configurations |
| Editor fake rejects write:true before parsing | editor_app_test.mix:44 | Passed with freshly built Mix, full script 852 ms |
| Store rejects write:true with bad desk theme beside read-only forms | store_tests.rs:63 | Passed in isolated native child, parent 1.71 s |

The known toolkit macros reap flake did not occur in any of the four configurations: four passes, no panic text to copy. No retries or saturation experiments. All four CBC jobs returned BUILD_RC=0. The following receipt-only commit does not change tested code.

## Lock acquisition audit

Production registry acquisitions (all):
- libraries/viu/src/app_host.rs:3484 setup: registry -> Context clone (Arc only), no egui lock. icons install before, appearance install after the guard.
- libraries/viu/src/app_host.rs:3589 removed: registry -> retain/Context equality (Arc::ptr_eq only), no egui lock.
- libraries/viu/src/app_host.rs:3629 contexts_wake: registry -> clone Vec -> release guard -> Context repaint (viewport read, then write). Before this PR the loop was at :3490 and held the registry across both acquisitions.
- libraries/viu/src/native_bus.rs:50 passive_frame_stats: separate map registry -> clone -> release -> toolkit frame_stats Context reads. Already correct.
- libraries/viu/src/native_bus.rs:446 stats_surface and :450 remove_stats_surface: map insert/remove only.

Test-only registry acquisitions: app_host.rs:3658 (setup-equivalent push); app_host.rs:3668 (deliberately held-guard negative loop); native_bus.rs:1029 (stats registry length assertion). These are excluded from the production ordering claim.

Opposite order: no production Context -> LayerHost registry path demonstrated. toolkit layer/state.rs:396 calls setup directly, outside a Context closure; :478 calls removed directly from surface retention. app_host Context closure bodies contain no setup/removed calls. app_host.rs:2085 text_input_active reads only stored scene state (view.rs:700). The repaired callback removes the latent order regardless. In the regression app_host.rs:3653 the egui repaint callback holds the Context write lock while another thread performs the setup registry push; fixed wake completes, held-guard control times out at 200 ms and then releases and joins both threads.

Context lock semantics checked against the worker Cargo registry egui/epaint 0.37.0 source: data/memory/options use read; their mut forms use write; input, fonts, graphics and output (including read-only forms) use write. viewport_id reads; request_repaint reads the viewport then writes; request_repaint_of writes; style mutation, viewport commands and callback setters write. run_ui/run_pass release their internal locks before user UI closures. Context clone/equality do not acquire its RwLock. Epaint debug assertions enable the ten-second lock detector; regression watchdogs finish earlier.

Explicit Context closure acquisition sites across lib-viu (all matching entries, includes test controls; unrelated same-name methods are conservative extra entries). None can reach the LayerHost registry except the explicit test callback above. Synchronous nested acquisition in controls is intentional.

```text
libraries/viu/src/child_windows.rs:308:        let dropped = ctx.input(|i| {
libraries/viu/src/child_windows.rs:366:        if ctx.input(|i| i.events.contains(&egui::Event::WindowFocused(true))) {
libraries/viu/src/child_windows.rs:436:        if self.frame.close_requested || ctx.input(|i| i.viewport().close_requested()) {
libraries/viu/src/fonts.rs:31:    if ctx.data(|d| d.get_temp::<(u16, u16)>(key)) == Some((body, heading)) {
libraries/viu/src/fonts.rs:75:    ctx.data_mut(|d| d.insert_temp::<(u16, u16)>(key, (body, heading)));
libraries/viu/src/fonts.rs:79:        .data(|d| d.get_temp::<bool>(egui::Id::unique("viu-gold-fonts")))
libraries/viu/src/fonts.rs:85:        .data(|d| d.get_temp::<(u16, u16)>(egui::Id::unique("viu-font-weights")))
libraries/viu/src/icons.rs:28:    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("viu-icon-style"), (stroke, mask)));
libraries/viu/src/icons.rs:34:        ctx.data(|d| d.get_temp::<(f32, [u8; 4])>(egui::Id::unique("viu-icon-style")))
libraries/viu/src/terminal/view.rs:178:                    ctx.memory(|m| m.focused())
libraries/viu/src/terminal/view.rs:189:                let field = ctx.memory(|m| m.focused());
libraries/viu/src/terminal/view.rs:243:                        ctx.input(|i| i
libraries/viu/src/terminal/view.rs:257:                    assert_eq!(ctx.memory(|m| m.focused()), field);
libraries/viu/src/terminal/view.rs:1135:        let raw = ui.input(|i| i.events.clone());
libraries/viu/src/terminal/view.rs:1136:        let accepting = ui.is_enabled() && ui.input(|i| i.focused);
libraries/viu/src/terminal/view.rs:1180:                            ui.memory_mut(|m| m.surrender_focus(placed.id));
libraries/viu/src/terminal/view.rs:1201:                    .find(|p| ui.memory(|m| m.has_focus(p.id)))
libraries/viu/src/terminal/view.rs:1232:                    let focused = ui.memory(|m| m.has_focus(placed.id));
libraries/viu/src/terminal/view.rs:1233:                    ui.memory_mut(|m| m.request_focus(placed.id));
libraries/viu/src/terminal/view.rs:1385:        let surrender = ui.memory_mut(|m| {
libraries/viu/src/terminal/view.rs:1391:        ui.memory_mut(|m| m.options.input_options.surrender_focus_on = surrender);
libraries/viu/src/terminal/view.rs:1442:        ui.memory_mut(|m| {
libraries/viu/src/terminal/view.rs:1477:            response.has_focus() && ui.input(|i| i.focused),
libraries/viu/src/terminal/view.rs:1500:            ui.ctx().output_mut(|o| {
libraries/viu/src/terminal/primary_font_tests.rs:31:        .fonts()
libraries/viu/src/terminal/primary_font_tests.rs:205:        .output()
libraries/viu/src/view_motion_identity_tests.rs:25:            let focused = ctx.memory(|m| m.focused()).expect("field focus");
libraries/viu/src/terminal/selection_worker.rs:532:                .and_then(|d| d.data().selection_offer())
libraries/viu/src/terminal/selection_worker.rs:542:                .and_then(|d| d.data().selection_offer())
libraries/viu/src/terminal/admission.rs:137:        let supervisor = Supervisor::with_options(broker.options(), broker.url.clone()).unwrap();
libraries/viu/src/terminal/admission.rs:150:            let UnixConnectOutcome::VerifiedUnix(caller) = NodedClient::connect_unix("", &broker.url, &broker.options(), None).await.unwrap() else {panic!("verified observer required")};
libraries/viu/src/widgets.rs:1167:                .data_mut(|d| d.insert_temp(focus_key(), id.to_owned()));
libraries/viu/src/widgets.rs:1763:                        && ui.input(|i| {
libraries/viu/src/widgets.rs:1781:                    && ui.input(|i| i.key_pressed(egui::Key::Escape))
libraries/viu/src/widgets.rs:1789:                } else if (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
libraries/viu/src/widgets.rs:1792:                        && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command))
libraries/viu/src/time_series.rs:338:    ctx.data_mut(|data| {
libraries/viu/src/time_series.rs:667:        .data_mut(|data| {
libraries/viu/src/time_series.rs:734:            && let Some(pointer) = plot_ui.ctx().input(|input| input.pointer.hover_pos())
libraries/viu/src/time_series.rs:738:            let factor = plot_ui.ctx().input_mut(|input| {
libraries/viu/src/time_series.rs:774:    ui.ctx().data_mut(|data| {
libraries/viu/src/terminal/sessions.rs:901:                .output()
libraries/viu/src/header_controls.rs:205:                                if suggestions.is_some() && ui.memory(|m| m.has_focus(field_id)) {
libraries/viu/src/header_controls.rs:206:                                    ui.input_mut(|i| {
libraries/viu/src/header_controls.rs:332:                                    && ui.ctx().data(|d| d.get_temp::<Value>(marker)).as_ref()
libraries/viu/src/header_controls.rs:336:                                    ui.ctx().data_mut(|d| {
libraries/viu/src/header_controls.rs:341:                                    ui.ctx().data_mut(|d| {
libraries/viu/src/header_controls.rs:467:        let chooser = ui.ctx().data_mut(|d| {
libraries/viu/src/header_controls.rs:495:            ui.ctx().data_mut(|d| {
libraries/viu/src/header_controls.rs:501:            .data_mut(|d| d.remove::<Arc<Mutex<toolkit::file_chooser::FileChooser>>>(chooser_id));
libraries/viu/src/input.rs:409:        .is_some_and(|chord| ctx.input_mut(|i| i.consume_key(chord.modifiers, chord.logical_key)))
libraries/viu/src/terminal/session_fd.rs:233:                .output()
libraries/viu/src/terminal/session_fd.rs:365:                .output()
libraries/viu/src/rich.rs:30:    ctx.data(|data| data.get_temp(key)).unwrap_or_default()
libraries/viu/src/rich.rs:36:    ctx.data_mut(|data| {
libraries/viu/src/rich.rs:47:    ctx.data_mut(|d| {
libraries/viu/src/rich.rs:55:    ctx.data_mut(|d| {
libraries/viu/src/rich.rs:118:    ctx.data_mut(|data| {
libraries/viu/src/rich.rs:578:                if (response.hovered() && ui.input(|i| i.pointer.primary_pressed()))
libraries/viu/src/rich.rs:615:            ui.ctx().data(|d| d.get_temp::<Value>(marker)).as_ref() != Some(&value["instance"]);
libraries/viu/src/rich.rs:652:                .data_mut(|d| d.insert_temp(marker, value["instance"].clone()));
libraries/viu/src/rich.rs:1017:                                ui.ctx().data_mut(|d| d.insert_temp(anchor, response.rect));
libraries/viu/src/rich.rs:1018:                                if ui.ctx().data(|d| d.get_temp::<bool>(busy)).unwrap_or(false) {
libraries/viu/src/rich.rs:1173:                            response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
libraries/viu/src/rich.rs:1174:                        if !response.has_focus() && ui.memory(|m| m.focused().is_none()) {
libraries/viu/src/rich.rs:1242:                        if ui.ctx().data(|d|d.get_temp::<Value>(marker)).as_ref()!=Some(&value["instance"])
libraries/viu/src/rich.rs:1243:                            || (!response.has_focus() && ui.memory(|m|m.focused().is_none())) {
libraries/viu/src/rich.rs:1245:                            ui.ctx().data_mut(|d| d.insert_temp(marker,value["instance"].clone()));
libraries/viu/src/rich.rs:1299:            } else if ui.input(|i| {
libraries/viu/src/rich.rs:1353:            if response.lost_focus() && ui.input(|i|i.key_pressed(egui::Key::Enter)){accept=true;}
libraries/viu/src/rich.rs:1354:            else if !response.has_focus() && ui.memory(|m| m.focused().is_none()){response.request_focus();}
libraries/viu/src/media_gallery.rs:115:    ui.ctx().data_mut(|d| {
libraries/viu/src/image_view.rs:203:        let mut points = ui.data_mut(|d| d.get_temp::<Vec<Point>>(gesture_id).unwrap_or_default());
libraries/viu/src/image_view.rs:204:        let panning = ui.input(|i| {
libraries/viu/src/image_view.rs:215:        if ui.data_mut(|d| d.get_temp::<f32>(scale_id)) != Some(view.scale) {
libraries/viu/src/image_view.rs:216:            ui.data_mut(|d| d.insert_temp(scale_id, view.scale));
libraries/viu/src/image_view.rs:220:            ui.data_mut(|d| d.remove::<Vec<Point>>(gesture_id));
libraries/viu/src/image_view.rs:232:            let zoom = ui.input(|i| i.zoom_delta());
libraries/viu/src/image_view.rs:236:            let wheel = ui.input(|i| i.smooth_scroll_delta);
libraries/viu/src/image_view.rs:242:            let delta = ui.input(|i| i.pointer.delta());
libraries/viu/src/image_view.rs:251:                    .input(|i| i.pointer.press_origin())
libraries/viu/src/image_view.rs:318:        ui.data_mut(|d| d.insert_temp(gesture_id, points));
libraries/viu/src/image_view.rs:380:                    let limit = ui.input(|i| i.max_texture_side);
libraries/viu/src/raster_fill.rs:137:                        let maximum = ui.input(|i| i.max_texture_side);
libraries/viu/src/appearance.rs:12:    ctx.data(|d| d.get_temp::<toolkit::Theme>(egui::Id::unique("viu-session-theme")))
libraries/viu/src/appearance.rs:36:    ctx.data(|d| {
libraries/viu/src/appearance.rs:59:    ctx.data_mut(|d| {
libraries/viu/src/appearance.rs:127:    ctx.data_mut(|d| {
libraries/viu/src/appearance.rs:140:        ctx.style_mut_of(theme, |style| style.debug.show_unaligned = false);
libraries/viu/src/appearance.rs:142:    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("viu-session-theme"), theme.clone()));
libraries/viu/src/appearance.rs:156:    ctx.data_mut(|d| {
libraries/viu/src/appearance.rs:195:    if ctx.data(|d| d.get_temp::<Value>(key)).as_ref() == Some(value) {
libraries/viu/src/appearance.rs:210:        .data(|d| d.get_temp::<toolkit::Theme>(egui::Id::unique("viu-session-theme")))
libraries/viu/src/appearance.rs:227:    ctx.data_mut(|d| {
libraries/viu/src/appearance.rs:278:    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("viu-gold-fonts"), false));
libraries/viu/src/appearance.rs:287:    ctx.data_mut(|d| {
libraries/viu/src/debug.rs:204:                .env(CHILD,"1").env_remove("RUST_LOG").output().unwrap();
libraries/viu/src/window_frame.rs:30:    let changed = ctx.data_mut(|data| {
libraries/viu/src/window_frame.rs:124:    ctx.data(|d| d.get_temp::<bool>(egui::Id::unique("viu-window-client")))
libraries/viu/src/window_frame.rs:131:            ctx.data_mut(|d| d.insert_temp(egui::Id::unique(("eframe.window-menu", vp)), position));
libraries/viu/src/window_frame.rs:162:                        ui.ctx().data_mut(|d| {
libraries/viu/src/window_frame.rs:187:            assert_eq!(ctx.data(|d| d.get_temp::<egui::Pos2>(id)), Some(position));
libraries/viu/src/window_frame.rs:201:            ctx.data_mut(|d| {
libraries/viu/src/window_frame.rs:226:        ctx.data(|d| d.get_temp::<Self>(egui::Id::unique("viu-window-metrics")))
libraries/viu/src/window_frame.rs:302:            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-caption-alignment"), caption_align));
libraries/viu/src/window_frame.rs:321:            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-window-client"), self.client));
libraries/viu/src/window_frame.rs:376:            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-caption-alignment"), alignment));
libraries/viu/src/window_frame.rs:418:        let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
libraries/viu/src/window_frame.rs:469:            || ui.input(|i| {
libraries/viu/src/window_frame.rs:470:                i.viewport().maximized.unwrap_or(false) || i.viewport().fullscreen.unwrap_or(false)
libraries/viu/src/window_frame.rs:560:    let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
libraries/viu/src/window_frame.rs:675:            !ctx.input(|i| i.viewport().maximized.unwrap_or(false)),
libraries/viu/src/debug_recent.rs:256:                    .output()
libraries/viu/src/shortcuts.rs:98:    ctx.input_mut(|input| {
libraries/viu/src/shortcuts.rs:136:    ctx.input_mut(|input| {
libraries/viu/src/native_menu.rs:71:    ctx.data(|data| data.get_temp::<Model>(key))?
libraries/viu/src/native_menu.rs:78:    let index = ctx.data_mut(|d| {
libraries/viu/src/native_menu.rs:164:    if let Some(old) = ui.ctx().data(|d| d.get_temp::<Model>(key)) {
libraries/viu/src/native_menu.rs:175:        .data_mut(|d| d.insert_temp(key, Model { ids, menu_len }));
libraries/viu/src/native_menu.rs:214:                if (response.hovered() && ui.input(|i| i.pointer.primary_pressed()))
libraries/viu/src/native_menu.rs:228:                .data_mut(|d| d.insert_temp(key.with("picked"), serde_json::json!({"command":command,"origin":if pick.1 {"palette"}else{"menu"}})));
libraries/viu/src/native_menu.rs:235:    ctx.data_mut(|d| d.remove_temp::<Value>(key))
libraries/viu/src/chooser_host.rs:270:        if ctx.input(|i| i.viewport().close_requested()) && !self.closed {
libraries/viu/src/chooser_host.rs:301:                let v = ctx.input(|i| i.viewport().clone());
libraries/viu/src/chooser_host.rs:390:            let keys:Vec<_>=ctx.input(|input|input.events.iter().filter_map(|event|{
libraries/viu/src/annotation/paint.rs:63:            .data_mut()
libraries/viu/src/annotation/paint.rs:330:    let data = pixmap.data_mut();
libraries/viu/src/terminal/session_actor_tests.rs:205:                options: broker.options(),
libraries/viu/src/documents.rs:647:                .data_mut(|d| d.insert_temp(egui::Id::unique("viu-editor-focused"), true));
libraries/viu/src/editor.rs:150:        if ui.memory(|m| m.has_focus(editor_id))
libraries/viu/src/editor.rs:151:            && ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::F))
libraries/viu/src/editor.rs:261:        .data(|d| d.get_temp::<editor::Palette>(egui::Id::unique("viu-gold-editor-palette")))
libraries/viu/src/terminal/session_actor.rs:1661:        let supervisor = Supervisor::with_options(broker.options(), broker.url.clone()).unwrap();
libraries/viu/src/terminal/session_actor.rs:1731:        let mut options = broker.options();
libraries/viu/src/terminal/session_actor.rs:1744:            options: broker.options(),
libraries/viu/src/terminal/session_actor.rs:1798:        let supervisor = Supervisor::with_options(broker.options(), broker.url.clone()).unwrap();
libraries/viu/src/file_tiles.rs:98:        .data(|d| {
libraries/viu/src/file_tiles.rs:313:            writer.data_mut(|_| {});
libraries/viu/src/file_tiles.rs:333:            ui.ctx().data_mut(|d| {
libraries/viu/src/file_tiles.rs:341:                ui.ctx().data(|d| {
libraries/viu/src/file_tiles.rs:440:                ui.ctx().data_mut(|d| {
libraries/viu/src/app_host/osd.rs:209:        let presses = ui.input(|i| {
libraries/viu/src/app_host.rs:776:            && ctx.input(|i| i.viewport().close_requested())
libraries/viu/src/app_host.rs:2045:                let time = ui.input(|i| i.time);
libraries/viu/src/app_host.rs:2085:                let keys=ui.input(|i|i.events.iter().filter_map(|e|match e{egui::Event::Key{key,pressed:true,repeat:false,modifiers,..}=>Some(json!({"key":crate::input::key_name(*key),"ctrl":modifiers.ctrl||modifiers.command,"alt":modifiers.alt,"shift":modifiers.shift,"typing":self.view.text_input_active()})),_=>None}).collect::<Vec<_>>());
libraries/viu/src/app_host.rs:2143:                            .data(|d| d.get_temp::<bool>(egui::Id::unique("viu-editor-focused")))
libraries/viu/src/app_host.rs:2147:                            .data(|d| d.get_temp::<String>(egui::Id::unique("viu-focused-widget")));
libraries/viu/src/app_host.rs:2154:                            || ui.ctx().memory(|m| m.focused().is_some()))
libraries/viu/src/app_host.rs:2981:            .viewport(viewport);
libraries/viu/src/app_host.rs:3449:            ctx.data(|d| d.get_temp::<String>(egui::Id::unique("viu-layer-output"))),
libraries/viu/src/app_host.rs:3498:            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-layer-output"), output.clone()));
libraries/viu/src/app_host.rs:3529:            if let Some(pos) = ui.input(|i| {
libraries/viu/src/app_host.rs:3549:                if ui.ctx().data(|d| d.get_temp::<Value>(key)).as_ref() != Some(&stamp) {
libraries/viu/src/app_host.rs:3558:                        ui.ctx().data_mut(|d| d.insert_temp(key, stamp));
libraries/viu/src/backdrop.rs:74:    ctx.data_mut(|d| {
libraries/viu/src/backdrop.rs:107:            ctx.data(|d| {
libraries/viu/src/capture.rs:58:        let pixels = ctx.input(|i| i.viewport().inner_rect.map(|r| r.area() * scale.powi(2)));
libraries/viu/src/tree_table.rs:22:    let snapshot = ui.data_mut(|data| {
libraries/viu/src/tree_table.rs:31:    ui.fonts(|fonts| {
libraries/viu/src/tree_table.rs:137:    ui.data_mut(|data| {
libraries/viu/src/tree_table.rs:474:                ctx.style_mut_of(egui::Theme::Dark, |style| {
libraries/viu/src/tree_table.rs:925:                json!({"source":payload.source,"keys":payload.keys,"destination":destination,"ctrl":ui.input(|i|i.modifiers.ctrl||i.modifiers.command),"shift":ui.input(|i|i.modifiers.shift)}),
libraries/viu/src/tree_table.rs:1156:                let modifiers=ui.input(|i|i.modifiers);
libraries/viu/src/tree_table.rs:1444:            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
libraries/viu/src/tree_table.rs:1491:        let changed = ui.ctx().data_mut(|data| {
libraries/viu/src/tree_table.rs:1621:                let changed = ui.ctx().data_mut(|d| {
libraries/viu/src/tree_table.rs:1665:        if ui.input(|i| i.pointer.primary_pressed()) && ui.rect_contains_pointer(area) {
libraries/viu/src/tree_table.rs:1668:        if ui.memory(|m| m.has_focus(focus_id)) {
libraries/viu/src/tree_table.rs:1669:            ui.memory_mut(|m| {
libraries/viu/src/tree_table.rs:1686:                if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
libraries/viu/src/tree_table.rs:1690:            if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
libraries/viu/src/tree_table.rs:1735:                                            ui.memory_mut(|m| m.request_focus(focus_id));
libraries/viu/src/tree_table.rs:1767:                                        ui.memory_mut(|m| m.request_focus(focus_id));
libraries/viu/src/tree_table.rs:1781:                    if focused(value, row) && (!keyboard || ui.memory(|m| m.has_focus(focus_id))) {
libraries/viu/src/numbers.rs:25:        let now = ui.input(|i| i.time);
libraries/viu/src/view.rs:500:            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-editor-focused"), false));
libraries/viu/src/view.rs:544:                .data_mut(|d| d.remove::<String>(crate::widgets::focus_key()));
libraries/viu/src/view.rs:579:                .data(|d| d.get_temp::<String>(crate::widgets::focus_key()));
libraries/viu/src/view.rs:907:                        let fill=if node.ports.get("surface").and_then(Value::as_str)==Some("editor") {ui.ctx().data(|d|d.get_temp::<editor::Palette>(egui::Id::unique("viu-gold-editor-palette"))).map(|p|p.background).unwrap_or(ui.visuals().panel_fill)} else if content.light {toolkit::chrome::Chrome::for_theme(toolkit::content::light_theme()).palette.canvas}else{ui.visuals().panel_fill};
libraries/viu/src/view.rs:914:                                    if ui.memory(|m|m.has_focus(egui::Id::unique(target))) {
libraries/viu/src/view.rs:917:                                        let step=ui.input_mut(|i|{
libraries/viu/src/view.rs:925:                                            if let Some(id)=ui.ctx().data(|d|d.get_temp::<egui::Id>(marker)) {
libraries/viu/src/view.rs:932:                                ui.ctx().data_mut(|d|d.insert_temp(marker,shown.id));
libraries/viu/src/view.rs:1034:                        let groups=band_groups(node,crate::window_frame::Metrics::of(ui.ctx()).caption_width*3.0,ui.ctx().data(|d| d.get_temp::<design::title_band::Alignment>(egui::Id::unique("viu-caption-alignment"))).unwrap_or_else(|| toolkit::chrome::Chrome::of(ui.ctx()).captions.alignment(design::title_band::Alignment::Right)),Some(ui.ctx()),&measure).expect("validated band groups");
libraries/viu/src/view.rs:1251:                                                    ui.ctx().data(|d|d.get_temp::<f32>(row_id.with(("fixed-width",child)))).unwrap_or(0.)
libraries/viu/src/view.rs:1261:                                            let height = ui.ctx().data(|d|d.get_temp::<f32>(cell)).unwrap_or(0.);
libraries/viu/src/view.rs:1266:                                                let previous = ui.ctx().data(|d|d.get_temp::<f32>(width_key)).unwrap_or(0.);
libraries/viu/src/view.rs:1268:                                                    ui.ctx().data_mut(|d|d.insert_temp(width_key,measured));
libraries/viu/src/view.rs:1285:                                                ui.ctx().data_mut(|d|d.insert_temp(cell,actual));
libraries/viu/src/view.rs:1449:            let previous_model = ui.ctx().data_mut(|d| {
libraries/viu/src/view.rs:1459:                        ui.memory_mut(|m| m.request_focus(egui::Id::unique(target)));
libraries/viu/src/view.rs:1470:            let previous = ui.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab));
libraries/viu/src/view.rs:1472:                !previous && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab));
libraries/viu/src/view.rs:1480:                        ui.memory_mut(|m| m.request_focus(egui::Id::unique(target)));
libraries/viu/src/view.rs:1489:                            ui.memory(|m| m.has_focus(egui::Id::unique(target)))
libraries/viu/src/view.rs:1763:            if ui.input(|i| i.pointer.primary_pressed()) && ui.rect_contains_pointer(area) {
libraries/viu/src/view.rs:1766:            if ui.memory(|m| m.has_focus(id)) {
libraries/viu/src/view.rs:1767:                ui.memory_mut(|m| {
libraries/viu/src/view.rs:1926:                ui.ctx().data_mut(|d| d.insert_temp(marker, limit));
libraries/viu/src/view.rs:1951:                ui.ctx().data_mut(|d| {
libraries/viu/src/view.rs:1962:                let height = ui.ctx().data(|d| d.get_temp::<f32>(marker)).unwrap_or(0.0);
libraries/viu/src/view.rs:1974:                    ui.ctx().data_mut(|d| d.insert_temp(marker, measured));
libraries/viu/src/design_validation.rs:282:    ctx.data_mut(|data| {
libraries/viu/src/design_validation.rs:293:        .data_mut(|data| {
libraries/viu/src/editor_controls.rs:253:            ui.ctx().data_mut(|d| d.remove::<String>(key));
libraries/viu/src/editor_controls.rs:256:        let previous = ui.ctx().data(|d| d.get_temp::<String>(key));
libraries/viu/src/editor_controls.rs:279:        ui.ctx().data_mut(|d| d.insert_temp(key, query));
libraries/viu/src/editor_controls.rs:474:                            && ui.ctx().data(|d| d.get_temp::<Value>(marker)).as_ref()
libraries/viu/src/editor_controls.rs:479:                                .data_mut(|d| d.insert_temp(marker, row["focus_instance"].clone()));
libraries/viu/src/editor_controls.rs:482:                            ui.ctx().data_mut(|d| {
libraries/viu/src/editor_controls.rs:542:                            .data(|d| {
libraries/viu/src/editor_controls.rs:566:                let submit_event = ui.input(|i| {
libraries/viu/src/editor_controls.rs:584:                    && ui.ctx().data(|d| d.get_temp::<Value>(marker)).as_ref()
libraries/viu/src/editor_controls.rs:589:                        .data_mut(|d| d.insert_temp(marker, value["instance"].clone()));
libraries/viu/src/editor_controls.rs:598:                        ui.input_mut(|input| {
libraries/viu/src/editor_controls.rs:611:                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
libraries/viu/src/editor_controls.rs:624:                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
libraries/viu/src/settings_browser.rs:342:    ctx.data_mut(|data| {
libraries/viu/src/settings_browser.rs:369:    ctx.data_mut(|data| {
libraries/viu/src/settings_browser.rs:586:        .data(|d| d.get_temp::<String>(key))
libraries/viu/src/settings_browser.rs:590:        response.ctx.data_mut(|d| {
libraries/viu/src/settings_browser.rs:605:    ui.ctx().data_mut(|d| {
libraries/viu/src/settings_browser.rs:681:                    if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
libraries/viu/src/settings_browser.rs:684:                        ui.memory_mut(|m| m.surrender_focus(r.id));
libraries/viu/src/settings_browser.rs:1126:        .data_mut(|d| d.remove_temp::<bool>(focus_key.with("done")))
libraries/viu/src/settings_browser.rs:2312:                        .data(|d| d.get_temp::<String>(filter_id))
libraries/viu/src/settings_browser.rs:2322:                        ui.data_mut(|d| d.insert_temp(filter_id, query.clone()));
libraries/viu/src/settings_browser.rs:2666:                                            ui.ctx().data_mut(|d| {
libraries/viu/src/settings_browser.rs:2675:                                                .data_mut(|d| d.get_temp::<String>(field_id))
libraries/viu/src/settings_browser.rs:2709:                                                && ui.input(|i| i.key_pressed(egui::Key::Enter)));
libraries/viu/src/settings_browser.rs:2712:                                                ui.ctx().data_mut(|d| {
libraries/viu/src/settings_browser.rs:2803:        .data(|d| d.get_temp::<String>(filter_id))
libraries/viu/src/settings_browser.rs:2812:        ui.data_mut(|d| d.insert_temp(filter_id, query.clone()));
libraries/viu/src/settings_browser.rs:2815:    let mut extended = ui.data(|d| d.get_temp::<bool>(more_id)).unwrap_or(false);
libraries/viu/src/settings_browser.rs:2832:                                    ui.data_mut(|d| d.insert_temp(more_id, extended));
libraries/viu/src/settings_browser.rs:2944:    let mut value = ui.data(|d| d.get_temp::<f64>(key)).unwrap_or(current);
libraries/viu/src/settings_browser.rs:2950:                let escaped = ui.input(|i| i.key_pressed(egui::Key::Escape));
libraries/viu/src/settings_browser.rs:2964:                    ui.data_mut(|d| d.remove::<f64>(key));
libraries/viu/src/settings_browser.rs:2967:                    ui.data_mut(|d| d.insert_temp(key, value));
libraries/viu/src/settings_browser.rs:2977:                        ui.data_mut(|d| d.insert_temp(key, value));
libraries/viu/src/settings_browser.rs:2988:    ui.data_mut(|d| d.remove::<f64>(key));
libraries/viu/src/settings_browser.rs:3010:                .data_mut(|d| d.get_temp::<String>(query_id))
libraries/viu/src/settings_browser.rs:3018:                    .data_mut(|d| d.insert_temp(query_id, query.clone()));
libraries/viu/src/settings_browser.rs:3182:    let held: Option<f64> = ui.data(|d| d.get_temp(id));
libraries/viu/src/settings_browser.rs:3248:        ui.data_mut(|d| d.insert_temp(id, at(pos)));
libraries/viu/src/settings_browser.rs:3253:        let delta = ui.input(|i| {
libraries/viu/src/settings_browser.rs:3267:        out = ui.data_mut(|d| d.remove_temp::<f64>(id));
libraries/viu/src/settings_browser.rs:3383:        .data(|d| d.get_temp::<Color32>(held_id))
libraries/viu/src/settings_browser.rs:3395:        ui.data_mut(|d| d.insert_temp(held_id, picking));
libraries/viu/src/settings_browser.rs:3397:    if ui.input(|i| i.pointer.any_released())
libraries/viu/src/settings_browser.rs:3398:        && let Some(held) = ui.data(|d| d.get_temp::<Color32>(held_id))
libraries/viu/src/settings_browser.rs:3401:        ui.data_mut(|d| d.remove_temp::<Color32>(held_id));
libraries/viu/src/settings_browser.rs:3412:        .data(|d| d.get_temp::<String>(hex_id))
libraries/viu/src/settings_browser.rs:3426:            ui.data_mut(|d| d.insert_temp(hex_id, value.clone()));
libraries/viu/src/settings_browser.rs:3428:        if edit.lost_focus() && ui.data(|d| d.get_temp::<String>(hex_id)).is_some() {
libraries/viu/src/settings_browser.rs:3429:            ui.data_mut(|d| d.remove_temp::<String>(hex_id));
libraries/viu/src/settings_browser.rs:3449:        ui.data_mut(|d| {
libraries/viu/src/settings_browser.rs:3464:        .data(|d| d.get_temp::<Color32>(held_id))
libraries/viu/src/settings_browser.rs:3471:        ui.data_mut(|d| d.insert_temp(held_id, picking));
libraries/viu/src/settings_browser.rs:3475:    if ui.data(|d| d.get_temp::<Color32>(held_id))
libraries/viu/src/settings_browser.rs:3476:        != ui.data(|d| d.get_temp::<Color32>(hex_id.with("at")))
libraries/viu/src/settings_browser.rs:3478:        ui.data_mut(|d| {
libraries/viu/src/settings_browser.rs:3484:        .data(|d| d.get_temp::<String>(hex_id))
libraries/viu/src/settings_browser.rs:3498:            ui.data_mut(|d| {
libraries/viu/src/settings_browser.rs:3508:        ui.data_mut(|d| {
libraries/viu/src/network_map.rs:96:    let cached = ui.ctx().data_mut(|d| d.get_temp::<Arc<Geometry>>(id));
libraries/viu/src/network_map.rs:101:        ui.ctx().data_mut(|d| d.insert_temp(id, geometry.clone()));
libraries/viu/src/network_map.rs:499:    if !activated && ui.rect_contains_pointer(rect) && ui.input(|i| i.pointer.primary_clicked()) {
libraries/viu/src/network_map.rs:500:        if let Some(pointer) = ui.input(|i| i.pointer.interact_pos()).filter(|pointer| {
libraries/viu/src/network_map.rs:524:    let frame = ui.input(|i| i.time);
libraries/viu/src/network_map.rs:525:    let (shapes, moving, next) = ui.ctx().data_mut(|d| {
libraries/viu/src/saved_state.rs:167:                ctx.input(|i| i.pointer.any_down()),
libraries/viu/src/native_bus.rs:905:                            NodedClient::connect_unix(base, &broker.url, &broker.options(), None)
libraries/viu/src/native_bus.rs:940:                    ctx.data_mut(|data| {
libraries/viu/src/native_bus.rs:974:                        NodedClient::connect_unix("", &broker.url, &broker.options(), None)
libraries/viu/src/native_bus.rs:1166:                NodedClient::connect_unix("", &broker.url, &broker.options(), None)

```

Public Context convenience methods, repaint/style/viewport calls, including tests and examples (conservative inventory using every public method in egui context.rs; includes same-name non-egui calls):

```text
libraries/viu/src/child_windows.rs:127:            ctx.send_viewport_cmd_to(child.viewport, egui::ViewportCommand::Close);
libraries/viu/src/child_windows.rs:182:                            Arc::new(move || wake.request_repaint()),
libraries/viu/src/child_windows.rs:221:                ctx.request_repaint_of(child.viewport);
libraries/viu/src/child_windows.rs:262:            ctx.show_viewport_deferred(viewport, builder, move |ui, _| {
libraries/viu/src/child_windows.rs:291:        ctx.request_repaint_of(ViewportId::ROOT);
libraries/viu/src/child_windows.rs:308:        let dropped = ctx.input(|i| {
libraries/viu/src/child_windows.rs:330:                ctx.request_repaint();
libraries/viu/src/child_windows.rs:366:        if ctx.input(|i| i.events.contains(&egui::Event::WindowFocused(true))) {
libraries/viu/src/child_windows.rs:436:        if self.frame.close_requested || ctx.input(|i| i.viewport().close_requested()) {
libraries/viu/src/child_windows.rs:437:            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
libraries/viu/src/child_windows.rs:445:            ctx.request_repaint();
libraries/viu/src/child_windows.rs:465:            ctx.request_repaint();
libraries/viu/src/child_windows.rs:515:            ctx.run_ui(
libraries/viu/src/fonts.rs:31:    if ctx.data(|d| d.get_temp::<(u16, u16)>(key)) == Some((body, heading)) {
libraries/viu/src/fonts.rs:74:    ctx.set_fonts(fonts);
libraries/viu/src/fonts.rs:75:    ctx.data_mut(|d| d.insert_temp::<(u16, u16)>(key, (body, heading)));
libraries/viu/src/fonts.rs:79:        .data(|d| d.get_temp::<bool>(egui::Id::unique("viu-gold-fonts")))
libraries/viu/src/fonts.rs:85:        .data(|d| d.get_temp::<(u16, u16)>(egui::Id::unique("viu-font-weights")))
libraries/viu/src/icons.rs:28:    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("viu-icon-style"), (stroke, mask)));
libraries/viu/src/icons.rs:34:        ctx.data(|d| d.get_temp::<(f32, [u8; 4])>(egui::Id::unique("viu-icon-style")))
libraries/viu/src/icons.rs:88:        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/bulk_profile.rs:91:            ctx.run_ui(
libraries/viu/tests/dopus_contract.rs:121:    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/ced_editor.rs:89:    ctx.run_ui(
libraries/viu/tests/backdrop.rs:322:                NodedClient::connect_unix("", &broker.url, &broker.options(), None)
libraries/viu/tests/prefs_parity.rs:35:    ctx.run_ui(
libraries/viu/tests/prefs_parity.rs:107:        ctx.run_ui(
libraries/viu/tests/gold_frame.rs:36:        ctx.run_ui(
libraries/viu/tests/gold_frame.rs:65:        ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/gold_frame.rs:117:        ctx.run_ui(
libraries/viu/tests/toolbar.rs:17:    ctx.run_ui(
libraries/viu/tests/toolbar.rs:30:    let anchor = ctx.data(|d| {
libraries/viu/tests/toolbar.rs:55:            ctx.run_ui(
libraries/viu/tests/toolbar.rs:100:            ctx.run_ui(
libraries/viu/tests/text_input.rs:7:    let mut output = ctx.run_ui(
libraries/viu/tests/text_input.rs:56:        let field = ctx.memory(|m| m.focused()).expect("initial field focus");
libraries/viu/tests/text_input.rs:61:        let button = ctx.memory(|m| m.focused()).expect("Cancel focus");
libraries/viu/tests/text_input.rs:178:        ctx.run_ui(RawInput::default(), |ui| {
libraries/viu/tests/text_input.rs:347:    let field = ctx.memory(|m| m.focused()).expect("field focus");
libraries/viu/tests/text_input.rs:352:    let button = ctx.memory(|m| m.focused()).expect("button focus");
libraries/viu/src/terminal/view.rs:144:                    ctx.run_ui(
libraries/viu/src/terminal/view.rs:178:                    ctx.memory(|m| m.focused())
libraries/viu/src/terminal/view.rs:189:                let field = ctx.memory(|m| m.focused());
libraries/viu/src/terminal/view.rs:243:                        ctx.input(|i| i
libraries/viu/src/terminal/view.rs:257:                    assert_eq!(ctx.memory(|m| m.focused()), field);
libraries/viu/src/terminal/view.rs:294:            ctx.run_ui(
libraries/viu/src/terminal/view.rs:337:        ctx.run_ui(Default::default(), |ui| {
libraries/viu/src/terminal/view.rs:380:            ctx.run_ui(
libraries/viu/src/terminal/view.rs:434:            ctx.run_ui(
libraries/viu/src/terminal/view.rs:483:            ctx.run_ui(
libraries/viu/src/terminal/view.rs:558:            ctx.run_ui(
libraries/viu/src/terminal/view.rs:648:            let mut output = ctx.run_ui(
libraries/viu/src/terminal/view.rs:740:            ctx.run_ui(
libraries/viu/src/terminal/view.rs:803:            ctx.run_ui(
libraries/viu/src/terminal/view.rs:831:        ctx.run_ui(
libraries/viu/src/terminal/view.rs:1080:        let scale = ui.ctx().pixels_per_point();
libraries/viu/src/terminal/view.rs:1135:        let raw = ui.input(|i| i.events.clone());
libraries/viu/src/terminal/view.rs:1136:        let accepting = ui.is_enabled() && ui.input(|i| i.focused);
libraries/viu/src/terminal/view.rs:1161:                            .layer_id_at(pos)
libraries/viu/src/terminal/view.rs:1180:                            ui.memory_mut(|m| m.surrender_focus(placed.id));
libraries/viu/src/terminal/view.rs:1201:                    .find(|p| ui.memory(|m| m.has_focus(p.id)))
libraries/viu/src/terminal/view.rs:1207:                ui.ctx().request_repaint();
libraries/viu/src/terminal/view.rs:1215:            ui.ctx().request_repaint();
libraries/viu/src/terminal/view.rs:1232:                    let focused = ui.memory(|m| m.has_focus(placed.id));
libraries/viu/src/terminal/view.rs:1233:                    ui.memory_mut(|m| m.request_focus(placed.id));
libraries/viu/src/terminal/view.rs:1317:        let scale = ui.ctx().pixels_per_point();
libraries/viu/src/terminal/view.rs:1385:        let surrender = ui.memory_mut(|m| {
libraries/viu/src/terminal/view.rs:1391:        ui.memory_mut(|m| m.options.input_options.surrender_focus_on = surrender);
libraries/viu/src/terminal/view.rs:1442:        ui.memory_mut(|m| {
libraries/viu/src/terminal/view.rs:1477:            response.has_focus() && ui.input(|i| i.focused),
libraries/viu/src/terminal/view.rs:1482:            ui.ctx().request_repaint_after(next);
libraries/viu/src/terminal/view.rs:1500:            ui.ctx().output_mut(|o| {
libraries/viu/src/view_motion_identity_tests.rs:15:            ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/src/view_motion_identity_tests.rs:25:            let focused = ctx.memory(|m| m.focused()).expect("field focus");
libraries/viu/tests/widget_rectangles.rs:16:    ctx.run_ui(
libraries/viu/tests/widget_rectangles.rs:134:    ctx.run_ui(
libraries/viu/tests/widget_rectangles.rs:269:    ctx.set_zoom_factor(1.5);
libraries/viu/src/terminal/primary_font_tests.rs:31:        .fonts()
libraries/viu/src/terminal/primary_font_tests.rs:205:        .output()
libraries/viu/tests/scene_motion.rs:102:            ctx.global_style_mut(|s| s.animation_time = 0.);
libraries/viu/tests/scene_motion.rs:128:            ctx.run_ui(raw, |ui| commands.extend(view.show(ui)))
libraries/viu/tests/scene_motion.rs:214:                ctx.run_ui(
libraries/viu/tests/scene_motion.rs:251:    assert!(!ctx.has_requested_repaint());
libraries/viu/tests/scene_motion.rs:278:                ctx.global_style_mut(|s| s.animation_time = 0.);
libraries/viu/tests/scene_motion.rs:328:            ctx.run_ui(raw, |ui| commands.extend(view.show(ui)))
libraries/viu/tests/scene_motion.rs:354:    ctx.run_ui(
libraries/viu/tests/scene_motion.rs:429:            let _ = ctx.tessellate(std::mem::take(&mut out.shapes), out.pixels_per_point);
libraries/viu/tests/session.rs:92:        ctx.global_style().visuals.dark_mode,
libraries/viu/tests/session.rs:109:    let before = ctx.global_style();
libraries/viu/tests/session.rs:115:    let after = ctx.global_style();
libraries/viu/tests/session.rs:130:        ctx.global_style().visuals.panel_fill
libraries/viu/tests/session.rs:162:    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/session.rs:180:    let mut output = ctx.run_ui(
libraries/viu/tests/session.rs:213:    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/session.rs:234:        let mut output = ctx.run_ui(
libraries/viu/tests/session.rs:268:        let mut output = ctx.run_ui(
libraries/viu/tests/session.rs:321:        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/session.rs:353:        ctx.data_mut(|d| d.insert_temp(key, serde_json::json!({"mode":"dark"})));
libraries/viu/tests/session.rs:374:        assert!(ctx.data(|d| d.get_temp::<serde_json::Value>(key)).is_none());
libraries/viu/tests/frame_geometry.rs:8:        ctx.run_ui(
libraries/viu/tests/workshop_frame.rs:205:        ctx.run_ui(
libraries/viu/tests/preview.rs:19:        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/monitor_performance.rs:57:            ctx.run_ui(
libraries/viu/src/app_host_model_tests.rs:474:    let mut style = (*ctx.global_style()).clone();
libraries/viu/src/app_host_model_tests.rs:476:    ctx.set_global_style(style);
libraries/viu/src/app_host_model_tests.rs:518:        json!(ctx.global_style().spacing.item_spacing.x)
libraries/viu/src/widgets.rs:308:                + 2.0 * ui.ctx().global_style().spacing.interact_size.x
libraries/viu/src/widgets.rs:309:                + 2.0 * ui.ctx().global_style().spacing.item_spacing.x);
libraries/viu/src/widgets.rs:1144:                .layer_transform_to_global(ui.layer_id())
libraries/viu/src/widgets.rs:1167:                .data_mut(|d| d.insert_temp(focus_key(), id.to_owned()));
libraries/viu/src/widgets.rs:1258:                ui.spacing_mut().item_spacing.x = ui.ctx().global_style().spacing.item_spacing.x;
libraries/viu/src/widgets.rs:1586:                let start = ui.ctx().graphics_mut(|g| g.entry(ui.layer_id()).next_idx());
libraries/viu/src/widgets.rs:1623:                        .map(|key| ui.ctx().format_shortcut(&key));
libraries/viu/src/widgets.rs:1763:                        && ui.input(|i| {
libraries/viu/src/widgets.rs:1781:                    && ui.input(|i| i.key_pressed(egui::Key::Escape))
libraries/viu/src/widgets.rs:1789:                } else if (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
libraries/viu/src/widgets.rs:1792:                        && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command))
libraries/viu/src/terminal/selection_worker.rs:532:                .and_then(|d| d.data().selection_offer())
libraries/viu/src/terminal/selection_worker.rs:542:                .and_then(|d| d.data().selection_offer())
libraries/viu/tests/documents.rs:41:    let output = ctx.run_ui(
libraries/viu/tests/file_widgets.rs:37:    let mut out = ctx.run_ui(input, |ui| {
libraries/viu/tests/file_widgets.rs:152:    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
libraries/viu/tests/file_widgets.rs:154:    ctx.set_theme(egui::Theme::Dark);
libraries/viu/tests/file_widgets.rs:155:    ctx.set_style_of(egui::Theme::Dark, style);
libraries/viu/tests/file_widgets.rs:166:    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/file_widgets.rs:374:    let mut out = ctx.run_ui(
libraries/viu/tests/file_widgets.rs:431:    let mut out = ctx.run_ui(raw(Vec::new()), |ui| {
libraries/viu/tests/file_widgets.rs:436:    let mut out = ctx.run_ui(
libraries/viu/tests/file_widgets.rs:452:    let mut out = ctx.run_ui(
libraries/viu/src/network_map.rs:96:    let cached = ui.ctx().data_mut(|d| d.get_temp::<Arc<Geometry>>(id));
libraries/viu/src/network_map.rs:101:        ui.ctx().data_mut(|d| d.insert_temp(id, geometry.clone()));
libraries/viu/src/network_map.rs:499:    if !activated && ui.rect_contains_pointer(rect) && ui.input(|i| i.pointer.primary_clicked()) {
libraries/viu/src/network_map.rs:500:        if let Some(pointer) = ui.input(|i| i.pointer.interact_pos()).filter(|pointer| {
libraries/viu/src/network_map.rs:524:    let frame = ui.input(|i| i.time);
libraries/viu/src/network_map.rs:525:    let (shapes, moving, next) = ui.ctx().data_mut(|d| {
libraries/viu/src/network_map.rs:581:        ui.ctx().request_repaint();
libraries/viu/src/network_map.rs:585:                .request_repaint_after(std::time::Duration::from_secs_f64((next - frame).max(0.)));
libraries/viu/tests/scene_adapter.rs:27:        ctx.run_ui(
libraries/viu/tests/scene_adapter.rs:66:            ctx.run_ui(
libraries/viu/tests/scene_adapter.rs:110:            ctx.run_ui(
libraries/viu/tests/scene_adapter.rs:168:            ctx.run_ui(
libraries/viu/tests/scene_adapter.rs:202:    let mut output = ctx.run_ui(input(), |ui| {
libraries/viu/tests/scene_adapter.rs:214:        let mut output = ctx.run_ui(input(), |ui| {
libraries/viu/tests/scene_adapter.rs:229:        let mut output = ctx.run_ui(input(), |ui| {
libraries/viu/tests/scene_adapter.rs:256:    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/scene_adapter.rs:271:    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/scene_adapter.rs:324:    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/scene_adapter.rs:331:    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/scene_adapter.rs:371:        let mut out = ctx.run_ui(raw, |ui| {
libraries/viu/tests/scene_adapter.rs:395:    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/scene_adapter.rs:414:        let mut out = ctx.run_ui(
libraries/viu/tests/scene_adapter.rs:487:        let mut out = ctx.run_ui(raw, |ui| {
libraries/viu/tests/scene_adapter.rs:513:        let mut out = ctx.run_ui(
libraries/viu/tests/scene_adapter.rs:533:    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/scene_adapter.rs:591:        let mut out = ctx.run_ui(
libraries/viu/tests/scene_adapter.rs:632:    let mut out = ctx.run_ui(
libraries/viu/tests/scene_adapter.rs:648:    let mut out = ctx.run_ui(
libraries/viu/tests/dopus_bundle.rs:74:    ctx.run_ui(
libraries/viu/tests/dopus_bundle.rs:158:    ctx.run_ui(
libraries/viu/src/time_series.rs:338:    ctx.data_mut(|data| {
libraries/viu/src/time_series.rs:667:        .data_mut(|data| {
libraries/viu/src/time_series.rs:734:            && let Some(pointer) = plot_ui.ctx().input(|input| input.pointer.hover_pos())
libraries/viu/src/time_series.rs:738:            let factor = plot_ui.ctx().input_mut(|input| {
libraries/viu/src/time_series.rs:774:    ui.ctx().data_mut(|data| {
libraries/viu/tests/deck_native.rs:147:        let UnixConnectOutcome::VerifiedUnix(caller)=NodedClient::connect_unix("",&broker.url,&broker.options(),None).await.unwrap() else {panic!("verified native ingress required")};
libraries/viu/tests/deck_native.rs:184:        let verified=Command::new(bin.join("mixd")).args(["vfl","verify"]).arg(&full_path).output().unwrap();
libraries/viu/tests/deck_native.rs:191:        .output()
libraries/viu/src/input.rs:409:        .is_some_and(|chord| ctx.input_mut(|i| i.consume_key(chord.modifiers, chord.logical_key)))
libraries/viu/src/debug_recent.rs:256:                    .output()
libraries/viu/src/design_validation.rs:282:    ctx.data_mut(|data| {
libraries/viu/src/design_validation.rs:293:        .data_mut(|data| {
libraries/viu/tests/term_bundle.rs:70:        .output()
libraries/viu/src/cuboids.rs:60:            .min(ui.ctx().pixels_per_point());
libraries/viu/src/cuboids.rs:345:            let mut out = ctx.run_ui(
libraries/viu/src/editor.rs:109:                        .send_viewport_cmd(egui::ViewportCommand::RequestPaste);
libraries/viu/src/editor.rs:116:                        ui.ctx().copy_text(text.to_owned());
libraries/viu/src/editor.rs:150:        if ui.memory(|m| m.has_focus(editor_id))
libraries/viu/src/editor.rs:151:            && ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::F))
libraries/viu/src/editor.rs:261:        .data(|d| d.get_temp::<editor::Palette>(egui::Id::unique("viu-gold-editor-palette")))
libraries/viu/src/terminal/admission.rs:137:        let supervisor = Supervisor::with_options(broker.options(), broker.url.clone()).unwrap();
libraries/viu/src/terminal/admission.rs:150:            let UnixConnectOutcome::VerifiedUnix(caller) = NodedClient::connect_unix("", &broker.url, &broker.options(), None).await.unwrap() else {panic!("verified observer required")};
libraries/viu/src/file_tiles.rs:96:    let vp = ui.ctx().viewport_id();
libraries/viu/src/file_tiles.rs:98:        .data(|d| {
libraries/viu/src/file_tiles.rs:313:            writer.data_mut(|_| {});
libraries/viu/src/file_tiles.rs:331:        ctx.run_ui(Default::default(), |ui| {
libraries/viu/src/file_tiles.rs:332:            let vp = ui.ctx().viewport_id();
libraries/viu/src/file_tiles.rs:333:            ui.ctx().data_mut(|d| {
libraries/viu/src/file_tiles.rs:341:                ui.ctx().data(|d| {
libraries/viu/src/file_tiles.rs:344:                        ui.ctx().viewport_id(),
libraries/viu/src/file_tiles.rs:438:            let output = ctx.run_ui(Default::default(), |ui| {
libraries/viu/src/file_tiles.rs:439:                let vp = ui.ctx().viewport_id();
libraries/viu/src/file_tiles.rs:440:                ui.ctx().data_mut(|d| {
libraries/viu/src/chooser_host.rs:70:            context.request_repaint_of(vid);
libraries/viu/src/chooser_host.rs:71:            context.request_repaint_of(egui::ViewportId::ROOT);
libraries/viu/src/chooser_host.rs:218:                    ctx.request_repaint();
libraries/viu/src/chooser_host.rs:253:                let style = ctx.global_style();
libraries/viu/src/chooser_host.rs:256:                projected["__viu_metrics"] = json!({"item_spacing_x":style.spacing.item_spacing.x,"item_spacing_y":style.spacing.item_spacing.y,"control_row_height":toolkit::button::icon_button_of(&ctx).max(style.spacing.interact_size.y+4.0),"field_height":toolkit::field::height_of(&ctx),"height":ctx.content_rect().height(),"dialog_resize_max_share":toolkit::chrome::Chrome::of(&ctx).style.dialog_resize_max_share});
libraries/viu/src/chooser_host.rs:270:        if ctx.input(|i| i.viewport().close_requested()) && !self.closed {
libraries/viu/src/chooser_host.rs:301:                let v = ctx.input(|i| i.viewport().clone());
libraries/viu/src/chooser_host.rs:313:                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
libraries/viu/src/chooser_host.rs:329:                    ctx.request_repaint();
libraries/viu/src/chooser_host.rs:383:                ctx.request_repaint();
libraries/viu/src/chooser_host.rs:390:            let keys:Vec<_>=ctx.input(|input|input.events.iter().filter_map(|event|{
libraries/viu/src/chooser_host.rs:400:                ctx.request_repaint();
libraries/viu/src/chooser_host.rs:543:                        ctx.request_repaint_of(viewport(&window.id));
libraries/viu/src/chooser_host.rs:600:            ctx.request_repaint();
libraries/viu/src/chooser_host.rs:603:            ctx.request_repaint_after(
libraries/viu/src/chooser_host.rs:695:                ctx.send_viewport_cmd_to(viewport(&id), egui::ViewportCommand::Close);
libraries/viu/src/chooser_host.rs:711:                        ctx.request_repaint_after(std::time::Duration::from_secs(5));
libraries/viu/src/chooser_host.rs:730:                ctx.request_repaint_of(viewport(&id));
libraries/viu/src/chooser_host.rs:734:                ctx.request_repaint_of(viewport(&id));
libraries/viu/src/chooser_host.rs:775:            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
libraries/viu/src/chooser_host.rs:811:            ui.ctx().show_viewport_deferred(
libraries/viu/src/chooser_host.rs:869:            cc.egui_ctx.set_embed_viewports(false);
libraries/viu/src/chooser_host.rs:871:                .send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
libraries/viu/src/chooser_host.rs:873:            let wake: Wake = Arc::new(move || context.request_repaint_of(egui::ViewportId::ROOT));
libraries/viu/src/tree_table.rs:22:    let snapshot = ui.data_mut(|data| {
libraries/viu/src/tree_table.rs:31:    ui.fonts(|fonts| {
libraries/viu/src/tree_table.rs:137:    ui.data_mut(|data| {
libraries/viu/src/tree_table.rs:174:        scale: ui.ctx().pixels_per_point(),
libraries/viu/src/tree_table.rs:176:    let cache_id = Id::unique((ui.ctx().viewport_id(), id, index, "sample-width"));
libraries/viu/src/tree_table.rs:298:            ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/src/tree_table.rs:341:                44 => ctx.set_pixels_per_point(2.5),
libraries/viu/src/tree_table.rs:342:                45 => ctx.all_styles_mut(|style| {
libraries/viu/src/tree_table.rs:353:                    ctx.set_fonts(definitions);
libraries/viu/src/tree_table.rs:364:            ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/src/tree_table.rs:401:                        Id::unique((ui.ctx().viewport_id(), "stream", 1_usize, "column-widths"));
libraries/viu/src/tree_table.rs:471:                ctx.set_pixels_per_point(2.5);
libraries/viu/src/tree_table.rs:474:                ctx.style_mut_of(egui::Theme::Dark, |style| {
libraries/viu/src/tree_table.rs:486:                ctx.set_fonts(definitions);
libraries/viu/src/tree_table.rs:489:            ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/src/tree_table.rs:522:                ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/src/tree_table.rs:547:                    ctx.set_pixels_per_point(scale);
libraries/viu/src/tree_table.rs:554:                        ctx.run_ui(
libraries/viu/src/tree_table.rs:593:            ctx.set_pixels_per_point(scale);
libraries/viu/src/tree_table.rs:605:            ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/src/tree_table.rs:876:            scale: ui.ctx().pixels_per_point(),
libraries/viu/src/tree_table.rs:878:        let cache_id = Id::unique((ui.ctx().viewport_id(), id, i, "column-widths"));
libraries/viu/src/tree_table.rs:925:                json!({"source":payload.source,"keys":payload.keys,"destination":destination,"ctrl":ui.input(|i|i.modifiers.ctrl||i.modifiers.command),"shift":ui.input(|i|i.modifiers.shift)}),
libraries/viu/src/tree_table.rs:1056:        ui.ctx().content_rect().height() < number(&widget.properties, "compact_below_height", 0.0);
libraries/viu/src/tree_table.rs:1083:            let response=ui.interact(cell,Id::unique((ui.ctx().viewport_id(),id,"column",i)),Sense::click());
libraries/viu/src/tree_table.rs:1096:        let background=ui.interact(body,Id::unique((ui.ctx().viewport_id(),id,"background")),Sense::click());
libraries/viu/src/tree_table.rs:1101:            ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
libraries/viu/src/tree_table.rs:1128:                    let toggle=ui.interact(chevron,Id::unique((ui.ctx().viewport_id(),id,key,"expand")),Sense::click());
libraries/viu/src/tree_table.rs:1156:                let modifiers=ui.input(|i|i.modifiers);
libraries/viu/src/tree_table.rs:1171:        if let Some(pos) = ui.ctx().pointer_hover_pos() {
libraries/viu/src/tree_table.rs:1172:            egui::Area::new(Id::unique((ui.ctx().viewport_id(), id, "drag-label")))
libraries/viu/src/tree_table.rs:1444:            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
libraries/viu/src/tree_table.rs:1489:        let marker = Id::unique((ui.ctx().viewport_id(), id, "reveal-scroll"));
libraries/viu/src/tree_table.rs:1491:        let changed = ui.ctx().data_mut(|data| {
libraries/viu/src/tree_table.rs:1621:                let changed = ui.ctx().data_mut(|d| {
libraries/viu/src/tree_table.rs:1665:        if ui.input(|i| i.pointer.primary_pressed()) && ui.rect_contains_pointer(area) {
libraries/viu/src/tree_table.rs:1668:        if ui.memory(|m| m.has_focus(focus_id)) {
libraries/viu/src/tree_table.rs:1669:            ui.memory_mut(|m| {
libraries/viu/src/tree_table.rs:1686:                if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
libraries/viu/src/tree_table.rs:1690:            if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
libraries/viu/src/tree_table.rs:1735:                                            ui.memory_mut(|m| m.request_focus(focus_id));
libraries/viu/src/tree_table.rs:1767:                                        ui.memory_mut(|m| m.request_focus(focus_id));
libraries/viu/src/tree_table.rs:1781:                    if focused(value, row) && (!keyboard || ui.memory(|m| m.has_focus(focus_id))) {
libraries/viu/src/debug.rs:204:                .env(CHILD,"1").env_remove("RUST_LOG").output().unwrap();
libraries/viu/src/media_gallery.rs:33:            wake.request_repaint();
libraries/viu/src/media_gallery.rs:68:                    image.map(|image| ctx.load_texture(&key, image, egui::TextureOptions::LINEAR)),
libraries/viu/src/media_gallery.rs:114:    let key = egui::Id::unique((ui.ctx().viewport_id(), id, "media"));
libraries/viu/src/media_gallery.rs:115:    ui.ctx().data_mut(|d| {
libraries/viu/src/media_gallery.rs:194:        ui.ctx().pixels_per_point(),
libraries/viu/src/media_gallery.rs:643:        ui.ctx().pixels_per_point(),
libraries/viu/src/command_palette.rs:37:            .run_ui(egui::RawInput::default(), |ui| {
libraries/viu/src/design_style.rs:605:            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/src/documents.rs:647:                .data_mut(|d| d.insert_temp(egui::Id::unique("viu-editor-focused"), true));
libraries/viu/src/editor_controls.rs:251:        let key = Id::unique((ui.ctx().viewport_id(), id, "query"));
libraries/viu/src/editor_controls.rs:253:            ui.ctx().data_mut(|d| d.remove::<String>(key));
libraries/viu/src/editor_controls.rs:256:        let previous = ui.ctx().data(|d| d.get_temp::<String>(key));
libraries/viu/src/editor_controls.rs:279:        ui.ctx().data_mut(|d| d.insert_temp(key, query));
libraries/viu/src/editor_controls.rs:474:                            && ui.ctx().data(|d| d.get_temp::<Value>(marker)).as_ref()
libraries/viu/src/editor_controls.rs:479:                                .data_mut(|d| d.insert_temp(marker, row["focus_instance"].clone()));
libraries/viu/src/editor_controls.rs:482:                            ui.ctx().data_mut(|d| {
libraries/viu/src/editor_controls.rs:542:                            .data(|d| {
libraries/viu/src/editor_controls.rs:566:                let submit_event = ui.input(|i| {
libraries/viu/src/editor_controls.rs:584:                    && ui.ctx().data(|d| d.get_temp::<Value>(marker)).as_ref()
libraries/viu/src/editor_controls.rs:589:                        .data_mut(|d| d.insert_temp(marker, value["instance"].clone()));
libraries/viu/src/editor_controls.rs:598:                        ui.input_mut(|input| {
libraries/viu/src/editor_controls.rs:611:                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
libraries/viu/src/editor_controls.rs:624:                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
libraries/viu/src/annotation/paint.rs:63:            .data_mut()
libraries/viu/src/annotation/paint.rs:330:    let data = pixmap.data_mut();
libraries/viu/src/terminal/model.rs:1384:                    let style = term.grid.style_of(square);
libraries/viu/src/navigation_list.rs:31:    *ui.spacing_mut() = ui.ctx().global_style().spacing.clone();
libraries/viu/src/hosting.rs:151:                ctx.request_repaint_after(at.saturating_duration_since(Instant::now()));
libraries/viu/src/hosting.rs:167:                ctx.request_repaint_after(Duration::from_millis(100));
libraries/viu/src/appearance.rs:12:    ctx.data(|d| d.get_temp::<toolkit::Theme>(egui::Id::unique("viu-session-theme")))
libraries/viu/src/appearance.rs:34:        return Some(ctx.global_style().visuals.selection.bg_fill);
libraries/viu/src/appearance.rs:36:    ctx.data(|d| {
libraries/viu/src/appearance.rs:59:    ctx.data_mut(|d| {
libraries/viu/src/appearance.rs:127:    ctx.data_mut(|d| {
libraries/viu/src/appearance.rs:140:        ctx.style_mut_of(theme, |style| style.debug.show_unaligned = false);
libraries/viu/src/appearance.rs:142:    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("viu-session-theme"), theme.clone()));
libraries/viu/src/appearance.rs:156:    ctx.data_mut(|d| {
libraries/viu/src/appearance.rs:195:    if ctx.data(|d| d.get_temp::<Value>(key)).as_ref() == Some(value) {
libraries/viu/src/appearance.rs:210:        .data(|d| d.get_temp::<toolkit::Theme>(egui::Id::unique("viu-session-theme")))
libraries/viu/src/appearance.rs:227:    ctx.data_mut(|d| {
libraries/viu/src/appearance.rs:267:    ctx.set_theme(theme);
libraries/viu/src/appearance.rs:268:    ctx.set_style_of(theme, style);
libraries/viu/src/appearance.rs:278:    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("viu-gold-fonts"), false));
libraries/viu/src/appearance.rs:287:    ctx.data_mut(|d| {
libraries/viu/src/appearance.rs:294:    ctx.set_theme(theme);
libraries/viu/src/appearance.rs:295:    ctx.set_style_of(egui::Theme::Light, style.clone());
libraries/viu/src/appearance.rs:296:    ctx.set_style_of(egui::Theme::Dark, style);
libraries/viu/src/thumbnail.rs:258:        ctx.request_repaint();
libraries/viu/src/raster.rs:33:            ctx.request_repaint();
libraries/viu/src/session.rs:54:                ctx.global_style().visuals.panel_fill,
libraries/viu/src/session.rs:55:                ctx.global_style().visuals.dark_mode,
libraries/viu/src/terminal/clipboard.rs:90:            None if !primary => ctx.copy_text(text),
libraries/viu/tests/app_mode.rs:227:        .output()
libraries/viu/src/native_menu.rs:66:    egui::Id::unique((ctx.viewport_id(), "viu-native-menu-model"))
libraries/viu/src/native_menu.rs:71:    ctx.data(|data| data.get_temp::<Model>(key))?
libraries/viu/src/native_menu.rs:78:    let index = ctx.data_mut(|d| {
libraries/viu/src/native_menu.rs:96:            .and_then(|shortcut| ctx.map(|ctx| ctx.format_shortcut(&shortcut)))
libraries/viu/src/native_menu.rs:99:            ctx.and_then(|ctx| toolkit::keymap::parse_chord(s).map(|key| ctx.format_shortcut(&key)))
libraries/viu/src/native_menu.rs:164:    if let Some(old) = ui.ctx().data(|d| d.get_temp::<Model>(key)) {
libraries/viu/src/native_menu.rs:175:        .data_mut(|d| d.insert_temp(key, Model { ids, menu_len }));
libraries/viu/src/native_menu.rs:192:                let rect = ui.ctx().content_rect();
libraries/viu/src/native_menu.rs:214:                if (response.hovered() && ui.input(|i| i.pointer.primary_pressed()))
libraries/viu/src/native_menu.rs:228:                .data_mut(|d| d.insert_temp(key.with("picked"), serde_json::json!({"command":command,"origin":if pick.1 {"palette"}else{"menu"}})));
libraries/viu/src/native_menu.rs:235:    ctx.data_mut(|d| d.remove_temp::<Value>(key))
libraries/viu/src/native_menu.rs:261:                let out = ctx.run_ui(
libraries/viu/src/native_menu.rs:297:            ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/src/native_menu.rs:349:            .round_to_pixels(ui.pixels_per_point());
libraries/viu/src/backdrop.rs:74:    ctx.data_mut(|d| {
libraries/viu/src/backdrop.rs:79:    let texture = ctx.load_texture(
libraries/viu/src/backdrop.rs:107:            ctx.data(|d| {
libraries/viu/src/composition.rs:145:            ui.ctx().request_repaint_after(Duration::from_millis(
libraries/viu/src/shortcuts.rs:96:    let typing = ctx.egui_wants_keyboard_input();
libraries/viu/src/shortcuts.rs:98:    ctx.input_mut(|input| {
libraries/viu/src/shortcuts.rs:135:    let clipboard = !ctx.egui_wants_keyboard_input();
libraries/viu/src/shortcuts.rs:136:    ctx.input_mut(|input| {
libraries/viu/src/shortcuts.rs:240:        ctx.run_ui(egui::RawInput { events:vec![event(egui::Key::B),event(egui::Key::A)], ..Default::default() },|ui| {
libraries/viu/src/shortcuts.rs:251:            ctx.run_ui(
libraries/viu/src/shortcuts.rs:278:        ctx.run_ui(
libraries/viu/tests/term_host.rs:116:            NodedClient::connect_unix("", &broker.url, &broker.options(), None)
libraries/viu/src/native_bus.rs:905:                            NodedClient::connect_unix(base, &broker.url, &broker.options(), None)
libraries/viu/src/native_bus.rs:940:                    ctx.data_mut(|data| {
libraries/viu/src/native_bus.rs:974:                        NodedClient::connect_unix("", &broker.url, &broker.options(), None)
libraries/viu/src/native_bus.rs:1166:                NodedClient::connect_unix("", &broker.url, &broker.options(), None)
libraries/viu/src/image_view.rs:203:        let mut points = ui.data_mut(|d| d.get_temp::<Vec<Point>>(gesture_id).unwrap_or_default());
libraries/viu/src/image_view.rs:204:        let panning = ui.input(|i| {
libraries/viu/src/image_view.rs:215:        if ui.data_mut(|d| d.get_temp::<f32>(scale_id)) != Some(view.scale) {
libraries/viu/src/image_view.rs:216:            ui.data_mut(|d| d.insert_temp(scale_id, view.scale));
libraries/viu/src/image_view.rs:220:            ui.data_mut(|d| d.remove::<Vec<Point>>(gesture_id));
libraries/viu/src/image_view.rs:225:                .set_cursor_icon(if value["guide_mode"] == "select" {
libraries/viu/src/image_view.rs:232:            let zoom = ui.input(|i| i.zoom_delta());
libraries/viu/src/image_view.rs:236:            let wheel = ui.input(|i| i.smooth_scroll_delta);
libraries/viu/src/image_view.rs:242:            let delta = ui.input(|i| i.pointer.delta());
libraries/viu/src/image_view.rs:251:                    .input(|i| i.pointer.press_origin())
libraries/viu/src/image_view.rs:318:        ui.data_mut(|d| d.insert_temp(gesture_id, points));
libraries/viu/src/image_view.rs:380:                    let limit = ui.input(|i| i.max_texture_side);
libraries/viu/src/image_view.rs:391:                                .load_texture(handle, image, egui::TextureOptions::LINEAR);
libraries/viu/src/image_view.rs:434:                    ui.ctx().load_texture(
libraries/viu/src/image_view.rs:462:                / ui.ctx().pixels_per_point();
libraries/viu/src/image_view.rs:771:        ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/src/raster_fill.rs:78:        ui.ctx().pixels_per_point(),
libraries/viu/src/raster_fill.rs:137:                        let maximum = ui.input(|i| i.max_texture_side);
libraries/viu/src/raster_fill.rs:150:                                    let texture = ui.ctx().load_texture(
libraries/viu/src/raster_fill.rs:201:                ui.ctx().request_repaint();
libraries/viu/tests/app_mix.rs:46:            .output()
libraries/viu/src/terminal/sessions.rs:901:                .output()
libraries/viu/tests/app_ports.rs:73:        ctx.run_ui(
libraries/viu/src/app_host/osd.rs:149:                    ctx.request_repaint();
libraries/viu/src/app_host/osd.rs:209:        let presses = ui.input(|i| {
libraries/viu/src/app_host/osd.rs:298:            ui.ctx().transform_layer_shapes(
libraries/viu/src/app_host/osd.rs:306:                ui.ctx().request_repaint_after(Duration::from_millis(16));
libraries/viu/src/app_host/osd.rs:481:            let output = context.run_ui(
libraries/viu/src/header_controls.rs:205:                                if suggestions.is_some() && ui.memory(|m| m.has_focus(field_id)) {
libraries/viu/src/header_controls.rs:206:                                    ui.input_mut(|i| {
libraries/viu/src/header_controls.rs:332:                                    && ui.ctx().data(|d| d.get_temp::<Value>(marker)).as_ref()
libraries/viu/src/header_controls.rs:336:                                    ui.ctx().data_mut(|d| {
libraries/viu/src/header_controls.rs:341:                                    ui.ctx().data_mut(|d| {
libraries/viu/src/header_controls.rs:467:        let chooser = ui.ctx().data_mut(|d| {
libraries/viu/src/header_controls.rs:495:            ui.ctx().data_mut(|d| {
libraries/viu/src/header_controls.rs:501:            .data_mut(|d| d.remove::<Arc<Mutex<toolkit::file_chooser::FileChooser>>>(chooser_id));
libraries/viu/src/rich.rs:26:    egui::Id::unique((ctx.viewport_id(), "viu-menu-presentation"))
libraries/viu/src/rich.rs:30:    ctx.data(|data| data.get_temp(key)).unwrap_or_default()
libraries/viu/src/rich.rs:36:    ctx.data_mut(|data| {
libraries/viu/src/rich.rs:47:    ctx.data_mut(|d| {
libraries/viu/src/rich.rs:55:    ctx.data_mut(|d| {
libraries/viu/src/rich.rs:61:        egui::Id::unique((ctx.viewport_id(), "viu-hosted-command-popup")),
libraries/viu/src/rich.rs:63:    ctx.request_repaint();
libraries/viu/src/rich.rs:79:    } else if style == design::MenuStyle::Bar && ctx.content_rect().width() >= 640.0 {
libraries/viu/src/rich.rs:104:    let bounds = ctx.content_rect();
libraries/viu/src/rich.rs:118:    ctx.data_mut(|data| {
libraries/viu/src/rich.rs:124:        egui::Id::unique((ctx.viewport_id(), id, "vertical-menu")),
libraries/viu/src/rich.rs:126:    ctx.request_repaint();
libraries/viu/src/rich.rs:556:                let rect = ui.ctx().content_rect();
libraries/viu/src/rich.rs:578:                if (response.hovered() && ui.input(|i| i.pointer.primary_pressed()))
libraries/viu/src/rich.rs:613:        let marker = egui::Id::unique((ui.ctx().viewport_id(), id, "search-instance"));
libraries/viu/src/rich.rs:615:            ui.ctx().data(|d| d.get_temp::<Value>(marker)).as_ref() != Some(&value["instance"]);
libraries/viu/src/rich.rs:652:                .data_mut(|d| d.insert_temp(marker, value["instance"].clone()));
libraries/viu/src/rich.rs:738:                let popup_id = egui::Id::unique((ui.ctx().viewport_id(), id, "vertical-menu"));
libraries/viu/src/rich.rs:743:                        egui::pos2(x, ui.ctx().content_rect().top()),
libraries/viu/src/rich.rs:744:                        egui::pos2(x, ui.ctx().content_rect().top()),
libraries/viu/src/rich.rs:788:                                            ui.ctx().format_shortcut(&s)
libraries/viu/src/rich.rs:839:                        ui.ctx().content_rect().right() - 3.0 * chrome.metrics.caption_width;
libraries/viu/src/rich.rs:864:                                    ui.ctx().format_shortcut(&shortcut)
libraries/viu/src/rich.rs:971:                                                            ui.ctx().format_shortcut(&s)
libraries/viu/src/rich.rs:1013:                                let viewport = ui.ctx().viewport_id();
libraries/viu/src/rich.rs:1017:                                ui.ctx().data_mut(|d| d.insert_temp(anchor, response.rect));
libraries/viu/src/rich.rs:1018:                                if ui.ctx().data(|d| d.get_temp::<bool>(busy)).unwrap_or(false) {
libraries/viu/src/rich.rs:1055:                let bounds = ui.ctx().content_rect();
libraries/viu/src/rich.rs:1057:                    egui::Id::unique((ui.ctx().viewport_id(), "viu-hosted-command-popup")),
libraries/viu/src/rich.rs:1095:                    ui.ctx().viewport_id(),
libraries/viu/src/rich.rs:1141:                                                    .map(|s| ui.ctx().format_shortcut(&s))
libraries/viu/src/rich.rs:1173:                            response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
libraries/viu/src/rich.rs:1174:                        if !response.has_focus() && ui.memory(|m| m.focused().is_none()) {
libraries/viu/src/rich.rs:1215:            let modal = egui::Modal::new(egui::Id::unique((ui.ctx().viewport_id(), id, "modal")))
libraries/viu/src/rich.rs:1218:                    ui.set_min_width(width.min(ui.ctx().content_rect().width()-32.0).max(280.0));
libraries/viu/src/rich.rs:1233:                                .id(egui::Id::unique((ui.ctx().viewport_id(), id, "field")))
libraries/viu/src/rich.rs:1238:                            ui.ctx().viewport_id(),
libraries/viu/src/rich.rs:1242:                        if ui.ctx().data(|d|d.get_temp::<Value>(marker)).as_ref()!=Some(&value["instance"])
libraries/viu/src/rich.rs:1243:                            || (!response.has_focus() && ui.memory(|m|m.focused().is_none())) {
libraries/viu/src/rich.rs:1245:                            ui.ctx().data_mut(|d| d.insert_temp(marker,value["instance"].clone()));
libraries/viu/src/rich.rs:1299:            } else if ui.input(|i| {
libraries/viu/src/rich.rs:1336:        ui.ctx().viewport_id(),
libraries/viu/src/rich.rs:1353:            if response.lost_focus() && ui.input(|i|i.key_pressed(egui::Key::Enter)){accept=true;}
libraries/viu/src/rich.rs:1354:            else if !response.has_focus() && ui.memory(|m| m.focused().is_none()){response.request_focus();}
libraries/viu/src/rich.rs:1386:            let max_height=ui.ctx().content_rect().height()*toolkit::chrome::Chrome::of(ui.ctx()).style.dialog_resize_max_share as f32/2.0;
libraries/viu/src/rich.rs:1401:                                    .map(|s| ui.ctx().format_shortcut(&s))
libraries/viu/src/rich.rs:1422:                            .map(|key|toolkit::keymap::parse_chord(key).map_or_else(||key.to_owned(),|key|ui.ctx().format_shortcut(&key)))
libraries/viu/src/rich.rs:1458:                        if let Some(shortcut)=row["shortcut"].as_str(){ui.monospace(toolkit::keymap::parse_chord(shortcut).map_or_else(||shortcut.to_owned(),|key|ui.ctx().format_shortcut(&key)));}
libraries/viu/src/rich.rs:1527:            ctx.run_ui(
libraries/viu/src/terminal/session_fd.rs:233:                .output()
libraries/viu/src/terminal/session_fd.rs:365:                .output()
libraries/viu/src/terminal/session_actor.rs:1661:        let supervisor = Supervisor::with_options(broker.options(), broker.url.clone()).unwrap();
libraries/viu/src/terminal/session_actor.rs:1731:        let mut options = broker.options();
libraries/viu/src/terminal/session_actor.rs:1744:            options: broker.options(),
libraries/viu/src/terminal/session_actor.rs:1798:        let supervisor = Supervisor::with_options(broker.options(), broker.url.clone()).unwrap();
libraries/viu/src/settings_browser.rs:342:    ctx.data_mut(|data| {
libraries/viu/src/settings_browser.rs:369:    ctx.data_mut(|data| {
libraries/viu/src/settings_browser.rs:414:                    let mut output = ctx.run_ui(
libraries/viu/src/settings_browser.rs:498:            ctx.run_ui(
libraries/viu/src/settings_browser.rs:586:        .data(|d| d.get_temp::<String>(key))
libraries/viu/src/settings_browser.rs:590:        response.ctx.data_mut(|d| {
libraries/viu/src/settings_browser.rs:605:    ui.ctx().data_mut(|d| {
libraries/viu/src/settings_browser.rs:640:            ui.ctx().viewport_id(),
libraries/viu/src/settings_browser.rs:681:                    if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
libraries/viu/src/settings_browser.rs:684:                        ui.memory_mut(|m| m.surrender_focus(r.id));
libraries/viu/src/settings_browser.rs:1126:        .data_mut(|d| d.remove_temp::<bool>(focus_key.with("done")))
libraries/viu/src/settings_browser.rs:2312:                        .data(|d| d.get_temp::<String>(filter_id))
libraries/viu/src/settings_browser.rs:2322:                        ui.data_mut(|d| d.insert_temp(filter_id, query.clone()));
libraries/viu/src/settings_browser.rs:2666:                                            ui.ctx().data_mut(|d| {
libraries/viu/src/settings_browser.rs:2675:                                                .data_mut(|d| d.get_temp::<String>(field_id))
libraries/viu/src/settings_browser.rs:2709:                                                && ui.input(|i| i.key_pressed(egui::Key::Enter)));
libraries/viu/src/settings_browser.rs:2712:                                                ui.ctx().data_mut(|d| {
libraries/viu/src/settings_browser.rs:2803:        .data(|d| d.get_temp::<String>(filter_id))
libraries/viu/src/settings_browser.rs:2812:        ui.data_mut(|d| d.insert_temp(filter_id, query.clone()));
libraries/viu/src/settings_browser.rs:2815:    let mut extended = ui.data(|d| d.get_temp::<bool>(more_id)).unwrap_or(false);
libraries/viu/src/settings_browser.rs:2832:                                    ui.data_mut(|d| d.insert_temp(more_id, extended));
libraries/viu/src/settings_browser.rs:2944:    let mut value = ui.data(|d| d.get_temp::<f64>(key)).unwrap_or(current);
libraries/viu/src/settings_browser.rs:2950:                let escaped = ui.input(|i| i.key_pressed(egui::Key::Escape));
libraries/viu/src/settings_browser.rs:2964:                    ui.data_mut(|d| d.remove::<f64>(key));
libraries/viu/src/settings_browser.rs:2967:                    ui.data_mut(|d| d.insert_temp(key, value));
libraries/viu/src/settings_browser.rs:2977:                        ui.data_mut(|d| d.insert_temp(key, value));
libraries/viu/src/settings_browser.rs:2988:    ui.data_mut(|d| d.remove::<f64>(key));
libraries/viu/src/settings_browser.rs:3010:                .data_mut(|d| d.get_temp::<String>(query_id))
libraries/viu/src/settings_browser.rs:3018:                    .data_mut(|d| d.insert_temp(query_id, query.clone()));
libraries/viu/src/settings_browser.rs:3182:    let held: Option<f64> = ui.data(|d| d.get_temp(id));
libraries/viu/src/settings_browser.rs:3248:        ui.data_mut(|d| d.insert_temp(id, at(pos)));
libraries/viu/src/settings_browser.rs:3253:        let delta = ui.input(|i| {
libraries/viu/src/settings_browser.rs:3267:        out = ui.data_mut(|d| d.remove_temp::<f64>(id));
libraries/viu/src/settings_browser.rs:3383:        .data(|d| d.get_temp::<Color32>(held_id))
libraries/viu/src/settings_browser.rs:3395:        ui.data_mut(|d| d.insert_temp(held_id, picking));
libraries/viu/src/settings_browser.rs:3397:    if ui.input(|i| i.pointer.any_released())
libraries/viu/src/settings_browser.rs:3398:        && let Some(held) = ui.data(|d| d.get_temp::<Color32>(held_id))
libraries/viu/src/settings_browser.rs:3401:        ui.data_mut(|d| d.remove_temp::<Color32>(held_id));
libraries/viu/src/settings_browser.rs:3412:        .data(|d| d.get_temp::<String>(hex_id))
libraries/viu/src/settings_browser.rs:3426:            ui.data_mut(|d| d.insert_temp(hex_id, value.clone()));
libraries/viu/src/settings_browser.rs:3428:        if edit.lost_focus() && ui.data(|d| d.get_temp::<String>(hex_id)).is_some() {
libraries/viu/src/settings_browser.rs:3429:            ui.data_mut(|d| d.remove_temp::<String>(hex_id));
libraries/viu/src/settings_browser.rs:3449:        ui.data_mut(|d| {
libraries/viu/src/settings_browser.rs:3464:        .data(|d| d.get_temp::<Color32>(held_id))
libraries/viu/src/settings_browser.rs:3471:        ui.data_mut(|d| d.insert_temp(held_id, picking));
libraries/viu/src/settings_browser.rs:3475:    if ui.data(|d| d.get_temp::<Color32>(held_id))
libraries/viu/src/settings_browser.rs:3476:        != ui.data(|d| d.get_temp::<Color32>(hex_id.with("at")))
libraries/viu/src/settings_browser.rs:3478:        ui.data_mut(|d| {
libraries/viu/src/settings_browser.rs:3484:        .data(|d| d.get_temp::<String>(hex_id))
libraries/viu/src/settings_browser.rs:3498:            ui.data_mut(|d| {
libraries/viu/src/settings_browser.rs:3508:        ui.data_mut(|d| {
libraries/viu/tests/ports_round5.rs:72:            ctx.run_ui(
libraries/viu/src/terminal/session_actor_tests.rs:205:                options: broker.options(),
libraries/viu/src/app_host.rs:168:        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
libraries/viu/src/app_host.rs:177:        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/src/app_host.rs:314:        ctx.request_repaint();
libraries/viu/src/app_host.rs:372:        let style = ctx.global_style();
libraries/viu/src/app_host.rs:541:                ctx.request_repaint();
libraries/viu/src/app_host.rs:734:                        ctx.request_repaint();
libraries/viu/src/app_host.rs:776:            && ctx.input(|i| i.viewport().close_requested())
libraries/viu/src/app_host.rs:778:            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
libraries/viu/src/app_host.rs:937:                    ctx.copy_text(request["body"]["text"].as_str().unwrap_or("").into());
libraries/viu/src/app_host.rs:972:            ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaste);
libraries/viu/src/app_host.rs:975:            ctx.request_repaint_after(at.saturating_duration_since(Instant::now()));
libraries/viu/src/app_host.rs:1045:            ctx.request_repaint_after(at.saturating_duration_since(Instant::now()));
libraries/viu/src/app_host.rs:1053:                ctx.request_repaint();
libraries/viu/src/app_host.rs:1113:                        ctx.copy_text(text.into());
libraries/viu/src/app_host.rs:1145:                    ctx.copy_image(image);
libraries/viu/src/app_host.rs:1180:            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
libraries/viu/src/app_host.rs:1484:                                            ctx.send_viewport_cmd(egui::ViewportCommand::Title(
libraries/viu/src/app_host.rs:1487:                                            ctx.send_viewport_cmd(
libraries/viu/src/app_host.rs:1509:                                    ctx.request_repaint();
libraries/viu/src/app_host.rs:1628:            ctx.request_repaint_after(deadline.saturating_duration_since(Instant::now()));
libraries/viu/src/app_host.rs:1694:                ctx.request_repaint();
libraries/viu/src/app_host.rs:1861:                    ctx.request_repaint_after(at.saturating_duration_since(Instant::now()));
libraries/viu/src/app_host.rs:1873:                ctx.request_repaint();
libraries/viu/src/app_host.rs:1884:                ctx.request_repaint();
libraries/viu/src/app_host.rs:1893:                    ctx.request_repaint();
libraries/viu/src/app_host.rs:1918:            ctx.request_repaint();
libraries/viu/src/app_host.rs:1934:            ctx.request_repaint_after(if wait.is_zero() { tick } else { wait });
libraries/viu/src/app_host.rs:1938:        let rect = ui.ctx().content_rect();
libraries/viu/src/app_host.rs:1939:        let viewport = [rect.width(), rect.height(), ui.ctx().pixels_per_point()];
libraries/viu/src/app_host.rs:2020:                ui.ctx().set_sublayer(ui.layer_id(), layer);
libraries/viu/src/app_host.rs:2021:                ui.ctx().set_transform_layer(
libraries/viu/src/app_host.rs:2045:                let time = ui.input(|i| i.time);
libraries/viu/src/app_host.rs:2052:                ui.ctx().request_repaint();
libraries/viu/src/app_host.rs:2085:                let keys=ui.input(|i|i.events.iter().filter_map(|e|match e{egui::Event::Key{key,pressed:true,repeat:false,modifiers,..}=>Some(json!({"key":crate::input::key_name(*key),"ctrl":modifiers.ctrl||modifiers.command,"alt":modifiers.alt,"shift":modifiers.shift,"typing":self.view.text_input_active()})),_=>None}).collect::<Vec<_>>());
libraries/viu/src/app_host.rs:2112:                ui.ctx().request_repaint();
libraries/viu/src/app_host.rs:2133:                    ui.ctx().request_repaint();
libraries/viu/src/app_host.rs:2143:                            .data(|d| d.get_temp::<bool>(egui::Id::unique("viu-editor-focused")))
libraries/viu/src/app_host.rs:2147:                            .data(|d| d.get_temp::<String>(egui::Id::unique("viu-focused-widget")));
libraries/viu/src/app_host.rs:2154:                            || ui.ctx().memory(|m| m.focused().is_some()))
libraries/viu/src/app_host.rs:2200:                    ui.ctx().request_repaint();
libraries/viu/src/app_host.rs:2207:                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
libraries/viu/src/app_host.rs:2214:            ui.ctx().content_rect(),
libraries/viu/src/app_host.rs:2228:            ui.ctx().request_repaint();
libraries/viu/src/app_host.rs:2833:        ctx.set_request_repaint_callback(move |_| repaint());
libraries/viu/src/app_host.rs:2838:            ctx.run_ui(
libraries/viu/src/app_host.rs:2981:            .viewport(viewport);
libraries/viu/src/app_host.rs:3227:                .run_ui(
libraries/viu/src/app_host.rs:3264:            let output = self.ctx.run_ui(
libraries/viu/src/app_host.rs:3330:        let output = ctx.run_ui(egui::RawInput::default(), |ui| host.pump(ui.ctx()));
libraries/viu/src/app_host.rs:3422:    Arc::new(move || ctx.request_repaint_of(egui::ViewportId::ROOT))
libraries/viu/src/app_host.rs:3449:            ctx.data(|d| d.get_temp::<String>(egui::Id::unique("viu-layer-output"))),
libraries/viu/src/app_host.rs:3498:            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-layer-output"), output.clone()));
libraries/viu/src/app_host.rs:3529:            if let Some(pos) = ui.input(|i| {
libraries/viu/src/app_host.rs:3549:                if ui.ctx().data(|d| d.get_temp::<Value>(key)).as_ref() != Some(&stamp) {
libraries/viu/src/app_host.rs:3558:                        ui.ctx().data_mut(|d| d.insert_temp(key, stamp));
libraries/viu/src/app_host.rs:3631:            ctx.request_repaint();
libraries/viu/src/app_host.rs:3646:            ctx.run_ui(Default::default(), |_| {})
libraries/viu/src/app_host.rs:3653:        ctx.set_request_repaint_callback(move |_| {
libraries/viu/src/app_host.rs:3669:                    ctx.request_repaint();
libraries/viu/src/app_host.rs:3702:            ctx.run_ui(egui::RawInput::default(), |_| {})
libraries/viu/src/app_host.rs:3708:        ctx.set_request_repaint_callback(move |request| {
libraries/viu/src/app_host.rs:3717:        ctx.run_ui(input, |ui| {
libraries/viu/src/app_host.rs:3718:            assert_eq!(ui.ctx().viewport_id(), child);
libraries/viu/tests/ports_round2.rs:59:        ctx.run_ui(
libraries/viu/tests/gallery_contract.rs:74:        ctx.run_ui(
libraries/viu/tests/deck_spans.rs:23:            let mut output = ctx.run_ui(raw, |ui| {
libraries/viu/tests/deck_spans.rs:69:    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/deck_spans.rs:146:        let mut out = ctx.run_ui(
libraries/viu/tests/deck_spans.rs:183:    let mut out = ctx.run_ui(
libraries/viu/tests/deck_spans.rs:212:            let rect = ui.ctx().content_rect();
libraries/viu/tests/deck_spans.rs:237:    ctx.set_pixels_per_point(2.4);
libraries/viu/tests/deck_spans.rs:239:        ctx.run_ui(
libraries/viu/tests/deck_spans.rs:256:    let backdrop_builds = ctx.data(|d| {
libraries/viu/tests/deck_spans.rs:268:        let _ = ctx.tessellate(std::mem::take(&mut out.shapes), out.pixels_per_point);
libraries/viu/tests/deck_spans.rs:291:        ctx.data(|d| d
libraries/viu/tests/deck_spans.rs:306:    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/deck_spans.rs:324:        let mut out = ctx.run_ui(
libraries/viu/tests/deck_spans.rs:379:        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/tests/deck_spans.rs:405:        ctx.run_ui(egui::RawInput::default(), |ui| {
libraries/viu/src/capture.rs:57:        let scale = ctx.pixels_per_point();
libraries/viu/src/capture.rs:58:        let pixels = ctx.input(|i| i.viewport().inner_rect.map(|r| r.area() * scale.powi(2)));
libraries/viu/src/capture.rs:80:        ctx.send_viewport_cmd(ViewportCommand::Screenshot(ScreenshotCallback::new(
libraries/viu/src/capture.rs:90:        ctx.request_repaint();
libraries/viu/src/capture.rs:106:        ctx.request_repaint_after(pending.deadline.saturating_duration_since(Instant::now()));
libraries/viu/src/window_frame.rs:29:    let key = egui::Id::unique((ctx.viewport_id(), "viu-native-title"));
libraries/viu/src/window_frame.rs:30:    let changed = ctx.data_mut(|data| {
libraries/viu/src/window_frame.rs:40:        ctx.send_viewport_cmd(Command::Title(title.to_owned()));
libraries/viu/src/window_frame.rs:49:        let output = ctx.run_ui(Default::default(), |ui| {
libraries/viu/src/window_frame.rs:93:            let output = ctx.run_ui(Default::default(), |ui| super::set_title(ui.ctx(), title));
libraries/viu/src/window_frame.rs:124:    ctx.data(|d| d.get_temp::<bool>(egui::Id::unique("viu-window-client")))
libraries/viu/src/window_frame.rs:130:            let vp = ctx.viewport_id();
libraries/viu/src/window_frame.rs:131:            ctx.data_mut(|d| d.insert_temp(egui::Id::unique(("eframe.window-menu", vp)), position));
libraries/viu/src/window_frame.rs:160:                let output = ctx.run_ui(Default::default(), |ui| {
libraries/viu/src/window_frame.rs:162:                        ui.ctx().data_mut(|d| {
libraries/viu/src/window_frame.rs:186:            let id = egui::Id::unique(("eframe.window-menu", ctx.viewport_id()));
libraries/viu/src/window_frame.rs:187:            assert_eq!(ctx.data(|d| d.get_temp::<egui::Pos2>(id)), Some(position));
libraries/viu/src/window_frame.rs:201:            ctx.data_mut(|d| {
libraries/viu/src/window_frame.rs:203:                    egui::Id::unique(("eframe.window-menu", ctx.viewport_id())),
libraries/viu/src/window_frame.rs:226:        ctx.data(|d| d.get_temp::<Self>(egui::Id::unique("viu-window-metrics")))
libraries/viu/src/window_frame.rs:302:            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-caption-alignment"), caption_align));
libraries/viu/src/window_frame.rs:317:            ui.ctx().send_viewport_cmd(Command::Decorations(server));
libraries/viu/src/window_frame.rs:321:            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-window-client"), self.client));
libraries/viu/src/window_frame.rs:376:            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-caption-alignment"), alignment));
libraries/viu/src/window_frame.rs:418:        let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
libraries/viu/src/window_frame.rs:422:            ui.ctx().send_viewport_cmd(Command::Maximized(!maximized));
libraries/viu/src/window_frame.rs:424:            ui.ctx().send_viewport_cmd(Command::StartDrag);
libraries/viu/src/window_frame.rs:469:            || ui.input(|i| {
libraries/viu/src/window_frame.rs:470:                i.viewport().maximized.unwrap_or(false) || i.viewport().fullscreen.unwrap_or(false)
libraries/viu/src/window_frame.rs:476:        let r = ui.ctx().content_rect();
libraries/viu/src/window_frame.rs:537:                ui.ctx().send_viewport_cmd(Command::BeginResize(direction));
libraries/viu/src/window_frame.rs:560:    let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
libraries/viu/src/window_frame.rs:657:                ui.ctx().send_viewport_cmd(if name == "minimize" {
libraries/viu/src/window_frame.rs:673:        "minimize" => ctx.send_viewport_cmd(Command::Minimized(true)),
libraries/viu/src/window_frame.rs:674:        "toggle-maximize" => ctx.send_viewport_cmd(Command::Maximized(
libraries/viu/src/window_frame.rs:675:            !ctx.input(|i| i.viewport().maximized.unwrap_or(false)),
libraries/viu/tests/deck.rs:131:        let out = ctx.run_ui(
libraries/viu/tests/deck.rs:328:    let output = ctx.run_ui(
libraries/viu/tests/deck.rs:465:        let mut out = ctx.run_ui(
libraries/viu/tests/email_frame.rs:366:        ctx.run_ui(
libraries/viu/tests/email_frame.rs:408:    ctx.run_ui(
libraries/viu/tests/email_frame.rs:435:    ctx.run_ui(
libraries/viu/tests/email_frame.rs:456:        ctx.run_ui(
libraries/viu/tests/email_frame.rs:540:        ctx.run_ui(
libraries/viu/tests/email_frame.rs:596:        ctx.run_ui(
libraries/viu/src/saved_state.rs:167:                ctx.input(|i| i.pointer.any_down()),
libraries/viu/tests/ced_bus.rs:347:                ctx.set_pixels_per_point(scale);
libraries/viu/tests/ced_bus.rs:356:                let mut output = ctx.run_ui(
libraries/viu/tests/ced_bus.rs:430:        ctx.set_pixels_per_point(scale);
libraries/viu/tests/ced_bus.rs:438:        let mut output = ctx.run_ui(
libraries/viu/tests/ced_bus.rs:871:    ctx.run_ui(
libraries/viu/tests/monitor_signal.rs:188:    ctx.run_ui(
libraries/viu/src/numbers.rs:25:        let now = ui.input(|i| i.time);
libraries/viu/src/numbers.rs:41:                    .request_repaint_after(std::time::Duration::from_secs_f64(
libraries/viu/src/numbers.rs:45:                ui.ctx().request_repaint();
libraries/viu/src/view.rs:500:            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-editor-focused"), false));
libraries/viu/src/view.rs:544:                .data_mut(|d| d.remove::<String>(crate::widgets::focus_key()));
libraries/viu/src/view.rs:561:                    .fixed_pos(ui.ctx().content_rect().left_top())
libraries/viu/src/view.rs:579:                .data(|d| d.get_temp::<String>(crate::widgets::focus_key()));
libraries/viu/src/view.rs:580:            let zoom = ui.ctx().zoom_factor();
libraries/viu/src/view.rs:740:        let start = ui.ctx().graphics_mut(|g| g.entry(ui.layer_id()).next_idx());
libraries/viu/src/view.rs:815:                .layer_transform_to_global(layer)
libraries/viu/src/view.rs:893:                    ui.spacing_mut().item_spacing=ui.ctx().global_style().spacing.item_spacing;
libraries/viu/src/view.rs:907:                        let fill=if node.ports.get("surface").and_then(Value::as_str)==Some("editor") {ui.ctx().data(|d|d.get_temp::<editor::Palette>(egui::Id::unique("viu-gold-editor-palette"))).map(|p|p.background).unwrap_or(ui.visuals().panel_fill)} else if content.light {toolkit::chrome::Chrome::for_theme(toolkit::content::light_theme()).palette.canvas}else{ui.visuals().panel_fill};
libraries/viu/src/view.rs:914:                                    if ui.memory(|m|m.has_focus(egui::Id::unique(target))) {
libraries/viu/src/view.rs:917:                                        let step=ui.input_mut(|i|{
libraries/viu/src/view.rs:925:                                            if let Some(id)=ui.ctx().data(|d|d.get_temp::<egui::Id>(marker)) {
libraries/viu/src/view.rs:932:                                ui.ctx().data_mut(|d|d.insert_temp(marker,shown.id));
libraries/viu/src/view.rs:937:                        ui.spacing_mut().item_spacing=ui.ctx().global_style().spacing.item_spacing;
libraries/viu/src/view.rs:984:                            ui.spacing_mut().item_spacing=ui.ctx().global_style().spacing.item_spacing;
libraries/viu/src/view.rs:1034:                        let groups=band_groups(node,crate::window_frame::Metrics::of(ui.ctx()).caption_width*3.0,ui.ctx().data(|d| d.get_temp::<design::title_band::Alignment>(egui::Id::unique("viu-caption-alignment"))).unwrap_or_else(|| toolkit::chrome::Chrome::of(ui.ctx()).captions.alignment(design::title_band::Alignment::Right)),Some(ui.ctx()),&measure).expect("validated band groups");
libraries/viu/src/view.rs:1063:                        let spacing=ui.ctx().global_style().spacing.item_spacing;
libraries/viu/src/view.rs:1156:                            ui.spacing_mut().item_spacing=ui.ctx().global_style().spacing.item_spacing;
libraries/viu/src/view.rs:1251:                                                    ui.ctx().data(|d|d.get_temp::<f32>(row_id.with(("fixed-width",child)))).unwrap_or(0.)
libraries/viu/src/view.rs:1261:                                            let height = ui.ctx().data(|d|d.get_temp::<f32>(cell)).unwrap_or(0.);
libraries/viu/src/view.rs:1266:                                                let previous = ui.ctx().data(|d|d.get_temp::<f32>(width_key)).unwrap_or(0.);
libraries/viu/src/view.rs:1268:                                                    ui.ctx().data_mut(|d|d.insert_temp(width_key,measured));
libraries/viu/src/view.rs:1269:                                                    ui.ctx().request_repaint();
libraries/viu/src/view.rs:1285:                                                ui.ctx().data_mut(|d|d.insert_temp(cell,actual));
libraries/viu/src/view.rs:1286:                                                ui.ctx().request_repaint();
libraries/viu/src/view.rs:1407:                        .is_some_and(|width| ui.ctx().content_rect().width() < width)
libraries/viu/src/view.rs:1438:            ui.set_clip_rect(ui.ctx().content_rect());
libraries/viu/src/view.rs:1442:        if node.ports.get("keyboard") == Some(&Value::Bool(true)) && !ui.ctx().text_edit_focused() {
libraries/viu/src/view.rs:1447:            let marker = egui::Id::unique((ui.ctx().viewport_id(), key, "pane-model-focus"));
libraries/viu/src/view.rs:1449:            let previous_model = ui.ctx().data_mut(|d| {
libraries/viu/src/view.rs:1459:                        ui.memory_mut(|m| m.request_focus(egui::Id::unique(target)));
libraries/viu/src/view.rs:1470:            let previous = ui.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab));
libraries/viu/src/view.rs:1472:                !previous && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab));
libraries/viu/src/view.rs:1480:                        ui.memory_mut(|m| m.request_focus(egui::Id::unique(target)));
libraries/viu/src/view.rs:1489:                            ui.memory(|m| m.has_focus(egui::Id::unique(target)))
libraries/viu/src/view.rs:1566:            ui.ctx().viewport_id(),
libraries/viu/src/view.rs:1591:                ui.ctx().viewport_id(),
libraries/viu/src/view.rs:1705:            egui::Id::unique((ui.ctx().viewport_id(), key, "divider")),
libraries/viu/src/view.rs:1710:            ui.ctx().set_cursor_icon(if horizontal {
libraries/viu/src/view.rs:1763:            if ui.input(|i| i.pointer.primary_pressed()) && ui.rect_contains_pointer(area) {
libraries/viu/src/view.rs:1766:            if ui.memory(|m| m.has_focus(id)) {
libraries/viu/src/view.rs:1767:                ui.memory_mut(|m| {
libraries/viu/src/view.rs:1925:                    egui::Id::unique((ui.ctx().viewport_id(), "viu.dock.intrinsic-height"));
libraries/viu/src/view.rs:1926:                ui.ctx().data_mut(|d| d.insert_temp(marker, limit));
libraries/viu/src/view.rs:1951:                ui.ctx().data_mut(|d| {
libraries/viu/src/view.rs:1961:                let marker = egui::Id::unique((ui.ctx().viewport_id(), key, child, "dock-height"));
libraries/viu/src/view.rs:1962:                let height = ui.ctx().data(|d| d.get_temp::<f32>(marker)).unwrap_or(0.0);
libraries/viu/src/view.rs:1974:                    ui.ctx().data_mut(|d| d.insert_temp(marker, measured));
libraries/viu/src/view.rs:1975:                    ui.ctx().request_repaint();
libraries/viu/src/view.rs:2005:                        + ui.ctx().global_style().spacing.item_spacing.y)
libraries/viu/src/view.rs:2154:                    egui::Id::unique((ui.ctx().viewport_id(), key, child, "resize")),
libraries/viu/src/view.rs:2159:                    ui.ctx().set_cursor_icon(if horizontal {

```

## Changes

- app_host.rs:3624 callback snapshots contexts, matching toolkit 3ec87dba. Positive/negative concurrency tests at :3689/:3694.
- file_tiles.rs:98 test-only entry hook; :301 queued-writer fixture uses barrier then /proc task wchan to confirm futex blocking. No sleeps. Positive production helper child and negative restored nested-read child run under a bounded subprocess watchdog (:357).
- window_frame.rs:198 documents expected later debug detector noise in its pre-existing detached negative control.
- engines/viu/src/store_tests.rs:63 pins map write:true bad-theme refusal beside read-only map forms.
- engines/viud/tests/editor_app_test.mix:18 read-only fake rejects writes; single install step explicitly uses an admission-capable fake.
- No dependencies or MIX_* variables added. No deployment or PR merge performed; main was merged into this branch for round 2.

## Round 1 CBC evidence (historical)

- 20261122-072031-c3838b9b-a1: fmt generation, rc 0, initial head 2625f807.
- 20261122-072126-56131a80-a1: fmt generation, rc 0, 5bb996c8; applied worker diff locally, worker edits reverted in same job.
- 20261122-072202-1e216480-a1: fmt check passed; non-standard clippy -D warnings rejected existing warnings, rc 101. Superseded by normal-shape clippy below.
- 20261122-072235-abd2f18e-a1: normal clippy default + no-default all targets passed; native bad-theme store test passed (1.95 s, one ignored subprocess helper); editor fake initially rejected legitimate install admission (PR-introduced issue, fixed in 1eb79ea5). rc 1 overall.
- 20261122-072127-c8e2cd79-a1: all lib-viu targets, 8-wide pool, default then no-default, workshop_bus selected in the serial lane (zero tests: test-support was not enabled). Default configuration passed all 56 binaries (unit: 430 passed, 7 ignored). No-default failed only unchanged decorations.rs:300 (unit: 429 passed, 1 failed, 7 ignored). All new positive and negative controls passed in both. Rust tree identical to head apart from rustfmt.
- 20261122-072520-b8c6361b-a1: editor fake correction and doc tests both configurations passed, rc 0 (doc targets have zero tests).

## Round 1 stop (resolved by main PR #232)

Job 20261122-072127-c8e2cd79-a1, no-default features, normal 8-wide test pool: decorations::tests::silent_registry_times_out_within_the_reply_budget panicked at libraries/viu/src/decorations.rs:300:9: start.elapsed() < Duration::from_millis(250). This file has no changes from origin/main. Default passed the same test. No rerun, fix, saturation experiment or further CBC submission after observing the failure. PR kept draft.

app_mix passed in the 8-wide pool in both configurations (36.96 s / 38.35 s for the whole existing binary). No new app tests were added. Standalone per-app <=5 s budget was not measured before the stop. workshop_bus behavioural verification still needs --features test-support; the selected serial target exercised zero tests, so this is not claimed as a pass. Feature-gated targets such as design_validation also had zero tests. Follow-up validation must enable test-support before claiming complete coverage.
