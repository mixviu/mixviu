// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStringExt;
use std::process::{Command, Output};

const STAMP: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER: &str = "fedcba9876543210fedcba9876543210fedcba98";

/// The mode a child run of this test binary was started in, if any.
fn child_mode(prefix: &str) -> Option<String> {
    std::env::var_os(env::test::TEST_PATH_CACHE)?
        .into_string()
        .ok()?
        .strip_prefix(prefix)
        .map(str::to_owned)
}

/// Run one test of `exe` alone, with a cleared environment plus `vars`.
fn child(exe: &Path, test: &str, mode: &str, vars: &[(&str, &OsStr)]) -> Output {
    let mut cmd = Command::new(exe);
    cmd.env_clear()
        .env("HOME", "/home/user")
        .env(env::test::TEST_PATH_CACHE, mode)
        .args(["--exact", test, "--nocapture"]);
    for (key, value) in vars {
        cmd.env(key, value);
    }
    cmd.output().unwrap()
}

fn assert_passed(output: &Output, label: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("test result: ok. 1 passed"),
        "{label}: {stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The registry's installed default for `name`.
fn registry_default(name: &str) -> String {
    let registry = include_str!("../../../etc/env.conf.mix");
    let line = registry
        .lines()
        .find(|l| l.contains(&format!("name: {name:?}")))
        .unwrap();
    let value = line.split("default: ").nth(1).unwrap();
    let value = value.split(',').next().unwrap();
    value
        .trim_matches(|c: char| !c.is_ascii_alphanumeric())
        .into()
}

#[test]
fn installed_settings_success_errors_and_override_precedence() {
    const TEST: &str = "env_tests::installed_settings_success_errors_and_override_precedence";
    if let Some(mode) = child_mode("setting-") {
        if mode != "unregistered" {
            env::register_snapshot(STAMP);
            env::register_snapshot(STAMP);
            // Two conflicting values: reported once, in debug builds.
            env::register_snapshot(OTHER);
            env::register_snapshot("ignored-later-snapshot");
        }
        // These succeed even when the file is missing, unreadable or malformed.
        assert_eq!(env::setting(env::COMP_SERVICE).unwrap(), "override");
        assert_eq!(
            env::setting_or(env::COMP_SERVICE, || unreachable!()).unwrap(),
            "override"
        );
        assert_eq!(
            env::setting(env::VT),
            Err(env::SettingError::NotUtf8 {
                name: env::VT.into()
            })
        );
        assert_eq!(
            env::setting_or(env::NODED_SERVICE, || Some("middle".into())).unwrap(),
            "middle"
        );
        let result = env::setting(env::NODED_SERVICE);
        match mode.as_str() {
            "good" => {
                let prefix = PathBuf::from(std::env::var_os(env::PREFIX).unwrap());
                assert_eq!(result.unwrap(), "installed");
                assert_eq!(
                    env::setting_or(env::NODED_SERVICE, || None).unwrap(),
                    "installed"
                );
                assert_eq!(
                    env::setting_or(env::DEBUG, || None),
                    Err(env::SettingError::NotSet {
                        name: env::DEBUG.into()
                    })
                );
                assert_eq!(
                    env::setting(env::HOSTD_SOCKET).unwrap(),
                    prefix.join("run/hostd/hostd.sock").to_str().unwrap()
                );
                std::fs::write(prefix.join("env.mx"), "bad").unwrap();
                assert_eq!(env::setting(env::NODED_SERVICE).unwrap(), "installed");
            }
            "malformed" => assert!(matches!(result, Err(env::SettingError::Malformed { .. }))),
            "nonutf" => {
                assert_eq!(result.unwrap(), "installed");
                assert!(matches!(
                    env::setting(env::HOSTD_SOCKET),
                    Err(env::SettingError::Layout { .. })
                ));
            }
            "noprefix" => assert!(matches!(result, Err(env::SettingError::NoPrefix { .. }))),
            "schema" => assert!(matches!(
                result,
                Err(env::SettingError::UnknownSchema { .. })
            )),
            "mismatch" => assert_eq!(
                result,
                Err(env::SettingError::SnapshotMismatch {
                    name: env::NODED_SERVICE.into(),
                    expected: STAMP.into(),
                    actual: OTHER.into(),
                })
            ),
            "unregistered" => {
                assert_eq!(result.unwrap(), "installed");
                // The read above skipped the commit check, so a registration
                // now could check nothing: a debug build refuses it.
                let late = std::panic::catch_unwind(|| env::register_snapshot(STAMP));
                assert_eq!(late.is_err(), cfg!(debug_assertions));
            }
            "unreadable" => assert!(matches!(result, Err(env::SettingError::Unreadable { .. }))),
            _ => panic!("unexpected mode"),
        }
        return;
    }
    let exe = std::env::current_exe().unwrap();
    for mode in [
        "good",
        "nonutf",
        "malformed",
        "noprefix",
        "schema",
        "mismatch",
        "unreadable",
        "unregistered",
    ] {
        let prefix =
            std::env::temp_dir().join(format!("paths-setting-{}-{mode}", std::process::id()));
        std::fs::create_dir_all(&prefix).unwrap();
        let source = match mode {
            "good" | "nonutf" | "unregistered" => format!(
                "{{schema: 1, commit: '{STAMP}', {}: 'installed', {}: 'mix:run/hostd/hostd.sock'}}",
                env::NODED_SERVICE,
                env::HOSTD_SOCKET
            ),
            "schema" => format!("schema: 9\ncommit: '{STAMP}'"),
            "mismatch" => format!("schema: 1\ncommit: '{OTHER}'"),
            _ => "bad".into(),
        };
        if mode == "unreadable" {
            std::fs::create_dir(prefix.join("env.mx")).unwrap();
        } else {
            std::fs::write(prefix.join("env.mx"), source).unwrap();
        }
        let run = if mode == "nonutf" {
            OsString::from_vec(b"/srv/fixture/nonutf-\xff".to_vec())
        } else {
            prefix.join("run").into()
        };
        let prefix_var: OsString = if mode == "noprefix" {
            "relative/prefix".into()
        } else {
            prefix.clone().into()
        };
        let not_utf8 = OsString::from_vec(b"\xff".to_vec());
        let output = child(
            &exe,
            TEST,
            &format!("setting-{mode}"),
            &[
                (env::PREFIX, prefix_var.as_os_str()),
                (env::RUN, run.as_os_str()),
                (env::COMP_SERVICE, OsStr::new("override")),
                (env::VT, not_utf8.as_os_str()),
            ],
        );
        std::fs::remove_dir_all(prefix).unwrap();
        assert_passed(&output, mode);
        let conflicts = String::from_utf8_lossy(&output.stderr)
            .matches("conflicting build snapshot")
            .count();
        let expected = usize::from(mode != "unregistered" && cfg!(debug_assertions));
        assert_eq!(conflicts, expected, "{mode}");
    }
}

/// Installed defaults follow the strict prefix rule. A copy of this test
/// binary in `<prefix>/bin` finds `<prefix>/env.mx` from its own location,
/// with no prefix override; the cargo-tree original has no installed
/// defaults at all, so it cannot read a host install's env.mx. The
/// `stage-smoke` stage of tools/ci.mix runs the same child against a real
/// `stage-install`.
#[test]
fn staged_install_resolves_installed_defaults() {
    const TEST: &str = "env_tests::staged_install_resolves_installed_defaults";
    let default = registry_default(env::NODED_SERVICE);
    if let Some(commit) = child_mode("staged:") {
        assert_eq!(env::raw(env::PREFIX), None);
        env::register_snapshot(Box::leak(commit.into_boxed_str()));
        assert_eq!(env::setting(env::NODED_SERVICE).unwrap(), default);
        return;
    }
    if child_mode("cargo-tree").is_some() {
        assert_eq!(env::raw(env::PREFIX), None);
        assert_eq!(
            env::setting(env::NODED_SERVICE),
            Err(env::SettingError::NotSet {
                name: env::NODED_SERVICE.into()
            })
        );
        return;
    }
    let exe = std::env::current_exe().unwrap();
    // Stage beside the test binary, inside the cargo tree, never in /tmp.
    let prefix = exe
        .parent()
        .unwrap()
        .join(format!("paths-stage-{}", std::process::id()));
    std::fs::create_dir_all(prefix.join("bin")).unwrap();
    let staged = prefix.join("bin").join("paths-staged");
    std::fs::hard_link(&exe, &staged)
        .or_else(|_| std::fs::copy(&exe, &staged).map(drop))
        .unwrap();
    std::fs::write(
        prefix.join("env.mx"),
        format!(
            "{{schema: 1, commit: '{STAMP}', {}: '{default}'}}",
            env::NODED_SERVICE
        ),
    )
    .unwrap();
    let staged_out = child(&staged, TEST, &format!("staged:{STAMP}"), &[]);
    let tree_out = child(&exe, TEST, "cargo-tree", &[]);
    std::fs::remove_dir_all(&prefix).unwrap();
    assert_passed(&staged_out, "staged");
    assert_passed(&tree_out, "cargo tree");
}

#[test]
fn settings_prefix_is_the_override_or_the_strict_rule() {
    let with = |prefix: Option<&str>, exe: Option<&str>| Environment {
        prefix: prefix.map(PathBuf::from),
        exe: exe.map(PathBuf::from),
        ..Environment::default()
    };
    assert_eq!(
        settings_prefix(&with(Some("/srv/fixture"), Some("/srv/other/bin/mix"))).unwrap(),
        Some(PathBuf::from("/srv/fixture"))
    );
    assert_eq!(
        settings_prefix(&with(None, Some("/srv/fixture/bin/mix"))).unwrap(),
        Some(PathBuf::from("/srv/fixture"))
    );
    for tree in [
        "/srv/fixture/target/debug/mix",
        "/srv/fixture/target/debug/deps/paths-0123",
    ] {
        assert_eq!(settings_prefix(&with(None, Some(tree))).unwrap(), None);
    }
    assert_eq!(settings_prefix(&with(None, None)).unwrap(), None);
    assert!(settings_prefix(&with(Some("relative"), None)).is_err());
}

#[test]
fn registry_names_and_scope_match_constants_and_literals_are_unique() {
    let registry = include_str!("../../../etc/env.conf.mix");
    let names = include_str!("env_names.rs");
    let mut expected = BTreeSet::new();
    for line in registry.lines().filter(|l| l.contains("{name:")) {
        if line.contains("not an environment variable") {
            continue;
        }
        let name = line
            .split("name: \"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        let id = name.strip_prefix("MIX_").unwrap();
        let test_start = names.find("pub mod test").unwrap();
        let location = names.find(&format!("pub const {id}:"));
        if line.contains("scope: \"test\"") {
            assert!(location.unwrap() > test_start, "{name}");
        } else if let Some(location) = location {
            assert!(location < test_start, "{name}");
        } else {
            assert!(names.contains(&format!("{id} = \"{name}\";")), "{name}");
        }
        expected.insert(name.to_owned());
    }
    let mut actual = BTreeSet::new();
    let mut counts = BTreeMap::<String, usize>::new();
    for entry in std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("src")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let source = std::fs::read_to_string(path).unwrap();
        for literal in source.split('"').skip(1).step_by(2) {
            if literal.starts_with("MIX_")
                && literal.len() > 4
                && literal
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            {
                actual.insert(literal.to_owned());
                *counts.entry(literal.to_owned()).or_default() += 1;
            }
        }
    }
    assert_eq!(actual, expected);
    let repeated: BTreeMap<_, _> = counts.into_iter().filter(|(_, n)| *n > 1).collect();
    assert!(
        repeated.is_empty(),
        "names spelled more than once: {repeated:?}"
    );
}

#[test]
fn strict_prefix_and_derived_roots_preserve_legacy_entry_point() {
    let prefix = Path::new("/srv/fixture/install");
    assert_eq!(
        prefix_from_exe_strict(&prefix.join("bin/mix")).unwrap(),
        prefix
    );
    // The rootfs-install case.
    assert_eq!(
        prefix_from_exe_strict(Path::new("/bin/mix")).unwrap(),
        Path::new("/")
    );
    assert!(prefix_from_exe_strict(&prefix.join("debug/mix")).is_err());
    assert_eq!(prefix_from_exe(&prefix.join("debug/mix")).unwrap(), prefix);
    let env = Environment::default();
    let dirs = Dirs::derived(prefix, &env).unwrap();
    for root in Root::ALL {
        let expected = if root == Root::Prefix {
            prefix.into()
        } else {
            prefix.join(root.name())
        };
        assert_eq!(dirs.root(root), expected);
    }
    let env = Environment {
        root: Some("/srv/fixture/root".into()),
        etc: Some("/srv/fixture/etc".into()),
        lib: Some("/srv/fixture/lib".into()),
        ..env
    };
    let dirs = Dirs::derived(prefix, &env).unwrap();
    assert_eq!(dirs.root(Root::Etc), Path::new("/srv/fixture/etc"));
    assert_eq!(dirs.root(Root::Var), Path::new("/srv/fixture/root/var"));
    assert_eq!(dirs.root(Root::Lib), Path::new("/srv/fixture/lib"));
    let user = Environment {
        uid: 1000,
        home: Some("/home/user".into()),
        ..Environment::default()
    };
    assert_eq!(
        Dirs::derived(prefix, &user).unwrap().root(Root::Etc),
        Path::new("/home/user/.config/mix")
    );
    assert_eq!(
        parse_layout("lib: '/srv/fixture/lib'").unwrap()[&Root::Lib],
        Path::new("/srv/fixture/lib")
    );
}

/// Revision 4: own variable > MIX_ROOT > layout > under the prefix; share
/// and lib skip MIX_ROOT; the user tier keeps XDG below the layout.
#[test]
fn derived_with_layout_has_the_full_precedence() {
    let prefix = Path::new("/srv/fixture/install");
    let layout = "etc: '/srv/fixture/layout/etc'\nvar: '/srv/fixture/layout/var'\nshare: '/srv/fixture/layout/share'";
    let system = Environment {
        uid: 0,
        ..Environment::default()
    };
    let roots = |env: &Environment| {
        let dirs = Dirs::derived_with_layout(prefix, env, layout).unwrap();
        [Root::Etc, Root::Var, Root::Run, Root::Share, Root::Lib].map(|r| dirs.root(r))
    };
    let p = |s: &str| PathBuf::from(s);
    assert_eq!(
        roots(&system),
        [
            p("/srv/fixture/layout/etc"),
            p("/srv/fixture/layout/var"),
            prefix.join("run"),
            p("/srv/fixture/layout/share"),
            prefix.join("lib"),
        ]
    );
    let rooted = Environment {
        root: Some("/srv/fixture/root".into()),
        ..system.clone()
    };
    assert_eq!(
        roots(&rooted),
        [
            p("/srv/fixture/root/etc"),
            p("/srv/fixture/root/var"),
            p("/srv/fixture/root/run"),
            p("/srv/fixture/layout/share"),
            prefix.join("lib"),
        ]
    );
    let own = Environment {
        etc: Some("/srv/fixture/own/etc".into()),
        share: Some("/srv/fixture/own/share".into()),
        lib: Some("/srv/fixture/own/lib".into()),
        ..rooted.clone()
    };
    assert_eq!(
        roots(&own),
        [
            p("/srv/fixture/own/etc"),
            p("/srv/fixture/root/var"),
            p("/srv/fixture/root/run"),
            p("/srv/fixture/own/share"),
            p("/srv/fixture/own/lib"),
        ]
    );
    let user = Environment {
        uid: 1000,
        home: Some("/home/user".into()),
        xdg_runtime_dir: Some("/run/user/1000".into()),
        ..Environment::default()
    };
    assert_eq!(
        roots(&user),
        [
            p("/srv/fixture/layout/etc"),
            p("/srv/fixture/layout/var"),
            p("/run/user/1000/mix"),
            p("/srv/fixture/layout/share"),
            prefix.join("lib"),
        ]
    );
    assert_eq!(
        Dirs::derived(prefix, &system).unwrap(),
        Dirs::derived_with_layout(prefix, &system, "").unwrap()
    );
    assert!(Dirs::derived_with_layout(prefix, &system, "etc: broken(").is_err());
}

#[test]
fn no_home_user_retains_root_home_fallback() {
    let dirs = Dirs::resolve(&Environment {
        uid: 1000,
        ..Environment::default()
    });
    assert_eq!(dirs.root(Root::Etc), Path::new("/root/.config/mix"));
    assert_eq!(dirs.root(Root::Var), Path::new("/root/.local/share/mix"));
}

#[test]
fn defaults_schema_bounds_and_snapshot_errors() {
    use env_document::{Error, parse};
    let source = format!(
        "{{schema: 1, commit: '{STAMP}', {}: 'mix:run/hostd/hostd.sock'}}",
        env::HOSTD_SOCKET
    );
    let defaults = parse(&source, Some(STAMP)).unwrap();
    assert_eq!(
        defaults.values[env::HOSTD_SOCKET],
        "mix:run/hostd/hostd.sock"
    );
    assert!(matches!(
        parse(&source, Some(OTHER)),
        Err(Error::SnapshotMismatch { .. })
    ));
    assert!(parse(&source, None).is_ok());
    assert!(matches!(
        parse(&format!("schema: 2\ncommit: '{STAMP}'"), None),
        Err(Error::UnknownSchema(_))
    ));
    let newline = format!("{}: 'a\nb'", env::COMP_SERVICE);
    let escaped = format!("{}: \"a\\nb\"", env::COMP_SERVICE);
    for bad in [
        "schema: 1".to_owned(),
        format!("schema: 1\ncommit: '{STAMP}'\ncommit: '{STAMP}'"),
        format!("{{schema: 1; commit: '{STAMP}'}}"),
        format!("schema: '1'\ncommit: '{STAMP}'"),
        format!("{{schema: 1, commit: '{STAMP}'"),
        "schema: 1\ncommit: run('x')".to_owned(),
        // The commit is a full lower-case object id.
        "schema: 1\ncommit: 'abc'".to_owned(),
        format!("schema: 1\ncommit: '{}'", STAMP.to_uppercase()),
        format!("schema: 1\ncommit: '{STAMP}0'"),
        // No byte order mark, and no newline in a value, raw or escaped.
        format!("\u{feff}schema: 1\ncommit: '{STAMP}'"),
        format!("schema: 1\ncommit: '{STAMP}'\n{newline}"),
        format!("schema: 1\ncommit: '{STAMP}'\n{escaped}"),
    ] {
        assert!(
            matches!(parse(&bad, None), Err(Error::Malformed(_))),
            "{bad}"
        );
    }
    assert!(parse(&" ".repeat(65537), None).is_err());
}

/// `tools/ci.mix stage-install` writes env.mx with Mix's `data_encode`:
/// one braced line, double-quoted keys and values, comma separated,
/// backslash escapes. The reader must accept exactly that shape.
#[test]
fn defaults_accept_the_shape_data_encode_writes() {
    use env_document::parse;
    let commit = "0".repeat(40);
    let source = format!(
        concat!(
            "{{\"schema\": 1, \"commit\": \"{}\", \"{}\": \"comp\", ",
            "\"{}\": \"mix:run/hostd/hostd.sock\", \"{}\": \"it's \\\"q\\\" \\\\ ok\"}}"
        ),
        commit,
        env::COMP_SERVICE,
        env::HOSTD_SOCKET,
        env::INSTANCE
    );
    let defaults = parse(&source, Some(&commit)).unwrap();
    assert_eq!(defaults.values[env::COMP_SERVICE], "comp");
    assert_eq!(
        defaults.values[env::HOSTD_SOCKET],
        "mix:run/hostd/hostd.sock"
    );
    assert_eq!(defaults.values[env::INSTANCE], "it's \"q\" \\ ok");
    assert_eq!(defaults.values.len(), 3);
}

/// A root reference cannot climb out of its root, so an env.mx default
/// cannot name a path outside the install's roots.
#[test]
fn root_references_cannot_escape_their_root() {
    let dirs = Dirs::derived(
        Path::new("/srv/fixture"),
        &Environment {
            uid: 0,
            ..Environment::default()
        },
    )
    .unwrap();
    assert_eq!(
        dirs.resolve_path("mix:run/hostd/hostd.sock").unwrap(),
        Path::new("/srv/fixture/run/hostd/hostd.sock")
    );
    assert_eq!(
        dirs.resolve_path("mix:run/a/../b").unwrap(),
        Path::new("/srv/fixture/run/b")
    );
    for escape in ["mix:run/..", "mix:run/../../etc/passwd", "mix:etc//abs"] {
        assert!(dirs.resolve_path(escape).is_err(), "{escape}");
    }
}

/// `derived` refuses a relative or `..`-bearing root from any override, for
/// both tiers, naming the root, instead of handing back a root that
/// resolves against the cwd. Every input is an explicit environment.
#[test]
fn derived_refuses_relative_and_parent_overrides_naming_the_root() {
    let prefix = Path::new("/srv/fixture");
    for uid in [0, 1000] {
        let with = |key: &str, value: &str| {
            Environment::from_lookup(
                |k| {
                    if k == key {
                        Some(PathBuf::from(value))
                    } else if k == "HOME" {
                        Some(PathBuf::from("/home/user"))
                    } else {
                        None
                    }
                },
                uid,
                None,
            )
        };
        assert!(Dirs::derived(prefix, &with("unused", "")).is_ok());
        let parent = "/srv/fixture/../elsewhere";
        for root in Root::ALL.into_iter().filter(|r| *r != Root::Prefix) {
            let err = Dirs::derived(prefix, &with(root.env_var(), parent)).unwrap_err();
            assert!(err.starts_with(&format!("root {}: ", root.name())), "{err}");
        }
        for root in [Root::Etc, Root::Var, Root::Run, Root::Cache] {
            let err = Dirs::derived(prefix, &with(root.env_var(), "relative/x")).unwrap_err();
            assert!(err.starts_with(&format!("root {}: ", root.name())), "{err}");
        }
        // A relative share or lib override is ignored, as it always was.
        for root in [Root::Share, Root::Lib] {
            let dirs = Dirs::derived(prefix, &with(root.env_var(), "relative/x")).unwrap();
            assert_eq!(dirs.root(root), prefix.join(root.name()));
        }
        // MIX_ROOT moves etc first, so etc is the root named.
        for bad in [parent, "relative/x"] {
            let err = Dirs::derived(prefix, &with(ROOT_VAR, bad)).unwrap_err();
            assert!(err.starts_with("root etc: "), "{err}");
        }
        for bad in ["relative", "/srv/fixture/../install"] {
            let err = Dirs::derived(Path::new(bad), &with("unused", "")).unwrap_err();
            assert!(err.starts_with("root prefix: "), "{err}");
        }
    }
}

#[test]
fn layout_is_read_once_and_only_a_parse_error_spares_the_user_tier() {
    const TEST: &str = "env_tests::layout_is_read_once_and_only_a_parse_error_spares_the_user_tier";
    if let Some(mode) = child_mode("layout-") {
        let prefix = PathBuf::from(std::env::var_os(env::PREFIX).unwrap());
        match mode.as_str() {
            "broken" => {
                assert_eq!(env::raw(env::NODED_SERVICE), None);
                assert!(matches!(
                    env::setting(env::NODED_SERVICE),
                    Err(env::SettingError::NotSet { .. })
                ));
                // Missing env.mx is cached too: later file creation cannot change it.
                std::fs::write(prefix.join("env.mx"), "schema: 99").unwrap();
                assert!(matches!(
                    env::setting(env::NODED_SERVICE),
                    Err(env::SettingError::NotSet { .. })
                ));
                let user = Layout::for_tier(Tier::User).unwrap();
                assert_eq!(user.root(Root::Etc), Path::new("/home/user/.config/mix"));
                assert!(Layout::for_tier(Tier::System).is_err());
                // The process tier stays strict for a non-root uid too: a
                // service account must not drift to XDG paths behind a
                // broken layout.
                assert!(Layout::discover().is_err());
                assert!(try_root(Root::Etc).is_err());
            }
            "unreadable" => {
                // Read errors are not parse errors: every tier fails.
                assert!(Layout::for_tier(Tier::User).is_err());
                assert!(Layout::for_tier(Tier::System).is_err());
                assert!(Layout::discover().is_err());
            }
            "once" => {
                let etc = Path::new("/srv/fixture/layout/etc");
                assert_eq!(Layout::discover().unwrap().root(Root::Etc), etc);
                std::fs::write(prefix.join("layout.mx"), "etc: '/srv/fixture/other'").unwrap();
                assert_eq!(Layout::for_tier(Tier::System).unwrap().root(Root::Etc), etc);
                assert_eq!(Layout::for_tier(Tier::User).unwrap().root(Root::Etc), etc);
            }
            _ => panic!("unexpected mode"),
        }
        return;
    }
    let exe = std::env::current_exe().unwrap();
    for mode in ["broken", "unreadable", "once"] {
        let prefix =
            std::env::temp_dir().join(format!("paths-layout-{}-{mode}", std::process::id()));
        std::fs::create_dir_all(&prefix).unwrap();
        let layout = prefix.join("layout.mx");
        match mode {
            "broken" => std::fs::write(&layout, "etc: broken(").unwrap(),
            "unreadable" => std::fs::create_dir(&layout).unwrap(),
            _ => std::fs::write(&layout, "etc: '/srv/fixture/layout/etc'").unwrap(),
        }
        let output = child(
            &exe,
            TEST,
            &format!("layout-{mode}"),
            &[
                (env::PREFIX, prefix.as_os_str()),
                (env::NODED_SERVICE, OsStr::new("")),
            ],
        );
        std::fs::remove_dir_all(&prefix).unwrap();
        assert_passed(&output, mode);
    }
}
