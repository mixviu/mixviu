// SPDX-License-Identifier: MIT OR Apache-2.0
use std::path::Path;
use viu::runtime::{Program, Runtime};

/// The comparison host defers startup until its explicit profile is supplied.
pub fn initialise(worker: &mut Runtime, app_dir: &Path, profile: &Path) -> Result<(), String> {
    let path = app_dir.join("app.mix");
    worker.load(Program {
        source: "$app_args=nil\n".to_owned() + &std::fs::read_to_string(&path).map_err(|e| e.to_string())?,
        path,
        scene: "dopus".into(),
        citizen: "dopus-mix".into(),
    }).map_err(|e| format!("load Dopus: {e:?}"))?;
    worker.event("dopus.initialise".into(), serde_json::json!({"app_dir":app_dir,"profile":profile}))
        .map_err(|e| format!("initialise Dopus: {e:?}"))?;
    Ok(())
}
