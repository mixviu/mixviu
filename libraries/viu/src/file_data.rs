// SPDX-License-Identifier: MIT OR Apache-2.0
//! Byte-preserving filesystem data for Mix-authored file controls. No navigation,
//! selection, presentation or operation policy is implemented here.
use files::chooser::{Filter, file_uri, location, name};
use serde_json::{Value, json};
use std::{
    ffi::OsStr,
    io::{Read,Write},
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, PermissionsExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    sync::OnceLock,
};
fn path(value: &Value) -> Result<PathBuf, String> {
    Ok(Path::new(OsStr::from_bytes(&location(value).map_err(|e| e.message)?)).to_owned())
}
fn uri(path: &Path) -> Result<String, String> {
    file_uri(path.as_os_str().as_bytes()).map_err(|e| e.message)
}
fn token(path: &Path) -> Result<Value, String> {
    Ok(json!(files::local_path::encode(path)?))
}
pub fn encode_path(path: &Path) -> Result<Value, String> {
    token(path)
}
pub fn decode_path(text: &str) -> Result<PathBuf, String> {
    files::local_path::decode(text)
}
fn raw_link(value: &Value) -> Result<std::ffi::OsString, String> {
    use std::os::unix::ffi::OsStringExt;
    let bytes = if let Some(text) = value.as_str() {
        text.as_bytes().to_vec()
    } else {
        files::chooser::percent_decode(
            value["bytes"]
                .as_str()
                .ok_or("Link target requires text or encoded bytes")?,
        )
        .map_err(|e| e.message)?
    };
    if bytes.contains(&0) {
        return Err("Link target contains NUL".into());
    }
    Ok(std::ffi::OsString::from_vec(bytes))
}
fn link_value(path: &Path) -> Value {
    match path.to_str() {
        Some(path) => json!(path),
        None => {
            json!({"bytes":path.as_os_str().as_bytes().iter().map(|b|format!("%{b:02X}")).collect::<String>()})
        }
    }
}
fn metadata(path: &Path, follow: bool) -> Result<Value, String> {
    let link = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    let m = if follow {
        std::fs::metadata(path).map_err(|e| e.to_string())?
    } else {
        link.clone()
    };
    Ok(
        json!({"is_file":m.is_file(),"is_dir":m.is_dir(),"is_symlink":link.file_type().is_symlink(),"size":m.len(),"perm":m.mode()&0o7777,"uid":m.uid(),"gid":m.gid(),"dev":m.dev().to_string(),"ino":m.ino().to_string(),"mtime":m.mtime(),"atime":m.atime(),"btime":m.created().ok().and_then(|t|t.duration_since(std::time::UNIX_EPOCH).ok()).map(|t|t.as_secs())}),
    )
}
fn regular_file(path:&Path)->Result<std::fs::File,String>{
    let file=std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK|libc::O_NOCTTY|libc::O_CLOEXEC).open(path).map_err(|e|e.to_string())?;
    if !file.metadata().map_err(|e|e.to_string())?.is_file(){return Err("Expected a regular file".into());}
    Ok(file)
}
fn write_segments(args:&Value)->Result<Value,String>{
    use base64::Engine;
    let destination=path(&args["path"])?;
    let limit=crate::widgets::whole(&args["limit"]).filter(|n|*n>0 && *n<=64*1024*1024).ok_or("Segment byte limit must be 1..64 MiB")?;
    let segments=args["segments"].as_array().filter(|a|a.len()<=1024).ok_or("At most 1024 segments required")?;
    let mut output=std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&destination).map_err(|e|e.to_string())?;
    let result=(||{
        let mut bytes=0;
        let mut append=|chunk:&[u8]|->Result<(),String>{
            bytes+=chunk.len() as u64;
            if bytes>limit {return Err("Segment byte limit exceeded".into());}
            output.write_all(chunk).map_err(|e|e.to_string())
        };
        for segment in segments {
            if let Some(text)=segment["text"].as_str(){
                if !segment["file"].is_null(){return Err("Text and file segments exclude one another".into());}
                append(text.as_bytes())?;
            }else{
                let source=path(&segment["file"])?;
                let mut input=regular_file(&source)?;
                if input.metadata().map_err(|e|e.to_string())?.len()>limit {return Err("Source exceeds segment byte limit".into());}
                match segment["encoding"].as_str().unwrap_or("raw") {
                    "base64"=>{
                        let mut chunk=[0;57];
                        loop {
                            let mut len=0;
                            while len<chunk.len(){let n=input.read(&mut chunk[len..]).map_err(|e|e.to_string())?;if n==0{break;}len+=n;}
                            if len==0{break;}
                            append(base64::engine::general_purpose::STANDARD.encode(&chunk[..len]).as_bytes())?;
                            append(b"\r\n")?;
                        }
                    }
                    "raw"=>{
                        let mut chunk=[0;8192];
                        loop {let len=input.read(&mut chunk).map_err(|e|e.to_string())?;if len==0{break;}append(&chunk[..len])?;}
                    }
                    _=>return Err("Unknown segment encoding".into()),
                }
            }
        }
        output.sync_all().map_err(|e|e.to_string())?;
        Ok(json!({"bytes":bytes,"path":token(&destination)?}))
    })();
    if result.is_err(){let _=std::fs::remove_file(&destination);}
    result
}
pub fn command(command: &str, args: &Value) -> Result<Value, String> {
    match command {
        "read-text"=>{
            let file=regular_file(&path(&args["path"])?)?;
            let limit=crate::widgets::whole(&args["limit"]).filter(|n|*n>0 && *n<=32*1024*1024).ok_or("Text byte limit must be 1..32 MiB")?;
            if file.metadata().map_err(|e|e.to_string())?.len()>limit{return Err("Text file exceeds its byte limit".into());}
            let mut bytes=Vec::new();file.take(limit+1).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
            if bytes.len() as u64>limit{return Err("Text file exceeds its byte limit".into());}
            Ok(json!(String::from_utf8(bytes).map_err(|e|e.to_string())?))
        }
        "digest"=>{
            use sha2::Digest;
            let mut file=regular_file(&path(&args["path"])?)?;
            let limit=crate::widgets::whole(&args["limit"]).filter(|n|*n>0 && *n<=64*1024*1024).ok_or("Digest byte limit must be 1..64 MiB")?;
            let mut hasher=sha2::Sha256::new();let mut bytes=0;let mut chunk=[0;8192];
            loop {let len=file.read(&mut chunk).map_err(|e|e.to_string())?;if len==0{break;}bytes+=len as u64;if bytes>limit{return Err("Digest byte limit exceeded".into());}hasher.update(&chunk[..len]);}
            Ok(json!(format!("{:x}",hasher.finalize())))
        }
        "write-segments"=>write_segments(args),
        "inspect"=>{
            let source=path(&args["path"])?;
            let mut file=regular_file(&source)?;
            static MIME:OnceLock<mimedb::Database>=OnceLock::new();
            let db=MIME.get_or_init(mimedb::Database::load);
            let mut bytes=vec![0;db.sniff_len()];
            let len=file.read(&mut bytes).map_err(|e|e.to_string())?;
            let name=source.file_name().unwrap_or_default().to_string_lossy();
            Ok(json!({"name":name,"size":file.metadata().map_err(|e|e.to_string())?.len(),"type":db.guess(Some(&name),Some(&bytes[..len])).mime}))
        }
        "token" => token(&path(&args["path"])?),
        "stat" => metadata(&path(&args["path"])?, args["follow"] != false),
        "canonical" => {
            token(&std::fs::canonicalize(path(&args["path"])?).map_err(|e| e.to_string())?)
        }
        "parent-token" => {
            let p = path(&args["path"])?;
            token(p.parent().unwrap_or(Path::new("/")))
        }
        "join-token" => {
            let p = path(&args["folder"])?;
            token(&p.join(OsStr::from_bytes(
                &name(&args["name"]).map_err(|e| e.message)?,
            )))
        }
        "display-leaf" => {
            let p = path(&args["path"])?;
            Ok(json!(
                p.file_name()
                    .unwrap_or(p.as_os_str())
                    .to_string_lossy()
                    .chars()
                    .map(|c| if c.is_control() { '�' } else { c })
                    .collect::<String>()
            ))
        }
        "names" => {
            let p = path(&args["path"])?;
            let limit = match args.get("limit") {
                None => 100_000,
                Some(v) => v
                    .as_f64()
                    .filter(|n| n.is_finite() && n.fract() == 0.0 && (0.0..=100_000.0).contains(n))
                    .ok_or("Directory limit must be an integer in 0..100000")?
                    as usize,
            };
            let mut entries = Vec::new();
            for entry in std::fs::read_dir(p).map_err(|e| e.to_string())? {
                if entries.len() == limit {
                    return Err("ENUMERATION_BUDGET: directory admission bound exceeded".into());
                }
                entries.push(entry.map_err(|e| e.to_string())?);
            }
            entries.sort_by_key(|e| e.file_name());
            Ok(json!(entries.into_iter().map(|e|Ok(json!({"path":token(&e.path())?,"name":e.file_name().to_string_lossy(),"leaf":link_value(Path::new(&e.file_name()))}))).collect::<Result<Vec<Value>,String>>()?))
        }
        "read-link" => Ok(link_value(
            &std::fs::read_link(path(&args["path"])?).map_err(|e| e.to_string())?,
        )),
        "display-link" => Ok(json!(
            std::fs::read_link(path(&args["path"])?)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
        )),
        "copy" => {
            std::fs::copy(path(&args["source"])?, path(&args["target"])?)
                .map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "rename" => {
            std::fs::rename(path(&args["source"])?, path(&args["target"])?)
                .map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "link-new" => match std::fs::hard_link(path(&args["source"])?, path(&args["target"])?) {
            Ok(()) => Ok(json!(true)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(json!(false)),
            Err(error) => Err(error.to_string()),
        },
        "user-directory" => {
            let key = args["key"].as_str().ok_or("User directory key required")?;
            if !matches!(
                key,
                "XDG_DOWNLOAD_DIR"
                    | "XDG_DOCUMENTS_DIR"
                    | "XDG_PICTURES_DIR"
                    | "XDG_DESKTOP_DIR"
                    | "XDG_MUSIC_DIR"
                    | "XDG_VIDEOS_DIR"
            ) {
                return Err("Unknown user directory key".into());
            }
            let home = std::env::var_os("HOME")
                .filter(|v| !v.is_empty())
                .ok_or("No home directory")?;
            let home = PathBuf::from(home);
            let config = std::env::var_os("XDG_CONFIG_HOME")
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config"));
            let source = std::fs::read_to_string(config.join("user-dirs.dirs")).ok();
            let value = std::env::var(key).ok();
            match toolkit::file_chooser::user_directory(
                value.as_deref(),
                source.as_deref(),
                key,
                &home,
            ) {
                Some(path) => token(&path),
                None => Ok(Value::Null),
            }
        }
        "mkdir" => {
            std::fs::create_dir(path(&args["path"])?).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "remove" => {
            std::fs::remove_file(path(&args["path"])?).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "remove-dir" => {
            std::fs::remove_dir_all(path(&args["path"])?).map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "chmod" => {
            let mode = args["mode"]
                .as_u64()
                .filter(|mode| *mode <= 0o7777)
                .ok_or("Invalid permissions")?;
            std::fs::set_permissions(
                path(&args["path"])?,
                std::fs::Permissions::from_mode(mode as u32),
            )
            .map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "symlink" => {
            std::os::unix::fs::symlink(raw_link(&args["link"])?, path(&args["path"])?)
                .map_err(|e| e.to_string())?;
            Ok(Value::Null)
        }
        "uri" => Ok(json!(uri(&path(&args["path"])?)?)),
        "display-path" => Ok(json!(
            path(&args["path"])?
                .to_string_lossy()
                .chars()
                .map(|c| if c.is_control() { '�' } else { c })
                .collect::<String>()
        )),
        "count" => Ok(json!(
            command_fn("scan", args)?
                .as_array()
                .ok_or("Expected directory rows")?
                .len()
        )),
        "parent" => {
            let p = path(&args["path"])?;
            Ok(json!(uri(p.parent().unwrap_or(Path::new("/")))?))
        }
        "join" => {
            let p = path(&args["folder"])?;
            let n = name(&args["name"]).map_err(|e| e.message)?;
            Ok(json!(uri(&p.join(OsStr::from_bytes(&n)))?))
        }
        "name" => Ok(json!(String::from_utf8_lossy(
            &name(&args["name"]).map_err(|e| e.message)?
        ))),
        "leaf" => {
            let p = path(&args["path"])?;
            let raw = p.file_name().ok_or("Path has no file name")?.as_bytes();
            Ok(match std::str::from_utf8(raw) {
                Ok(s) => json!(s),
                Err(_) => {
                    json!({"bytes":raw.iter().map(|b|format!("%{b:02X}")).collect::<String>()})
                }
            })
        }
        "scan" => {
            let folder = path(&args["folder"])?;
            let filter = if args["filter"].is_null() {
                None
            } else {
                Some(Filter::parse(&args["filter"]).map_err(|e| e.message)?)
            };
            static MIME: OnceLock<mimedb::Database> = OnceLock::new();
            let mut rows = Vec::new();
            for entry in std::fs::read_dir(folder).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let raw = entry.file_name();
                if args["hidden"] != true && raw.as_bytes().first() == Some(&b'.') {
                    continue;
                }
                let name = raw.to_string_lossy();
                let meta = entry.metadata().map_err(|e| e.to_string())?;
                if let Some(filter) = &filter {
                    let mime = MIME
                        .get_or_init(mimedb::Database::load)
                        .guess(Some(&name), None)
                        .mime;
                    if !filter.matches(&name, meta.is_dir(), &mime) {
                        continue;
                    }
                }
                if rows.len() == 100_000 {
                    return Err("Directory exceeds the 100000-row admission bound".into());
                }
                let display: String = name
                    .chars()
                    .map(|c| if c.is_control() { '�' } else { c })
                    .collect();
                rows.push(json!({"key":uri(&entry.path())?,"name":display,"dir":meta.is_dir(),"file":meta.is_file(),"size":meta.len(),"modified":meta.modified().ok().and_then(|t|t.duration_since(std::time::UNIX_EPOCH).ok()).map(|t|t.as_secs())}));
            }
            Ok(json!(rows))
        }
        _ => Err("Unknown file data primitive".into()),
    }
}
pub(crate) fn binding() -> mix::evaluator::ExtFn {
    mix::sync_ext(|args| {
        let fail = |msg: String| {
            if msg.starts_with("ENUMERATION_BUDGET:") {
                mix::MixError::structured("ENUMERATION_BUDGET", msg)
            } else {
                mix::MixError::RuntimeError { msg, span: None }
            }
        };
        let [mix::value::Value::String(command), args] = args.as_slice() else {
            return Err(fail("viu_files expects command and object".into()));
        };
        let args = crate::runtime::to_json(args)?;
        crate::runtime::from_json(command_fn(command, &args).map_err(fail)?)
    })
}
fn command_fn(name: &str, args: &Value) -> Result<Value, String> {
    command(name, args)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn segment_writer_wraps_binary_without_replacing_or_leaving_failed_outputs() {
        use base64::Engine;
        let dir=tempfile::tempdir().unwrap();let source=dir.path().join("source");let target=dir.path().join("target");
        let bytes=(0..255).collect::<Vec<u8>>();std::fs::write(&source,&bytes).unwrap();
        let segments=json!([{"text":"before\r\n"},{"file":source,"encoding":"base64"},{"text":"after"}]);
        let args=json!({"path":target,"limit":1000,"segments":segments});
        command("write-segments",&args).unwrap();
        let output=std::fs::read_to_string(&target).unwrap();
        let body=output.strip_prefix("before\r\n").unwrap().strip_suffix("after").unwrap();
        assert!(body.split("\r\n").all(|line|line.len()<=76));
        assert_eq!(base64::engine::general_purpose::STANDARD.decode(body.replace("\r\n","")).unwrap(),bytes);
        assert!(command("write-segments",&args).is_err());
        let limited=dir.path().join("limited");
        assert!(command("write-segments",&json!({"path":limited,"limit":100,"segments":segments})).is_err());
        assert!(!limited.exists());assert_eq!(std::fs::read_to_string(target).unwrap(),output);
    }
    #[test]
    fn link_new_never_replaces_a_file_and_keeps_private_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("partial");
        let target = dir.path().join("saved");
        std::fs::write(&source, [0, 255, 42]).unwrap();
        std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o600)).unwrap();
        let args = json!({"source":source,"target":target});
        assert_eq!(command("link-new", &args).unwrap(), true);
        assert_eq!(command("link-new", &args).unwrap(), false);
        std::fs::remove_file(source).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), [0, 255, 42]);
        assert_eq!(std::fs::metadata(target).unwrap().mode() & 0o777, 0o600);
    }
    #[test]
    fn directory_admission_stops_at_the_callers_remaining_work_budget() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(
            command("names", &json!({"path":d.path(),"limit":0})).unwrap(),
            json!([])
        );
        std::fs::write(d.path().join("one"), "x").unwrap();
        assert!(
            command("names", &json!({"path":d.path(),"limit":0}))
                .unwrap_err()
                .starts_with("ENUMERATION_BUDGET:")
        );
        assert_eq!(
            command("names", &json!({"path":d.path(),"limit":1.0}))
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
        for limit in [json!(-1), json!(1.5), json!(100001), json!("1")] {
            assert!(command("names", &json!({"path":d.path(),"limit":limit})).is_err());
        }
    }
    #[test]
    fn uri_rows_keep_non_utf8_names_and_filters_keep_directories() {
        let dir = tempfile::tempdir().unwrap();
        let raw = OsStr::from_bytes(b"a\xff.txt");
        std::fs::write(dir.path().join(raw), "x").unwrap();
        std::fs::create_dir(dir.path().join("folder")).unwrap();
        std::fs::write(dir.path().join("skip.png"), "x").unwrap();
        let rows=command("scan",&json!({"folder":dir.path(),"filter":{"name":"Text","patterns":["*.txt"],"mime_types":[]}})).unwrap();
        assert_eq!(rows.as_array().unwrap().len(), 2);
        let row = rows
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["file"] == true)
            .unwrap();
        assert_eq!(path(&row["key"]).unwrap(), dir.path().join(raw));
        assert!(row["key"].as_str().unwrap().contains("%FF"));
        assert!(command("join", &json!({"folder":dir.path(),"name":"../escape"})).is_err());
    }
    #[test]
    fn file_primitives_preserve_raw_names_links_and_recursive_policy_stays_with_caller() {
        let dir = tempfile::tempdir().unwrap();
        let raw = dir.path().join(OsStr::from_bytes(b"raw\xff.txt"));
        std::fs::write(&raw, b"unchanged bytes").unwrap();
        let names = command("names", &json!({"path":dir.path()})).unwrap();
        let source = &names[0]["path"];
        assert_eq!(path(source).unwrap(), raw);
        assert_eq!(
            command("stat", &json!({"path":source,"follow":false})).unwrap()["size"],
            15
        );
        let target = command(
            "join-token",
            &json!({"folder":dir.path(),"name":{"bytes":"%63%6F%70%79%FF%2E%74%78%74"}}),
        )
        .unwrap();
        command("copy", &json!({"source":source,"target":target})).unwrap();
        assert_eq!(
            std::fs::read(path(&target).unwrap()).unwrap(),
            b"unchanged bytes"
        );
        let link = dir.path().join("raw-link");
        std::os::unix::fs::symlink(raw.file_name().unwrap(), &link).unwrap();
        let bytes = command("read-link", &json!({"path":link})).unwrap();
        let copy_link = dir.path().join("copied-link");
        command("symlink", &json!({"link":bytes,"path":copy_link})).unwrap();
        assert_eq!(
            std::fs::read_link(&copy_link).unwrap(),
            raw.file_name().unwrap()
        );
        let moved = dir.path().join("renamed.txt");
        command("rename", &json!({"source":target,"target":moved})).unwrap();
        assert!(!path(&target).unwrap().exists());
        assert_eq!(std::fs::read(&moved).unwrap(), b"unchanged bytes");
        assert!(
            command(
                "join-token",
                &json!({"folder":dir.path(),"name":{"bytes":"%2E%2E%2F%65%73%63%61%70%65"}})
            )
            .is_err()
        );
        assert!(command("token", &json!({"path":"file://remote.example/path"})).is_err());
    }
}
