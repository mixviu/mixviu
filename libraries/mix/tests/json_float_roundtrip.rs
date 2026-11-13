// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exercise the interpreter paths without linking settings or another engine.
use mix::evaluator::Evaluator;
use mix::lexer::Lexer;
use mix::parser::Parser;
use mix::value::Value;

#[tokio::test]
async fn json_float_roundtrip() {
    let path = std::env::temp_dir().join(format!("mix-floats-{}.json", std::process::id()));
    let source = format!(
        r#"
$path = {path:?}
$values = [1794606090.8718243, 0.1 + 0.2, 1e300, 5e-324,
  1794606090.0000002, 2147483647.9999998, -1794606090.8718243]
for $x in $values
  $wire = json_encode($x)
  assert(json_parse($wire) == $x)
  write_file($path, $wire)
  assert(read_json($path) == $x)
  write_file($path, $wire .. "\n" .. $wire .. "\n")
  $rows = read_jsonl($path)
  assert($rows[0] == $x and $rows[1] == $x)
end
true
"#,
        path = path.to_str().unwrap()
    );
    let tokens = Lexer::new(&source).tokenize().unwrap();
    let stmts = Parser::new(tokens, &source).parse_program().unwrap();
    let result = Evaluator::new().execute(&stmts).await;
    std::fs::remove_file(path).unwrap();
    assert_eq!(result.unwrap(), Value::Bool(true));
}
