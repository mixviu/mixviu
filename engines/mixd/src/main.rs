// SPDX-License-Identifier: MIT OR Apache-2.0
//! `mixd`, the service engine (lab). It carries the Mix interpreter built with
//! the service profile and the headless personalities as argv[0] symlinks.
//! It has no shell personality: `mix` is its own engine.
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
    #[cfg(feature = "monitord")]
    ("monitord", monitord::COMPONENT_VERSION),
    #[cfg(feature = "dnsd")]
    ("dnsd", dnsd::COMPONENT_VERSION),
    #[cfg(feature = "blobd")]
    ("blobd", blobd::COMPONENT_VERSION),
    #[cfg(feature = "hostd")]
    ("hostd", hostd::COMPONENT_VERSION),
    #[cfg(feature = "mixtrayd")]
    ("mixtrayd", mixtrayd::COMPONENT_VERSION),
    #[cfg(feature = "maild")]
    ("maild", maild::COMPONENT_VERSION),
    #[cfg(feature = "webd")]
    ("webd", webd::COMPONENT_VERSION),
    #[cfg(feature = "indexd")]
    ("indexd", indexd::COMPONENT_VERSION),
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
    match route(&args) {
        Route::Personality(name) => run_personality(name),
        Route::List => {
            println!("{}", list());
            ExitCode::SUCCESS
        }
        Route::Usage => {
            eprintln!("usage: mixd run FILE [ARGS...] | mixd serve FILE [ARGS...]");
            ExitCode::from(2)
        }
        Route::Shell(Some(rewritten)) => mix_shell::main_with_args(rewritten),
        Route::Shell(None) => mix_shell::main(),
    }
}

/// What an invocation runs. Personalities are decided by argv[0] FIRST and
/// receive their argv untouched: `editd serve` is editd's own subcommand,
/// never mixd's `serve` (desk4 P1 found this: the usage check ran first).
#[derive(Debug, PartialEq)]
enum Route {
    Personality(&'static str),
    List,
    Usage,
    Shell(Option<Vec<OsString>>),
}

fn route(args: &[OsString]) -> Route {
    let name = args
        .first()
        .map(|a| personality_name(a))
        .unwrap_or_default();
    if let Some((p, _)) = PERSONALITIES
        .iter()
        .find(|(p, _)| *p != "mixd" && *p == name)
    {
        return Route::Personality(p);
    }
    if args.get(1).is_some_and(|first| first == "--personalities") {
        return Route::List;
    }
    if matches!(args.get(1).and_then(|a| a.to_str()), Some("run" | "serve")) && args.len() < 3 {
        return Route::Usage;
    }
    Route::Shell(rewrite(args))
}

fn run_personality(name: &str) -> ExitCode {
    match name {
        #[cfg(feature = "noded")]
        "noded" => noded::main(),
        #[cfg(feature = "settingsd")]
        "settingsd" => exit_of(settingsd::main()),
        #[cfg(feature = "editd")]
        "editd" => exit_of(editd::main()),
        #[cfg(feature = "monitord")]
        "monitord" => monitord::main(),
        #[cfg(feature = "dnsd")]
        "dnsd" => dnsd::main(),
        #[cfg(feature = "blobd")]
        "blobd" => exit_of(blobd::main()),
        #[cfg(feature = "hostd")]
        "hostd" => exit_of(hostd::main()),
        #[cfg(feature = "mixtrayd")]
        "mixtrayd" => exit_of(mixtrayd::main()),
        #[cfg(feature = "maild")]
        "maild" => maild::main(),
        #[cfg(feature = "webd")]
        "webd" => webd::main(),
        #[cfg(feature = "indexd")]
        "indexd" => exit_of(indexd::main()),
        other => unreachable!("personality {other} is listed but not dispatched"),
    }
}

#[cfg(test)]
mod tests {
    use super::{Route, personality_name, rewrite, route};

    #[test]
    fn a_personality_keeps_its_own_serve_subcommand() {
        // desk4 P1 regression: `editd serve` hit mixd's usage error.
        let r = route(&[
            OsString::from("/opt/mixos/bin/editd"),
            OsString::from("serve"),
        ]);
        assert_eq!(r, Route::Personality("editd"));
        let r = route(&[OsString::from("noded"), OsString::from("--personalities")]);
        assert_eq!(r, Route::Personality("noded"));
    }

    #[test]
    fn mixd_itself_routes_run_serve_list_and_usage() {
        assert_eq!(
            route(&[OsString::from("mixd"), OsString::from("serve")]),
            Route::Usage
        );
        assert_eq!(
            route(&[OsString::from("mixd"), OsString::from("--personalities")]),
            Route::List
        );
        assert_eq!(
            route(&[
                OsString::from("mixd"),
                OsString::from("run"),
                OsString::from("a.mix")
            ]),
            Route::Shell(Some(vec![OsString::from("mixd"), OsString::from("a.mix")]))
        );
    }
    use std::ffi::{OsStr, OsString};

    fn os(v: &[&str]) -> Vec<OsString> {
        v.iter().map(OsString::from).collect()
    }

    #[test]
    fn run_drops_the_word_and_serve_becomes_the_flag() {
        assert_eq!(
            rewrite(&os(&["mixd", "run", "a.mix", "x"])),
            Some(os(&["mixd", "a.mix", "x"]))
        );
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
