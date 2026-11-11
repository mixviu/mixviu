// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use std::collections::BTreeSet;

#[test]
fn installed_settings_success_errors_and_override_precedence() {
    if let Some(mode) = std::env::var_os(env::test::TEST_PATH_CACHE)
        .and_then(|s| s.into_string().ok())
        .and_then(|s| s.strip_prefix("setting-").map(str::to_owned))
    {
        if mode != "unregistered" {
            env::register_snapshot("fixture-snapshot");
            env::register_snapshot("fixture-snapshot");
            env::register_snapshot("ignored-later-snapshot");
        }
        // This succeeds even when the file is missing, unreadable or malformed.
        assert_eq!(env::setting(env::COMP_SERVICE).unwrap(), "override");
        let result = env::setting(env::NODED_SERVICE);
        match mode.as_str() {
            "good" => {
                let prefix = PathBuf::from(std::env::var_os(env::PREFIX).unwrap());
                assert_eq!(result.unwrap(), "installed");
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
                    Err(env::SettingError::Malformed { .. })
                ));
            }
            "schema" => assert!(matches!(
                result,
                Err(env::SettingError::UnknownSchema { .. })
            )),
            "mismatch" => assert_eq!(
                result,
                Err(env::SettingError::SnapshotMismatch {
                    name: env::NODED_SERVICE.into(),
                    expected: "fixture-snapshot".into(),
                    actual: "another-snapshot".into(),
                })
            ),
            "unregistered" => assert_eq!(result.unwrap(), "installed"),
            "unreadable" => assert!(matches!(result, Err(env::SettingError::Unreadable { .. }))),
            _ => panic!("unexpected mode"),
        }
        return;
    }
    for mode in [
        "good",
        "nonutf",
        "malformed",
        "schema",
        "mismatch",
        "unreadable",
        "unregistered",
    ] {
        let prefix =
            std::env::temp_dir().join(format!("paths-setting-{}-{mode}", std::process::id()));
        std::fs::create_dir_all(&prefix).unwrap();
        let stamp = "fixture-snapshot";
        let source = match mode {
            "good" | "nonutf" | "unregistered" => format!(
                "{{schema: 1, commit: '{stamp}', {}: 'installed', {}: 'mix:run/hostd/hostd.sock'}}",
                env::NODED_SERVICE,
                env::HOSTD_SOCKET
            ),
            "schema" => "schema: 9\ncommit: 'fixture'".into(),
            "mismatch" => "schema: 1\ncommit: 'another-snapshot'".into(),
            _ => "bad".into(),
        };
        if mode == "unreadable" {
            std::fs::create_dir(prefix.join("env.mx")).unwrap();
        } else {
            std::fs::write(prefix.join("env.mx"), source).unwrap();
        }
        use std::os::unix::ffi::OsStringExt;
        let run = if mode == "nonutf" {
            PathBuf::from(std::ffi::OsString::from_vec(
                b"/srv/fixture/nonutf-\xff".to_vec(),
            ))
        } else {
            prefix.join("run")
        };
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .env_clear()
            .env("HOME", "/home/user")
            .env(env::PREFIX, &prefix)
            .env(env::RUN, run)
            .env(env::COMP_SERVICE, "override")
            .env(env::test::TEST_PATH_CACHE, format!("setting-{mode}"))
            .args([
                "--exact",
                "env_tests::installed_settings_success_errors_and_override_precedence",
            ])
            .output()
            .unwrap();
        std::fs::remove_dir_all(prefix).unwrap();
        assert!(
            output.status.success(),
            "{mode}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
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
    assert!(counts.values().all(|n| *n == 1), "{counts:?}");
}

#[test]
fn strict_prefix_and_derived_roots_preserve_legacy_entry_point() {
    let prefix = Path::new("/srv/fixture/install");
    assert_eq!(
        prefix_from_exe_strict(&prefix.join("bin/mix")).unwrap(),
        prefix
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
        "{{schema: 1, commit: 'abc', {}: 'mix:run/hostd/hostd.sock'}}",
        env::HOSTD_SOCKET
    );
    let defaults = parse(&source, Some("abc")).unwrap();
    assert_eq!(
        defaults.values[env::HOSTD_SOCKET],
        "mix:run/hostd/hostd.sock"
    );
    assert!(matches!(
        parse(&source, Some("def")),
        Err(Error::SnapshotMismatch { .. })
    ));
    assert!(parse(&source, None).is_ok());
    assert!(matches!(
        parse("schema: 2\ncommit: 'abc'", None),
        Err(Error::UnknownSchema(_))
    ));
    for bad in [
        "schema: 1",
        "schema: 1\ncommit: 'a'\ncommit: 'b'",
        "{schema: 1; commit: 'a'}",
        "schema: '1'\ncommit: 'a'",
        "{schema: 1, commit: 'a'",
        "schema: 1\ncommit: run('x')",
    ] {
        assert!(
            matches!(parse(bad, None), Err(Error::Malformed(_))),
            "{bad}"
        );
    }
    assert!(parse(&" ".repeat(65537), None).is_err());
}

#[test]
fn independent_tiers_and_lazy_settings_in_child() {
    if std::env::var_os(env::test::TEST_PATH_CACHE).as_deref()
        == Some(std::ffi::OsStr::new("settings"))
    {
        let prefix = PathBuf::from(std::env::var_os(env::PREFIX).unwrap());
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
        return;
    }
    let prefix = std::env::temp_dir().join(format!("paths-independent-{}", std::process::id()));
    std::fs::create_dir_all(&prefix).unwrap();
    std::fs::write(prefix.join("layout.mx"), "etc: broken(").unwrap();
    let mut cmd = std::process::Command::new(std::env::current_exe().unwrap());
    cmd.env_clear()
        .env("HOME", "/home/user")
        .env(env::PREFIX, &prefix)
        .env(env::test::TEST_PATH_CACHE, "settings")
        .env(env::NODED_SERVICE, "")
        .args([
            "--exact",
            "env_tests::independent_tiers_and_lazy_settings_in_child",
        ]);
    let out = cmd.output().unwrap();
    std::fs::remove_dir_all(prefix).unwrap();
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
