// SPDX-License-Identifier: MIT OR Apache-2.0
//! A job-owned install: subprocesses never borrow the host's prefix.
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

pub fn prefix() -> &'static Path {
    static PREFIX: OnceLock<tempfile::TempDir> = OnceLock::new();
    PREFIX.get_or_init(|| {
        let source = Path::new(env!("CARGO_BIN_EXE_mix"));
        let prefix = tempfile::tempdir_in(source.parent().unwrap()).expect("temporary install prefix");
        std::fs::create_dir(prefix.path().join("bin")).unwrap();
        for name in ["mix", "mixd"] {
            let binary = source.with_file_name(name);
            if binary.is_file() {
                std::fs::hard_link(&binary, prefix.path().join("bin").join(name)).unwrap();
            }
        }
        let bytes = std::fs::read(source).unwrap();
        let markers = buildinfo::find_markers(&bytes);
        let commit = markers.iter().find_map(|marker| {
            let info: serde_json::Value = serde_json::from_str(marker).ok()?;
            (info["component"] == "mix-shell")
                .then(|| info["git_sha_full"].as_str().unwrap().to_owned())
        }).expect("built shell provenance");
        let mut defaults = serde_json::Map::new();
        defaults.insert("schema".into(), 1.into());
        defaults.insert("commit".into(), commit.into());
        let registry = mix::parse_data(include_str!("../../../../etc/env.conf.mix")).unwrap();
        let mix::Value::Map(registry) = &registry else { panic!("registry map") };
        let mix::Value::List(vars) = &registry["vars"] else { panic!("registry vars") };
        for entry in vars.iter() {
            let mix::Value::Map(entry) = entry else { panic!("registry entry") };
            if let (Some(mix::Value::String(name)), Some(mix::Value::String(value))) = (entry.get("name"), entry.get("default")) {
                defaults.insert(name.clone(), value.clone().into());
            }
        }
        std::fs::write(prefix.path().join("env.mx"), serde_json::to_vec(&defaults).unwrap()).unwrap();
        prefix
    }).path()
}

pub fn command() -> Command {
    let mut command = Command::new(prefix().join("bin/mix"));
    command.env(paths::env::PREFIX, prefix());
    command
}

pub fn exe() -> &'static str {
    static EXE: OnceLock<String> = OnceLock::new();
    EXE.get_or_init(|| prefix().join("bin/mix").to_str().unwrap().to_owned())
}
