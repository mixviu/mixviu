// SPDX-License-Identifier: MIT OR Apache-2.0
//! Small evaluator bindings to existing native primitives. The Mix program
//! chooses the file, remote-read policy and lifetime, normally on a Jobs worker.
use mix::{MixError, value::Value};
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};

/// Installed font names and the database's monospace flag. Call on an owned
/// worker; the application chooses ordering, defaults and presentation.
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

/// OS-random correlation identifiers without enabling Mix's optional crypto
/// bundle. Account names, request tokens and operation receipts remain policy
/// owned by their Mix caller; this binding provides only 128 random bits.
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

/// Pure settings admission/defaults through the retained native registry.
/// The caller owns navigation, edits, fences, service traffic and retries.
pub(crate) fn settings_values() -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        let fail = || MixError::RuntimeError {
            msg: "viu_settings expects default/path or apply|admit/values/changes/reset".into(),
            span: None,
        };
        match args.as_slice() {
            [Value::String(op), account] if op == "email-fields" => {
                let account = crate::runtime::to_json(account)?;
                if !account.is_object() { return Err(fail()); }
                let text = |field: &str| account[field].as_str().unwrap_or("");
                let checks = [
                    ("name", settings::email::valid_name(text("name"))),
                    ("url", settings::email::valid_url(text("url"))),
                    ("email", settings::email::valid_address(text("email"))),
                    ("ca_path", account["ca_path"].as_str().filter(|s| !s.is_empty()).is_none_or(settings::email::valid_ca_path)),
                    ("signature", settings::email::valid_signature(text("signature"))),
                ];
                crate::runtime::from_json(serde_json::json!(checks.into_iter().filter_map(|(field, valid)| (!valid).then_some(field)).collect::<Vec<_>>()))
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
        let [Value::String(path), Value::String(text)] = args.as_slice() else {
            return Err(error(
                "viu_atomic_replace expects absolute path and text".into(),
            ));
        };
        if !std::path::Path::new(path).is_absolute() || text.len() > 1024 * 1024 {
            return Err(error(
                "atomic replacement requires an absolute path and at most 1 MiB".into(),
            ));
        }
        config::atomic::replace(std::path::Path::new(path), text.as_bytes())
            .map_err(|e| error(e.to_string()))?;
        Ok(Value::Nil)
    })
}

/// Bounded, nonblocking optional text read through the existing held-directory
/// primitive. Only NotFound means absent; unreadable/linked/special files fail.
pub(crate) fn read_optional() -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        let fail = |msg: String| MixError::RuntimeError { msg, span: None };
        let [Value::String(path)] = args.as_slice() else {
            return Err(fail("viu_read_optional expects one absolute path".into()));
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
            config::atomic::read_in(
                &parent,
                path.file_name()
                    .ok_or_else(|| std::io::Error::other("missing name"))?,
                1024 * 1024,
            )
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
