// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

//! noded — Consolidated node daemon.
//!
//! One process running the broker (WebSocket message broker), system
//! monitor, and Bus traffic logger as async tasks. The `noded` binary, and
//! the mixd engine started as `noded`, both run [`main`].

use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;

// The global allocator belongs to each binary (src/main.rs, or the system
// binary), never to this library.

mod admission;
mod authority;
mod log_props;
mod logger;
mod mon_props;
mod monitor;
mod native_ingress;
mod noded;
mod observe;
mod props;
mod props_reservation;
mod protection;
mod routing;
mod spec;
mod spec_release;
mod subscription;
mod tls_ingress;

/// `--version` line including git sha + build time (build-provenance
/// contract). `MIX_*` are set by `build.rs` → `buildinfo::emit()`.
///
/// NB: this uses `env!` (not `option_env!`) deliberately — a daemon that
/// advertises a provenance version line MUST have the `build.rs`; if a
/// future daemon copies this `const` but forgets `build.rs`, it fails to
/// compile loudly rather than silently shipping `"unknown"`. (The
/// `build_info!()` macro, by contrast, uses `option_env!` so library
/// crates without a `build.rs` still build.)
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("MIX_GIT_SHA"),
    ", built ",
    env!("MIX_BUILD_TIME"),
    ")"
);

#[derive(Parser)]
#[command(name = "noded", version = VERSION, about = "Native ABP node daemon")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Start the node daemon (default)
    Serve {
        /// Listen address override (default: derived from node.conf.mix)
        #[arg(long)]
        listen: Option<String>,

        /// Node name override (default: from node.conf.mix)
        #[arg(long)]
        node: Option<String>,

        /// Path to mesh config file (default: from node.conf.mix)
        #[arg(long)]
        mesh_config: Option<String>,

        /// Disable the system monitor module
        #[arg(long)]
        no_monitor: bool,

        /// Disable the Bus traffic logger module
        #[arg(long)]
        no_log: bool,

        /// Path to the `_spec/` directory (default: discovered via env/cwd)
        #[arg(long)]
        spec_dir: Option<String>,
    },
}

/// This component's version, for the mixd engine's `--personalities`.
pub const COMPONENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The whole noded program: the `noded` binary and the mixd engine's
/// `noded` personality are thin calls into this.
pub fn main() -> ExitCode {
    // --version/-V first, before the tokio runtime exists: a thread- or
    // fd-starved host must still get an answer, not a runtime-build panic.
    buildinfo::exit_on_version!();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build the tokio runtime");
    let result = runtime.block_on(async_main());
    // A wedged history syscall must not hold process shutdown in runtime Drop.
    runtime.shutdown_timeout(std::time::Duration::ZERO);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // What `fn main() -> anyhow::Result<()>` prints on error.
            eprintln!("Error: {error:?}");
            ExitCode::FAILURE
        }
    }
}

async fn async_main() -> Result<()> {
    let _log = logging::init(
        &logging::LogOpts::default(),
        &logging::StatsOpts::default(),
        logging::LogDefaults::daemon("noded").with_stats(false),
    )?;

    let cli = Cli::parse();

    // Extract CLI args (default to Serve if no subcommand)
    let (cli_listen, cli_node, cli_mesh_config, no_monitor, no_log, cli_spec_dir) =
        match cli.command {
            Some(Command::Serve {
                listen,
                node,
                mesh_config,
                no_monitor,
                no_log,
                spec_dir,
            }) => (listen, node, mesh_config, no_monitor, no_log, spec_dir),
            None => (None, None, None, false, false, None),
        };

    // Load node.conf.mix, then apply CLI overrides
    let node_cfg = config::node::require_node_config()?;
    let listen = cli_listen.unwrap_or_else(|| node_cfg.noded_listen());
    let node = cli_node.unwrap_or_else(|| node_cfg.node.clone());
    let mesh_config = cli_mesh_config.or_else(|| node_cfg.noded.mesh_config.clone());
    let history_config = if no_log {
        None
    } else {
        logger::HistoryConfig::load(mesh_config.as_deref())
    };

    let noded_url = format!("unix://{}", node_cfg.noded.broker_unix_endpoint().display());

    let spec_dir = spec::locate_spec_dir(cli_spec_dir.as_deref());
    tracing::info!(
        node = %node,
        listen = %listen,
        spec_dir = ?spec_dir,
        "Starting noded",
    );

    // Start the broker with a readiness signal
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let noded_node = node.clone();
    let admission_mode = node_cfg.noded.admission;
    let frame_policy = noded::frame_policy_config(
        &node_cfg.noded,
        std::env::var(paths::env::NODED_FRAME_POLICY)
            .ok()
            .as_deref(),
    )?;
    let mesh_addr = node_cfg.mesh_addr.clone();
    let unix_socket = Some(node_cfg.noded.broker_unix_endpoint());
    let pending_grants_per_parent = node_cfg.noded.pending_grants_per_parent;
    let network = Some(node_cfg.noded.clone());
    let tasks = noded::task_identity::TaskIdentities {
        logger: history_config.as_ref().map(|_| noded::task_identity::Token::new()),
        monitor: (!no_monitor).then(noded::task_identity::Token::new),
    };
    let broker_tasks = tasks.clone();
    let mut noded_handle = tokio::spawn(async move {
        noded::run_with_tasks(
            noded::RunConfig {
                network,
                #[cfg(test)]
                session_probe: None,
                listen,
                node: noded_node,
                mesh_addr,
                mesh_config_path: mesh_config,
                spec_dir,
                admission_mode,
                frame_policy,
                unix_socket,
                pending_grants_per_parent,
            },
            ready_tx,
            broker_tasks,
        )
        .await
    });

    // Wait for the broker listener to be bound
    if ready_rx.await.is_err() {
        return match noded_handle.await {
            Ok(Err(error)) => Err(error.context("Broker failed to start")),
            Ok(Ok(())) => Err(anyhow::anyhow!(
                "Broker exited without signalling readiness"
            )),
            Err(error) => Err(anyhow::anyhow!("Broker startup task failed: {error}")),
        };
    }

    // Spawn client modules
    let monitor_handle = if !no_monitor {
        let mon_url = noded_url.clone();
        Some(tokio::spawn(async move {
            if let Err(e) = monitor::run(&mon_url, tasks.monitor.expect("enabled monitor token")).await {
                tracing::error!("Monitor module failed: {e}");
            }
        }))
    } else {
        None
    };

    let (logger_stop, logger_shutdown) = tokio::sync::oneshot::channel();
    let logger_handle = if let Some(history_config) = history_config {
        let log_url = noded_url.clone();
        Some(tokio::spawn(async move {
            if let Err(e) = logger::run(&log_url, node, history_config, logger_shutdown, tasks.logger.expect("enabled logger token")).await {
                tracing::error!("Logger module failed: {e}");
            }
        }))
    } else {
        None
    };

    // Wait for shutdown signal or broker failure
    let broker_exit = tokio::select! {
        _ = shutdown_signal() => {
            tracing::info!("Shutdown signal received");
            None
        }
        result = &mut noded_handle => {
            Some(result)
        }
    };

    // Cleanup — abort remaining tasks
    if let Some(h) = monitor_handle {
        h.abort();
    }
    if let Some(h) = logger_handle {
        let _ = logger_stop.send(());
        stop_logger(h).await;
    }

    if let Some(result) = broker_exit {
        return match result {
            Ok(Ok(())) => anyhow::bail!("Broker exited unexpectedly"),
            Ok(Err(error)) => Err(error),
            Err(error) => anyhow::bail!("Broker task failed: {error}"),
        };
    }

    tracing::info!("noded stopped");
    Ok(())
}

async fn stop_logger(mut handle: tokio::task::JoinHandle<()>) {
    if tokio::time::timeout(std::time::Duration::from_secs(2), &mut handle)
        .await
        .is_err()
    {
        tracing::warn!("History shutdown exceeded 2 seconds; aborting logger");
        handle.abort();
    }
}

async fn shutdown_signal() {
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("install SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = terminate.recv() => {},
    }
}
