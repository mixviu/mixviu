// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Shell source-checkout discovery, separate from installed paths.
//!
//! The explicit root override wins, then an ancestor of this executable with
//! both AGENTS.md and Cargo.toml. Without either there is no source checkout.
//! The source override can independently select source tooling; binary and
//! shared-data directories use the common paths resolver.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir {
    Bin,
    // Mirror the shared path contract even before a Mix caller needs this kind.
    #[allow(dead_code)]
    Share,
}

struct ResolvedPaths {
    root: Option<PathBuf>,
    src: Option<PathBuf>,
    bin: PathBuf,
    share: PathBuf,
}

static PATHS: OnceLock<ResolvedPaths> = OnceLock::new();

pub fn mix_path(kind: Dir) -> PathBuf {
    let paths = PATHS.get_or_init(resolve_all);
    match kind {
        Dir::Bin => paths.bin.clone(),
        Dir::Share => paths.share.clone(),
    }
}

/// The source root when it is known — from the environment
/// or by self-location — else `None`. Cached.
pub fn mix_root() -> Option<PathBuf> {
    PATHS.get_or_init(resolve_all).root.clone()
}

pub fn mix_src() -> Option<PathBuf> {
    PATHS.get_or_init(resolve_all).src.clone()
}

pub fn current_uid() -> u32 {
    unsafe { libc::getuid() }
}

/// A directory is a MixViu root iff it carries the two files every
/// checkout has and no runtime tree does.
fn is_root(dir: &Path) -> bool {
    dir.join("AGENTS.md").is_file() && dir.join("Cargo.toml").is_file()
}

/// Resolve the explicit root override, else the
/// nearest of the running binary's first six ancestors that [`is_root`].
pub fn locate_root(env_root: Option<PathBuf>, exe: Option<&Path>) -> Option<PathBuf> {
    if let Some(root) = env_root {
        return Some(root);
    }
    exe?.ancestors()
        .skip(1)
        .take(6)
        .find(|d| is_root(d))
        .map(Path::to_path_buf)
}

fn resolve_all() -> ResolvedPaths {
    let exe = std::env::current_exe().ok();
    let root = locate_root(
        ::paths::env::raw(::paths::env::ROOT).map(PathBuf::from),
        exe.as_deref(),
    );

    let src = ::paths::env::raw(::paths::env::SRC)
        .map(PathBuf::from)
        .or_else(|| root.clone());

    let bin = env_or(::paths::env::BIN, || ::paths::bin(""));

    let share = config::path(config::Dir::Share);

    ResolvedPaths {
        root,
        src,
        bin,
        share,
    }
}

fn env_or(var: &str, fallback: impl FnOnce() -> PathBuf) -> PathBuf {
    ::paths::env::raw(var)
        .map(PathBuf::from)
        .unwrap_or_else(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_root_wins_over_self_location() {
        let root = locate_root(
            Some(PathBuf::from("/srv/mix")),
            Some(Path::new("/nowhere/bin/mix")),
        );
        assert_eq!(root, Some(PathBuf::from("/srv/mix")));
    }

    #[test]
    fn self_location_finds_the_checkout_above_bin_and_above_target() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("MixOS");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("bin")).unwrap();
        std::fs::create_dir_all(root.join("src/target/release")).unwrap();
        std::fs::write(root.join("AGENTS.md"), "").unwrap();
        std::fs::write(root.join("Cargo.toml"), "").unwrap();
        assert_eq!(
            locate_root(None, Some(&root.join("bin/mix"))),
            Some(root.clone())
        );
        assert_eq!(
            locate_root(None, Some(&root.join("src/target/release/mix"))),
            Some(root.clone())
        );
        // a system install has no checkout above it
        assert_eq!(
            locate_root(None, Some(Path::new("/srv/fixture/bin/mix"))),
            None
        );
    }

    #[test]
    fn a_runtime_tree_is_not_a_root() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("bin")).unwrap();
        assert_eq!(locate_root(None, Some(&tmp.path().join("bin/mix"))), None);
    }
}
