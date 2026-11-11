// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The `mix` engine: the Mix shell profile. The service profile and the
//! headless daemons are the `mixd` engine; the shell itself is the
//! `mix-shell` library's `main`.

use std::process::ExitCode;

mod mesh;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// The mix engine is the shell profile. A build that unified mixd's features
// into mix-shell (`cargo build --workspace`) fails here instead of shipping a
// service-profile `mix`. Build each engine with its own `-p`.
const _: () = assert!(
    !mix_shell::HAS_SERVICE_BUILTINS && !mix_shell::HAS_SERVE,
    "mix built with mixd features (Cargo feature unification): build with `cargo build -p mix`, not --workspace"
);

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() == Some(std::ffi::OsStr::new("mesh")) {
        return match mesh::run(args.collect()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Error: {error:?}");
                ExitCode::FAILURE
            }
        };
    }
    mix_shell::main()
}
