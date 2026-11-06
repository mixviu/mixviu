// SPDX-License-Identifier: MIT OR Apache-2.0
//! `mixd`, the service engine (lab). It carries the Mix interpreter built with
//! the service profile and, later, the headless personalities as argv[0]
//! symlinks, the way MixOS `cli/system` did, but without the shell
//! personality: `mix` is its own engine.
//!
//! - `mixd run FILE [ARGS]`: run a script under the service profile.
//! - `mixd serve FILE [ARGS]`: run a script as a Bus citizen (`--serve`).
//! - `mixd --personalities`: list what this binary carries.
//! - anything else is handed to the service-profile shell unchanged, so a
//!   self re-exec (the job-control stage trampoline, `mix` resolving to
//!   `current_exe`) keeps working.

use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::ExitCode;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Every personality this binary carries, with its component's version.
/// The service shell is always present; daemons join here.
const PERSONALITIES: &[(&str, &str)] = &[
    ("mixd", mix_shell::COMPONENT_VERSION),
    #[cfg(feature = "noded")]
    ("noded", noded::COMPONENT_VERSION),
    #[cfg(feature = "settingsd")]
    ("settingsd", settingsd::COMPONENT_VERSION),
    #[cfg(feature = "editd")]
    ("editd", editd::COMPONENT_VERSION),
];

/// A personality whose main returns `anyhow::Result`, reported the way Rust's
/// own `main` reports an `Err` (Debug form, exit 1), so units see the same
/// behaviour as the standalone binary.
#[allow(dead_code)]
fn exit_of<E: std::fmt::Debug>(result: Result<(), E>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn personality_name(argv0: &OsStr) -> String {
    let base = Path::new(argv0)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    match base.strip_prefix('-') {
        Some(login) => login.to_owned(),
        None => base,
    }
}

fn list() -> String {
    PERSONALITIES
        .iter()
        .map(|(name, version)| format!("{name} {version}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The argv the shell sees for a subcommand: `run` drops the word, `serve`
/// becomes `--serve`. Returns None when argv carries no subcommand.
fn rewrite(args: &[OsString]) -> Option<Vec<OsString>> {
    let argv0 = args.first()?.clone();
    let sub = args.get(1)?;
    let rest = &args[2..];
    let mut out = vec![argv0];
    if sub == "run" {
        if rest.is_empty() {
            return None;
        }
    } else if sub == "serve" {
        if rest.is_empty() {
            return None;
        }
        out.push(OsString::from("--serve"));
    } else {
        return None;
    }
    out.extend(rest.iter().cloned());
    Some(out)
}

fn main() -> ExitCode {
    // A mix -> mixd handoff marks the environment so a shell-profile binary
    // installed as `mixd` refuses instead of looping. This is the service
    // engine: clear it, so scripts and their children start clean.
    // SAFETY: first statement of main; no other thread exists yet.
    unsafe { std::env::remove_var(mix_shell::ENGINE_HANDOFF_ENV) };
    let args: Vec<OsString> = std::env::args_os().collect();
    if args.get(1).is_some_and(|first| first == "--personalities") {
        println!("{}", list());
        return ExitCode::SUCCESS;
    }
    if matches!(args.get(1).and_then(|a| a.to_str()), Some("run" | "serve")) && args.len() < 3 {
        eprintln!("usage: mixd run FILE [ARGS...] | mixd serve FILE [ARGS...]");
        return ExitCode::from(2);
    }
    // Daemon personalities dispatch on argv[0] first. Every other name
    // (mixd, a dev copy, `mix` from a self re-exec, `exe`) is the service
    // shell.
    let personality = args.first().map(|a| personality_name(a)).unwrap_or_default();
    match personality.as_str() {
        #[cfg(feature = "noded")]
        "noded" => noded::main(),
        #[cfg(feature = "settingsd")]
        "settingsd" => exit_of(settingsd::main()),
        #[cfg(feature = "editd")]
        "editd" => exit_of(editd::main()),
        _ => match rewrite(&args) {
            Some(rewritten) => mix_shell::main_with_args(rewritten),
            None => mix_shell::main(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{personality_name, rewrite};
    use std::ffi::{OsStr, OsString};

    fn os(v: &[&str]) -> Vec<OsString> {
        v.iter().map(OsString::from).collect()
    }

    #[test]
    fn run_drops_the_word_and_serve_becomes_the_flag() {
        assert_eq!(rewrite(&os(&["mixd", "run", "a.mix", "x"])), Some(os(&["mixd", "a.mix", "x"])));
        assert_eq!(
            rewrite(&os(&["mixd", "serve", "svc.mix"])),
            Some(os(&["mixd", "--serve", "svc.mix"]))
        );
    }

    #[test]
    fn anything_else_passes_through() {
        assert_eq!(rewrite(&os(&["mixd", "-c", "print(1)"])), None);
        assert_eq!(rewrite(&os(&["mixd", "run"])), None);
        assert_eq!(rewrite(&os(&["mixd"])), None);
    }

    #[test]
    fn the_name_is_the_basename_without_a_login_dash() {
        assert_eq!(personality_name(OsStr::new("/opt/mixos/bin/mixd")), "mixd");
        assert_eq!(personality_name(OsStr::new("-mixd")), "mixd");
    }
}
