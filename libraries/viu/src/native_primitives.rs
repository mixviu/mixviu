// SPDX-License-Identifier: MIT OR Apache-2.0
//! Small evaluator bindings to existing native primitives. The Mix program
//! chooses the file, remote-read policy and lifetime, normally on a Jobs worker.
use mix::{MixError, value::Value};
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};

/// Pure resolution through the shared native application-root contract.
pub(crate) fn app_dirs() -> mix::evaluator::ExtFn {
    mix::sync_ext(|args| {
        let [Value::String(component)] = args.as_slice() else {
            return Err(MixError::RuntimeError {
                msg: "viu_app_dirs expects an application slug".into(),
                span: None,
            });
        };
        crate::runtime::from_json(match config::AppDirs::resolve(component) {
            Some(d) => {
                serde_json::json!({"root":d.root(),"config":d.config(),"state":d.state(),"cache":d.cache()})
            }
            None => serde_json::Value::Null,
        })
    })
}

/// Inert, owned ABP mail documents. No storage, application or send policy.
pub(crate) fn mail_document() -> mix::evaluator::ExtFn {
    mix::sync_ext(|args| {
        let fail = |msg: String| MixError::RuntimeError { msg, span: None };
        let [Value::String(op), value] = args.as_slice() else {
            return Err(fail("viu_maildoc expects parse/write and data".into()));
        };
        let value = crate::runtime::to_json(value)?;
        if op == "parse" {
            let d = maildoc::parse(
                value
                    .as_str()
                    .ok_or_else(|| fail("mail source requires text".into()))?,
            )
            .map_err(|e| fail(e.to_string()))?;
            return crate::runtime::from_json(
                serde_json::json!({"to":d.to,"cc":d.cc,"bcc":d.bcc,"from":d.from,"reply_to":d.reply_to,"subject":d.subject,"in_reply_to":d.in_reply_to,"references":d.references,"attachments":d.attachments,"body":d.body}),
            );
        }
        if op != "write" {
            return Err(fail("unknown mail document operation".into()));
        }
        let list = |key: &str| {
            serde_json::from_value::<Vec<String>>(value[key].clone())
                .map_err(|e| fail(e.to_string()))
        };
        let optional = |key: &str| {
            serde_json::from_value::<Option<String>>(value[key].clone())
                .map_err(|e| fail(e.to_string()))
        };
        let document = maildoc::MailDocument {
            to: list("to")?,
            cc: list("cc")?,
            bcc: list("bcc")?,
            from: optional("from")?,
            reply_to: optional("reply_to")?,
            subject: value["subject"]
                .as_str()
                .ok_or_else(|| fail("subject required".into()))?
                .into(),
            in_reply_to: optional("in_reply_to")?,
            references: list("references")?,
            attachments: list("attachments")?.into_iter().map(Into::into).collect(),
            body: value["body"]
                .as_str()
                .ok_or_else(|| fail("body required".into()))?
                .into(),
        };
        crate::runtime::from_json(serde_json::json!(
            maildoc::write(&document).map_err(|e| fail(e.to_string()))?
        ))
    })
}

pub(crate) fn digest() -> mix::evaluator::ExtFn {
    mix::sync_ext(|args| {
        let fail = || MixError::RuntimeError {
            msg: "viu_digest expects blake3/sha256 and text".into(),
            span: None,
        };
        let [Value::String(algorithm), Value::String(text)] = args.as_slice() else {
            return Err(fail());
        };
        let hash = match algorithm.as_str() {
            "blake3" => blake3::hash(text.as_bytes()).to_hex().to_string(),
            "sha256" => {
                use sha2::Digest;
                format!("{:x}", sha2::Sha256::digest(text.as_bytes()))
            }
            _ => return Err(fail()),
        };
        Ok(Value::String(hash))
    })
}

/// The retained gold event codec; the Mix owner chooses when to publish it.
pub(crate) fn action_event() -> mix::evaluator::ExtFn {
    let journal = std::sync::Mutex::new(toolkit::executed::Journal::new(1));
    mix::sync_ext(move |args| {
        let fail = || MixError::RuntimeError {
            msg: "viu_action_event expects id, origin, params, result".into(),
            span: None,
        };
        let [Value::String(id), Value::String(origin), params, result] = args.as_slice() else {
            return Err(fail());
        };
        use toolkit::executed::Origin;
        let origin = match origin.as_str() {
            "menu" => Origin::Menu,
            "shortcut" => Origin::Shortcut,
            "palette" => Origin::Palette,
            "ui" => Origin::Ui,
            "bus" => Origin::Bus,
            "macro" => Origin::Macro,
            _ => return Err(fail()),
        };
        let mut journal = journal.lock().map_err(|_| fail())?;
        journal.record(
            id,
            origin,
            crate::runtime::to_json(params)?,
            crate::runtime::to_json(result)?,
        );
        crate::runtime::from_json(journal.take().0.remove(0).body())
    })
}

/// Gold's replacing editor writer, retaining permissions and target links.
/// Mix supplies any expected bytes and decides how a refusal affects its UI.
pub(crate) fn editor_write() -> mix::evaluator::ExtFn {
    mix::sync_ext(|args| {
        let fail = |msg: String| MixError::RuntimeError { msg, span: None };
        let [Value::String(path), Value::String(text), expected] = args.as_slice() else {
            return Err(fail(
                "viu_editor_write expects path, text, expected text or nil".into(),
            ));
        };
        if !std::path::Path::new(path).is_absolute() || text.len() > 8 * 1024 * 1024 {
            return Err(fail(
                "editor writer requires an absolute path and at most 8 MiB".into(),
            ));
        }
        if let Value::String(expected) = expected {
            match std::fs::read_to_string(path) {
                Ok(current) if &current != expected => {
                    return Err(fail(
                        "CONFLICT: document changed on disk; save to another file or reopen it"
                            .into(),
                    ));
                }
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                    return Err(fail(e.to_string()));
                }
                _ => {}
            }
        } else if !matches!(expected, Value::Nil) {
            return Err(fail("expected bytes must be text or nil".into()));
        }
        let mut doc =
            editor::local::Document::open_bytes(std::path::Path::new(path), text.as_bytes())
                .map_err(|e| fail(e.to_string()))?;
        doc.save_as(path).map_err(|e| fail(e.to_string()))?;
        Ok(Value::Nil)
    })
}

/// Bounded image metadata and exclusive PNG encoding; callers own naming,
/// storage directories, save policy and document lifetime.
pub(crate) fn image_file() -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        let fail = |msg: String| MixError::RuntimeError { msg, span: None };
        let [Value::String(op), Value::String(source), rest @ ..] = args.as_slice() else {
            return Err(fail(
                "viu_image expects info/export, source, destination?".into(),
            ));
        };
        if !["info", "export"].contains(&op.as_str())
            || (op == "info" && !rest.is_empty())
            || (op == "export" && rest.len() != 1)
        {
            return Err(fail("invalid image operation arguments".into()));
        }
        let image =
            crate::image_view::decode_surface(std::path::Path::new(source)).ok_or_else(|| {
                fail("image refused by regular-file/format/byte/pixel budgets".into())
            })?;
        if op == "export" {
            let Value::String(destination) = &rest[0] else {
                return Err(fail("destination must be a path".into()));
            };
            if !std::path::Path::new(destination).is_absolute() {
                return Err(fail("destination must be absolute".into()));
            }
            crate::capture::write_png(std::path::Path::new(destination), &image).map_err(fail)?;
        }
        crate::runtime::from_json(serde_json::json!({"width":image.size[0],"height":image.size[1]}))
    })
}

/// Reuse native widget type/range/enabled admission from any Mix reducer.
pub(crate) fn widget_admit() -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        let fail = |msg: String| MixError::RuntimeError { msg, span: None };
        let [definition, model, Value::String(action), value] = args.as_slice() else {
            return Err(fail(
                "viu_widget_admit expects definition, model, action, value".into(),
            ));
        };
        let widget =
            crate::widgets::Widget::decode(&crate::runtime::to_json(definition)?).map_err(fail)?;
        widget
            .admit(
                &crate::runtime::to_json(model)?,
                action,
                &crate::runtime::to_json(value)?,
            )
            .map_err(fail)?;
        Ok(Value::Bool(true))
    })
}

/// Existing native catalogue/storage/process-group runner, owned by the host.
/// Filesystem/deferred work runs on the calling Mix worker, never the UI.
pub(crate) fn macro_commands(session: macros::Session) -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        let fail = || MixError::RuntimeError {
            msg: "viu_macros expects a command suffix and argument object".into(),
            span: None,
        };
        let [Value::String(command), body] = args.as_slice() else {
            return Err(fail());
        };
        if !macros::commands::SUFFIXES.contains(&command.as_str()) {
            return Err(fail());
        }
        let body = crate::runtime::to_json(body)?;
        let result = macros::commands::handle(&session, command, &body).wait();
        crate::runtime::from_json(match result {
            Ok(body) => serde_json::json!({"rc":0,"body":body}),
            Err(error) => {
                serde_json::json!({"rc":10,"body":{"error_code":error.code,"message":error.message}})
            }
        })
    })
}

/// The existing descriptor-relative, durable replacement primitive. Unlike
/// Mix's path-following write_atomic this refuses a symlink destination.
/// The caller owns serialization and decides whether saving is allowed.
pub(crate) fn atomic_replace() -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        let error = |msg: String| MixError::RuntimeError { msg, span: None };
        let [Value::String(path), Value::String(text), rest @ ..] = args.as_slice() else {
            return Err(error(
                "viu_atomic_replace expects absolute path and text".into(),
            ));
        };
        let limit = if rest.is_empty() {
            1024 * 1024
        } else if rest.len() == 1 {
            crate::widgets::whole(&crate::runtime::to_json(&rest[0])?)
                .filter(|n| *n <= 32 * 1024 * 1024)
                .ok_or_else(|| error("replacement limit must be at most 32 MiB".into()))?
                as usize
        } else {
            return Err(error("unexpected replacement arguments".into()));
        };
        if !std::path::Path::new(path).is_absolute() || text.len() > limit {
            return Err(error(
                "atomic replacement requires an absolute path and at most 1 MiB".into(),
            ));
        }
        config::atomic::replace(std::path::Path::new(path), text.as_bytes())
            .map_err(|e| error(e.to_string()))?;
        Ok(Value::Nil)
    })
}

/// Durable replacement with a structured post-rename uncertainty receipt.
/// The caller supplies serialised text and owns the decision to apply it.
pub(crate) fn store() -> mix::evaluator::ExtFn {
    mix::sync_ext(|args| {
        let fail = |msg: &str| MixError::RuntimeError {
            msg: msg.into(),
            span: None,
        };
        let [Value::String(path), Value::String(text)] = args.as_slice() else {
            return Err(fail("viu_store expects absolute path and text"));
        };
        let path = std::path::Path::new(path);
        if !path.is_absolute() || text.len() > 1024 * 1024 {
            return Err(fail("store requires an absolute path and at most 1 MiB"));
        }
        let result = match config::atomic::create_directory(
            path.parent().ok_or_else(|| fail("missing parent"))?,
        ) {
            Err(e) => {
                serde_json::json!({"stored":false,"unconfirmed":false,"message":e.to_string()})
            }
            Ok(_) => match config::atomic::replace(path, text.as_bytes()) {
                Ok(()) => serde_json::json!({"stored":true,"unconfirmed":false,"message":null}),
                Err(e) => {
                    serde_json::json!({"stored":e.may_have_replaced,"unconfirmed":e.may_have_replaced,"message":e.to_string()})
                }
            },
        };
        crate::runtime::from_json(result)
    })
}

/// Bounded, nonblocking optional text read through the existing held-directory
/// primitive. Only NotFound means absent; unreadable/linked/special files fail.
pub(crate) fn read_optional() -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        let fail = |msg: String| MixError::RuntimeError { msg, span: None };
        let [Value::String(path), options @ ..] = args.as_slice() else {
            return Err(fail(
                "viu_read_optional expects absolute path and optional tail_lines bound".into(),
            ));
        };
        let tail = if options.is_empty() {
            None
        } else if options.len() == 1 {
            let option = crate::runtime::to_json(&options[0])?;
            Some(
                option["tail_lines"]
                    .as_u64()
                    .filter(|n| (1..=1_048_576).contains(n))
                    .ok_or_else(|| fail("tail_lines must be 1..1048576 bytes".into()))?
                    as usize,
            )
        } else {
            return Err(fail("too many read options".into()));
        };
        let path = std::path::Path::new(path);
        if !path.is_absolute() {
            return Err(fail("expected an absolute path".into()));
        }
        let read = || -> std::io::Result<Vec<u8>> {
            let parent = config::atomic::open_directory(
                path.parent()
                    .ok_or_else(|| std::io::Error::other("missing parent"))?,
            )?;
            if let Some(limit) = tail {
                let file = config::atomic::open_nested(
                    &parent,
                    std::path::Path::new(
                        path.file_name()
                            .ok_or_else(|| std::io::Error::other("missing name"))?,
                    ),
                )?;
                read_tail_lines(file, limit)
            } else {
                config::atomic::read_in(
                    &parent,
                    path.file_name()
                        .ok_or_else(|| std::io::Error::other("missing name"))?,
                    1024 * 1024,
                )
            }
        };
        let value = match read() {
            Ok(bytes) => {
                serde_json::json!({"found":true,"text":String::from_utf8(bytes).map_err(|e|fail(e.to_string()))?})
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                serde_json::json!({"found":false})
            }
            Err(error) => return Err(fail(error.to_string())),
        };
        crate::runtime::from_json(value)
    })
}
fn read_tail_lines(mut file: std::fs::File, limit: usize) -> std::io::Result<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let skip = file.metadata()?.len().saturating_sub(limit as u64);
    file.seek(SeekFrom::Start(skip))?;
    let mut bytes = Vec::new();
    file.take(limit as u64).read_to_end(&mut bytes)?;
    if skip > 0 {
        let first = bytes
            .iter()
            .position(|b| *b == b'\n')
            .map_or(bytes.len(), |i| i + 1);
        bytes.drain(..first);
    }
    // A line reader retains complete earlier records when a later line has
    // corrupt UTF-8. Do not turn one damaged append into an empty history.
    if let Err(error) = std::str::from_utf8(&bytes) {
        let end = bytes[..error.valid_up_to()]
            .iter()
            .rposition(|b| *b == b'\n')
            .map_or(0, |i| i + 1);
        bytes.truncate(end);
    }
    Ok(bytes)
}

pub(crate) fn thumbnail(stop: Arc<AtomicBool>) -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        let fail = || MixError::RuntimeError {
            msg: "viu_thumbnail requires an absolute path and optional remote bool".into(),
            span: None,
        };
        if args.is_empty() || args.len() > 2 {
            return Err(fail());
        }
        let Some(Value::String(path)) = args.first() else {
            return Err(fail());
        };
        use std::os::unix::ffi::OsStringExt;
        let path = std::path::PathBuf::from(std::ffi::OsString::from_vec(
            files::chooser::location(&serde_json::json!(path)).map_err(|_| fail())?,
        ));
        let remote = match args.get(1) {
            None | Some(Value::Bool(false)) => false,
            Some(Value::Bool(true)) => true,
            _ => return Err(fail()),
        };
        static GENERATOR: OnceLock<Option<thumbs::generate::Generator>> = OnceLock::new();
        let result = GENERATOR.get_or_init(|| thumbs::generate::Generator::from_env("viu"));
        let value = match result {
            Some(generator) => {
                match generator.thumbnail_with(&path, thumbs::Size::XLarge, remote, &|| {
                    stop.load(Ordering::Acquire)
                }) {
                    thumbs::Outcome::Ready(path) => {
                        serde_json::json!({"phase":"ready","path":path})
                    }
                    thumbs::Outcome::Failed(error) => {
                        serde_json::json!({"phase":"failed","message":error})
                    }
                    thumbs::Outcome::Skipped(reason) => {
                        serde_json::json!({"phase":"skipped","message":reason})
                    }
                }
            }
            None => serde_json::json!({"phase":"skipped","message":"no thumbnail cache directory"}),
        };
        crate::runtime::from_json(value)
    })
}

pub(crate) fn installed_fonts() -> mix::evaluator::ExtFn {
    mix::sync_ext(|args| {
        if !args.is_empty() {
            return Err(MixError::RuntimeError {
                msg: "viu_fonts expects no arguments".into(),
                span: None,
            });
        }
        let mut database = fontdb::Database::new();
        database.load_system_fonts();
        let mut all = std::collections::BTreeSet::new();
        let mut mono = std::collections::BTreeSet::new();
        for face in database.faces() {
            for (name, _) in &face.families {
                if name.is_empty()
                    || name.len() > 128
                    || name.trim() != name
                    || name.chars().any(char::is_control)
                {
                    continue;
                }
                all.insert(name.clone());
                if face.monospaced {
                    mono.insert(name.clone());
                }
            }
        }
        crate::runtime::from_json(serde_json::json!({"all":all,"mono":mono}))
    })
}

pub(crate) fn token() -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        use std::io::Read;
        if !args.is_empty() {
            return Err(MixError::RuntimeError {
                msg: "viu_token expects no arguments".into(),
                span: None,
            });
        }
        let mut bytes = [0u8; 16];
        std::fs::File::open("/dev/urandom")
            .and_then(|mut file| file.read_exact(&mut bytes))
            .map_err(|e| MixError::RuntimeError {
                msg: format!("random token: {e}"),
                span: None,
            })?;
        Ok(Value::String(
            bytes.iter().map(|b| format!("{b:02x}")).collect(),
        ))
    })
}

pub(crate) fn settings_values() -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        let fail = || MixError::RuntimeError {
            msg: "viu_settings expects default/path or apply|admit/values/changes/reset".into(),
            span: None,
        };
        match args.as_slice() {
            [Value::String(op), account] if op == "email-fields" => {
                let account = crate::runtime::to_json(account)?;
                if !account.is_object() {
                    return Err(fail());
                }
                let text = |field: &str| account[field].as_str().unwrap_or("");
                let checks = [
                    ("name", settings::email::valid_name(text("name"))),
                    ("url", settings::email::valid_url(text("url"))),
                    ("email", settings::email::valid_address(text("email"))),
                    (
                        "ca_path",
                        account["ca_path"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .is_none_or(settings::email::valid_ca_path),
                    ),
                    (
                        "signature",
                        settings::email::valid_signature(text("signature")),
                    ),
                ];
                crate::runtime::from_json(serde_json::json!(
                    checks
                        .into_iter()
                        .filter_map(|(field, valid)| (!valid).then_some(field))
                        .collect::<Vec<_>>()
                ))
            }
            [Value::String(op), Value::String(path)] if op == "default" => {
                crate::runtime::from_json(
                    settings::registry::domain_default(path)
                        .or_else(|| settings::registry::effective(&Default::default(), path))
                        .unwrap_or(serde_json::Value::Null),
                )
            }
            [Value::String(op), values, changes, reset] if op == "apply" || op == "admit" => {
                let mut values: settings::registry::Values =
                    serde_json::from_value(crate::runtime::to_json(values)?).map_err(|_| fail())?;
                if op == "admit" {
                    // settings.get's effective view includes defaults, nulls and live
                    // observations. Restore the authored representation before asking
                    // the registry to validate the complete proposed transaction.
                    let mut authored = settings::registry::Values::new();
                    let mut diagnostics = Vec::new();
                    for (path, value) in &values {
                        if value.is_null() {
                            continue;
                        }
                        match settings::registry::lookup(path) {
                            Ok(m) if m.def.is_live() => continue,
                            Ok(m) => match settings::registry::check_value(path, value) {
                                Ok(normal) => {
                                    let default = settings::registry::domain_default(path)
                                        .or_else(|| m.default_value());
                                    if default.as_ref() != Some(&normal) {
                                        authored.insert(path.clone(), normal);
                                    }
                                }
                                Err(error) => diagnostics.push(error),
                            },
                            Err(error) => diagnostics.push(error),
                        }
                    }
                    if !diagnostics.is_empty() {
                        return crate::runtime::from_json(
                            serde_json::json!({"ok":false,"diagnostics":diagnostics}),
                        );
                    }
                    values = authored;
                }
                let changes = serde_json::from_value(crate::runtime::to_json(changes)?)
                    .map_err(|_| fail())?;
                let reset = serde_json::from_value::<Vec<String>>(crate::runtime::to_json(reset)?)
                    .map_err(|_| fail())?;
                crate::runtime::from_json(
                    match settings::registry::apply(&values, &changes, &reset) {
                        Ok(values) => {
                            serde_json::json!({"ok":true,"values":values,"diagnostics":[]})
                        }
                        Err(diagnostics) => {
                            serde_json::json!({"ok":false,"diagnostics":diagnostics})
                        }
                    },
                )
            }
            _ => Err(fail()),
        }
    })
}
#[cfg(test)]
mod tail_tests {
    #[test]
    fn line_tail_keeps_newest_complete_records_and_held_inode() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log");
        std::fs::write(&path, "€€€\nold\nnewest\n").unwrap();
        let parent = config::atomic::open_directory(dir.path()).unwrap();
        let file = config::atomic::open_nested(&parent, std::path::Path::new("log")).unwrap();
        std::fs::rename(&path, dir.path().join("old")).unwrap();
        std::fs::write(&path, "replacement").unwrap();
        assert_eq!(super::read_tail_lines(file, 13).unwrap(), b"old\nnewest\n");
        std::os::unix::fs::symlink("log", dir.path().join("link")).unwrap();
        assert!(config::atomic::open_nested(&parent, std::path::Path::new("link")).is_err());
    }

    #[test]
    fn line_tail_keeps_records_before_a_corrupt_utf8_append() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log");
        std::fs::write(&path, b"old\nnewest\npartial\xff\nlater\n").unwrap();
        let file = std::fs::File::open(path).unwrap();
        assert_eq!(
            super::read_tail_lines(file, 1024).unwrap(),
            b"old\nnewest\n"
        );
    }
}
