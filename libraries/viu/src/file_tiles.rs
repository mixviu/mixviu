// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native file tiles and transfer controls. Labels, eligibility, paths and
//! actions are supplied by the caller; this renderer never opens a file.
use crate::widgets::Intent;
use egui::{RichText, Sense, Ui};
use serde_json::Value;
use toolkit::{Icon, button::IconButton};

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
pub fn validate(v: &Value) -> Result<(), String> {
    if !v["expanded"].is_boolean()
        || !v["items"].as_array().is_some_and(|items| {
            items.len() <= 1024
                && items.iter().all(|p| {
                    ["key", "name", "size", "label", "icon"]
                        .iter()
                        .all(|key| p[key].as_str().is_some_and(|s| s.len() <= 4096))
                        && Icon::from_name(text(p, "icon")).is_some()
                        && p["enabled"].is_boolean()
                        && p["save_only"].is_boolean()
                        && (p["note"].is_null()
                            || p["note"].as_str().is_some_and(|s| s.len() <= 8192))
                })
        })
        || ![
            "title",
            "count",
            "size",
            "actions",
            "save_all",
            "open_all",
            "save",
            "open",
            "save_only",
            "undecodable",
        ]
        .iter()
        .all(|key| v[key].as_str().is_some_and(|s| s.len() <= 4096))
    {
        return Err("file tiles require bounded items, eligibility and labels".into());
    }
    Ok(())
}
fn emit(events: &mut Vec<Intent>, id: &str, action: &str, key: Value) {
    events.push(Intent {
        id: id.into(),
        action: action.into(),
        value: key,
    });
}
pub(crate) fn menu(
    response: &egui::Response,
    p: &Value,
    v: &Value,
    id: &str,
    events: &mut Vec<Intent>,
) {
    if p["enabled"] == true && response.clicked() {
        emit(
            events,
            id,
            if p["save_only"] == true {
                "save"
            } else {
                "open"
            },
            p["key"].clone(),
        );
    }
    response.context_menu(|ui| {
        let open = ui.add_enabled(
            p["enabled"] == true && p["save_only"] != true,
            egui::Button::new(text(v, "open")),
        );
        let open = if p["save_only"] == true {
            open.on_disabled_hover_text(text(v, "save_only"))
        } else {
            open
        };
        if open.clicked() {
            emit(events, id, "open", p["key"].clone());
            ui.close();
        }
        if ui
            .add_enabled(p["enabled"] == true, egui::Button::new(text(v, "save")))
            .clicked()
        {
            emit(events, id, "save", p["key"].clone());
            ui.close();
        }
    });
}
fn intrinsic_height(ui: &Ui) -> f32 {
    let vp = ui.ctx().viewport_id();
    ui.ctx()
        .data(|d| {
            #[cfg(test)]
            tests::queue_writer(ui.ctx());
            d.get_temp::<f32>(egui::Id::unique((vp, "viu.dock.intrinsic-height")))
        })
        .unwrap_or(ui.available_height())
}
pub fn draw(ui: &mut Ui, id: &str, v: &Value, events: &mut Vec<Intent>) -> egui::Response {
    let limit = intrinsic_height(ui);
    ui.scope(|ui| {
        let start_y = ui.cursor().top();
        toolkit::bars::hairline(ui);
        let items = v["items"].as_array().unwrap();
        let mut toggle = false;
        egui::Sides::new().show(
            ui,
            |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            IconButton::new(if v["expanded"] == true {
                                Icon::ChevronDown
                            } else {
                                Icon::ChevronRight
                            })
                            .tooltip(text(v, "title")),
                        )
                        .clicked()
                    {
                        toggle = true;
                    }
                    ui.add(toolkit::icons::image_of(
                        ui.ctx(),
                        Icon::Paperclip,
                        ui.text_style_height(&egui::TextStyle::Body),
                        ui.visuals().text_color(),
                    ));
                    ui.label(text(v, "count"));
                    ui.weak(text(v, "size"));
                });
            },
            |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let more =
                        ui.add(IconButton::new(Icon::ChevronDown).tooltip(text(v, "actions")));
                    let any = items.iter().any(|p| p["enabled"] == true);
                    if ui
                        .add_enabled(any, egui::Button::new(text(v, "save_all")))
                        .clicked()
                    {
                        emit(events, id, "save-all", Value::Null);
                    }
                    egui::Popup::menu(&more).show(|ui| {
                        if ui
                            .add_enabled(any, egui::Button::new(text(v, "save_all")))
                            .clicked()
                        {
                            emit(events, id, "save-all", Value::Null);
                            ui.close();
                        }
                        if ui
                            .add_enabled(
                                items
                                    .iter()
                                    .any(|p| p["enabled"] == true && p["save_only"] != true),
                                egui::Button::new(text(v, "open_all")),
                            )
                            .clicked()
                        {
                            emit(events, id, "open-all", Value::Null);
                            ui.close();
                        }
                    });
                });
            },
        );
        if toggle {
            emit(events, id, "toggle", Value::Null);
        }
        if v["expanded"] == true {
            // The caller's limit covers the entire bar. Reserve its measured
            // controls and trailing space before sizing the tile scroll area.
            // Otherwise an exact bottom panel can clip those controls when
            // the tile list alone consumes the whole height budget.
            let trailing = ui.spacing().item_spacing.y * 2.0;
            let tiles_height = (limit - (ui.cursor().top() - start_y) - trailing).max(0.0);
            egui::ScrollArea::vertical()
                .id_salt("attachment-tiles")
                .max_height(tiles_height)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.with_layout(
                        egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true),
                        |ui| {
                            for p in items {
                                let body = ui.text_style_height(&egui::TextStyle::Body);
                                let frame = egui::Frame::group(ui.style());
                                let width = (egui::TextStyle::Body.resolve(ui.style()).size * 16.0)
                                    .min(ui.max_rect().width());
                                let shown = ui
                                    .allocate_ui_with_layout(
                                        egui::vec2(width, 0.0),
                                        egui::Layout::top_down(egui::Align::Min),
                                        |ui| {
                                            frame
                                                .show(ui, |ui| {
                                                    ui.set_width(
                                                        (width - frame.total_margin().sum().x)
                                                            .max(body),
                                                    );
                                                    ui.horizontal(|ui| {
                                                        let ink = if p["enabled"] == true {
                                                            ui.visuals().text_color()
                                                        } else {
                                                            ui.visuals().weak_text_color()
                                                        };
                                                        ui.add(toolkit::icons::image_of(
                                                            ui.ctx(),
                                                            Icon::from_name(text(p, "icon"))
                                                                .unwrap(),
                                                            body,
                                                            ink,
                                                        ));
                                                        ui.add(
                                                            egui::Label::new(
                                                                RichText::new(text(p, "name"))
                                                                    .color(ink),
                                                            )
                                                            .truncate(),
                                                        );
                                                    });
                                                    ui.weak(text(p, "size"));
                                                    if p["enabled"] != true {
                                                        ui.label(
                                                            RichText::new(text(v, "undecodable"))
                                                                .weak()
                                                                .small(),
                                                        );
                                                    }
                                                    if let Some(note) = p["note"].as_str() {
                                                        ui.add(
                                                            egui::Label::new(
                                                                RichText::new(note).weak().small(),
                                                            )
                                                            .wrap(),
                                                        );
                                                    }
                                                })
                                                .response
                                        },
                                    )
                                    .inner;
                                let response = ui
                                    .interact(shown.rect, shown.id.with("open"), Sense::click())
                                    .on_hover_text(text(p, "name"));
                                response.widget_info(|| {
                                    egui::WidgetInfo::labeled(
                                        egui::Role::Button,
                                        ui.is_enabled() && p["enabled"] == true,
                                        text(p, "label"),
                                    )
                                });
                                menu(&response, p, v, id, events);
                            }
                        },
                    );
                });
            ui.add_space(trailing);
        }
    })
    .response
}

pub fn intent(value: &Value, action: &str, payload: &Value) -> bool {
    match action {
        "toggle" | "save-all" | "open-all" => payload.is_null(),
        "open" | "save" => value["items"].as_array().is_some_and(|a| {
            a.iter().any(|p| {
                p["key"] == *payload
                    && p["enabled"] == true
                    && (action == "save" || p["save_only"] != true)
            })
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::BufRead,
        sync::{Arc, Barrier, mpsc},
        time::{Duration, Instant},
    };

    thread_local! {
        static CONTEND: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }

    // Called with the outer read held. A barrier starts the writer, then
    // Linux's wait channel or futex syscall proves it has queued on the lock.
    // Merely announcing an intent to write is insufficient for this test.
    pub(super) fn queue_writer(ctx: &egui::Context) {
        if !CONTEND.with(|flag| flag.replace(false)) {
            return;
        }
        let barrier = Arc::new(Barrier::new(2));
        let start = barrier.clone();
        let writer = ctx.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            start.wait();
            // Rust 2024: libc declares gettid as an unsafe extern function.
            // SAFETY: gettid has no arguments or memory preconditions.
            tx.send(unsafe { libc::gettid() }).unwrap();
            writer.data_mut(|_| {});
        });
        barrier.wait();
        let tid = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        let task = format!("/proc/{}/task/{tid}", std::process::id());
        loop {
            let state = std::fs::read_to_string(format!("{task}/wchan"));
            let syscall = std::fs::read_to_string(format!("{task}/syscall"));
            assert!(
                state.is_ok() || syscall.is_ok(),
                "cannot inspect queued writer: wchan={state:?}, syscall={syscall:?}"
            );
            let in_futex = syscall
                .as_ref()
                .ok()
                .and_then(|s| s.split_whitespace().next()?.parse::<libc::c_long>().ok())
                == Some(libc::SYS_futex);
            if state.as_ref().is_ok_and(|s| s.contains("futex")) || in_futex {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "writer did not queue: wchan={state:?}, syscall={syscall:?}"
            );
            std::thread::yield_now();
        }
        println!("WRITER_QUEUED");
    }

    fn contending_frame(nested: bool) {
        let ctx = egui::Context::default();
        ctx.run_ui(Default::default(), |ui| {
            let vp = ui.ctx().viewport_id();
            ui.ctx().data_mut(|d| {
                d.insert_temp(
                    egui::Id::unique((vp, "viu.dock.intrinsic-height")),
                    90.0_f32,
                )
            });
            CONTEND.with(|flag| flag.set(true));
            if nested {
                // This control depends on viewport_id() taking the Context
                // read lock; making it lock-free must fail the control loudly.
                ui.ctx().data(|d| {
                    queue_writer(ui.ctx());
                    d.get_temp::<f32>(egui::Id::unique((
                        ui.ctx().viewport_id(),
                        "viu.dock.intrinsic-height",
                    )))
                });
            } else {
                assert_eq!(super::intrinsic_height(ui), 90.0);
            }
        })
        .drop_without_applying_deltas();
    }

    // Isolate the deliberate deadlock so the watchdog can reap it before
    // epaint's ten-second detector fires; no blocked thread survives the test.
    fn contending_child(name: &str, should_finish: bool) {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", name, "--ignored", "--nocapture"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            for line in std::io::BufReader::new(stdout).lines() {
                if line.unwrap().contains("WRITER_QUEUED") {
                    let _ = tx.send(());
                }
            }
        });
        let queued = rx.recv_timeout(Duration::from_secs(2));
        let deadline = Instant::now()
            + if should_finish {
                Duration::from_secs(1)
            } else {
                Duration::from_millis(100)
            };
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break Some(status);
            }
            if Instant::now() >= deadline {
                break None;
            }
            std::thread::yield_now();
        };
        if status.is_none() {
            child.kill().unwrap();
        }
        let output = child.wait_with_output().unwrap();
        reader.join().unwrap();
        assert!(
            queued.is_ok(),
            "writer never queued: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if should_finish {
            assert!(
                status.is_some_and(|s| s.success()),
                "lookup blocked or failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        } else {
            assert!(
                status.is_none(),
                "nested read unexpectedly returned: {status:?}"
            );
        }
    }

    #[test]
    fn viewport_height_with_queued_writer() {
        contending_child("file_tiles::tests::queued_writer_child", true);
    }
    #[test]
    fn nested_read_with_queued_writer_negative_control() {
        contending_child("file_tiles::tests::nested_read_child", false);
    }
    #[test]
    #[ignore = "subprocess fixture"]
    fn queued_writer_child() {
        contending_frame(false);
    }
    #[test]
    #[ignore = "subprocess negative control"]
    fn nested_read_child() {
        contending_frame(true);
    }

    #[test]
    fn reads_viewport_height_without_nested_context_lock() {
        let (tx, rx) = std::sync::mpsc::channel();
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let ctx = egui::Context::default();
            let output = ctx.run_ui(Default::default(), |ui| {
                let vp = ui.ctx().viewport_id();
                ui.ctx().data_mut(|d| {
                    d.insert_temp(
                        egui::Id::unique((vp, "viu.dock.intrinsic-height")),
                        90.0_f32,
                    )
                });
                started_tx.send(()).unwrap();
                assert_eq!(super::intrinsic_height(ui), 90.0);
            });
            output.drop_without_applying_deltas();
            let _ = tx.send(());
        });
        started_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("file tile frame did not start");
        rx.recv_timeout(std::time::Duration::from_secs(2))
            .expect("file tile height lookup did not return successfully");
    }
}
