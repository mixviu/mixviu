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
    pub tick_ms: Option<u64>,
    pub surface: String,
    pub frame: Option<String>,
    pub app_id: Option<String>,
    pub drive: Option<String>,
    pub url: Option<String>,
}
impl Default for Manifest {
    fn default() -> Self {
        Self {
            behaviour: "app.mix".into(),
            service: "viu-app".into(),
            title: "Viu".into(),
            size: [800.0, 600.0],
            commands: vec![],
            tick_ms: None,
            surface: "window".into(),
            frame: None,
            app_id: None,
            drive: None,
            url: None,
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
        #[derive(Default, Deserialize)]
        #[serde(default, deny_unknown_fields)]
        struct Options {
            service: Option<String>,
            surface: Option<String>,
            url: Option<String>,
        }
        let (options, args) = if args.first().is_some_and(|s| s == "--host-options") {
            let options: Options =
                serde_json::from_str(args.get(1).ok_or("host options require JSON")?)
                    .map_err(|e| e.to_string())?;
            (options, &args[2..])
        } else {
            (Options::default(), args)
        };
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let path = root.join("contract.json");
        let mut manifest: Manifest = if path.exists() {
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
        if let Some(service) = options.service {
            manifest.service = service;
        }
        if let Some(surface) = options.surface {
            manifest.surface = surface;
        }
        if let Some(url) = options.url {
            manifest.url = Some(url);
        }
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
            || manifest.app_id.as_deref().is_some_and(|id| {
                id.is_empty() || id.len() > 128 || id.chars().any(char::is_whitespace)
            })
            || !["window", "background", "headless"].contains(&manifest.surface.as_str())
            || manifest
                .frame
                .as_deref()
                .is_some_and(|frame| frame != "gold" || manifest.surface == "background")
            || manifest
                .tick_ms
                .is_some_and(|n| !(100..=3_600_000).contains(&n))
        {
            return Err("invalid host size, service, commands or timer budget".into());
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
