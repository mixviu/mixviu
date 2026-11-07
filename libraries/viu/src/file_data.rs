// SPDX-License-Identifier: MIT OR Apache-2.0
//! Byte-preserving filesystem data for Mix-authored file controls. No navigation,
//! selection, presentation or operation policy is implemented here.
use files::chooser::{Filter, file_uri, location, name};
use serde_json::{Value, json};
use std::{
    ffi::OsStr,
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, PermissionsExt},
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
pub fn command(command: &str, args: &Value) -> Result<Value, String> {
    match command {
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
        "count" => {
            let result = command_fn("scan", args)?;
            if args.get("work_budget").is_some() {
                Ok(
                    json!({"count":result["rows"].as_array().ok_or("Expected directory rows")?.len(),"work":result["work"]}),
                )
            } else {
                Ok(json!(
                    result.as_array().ok_or("Expected directory rows")?.len()
                ))
            }
        }
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
        "save-name" => {
            let raw = name(&args["name"]).map_err(|e| e.message)?;
            let raw = if args["filter"].is_null() {
                raw
            } else {
                Filter::parse(&args["filter"])
                    .map_err(|e| e.message)?
                    .save_name(&raw)
            };
            Ok(link_value(Path::new(OsStr::from_bytes(&raw))))
        }
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
            let limit = match args.get("work_budget") {
                None => 100_000,
                Some(value) => crate::widgets::whole(value)
                    .filter(|n| *n <= 65536)
                    .ok_or("Invalid directory work budget")?
                    as usize,
            };
            let mut work = 0;
            for entry in std::fs::read_dir(folder).map_err(|e| e.to_string())? {
                if work == limit {
                    return Err("SCAN_BUDGET: total directory enumeration budget exceeded".into());
                }
                work += 1;
                let entry = entry.map_err(|e| e.to_string())?;
                let raw = entry.file_name();
                if args["hidden"] != true && raw.as_bytes().first() == Some(&b'.') {
                    continue;
                }
                let name = raw.to_string_lossy();
                let meta = entry.metadata().map_err(|e| e.to_string())?;
                if args["directories_only"] == true && !meta.is_dir() {
                    continue;
                }
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
            if args.get("work_budget").is_some() {
                Ok(json!({"rows":rows,"work":work}))
            } else {
                Ok(json!(rows))
            }
        }
        _ => Err("Unknown file data primitive".into()),
    }
}
pub(crate) fn binding() -> mix::evaluator::ExtFn {
    mix::sync_ext(|args| {
        let fail = |msg: String| {
            if msg.starts_with("SCAN_BUDGET:") {
                mix::MixError::structured("SCAN_BUDGET", msg)
            } else if msg.starts_with("ENUMERATION_BUDGET:") {
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
    fn filtered_scans_charge_hidden_and_filtered_entries_before_returning_rows() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join(".hidden"), "x").unwrap();
        std::fs::write(d.path().join("skip.png"), "x").unwrap();
        std::fs::write(d.path().join("keep.txt"), "x").unwrap();
        let mut args = json!({"folder":d.path(),"hidden":false,"filter":{"name":"Text","patterns":["*.txt"],"mime_types":[]},"work_budget":2});
        assert!(
            command("scan", &args)
                .unwrap_err()
                .starts_with("SCAN_BUDGET:")
        );
        args["work_budget"] = json!(3);
        let scan = command("scan", &args).unwrap();
        assert_eq!(scan["work"], 3);
        assert_eq!(scan["rows"].as_array().unwrap().len(), 1);
        assert_eq!(
            command("count", &args).unwrap(),
            json!({"count":1,"work":3})
        );
        assert_eq!(
            command(
                "save-name",
                &json!({"name":{"bytes":"%6E%FF"},"filter":args["filter"]})
            )
            .unwrap(),
            json!({"bytes":"%6E%FF%2E%74%78%74"})
        );
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
