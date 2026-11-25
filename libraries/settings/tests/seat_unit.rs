// SPDX-License-Identifier: MIT OR Apache-2.0
#[test]
fn plasma_seat_unit_is_packaged_and_requires_an_explicit_profile() {
    let unit = include_str!("../../../services/settingsd/units/settingsd-seat.service");
    for line in [
        "After=graphical-session.target",
        "PartOf=graphical-session.target",
        "ConditionEnvironment=XDG_SESSION_DESKTOP=KDE",
        "StartLimitIntervalSec=60",
        "StartLimitBurst=5",
        "UMask=0077",
        "Restart=on-failure",
        "RestartSec=2",
        "ExecStartPre=@{prefix}/bin/settingsd seed --instance %H --root %h/.local/state/mix/settingsd/%H/default",
        "ExecStart=@{prefix}/bin/settingsd serve --instance %H --root %h/.local/state/mix/settingsd/%H/default",
        "WantedBy=graphical-session.target",
    ] {
        assert!(unit.lines().any(|actual| actual == line), "missing {line}");
    }
    assert!(!unit.contains("--allow-create"));
    assert!(
        !unit
            .lines()
            .any(|line| line.starts_with("User=") || line.starts_with("EnvironmentFile="))
    );

    let templates = include_str!("../../../share/layout/templates.mx");
    let templates: serde_json::Value = serde_json::from_str(
        &templates
            .lines()
            .filter(|line| !line.starts_with("#"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
    assert!(templates["files"].as_array().unwrap().iter().any(|file| {
        file["kind"] == "unit"
            && file["source"] == "../../services/settingsd/units/settingsd-seat.service"
            && file["target"] == "@{prefix}/share/settingsd/units/settingsd-seat.service"
    }));
}
