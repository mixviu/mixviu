// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native launch controls shared by Mix application bundles.
use crate::bundle::Bundle;
use std::path::Path;

#[derive(Default, Debug, PartialEq)]
pub struct Options {
    pub service: Option<String>,
    pub comp: Option<String>,
    pub url: Option<String>,
    pub args: Vec<String>,
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
    let app = bundle.manifest.app().to_owned();
    bundle.manifest.app = Some(app);
    if let Some(service) = options.service {
        bundle.manifest.service = service;
    }
    bundle.manifest.bus_url = options.url;
    let comp = options
        .comp
        .unwrap_or_else(|| std::env::var("MIXOS_COMP_SERVICE").unwrap_or_else(|_| "comp".into()));
    let launch =
        serde_json::to_string(&serde_json::json!({"comp":comp,"app_id":app_id(&bundle)})).unwrap();
    bundle.program.source = format!(
        "$app_launch = json_parse({})\n{}",
        serde_json::to_string(&launch)
            .unwrap()
            .replace("${", "\\${"),
        bundle.program.source
    );
    bundle.program.scene = bundle.manifest.service.clone();
    bundle.program.citizen = bundle.manifest.service.clone();
    if let Some(startup)=&mut bundle.startup {
        startup.scene=bundle.manifest.service.clone();
        startup.citizen=bundle.manifest.service.clone();
    }
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
