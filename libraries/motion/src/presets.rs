//! The canonical table is JSON, equally loadable by Mix and Rust hosts.
use crate::Preset;

pub const PRESETS_JSON: &str = include_str!("../presets.json");

/// Parse and validate external data once, before entering a render loop.
pub fn load_presets(json: &str) -> Result<Vec<Preset>, String> {
    let presets: Vec<Preset> = serde_json::from_str(json).map_err(|e| e.to_string())?;
    for preset in &presets {
        preset.validate().map_err(str::to_owned)?;
    }
    Ok(presets)
}

pub fn presets() -> Vec<Preset> {
    load_presets(PRESETS_JSON).expect("checked built-in motion table")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_table_is_valid_and_has_all_effects() {
        let p = presets();
        assert_eq!(p.len(), 19);
        for name in ["up", "down", "left", "right", "pop", "zoom", "fade", "wipe", "drop", "flip", "spin", "glow", "kb", "float", "draw", "blink", "bg", "under", "shine"] {
            assert!(p.iter().any(|p| p.name == name));
        }
    }
}
