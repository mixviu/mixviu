// SPDX-License-Identifier: MIT OR Apache-2.0
//! Local process identity and signalling primitives. Admission and UI policy
//! belong to the Mix owner. Start times stay decimal strings (no float loss).
use serde_json::{Value, json};

fn pid(args: &Value) -> Result<i32, String> {
    args["pid"]
        .as_i64()
        .and_then(|p| i32::try_from(p).ok())
        .filter(|p| *p > 0)
        .ok_or_else(|| "not a process id".into())
}
fn parse_start_time(stat: &str) -> Result<String, String> {
    // comm may contain spaces, newlines and parentheses. Field 3 follows
    // the LAST closing parenthesis; field 22 is its nineteenth successor.
    let (_, rest) = stat.rsplit_once(')').ok_or("invalid process stat")?;
    let value = rest
        .split_whitespace()
        .nth(19)
        .ok_or("missing process start time")?;
    value
        .parse::<u64>()
        .map_err(|_| "invalid process start time")?;
    Ok(value.into())
}
fn start_time(pid: i32) -> Result<String, String> {
    parse_start_time(
        &std::fs::read_to_string(format!("/proc/{pid}/stat")).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                std::io::Error::from_raw_os_error(libc::ESRCH).to_string()
            } else {
                e.to_string()
            }
        })?,
    )
}
pub fn command(command: &str, args: &Value) -> Result<Value, String> {
    match command {
        "snapshot" => {
            let pids = args["pids"]
                .as_array()
                .filter(|p| p.len() <= 100_000)
                .ok_or("expected at most 100000 process ids")?;
            let mut result = serde_json::Map::new();
            for value in pids {
                let id = pid(&json!({"pid":value}))?;
                result.insert(
                    id.to_string(),
                    start_time(id).map(Value::String).unwrap_or(Value::Null),
                );
            }
            Ok(Value::Object(result))
        }
        "signal" => {
            let id = pid(args)?;
            let number = match args["signal"].as_str() {
                Some("SIGTERM" | "TERM" | "term") => libc::SIGTERM,
                Some("SIGKILL" | "KILL" | "kill") => libc::SIGKILL,
                _ => return Err("expected SIGTERM or SIGKILL".into()),
            };
            let live = match start_time(id) {
                Ok(value) => value,
                Err(error) => return Ok(json!({"ok":false,"error":error})),
            };
            let expected = args["start_time"]
                .as_str()
                .map(str::to_owned)
                .or_else(|| args["start_time"].as_u64().map(|n| n.to_string()));
            if expected.as_deref() != Some(live.as_str()) {
                return Ok(json!({"ok":false,"pid_reused":true,"error":"pid reused"}));
            }
            // SAFETY: positive PID only; kernel applies the caller's credentials.
            // The immediately preceding stat read implements the agreed fence.
            if unsafe { libc::kill(id, number) } != 0 {
                return Ok(json!({"ok":false,"error":std::io::Error::last_os_error().to_string()}));
            }
            Ok(json!({"ok":true}))
        }
        _ => Err("unknown process primitive".into()),
    }
}
pub(crate) fn binding() -> mix::evaluator::ExtFn {
    mix::sync_ext(|args| {
        let fail = |msg: String| mix::MixError::RuntimeError { msg, span: None };
        let [mix::value::Value::String(name), args] = args.as_slice() else {
            return Err(fail("viu_process expects command and object".into()));
        };
        let args = crate::runtime::to_json(args)?;
        crate::runtime::from_json(command(name, &args).map_err(fail)?)
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;
    #[test]
    fn stat_identity_handles_embedded_parentheses_and_spaces() {
        let tail = (3..=21).map(|_| "0").collect::<Vec<_>>().join(" ");
        assert_eq!(
            parse_start_time(&format!("123 (name ) weird\n) {tail} 9007199254740993 0")).unwrap(),
            "9007199254740993"
        );
        assert!(parse_start_time("bad").is_err());
    }
    #[test]
    fn fence_and_both_signals_use_only_disposable_children() {
        for (signal, number) in [("SIGTERM", libc::SIGTERM), ("SIGKILL", libc::SIGKILL)] {
            let mut child = std::process::Command::new("sleep")
                .arg("60")
                .spawn()
                .unwrap();
            let id = child.id();
            let snapshot = command("snapshot", &json!({"pids":[id]})).unwrap();
            let refused = command(
                "signal",
                &json!({"pid":id,"signal":signal,"start_time":"0"}),
            )
            .unwrap();
            assert_eq!(refused["error"], "pid reused");
            assert_eq!(refused["pid_reused"], true);
            assert!(child.try_wait().unwrap().is_none());
            let allowed = command(
                "signal",
                &json!({"pid":id,"signal":signal,"start_time":snapshot[id.to_string()]}),
            )
            .unwrap();
            assert_eq!(allowed["ok"], true);
            assert_eq!(child.wait().unwrap().signal(), Some(number));
        }
        for invalid in [0, -1, i64::MAX] {
            assert!(command("signal", &json!({"pid":invalid,"signal":"SIGTERM"})).is_err());
        }
    }
}
