// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
fn main() {
    // Read the host's registration sites without linking the widget toolkit
    // into the shell. Conditional services are part of the host vocabulary.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../viu/src/runtime.rs");
    println!("cargo:rerun-if-changed={}", path.display());
    let standalone = !path.is_file();
    let source = if !standalone {
        std::fs::read_to_string(path).expect("viu registration source")
    } else {
        // A standalone lib-mix build has no widget host. Keep the shell
        // buildable, but do not pretend to know that host's vocabulary.
        println!(
            "cargo:warning=viu sources absent; host: viu lint vocabulary unavailable in this standalone build"
        );
        String::new()
    };
    let mut names = std::collections::BTreeSet::new();
    for tail in source.split("eval.register(").skip(1) {
        let tail = tail.trim_start();
        let name = tail
            .strip_prefix('"')
            .expect("literal host builtin name")
            .split('"')
            .next()
            .unwrap();
        assert!(name.starts_with("viu_"), "unexpected host builtin {name}");
        names.insert(name);
    }
    assert!(standalone || !names.is_empty(), "missing viu registrations");
    let output = format!(
        "pub const VIU_EXTENSIONS: &[&str] = &{:?};",
        names.into_iter().collect::<Vec<_>>()
    );
    std::fs::write(
        std::path::Path::new(
            &paths::env::os("OUT_DIR")
                .ok_or(std::env::VarError::NotPresent)
                .and_then(|v| v.into_string().map_err(std::env::VarError::NotUnicode))
                .unwrap(),
        )
        .join("viu_extensions.rs"),
        output,
    )
    .unwrap();
}
