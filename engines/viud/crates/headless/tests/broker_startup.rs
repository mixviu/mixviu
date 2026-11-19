// SPDX-License-Identifier: MIT OR Apache-2.0
#[path = "support/bus.rs"]
#[allow(dead_code)]
mod bus;

#[test]
fn private_broker_diagnoses_exit_and_retries_only_bind_conflicts() {
    let root = std::env::temp_dir().join(format!("broker-startup-{}", std::process::id()));
    let output = std::process::Command::new(
        std::env::var_os("MIX_TEST_MIX_EXE").unwrap_or_else(|| "/opt/mix/bin/mix".into()),
    )
    .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/broker-startup.mix"))
    .arg(&root)
    .arg(bus::binary("noded"))
    .output()
    .unwrap();
    assert!(output.status.success(), "{output:?}");
    std::fs::remove_dir_all(root).unwrap();
}
