// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
fn main() {
    buildinfo::emit();
    println!("cargo:rerun-if-env-changed=MIX_TEST_INDEXD_TOKENIZER");
    println!("cargo:rustc-check-cfg=cfg(indexd_test_tokenizer)");
    if std::env::var_os("MIX_TEST_INDEXD_TOKENIZER").is_some() {
        println!("cargo:rustc-cfg=indexd_test_tokenizer");
    }
}
