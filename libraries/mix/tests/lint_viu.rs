// SPDX-License-Identifier: MIT OR Apache-2.0
use mix::analyzer::{AnalyzerConfig, VIU_EXTENSIONS, analyze};

fn diagnostics(source: &str) -> Vec<String> {
    let tokens = mix::lexer::Lexer::new(source).tokenize().unwrap();
    let stmts = mix::parser::Parser::new(tokens, source)
        .parse_program()
        .unwrap();
    analyze(
        &stmts,
        None,
        &AnalyzerConfig {
            source: Some(source.to_owned()),
            ..Default::default()
        },
    )
    .diagnostics
    .into_iter()
    .map(|d| d.code.to_owned())
    .collect()
}

#[test]
fn host_vocabulary_is_opt_in_and_keeps_unknown_names_visible() {
    for host in ["", "-- host: mix\n", "-- host: mixd\n", "-- host: typo\n"] {
        assert!(diagnostics(&format!("{host}viu_context()\n")).contains(&"MIX-E1102".to_owned()));
    }
    let codes =
        diagnostics("-- host: viu\nviu_context()\nviu_typo()\nprint($app_args)\nprint($unknown)\n");
    assert_eq!(codes.iter().filter(|c| *c == "MIX-E1102").count(), 1);
    assert_eq!(codes.iter().filter(|c| *c == "MIX-E1101").count(), 2);
}

#[test]
fn every_registered_host_name_resolves_and_cannot_be_shadowed() {
    assert!(VIU_EXTENSIONS.contains(&"viu_context"));
    assert!(VIU_EXTENSIONS.contains(&"viu_exit"));
    assert!(VIU_EXTENSIONS.contains(&"viu_document"));
    for name in VIU_EXTENSIONS {
        assert!(
            !diagnostics(&format!("-- host: viu\n{name}()\n")).contains(&"MIX-E1102".to_owned()),
            "{name}"
        );
        assert!(
            diagnostics(&format!("-- host: viu\nfn {name}()\nend\n"))
                .contains(&"MIX-W2313".to_owned()),
            "{name}"
        );
    }
}

#[test]
fn local_host_names_do_not_leak_into_ssh_mix() {
    let codes = diagnostics("-- host: viu\nssh_mix(\"example.test\", \"viu_context()\")\n");
    assert!(codes.contains(&"MIX-E1102".to_owned()));
}
