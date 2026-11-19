use super::*;
#[test]
fn motion_ports_validate_without_changing_defaults() {
    let base = doc("root: {widget: \"text\", text: \"still\"}");
    let resolved = resolve(&base).unwrap();
    for port in ["enter", "exit", "motion", "transition", "opacity", "offset"] {
        assert!(!resolved.nodes["root"].ports.contains_key(port));
        assert!(describe("text").unwrap().iter().any(|p| p.path == port));
    }
    for ports in [
        "enter: \"up\", exit: \"fade\"",
        "enter: {preset: \"pop\", delay_ms: 30, stagger_ms: 20}",
        "transition: {prop: \"opacity\", ms: 120, ease: {kind: \"ease_out\"}}, opacity: 0.5",
        "transition: [{prop: \"offset\", ms: 0}, {prop: \"background\", ms: 60}], offset: [-4, 2]",
        "motion: false",
    ] {
        assert!(
            resolve(&doc(&format!(
                "root: {{widget: \"text\", text: \"motion\", {ports}}}"
            )))
            .is_ok(),
            "{ports}"
        );
    }
    for ports in [
        "enter: \"unknown\"",
        "enter: \"float\"",
        "enter: {preset: \"fade\", delay_ms: -1}",
        "enter: {preset: \"fade\", typo: 1}",
        "exit: true",
        "motion: \"off\"",
        "transition: {prop: \"width\", ms: 100}",
        "transition: {prop: \"opacity\", ms: -1}",
        "transition: {prop: \"opacity\", ms: 100, ease: {kind: \"bad\"}}",
        "transition: [{prop: \"opacity\", ms: 1}, {prop: \"opacity\", ms: 2}]",
        "offset: [1]",
        "offset: [1, \"x\"]",
        "opacity: 2",
    ] {
        assert!(
            resolve(&doc(&format!(
                "root: {{widget: \"text\", text: \"motion\", {ports}}}"
            )))
            .is_err(),
            "{ports}"
        );
    }
}
