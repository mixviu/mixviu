//! Bounded vector primitives in logical pixels, with externally sampled motion.
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Primitive {
    pub kind: String,
    pub centre: [f32; 2],
    pub radius: [f32; 2],
    pub colour: String,
    #[serde(default)]
    pub end_colour: Option<String>,
    #[serde(default)]
    pub width: f32,
    #[serde(default)]
    pub start: f32,
    #[serde(default)]
    pub sweep: f32,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub shadow: Option<Shadow>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    pub preset: String,
    #[serde(default)]
    pub delay: f64,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Shadow {
    pub colour: [u8; 4],
    pub offset: [f32; 2],
    pub blur: f32,
}
pub fn validate(value: &serde_json::Value) -> Result<(), String> {
    let primitives: Vec<Primitive> = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    if primitives.len() > 64 { return Err("at most 64 drawing primitives".into()); }
    for p in primitives {
        let colour = |s: &str| s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|b| b.is_ascii_hexdigit());
        if !["arc", "ellipse", "bar"].contains(&p.kind.as_str())
            || !colour(&p.colour) || p.end_colour.as_ref().is_some_and(|s| !colour(s))
            || !p.centre.iter().chain(p.radius.iter()).chain([&p.width, &p.start, &p.sweep]).all(|n| n.is_finite() && n.abs() <= 10000.)
            || p.radius.iter().any(|n| *n <= 0.) || p.width < 0. || p.sweep.abs() > 360.
            || p.effects.len() > 4
            || p.effects.iter().any(|e| !["draw", "pop", "blink", "float", "under"].contains(&e.preset.as_str()) || !e.delay.is_finite() || !(0. ..=60.).contains(&e.delay))
            || p.shadow.is_some_and(|s| !(0. ..=200.).contains(&s.blur) || !s.offset.iter().all(|n| n.is_finite() && n.abs() <= 1000.))
        { return Err("invalid bounded drawing primitive".into()); }
    }
    Ok(())
}
