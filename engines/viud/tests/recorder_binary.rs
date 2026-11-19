// SPDX-License-Identifier: MIT OR Apache-2.0
//! Ensure recorder integration fixtures use the compositor built by this job.
#[test]
fn recorder_compositor_binary_is_available() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_viud"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
}
