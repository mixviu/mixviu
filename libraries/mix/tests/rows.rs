// SPDX-License-Identifier: MIT OR Apache-2.0
use mix::{evaluator::{Evaluator,SharedBuf},lexer::Lexer,parser::Parser};
async fn run(s:&str)->Result<String,String> {
 let mut l=Lexer::new(s);let tokens=l.tokenize().map_err(|e|e.to_string())?;
 let mut p=Parser::new(tokens,s);let ast=p.parse_program().map_err(|e|e.to_string())?;
 let out=SharedBuf::new();let mut e=Evaluator::with_output(Box::new(out.clone()),Box::new(SharedBuf::new()));
 e.execute(&ast).await.map_err(|e|e.to_string())?;Ok(out.to_string_lossy())
}
#[tokio::test]
async fn projection_matches_map_for_captures_nested_cells_and_short_circuits() {
 let s=r#"
fn project($rows)
  $suffix="!"
  $f=fn($r)={key:$r.id,cells:[$r.text ?? "empty",fmt("%.1f",$r.n/2)..$suffix],chosen:$r.n>0 ? $r.n : 0,missing:$r.nope,flag:false and (1/0),fallback:3 ?? (1/0),yes:true or (1/0)}
  return deep_eq(rows_project($rows,$f),map($rows,$f))
end
print(project([{id:"a",text:"λ",n:2},{id:"b",text:nil,n:0}]))
print(project([]))
"#;
 assert_eq!(run(s).await.unwrap(),"true\ntrue\n");
}
#[tokio::test]
async fn unsupported_effects_and_mid_batch_failures_never_return_partial_rows() {
 for src in ["rows_project([],fn($r)=read_file($r))","rows_project([1],fn($r)=fn($x)=$x)","rows_project([1],fn($r)=$missing)"] {
  assert!(run(src).await.unwrap_err().contains("ROW_EXPRESSION"));
 }
 assert_eq!(run(r#"
$out=["previous"]
try $out=rows_project([2,0,4],fn($r)=10/$r) catch $m print($m) end
print(json_encode($out))
"#).await.unwrap(),"division by zero\n[\"previous\"]\n");
}
#[tokio::test]
async fn stat_many_matches_single_stat_missing_and_lstat() {
 let d=std::env::temp_dir().join(format!("mix-stat-many-{}",std::process::id()));std::fs::create_dir(&d).unwrap();
 let f=d.join("file");std::fs::write(&f,"hello").unwrap();let link=d.join("link");std::os::unix::fs::symlink(&f,&link).unwrap();
 let src=format!(r#"
$paths=[{}, {}, {}]
$rows=stat_many($paths,{{follow_symlinks:false}})
print(deep_eq($rows[0],stat($paths[0],{{follow_symlinks:false}})))
print($rows[1].is_symlink)
print($rows[2]==nil)
"#,serde_json::to_string(&f).unwrap(),serde_json::to_string(&link).unwrap(),serde_json::to_string(&d.join("absent")).unwrap());
 assert_eq!(run(&src).await.unwrap(),"true\ntrue\ntrue\n");
 for s in ["stat_many([1])","stat_many([],{follow_symlinks:1})","stat_many([],{unknown:true})"] {assert!(run(s).await.is_err());}
 std::fs::remove_dir_all(d).unwrap();
}
