// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Local freedesktop icon lookup for scene images, over the generic resolver
//! (`crate::freedesktop`): the apps service usually supplies a path; names and
//! missing paths also need a useful icon. `crate::images` decodes SVG/PNG at
//! the surface's output scale. The lookup follows the XDG environment of the
//! compositor process.

use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::freedesktop::{Lookup, Resolver};

pub(crate) fn is_svg(path: &Path) -> bool {
    crate::freedesktop::is_svg(path)
}

pub(crate) fn with_resolver<T>(f: impl FnOnce(&Resolver, u64) -> T) -> T {
    static STATE: OnceLock<Mutex<(Lookup, Resolver, u64, u64)>> = OnceLock::new();
    let lookup = Lookup::from_xdg();
    let mut state = STATE.get_or_init(|| Mutex::new((lookup.clone(), Resolver::new(lookup.clone()), 0, 0))).lock().unwrap();
    refresh(&mut state, lookup);
    f(&state.1, state.3)
}

fn refresh(state: &mut (Lookup, Resolver, u64, u64), lookup: Lookup) {
    if state.0 != lookup {
        state.1 = Resolver::new(lookup.clone());
        state.0 = lookup;
        state.2 = 0;
    }
    let generation = state.1.generation();
    if state.2 != generation {
        state.2 = generation;
        state.3 += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn theme_changes_invalidate_misses_and_prepared_generation() {
        let root = tempfile::tempdir().unwrap();
        let first = Lookup::in_data_dirs("First", [root.path().into()]);
        let mut state = (first.clone(), Resolver::new(first.clone()), 0, 0);
        refresh(&mut state, first);
        assert!(state.1.resolve("new-icon", 20, None).is_none());
        let generation = state.3;
        let path = root.path().join("icons/Second/32x32/apps/new-icon.svg");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "svg").unwrap();
        refresh(&mut state, Lookup::in_data_dirs("Second", [root.path().into()]));
        assert!(state.3 > generation);
        assert_eq!(state.1.resolve("new-icon", 20, None), Some(path));
    }

    #[test]
    fn resolver_follows_the_process_environment() {
        let lookup = Lookup::from_xdg();
        assert!(!lookup.theme.is_empty());
        assert!(!lookup.roots.is_empty());
        assert!(is_svg(Path::new("icon.SVG")));
    }

    #[test]
    #[ignore = "requires MIX_VIUD_ICON_ROOT pointing to an installed icon tree"]
    fn installed_icons_resolve_without_generic_placeholders() {
        let root =
            PathBuf::from(std::env::var_os("MIX_VIUD_ICON_ROOT").expect("MIX_VIUD_ICON_ROOT"));
        let lookup = Lookup::in_data_dirs(
            "Adwaita",
            [root.join("usr/local/share"), root.join("usr/share")],
        );
        for id in ["bssh", "bvnc", "avahi-discover", "dev.mixos.media"] {
            let path = lookup
                .resolve("", 32, Some(id))
                .unwrap_or_else(|| panic!("missing {id}"));
            let stem = path.file_stem().unwrap().to_string_lossy();
            assert!(
                !matches!(
                    stem.as_ref(),
                    "application-x-executable" | "application-default-icon"
                ),
                "{id}: {}",
                path.display()
            );
            assert!(path.is_file());
            println!("{id}: {}", path.display());
        }
    }
}
