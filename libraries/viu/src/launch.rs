// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native launch controls shared by Mix application bundles.
use crate::bundle::Bundle;
use std::path::Path;

fn application_plan(bundle: &Bundle, args: &[String]) -> Result<serde_json::Value, String> {
    let Some(relative) = &bundle.manifest.launch else {
        return Ok(serde_json::json!({}));
    };
    let path = Path::new(relative);
    if path
        .components()
        .any(|p| !matches!(p, std::path::Component::Normal(_)))
    {
        return Err("launch program must be bundle-relative".into());
    }
    let path = bundle
        .root
        .join(path)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !path.starts_with(&bundle.root) {
        return Err("launch program escapes bundle".into());
    }
    let result = std::rc::Rc::new(std::cell::RefCell::new(None));
    let output = result.clone();
    let mut eval = mix::evaluator::Evaluator::new();
    eval.set_entry_script(&path.to_string_lossy());
    eval.register(
        "viu_launch_plan",
        mix::sync_ext(move |args| {
            let [value] = args.as_slice() else {
                return Err(mix::MixError::RuntimeError {
                    msg: "launch plan requires one object".into(),
                    span: None,
                });
            };
            *output.borrow_mut() = Some(crate::runtime::to_json(value)?);
            Ok(mix::value::Value::Nil)
        }),
    );
    let quoted = |value: &serde_json::Value| {
        serde_json::to_string(&value.to_string())
            .unwrap()
            .replace("${", "\\${")
    };
    let source = format!(
        "$app_args=json_parse({})\n{}\nviu_launch_plan(plan($app_args))",
        quoted(&serde_json::json!(args)),
        std::fs::read_to_string(&path).map_err(|e| e.to_string())?
    );
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?
        .block_on(eval.execute_script_source(&source))
        .map_err(|e| e.to_string())?;
    let plan = result
        .borrow_mut()
        .take()
        .ok_or("launch program did not supply a plan")?;
    if !plan.is_object()
        || plan.get("headless").is_some_and(|v| !v.is_boolean())
        || plan.get("activation").is_some_and(|v| {
            v.as_array().is_none_or(|a| {
                a.len() > 8
                    || a.iter().any(|op| {
                        op["command"]
                            .as_str()
                            .is_none_or(|s| s.is_empty() || s.len() > 128)
                            || !op["body"].is_object()
                    })
            })
        })
    {
        return Err("invalid application launch plan".into());
    }
    Ok(plan)
}

fn mix_json(value: &impl serde::Serialize) -> String {
    serde_json::to_string(&serde_json::to_string(value).unwrap())
        .unwrap()
        .replace("${", "\\${")
}

fn policy(bundle: &Bundle, args: &[String]) -> Result<serde_json::Value, String> {
    if !bundle.manifest.launch_model {
        return application_plan(bundle, args);
    }
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
pub fn app_id(bundle: &Bundle) -> String {
    if let Some(app_id) = &bundle.manifest.window_app_id {
        return app_id.clone();
    }
    bundle
        .manifest
        .bus_surface
        .as_ref()
        .and_then(|s| s["app_id"].as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("mixviu.{}", bundle.manifest.service))
}
pub fn load(root: &Path, args: &[String]) -> Result<Bundle, String> {
    let options = parse(args)?;
    let mut bundle = Bundle::load(root, &options.args)?;
    if !bundle.manifest.allow_native_overrides
        && (options.service.is_some() || options.comp.is_some() || options.url.is_some())
    {
        return Err("this launch variant runs alone, without native service overrides".into());
    }
    let mut policy_args = options.args.clone();
    if let Some(index) = options.separator {
        policy_args.insert(index, "--".into());
    }
    let plan = policy(
        &bundle,
        if bundle.manifest.launch_model {
            &policy_args
        } else {
            &options.args
        },
    )?;
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
    if let Some(pattern) = &bundle.manifest.service_pattern {
        if !regex::Regex::new(pattern)
            .map_err(|e| e.to_string())?
            .is_match(&bundle.manifest.service)
        {
            return Err("invalid service name".into());
        }
    }
    bundle.manifest.bus_url = options.url;
    let comp = options
        .comp
        .unwrap_or_else(|| std::env::var("MIX_COMP_SERVICE").unwrap_or_else(|_| "comp".into()));
    let mut plan = bundle.launch_plan.clone();
    plan["comp"] = serde_json::json!(comp);
    bundle.launch = plan.clone();
    bundle.launch_plan = plan.clone();
    let launch = serde_json::to_string(&plan).unwrap();

    bundle.program.source = format!(
        "$app_launch = json_parse({})\n{}",
        serde_json::to_string(&launch)
            .unwrap()
            .replace("${", "\\${"),
        bundle.program.source
    );
    bundle.program.scene = bundle.manifest.service.clone();
    bundle.program.citizen = bundle.manifest.service.clone();
    if let Some(startup) = &mut bundle.startup {
        startup.scene = bundle.manifest.service.clone();
        startup.citizen = bundle.manifest.service.clone();
    }
    Ok(bundle)
}
#[cfg(test)]
mod tests {
    #[test]
    fn workshop_launch_policy_is_mix_owned_and_forwards_macro_before_activation() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop");
        let bundle = super::load(
            &root,
            &["--headless", "--app", "email", "--id", "macro-id"].map(String::from),
        )
        .unwrap();
        assert_eq!(bundle.launch["headless"], true);
        assert_eq!(
            bundle.launch["initial_macro"],
            serde_json::json!({"app":"email","id":"macro-id"})
        );
        assert_eq!(
            bundle.launch["activation"][0]["command"],
            "workshop.macros.open"
        );
        assert_eq!(bundle.launch["activation"][1]["command"], "workshop.show");
        for args in [
            vec!["--bogus"],
            vec!["--bogus", "x"],
            vec!["--app"],
            vec!["--id", ""],
        ] {
            assert!(
                super::load(
                    &root,
                    &args.into_iter().map(String::from).collect::<Vec<_>>()
                )
                .is_err()
            );
        }
    }
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
        assert_eq!(super::app_id(&bundle), "dev.mixos.monitor");
        assert!(bundle.program.source.contains("dev.mixos.monitor"));
        assert!(bundle.program.source.contains("peer"));
        assert!(!bundle.program.source.contains("\\\"--service\\\""));
    }
    #[test]
    fn explicit_window_app_id_survives_service_override() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/term");
        let mut bundle = super::load(&root, &["--service".into(), "term.other".into()]).unwrap();
        assert_eq!(super::app_id(&bundle), "dev.mixos.term");
        assert!(bundle.program.source.contains("dev.mixos.term"));
        bundle.manifest.window_app_id = None;
        assert_eq!(super::app_id(&bundle), "mixviu.term.other");
        bundle.manifest.bus_surface = Some(serde_json::json!({"app_id":"example.terminal"}));
        assert_eq!(super::app_id(&bundle), "example.terminal");
        bundle.manifest.service = "term.third".into();
        assert_eq!(super::app_id(&bundle), "example.terminal");
    }
}
