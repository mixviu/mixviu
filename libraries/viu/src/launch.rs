// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native launch controls shared by Mix application bundles.
use crate::bundle::Bundle;
use std::path::Path;

fn mix_json(value: &impl serde::Serialize) -> String {
    serde_json::to_string(&serde_json::to_string(value).unwrap())
        .unwrap()
        .replace("${", "\\${")
}

fn policy(bundle: &Bundle, args: &[String]) -> Result<serde_json::Value, String> {
    let Some(relative) = bundle.manifest.launch.as_ref() else {
        return Ok(serde_json::json!({}));
    };
    let path = bundle
        .root
        .join(relative)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !path.starts_with(&bundle.root) {
        return Err("launch policy escapes bundle".into());
    }
    let source = format!(
        "$app_args=json_parse({})\n{}",
        mix_json(&args),
        std::fs::read_to_string(&path).map_err(|e| e.to_string())?
    );
    let (tx, rx) = std::sync::mpsc::channel();
    let mut runtime = crate::runtime::Runtime::new(
        1,
        std::sync::Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime
        .load(crate::runtime::Program {
            path,
            source,
            scene: bundle.manifest.service.clone(),
            citizen: bundle.manifest.app().to_owned(),
        })
        .map_err(|e| format!("{e:?}"))?;
    rx.recv_timeout(std::time::Duration::from_secs(30))
        .map_err(|e| e.to_string())?;
    let completed = runtime
        .try_completion()
        .ok_or("launch policy did not complete")?;
    if let Some(error) = completed.error {
        return Err(error);
    }
    let plan = completed.model.ok_or("launch policy must publish a plan")?;
    if !plan.is_object()
        || plan["args"].as_array().is_none_or(|args| {
            args.len() > 256
                || args
                    .iter()
                    .any(|a| a.as_str().is_none_or(|s| s.len() > 8192))
        })
        || plan
            .get("stdout")
            .is_some_and(|v| v.as_str().is_none_or(|s| s.len() > 1_048_576))
        || ["headless", "exit"]
            .iter()
            .any(|key| plan.get(*key).is_some_and(|v| !v.is_boolean()))
        || plan.get("activation").is_some_and(|v| {
            v.as_array().is_none_or(|rows| {
                rows.len() > 4
                    || rows.iter().any(|row| {
                        row["command"]
                            .as_str()
                            .is_none_or(|s| s.is_empty() || s.len() > 128)
                            || !row["body"].is_object()
                    })
            })
        })
    {
        return Err("invalid bounded launch plan".into());
    }
    Ok(plan)
}

#[derive(Default, Debug, PartialEq)]
pub struct Options {
    pub service: Option<String>,
    pub comp: Option<String>,
    pub url: Option<String>,
    pub args: Vec<String>,
    pub separator: Option<usize>,
}
fn service_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
pub fn parse(args: &[String]) -> Result<Options, String> {
    let mut out = Options::default();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--" {
            out.separator = Some(out.args.len());
            out.args.extend(args.cloned());
            break;
        }
        if matches!(arg.as_str(), "--service" | "--comp" | "--noded-url") {
            let value = args
                .next()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| format!("{arg} needs a value"))?
                .clone();
            if arg != "--noded-url" && !service_name(&value) {
                return Err("invalid service name".into());
            }
            match arg.as_str() {
                "--service" => out.service = Some(value),
                "--comp" => out.comp = Some(value),
                _ => out.url = Some(value),
            }
        } else {
            out.args.push(arg.clone());
        }
    }
    Ok(out)
}
pub fn load(root: &Path, args: &[String]) -> Result<Bundle, String> {
    let options = parse(args)?;
    let mut bundle = Bundle::load(root, &options.args)?;
    let mut policy_args = options.args.clone();
    if let Some(index) = options.separator {
        policy_args.insert(index, "--".into());
    }
    let plan = policy(&bundle, &policy_args)?;
    if let Some(args) = plan["args"].as_array() {
        bundle = Bundle::load(
            root,
            &args
                .iter()
                .map(|arg| arg.as_str().unwrap().to_owned())
                .collect::<Vec<_>>(),
        )?;
    }
    bundle.launch_plan = plan;
    let app = bundle.manifest.app().to_owned();
    bundle.manifest.app = Some(app);
    if let Some(service) = options.service {
        bundle.manifest.service = service;
    }
    bundle.manifest.bus_url = options.url;
    let comp = options
        .comp
        .unwrap_or_else(|| std::env::var("MIXOS_COMP_SERVICE").unwrap_or_else(|_| "comp".into()));
    let launch = serde_json::to_string(
        &serde_json::json!({"comp":comp,"headless":bundle.launch_plan["headless"]==true}),
    )
    .unwrap();
    bundle.program.source = format!(
        "$app_launch = json_parse({})\n{}",
        serde_json::to_string(&launch)
            .unwrap()
            .replace("${", "\\${"),
        bundle.program.source
    );
    bundle.program.scene = bundle.manifest.service.clone();
    bundle.program.citizen = bundle.manifest.service.clone();
    Ok(bundle)
}
#[cfg(test)]
mod tests {
    #[test]
    fn native_options_never_reach_mix_as_app_arguments() {
        let args = [
            "--service",
            "monitor.test",
            "--comp",
            "comp-test",
            "--noded-url",
            "ws://example.invalid/ws",
            "--node",
            "peer",
        ]
        .map(String::from);
        let parsed = super::parse(&args).unwrap();
        assert_eq!(parsed.service.as_deref(), Some("monitor.test"));
        assert_eq!(parsed.comp.as_deref(), Some("comp-test"));
        assert_eq!(parsed.args, ["--node", "peer"]);
        for args in [
            vec!["--service"],
            vec!["--comp", "bad name"],
            vec!["--noded-url", ""],
        ] {
            assert!(super::parse(&args.into_iter().map(String::from).collect::<Vec<_>>()).is_err());
        }
    }
    #[test]
    fn service_override_keeps_canonical_facilities_and_literal_mix_arguments() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/monitor");
        let bundle = super::load(
            &root,
            &[
                "--service".into(),
                "monitor.test".into(),
                "--comp".into(),
                "comp-test".into(),
                "--node".into(),
                "peer".into(),
            ],
        )
        .unwrap();
        assert_eq!(bundle.manifest.app(), "monitor");
        assert_eq!(bundle.manifest.service, "monitor.test");
        assert!(bundle.program.source.contains("comp-test"));
        assert!(bundle.program.source.contains("peer"));
        assert!(!bundle.program.source.contains("\\\"--service\\\""));
    }
}
