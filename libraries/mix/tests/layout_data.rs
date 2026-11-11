// SPDX-License-Identifier: MIT OR Apache-2.0
//! Same fixtures as paths' reader, through the real load_data builtin.
#[tokio::test(flavor = "current_thread")]
async fn layout_reader_and_load_data_share_a_grammar() {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../paths/tests/layout");
    for entry in std::fs::read_dir(fixtures).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap();
        let source = std::fs::read_to_string(&path).unwrap();
        let quoted = format!("{:?}", path.to_string_lossy()).replace("${", "\\${");
        let loaded = mix::run(&format!("load_data({quoted})")).await;
        let resolved = paths::parse_layout(&source);
        assert_eq!(
            loaded.is_ok(),
            !name.starts_with("syntax_"),
            "load_data: {name}"
        );
        assert_eq!(
            resolved.is_ok(),
            name.starts_with("ok_"),
            "paths reader: {name}"
        );
        if name.starts_with("ok_") {
            let loaded = loaded.unwrap();
            let mix::value::Value::Map(map) = &loaded else {
                panic!("{name}: load_data did not return a map")
            };
            let roots = resolved.unwrap();
            assert_eq!(map.len(), roots.len(), "{name}");
            for (root, value) in roots {
                let Some(mix::value::Value::String(text)) = map.get(root.name()) else {
                    panic!("{name}: missing string root")
                };
                assert_eq!(std::path::Path::new(text), value, "{name}");
            }
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn mix_front_door_selects_the_explicit_tier_and_rejects_old_root_names() {
    use paths::env::test::TEST_LAYOUT_FRONT_DOOR as CHILD;
    const TEST: &str = "mix_front_door_selects_the_explicit_tier_and_rejects_old_root_names";
    if paths::env::raw(CHILD).is_none() {
        struct Scratch(std::path::PathBuf);
        impl Drop for Scratch {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let root =
            Scratch(std::env::temp_dir().join(format!("mix-front-door-{}", std::process::id())));
        std::fs::create_dir(&root.0).unwrap();
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture"])
            .env(CHILD, "1")
            .env(paths::env::PREFIX, &root.0)
            .env("HOME", root.0.join("home"))
            .env("XDG_CONFIG_HOME", root.0.join("config"))
            .env("XDG_DATA_HOME", root.0.join("data"))
            .env("XDG_RUNTIME_DIR", root.0.join("runtime"))
            .env("XDG_CACHE_HOME", root.0.join("cache"))
            .env(paths::env::test::TEST_ENV_EMPTY, "")
            .env(paths::env::test::TEST_ENV_SET, "present")
            .env_remove(paths::env::test::TEST_ENV_UNSET)
            .env_remove(paths::env::ROOT)
            .env_remove(paths::env::ETC)
            .env_remove(paths::env::VAR)
            .env_remove(paths::env::RUN)
            .env_remove(paths::env::SHARE)
            .env_remove(paths::env::CACHE)
            .env_remove(paths::env::LIB)
            .output()
            .unwrap();
        assert!(
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains("test result: ok. 1 passed"),
            "front-door child failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    // Self-location is never used: the child received every path input before
    // the process-global resolver was first called.
    let prefix = paths::env::raw(paths::env::PREFIX).expect("front-door test needs an explicit prefix");
    let evaluator = mix::evaluator::Evaluator::new();
    use mix::evaluator::ShellVarResolver;
    assert_eq!(evaluator.resolve(paths::env::test::TEST_ENV_EMPTY), None);
    assert_eq!(evaluator.resolve(paths::env::test::TEST_ENV_UNSET), None);
    assert_eq!(evaluator.resolve(paths::env::test::TEST_ENV_SET).as_deref(), Some("present"));
    for name in [paths::env::test::TEST_ENV_EMPTY, paths::env::test::TEST_ENV_UNSET] {
        let source = format!("\"${{{name} ?? 'fallback'}}\"");
        assert_eq!(mix::run(&source).await.unwrap(), mix::value::Value::String("fallback".into()));
        assert_eq!(mix::run(&format!("env('{name}', 'fallback')")).await.unwrap(), mix::value::Value::String("fallback".into()));
    }
    assert_eq!(mix::run("mix_prefix()").await.unwrap(), mix::value::Value::String(prefix.to_str().unwrap().into()));
    assert_eq!(mix::run("mix_exe()").await.unwrap(), mix::value::Value::String(std::env::current_exe().unwrap().to_str().unwrap().into()));
    for source in ["mix_prefix('extra')", "mix_exe('extra')"] {
        assert!(mix::run(source).await.is_err(), "unexpected argument accepted: {source}");
    }
    for (tier, uid) in [("system", 0), ("user", 1000)] {
        let inputs = paths::Environment {
            uid,
            ..paths::Environment::current()
        };
        let expected = paths::Layout::from_environment(&inputs).unwrap();
        assert_eq!(
            expected.root(paths::Root::Prefix),
            std::path::PathBuf::from(&prefix)
        );
        let lib = mix::run(&format!("mix_path('lib', '', {{tier: '{tier}'}})"))
            .await
            .unwrap();
        let mix::value::Value::String(lib) = &lib else {
            panic!("lib root did not return a string")
        };
        assert_eq!(
            std::path::Path::new(lib),
            std::path::PathBuf::from(&prefix).join("lib")
        );
        for root in paths::Root::ALL {
            let source = format!(
                "mix_path(\"{}\", \"sentinel\", {{tier: \"{tier}\"}})",
                root.name()
            );
            let value = mix::run(&source).await.unwrap();
            let mix::value::Value::String(value) = &value else {
                panic!("path builtin did not return a string")
            };
            assert_eq!(
                std::path::Path::new(&value),
                expected.path(root, "sentinel").unwrap(),
                "{tier} {root:?}"
            );
        }
    }
    for root in ["config", "state", "runtime", "bin"] {
        assert!(
            mix::run(&format!("mix_path(\"{root}\")")).await.is_err(),
            "old/unknown root accepted: {root}"
        );
    }
    for options in [
        "{tier: 'invalid'}",
        "{tier: 'system', unknown: true}",
        "'system'",
    ] {
        assert!(
            mix::run(&format!("mix_path('etc', '', {options})"))
                .await
                .is_err()
        );
    }
}
