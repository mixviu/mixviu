// SPDX-License-Identifier: MIT OR Apache-2.0
//! Byte-preserving filesystem data for Mix-authored file controls. No navigation,
//! selection, presentation or operation policy is implemented here.
use std::{ffi::OsStr,os::unix::ffi::OsStrExt,path::{Path,PathBuf},sync::OnceLock};
use serde_json::{Value,json};
use files::chooser::{location,file_uri,name,Filter};
fn path(value:&Value)->Result<PathBuf,String>{Ok(Path::new(OsStr::from_bytes(&location(value).map_err(|e|e.message)?)).to_owned())}
fn uri(path:&Path)->Result<String,String>{file_uri(path.as_os_str().as_bytes()).map_err(|e|e.message)}
pub fn command(command:&str,args:&Value)->Result<Value,String>{
    match command{
        "uri"=>Ok(json!(uri(&path(&args["path"])?)?)),
        "parent"=>{let p=path(&args["path"])?;Ok(json!(uri(p.parent().unwrap_or(Path::new("/")))?))},
        "join"=>{let p=path(&args["folder"])?;let n=name(&args["name"]).map_err(|e|e.message)?;Ok(json!(uri(&p.join(OsStr::from_bytes(&n)))?))},
        "name"=>Ok(json!(String::from_utf8_lossy(&name(&args["name"]).map_err(|e|e.message)?))),
        "leaf"=>{
            let p=path(&args["path"])?;let raw=p.file_name().ok_or("Path has no file name")?.as_bytes();
            Ok(match std::str::from_utf8(raw){Ok(s)=>json!(s),Err(_)=>json!({"bytes":raw.iter().map(|b|format!("%{b:02X}")).collect::<String>()})})
        },
        "scan"=>{
            let folder=path(&args["folder"])?;
            let filter=if args["filter"].is_null(){None}else{Some(Filter::parse(&args["filter"]).map_err(|e|e.message)?)};
            static MIME:OnceLock<mimedb::Database>=OnceLock::new();
            let mut rows=Vec::new();
            for entry in std::fs::read_dir(folder).map_err(|e|e.to_string())?{
                let entry=entry.map_err(|e|e.to_string())?;let raw=entry.file_name();
                if args["hidden"]!=true&&raw.as_bytes().first()==Some(&b'.'){continue;}
                let name=raw.to_string_lossy();let meta=entry.metadata().map_err(|e|e.to_string())?;
                if let Some(filter)=&filter{let mime=MIME.get_or_init(mimedb::Database::load).guess(Some(&name),None).mime;if !filter.matches(&name,meta.is_dir(),&mime){continue;}}
                if rows.len()==100_000{return Err("Directory exceeds the 100000-row admission bound".into());}
                let display:String=name.chars().map(|c|if c.is_control(){'�'}else{c}).collect();
                rows.push(json!({"key":uri(&entry.path())?,"name":display,"dir":meta.is_dir(),"file":meta.is_file(),"size":meta.len(),"modified":meta.modified().ok().and_then(|t|t.duration_since(std::time::UNIX_EPOCH).ok()).map(|t|t.as_secs())}));
            }
            Ok(json!(rows))
        },
        _=>Err("Unknown file data primitive".into()),
    }
}
pub(crate) fn binding()->mix::evaluator::ExtFn{
    mix::sync_ext(|args|{
        let fail=|msg:String|mix::MixError::RuntimeError{msg,span:None};
        let [mix::value::Value::String(command),args]=args.as_slice()else{return Err(fail("viu_files expects command and object".into()));};
        let args=crate::runtime::to_json(args)?;
        crate::runtime::from_json(command_fn(command,&args).map_err(fail)?)
    })
}
fn command_fn(name:&str,args:&Value)->Result<Value,String>{command(name,args)}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn uri_rows_keep_non_utf8_names_and_filters_keep_directories(){
        let dir=tempfile::tempdir().unwrap();
        let raw=OsStr::from_bytes(b"a\xff.txt");std::fs::write(dir.path().join(raw),"x").unwrap();
        std::fs::create_dir(dir.path().join("folder")).unwrap();std::fs::write(dir.path().join("skip.png"),"x").unwrap();
        let rows=command("scan",&json!({"folder":dir.path(),"filter":{"name":"Text","patterns":["*.txt"],"mime_types":[]}})).unwrap();
        assert_eq!(rows.as_array().unwrap().len(),2);
        let row=rows.as_array().unwrap().iter().find(|r|r["file"]==true).unwrap();
        assert_eq!(path(&row["key"]).unwrap(),dir.path().join(raw));
        assert!(row["key"].as_str().unwrap().contains("%FF"));
        assert!(command("join",&json!({"folder":dir.path(),"name":"../escape"})).is_err());
    }
}
