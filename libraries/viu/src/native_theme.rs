// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{Value, json};
pub fn choice(value: &Value) -> Result<toolkit::theme_menu::Choice, String> {
    use design::{CaptionSide, Mode, RequestedDecorations, Scheme, Style};
    fn axis<T>(
        value: &Value,
        key: &str,
        parse: fn(&str) -> Option<T>,
    ) -> Result<Option<T>, String> {
        if value[key].is_null() {
            return Ok(None);
        }
        value[key]
            .as_str()
            .and_then(parse)
            .map(Some)
            .ok_or_else(|| format!("unknown {key}"))
    }
    if !value.is_object() {
        return Err("theme choice must be an object".into());
    }
    let style = if value["style"] == "own" {
        Some(None)
    } else {
        axis(value, "style", Style::from_name)?.map(Some)
    };
    Ok(toolkit::theme_menu::Choice {
        scheme: axis(value, "scheme", Scheme::from_name)?,
        style,
        mode: axis(value, "mode", Mode::from_name)?,
        decorations: axis(value, "decorations", RequestedDecorations::from_name)?,
        captions: axis(value, "caption_side", CaptionSide::from_name)?,
    })
}
pub fn describe(theme: &toolkit::Theme) -> Value {
    json!({"scheme":theme.scheme().name(),"style":theme.style_axis().map_or("own",|s|s.name()),
        "mode":theme.mode().name(),"decorations":theme.decorations().name(),
        "decorations_reason":theme.decoration_resolution().reason,"caption_side":theme.captions().name()})
}
