// SPDX-License-Identifier: MIT OR Apache-2.0
//! App-authored launch data. No application names or reducers live here.
use serde::Deserialize;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Manifest {
    pub behaviour: String,
    pub service: String,
    pub title: String,
    pub size: [f32; 2],
    pub commands: Vec<String>,
    pub command_descriptors: Vec<bus::CommandDescriptor>,
    pub tick_ms: Option<u64>,
    pub surface: String,
    pub frame: Option<String>,
    pub follow_settings: bool,
    pub macros: bool,
    pub native_drive: bool,
    pub max_string_bytes: Option<usize>,
    /// App-authored compatibility surface; native facilities append descriptors.
    pub bus_surface: Option<serde_json::Value>,
    pub native_ui: bool,
    pub native_macros: bool,
    pub native_debug: bool,
}
impl Default for Manifest {
    fn default() -> Self {
        Self {
            behaviour: "app.mix".into(),
            service: "viu-app".into(),
            title: "Viu".into(),
            size: [800.0, 600.0],
            commands: vec![],
            command_descriptors: vec![],
            tick_ms: None,
            surface: "window".into(),
            frame: None,
            follow_settings: false,
            macros: false,
            native_drive: false,
            max_string_bytes: None,
            bus_surface: None,
            native_ui: false,
            native_macros: false,
            native_debug: false,
        }
    }
}
pub struct Bundle {
    pub root: PathBuf,
    pub manifest: Manifest,
    pub scene: String,
    pub program: crate::runtime::Program,
}
impl Bundle {
    pub fn load(root: &Path, args: &[String]) -> Result<Self, String> {
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let path = root.join("contract.json");
        let manifest: Manifest = if path.exists() {
            let value: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            serde_json::from_value(
                value
                    .get("host")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({})),
            )
            .map_err(|e| e.to_string())?
        } else {
            Manifest::default()
        };
        if !manifest
            .size
            .iter()
            .all(|n| n.is_finite() && (64.0..=8192.0).contains(n))
            || manifest.service.is_empty()
            || manifest.service.len() > 64
            || !manifest
                .service
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || manifest.commands.len() > 256
            || manifest.command_descriptors.len() > 256
            || manifest.command_descriptors.iter().any(|d| !manifest.commands.contains(&d.name) || d.description.len()>8192 || d.args.len()>64)
            || manifest
                .max_string_bytes
                .is_some_and(|n| !(1_048_576..=32 * 1024 * 1024).contains(&n))
            || !["window", "background"].contains(&manifest.surface.as_str())
            || manifest
                .frame
                .as_deref()
                .is_some_and(|frame| frame != "gold" || manifest.surface != "window")
            || manifest
                .tick_ms
                .is_some_and(|n| !(100..=3_600_000).contains(&n))
        {
            return Err("invalid host size, service, commands or timer budget".into());
        }
        if let Some(surface)=&manifest.bus_surface {
            if !surface.is_object() || surface["commands"].as_array().is_none_or(|a|a.len()>128 || a.iter().any(|c|c["name"].as_str().is_none_or(|s|s.is_empty() || s.len()>128) || !c["description"].is_string() || !c["read_only"].is_boolean())) {
                return Err("invalid app Bus surface".into());
            }
        }
        let relative = Path::new(&manifest.behaviour);
        if relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err("behaviour must be bundle-relative".into());
        }
        let path = root
            .join(relative)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !path.starts_with(&root) {
            return Err("behaviour escapes bundle".into());
        }
        // JSON is valid Mix data except null (arguments are strings only).
        let source = format!(
            "$app_args = json_parse({})\n{}",
            serde_json::to_string(&serde_json::to_string(args).unwrap())
                .unwrap()
                .replace("${", "\\${"),
            std::fs::read_to_string(&path).map_err(|e| e.to_string())?
        );
        let scene = std::fs::read_to_string(root.join("scene.mix")).map_err(|e| e.to_string())?;
        let program = crate::runtime::Program {
            path,
            source,
            scene: manifest.service.clone(),
            citizen: manifest.service.clone(),
        };
        Ok(Self {
            root,
            manifest,
            scene,
            program,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundles_refuse_escape_and_invalid_host_data() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("app.mix"), "print($app_args)").unwrap();
        std::fs::write(dir.path().join("scene.mix"), "scene").unwrap();
        assert!(
            Bundle::load(dir.path(), &["a\"b".into()])
                .unwrap()
                .program
                .source
                .starts_with("$app_args = json_parse(")
        );
        for host in [
            serde_json::json!({"behaviour":"../app.mix"}),
            serde_json::json!({"tick_ms":1}),
            serde_json::json!({"size":[0,100]}),
        ] {
            std::fs::write(
                dir.path().join("contract.json"),
                serde_json::json!({"host":host}).to_string(),
            )
            .unwrap();
            assert!(Bundle::load(dir.path(), &[]).is_err());
        }
    }
}
