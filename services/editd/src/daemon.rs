// Lifted from src/main.rs so the mixd engine can run editd as an argv[0]
// personality (mixviu engine-split lab). src/main.rs is now a wrapper.
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "editd",
    version,
    about = "The `edit` Bus citizen: shared text buffers for humans and agents"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Register the `edit` Bus service and serve buffers.
    Serve,
}

pub fn main() -> anyhow::Result<()> {
    // --version/-V first, before the tokio runtime exists: a thread- or
    // fd-starved host must still get an answer, not a runtime-build panic.
    buildinfo::exit_on_version!();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build the tokio runtime")
        .block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Serve => crate::bus::serve().await,
    }
}
