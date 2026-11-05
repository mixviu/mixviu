// SPDX-License-Identifier: MIT OR Apache-2.0
//! The builtin reports the OS creation timestamp, including lstat semantics.
use mix::{IndexMap,builtins::call_builtin,value::Value};
use std::{path::Path,time::UNIX_EPOCH};

fn check(path:&Path,follow:bool){
    let meta=if follow{std::fs::metadata(path)}else{std::fs::symlink_metadata(path)}.unwrap();
    let value=call_builtin("stat",vec![Value::String(path.to_string_lossy().into()),Value::map(IndexMap::from([("follow_symlinks".into(),Value::Bool(follow))]))]).unwrap().unwrap();
    let Value::Map(map)=value else{panic!("stat map")};
    match meta.created().ok().and_then(|time|time.duration_since(UNIX_EPOCH).ok()){
        Some(time)=>{assert_eq!(map["btime"],Value::Number(time.as_secs() as f64));assert_eq!(map["btime_nsec"],Value::Number(time.subsec_nanos() as f64));},
        None=>{assert_eq!(map["btime"],Value::Nil);assert_eq!(map["btime_nsec"],Value::Nil);}
    }
}
#[test]
fn stat_birth_time_matches_the_filesystem_for_files_and_links(){
    let dir=std::env::temp_dir().join(format!("mix-birthtime-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir(&dir).unwrap();
    let path=dir.join("file");std::fs::write(&path,"data").unwrap();
    let link=dir.join("link");std::os::unix::fs::symlink(&path,&link).unwrap();
    check(&path,true);check(&link,true);check(&link,false);
    std::fs::remove_dir_all(dir).unwrap();
}
