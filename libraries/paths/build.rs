// SPDX-License-Identifier: MIT OR Apache-2.0
// Dependency-free snapshot stamp for the lazy env.mx reader. Consumers still
// stamp themselves through buildinfo; both are built from the same checkout.
#[allow(dead_code)]
mod names {
    include!("src/env_names.rs");
}
fn main() {
    // Cargo's default package scan must remain active. Git refs are also watched
    // so a clean same-branch commit updates the snapshot identity.
    for args in [
        vec!["rev-parse", "--git-path", "HEAD"],
        vec!["rev-parse", "--git-path", "refs"],
        vec!["rev-parse", "--git-path", "packed-refs"],
    ] {
        if let Ok(out) = std::process::Command::new("git").args(args).output() {
            if out.status.success() {
                println!(
                    "cargo:rerun-if-changed={}",
                    String::from_utf8_lossy(&out.stdout).trim()
                );
            }
        }
    }
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=build.rs");
    if let Ok(out) = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
    {
        if out.status.success() {
            println!(
                "cargo:rustc-env={}={}",
                names::GIT_SHA_FULL,
                String::from_utf8_lossy(&out.stdout).trim()
            );
        }
    }
}
