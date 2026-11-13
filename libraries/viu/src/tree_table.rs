// SPDX-License-Identifier: MIT OR Apache-2.0
//! Renderer-neutral virtual tree/table. Rows, selection, commands and sorting
//! belong to the application; this control only draws and emits intents.
use crate::widgets::{Intent, Widget};
use egui::{Align2, FontId, Id, Rect, Sense, TextStyle, Ui, pos2, vec2};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};

fn table_rows(value: &Value) -> &[Value] {
    value["rows"].as_array().map(Vec::as_slice).unwrap_or(&[])
}

#[derive(Clone)]
struct SampleWidthCache {
    key: u64,
    width: f32,
}

#[cfg(test)]
thread_local! {
    static SAMPLE_LAYOUTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static UNCACHED_SAMPLES: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn number(v: &Value, key: &str, default: f32) -> f32 {
    v[key].as_f64().unwrap_or(default as f64) as f32
}
fn sample_width(ui: &Ui, id: &str, column: &Value, rows: &[Value], index: usize) -> Option<f32> {
    let samples = column["samples"].as_array()?;
    let body = TextStyle::Body.resolve(ui.style());
    let strong = toolkit::fonts::bound(ui.ctx(), toolkit::fonts::medium(body.size));
    #[cfg(test)]
    if UNCACHED_SAMPLES.get() {
        return Some(measure_sample_width(
            ui, column, samples, rows, index, &body, &strong,
        ));
    }
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    body.hash(&mut hash);
    strong.hash(&mut hash);
    ui.ctx().pixels_per_point().to_bits().hash(&mut hash);
    ui.spacing().item_spacing.x.to_bits().hash(&mut hash);
    // Family names alone do not detect replacement faces under the same name.
    ui.fonts(|fonts| {
        let definitions = fonts.definitions();
        for family in [&body.family, &strong.family] {
            if let Some(names) = definitions.families.get(family) {
                for name in names {
                    name.hash(&mut hash);
                    if let Some(data) = definitions.font_data.get(name) {
                        std::sync::Arc::as_ptr(data).hash(&mut hash);
                    }
                }
            }
        }
    });
    for value in std::iter::once(text(column, "label"))
        .chain(samples.iter().filter_map(Value::as_str))
        .chain(rows.iter().filter_map(|row| row["cells"][index].as_str()))
    {
        value.hash(&mut hash);
    }
    let key = hash.finish();
    let cache_id = Id::unique((ui.ctx().viewport_id(), id, index, "sample-width"));
    if let Some(cache) = ui.data(|data| data.get_temp::<SampleWidthCache>(cache_id)) {
        if cache.key == key {
            return Some(cache.width);
        }
    }
    let width = measure_sample_width(ui, column, samples, rows, index, &body, &strong);
    ui.data_mut(|data| data.insert_temp(cache_id, SampleWidthCache { key, width }));
    Some(width)
}

fn measure_sample_width(
    ui: &Ui,
    column: &Value,
    samples: &[Value],
    rows: &[Value],
    index: usize,
    body: &FontId,
    strong: &FontId,
) -> f32 {
    let width = std::iter::once(text(column, "label"))
        .chain(samples.iter().filter_map(Value::as_str))
        .chain(rows.iter().filter_map(|row| row["cells"][index].as_str()))
        .map(|text| {
            [body.clone(), strong.clone()]
                .into_iter()
                .map(|font| {
                    #[cfg(test)]
                    SAMPLE_LAYOUTS.set(SAMPLE_LAYOUTS.get() + 1);
                    ui.painter()
                        .layout_no_wrap(text.into(), font, ui.visuals().text_color())
                        .size()
                        .x
                })
                .fold(0.0, f32::max)
        })
        .fold(0.0, f32::max);
    width + ui.spacing().item_spacing.x + 2.0
}
fn native_table_state(value: &Value) -> Result<toolkit::datatable::TableState, String> {
    if value.is_null() {
        return Ok(Default::default());
    }
    if !value.is_object() {
        return Err("table state must be an object".into());
    }
    let mut value = value.clone();
    for key in ["page", "page_size", "focus"] {
        if let Some(number) = value.get(key).filter(|n| !n.is_null()) {
            let number = crate::widgets::whole(number)
                .ok_or_else(|| format!("table {key} must be a whole number"))?;
            if key == "page_size" && (number == 0 || number > u32::MAX as u64) {
                return Err("invalid table page size".into());
            }
            value[key] = json!(number);
        }
    }
    serde_json::from_value(value).map_err(|e| format!("invalid table state: {e}"))
}

#[cfg(test)]
mod numeric_state_tests {
    use super::*;

    #[test]
    fn sampled_columns_cache_layout_and_invalidate_on_render_inputs() {
        let ctx = egui::Context::default();
        crate::appearance::install(&ctx);
        let mut value = json!({"key":"cache-test","columns":[{"id":"date","label":"Date","samples":["Wed 23:58"]}],"rows":[{"key":"a","cells":["13:51"]}]});
        for stage in 0..7 {
            if stage == 2 {
                value["rows"][0]["cells"][0] = json!("A much wider value");
            }
            if stage == 3 {
                value["columns"][0]["samples"] = json!(["An even wider sample value"]);
            }
            if stage == 4 {
                ctx.set_pixels_per_point(2.5);
            }
            if stage == 5 {
                ctx.style_mut_of(egui::Theme::Dark, |style| {
                    style
                        .text_styles
                        .insert(TextStyle::Body, FontId::monospace(24.0));
                });
            }
            if stage == 6 {
                let mut definitions = egui::FontDefinitions::default();
                definitions.families.insert(
                    egui::FontFamily::Name("medium".into()),
                    definitions.families[&egui::FontFamily::Proportional].clone(),
                );
                ctx.set_fonts(definitions);
            }
            SAMPLE_LAYOUTS.set(0);
            ctx.run_ui(egui::RawInput::default(), |ui| {
                sample_width(
                    ui,
                    "cache-test",
                    &value["columns"][0],
                    table_rows(&value),
                    0,
                );
            })
            .drop_without_applying_deltas();
            if stage == 1 {
                assert_eq!(SAMPLE_LAYOUTS.get(), 0);
            } else {
                assert!(SAMPLE_LAYOUTS.get() > 0, "stage {stage}");
            }
        }
    }

    #[test]
    fn table_renderers_tolerate_missing_or_malformed_rows() {
        let ctx = egui::Context::default();
        crate::appearance::install(&ctx);
        for rows in [None, Some(json!(null)), Some(json!("bad")), Some(json!({}))] {
            for sampled in [false, true] {
                let mut value = json!({"key":"empty","columns":[{"id":"date","label":"Date"}]});
                if sampled {
                    value["columns"][0]["samples"] = json!(["13:51"]);
                }
                if let Some(rows) = &rows {
                    value["rows"] = rows.clone();
                }
                // Admission still refuses malformed scenes; direct rendering is defensive too.
                assert!(validate(&value).is_err());
                ctx.run_ui(egui::RawInput::default(), |ui| {
                    let widget = Widget::new("empty", "tree-table", "", value.clone());
                    draw_gold(ui, &widget, &value, &mut Vec::new());
                    draw_grid(ui, &widget, &value, &mut BTreeMap::new(), &mut Vec::new());
                })
                .drop_without_applying_deltas();
            }
        }
    }

    #[test]
    fn sampled_table_steady_frame_measurement() {
        // Same full native table draw, with the pre-cache measurement path as
        // baseline. No GPU/presentation or application publication is included.
        for count in [500, 2000] {
            for scale in [1.0, 2.5] {
                let rows = (0..count).map(|i| json!({"key":format!("r{i}"),"cells":[format!("Message {i:04}"),format!("Sender {i:04}"),format!("{:02}/09/2025",i%30+1)],"strong":i%2==0})).collect::<Vec<_>>();
                let value = json!({"key":"bench","rows":rows,"columns":[
                    {"id":"subject","label":"Subject","samples":["Message 2000"]},
                    {"id":"from","label":"From","samples":["Sender 2000"]},
                    {"id":"date","label":"Date","samples":["Wed 23:58"]}]});
                let widget = Widget::new("bench", "tree-table", "", value.clone());
                for uncached in [true, false] {
                    let ctx = egui::Context::default();
                    crate::appearance::install(&ctx);
                    ctx.set_pixels_per_point(scale);
                    UNCACHED_SAMPLES.set(uncached);
                    let mut times = Vec::new();
                    let mut steady_layouts = 0;
                    for frame in 0..24 {
                        SAMPLE_LAYOUTS.set(0);
                        let began = std::time::Instant::now();
                        ctx.run_ui(
                            egui::RawInput {
                                screen_rect: Some(Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    vec2(3840.0 / scale, 2160.0 / scale),
                                )),
                                ..Default::default()
                            },
                            |ui| {
                                draw_gold(ui, &widget, &value, &mut Vec::new());
                            },
                        )
                        .drop_without_applying_deltas();
                        if frame >= 3 {
                            times.push(began.elapsed().as_secs_f64() * 1000.0);
                            steady_layouts += SAMPLE_LAYOUTS.get();
                        }
                    }
                    if uncached {
                        assert!(steady_layouts >= 21 * count * 6);
                    } else {
                        assert_eq!(steady_layouts, 0);
                    }
                    times.sort_by(f64::total_cmp);
                    eprintln!(
                        "sampled table rows={count} scale={scale} uncached={uncached}: median_ms={:.3} p95_ms={:.3} measured_layouts={steady_layouts}",
                        times[10], times[19]
                    );
                    UNCACHED_SAMPLES.set(false);
                }
            }
        }
    }

    #[test]
    fn email_date_measurement_fits_current_rows_at_desktop_scales() {
        for scale in [1.0, 2.5] {
            let ctx = egui::Context::default();
            crate::appearance::install(&ctx);
            ctx.set_pixels_per_point(scale);
            let column =
                json!({"label":"Date","samples":["23:58","Wed 23:58","28 May","28/05/2025"]});
            let rows = [
                "3 Nov",
                "28 Oct",
                "13:51",
                "Wed 23:59",
                "30/09/2025",
                "A wider formatted date",
            ]
            .map(|date| json!({"cells":[date]}));
            ctx.run_ui(egui::RawInput::default(), |ui| {
                let width = sample_width(ui, "email", &column, &rows, 0).unwrap();
                let body = TextStyle::Body.resolve(ui.style());
                for font in [
                    body.clone(),
                    toolkit::fonts::bound(ui.ctx(), toolkit::fonts::medium(body.size)),
                ] {
                    for row in &rows {
                        let galley = ui.painter().layout_no_wrap(
                            row["cells"][0].as_str().unwrap().into(),
                            font.clone(),
                            ui.visuals().text_color(),
                        );
                        // Native table cells inset half the item gap on each side.
                        assert!(galley.size().x <= width - ui.spacing().item_spacing.x);
                    }
                }
            })
            .drop_without_applying_deltas();
        }
    }

    #[test]
    fn scene_numbers_preserve_selection_page_and_focus() {
        let state = native_table_state(&json!({
            "page": 2.0, "page_size": 100.0, "focus": 3.0,
            "selection": ["message-4"], "sort": {"column":"date", "order":"descending"}
        }))
        .unwrap();
        assert_eq!(state.page, 2);
        assert_eq!(state.page_size, 100);
        assert_eq!(state.focus, Some(3));
        assert_eq!(state.selection, ["message-4"]);
        assert!(state.sort.is_some());
        for invalid in [
            json!({"page":1.5}),
            json!({"page_size":0.0}),
            json!({"focus":-1.0}),
        ] {
            assert!(native_table_state(&invalid).is_err());
        }
    }
}
fn emit(events: &mut Vec<Intent>, id: &str, action: &str, value: Value) {
    events.push(Intent {
        id: id.into(),
        action: action.into(),
        value,
    });
}

pub fn validate(value: &Value) -> Result<(), String> {
    if text(value, "key").is_empty() {
        return Err("tree-table requires a container key".into());
    }
    let rows = value["rows"].as_array().ok_or("tree-table requires rows")?;
    let columns = value["columns"]
        .as_array()
        .filter(|c| !c.is_empty() && c.len() <= 16)
        .ok_or("tree-table requires 1..16 columns")?;
    let mut keys = BTreeSet::new();
    native_table_state(&value["state"])?;
    if !value["total"].is_null() && crate::widgets::whole(&value["total"]).is_none() {
        return Err("table total must be a whole number".into());
    }
    let mut column_keys = BTreeSet::new();
    for column in columns {
        if let Some(samples) = column.get("samples") {
            if samples.as_array().is_none_or(|samples| {
                samples.len() > 64
                    || samples
                        .iter()
                        .any(|s| s.as_str().is_none_or(|s| s.len() > 4096))
            }) {
                return Err("invalid column measurement samples".into());
            }
        }
        if !column["fraction"].is_null()
            && column["fraction"]
                .as_f64()
                .is_none_or(|n| !n.is_finite() || n <= 0.0 || n > 1.0)
        {
            return Err("invalid column width fraction".into());
        }
        if text(column, "id").is_empty()
            || !column_keys.insert(text(column, "id"))
            || !column["label"].is_string()
        {
            return Err("invalid tree-table column".into());
        }
    }
    for row in rows {
        if text(row, "key").is_empty() || !keys.insert(text(row, "key")) {
            return Err("tree-table requires unique non-empty row keys".into());
        }
        let cells = row["cells"]
            .as_array()
            .ok_or("tree-table row requires cells")?;
        if cells.len() != columns.len()
            || cells
                .iter()
                .any(|c| c.as_str().is_none_or(|s| s.len() > 4096))
        {
            return Err("tree-table cell/column mismatch".into());
        }
        if row["depth"]
            .as_f64()
            .is_none_or(|d| !d.is_finite() || d < 0.0 || d > 256.0 || d.fract() != 0.0)
        {
            return Err("invalid tree-table depth".into());
        }
        if !row["branch"].is_boolean()
            || !row["selected"].is_boolean()
            || !row["focused"].is_boolean()
            || !row["expanded"].is_boolean()
        {
            return Err("tree-table flags must be bool".into());
        }
        if let Some(icon) = row["icon"].as_str() {
            if crate::icons::Icon::from_name(icon).is_none() {
                return Err("unknown tree-table icon".into());
            }
        }
    }
    if let Some(selected) = value.get("selected_keys") {
        let selected = selected
            .as_array()
            .ok_or("selected_keys must be an array")?;
        let mut seen = BTreeSet::new();
        for key in selected {
            let key = key.as_str().ok_or("selection key must be text")?;
            if !keys.contains(key) || !seen.insert(key) {
                return Err("unknown or duplicate selection key".into());
            }
        }
    }
    if let Some(focused) = value.get("focused_key") {
        if !focused.is_null() && focused.as_str().is_none_or(|key| !keys.contains(key)) {
            return Err("unknown focus key".into());
        }
    }
    Ok(())
}

fn selected(value: &Value, row: &Value) -> bool {
    value
        .get("selected_keys")
        .map_or(row["selected"] == true, |keys| {
            keys.as_array()
                .is_some_and(|keys| keys.contains(&row["key"]))
        })
}

fn focused(value: &Value, row: &Value) -> bool {
    value
        .get("focused_key")
        .map_or(row["focused"] == true, |key| key == &row["key"])
}

pub fn validate_properties(properties: &Value) -> Result<(), String> {
    for key in ["height", "row_height", "name_min", "metadata_scale"] {
        if let Some(value) = properties.get(key) {
            if value.as_f64().is_none_or(|v| {
                !v.is_finite()
                    || v < 0.0
                    || (["row_height", "metadata_scale"].contains(&key) && v == 0.0)
            }) {
                return Err(format!("invalid tree-table {key}"));
            }
        }
    }
    Ok(())
}

pub fn admit(value: &Value, action: &str, payload: &Value) -> Result<(), String> {
    validate(value)?;
    if ["ctrl", "shift"]
        .iter()
        .any(|key| payload.get(*key).is_some_and(|v| !v.is_boolean()))
    {
        return Err("selection modifiers must be bool".into());
    }
    let row = || {
        value["rows"]
            .as_array()
            .and_then(|rows| rows.iter().find(|r| r["key"] == payload["key"]))
            .ok_or("unknown row")
    };
    match action {
        "table" => {
            if !payload.is_object() || !payload["event"].is_string() {
                return Err("invalid native table event".into());
            }
        }
        "select" | "activate" | "context" => {
            row()?;
        }
        "expand" => {
            if row()?["branch"] != true {
                return Err("row is not expandable".into());
            }
        }
        "sort" => {
            if !value["columns"]
                .as_array()
                .is_some_and(|c| c.iter().any(|c| c["id"] == payload["column"]))
            {
                return Err("unknown column".into());
            }
        }
        "navigate" => {
            if ![
                "ArrowUp",
                "ArrowDown",
                "Home",
                "End",
                "ArrowLeft",
                "ArrowRight",
            ]
            .contains(&text(payload, "key"))
            {
                return Err("unknown navigation key".into());
            }
        }
        "drop" => {
            if !payload["source"].is_string()
                || !payload["destination"].is_string()
                || payload["keys"]
                    .as_array()
                    .is_none_or(|a| a.is_empty() || a.iter().any(|v| !v.is_string()))
            {
                return Err("invalid drop payload".into());
            }
        }
        "command" => {
            if payload["key"].is_string() {
                row()?;
            }
            if !value["context"]
                .as_array()
                .into_iter()
                .flatten()
                .chain(value["background_context"].as_array().into_iter().flatten())
                .any(|c| c["id"] == payload["command"] && c["enabled"] == true)
            {
                return Err("disabled or unknown context command".into());
            }
        }
        "activate-pane" => {}
        _ => return Err("unsupported tree-table action".into()),
    }
    Ok(())
}

#[derive(Clone, PartialEq)]
struct WidthKey {
    revision: Value,
    columns: Value,
    rows: usize,
    body: FontId,
    small: FontId,
}
#[derive(Clone)]
struct WidthCache {
    key: WidthKey,
    widths: Vec<f32>,
}
fn widths(ui: &Ui, id: &str, value: &Value, metadata_scale: f32) -> Vec<f32> {
    let columns = value["columns"].as_array().expect("validated columns");
    let rows = table_rows(value);
    let key = WidthKey {
        revision: value["revision"].clone(),
        columns: value["columns"].clone(),
        rows: rows.len(),
        body: {
            let mut font = TextStyle::Body.resolve(ui.style());
            font.size *= metadata_scale;
            font
        },
        small: TextStyle::Small.resolve(ui.style()),
    };
    let cache_id = Id::unique((ui.ctx().viewport_id(), id, "column-widths"));
    if let Some(cache) = ui.data(|d| d.get_temp::<WidthCache>(cache_id)) {
        if cache.key == key {
            return cache.widths;
        }
    }
    let mut widths = Vec::with_capacity(columns.len());
    for (i, column) in columns.iter().enumerate() {
        let mut width = ui
            .painter()
            .layout_no_wrap(
                format!("{} ▼", text(column, "label")),
                key.small.clone(),
                ui.visuals().text_color(),
            )
            .size()
            .x
            + 12.0;
        if i != 0 {
            for row in rows {
                width = width.max(
                    ui.painter()
                        .layout_no_wrap(
                            row["cells"][i].as_str().unwrap_or("").into(),
                            key.body.clone(),
                            ui.visuals().text_color(),
                        )
                        .size()
                        .x
                        + 12.0,
                );
            }
        }
        widths.push(((width - 12.0).ceil() + 12.0).max(number(column, "min_width", 40.0)));
    }
    ui.data_mut(|d| {
        d.insert_temp(
            cache_id,
            WidthCache {
                key,
                widths: widths.clone(),
            },
        )
    });
    widths
}

#[derive(Clone)]
struct DragRows {
    source: String,
    keys: Vec<String>,
    label: String,
}
fn drop_target(
    ui: &Ui,
    response: &egui::Response,
    destination: &str,
    id: &str,
    events: &mut Vec<Intent>,
) {
    if let Some(payload) = response.dnd_release_payload::<DragRows>() {
        {
            emit(
                events,
                id,
                "drop",
                json!({"source":payload.source,"keys":payload.keys,"destination":destination,"ctrl":ui.input(|i|i.modifiers.ctrl||i.modifiers.command),"shift":ui.input(|i|i.modifiers.shift)}),
            );
        }
    }
}
fn icon(ui: &Ui, name: &str, rect: Rect, colour: egui::Color32) {
    if let Some(icon) = crate::icons::Icon::from_name(name) {
        crate::icons::image_of(ui.ctx(), icon, rect.height(), colour).paint_at(ui, rect);
    }
}
fn context(
    ui: &mut Ui,
    items: &Value,
    key: &Value,
    id: &str,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    for item in items.as_array().into_iter().flatten() {
        if item["separator"] == true {
            ui.separator();
            continue;
        }
        let response = ui.add_enabled(
            item["enabled"] == true,
            egui::Button::new(text(item, "label")),
        );
        geometry.insert(format!("{id}.context.{}", text(item, "id")), response.rect);
        if response.clicked() {
            emit(
                events,
                id,
                "command",
                json!({"command":item["id"],"key":key}),
            );
            ui.close();
        }
    }
}

pub(crate) fn draw(
    ui: &mut Ui,
    widget: &Widget,
    value: &Value,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> egui::Response {
    if widget.properties["style"] == "outline" && widget.properties["row_layout"] != "sides" {
        return draw_outline(ui, widget, value, geometry, events);
    }
    if widget.properties["style"] == "gold" {
        return draw_gold(ui, widget, value, events);
    }
    if widget.properties["style"] == "outline" {
        return draw_counted_outline(ui, widget, value, geometry, events);
    }
    if widget.properties["style"] == "sides" {
        return ui
            .scope(|ui| {
                if value["busy"] == true {
                    ui.weak(value["loading"].as_str().unwrap_or("Loading…"));
                }
                egui::ScrollArea::vertical()
                    .id_salt((&widget.id, "sides"))
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        let rows = table_rows(value);
                        if rows.is_empty() && value["busy"] != true {
                            ui.weak(value["empty"].as_str().unwrap_or("No items"));
                        }
                        for row in rows {
                            egui::Sides::new().show(
                                ui,
                                |ui| {
                                    if let Some(icon) =
                                        crate::icons::Icon::from_name(text(row, "icon"))
                                    {
                                        ui.add(crate::icons::image_of(
                                            ui.ctx(),
                                            icon,
                                            12.0,
                                            ui.visuals().weak_text_color(),
                                        ));
                                    }
                                    let chosen = selected(value, row);
                                    let mut label = egui::RichText::new(
                                        row["cells"][0].as_str().unwrap_or_default(),
                                    );
                                    if chosen {
                                        label = label.strong();
                                    }
                                    let response = toolkit::selectable(ui, chosen, label);
                                    geometry.insert(
                                        format!("{}.key.{}", widget.id, text(row, "key")),
                                        response.rect,
                                    );
                                    if response.clicked() {
                                        emit(
                                            events,
                                            &widget.id,
                                            "select",
                                            json!({"key":text(row,"key")}),
                                        );
                                    }
                                },
                                |ui| {
                                    ui.weak(row["cells"][1].as_str().unwrap_or_default());
                                },
                            );
                        }
                    });
            })
            .response;
    }

    if widget.properties["style"] == "grid" {
        return draw_grid(ui, widget, value, geometry, events);
    }
    let id = &widget.id;
    let rows = table_rows(value);
    let columns = value["columns"].as_array().expect("validated columns");
    let metadata_scale = number(&widget.properties, "metadata_scale", 1.0);
    let mut measured = widths(ui, id, value, metadata_scale);
    let available = ui.available_width();
    let name_min = number(&widget.properties, "name_min", 140.0);
    let mut visible = columns.len();
    while visible > 1 && measured[1..visible].iter().sum::<f32>() + name_min > available {
        visible -= 1;
    }
    measured[0] = (available - measured[1..visible].iter().sum::<f32>()).max(20.0);
    let compact =
        ui.ctx().content_rect().height() < number(&widget.properties, "compact_below_height", 0.0);
    let reserve = if !compact && widget.properties["reserve_bottom_style"] == "small" {
        ui.text_style_height(&TextStyle::Small)
            + number(&widget.properties, "reserve_bottom_padding", 0.0)
    } else {
        0.0
    };
    let height = number(
        &widget.properties,
        "height",
        ui.available_height() - reserve,
    )
    .max(32.0);
    let row_h = number(
        &widget.properties,
        "row_height",
        (ui.text_style_height(&TextStyle::Body) + 6.0).round(),
    );
    let result=ui.allocate_ui(vec2(available,height),|ui| {
        ui.set_min_size(vec2(available,height));
        let header_h=if compact {0.0} else {ui.text_style_height(&TextStyle::Small)+8.0};
        if !compact {
        let (header,_)=ui.allocate_exact_size(vec2(available,header_h),Sense::hover());
        let mut x=header.left();
        for i in 0..visible {
            let column=&columns[i];
            let cell=Rect::from_min_size(pos2(x,header.top()),vec2(measured[i],header_h));x+=measured[i];
            let response=ui.interact(cell,Id::unique((ui.ctx().viewport_id(),id,"column",i)),Sense::click());
            response.widget_info(||egui::WidgetInfo::labeled(egui::Role::Button,true,text(column,"label")));
            geometry.insert(format!("{id}.column.{}",text(column,"id")),cell);
            if response.hovered(){ui.painter().rect_filled(cell,2.0,ui.visuals().widgets.hovered.weak_bg_fill);}
            let right=text(column,"align")=="right";
            let arrow=if value["sort"]==column["id"] {if value["ascending"]==true {" ▲"}else{" ▼"}}else{""};
            ui.painter().text(pos2(if right{cell.right()-6.0}else{cell.left()+6.0},cell.center().y),if right{Align2::RIGHT_CENTER}else{Align2::LEFT_CENTER},format!("{}{}",text(column,"label"),arrow),TextStyle::Small.resolve(ui.style()),ui.visuals().weak_text_color());
            if response.clicked(){emit(events,id,"sort",json!({"column":column["id"]}));}
        }
        ui.painter().hline(header.x_range(),header.bottom(),ui.visuals().widgets.noninteractive.bg_stroke);
        }
        let list_h=(height-header_h-if compact {0.0}else{ui.spacing().item_spacing.y}).max(row_h);
        let body=Rect::from_min_size(ui.cursor().min,vec2(available,list_h));
        let background=ui.interact(body,Id::unique((ui.ctx().viewport_id(),id,"background")),Sense::click());
        geometry.insert(format!("{id}.background"),body);
        if background.clicked(){background.request_focus();emit(events,id,"activate-pane",Value::Null);}
        background.context_menu(|ui|context(ui,&value["background_context"],&Value::Null,id,geometry,events));
        egui::ScrollArea::vertical().id_salt((id,"rows")).auto_shrink([false,false]).max_height(list_h).show_rows(ui,row_h,rows.len(),|ui,range|{
            for index in range {
                let row=&rows[index];
                let key=text(row,"key");
                let (rect,response)=ui.allocate_exact_size(vec2(ui.available_width(),row_h),Sense::click_and_drag());
                geometry.insert(format!("{id}.row.{index}"),rect);
                geometry.insert(format!("{id}.key.{key}"),rect);
                response.widget_info(||egui::WidgetInfo::selected(egui::Role::Button,true,selected(value,row),row["cells"][0].as_str().unwrap_or("")));
                let is_selected=selected(value,row);
                let colour=if is_selected{ui.visuals().selection.stroke.color}else{ui.visuals().text_color()};
                let painter=ui.painter_at(rect);
                if is_selected||response.hovered(){painter.rect_filled(rect,2.0,if is_selected{ui.visuals().selection.bg_fill}else{ui.visuals().widgets.hovered.weak_bg_fill});}
                if row["branch"]==true && response.dnd_hover_payload::<DragRows>().is_some_and(|drag|drag.source!=*id) {
                    painter.rect_stroke(rect.shrink(1.0),2.0,egui::Stroke::new(1.5,ui.visuals().selection.stroke.color),egui::StrokeKind::Inside);
                }
                if focused(value,row)&&value["active"]==true{painter.rect_stroke(rect.shrink(0.5),2.0,egui::Stroke::new(1.0,ui.visuals().selection.stroke.color.gamma_multiply(0.6)),egui::StrokeKind::Inside);}
                let mut tx=rect.left()+4.0+number(row,"depth",0.0)*14.0;
                let chevron=Rect::from_center_size(pos2(tx+8.0,rect.center().y),vec2(16.0,16.0));
                let mut toggled=false;
                if row["branch"]==true {
                    let toggle=ui.interact(chevron,Id::unique((ui.ctx().viewport_id(),id,key,"expand")),Sense::click());
                    toggle.widget_info(||egui::WidgetInfo::labeled(egui::Role::Button,true,format!("{} {}",if row["expanded"]==true{"Collapse"}else{"Expand"},row["cells"][0].as_str().unwrap_or(""))));
                    geometry.insert(format!("{id}.expand.{index}"),chevron);
                    if let Some(icon) = crate::icons::Icon::from_name(if row["expanded"]==true{"chevron-down"}else{"chevron-right"}) { crate::icons::image_of(ui.ctx(),icon,12.8,ui.visuals().weak_text_color()).paint_at(ui,chevron.shrink(1.0)); }
                    if toggle.clicked(){emit(events,id,"expand",json!({"key":key}));toggled=true;}
                }
                tx+=18.0;
                let icon_rect=Rect::from_center_size(pos2(tx+8.0,rect.center().y),vec2(16.0,16.0));
                icon(ui,text(row,"icon"),icon_rect,if is_selected{colour}else if row["branch"]==true{ui.visuals().hyperlink_color}else{ui.visuals().weak_text_color()});
                tx+=22.0;
                for i in 0..visible {
                    // The scroll area's row may snap its available width.
                    // Anchor metadata to that final rect, as native panels do.
                    let right=rect.right()-measured[i+1..visible].iter().sum::<f32>();
                    let cx=if i==0 {rect.left()} else {right-measured[i]};
                    let cell=Rect::from_min_max(pos2(cx,rect.top()),pos2(right,rect.bottom()));
                    let mut font=TextStyle::Body.resolve(ui.style());
                    if i>0 {font.size*=metadata_scale;}
                    let text=row["cells"][i].as_str().unwrap_or("");
                    if i==0 {
                        let width=(right-tx-4.0).max(0.0);
                        let text=crate::elide::fit(ui,text,&font,width);
                        painter.with_clip_rect(cell.intersect(ui.clip_rect())).text(pos2(tx,rect.center().y),Align2::LEFT_CENTER,text,font,colour);
                    } else {
                        let right_align=text_value(&columns[i],"align")=="right";
                        painter.with_clip_rect(cell.intersect(ui.clip_rect())).text(pos2(if right_align{right-6.0}else{cx+6.0},rect.center().y),if right_align{Align2::RIGHT_CENTER}else{Align2::LEFT_CENTER},text,font,if is_selected{colour}else{ui.visuals().weak_text_color()});
                    }
                }
                let modifiers=ui.input(|i|i.modifiers);
                if !toggled&&response.clicked(){response.request_focus();emit(events,id,"select",json!({"key":key,"ctrl":modifiers.ctrl||modifiers.command,"shift":modifiers.shift}));}
                if !toggled&&response.double_clicked(){emit(events,id,"activate",json!({"key":key}));}
                if response.secondary_clicked(){emit(events,id,"context",json!({"key":key}));}
                if response.drag_started(){
                    let keys=if is_selected{rows.iter().filter(|r|selected(value,r)).map(|r|text_value(r,"key").to_owned()).collect()}else{vec![key.to_owned()]};
                    response.dnd_set_drag_payload(DragRows{source:id.clone(),keys,label:row["cells"][0].as_str().unwrap_or("").into()});
                }
                if row["branch"]==true {drop_target(ui,&response,key,id,events);}
                response.context_menu(|ui|context(ui,&value["context"],&row["key"],id,geometry,events));
            }
        });
        drop_target(ui,&background,text(value,"key"),id,events);
    });
    if let Some(payload) = egui::DragAndDrop::payload::<DragRows>(ui.ctx()) {
        if let Some(pos) = ui.ctx().pointer_hover_pos() {
            egui::Area::new(Id::unique((ui.ctx().viewport_id(), id, "drag-label")))
                .order(egui::Order::Tooltip)
                .fixed_pos(pos + vec2(12.0, 12.0))
                .interactable(false)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.label(&payload.label);
                    });
                });
        }
    }
    result.response
}

/// Resizable, striped native table. Column sizing and cell semantics are
/// supplied by Mix; this renderer has no resource or application knowledge.
fn draw_grid(
    ui: &mut Ui,
    widget: &Widget,
    value: &Value,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> egui::Response {
    use egui_extras::{Column, TableBuilder};
    let id = &widget.id;
    let weak = ui.visuals().weak_text_color();
    let rows = table_rows(value);
    let columns = value["columns"].as_array().expect("validated columns");
    let height = ui.text_style_height(&TextStyle::Body)
        + number(&widget.properties, "row_padding", 6.0).clamp(0.0, 32.0);
    ui.scope(|ui| {
        let read_only = widget.properties["read_only"] == true;
        let room = (ui.available_width()
            - ui.spacing().item_spacing.x * (columns.len().saturating_sub(1) as f32))
            .max(1.0);
        let measured = columns
            .iter()
            .enumerate()
            .map(|(index, column)| sample_width(ui, id, column, rows, index))
            .collect::<Vec<_>>();
        let mut table = TableBuilder::new(ui)
            .id_salt(id)
            .auto_shrink([widget.properties["auto_shrink"] == true; 2])
            .vscroll(widget.properties["vscroll"] != false)
            .striped(true)
            .resizable(widget.properties["resizable"] != false)
            .sense(if read_only {
                Sense::hover()
            } else {
                Sense::click()
            })
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center));
        for (column, measured) in columns.iter().zip(measured) {
            let width = if let Some(width) = measured {
                Column::exact(width.max(number(column, "min_width", 50.0)))
            } else if column["remainder"] == true {
                Column::remainder()
            } else if let Some(fraction) = column["fraction"].as_f64() {
                Column::exact(room * fraction as f32)
            } else {
                Column::initial(number(column, "width", 90.0))
            };
            table = table.column(width.at_least(number(column, "min_width", 50.0)).clip(true));
        }
        table
            .header(height, |mut header| {
                for column in columns {
                    header.col(|ui| {
                        let arrow = if value["sort"] == column["id"] {
                            if value["reverse"] == true {
                                " ↑"
                            } else {
                                " ↓"
                            }
                        } else {
                            ""
                        };
                        let label =
                            egui::RichText::new(format!("{}{arrow}", text(column, "label")))
                                .strong();
                        let response = if read_only || column["sortable"] == false {
                            ui.label(label)
                        } else {
                            ui.add(egui::Button::new(label).frame(false))
                        };
                        let response = if let Some(tip) = column["tooltip"].as_str() {
                            response.on_hover_text(tip)
                        } else {
                            response
                        };
                        geometry
                            .insert(format!("{id}.column.{}", text(column, "id")), response.rect);
                        if !read_only && response.clicked() && column["sortable"] != false {
                            emit(events, id, "sort", json!({"column":column["id"]}));
                        }
                    });
                }
            })
            .body(|body| {
                body.rows(height, rows.len(), |mut row| {
                    let data = &rows[row.index()];
                    let key = text(data, "key");
                    row.set_selected(selected(value, data));
                    for (i, column) in columns.iter().enumerate() {
                        row.col(|ui| {
                            if i == 0 {
                                ui.add_space(number(data, "depth", 0.0) * 14.0);
                            }
                            let label =
                                egui::RichText::new(data["cells"][i].as_str().unwrap_or(""));
                            let label = if column["weak"] == true {
                                label.color(weak)
                            } else {
                                label
                            };
                            if column["align"] == "right" {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| ui.label(label),
                                );
                            } else {
                                if column["leading_gap"] == true {
                                    ui.add_space(ui.spacing().item_spacing.x);
                                }
                                let response = ui.label(label);
                                if let Some(tip) = data["tooltips"][i].as_str() {
                                    response.on_hover_text(tip);
                                }
                            }
                        });
                    }
                    let response = row.response();
                    geometry.insert(format!("{id}.key.{key}"), response.rect);
                    if !read_only && response.clicked() {
                        emit(events, id, "select", json!({"key":key}));
                    }
                });
            });
    })
    .response
}
fn text_value<'a>(v: &'a Value, key: &str) -> &'a str {
    text(v, key)
}

// Fixed internal column keys keep toolkit's static catalogue contract without
// interning or leaking application-authored strings. Mix sees its original ids.
const COLUMN_KEYS: [&str; 16] = [
    "c0", "c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8", "c9", "c10", "c11", "c12", "c13", "c14",
    "c15",
];
fn draw_gold(
    ui: &mut Ui,
    widget: &Widget,
    value: &Value,
    events: &mut Vec<Intent>,
) -> egui::Response {
    use toolkit::datatable::{
        Align, Cell, Column, DataTable, Everything, Order, Pick, RowStyle, Rows, SelectionMode,
        TableEvent, TableState, Total, Width,
    };
    struct Model<'a> {
        rows: &'a [Value],
    }
    impl Rows for Model<'_> {
        fn len(&self) -> usize {
            self.rows.len()
        }
        fn key(&self, row: usize) -> String {
            text(&self.rows[row], "key").into()
        }
        fn cell(&self, row: usize, column: &str) -> Cell {
            let index = COLUMN_KEYS.iter().position(|key| *key == column).unwrap();
            let data = &self.rows[row];
            let mut cell = Cell::new(data["cells"][index].as_str().unwrap_or(""))
                .weak(data["weak_cells"][index] == true);
            for marker in data["markers"][index].as_array().into_iter().flatten() {
                if let Some(icon) = marker.as_str().and_then(crate::icons::Icon::from_name) {
                    cell = cell.icon(icon);
                }
            }
            cell
        }
        fn style(&self, row: usize) -> RowStyle {
            RowStyle {
                strong: self.rows[row]["strong"] == true,
                weak: self.rows[row]["weak"] == true,
            }
        }
    }
    let authored = value["columns"].as_array().unwrap();
    let mut catalogue = String::new();
    let columns = authored
        .iter()
        .enumerate()
        .map(|(index, column)| {
            // Labels are plain text, never application-supplied Fluent source.
            let label = text(column, "label").replace(['\r', '\n', '{', '}'], " ");
            catalogue.push_str(&format!("{} = {}\n", COLUMN_KEYS[index], label));
            let min = number(column, "min_width", 40.0);
            let width = if let Some(width) =
                sample_width(ui, &widget.id, column, table_rows(value), index)
            {
                Width::Fixed(width.max(min))
            } else if column["remainder"] == true {
                Width::Remainder { min }
            } else if let Some(share) = column["fraction"].as_f64() {
                Width::Share {
                    share: share as f32,
                    min,
                }
            } else if column["resizable"] == true {
                Width::Resizable {
                    initial: number(column, "width", 90.0),
                    min,
                }
            } else {
                Width::Fixed(number(column, "width", 90.0))
            };
            let mut result = Column::new(COLUMN_KEYS[index], COLUMN_KEYS[index])
                .width(width)
                .align(if column["align"] == "right" {
                    Align::End
                } else {
                    Align::Start
                });
            if column["sortable"] != false {
                result = result.sortable(Order::Ascending);
            }
            result
        })
        .collect::<Vec<_>>();
    let mut state: TableState = native_table_state(&value["state"]).expect("validated table state");
    let external = |key: &str| {
        authored
            .iter()
            .position(|c| text(c, "id") == key)
            .map(|i| COLUMN_KEYS[i].to_owned())
    };
    if let Some(sort) = &mut state.sort {
        if let Some(key) = external(&sort.column) {
            sort.column = key;
        } else {
            state.sort = None;
        }
    }
    state.widths = state
        .widths
        .into_iter()
        .filter_map(|(k, v)| external(&k).map(|k| (k, v)))
        .collect();
    state.hidden = state
        .hidden
        .into_iter()
        .filter_map(|k| external(&k))
        .collect();
    let original = |key: &str| {
        COLUMN_KEYS
            .iter()
            .position(|k| *k == key)
            .map(|i| text(&authored[i], "id").to_owned())
            .unwrap_or_default()
    };
    let rows = Model {
        rows: table_rows(value),
    };
    let strings = toolkit::Strings::new(&catalogue);
    ui.scope(|ui| {
        if value["cancel_pending"] == true
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            emit(events, &widget.id, "table", json!({"event":"cancel"}));
        }
        let mut table=DataTable::new(&widget.id,&columns,&state).strings(&strings).keeps_tab(true)
            .selection(if widget.properties["multi"]==false{SelectionMode::Single}else{SelectionMode::Multi})
            .total(Total::Known(crate::widgets::whole(&value["total"]).unwrap_or(rows.len() as u64)))
            .pager(widget.properties["pager"]!=false).empty_text(value["empty"].as_str().unwrap_or("No items"));
        if let Some(link)=value["select_everything"].as_str(){ table=table.select_everything(Everything{link:link.into(),done:value["everything_done"].as_str().map(Into::into)}); }
        if let Some(count) = crate::widgets::whole(&value["selected_count"]) { table = table.selected_count(Some(count)); }
        for event in table.show(ui,&rows) {
            let payload=match event {
                TableEvent::SortChanged(sort)=>json!({"event":"sort","sort":sort.map(|s|json!({"column":original(&s.column),"order":s.order}))}),
                TableEvent::PageChanged(page)=>json!({"event":"page","page":page}),
                TableEvent::PageSizeChanged(size)=>json!({"event":"page_size","size":size}),
                TableEvent::Selected{key,pick,..}=>json!({"event":"select","key":key,"toggle":pick==Pick::Toggle}),
                TableEvent::Activated{key,..}=>json!({"event":"activate","key":key}),
                TableEvent::ColumnResized{column,width}=>json!({"event":"width","column":original(&column),"width":width}),
                TableEvent::ColumnShown{column,visible}=>json!({"event":"visible","column":original(&column),"visible":visible}),
                TableEvent::DensityChanged(density)=>json!({"event":"density","density":density}),
                TableEvent::ContextMenu{key,..}=>json!({"event":"context","key":key}),
                TableEvent::DragStarted{keys}=>json!({"event":"drag","keys":keys}),
                TableEvent::SelectionChanged{keys,focus}=>json!({"event":"selection","keys":keys,"focus":focus}),
                TableEvent::SelectEverything=>json!({"event":"everything"}),
            };
            emit(events,&widget.id,"table",payload);
        }
    }).response
}

/// A theme-native navigation outline. The caller owns hierarchy, expansion,
/// cursor and selection; this draws only the rows the caller supplies.

fn draw_outline(
    ui: &mut Ui,
    widget: &Widget,
    value: &Value,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> egui::Response {
    let id = &widget.id;
    if widget.properties["reveal_row"]
        .as_str()
        .is_some_and(|key| !key.is_empty())
    {
        let marker = Id::unique((ui.ctx().viewport_id(), id, "reveal-scroll"));
        let serial = widget.properties["reveal_serial"].clone();
        let changed = ui.ctx().data_mut(|data| {
            if data.get_temp::<Value>(marker).as_ref() == Some(&serial) {
                false
            } else {
                data.insert_temp(marker, serial);
                true
            }
        });
        if changed {
            if let Some(name) = widget.properties["saved_scroll"].as_str() {
                toolkit::saved::widgets::forget_scroll(ui.ctx(), name);
            }
        }
    }
    let area = egui::ScrollArea::vertical()
        .id_salt((id, "outline"))
        .auto_shrink([false, false]);
    let mut draw = |ui: &mut Ui| {
        let mut next = 0;
        outline_rows(
            ui,
            widget,
            table_rows(value),
            &mut next,
            0,
            geometry,
            events,
        );
    };
    let rect = if let Some(name) = widget.properties["saved_scroll"].as_str() {
        toolkit::saved::widgets::scroll(
            ui,
            name,
            None,
            widget.properties["scroll_ready"] != false,
            area,
            &mut draw,
        )
        .inner_rect
    } else {
        area.show(ui, &mut draw).inner_rect
    };
    ui.interact(rect, ui.scope_id().with((id, "outline")), Sense::hover())
}

#[expect(
    clippy::too_many_arguments,
    reason = "reusable outline traversal retains caller-owned rows and intents"
)]
fn outline_rows(
    ui: &mut Ui,
    widget: &Widget,
    rows: &[Value],
    next: &mut usize,
    depth: usize,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    let id = &widget.id;
    while let Some(row) = rows.get(*next) {
        let row_depth = number(row, "depth", 0.) as usize;
        if row_depth < depth {
            break;
        }
        if row_depth > depth {
            ui.indent((id, "level", *next), |ui| {
                outline_rows(ui, widget, rows, next, depth + 1, geometry, events)
            });
            continue;
        }
        *next += 1;
        let key = text(row, "key");
        let response = ui.horizontal(|ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            let colour = ui.visuals().text_color();
            if row["branch"] == true {
                let icon = if row["expanded"] == true {
                    crate::icons::Icon::ChevronDown
                } else {
                    crate::icons::Icon::ChevronRight
                };
                let toggle = ui.add(
                    egui::Button::image(crate::icons::image_of(ui.ctx(), icon, 14., colour))
                        .frame(false),
                );
                toggle.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::Role::Button,
                        ui.is_enabled(),
                        text(row, "toggle_label"),
                    )
                });
                geometry.insert(format!("{id}.expand.{key}"), toggle.rect);
                if toggle.clicked() {
                    emit(
                        events,
                        id,
                        "expand",
                        json!({"key":key,"open":row["expanded"]!=true}),
                    );
                }
            } else {
                ui.add_space(14. + ui.spacing().item_spacing.x);
            }
            if let Some(icon) = row["icon"].as_str().and_then(crate::icons::Icon::from_name) {
                ui.add(crate::icons::image_of(
                    ui.ctx(),
                    icon,
                    14.,
                    ui.visuals().weak_text_color(),
                ));
            }
            let mut label = egui::RichText::new(row["cells"][0].as_str().unwrap_or(""));
            if row["mono"] == true {
                label = label.monospace();
            }
            if row["bold"] == true {
                label = label.strong();
            }
            if row["weak"] == true {
                label = label.weak();
            }
            if row["error"] == true {
                label = label.color(ui.visuals().error_fg_color);
            }
            let r = toolkit::selectable(ui, row["selected"] == true, label)
                .on_hover_text(text(row, "tooltip"));
            if widget.properties["reveal_row"] == key {
                let marker = ui.scope_id().with((id, "revealed"));
                let serial = widget.properties["reveal_serial"].clone();
                let changed = ui.ctx().data_mut(|d| {
                    if d.get_temp::<Value>(marker).as_ref() != Some(&serial) {
                        d.insert_temp(marker, serial);
                        true
                    } else {
                        false
                    }
                });
                if changed {
                    r.scroll_to_me(Some(egui::Align::Center));
                }
            }
            geometry.insert(format!("{id}.key.{key}"), r.rect);
            if r.clicked() {
                emit(events, id, "select", json!({"key":key}));
            }
        });
        geometry.insert(format!("{id}.row.{key}"), response.response.rect);
        if row["expanded"] == true
            || rows
                .get(*next)
                .is_some_and(|row| number(row, "depth", 0.) as usize > depth)
        {
            ui.indent((id, key), |ui| {
                outline_rows(ui, widget, rows, next, depth + 1, geometry, events)
            });
        }
    }
}

fn draw_counted_outline(
    ui: &mut Ui,
    widget: &Widget,
    value: &Value,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> egui::Response {
    let id = &widget.id;
    let rows = table_rows(value);
    let keyboard = widget.properties["keyboard"] == true;
    let focus_id = Id::unique(id);
    if keyboard {
        let area = ui.available_rect_before_wrap();
        let back = ui.interact(area, focus_id, Sense::focusable_noninteractive());
        if ui.input(|i| i.pointer.primary_pressed()) && ui.rect_contains_pointer(area) {
            back.request_focus();
        }
        if ui.memory(|m| m.has_focus(focus_id)) {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    focus_id,
                    egui::EventFilter {
                        tab: true,
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        ..Default::default()
                    },
                )
            });
            for key in [
                egui::Key::ArrowUp,
                egui::Key::ArrowDown,
                egui::Key::ArrowLeft,
                egui::Key::ArrowRight,
            ] {
                if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                    emit(events, id, "navigate", json!({"key":format!("{key:?}")}));
                }
            }
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                if let Some(row) = rows
                    .iter()
                    .find(|r| focused(value, r))
                    .or_else(|| rows.iter().find(|r| selected(value, r)))
                    .or(rows.first())
                {
                    emit(events, id, "select", json!({"key":row["key"]}));
                }
            }
        }
    }
    ui.scope(|ui| {
        egui::ScrollArea::vertical()
            .id_salt((id, "outline"))
            .auto_shrink(false)
            .show(ui, |ui| {
                for row in rows {
                    let key = text(row, "key");
                    let shown = ui.scope(|ui| {
                        egui::Sides::new().show(
                            ui,
                            |ui| {
                                ui.add_space(ui.spacing().indent * number(row, "depth", 0.0));
                                if row["branch"] == true {
                                    let icon = if row["expanded"] == true {
                                        crate::icons::Icon::ChevronDown
                                    } else {
                                        crate::icons::Icon::ChevronRight
                                    };
                                    let mut r = ui.add(
                                        crate::icons::image_of(
                                            ui.ctx(),
                                            icon,
                                            12.0,
                                            ui.visuals().weak_text_color(),
                                        )
                                        .sense(Sense::click()),
                                    );
                                    if let Some(label) = row["disclosure_label"].as_str() {
                                        r = r.accessible_name(label);
                                    }
                                    geometry.insert(format!("{id}.expand.{key}"), r.rect);
                                    if r.clicked() {
                                        if keyboard {
                                            ui.memory_mut(|m| m.request_focus(focus_id));
                                        }
                                        emit(events, id, "expand", json!({"key":key}));
                                    }
                                } else {
                                    ui.add_space(12.0 + ui.spacing().item_spacing.x);
                                }
                                if let Some(icon) = crate::icons::Icon::from_name(text(row, "icon"))
                                {
                                    ui.add(crate::icons::image_of(
                                        ui.ctx(),
                                        icon,
                                        12.0,
                                        if row["error"] == true {
                                            ui.visuals().error_fg_color
                                        } else {
                                            ui.visuals().weak_text_color()
                                        },
                                    ));
                                }
                                let mut label =
                                    egui::RichText::new(row["cells"][0].as_str().unwrap_or(""));
                                if row["strong"] == true || selected(value, row) {
                                    label = label.strong();
                                }
                                if row["weak"] == true {
                                    label = label.weak();
                                }
                                let r = toolkit::selectable(ui, selected(value, row), label);
                                geometry.insert(format!("{id}.key.{key}"), r.rect);
                                if r.clicked() {
                                    if keyboard {
                                        ui.memory_mut(|m| m.request_focus(focus_id));
                                    }
                                    emit(events, id, "select", json!({"key":key}));
                                }
                            },
                            |ui| {
                                if let Some(n) =
                                    crate::widgets::whole(&row["count"]).filter(|n| *n > 0)
                                {
                                    ui.label(egui::RichText::new(n.to_string()).strong());
                                }
                            },
                        );
                    });
                    if focused(value, row) && (!keyboard || ui.memory(|m| m.has_focus(focus_id))) {
                        ui.painter().rect_stroke(
                            shown.response.rect,
                            ui.visuals().widgets.inactive.corner_radius,
                            ui.visuals().selection.stroke,
                            egui::StrokeKind::Inside,
                        );
                    }
                }
            });
    })
    .response
}
