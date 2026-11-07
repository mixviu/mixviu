// SPDX-License-Identifier: MIT OR Apache-2.0
//! Compiled widget search requests. Mix owns the query and navigation; this
//! cache never keeps a matched range across a native grid update.
use super::model::{
    GridSnapshot, SearchColors, SearchPattern, SearchPoint, SearchRequest, Terminal,
};
use crate::widgets::{Intent, Widget};
use serde::Deserialize;
use serde_json::{Value, json};

pub(super) fn colors(ctx: &egui::Context) -> SearchColors {
    let palette = toolkit::chrome::Chrome::of(ctx).palette;
    let rgb = |c: egui::Color32| [c.r(), c.g(), c.b()];
    SearchColors {
        foreground: rgb(palette.menu_highlight_text),
        background: rgb(palette.menu_highlight),
    }
}

#[derive(Deserialize)]
struct Options {
    query: String,
    #[serde(default)]
    regex: bool,
    #[serde(default = "yes")]
    case_sensitive: bool,
    #[serde(default)]
    backwards: bool,
    #[serde(default)]
    origin: Option<SearchPoint>,
    #[serde(default = "budget")]
    max_lines: usize,
    request: u64,
}
fn yes() -> bool {
    true
}
fn budget() -> usize {
    10000
}

struct Compiled {
    pattern: SearchPattern,
    request: SearchRequest,
}

fn compile(value: &Value) -> Result<Compiled, String> {
    // Scene expressions use f64 numbers even for whole Mix values. Convert
    // only validated numeric fields; never coerce strings or round fractions.
    let mut value = value.clone();
    for field in ["request", "max_lines"] {
        if value.get(field).is_some() {
            let number = crate::widgets::whole(&value[field])
                .filter(|n| *n <= 9_007_199_254_740_991)
                .ok_or_else(|| format!("search {field} requires a non-negative whole number"))?;
            value[field] = json!(number);
        }
    }
    if !value["origin"].is_null() {
        let origin = &value["origin"];
        let row = origin["row"]
            .as_f64()
            .filter(|n| {
                n.is_finite() && n.fract() == 0.0 && *n >= i32::MIN as f64 && *n <= i32::MAX as f64
            })
            .ok_or("search origin requires signed row and non-negative col")?
            as i32;
        let col = crate::widgets::whole(&origin["col"])
            .filter(|n| *n <= 9_007_199_254_740_991)
            .ok_or("search origin requires signed row and non-negative col")?;
        value["origin"] = json!({"row":row,"col":col});
    }
    let options: Options = serde_json::from_value(value)
        .map_err(|error| format!("invalid terminal search options: {error}"))?;
    let _request = options.request;
    if !(1..=10000).contains(&options.max_lines) {
        return Err("search line budget must be 1..10000".into());
    }
    Ok(Compiled {
        pattern: SearchPattern::new(&options.query, options.regex, options.case_sensitive)?,
        request: SearchRequest {
            origin: options.origin,
            backwards: options.backwards,
            max_lines: options.max_lines,
        },
    })
}

pub(super) struct Search {
    widget: String,
    value: Value,
    compiled: Result<Compiled, String>,
    reported: Option<Value>,
}

pub(super) fn snapshot(
    cache: &mut Option<Search>,
    widget: &Widget,
    handle: &str,
    terminal: &Terminal,
    colors: SearchColors,
    events: &mut Vec<Intent>,
) -> GridSnapshot {
    let value = &widget.properties["search"];
    // An explicitly named native owner is mandatory. Switching focus does
    // not move a query to another terminal, even for identical pane titles.
    if value.is_null() || value["handle"].as_str() != Some(handle) {
        *cache = None;
        return terminal.grid_snapshot();
    }
    if cache
        .as_ref()
        .is_none_or(|c| c.widget != widget.id || c.value != *value)
    {
        *cache = Some(Search {
            widget: widget.id.clone(),
            value: value.clone(),
            compiled: compile(value),
            reported: None,
        });
    }
    let cache = cache.as_mut().unwrap();
    let result = match &mut cache.compiled {
        Ok(compiled) => {
            terminal.grid_snapshot_with_search(&mut compiled.pattern, compiled.request, colors)
        }
        Err(error) => Err(error.clone()),
    };
    let (snapshot, mut status) = match result {
        Ok((snapshot, outcome)) => (
            snapshot,
            serde_json::to_value(outcome).expect("search result"),
        ),
        // A once-valid origin may expire on resize or scrollback eviction.
        // Paint the current unhighlighted grid and let Mix choose recovery.
        Err(error) => (terminal.grid_snapshot(), json!({"error":error})),
    };
    status["handle"] = json!(handle);
    status["request"] = value["request"].clone();
    if cache.reported.as_ref() != Some(&status) {
        cache.reported = Some(status.clone());
        events.push(Intent {
            id: widget.id.clone(),
            action: "search".into(),
            value: status,
        });
    }
    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;

    const COLORS: SearchColors = SearchColors {
        foreground: [1, 2, 3],
        background: [80, 90, 100],
    };

    #[test]
    fn studio_search_uses_a_readable_theme_pair_in_dark_and_light_modes() {
        let luminance = |rgb: [u8; 3]| {
            let channels = rgb.map(|v| {
                let v = v as f64 / 255.0;
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            });
            0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]
        };
        for mode in ["dark", "light"] {
            let ctx = egui::Context::default();
            let appearance =
                crate::appearance::resolve(&json!({"scheme":"studio","mode":mode})).unwrap();
            crate::appearance::install_style(
                &ctx,
                &json!({"scheme":"studio","mode":mode}),
                appearance,
            );
            let colors = colors(&ctx);
            let foreground = luminance(colors.foreground);
            let background = luminance(colors.background);
            let contrast =
                (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05);
            assert!(contrast >= 4.5, "{mode}: {colors:?} contrast={contrast}");
        }
    }

    #[test]
    fn owner_request_output_and_invalid_origin_keep_search_and_clipboard_separate() {
        let terminal = Terminal::from_test_vt(12, 3, b"hit\r\nmiss\r\nhit");
        terminal.selection_start(
            0,
            1,
            super::super::model::SelectionSide::Left,
            super::super::model::SelectionType::Simple,
        );
        terminal.selection_update(3, 1, super::super::model::SelectionSide::Right);
        let selection = terminal.selection_text();
        let mut widget = Widget::new("one", "terminal", "Terminal", json!("vt-1"));
        widget.properties = json!({"search":{"handle":"vt-1","query":"hit","request":1}});
        let mut cache = None;
        let mut events = Vec::new();
        let first = snapshot(&mut cache, &widget, "vt-1", &terminal, COLORS, &mut events);
        assert_eq!(first.screen.cells[0].bg, COLORS.background);
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].value["matched"]["start"],
            json!({"row":0,"col":0})
        );
        snapshot(&mut cache, &widget, "vt-1", &terminal, COLORS, &mut events);
        assert_eq!(
            events.len(),
            1,
            "idle results must not flood the Mix mailbox"
        );

        widget.properties["search"]["origin"] = json!({"row":2,"col":0});
        widget.properties["search"]["request"] = json!(2);
        let moved = snapshot(&mut cache, &widget, "vt-1", &terminal, COLORS, &mut events);
        assert_eq!(moved.screen.cells[24].bg, COLORS.background);
        assert_ne!(moved.screen.cells[0].bg, COLORS.background);
        assert_eq!(events.last().unwrap().value["request"], 2);

        widget.properties["search"]["origin"] = json!({"row":-99,"col":0});
        let refused = snapshot(&mut cache, &widget, "vt-1", &terminal, COLORS, &mut events);
        assert!(
            events.last().unwrap().value["error"]
                .as_str()
                .unwrap()
                .contains("origin")
        );
        assert_ne!(refused.screen.cells[24].bg, COLORS.background);
        widget.properties["search"]["origin"] = Value::Null;
        widget.properties["search"]["query"] = json!("[");
        widget.properties["search"]["regex"] = json!(true);
        snapshot(&mut cache, &widget, "vt-1", &terminal, COLORS, &mut events);
        assert!(
            events.last().unwrap().value["error"]
                .as_str()
                .unwrap()
                .contains("expression")
        );

        let before = events.len();
        snapshot(&mut cache, &widget, "vt-2", &terminal, COLORS, &mut events);
        assert!(cache.is_none());
        assert_eq!(
            events.len(),
            before,
            "a sibling must never answer for the requested owner"
        );
        assert_eq!(terminal.selection_text(), selection);
        assert_eq!(terminal.display_offset(), 0);
    }

    #[test]
    fn live_grid_and_colours_are_recomputed_without_recompiling_the_query() {
        let terminal = Terminal::from_test_vt(12, 3, b"hit");
        let mut widget = Widget::new("one", "terminal", "Terminal", json!("vt-1"));
        widget.properties = json!({"search":{"handle":"vt-1","query":"hit","request":1}});
        let mut cache = None;
        let mut events = Vec::new();
        snapshot(&mut cache, &widget, "vt-1", &terminal, COLORS, &mut events);
        terminal.advance_test_vt(b"\x1b[H\x1b[M");
        snapshot(&mut cache, &widget, "vt-1", &terminal, COLORS, &mut events);
        // Pinned Rio retains a deleted top row in scrollback. Searching
        // history must still find it; only an explicit history clear removes it.
        assert_eq!(events.last().unwrap().value["matched"]["start"]["row"], -1);
        assert_eq!(events.len(), 2);
        terminal.advance_test_vt(b"\x1b[3J\x1b[2J\x1b[H");
        snapshot(&mut cache, &widget, "vt-1", &terminal, COLORS, &mut events);
        assert!(events.last().unwrap().value["matched"].is_null());
        assert_eq!(events.len(), 3);
        terminal.advance_test_vt(b"\x1b[Hhit");
        let colors = SearchColors {
            background: [100, 110, 120],
            ..COLORS
        };
        let painted = snapshot(&mut cache, &widget, "vt-1", &terminal, colors, &mut events);
        assert_eq!(painted.screen.cells[0].bg, colors.background);
        assert_eq!(events.len(), 4);
        widget.properties["search"] = Value::Null;
        let cleared = snapshot(&mut cache, &widget, "vt-1", &terminal, colors, &mut events);
        assert!(cache.is_none());
        assert_ne!(cleared.screen.cells[0].bg, colors.background);
    }

    #[test]
    fn scene_float_numbers_accept_only_exact_whole_options_and_origins() {
        let terminal = Terminal::from_test_vt(12, 3, b"hit");
        let mut widget = Widget::new("one", "terminal", "Terminal", json!("vt-1"));
        widget.properties = json!({"search":{"handle":"vt-1","query":"hit",
            "request":1.0,"max_lines":10000.0,"origin":{"row":0.0,"col":0.0}}});
        let mut cache = None;
        let mut events = Vec::new();
        let painted = snapshot(&mut cache, &widget, "vt-1", &terminal, COLORS, &mut events);
        assert_eq!(painted.screen.cells[0].bg, COLORS.background);
        assert_eq!(events[0].value["matched"]["end"], json!({"row":0,"col":2}));
        assert_eq!(events[0].value["request"], json!(1.0));
        for (field, bad) in [
            ("request", json!(1.5)),
            ("request", json!("1")),
            ("request", json!(-1.0)),
            ("request", json!(9007199254740992.0)),
            ("max_lines", json!(0.0)),
            ("max_lines", json!(10001.0)),
            ("origin", json!({"row":0.5,"col":0.0})),
            ("origin", json!({"row":0.0,"col":-1.0})),
            ("origin", json!({"row":2147483648.0,"col":0.0})),
            ("origin", json!({"row":"0","col":0.0})),
        ] {
            let mut value = widget.properties["search"].clone();
            value[field] = bad;
            assert!(compile(&value).is_err(), "{value}");
        }
    }
}
