// SPDX-License-Identifier: MIT OR Apache-2.0
use std::process::Command;

// Use child processes so parallel host tests never observe a mutated environment.
#[test]
#[ignore = "child process probe for host_instance_uses_hostname_and_explicit_overrides"]
fn instance_probe() {
    println!("INSTANCE={}", settings::session::app_instance().unwrap());
}

#[test]
fn host_instance_uses_hostname_and_explicit_overrides() {
    let hostname = std::fs::read_to_string("/proc/sys/kernel/hostname").unwrap();
    for (app, settings, expected) in [
        (None, None, hostname.trim()),
        (None, Some("settings-host"), "settings-host"),
        (Some("app-host"), Some("settings-host"), "app-host"),
        (Some("app-host"), None, "app-host"),
    ] {
        let mut child = Command::new(std::env::current_exe().unwrap());
        child
            .args(["--ignored", "--exact", "instance_probe", "--nocapture"])
            .env_remove("MIX_INSTANCE")
            .env_remove("MIX_SETTINGS_INSTANCE");
        if let Some(value) = app {
            child.env("MIX_INSTANCE", value);
        }
        if let Some(value) = settings {
            child.env("MIX_SETTINGS_INSTANCE", value);
        }
        let output = child.output().unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .lines()
                .any(|line| line == format!("INSTANCE={expected}")),
            "app={app:?}, settings={settings:?}, expected={expected}"
        );
    }
}
