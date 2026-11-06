// SPDX-License-Identifier: MIT OR Apache-2.0
//! Process-local live tracing and the retained native bounded recent-event ring.
//! No settings service, metrics recorder or logging transport is introduced.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::{Mutex, OnceLock};
use tracing_subscriber::{EnvFilter, prelude::*};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
impl Level {
    fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}
type Reload = tracing_subscriber::reload::Handle<EnvFilter, tracing_subscriber::Registry>;
struct Control {
    reload: Reload,
    baseline: String,
    filter: String,
}
static CONTROL: OnceLock<Mutex<Control>> = OnceLock::new();
fn filter_for(level: Level) -> String {
    if level <= Level::Warn {
        return level.name().into();
    }
    let mut filter = "info".to_owned();
    for target in [
        "viu",
        "viu_dopus",
        "dopus_core",
        "scene",
        "mix",
        "bus",
        "files",
        "thumbs",
        "macros",
        "config",
        "design",
    ] {
        filter.push_str(&format!(",{target}={}", level.name()));
    }
    for target in [
        "wgpu_core",
        "wgpu_hal",
        "naga",
        "winit",
        "egui",
        "eframe",
        "tokio",
        "calloop",
    ] {
        filter.push_str(&format!(",{target}=warn"));
    }
    filter
}
fn state(control: &Control) -> Result<Value, String> {
    let level = EnvFilter::try_new(&control.filter)
        .map_err(|e| e.to_string())?
        .max_level_hint()
        .unwrap_or(tracing::level_filters::LevelFilter::TRACE);
    Ok(
        json!({"level":level.to_string().to_lowercase(),"on":level>=tracing::level_filters::LevelFilter::DEBUG,"filter":control.filter,"baseline":control.baseline}),
    )
}
/// Strip only recognised --debug flags before --. Invalid flags refuse startup.
pub fn init(baseline: &str, args: &mut Vec<String>) -> Result<(), String> {
    let mut chosen = None;
    let mut options = true;
    let mut kept = Vec::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 && arg == "--" {
            options = false;
        }
        if i > 0 && options && arg == "--debug" {
            chosen = Some(Level::Debug);
        } else if i > 0
            && options
            && let Some(value) = arg.strip_prefix("--debug=")
        {
            chosen =
                Some(serde_json::from_value::<Level>(json!(value)).map_err(|e| e.to_string())?);
        } else {
            kept.push(arg.clone());
        }
    }
    *args = kept;
    let candidate = std::env::var("RUST_LOG")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| baseline.into());
    let baseline = if EnvFilter::try_new(&candidate).is_ok() {
        candidate
    } else {
        baseline.into()
    };
    let filter = chosen.map(filter_for).unwrap_or_else(|| baseline.clone());
    let parsed = EnvFilter::try_new(&filter).map_err(|e| e.to_string())?;
    let base = EnvFilter::try_new(&baseline).map_err(|e| e.to_string())?;
    crate::debug_recent::configure(&parsed, &base);
    let (layer, reload) = tracing_subscriber::reload::Layer::new(parsed);
    tracing_subscriber::registry()
        .with(layer)
        .with(crate::debug_recent::RecentLayer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(std::io::stderr),
        )
        .try_init()
        .map_err(|e| e.to_string())?;
    CONTROL
        .set(Mutex::new(Control {
            reload,
            baseline,
            filter,
        }))
        .map_err(|_| "debug already initialised")?;
    Ok(())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetRequest {
    level: Option<Level>,
    on: Option<bool>,
}
pub fn request(service: &str, command: &str, body: &str) -> Option<(u8, Value)> {
    let query = command == format!("{service}.debug");
    let recent = command == format!("{service}.debug.recent");
    if !query && !recent && command != format!("{service}.debug.set") {
        return None;
    }
    let result = (|| -> Result<Value, String> {
        if body.len() > 8192 {
            return Err("request exceeds 8192 bytes".into());
        }
        if recent {
            return crate::debug_recent::query(
                serde_json::from_str(body).map_err(|e| e.to_string())?,
            );
        }
        let shape: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
        let mut control = CONTROL
            .get()
            .ok_or("debug logging is not initialised")?
            .lock()
            .map_err(|e| e.to_string())?;
        if query {
            if !shape.as_object().is_some_and(|v| v.is_empty()) {
                return Err("debug query accepts an empty object".into());
            }
        } else {
            if !shape.as_object().is_some_and(|v| v.len() == 1) {
                return Err("provide exactly one of level or on".into());
            }
            let request: SetRequest = serde_json::from_value(shape).map_err(|e| e.to_string())?;
            let filter = match (request.level, request.on) {
                (Some(level), None) => filter_for(level),
                (None, Some(true)) => filter_for(Level::Debug),
                (None, Some(false)) => control.baseline.clone(),
                _ => return Err("provide exactly one of level or on".into()),
            };
            let parsed = EnvFilter::try_new(&filter).map_err(|e| e.to_string())?;
            let baseline = EnvFilter::try_new(&control.baseline).map_err(|e| e.to_string())?;
            control.reload.reload(parsed).map_err(|e| e.to_string())?;
            crate::debug_recent::configure(
                &EnvFilter::try_new(&filter).map_err(|e| e.to_string())?,
                &baseline,
            );
            control.filter = filter;
        }
        state(&control)
    })();
    Some(match result {
        Ok(value) => (0, value),
        Err(error) => (10, json!({"error_code":"INVALID_ARGUMENT","message":error})),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn live_filter_recent_paging_and_refusals_use_the_real_tracing_path() {
        const CHILD: &str = "VIU_DEBUG_CONTRACT_TEST";
        if std::env::var_os(CHILD).is_none() {
            let output=std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact","debug::tests::live_filter_recent_paging_and_refusals_use_the_real_tracing_path"])
                .env(CHILD,"1").env_remove("RUST_LOG").output().unwrap();
            assert!(output.status.success(), "{output:?}");
            return;
        }
        let mut args = vec!["fixture".into()];
        init("warn", &mut args).unwrap();
        assert_eq!(request("test", "test.debug", "{}").unwrap().1["on"], false);
        for body in [
            "{}",
            r#"{"on":null}"#,
            r#"{"on":"yes"}"#,
            r#"{"level":"missing"}"#,
            r#"{"on":true,"level":"debug"}"#,
            r#"{"on":true,"extra":1}"#,
        ] {
            assert_eq!(request("test", "test.debug.set", body).unwrap().0, 10);
        }
        assert!(request("other", "test.debug", "{}").is_none());
        assert_eq!(
            request("test", "test.debug.set", r#"{"on":true}"#)
                .unwrap()
                .0,
            0
        );
        tracing::debug!(target:"viu",count=3,"native diagnostic witness");
        let page = request("test", "test.debug.recent", r#"{"limit":1}"#)
            .unwrap()
            .1;
        assert_eq!(page["recording"], true);
        assert_eq!(page["events"][0]["fields"]["count"], 3);
        assert_eq!(page["events"][0]["message"], "native diagnostic witness");
        let next = request(
            "test",
            "test.debug.recent",
            &json!({"since":page["next"]}).to_string(),
        )
        .unwrap()
        .1;
        assert_eq!(next["events"], json!([]));
        for body in [r#"{"limit":0}"#, r#"{"limit":1001}"#, r#"{"extra":1}"#] {
            assert_eq!(request("test", "test.debug.recent", body).unwrap().0, 10);
        }
        assert_eq!(
            request("test", "test.debug.set", r#"{"on":false}"#)
                .unwrap()
                .1["on"],
            false
        );
        assert!(!crate::debug_recent::allocated());
        assert_eq!(
            request("test", "test.debug.recent", "{}").unwrap().1["events"],
            json!([])
        );
    }
}
