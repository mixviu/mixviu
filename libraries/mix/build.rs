// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
fn main() {
    // Read the host's registration sites without linking the widget toolkit
    // into the shell. Conditional services are part of the host vocabulary.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../viu/src/runtime.rs");
    println!("cargo:rerun-if-changed={}", path.display());
    let source = std::fs::read_to_string(path).expect("viu registration source");
    let mut names = std::collections::BTreeSet::new();
    for tail in source.split("eval.register(").skip(1) {
        let tail = tail.trim_start();
        let name = tail.strip_prefix('"').expect("literal host builtin name")
            .split('"').next().unwrap();
        assert!(name.starts_with("viu_"), "unexpected host builtin {name}");
        names.insert(name);
    }
    assert!(!names.is_empty(), "missing viu registrations");
    let output = format!("pub const VIU_EXTENSIONS: &[&str] = &{:?};", names.into_iter().collect::<Vec<_>>());
    std::fs::write(std::path::Path::new(&std::env::var("OUT_DIR").unwrap())
        .join("viu_extensions.rs"), output).unwrap();
}
