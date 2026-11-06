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

/// Descriptor-relative private directory creation, refusing linked ancestors.
pub(crate) fn private_directory() -> mix::evaluator::ExtFn {
    mix::sync_ext(|args| {
        let [Value::String(path)] = args.as_slice() else {
            return Err(MixError::RuntimeError {
                msg: "viu_private_directory expects an absolute path".into(),
                span: None,
            });
        };
        config::atomic::create_directory(std::path::Path::new(path)).map_err(|e| {
            MixError::RuntimeError {
                msg: format!("Private directory: {e}"),
                span: None,
            }
        })?;
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
