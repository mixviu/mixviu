//! Toolkit-neutral scene motion declarations. Time is seconds; authored delays
//! and durations are milliseconds. Hosts choose the presentation timestamp.
use crate::{Curve, Direction, Fill, Preset, Values};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Paint effects supported by the externally clocked scene painter. Unlike
/// lifecycle entrances these may require clipping or a text/vector filter.
pub fn paint_preset(name: &str) -> Result<Preset, &'static str> {
    if let Some(legacy) = name.strip_prefix("legacy-") {
        if !["fade", "zoom", "wipe"].contains(&legacy) {
            return Err("unsupported legacy paint preset");
        }
        static LEGACY: std::sync::OnceLock<Vec<Preset>> = std::sync::OnceLock::new();
        return Ok(LEGACY.get_or_init(|| crate::load_presets(include_str!("../legacy-presets.json")).expect("frozen table"))
            .iter().find(|p| p.name == legacy).expect("frozen entrance").clone());
    }
    if ![
        "fade", "up", "down", "left", "right", "pop", "drop", "zoom", "wipe",
    ]
    .contains(&name)
    {
        return Err("unsupported paint preset");
    }
    static TABLE: std::sync::OnceLock<Vec<Preset>> = std::sync::OnceLock::new();
    Ok(TABLE
        .get_or_init(crate::presets)
        .iter()
        .find(|p| p.name == name)
        .expect("canonical paint preset")
        .clone())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Entrance {
    Name(String),
    Options(EntranceOptions),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntranceOptions {
    pub preset: String,
    #[serde(default)]
    pub delay_ms: f64,
    #[serde(default)]
    pub stagger_ms: f64,
}
impl Entrance {
    pub fn options(&self) -> EntranceOptions {
        match self {
            Self::Name(preset) => EntranceOptions {
                preset: preset.clone(),
                delay_ms: 0.0,
                stagger_ms: 0.0,
            },
            Self::Options(options) => options.clone(),
        }
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        let o = self.options();
        if ![o.delay_ms, o.stagger_ms]
            .iter()
            .all(|n| n.is_finite() && (0.0..=60_000.0).contains(n))
        {
            return Err("motion delays must be finite milliseconds in 0..60000");
        }
        // Only finite, representable furniture effects. Other table entries
        // require effects not yet available in the scene painter.
        if !["fade", "up", "down", "left", "right", "pop", "drop"].contains(&o.preset.as_str()) {
            return Err("unsupported scene motion preset");
        }
        Ok(())
    }
    pub fn playback(&self, now: f64, index: u32, exit: bool) -> Preset {
        let o = self.options();
        static TABLE: std::sync::OnceLock<Vec<Preset>> = std::sync::OnceLock::new();
        let mut p = TABLE
            .get_or_init(crate::presets)
            .iter()
            .find(|p| p.name == o.preset)
            .expect("validated scene preset")
            .clone();
        p.timing.beat_start = now;
        p.timing.delay = o.delay_ms / 1000.0;
        p.timing.stagger = o.stagger_ms / 1000.0;
        p.timing.index = index;
        p.timing.fill = Fill::Both;
        if exit {
            p.timing.direction = Direction::Reverse;
        }
        p
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub prop: String,
    pub ms: f64,
    #[serde(default)]
    pub ease: Curve,
}
impl Transition {
    pub fn validate(&self) -> Result<(), &'static str> {
        if ![
            "opacity",
            "offset",
            "color",
            "background",
            "hover",
            "highlight",
        ]
        .contains(&self.prop.as_str())
        {
            return Err("unsupported transition property");
        }
        if !self.ms.is_finite() || !(0.0..=60_000.0).contains(&self.ms) {
            return Err("transition ms must be in 0..60000");
        }
        self.ease.validate()
    }
    pub fn progress(&self, start: f64, now: f64) -> f64 {
        if self.ms == 0.0 || now >= start + self.ms / 1000.0 {
            1.0
        } else {
            self.ease.sample((now - start) * 1000.0 / self.ms)
        }
    }
    pub fn interpolate(&self, start: f64, now: f64, from: f64, to: f64) -> f64 {
        let progress = self.progress(start, now);
        from * (1.0 - progress) + to * progress
    }
}
pub fn transitions(value: &Value) -> Result<Vec<Transition>, String> {
    let list: Vec<Transition> = if value.is_array() {
        serde_json::from_value(value.clone())
    } else {
        serde_json::from_value(value.clone()).map(|t| vec![t])
    }
    .map_err(|e| e.to_string())?;
    let mut seen = std::collections::BTreeSet::new();
    for t in &list {
        t.validate().map_err(str::to_owned)?;
        if !seen.insert(&t.prop) {
            return Err("duplicate transition property".into());
        }
    }
    Ok(list)
}
pub fn active(p: &Preset, now: f64) -> bool {
    now < p.timing.start() + p.timing.duration
}
pub fn sample(p: Option<&Preset>, now: f64, reduced: bool) -> Values {
    if reduced {
        Values::default()
    } else {
        p.map_or_else(Values::default, |p| p.sample(now))
    }
}
