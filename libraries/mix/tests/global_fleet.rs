// SPDX-License-Identifier: MIT OR Apache-2.0
//! Opt-in fleet sweep. The corpus stays outside the public repository.
use mix::{
    analyzer::{AnalyzerConfig, analyze},
    lexer::Lexer,
    parser::Parser,
};

#[test]
#[ignore = "requires an operator-supplied test global-fleet JSON corpus"]
fn global_bug_signature_fleet_sweep() {
    let path = paths::env::os(paths::env::test::TEST_GLOBAL_FLEET)
        .ok_or(std::env::VarError::NotPresent)
        .and_then(|v| v.into_string().map_err(std::env::VarError::NotUnicode))
        .unwrap();
    let corpus: Vec<(String, String)> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut parsed = 0;
    let mut hits = 0;
    let mut failed = 0;
    for (path, source) in &corpus {
        let Ok(tokens) = Lexer::new(source).tokenize() else {
            failed += 1;
            continue;
        };
        let Ok(stmts) = Parser::new(tokens, source).parse_program() else {
            failed += 1;
            continue;
        };
        parsed += 1;
        let cfg = AnalyzerConfig {
            agent: true,
            source: Some(source.clone()),
            ..Default::default()
        };
        for d in analyze(&stmts, Some(path), &cfg).diagnostics {
            if d.code == "MIX-W2316" {
                hits += 1;
                println!("{}:{:?}: {}", path, d.line, d.message);
            }
        }
    }
    println!(
        "GLOBAL_FLEET files={} parsed={parsed} parse_failures={failed} hits={hits}",
        corpus.len()
    );
}
