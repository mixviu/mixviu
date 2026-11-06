// SPDX-License-Identifier: MIT OR Apache-2.0
//! Small evaluator bindings to existing native primitives. The Mix program
//! chooses the file, remote-read policy and lifetime, normally on a Jobs worker.
use mix::{MixError, value::Value};
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};

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

/// Disk catalogue access for library browsers. Routing to a running app and
/// workflow decisions remain in the calling Mix program.
pub(crate) fn macro_library(session: macros::Session) -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        if !args.is_empty() { return Err(MixError::RuntimeError {msg:"viu_macro_library expects no arguments".into(),span:None}); }
        let roots=session.roots().ok_or_else(|| MixError::RuntimeError {msg:"macro roots unavailable".into(),span:None})?;
        let library=macros::Library::scan(&roots).map_err(|e|MixError::RuntimeError{msg:e.to_string(),span:None})?;
        let mut out=serde_json::Map::new();
        let global=macros::commands::handle(&session,"reload",&serde_json::json!({})).wait().map_err(|e|MixError::RuntimeError{msg:e.to_string(),span:None})?;
        out.insert("global".into(),global);
        for (app,catalogue) in library.apps {
            if !macros::valid_destination(&app) {continue;}
            let local=macros::Session::with_roots(&app,Some(roots.clone()));
            local.replace_catalogue(catalogue);
            let mut list=macros::commands::handle(&local,"list",&serde_json::json!({})).wait().map_err(|e|MixError::RuntimeError{msg:e.to_string(),span:None})?;
            list["running"]=serde_json::json!(false);out.insert(app,list);
        }
        crate::runtime::from_json(serde_json::Value::Object(out))
    })
}

pub(crate) fn macro_storage(session: macros::Session) -> mix::evaluator::ExtFn {
    mix::sync_ext(move |args| {
        let fail=||MixError::RuntimeError{msg:"viu_macro_storage expects app, suffix and body".into(),span:None};
        let [Value::String(app),Value::String(suffix),body]=args.as_slice() else{return Err(fail());};
        if (app!="global" && !macros::valid_destination(app)) || !macros::commands::SUFFIXES.contains(&suffix.as_str()) {return Err(fail());}
        let owner=if app=="global" {session.clone()}else{macros::Session::with_roots(app,session.roots())};
        let body=crate::runtime::to_json(body)?;
        crate::runtime::from_json(match macros::commands::handle(&owner,suffix,&body).wait() {
            Ok(body)=>serde_json::json!({"rc":0,"body":body}),
            Err(e)=>serde_json::json!({"rc":10,"body":{"error_code":e.code,"message":e.message}}),
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
