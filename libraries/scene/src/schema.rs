// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The widget families and their ports: names, types, defaults, enum
//! values and minimums, plus the value checks every port goes through.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashSet;

use crate::{Diagnostic, MAX_ROWS};

/// One port as `describe` reports it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PortDescribe {
    pub path: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub mutable: bool,
    pub sensitive: bool,
    pub description: String,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

#[derive(Clone, Copy)]
pub(crate) struct Port {
    pub(crate) name: &'static str,
    pub(crate) ty: &'static str,
    pub(crate) required: bool,
    pub(crate) default: Option<&'static str>,
    pub(crate) enum_values: &'static [&'static str],
    pub(crate) min: Option<f64>,
}

const fn p(n: &'static str, t: &'static str, r: bool, d: Option<&'static str>) -> Port {
    Port {
        name: n,
        ty: t,
        required: r,
        default: d,
        enum_values: &[],
        min: None,
    }
}
const fn pe(n: &'static str, d: &'static str, e: &'static [&'static str]) -> Port {
    Port {
        name: n,
        ty: "string",
        required: false,
        default: Some(d),
        enum_values: e,
        min: None,
    }
}
const fn pnd(n: &'static str, d: &'static str, m: Option<f64>) -> Port {
    Port {
        name: n,
        ty: "number",
        required: false,
        default: Some(d),
        enum_values: &[],
        min: Some(match m {
            Some(value) => value,
            None => 0.0,
        }),
    }
}
const fn pn(n: &'static str, r: bool, m: Option<f64>) -> Port {
    Port {
        name: n,
        ty: "number",
        required: r,
        default: None,
        enum_values: &[],
        min: Some(match m {
            Some(value) => value,
            None => 0.0,
        }),
    }
}

/// The ports of family `f`, or `None` for an unknown family.
pub(crate) fn schema(f: &str) -> Option<Vec<Port>> {
    static EDGE: &[&str] = &["right", "left", "top", "bottom"];
    static ALIGN: &[&str] = &["start", "center", "end", "stretch"];
    static TONE: &[&str] = &["normal", "danger", "primary"];
    static KIND: &[&str] = &["edge", "dialog", "toast"];
    static WINDOW: [Port; 6] = [
        Port {
            name: "kind",
            ty: "string",
            required: true,
            default: None,
            enum_values: KIND,
            min: None,
        },
        pe("edge", "\"right\"", EDGE),
        p("title", "string", false, None),
        pn("w", false, None),
        pn("h", false, None),
        p("chrome", "bool", false, Some("true")),
    ];
    static BOX: [Port; 9] = [
        p("children", "list", true, None),
        pnd("gap", "0", None),
        pnd("padding", "0", None),
        p("fill", "bool", false, Some("false")),
        // A column stretched its children before `align` existed; the
        // default keeps every v0 document laying out exactly as it did.
        pe("align", "\"stretch\"", ALIGN),
        pn("height", false, None),
        pn("radius", false, None),
        p("background", "string", false, None),
        p("scroll", "bool", false, Some("false")),
    ];
    static SECTION: [Port; 7] = [
        p("children", "list", true, None),
        p("titles", "list", true, None),
        pe("frame", "\"plain\"", &["plain", "dock"]),
        p("on_change", "string", false, None),
        p("actions", "list", false, None),
        p("on_action", "string", false, None),
        p("focused", "bool", false, None),
    ];
    static CONTENT: [Port; 5] = [
        p("children", "list", true, None),
        p("settings", "object", false, Some("{}")),
        p("scroll", "bool", false, Some("true")),
        pe("surface", "\"panel\"", &["panel", "editor"]),
        p("keyboard_scroll", "object", false, None),
    ];
    static PANES: [Port; 14] = [
        p("children", "list", true, None),
        pnd("left_fraction", "0.18", None),
        pnd("right_fraction", "0.42", None),
        p("right_visible", "bool", false, Some("true")),
        p("clip", "bool", false, Some("true")),
        pe("focus", "\"center\"", &["left", "center", "right"]),
        p("on_change", "string", false, None),
        pn("compact_below", false, None),
        pn("narrow_below", false, None),
        p("narrow_left_range", "list", false, None),
        p("narrow_right_range", "list", false, None),
        p("keyboard", "bool", false, Some("false")),
        p("focus_targets", "object", false, None),
        p("pane_labels", "object", false, None),
    ];
    static SIDES: [Port; 2] = [
        p("children", "list", true, None),
        p("wrap", "bool", false, Some("false")),
    ];
    static ROW: [Port; 12] = [
        p("children", "list", true, None),
        p("wrap", "bool", false, Some("false")),
        pnd("gap", "0", None),
        pnd("padding", "0", None),
        p("fill", "bool", false, Some("false")),
        Port {
            name: "align",
            ty: "string",
            required: false,
            default: Some("\"start\""),
            enum_values: ALIGN,
            min: None,
        },
        pn("height", false, None),
        pn("radius", false, None),
        p("background", "string", false, None),
        p("hover", "string", false, None),
        p("on_click", "string", false, None),
        // A secondary (right) click: the `context` event kind.
        p("on_context", "string", false, None),
    ];
    static TEXT: [Port; 12] = [
        p("text", "string", true, None),
        pnd("size", "13", Some(0.0)),
        p("bold", "bool", false, Some("false")),
        p("mono", "bool", false, Some("false")),
        p("color", "string", false, None),
        p("elide", "bool", false, Some("false")),
        p("wrap", "bool", false, Some("false")),
        pnd("max_lines", "3", Some(1.0)),
        pn("width", false, None),
        p("fill", "bool", false, Some("false")),
        p("hidden", "bool", false, Some("false")),
        pe("align", "\"left\"", &["left", "center", "right"]),
    ];
    static FIELD: [Port; 6] = [
        p("value", "string", true, None),
        p("placeholder", "string", false, None),
        pn("width", false, None),
        p("password", "bool", false, Some("false")),
        p("on_change", "string", false, None),
        p("on_submit", "string", false, None),
    ];
    static BUTTON: [Port; 5] = [
        p("label", "string", true, None),
        Port {
            name: "tone",
            ty: "string",
            required: false,
            default: Some("\"normal\""),
            enum_values: TONE,
            min: None,
        },
        pn("width", false, Some(0.0)),
        p("on_click", "string", false, None),
        // A secondary (right) click: the `context` event kind.
        p("on_context", "string", false, None),
    ];
    static TOGGLE: [Port; 3] = [
        p("value", "bool", true, None),
        p("label", "string", true, None),
        p("on_change", "string", false, None),
    ];
    static LIST: [Port; 11] = [
        p("rows", "list", true, None),
        p("row", "string", true, None),
        pn("row_height", true, Some(0.0)),
        pnd("gap", "0", None),
        pn("max_rows", false, Some(1.0)),
        p("fill", "bool", false, Some("false")),
        p("hidden_if_empty", "bool", false, Some("false")),
        p("on_click", "string", false, None),
        // A secondary (right) click: the `context` event kind.
        p("on_context", "string", false, None),
        // No default: absence keeps the historical resolved document and the
        // renderer's vertical list.
        p("flow", "string", false, None),
        p("align", "string", false, None),
    ];
    static IMAGE: [Port; 4] = [
        p("src", "string", true, None),
        pn("w", false, Some(0.0)),
        pn("h", false, Some(0.0)),
        p("color", "string", false, None),
    ];
    static SPACER: [Port; 1] = [pn("size", false, None)];
    static CONTROL: [Port; 2] = [
        p("definition", "object", true, None),
        p("on_intent", "string", false, None),
    ];
    static GRID: [Port; 6] = [
        p("children", "list", true, None),
        pn("columns", true, Some(1.0)),
        p("identity", "string", false, None),
        pnd("column_gap", "12", None),
        pnd("row_gap", "3", None),
        pnd("min_column_width", "0", None),
    ];
    static DOCK: [Port; 8] = [
        p("children", "list", true, None),
        p("placements", "object", true, None),
        p("clip", "bool", false, None),
        p("on_change", "string", false, None),
        p("outline", "string", false, None),
        pn("outline_width", false, Some(0.0)),
        p("passthrough_single", "bool", false, Some("false")),
        p("focus_target", "string", false, None),
    ];
    static BAND: [Port; 2] = [
        p("children", "list", true, None),
        p("groups", "object", true, None),
    ];
    static SPLIT: [Port; 7] = [
        p("children", "list", true, None),
        pnd("ratio", "0.5", None),
        pe("axis", "\"horizontal\"", &["horizontal", "vertical"]),
        pnd("divider", "6", Some(1.0)),
        pnd("minimum", "0.1", None),
        pnd("maximum", "0.9", None),
        p("on_change", "string", false, None),
    ];
    let family: &[Port] = match f {
        "window" => &WINDOW,
        "column" => &BOX,
        "section" => &SECTION,
        "content" => &CONTENT,
        "panes" => &PANES,
        "sides" => &SIDES,
        "row" => &ROW,
        "text" => &TEXT,
        "field" => &FIELD,
        "button" => &BUTTON,
        "toggle" => &TOGGLE,
        "list" => &LIST,
        "image" => &IMAGE,
        "drawing" => &IMAGE,
        "spacer" => &SPACER,
        "control" => &CONTROL,
        "grid" => &GRID,
        "dock" => &DOCK,
        "band" => &BAND,
        "split" => &SPLIT,
        _ => return None,
    };
    let mut ports = family.to_vec();
    if f == "drawing" {
        ports.retain(|p| p.name != "src");
        ports.push(p("primitives", "list", true, None));
        ports.push(pn("sample_time", false, None));
        ports.push(p("shadow", "object", false, None));
    }
    if f == "text" {
        ports.push(pn("letter_spacing", false, Some(-100.0)));
        ports.push(pn("line_height", false, Some(0.1)));
        ports.push(p("font", "string", false, None));
        ports.push(p("mono_font", "string", false, None));
        ports.push(pn("weight", false, Some(1.0)));
        ports.push(p("spans", "list", false, None));
        ports.push(p("number", "object", false, None));
    }
    if f == "grid" {
        ports.push(p("content_columns", "bool", false, None));
        ports.push(p("row_rule", "string", false, None));
    }
    if f == "row" {
        ports.extend([
            Port {
                name: "packing",
                ty: "string",
                required: false,
                default: None,
                enum_values: &["normal", "toolbar"],
                min: None,
            },
            p("end", "string", false, None),
            p("fit", "list", false, None),
            p("flex", "string", false, None),
        ]);
    }
    if f == "row" {
        let mut cross = p("cross_align", "string", false, None);
        cross.enum_values = &["start", "center", "end"];
        ports.push(cross);
        ports
            .iter_mut()
            .find(|p| p.name == "align")
            .unwrap()
            .enum_values = &["start", "center", "end", "stretch", "baseline"];
    }
    if f == "list" {
        ports
            .iter_mut()
            .find(|p| p.name == "flow")
            .unwrap()
            .enum_values = &["vertical", "horizontal"];
        ports
            .iter_mut()
            .find(|p| p.name == "align")
            .unwrap()
            .enum_values = ALIGN;
    }
    // The flex ports have no defaults: absence preserves the legacy mapping
    // and canonical resolved documents. A window is edge metadata, not a
    // flex child.
    if f != "window" {
        ports.extend([
            p("enter", "motion", false, None),
            p("exit", "motion", false, None),
            p("transition", "transitions", false, None),
            p("motion", "bool", false, None),
            pn("opacity", false, Some(0.0)),
            p("offset", "offset", false, None),
            p("paint_effect", "object", false, None),
        ]);
        // Optional Viu host geometry slots. No defaults: existing resolved
        // scenes retain their canonical form and ordinary dock placement.
        ports.push(p("window_slot", "string", false, None));
        ports.push(p("slot_only", "string", false, None));
        if f != "text" {
            ports.push(p("hidden", "bool", false, None));
        }
        ports.push(p("clip", "bool", false, None));
        ports.push(p("enabled", "bool", false, None));
        ports.push(pn("clip_below", false, None));
        ports.extend([
            p("align_self", "string", false, None),
            pn("grow", false, None),
            pn("shrink", false, None),
            pn("basis", false, None),
            pn("min_width", false, None),
            pn("max_width", false, None),
            pn("min_height", false, None),
            pn("max_height", false, None),
            pn("reserve_height", false, None),
        ]);
        ports
            .iter_mut()
            .find(|p| p.name == "align_self")
            .unwrap()
            .enum_values = &["auto", "start", "center", "end", "stretch"];
    }
    // A hover tip: {title, subtitle?, lines?}, shown on a card beside the
    // node after a short hover (the host draws it on its own surface). Only
    // an object is accepted, bound or literal.
    if matches!(f, "row" | "column" | "button") {
        ports.push(p("tooltip", "object", false, None));
    }
    if matches!(f, "row" | "column") {
        ports.push(p("background_inset", "list", false, None));
        ports.push(p("background_radials", "list", false, None));
        ports.push(p("outline", "string", false, None));
        ports.push(pn("outline_width", false, None));
        ports.push(p("scroll", "bool", false, None));
        let mut justify = p("justify", "string", false, None);
        justify.enum_values = &["start", "center", "end", "between", "around", "evenly"];
        ports.extend([
            justify,
            p("bar", "string", false, None),
            p("padding_x", "length", false, None),
            p("padding_y", "length", false, None),
            p("native_spacing", "bool", false, None),
            p("native_frame", "bool", false, None),
            p("group_frame", "bool", false, None),
            p("group_title", "string", false, None),
            p("scroll", "bool", false, None),
            p("group_key", "string", false, None),
            p("scroll_key", "string", false, None),
            p("scroll_ready", "bool", false, None),
            p("reserve_panel_after", "bool", false, None),
            pn("row_gap", false, None),
            pn("column_gap", false, None),
            pn("padding_top", false, None),
            pn("padding_right", false, None),
            pn("padding_bottom", false, None),
            pn("padding_left", false, None),
        ]);
    }
    Some(ports)
}

/// The ports of family `f` for an editor or an agent, or `None` for an
/// unknown family.
pub fn describe(f: &str) -> Option<Vec<PortDescribe>> {
    schema(f).map(|ps| {
        ps.iter()
            .map(|p| PortDescribe {
                path: p.name.into(),
                ty: p.ty.into(),
                mutable: true,
                sensitive: p.name == "value" && f == "field",
                description: format!("{} port of {}", p.name, f),
                enum_values: (!p.enum_values.is_empty())
                    .then(|| p.enum_values.iter().map(|x| (*x).into()).collect()),
                default: p
                    .default
                    .and_then(|x| serde_json::from_str(x).ok())
                    .map(normalize_number),
                min: p.min,
                max: None,
            })
            .collect()
    })
}

pub(crate) fn port_for(family: &str, name: &str) -> Option<Port> {
    schema(family)?.iter().find(|p| p.name == name).copied()
}

pub(crate) fn check_port_value(
    id: &str,
    line: usize,
    p: Port,
    v: &JsonValue,
    out: &mut Vec<Diagnostic>,
) {
    let k = p.name;
    if k == "sample_time" && v.as_f64().is_none_or(|n| !n.is_finite() || n.abs() > 3600.) {
        out.push(Diagnostic::error(
            "motion-value",
            line,
            "sample_time must be finite within +/-3600 seconds",
        ));
    }
    if k == "shadow" {
        if let Err(message) = crate::drawing::validate_shadow(v) {
            out.push(Diagnostic::error("drawing-value", line, message));
        }
    }
    if k == "paint_effect" {
        let valid = v.as_object().is_some_and(|o| o.len() == 2)
            && v["time"]
                .as_f64()
                .is_some_and(|t| t.is_finite() && t.abs() <= 3600.)
            && v["preset"]
                .as_str()
                .is_some_and(|s| motion::scene::paint_preset(s).is_ok());
        if !valid {
            out.push(Diagnostic::error(
                "motion-value",
                line,
                "invalid externally sampled paint effect",
            ));
        }
    }
    if k == "letter_spacing" && v.as_f64().is_none_or(|n| !n.is_finite() || n.abs() > 100.) {
        out.push(Diagnostic::error(
            "port-range",
            line,
            "letter spacing must be within +/-100",
        ));
    }
    if k == "primitives" {
        if let Err(message) = crate::drawing::validate(v) {
            out.push(Diagnostic::error("drawing-value", line, message));
        }
    }
    if k == "background_radials" {
        let valid = v.as_array().is_some_and(|a| {
            a.len() <= 4
                && a.iter().all(|r| {
                    r.as_object().is_some_and(|o| {
                        o.len() == 6
                            && ["x", "y", "rx", "ry", "stop", "color"]
                                .iter()
                                .all(|k| o.contains_key(*k))
                    }) && ["x", "y"].iter().all(|k| {
                        r[k].as_f64()
                            .is_some_and(|n| n.is_finite() && (-2.0..=3.0).contains(&n))
                    }) && ["rx", "ry"].iter().all(|k| {
                        r[k].as_f64()
                            .is_some_and(|n| n.is_finite() && n > 0.0 && n <= 10000.0)
                    }) && r["stop"].as_f64().is_some_and(|n| n > 0.0 && n <= 1.0)
                        && r["color"].as_str().is_some_and(|s| {
                            s.len() == 7
                                && s.starts_with('#')
                                && u32::from_str_radix(&s[1..], 16).is_ok()
                        })
                })
        });
        if !valid {
            out.push(Diagnostic::error(
                "radial-value",
                line,
                format!("invalid bounded background_radials on {id}"),
            ));
        }
    }
    if k == "number" {
        let valid = v.as_object().is_some_and(|o| {
            o.keys()
                .all(|k| matches!(k.as_str(), "from" | "to" | "ms" | "delay_ms" | "group"))
                && v["to"]
                    .as_f64()
                    .is_some_and(|n| n.is_finite() && n.abs() <= 1e15)
                && v.get("from")
                    .is_none_or(|v| v.as_f64().is_some_and(|n| n.is_finite() && n.abs() <= 1e15))
                && ["ms", "delay_ms"].iter().all(|k| {
                    v.get(k).is_none_or(|v| {
                        v.as_f64()
                            .is_some_and(|n| n.is_finite() && (0.0..=60000.0).contains(&n))
                    })
                })
                && v.get("group").is_none_or(JsonValue::is_boolean)
        });
        if !valid {
            out.push(Diagnostic::error(
                "number-value",
                line,
                format!("invalid number track on {id}"),
            ));
        }
    }
    if k == "spans" {
        let valid = v.as_array().is_some_and(|runs| {
            runs.len() <= 1024
                && runs.iter().all(|run| {
                    run.as_object().is_some_and(|o| {
                        o.keys().all(|k| {
                            matches!(
                                k.as_str(),
                                "text"
                                    | "role"
                                    | "color"
                                    | "mono"
                                    | "strike"
                                    | "highlight"
                                    | "bold"
                            )
                        }) && run["text"].as_str().is_some_and(|s| s.len() <= 65536)
                            && run.get("role").is_none_or(|v| {
                                v.as_str().is_some_and(|r| {
                                    [
                                        "text", "keyword", "function", "string", "comment",
                                        "value", "ok", "bad", "warn", "acc", "mut",
                                    ]
                                    .contains(&r)
                                })
                            })
                            && run.get("color").is_none_or(|v| {
                                v.as_str().is_some_and(|s| {
                                    s.len() == 7
                                        && s.starts_with('#')
                                        && s[1..].bytes().all(|b| b.is_ascii_hexdigit())
                                })
                            })
                            && ["mono", "strike", "highlight", "bold"]
                                .iter()
                                .all(|k| run.get(k).is_none_or(JsonValue::is_boolean))
                    })
                })
                && runs
                    .iter()
                    .filter_map(|r| r["text"].as_str())
                    .map(str::len)
                    .sum::<usize>()
                    <= 65536
        });
        if !valid {
            out.push(Diagnostic::error(
                "span-value",
                line,
                format!("invalid or unbounded inline spans on {id}"),
            ));
        }
    }
    let motion_error = match k {
        "enter" | "exit" => serde_json::from_value::<motion::scene::Entrance>(v.clone())
            .map_err(|e| e.to_string())
            .and_then(|e| e.validate().map_err(str::to_owned)),
        "transition" => motion::scene::transitions(v).map(|_| ()),
        "opacity" if v.as_f64().is_some_and(|n| !(0.0..=1.0).contains(&n)) => {
            Err("opacity must be in 0..1".into())
        }
        "offset"
            if !v.as_array().is_some_and(|a| {
                a.len() == 2
                    && a.iter()
                        .all(|n| n.as_f64().is_some_and(|n| n.is_finite() && n.abs() <= 1e6))
            }) =>
        {
            Err("offset must be two finite lengths within +/-1000000".into())
        }
        _ => Ok(()),
    };
    if let Err(message) = motion_error {
        out.push(Diagnostic::error(
            "motion-value",
            line,
            format!("{k} on {id}: {message}"),
        ));
    }
    if k == "background_inset"
        && !v.as_array().is_some_and(|a| {
            a.len() == 4
                && a.iter().all(|n| {
                    n.as_f64()
                        .is_some_and(|n| n.is_finite() && (0.0..=1e6).contains(&n))
                })
        })
    {
        out.push(Diagnostic::error(
            "port-type",
            line,
            format!("background_inset on {id} requires four finite nonnegative lengths"),
        ));
    }
    let ceiling = match k {
        "line_height" | "letter_spacing" => Some(4096.),
        "padding_top" | "padding_right" | "padding_bottom" | "padding_left" | "padding_x"
        | "padding_y" | "outline_width" | "size" | "padding" | "radius" | "w" | "h" | "basis"
        | "width" | "min_width" | "max_width" | "column_gap" | "min_column_width" | "height"
        | "row_gap" | "gap" => Some(1e6),
        "grow" | "shrink" => Some(1000.),
        _ => None,
    };
    if ceiling.is_some_and(|limit| v.as_f64().is_some_and(|n| !n.is_finite() || n > limit)) {
        out.push(Diagnostic::error(
            "port-max",
            line,
            format!("port {k} on {id} exceeds its finite layout ceiling"),
        ));
    }
    if ["ratio", "minimum", "maximum"].contains(&k) && v.as_f64().is_some_and(|n| n > 1.0) {
        out.push(Diagnostic::error(
            "port-max",
            line,
            format!("port {k} on {id} must be at most 1"),
        ));
    }
    if k == "columns" && v.as_f64().is_some_and(|n| n.fract() != 0.0 || n > 16.0) {
        out.push(Diagnostic::error(
            "port-type",
            line,
            format!("port columns on {id} must be an integer from 1 to 16"),
        ));
    }
    if k == "max_lines" && v.as_f64().is_some_and(|n| n.fract() != 0.0 || n > 1024.0) {
        out.push(Diagnostic::error(
            "port-type",
            line,
            format!("port {k} on {id} must be an integer from 1 to 1024"),
        ));
    }
    if !type_matches(v, p.ty) {
        out.push(Diagnostic::error(
            "port-type",
            line,
            format!("port {k} on {id} must be {}", p.ty),
        ));
    }
    if !p.enum_values.is_empty() && v.as_str().is_some_and(|x| !p.enum_values.contains(&x)) {
        out.push(Diagnostic::error(
            "enum-value",
            line,
            format!("invalid value for {k} on {id}"),
        ));
    }
    if let Some(min) = p.min {
        let exclusive = k == "row_height";
        if v.as_f64()
            .is_some_and(|x| if exclusive { x <= min } else { x < min })
        {
            out.push(Diagnostic::error(
                "port-min",
                line,
                format!(
                    "port {k} on {id} must be {} {min}",
                    if exclusive { ">" } else { ">=" }
                ),
            ));
        }
    }
    if k == "rows" {
        validate_rows(v, line, out);
    }
}

/// Every number as an `f64` JSON number, so `13` and `13.0` resolve alike.
pub(crate) fn normalize_number(v: JsonValue) -> JsonValue {
    if let Some(n) = v.as_f64() {
        serde_json::Number::from_f64(n)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null)
    } else if let Some(a) = v.as_array() {
        JsonValue::Array(a.iter().cloned().map(normalize_number).collect())
    } else if let Some(o) = v.as_object() {
        JsonValue::Object(
            o.iter()
                .map(|(k, v)| (k.clone(), normalize_number(v.clone())))
                .collect(),
        )
    } else {
        v
    }
}

fn type_matches(v: &JsonValue, t: &str) -> bool {
    match t {
        "string" => v.is_string(),
        "number" => v.is_number(),
        "length" => {
            v.as_f64().is_some_and(|n| n.is_finite() && n >= 0.0)
                || v.as_str().is_some_and(|s| {
                    matches!(
                        s,
                        "design.bar_inset" | "design.bar_padding_y" | "design.tool_margin"
                    )
                })
        }
        "bool" => v.is_boolean(),
        "list" => v.is_array(),
        "object" => v.is_object(),
        _ => true,
    }
}

fn validate_rows(value: &JsonValue, line: usize, o: &mut Vec<Diagnostic>) {
    if let Some(rs) = value.as_array() {
        if rs.len() > MAX_ROWS {
            o.push(Diagnostic::error(
                "row-limit",
                line,
                "list has more than 500 rows",
            ));
        }
        let mut ids = HashSet::new();
        for r in rs {
            if let Some(id) = r.get("id").and_then(JsonValue::as_str)
                && (id.is_empty() || !ids.insert(id))
            {
                o.push(Diagnostic::error(
                    "row-type",
                    line,
                    "row ids must be non-empty and unique within a list",
                ));
            }
            if r.get("id").and_then(JsonValue::as_str).is_none()
                || r.get("cells")
                    .and_then(JsonValue::as_array)
                    .is_none_or(|cs| cs.iter().any(|c| !c.is_string()))
            {
                o.push(Diagnostic::error(
                    "row-type",
                    line,
                    "each row must contain string id and string cells",
                ));
            }
        }
    }
}
