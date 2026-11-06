// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The replaceable large-chart renderer. The engine supplies the same rings
//! used by painted sparklines; plotting types never enter that data path.
use egui::{Ui, Vec2};
use egui_plot::{
    AxisHints, GridMark, HoverPosition, ItemId, ItemIdSet, Line, Plot, PlotMemory, PlotResponse,
};
use std::{cell::Cell, collections::HashMap};
use toolkit::graph::{self, Hue};

/// Bounded data contract for a Mix-authored interactive time-series widget.
pub fn validate(value: &serde_json::Value) -> Result<(), String> {
    let series=value["series"].as_array().filter(|a| !a.is_empty() && a.len()<=64).ok_or("expected 1..64 chart series")?;
    if value["samples"].as_u64().is_none_or(|n| !(2..=6000).contains(&n))
        || value["interval_s"].as_f64().is_none_or(|n| !n.is_finite() || n<=0.0)
        || value["height"].as_f64().is_none_or(|n| !n.is_finite() || !(80.0..=2000.0).contains(&n)) {
        return Err("invalid chart interval, window or height".into());
    }
    if !["percent","bytes","rate","count"].contains(&value["format"].as_str().unwrap_or(""))
        || !["percent","capacity","rate","count"].contains(&value["scale"].as_str().unwrap_or("")) {
        return Err("invalid chart units or scale".into());
    }
    for item in series {
        if !item["name"].is_string() || item["values"].as_array().is_none_or(|a| a.len()>6000 || a.iter().any(|n| !n.is_null() && n.as_f64().is_none_or(|n| !n.is_finite()))) {
            return Err("invalid chart series values".into());
        }
    }
    Ok(())
}
fn bytes(n: f64) -> String {
    let units=["B","KiB","MiB","GiB","TiB","PiB"];
    let mut v=n.max(0.0);let mut i=0;
    while v>=1024.0 && i<5 {v/=1024.0;i+=1;}
    if i==0 || v>=100.0 {format!("{v:.0} {}",units[i])} else {format!("{v:.1} {}",units[i])}
}
fn percent(n: f64) -> String {format!("{:.1}%",n.clamp(0.0,100.0))}
fn rate(n: f64) -> String {format!("{}/s",bytes(n))}
fn count(n: f64) -> String {format!("{n:.0}")}
pub(crate) fn draw(ui: &mut Ui, id: &str, value: &serde_json::Value) -> egui::Response {
    let raw=value["series"].as_array().expect("validated series");
    let values:Vec<Vec<f64>>=raw.iter().map(|s| s["values"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap_or(f64::NAN)).collect()).collect();
    let series:Vec<Series<'_>>=raw.iter().zip(&values).map(|(s,values)| Series {
        name:s["name"].as_str().unwrap(),legend_label:s["legend"].as_str(),values,
        hue:match s["hue"].as_str().unwrap_or("accent") {
            "accent-plus"=>Hue::AccentPlus,"accent-minus"=>Hue::AccentMinus,"warning"=>Hue::Warning,
            _=>s["hue"].as_u64().map_or(Hue::Accent,|n| Hue::Series(n.min(255) as usize)),
        },
    }).collect();
    set_context(ui.ctx(),value["context"].as_str());
    let peak=value["peak"].as_f64().unwrap_or(100.0);
    let scale=match value["scale"].as_str().unwrap() {"capacity"=>Scale::Capacity(peak),"rate"=>Scale::Rate(peak),"count"=>Scale::Count(peak),_=>Scale::Percent};
    let format=match value["format"].as_str().unwrap() {"bytes"=>bytes,"rate"=>rate,"count"=>count,_=>percent};
    let size=Vec2::new(ui.available_width(),value["height"].as_f64().unwrap() as f32);
    ui.scope(|ui| show_titled(ui,id,value["title"].as_str().unwrap_or(""),size,Chart {series:&series,samples:value["samples"].as_u64().unwrap() as usize,interval_s:value["interval_s"].as_f64().unwrap(),scale,format})).response
}

#[cfg(test)]
mod adapter_tests {
    use super::*;
    #[test]
    fn chart_admission_and_missing_samples_never_invent_zeroes() {
        let value=serde_json::json!({"series":[{"name":"Example","values":[1,null,3]}],"samples":60,"interval_s":1,"height":180,"scale":"percent","format":"percent"});
        validate(&value).unwrap();
        let mut bad=value.clone();bad["height"]=serde_json::json!(0);assert!(validate(&bad).is_err());
        let mut bad=value.clone();bad["samples"]=serde_json::json!(6001);assert!(validate(&bad).is_err());
        assert_eq!(points(&[1.0,f64::NAN,3.0],60,1.0),vec![[-2.0,1.0],[0.0,3.0]]);
        assert_eq!(bytes(1536.0),"1.5 KiB");assert_eq!(bytes(300.0*1024.0*1024.0),"300 MiB");
        assert_eq!(readout("CPU",12.5,12.0,percent),"CPU: 12.5% · 12 s ago");
    }
    #[test]
    fn capacity_ticks_remain_exact_and_passive_ranges_stay_in_window() {
        let ticks=YTicks::for_scale(Scale::Capacity(32.0*1024.0*1024.0*1024.0));
        assert_eq!(ticks.max(),32.0*1024.0*1024.0*1024.0);
        assert_eq!(ticks.marks().iter().map(|m| tick_label(bytes,m.value)).collect::<Vec<_>>(),["0 B","8 GiB","16 GiB","24 GiB","32 GiB"]);
        let range=TimeRange{left:-500.0,right:10.0}.clamped(60.0,1.0);
        assert_eq!(range,TimeRange::full(60.0));
    }
}

pub struct Series<'a> {
    pub name: &'a str,
    /// A compact legend label; the full name remains in hover and tooltips.
    pub legend_label: Option<&'a str>,
    pub hue: Hue,
    pub values: &'a [f64],
}

pub struct Chart<'a> {
    pub series: &'a [Series<'a>],
    pub samples: usize,
    pub interval_s: f64,
    pub scale: Scale,
    pub format: fn(f64) -> String,
}

/// Range semantics belong to the caller, independently of its unit formatter.
#[derive(Clone, Copy)]
pub enum Scale {
    Percent,
    /// A real capacity; the last tick must equal this value exactly.
    Capacity(f64),
    /// Cover this peak rate in bytes/s, rounded up in binary display units.
    Rate(f64),
    /// Event rates and gauges use decimal counts, with caller-owned units.
    Count(f64),
}

#[derive(Clone, Copy, Debug)]
struct YTicks {
    step: f64,
    intervals: usize,
}

impl YTicks {
    fn for_scale(scale: Scale) -> Self {
        if let Scale::Capacity(capacity) = scale {
            let capacity = if capacity.is_finite() {
                capacity.max(1.0)
            } else {
                1.0
            };
            return Self {
                step: capacity / 4.0,
                intervals: 4,
            };
        }
        let peak = match scale {
            Scale::Rate(peak) | Scale::Count(peak) => peak,
            _ => {
                return Self {
                    step: 25.0,
                    intervals: 4,
                };
            }
        };
        let peak = if peak.is_finite() { peak.max(1.0) } else { 1.0 };
        // Round in the same B/KiB/MiB/... units as the formatter, so the
        // labels read 10/20/30/40 GiB rather than 9.3/18.6/27.9 GiB.
        let mut unit = 1.0;
        for _ in 0..5 {
            if matches!(scale, Scale::Count(_)) || peak / unit < 1024.0 {
                break;
            }
            unit *= 1024.0;
        }
        let target = (peak / unit / 4.0).max(1.0 / unit);
        let magnitude = 10f64.powf(target.log10().floor());
        let factor = [1.0, 2.0, 2.5, 5.0, 10.0]
            .into_iter()
            .find(|factor| factor * magnitude >= target)
            .unwrap_or(10.0);
        let step = (factor * magnitude * unit).max(1.0);
        let intervals = (peak / step).ceil().clamp(2.0, 4.0) as usize;
        Self { step, intervals }
    }

    fn max(self) -> f64 {
        self.step * self.intervals as f64
    }

    fn marks(self) -> Vec<GridMark> {
        (0..=self.intervals)
            .map(|i| GridMark {
                value: self.step * i as f64,
                step_size: self.step,
            })
            .collect()
    }
}

/// Passive zoom state, discarded on a node change and retained per resource
/// while moving between Performance sections on the same node.
#[derive(Clone, Default)]
struct Charts {
    node: Option<String>,
    ranges: HashMap<egui::Id, TimeRange>,
}

fn state_id() -> egui::Id {
    egui::Id::unique("viu-time-series-ranges")
}

pub fn set_context(ctx: &egui::Context, node: Option<&str>) {
    ctx.data_mut(|data| {
        let charts = data.get_temp_mut_or_default::<Charts>(state_id());
        if charts.node.as_deref() != node {
            charts.node = node.map(str::to_owned);
            charts.ranges.clear();
        }
    });
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TimeRange {
    left: f64,
    right: f64,
}

impl TimeRange {
    fn full(window: f64) -> Self {
        Self {
            left: -window,
            right: 0.0,
        }
    }
    fn span(self) -> f64 {
        self.right - self.left
    }
    fn clamped(self, window: f64, minimum: f64) -> Self {
        if !self.left.is_finite() || !self.right.is_finite() {
            return Self::full(window);
        }
        let span = self.span().clamp(minimum.min(window), window);
        let right = self.right.clamp(-window + span, 0.0);
        Self {
            left: (right - span).max(-window),
            right,
        }
    }
    fn zoom(self, factor: f64, pivot: f64, window: f64, minimum: f64) -> Self {
        if !factor.is_finite() || factor <= 0.0 || factor == 1.0 {
            return self;
        }
        let fraction = ((pivot - self.left) / self.span()).clamp(0.0, 1.0);
        let span = (self.span() / factor).clamp(minimum.min(window), window);
        Self {
            left: pivot - fraction * span,
            right: pivot + (1.0 - fraction) * span,
        }
        .clamped(window, minimum)
    }
}

fn tick_label(format: fn(f64) -> String, value: f64) -> String {
    let text = format(value);
    let split = text
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .unwrap_or(text.len());
    let number = &text[..split];
    let number = if number.contains('.') {
        number.trim_end_matches('0').trim_end_matches('.')
    } else {
        number
    };
    let unit = text[split..].trim();
    if unit.is_empty() {
        number.to_owned()
    } else {
        format!("{number} {unit}")
    }
}

fn time_tick(value: f64) -> String {
    if value >= 0.0 {
        "0".into()
    } else {
        format!("{:.0}", -value)
    }
}

fn readout_at(chart: &Chart<'_>, x: f64, interval_s: f64, hidden: &ItemIdSet) -> Option<String> {
    if !x.is_finite() || x > 0.0 {
        return None;
    }
    let steps = (-x / interval_s).round() as usize;
    let rows: Vec<_> = chart
        .series
        .iter()
        .filter_map(|series| {
            let n = series.values.len().min(chart.samples);
            if steps < n && !hidden.contains(&ItemId::new(series.name)) {
                let value = series.values[series.values.len() - 1 - steps];
                if !value.is_finite() {
                    return None;
                }
                Some(readout(
                    series.name,
                    value,
                    steps as f64 * interval_s,
                    chart.format,
                ))
            } else {
                None
            }
        })
        .collect();
    (!rows.is_empty()).then(|| rows.join("\n"))
}

fn annotated_readout(
    chart: &Chart<'_>,
    x: f64,
    interval_s: f64,
    hidden: &ItemIdSet,
    annotation: Option<&dyn Fn(usize) -> Option<String>>,
) -> Option<String> {
    let mut text = readout_at(chart, x, interval_s, hidden)?;
    if let Some(extra) = annotation.and_then(|f| f((-x / interval_s).round() as usize)) {
        text.push('\n');
        text.push_str(&extra);
    }
    Some(text)
}

pub fn readout(name: &str, value: f64, age_s: f64, format: fn(f64) -> String) -> String {
    let value = format(value);
    let age = format!("{:.0} s ago", age_s.max(0.0));
    if name.is_empty() {
        format!("{value} · {age}")
    } else {
        format!("{name}: {value} · {age}")
    }
}

fn points(values: &[f64], samples: usize, interval_s: f64) -> Vec<[f64; 2]> {
    let n = values.len().min(samples);
    values[values.len() - n..]
        .iter()
        .enumerate()
        .filter(|(_, value)| value.is_finite())
        .map(|(i, value)| [-((n - 1 - i) as f64) * interval_s, *value])
        .collect()
}

pub fn show(ui: &mut Ui, id: impl egui::AsIdSalt, size: Vec2, chart: Chart<'_>) {
    let _ = render(ui, id, size, chart);
}

pub fn show_titled(
    ui: &mut Ui,
    id: impl egui::AsIdSalt,
    title: &str,
    size: Vec2,
    chart: Chart<'_>,
) {
    let _ = render_custom(ui, id, size, chart, Some(title), None);
}

/// A caller-owned derived readout at the same sample as the plotted data.
/// Keeps ratios and resource semantics out of the chart renderer.
pub fn show_annotated(
    ui: &mut Ui,
    id: impl egui::AsIdSalt,
    title: &str,
    size: Vec2,
    chart: Chart<'_>,
    annotation: &dyn Fn(usize) -> Option<String>,
) {
    let _ = render_custom(ui, id, size, chart, Some(title), Some(annotation));
}

// Return the actual response to renderer tests; do not reconstruct egui_plot's
// private memory key from its id salt. Plotting types stay inside this module.
fn render(ui: &mut Ui, id: impl egui::AsIdSalt, size: Vec2, chart: Chart<'_>) -> PlotResponse<()> {
    render_custom(ui, id, size, chart, None, None)
}

/// Fit entries alongside the title first, then wrap only overflowing entries.
/// Each row is right-aligned; no legend row is taken from the plot's height.
fn header_rows(width: f32, title: f32, entries: &[f32], gap: f32) -> Vec<Vec<usize>> {
    let mut rows = vec![Vec::new()];
    let mut remaining = (width - title - gap).max(0.0);
    for (i, entry) in entries.iter().enumerate() {
        let spacing = if rows.last().unwrap().is_empty() {
            0.0
        } else {
            gap
        };
        if *entry + spacing > remaining && (!rows.last().unwrap().is_empty() || rows.len() == 1) {
            rows.push(Vec::new());
            remaining = width;
        }
        let spacing = if rows.last().unwrap().is_empty() {
            0.0
        } else {
            gap
        };
        rows.last_mut().unwrap().push(i);
        remaining -= entry + spacing;
    }
    rows
}

/// One measured title header. Swatch and text have separate rectangles,
/// rather than relying on whitespace surviving a widget's text layout.
fn title_line(
    ui: &mut Ui,
    plot_id: egui::Id,
    title: &str,
    width: f32,
    series: &[Series<'_>],
    hidden: &mut ItemIdSet,
) {
    let font = egui::TextStyle::Small.resolve(ui.style());
    let height = ui.text_style_height(&egui::TextStyle::Small);
    let swatch = height * 0.4;
    let gap = ui.spacing().item_spacing.x * 0.5;
    let title =
        ui.painter()
            .layout_no_wrap(title.to_owned(), font.clone(), ui.visuals().text_color());
    let entries: Vec<_> = series
        .iter()
        .filter(|_| series.len() > 1)
        .map(|s| {
            let colour = if hidden.contains(&ItemId::new(s.name)) {
                ui.visuals().weak_text_color()
            } else {
                graph::tint(ui, s.hue)
            };
            (
                ui.painter().layout_no_wrap(
                    s.legend_label.unwrap_or(s.name).to_owned(),
                    font.clone(),
                    colour,
                ),
                colour,
            )
        })
        .collect();
    let widths: Vec<_> = entries
        .iter()
        .map(|(g, _)| swatch + gap + g.size().x)
        .collect();
    // Keep a full theme gap at the column edge, even when the next column
    // starts with a title on this same baseline.
    let inset = ui.spacing().item_spacing.x;
    let rows = header_rows((width - inset).max(0.0), title.size().x, &widths, gap);
    let total_height =
        rows.len() as f32 * height + (rows.len() - 1) as f32 * ui.spacing().item_spacing.y;
    let (mut rect, _) =
        ui.allocate_exact_size(egui::vec2(width, total_height), egui::Sense::hover());
    rect.max.x -= inset;
    let painter = ui.painter_at(rect);
    let title_rect = egui::Rect::from_min_size(rect.min, title.size()).intersect(rect);
    ui.interact(title_rect, plot_id.with("title"), egui::Sense::hover())
        .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, title.text()));
    painter.galley(rect.min, title, ui.visuals().text_color());
    for (row_index, row) in rows.iter().enumerate() {
        let row_width =
            row.iter().map(|&i| widths[i]).sum::<f32>() + row.len().saturating_sub(1) as f32 * gap;
        let mut x = rect.right() - row_width;
        let y = rect.top() + row_index as f32 * (height + ui.spacing().item_spacing.y);
        for &i in row {
            let entry = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(widths[i], height));
            let visible = !hidden.contains(&ItemId::new(series[i].name));
            let response = ui.interact(
                entry.intersect(rect),
                plot_id.with(series[i].name),
                egui::Sense::click(),
            );
            if response.hovered() || response.has_focus() {
                painter.rect_filled(entry, 0.0, ui.visuals().widgets.hovered.bg_fill);
            }
            let centre = egui::pos2(x + swatch * 0.5, entry.center().y);
            painter.circle_filled(centre, swatch * 0.5, entries[i].1);
            painter.galley(
                egui::pos2(x + swatch + gap, y),
                entries[i].0.clone(),
                entries[i].1,
            );
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::Role::Button, true, visible, series[i].name)
            });
            let clicked = response.clicked();
            response.on_hover_text(series[i].name);
            if clicked {
                let id = ItemId::new(series[i].name);
                if visible {
                    hidden.insert(id);
                } else {
                    hidden.remove(&id);
                }
            }
            x += widths[i] + gap;
        }
    }
}

fn render_custom(
    ui: &mut Ui,
    id: impl egui::AsIdSalt,
    size: Vec2,
    chart: Chart<'_>,
    title: Option<&str>,
    annotation: Option<&dyn Fn(usize) -> Option<String>>,
) -> PlotResponse<()> {
    let interval_s = if chart.interval_s.is_finite() && chart.interval_s > 0.0 {
        chart.interval_s
    } else {
        1.0
    };
    let window = chart.samples.saturating_sub(1).max(1) as f64 * interval_s;
    let minimum = (4.0 * interval_s).min(window);
    let ticks = YTicks::for_scale(chart.scale);
    let text_height = ui.text_style_height(&egui::TextStyle::Body);
    // Top labels extend above the plot. This padding is part of the supplied
    // height, not an extra row that pushes the rest of the pane down.
    let plot_id = ui.make_persistent_id(id);
    let mut hidden = PlotMemory::load(ui.ctx(), plot_id)
        .map(|memory| memory.hidden_items)
        .unwrap_or_default();
    if let Some(title) = title {
        title_line(ui, plot_id, title, size.x, chart.series, &mut hidden);
    }
    let padding = text_height * 0.5 + ui.spacing().item_spacing.y;
    ui.add_space(padding);
    if let Some(mut memory) = PlotMemory::load(ui.ctx(), plot_id) {
        memory.hidden_items = hidden.clone();
        memory.store(ui.ctx(), plot_id);
    }
    let mut range = ui
        .ctx()
        .data_mut(|data| {
            data.get_temp_mut_or_default::<Charts>(state_id())
                .ranges
                .get(&plot_id)
                .copied()
                .unwrap_or(TimeRange::full(window))
        })
        .clamped(window, minimum);
    let zoomed = range.span() < window - f64::EPSILON * window;
    let pointer_time = Cell::new(None);
    let y_axis = AxisHints::new_y()
        .formatter(|mark, _| tick_label(chart.format, mark.value))
        .tick_label_color(ui.visuals().text_color())
        .label_spacing(0.0..=text_height);
    // The default 60 px fade threshold hides 10 s marks in half-width
    // minute charts; its next base-10 step (100 s) lies outside the window.
    let x_axis = AxisHints::new_x()
        .formatter(|mark, _| time_tick(mark.value))
        .label_spacing(30.0..=50.0);
    let plot = Plot::new(plot_id)
        .id(plot_id)
        .width(size.x)
        .height((size.y - padding).max(text_height * 2.0))
        .default_x_bounds(-window, 0.0)
        .default_y_bounds(0.0, ticks.max())
        // egui_plot applies its own navigation after the build callback. Own
        // every input path here so the final clamp precedes axes and painting;
        // no native pan, axis drag, pinch or wheel can bypass the time limits.
        .allow_drag(false)
        .allow_zoom(false)
        .allow_axis_zoom_drag(false)
        .allow_boxed_zoom(false)
        .allow_scroll(false)
        .allow_double_click_reset(false)
        .sense(if zoomed {
            egui::Sense::click_and_drag()
        } else {
            egui::Sense::click()
        })
        .show_grid([false, true])
        .show_y(false)
        .custom_x_axes(vec![x_axis])
        .custom_y_axes(vec![y_axis])
        .y_grid_spacer(move |_| ticks.marks())
        .label_formatter(|hover| {
            let position = match hover {
                HoverPosition::NearDataPoint { position, .. }
                | HoverPosition::Elsewhere { position } => position,
            };
            let x = pointer_time.get().unwrap_or(position.x);
            annotated_readout(&chart, x, interval_s, &hidden, annotation)
        });
    let colours: Vec<_> = chart
        .series
        .iter()
        .map(|s| graph::tint(ui, s.hue))
        .collect();
    let output = plot.show(ui, |plot_ui| {
        let response = plot_ui.response();
        let rect = response.rect;
        let reset = response.double_clicked();
        if zoomed && response.dragged_by(egui::PointerButton::Primary) && rect.width() > 0.0 {
            let shift = -f64::from(response.drag_delta().x / rect.width()) * range.span();
            range.left += shift;
            range.right += shift;
        }
        if response.contains_pointer()
            && let Some(pointer) = plot_ui.ctx().input(|input| input.pointer.hover_pos())
        {
            let fraction = f64::from((pointer.x - rect.left()) / rect.width().max(1.0));
            let pivot = range.left + fraction * range.span();
            let factor = plot_ui.ctx().input_mut(|input| {
                let wheel = input.smooth_scroll_delta;
                input.smooth_scroll_delta = Vec2::ZERO;
                f64::from(input.zoom_delta_2d().x) * (f64::from(wheel.x + wheel.y) * 0.01).exp()
            });
            range = range.zoom(factor, pivot, window, minimum);
        }
        if reset {
            range = TimeRange::full(window);
        }
        // Apply after *all* navigation, including the native egui pinch/Ctrl
        // zoom delta. Native plot navigation stays disabled above.
        range = range.clamped(window, minimum);
        pointer_time.set(plot_ui.response().hover_pos().map(|pointer| {
            range.left + f64::from((pointer.x - rect.left()) / rect.width().max(1.0)) * range.span()
        }));
        plot_ui.set_plot_bounds_x(range.left..=range.right);
        plot_ui.set_plot_bounds_y(0.0..=ticks.max());
        for (series, colour) in chart.series.iter().zip(colours) {
            if hidden.contains(&ItemId::new(series.name)) {
                continue;
            }
            plot_ui.line(
                Line::new(
                    series.name,
                    points(series.values, chart.samples, interval_s),
                )
                .color(colour)
                .width(1.5),
            );
        }
    });
    if let Some(mut memory) = PlotMemory::load(ui.ctx(), plot_id) {
        memory.hidden_items = hidden;
        memory.store(ui.ctx(), plot_id);
    }
    ui.ctx().data_mut(|data| {
        data.get_temp_mut_or_default::<Charts>(state_id())
            .ranges
            .insert(plot_id, range);
    });
    output
}

