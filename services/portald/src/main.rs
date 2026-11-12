// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
use clap::{Parser, Subcommand};
use settings::Binding;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "portald",
    version,
    about = "XDG desktop portal on the session bus: Settings, FileChooser v4 and ScreenCast"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve org.freedesktop.portal.Desktop on the session bus.
    Serve {
        #[arg(long)]
        instance: String,
        /// The settings profile followed. Defaults to the desktop profile
        /// every settings client follows.
        #[arg(long, default_value = settings::follow::PROFILE)]
        profile: String,
        /// Holds the validated appearance cache. Defaults to $STATE_DIRECTORY;
        /// without either the portal runs cache-less.
        #[arg(long)]
        state_dir: Option<PathBuf>,
        /// Also serve org.freedesktop.portal.ScreenCast (needs a compositor
        /// with zwlr_screencopy_manager_v1 and a PipeWire session).
        #[arg(long)]
        screencast: bool,
        /// How screen-share requests are decided: `prompt` asks the person
        /// (or an agent over the Bus); `allow` grants every request, for
        /// headless and test sessions only. Default: MIX_PORTALD_CONSENT.
        #[arg(long)]
        screencast_consent: Option<portald::screencast::consent::Policy>,
        /// Frame-rate ceiling offered to ScreenCast consumers (1..=120).
        /// Default: MIX_PORTALD_MAX_FRAMERATE.
        #[arg(long)]
        screencast_max_framerate: Option<portald::screencast::MaxFramerate>,
    },
}

/// A ScreenCast setting from the environment or the installed defaults,
/// parsed. Every failure names the variable.
fn screencast_setting<T: std::str::FromStr<Err = String>>(name: &str) -> anyhow::Result<T> {
    let value =
        paths::env::setting(name).map_err(|error| anyhow::anyhow!("--screencast: {error}"))?;
    value
        .parse()
        .map_err(|error| anyhow::anyhow!("--screencast: {name}={value:?}: {error}"))
}

/// Everything `--screencast` needs, resolved and checked before anything
/// starts. Settings and FileChooser read none of it.
fn screencast_config(
    consent: Option<portald::screencast::consent::Policy>,
    max_framerate: Option<portald::screencast::MaxFramerate>,
) -> anyhow::Result<portald::screencast::Config> {
    let consent = match consent {
        Some(value) => value,
        None => screencast_setting(paths::env::PORTALD_CONSENT)?,
    };
    let max_framerate = match max_framerate {
        Some(value) => value,
        None => screencast_setting(paths::env::PORTALD_MAX_FRAMERATE)?,
    };
    portald::screencast::remote::name()
        .map_err(|error| anyhow::anyhow!("--screencast: {error}"))?;
    Ok(portald::screencast::Config {
        consent,
        max_framerate,
    })
}

fn main() -> anyhow::Result<()> {
    buildinfo::exit_on_version!();
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .try_init();
    let Command::Serve {
        instance,
        profile,
        state_dir,
        screencast,
        screencast_consent,
        screencast_max_framerate,
    } = Cli::parse().command;
    anyhow::ensure!(
        !screencast || cfg!(feature = "screencast"),
        "built without the screencast feature"
    );
    let screencast = if screencast {
        Some(screencast_config(
            screencast_consent,
            screencast_max_framerate,
        )?)
    } else {
        None
    };
    let binding = Binding { instance, profile };
    binding
        .validate()
        .map_err(|diagnostic| anyhow::anyhow!(diagnostic.message))?;
    let address = portald::env::session_address()?;
    let state_dir = state_dir.or_else(portald::env::state_directory);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        // This I/O daemon needs a small fixed runtime; creating one worker per
        // CPU would make process startup depend on the host's core count.
        .worker_threads(2)
        .enable_all()
        .build()?;
    let result = runtime.block_on(portald::run(binding, address, state_dir, screencast));
    // A blocking task stuck on a wedged compositor (ScreenCast) must not hold
    // up exit: give the rest a moment, then leave.
    runtime.shutdown_timeout(std::time::Duration::from_secs(2));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screencast_framerate_default_and_range() {
        // No built-in rate: an absent flag leaves the choice to the installed
        // MIX_PORTALD_MAX_FRAMERATE, read only with --screencast.
        let Command::Serve {
            screencast_max_framerate,
            profile,
            ..
        } = Cli::try_parse_from(["portald", "serve", "--instance", "test"])
            .unwrap()
            .command;
        assert_eq!(screencast_max_framerate, None);
        assert_eq!(profile, settings::follow::PROFILE);
        assert_eq!(
            "30".parse::<portald::screencast::MaxFramerate>()
                .unwrap()
                .get(),
            30
        );
        for rate in ["1", "60", "120"] {
            assert!(
                Cli::try_parse_from([
                    "portald",
                    "serve",
                    "--instance",
                    "test",
                    "--screencast-max-framerate",
                    rate
                ])
                .is_ok()
            );
        }
        for rate in ["0", "121", "invalid"] {
            let error = Cli::try_parse_from([
                "portald",
                "serve",
                "--instance",
                "test",
                "--screencast-max-framerate",
                rate,
            ])
            .err()
            .unwrap()
            .to_string();
            assert!(error.contains("screencast-max-framerate"), "{error}");
        }
        let negative = Cli::try_parse_from([
            "portald",
            "serve",
            "--instance",
            "test",
            "--screencast-max-framerate",
            "-1",
        ])
        .err()
        .unwrap();
        assert_eq!(negative.kind(), clap::error::ErrorKind::UnknownArgument);
        assert!(negative.to_string().contains("'-1'"), "{negative}");
        let missing = Cli::try_parse_from([
            "portald",
            "serve",
            "--screencast-max-framerate",
            "--instance",
            "test",
        ])
        .err()
        .unwrap();
        assert_eq!(missing.kind(), clap::error::ErrorKind::InvalidValue);
        assert!(
            missing.to_string().contains("a value is required"),
            "{missing}"
        );
        assert!(
            "0".parse::<portald::screencast::MaxFramerate>()
                .unwrap_err()
                .contains("SCREENCAST_MAX_FRAMERATE")
        );
    }
}
