// SPDX-License-Identifier: MIT OR Apache-2.0
//! Authored early launch decisions. No Bus, window, PTY or runtime worker is
//! created here. A bounded one-shot evaluator keeps inherited descriptors
//! untouched until an authored native configuration operation quarantines
//! them before its first child process.
use crate::{bundle::Bundle, runtime::Program};
use serde::Deserialize;
use serde_json::Value;
use std::{rc::Rc, time::Duration};

#[derive(Debug, Deserialize)]
#[serde(tag="mode",rename_all="lowercase",deny_unknown_fields)]
pub(crate) enum Choice {
    Exit { #[serde(deserialize_with="exit_code")] code:u8, stdout:String, stderr:String },
    Window { context:Value },
    Headless { context:Value, commands:Vec<String> },
}
fn exit_code<'de,D:serde::Deserializer<'de>>(deserializer:D)->Result<u8,D::Error> {
    let value=Value::deserialize(deserializer)?;
    crate::widgets::whole(&value).filter(|n|*n<=255).map(|n|n as u8)
        .ok_or_else(||serde::de::Error::custom("exit code must be a whole number in 0..255"))
}
pub(crate) fn evaluate(program: &Program) -> Result<Choice,String> {
    // Match the existing evaluator's stack budget in debug as well as release.
    // The caller joins this finite owner before any application owners start.
    std::thread::scope(|scope| {
        std::thread::Builder::new().name("viu-startup".into()).stack_size(64*1024*1024)
            .spawn_scoped(scope,move || evaluate_inner(program)).map_err(|e|e.to_string())?
            .join().map_err(|_|"startup evaluator panicked".to_owned())?
    })
}
fn evaluate_inner(program: &Program) -> Result<Choice,String> {
    // A current-thread executor creates no background worker or process.
    let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e|e.to_string())?;
    let mut eval=mix::evaluator::Evaluator::with_output(Box::new(std::io::sink()),Box::new(std::io::sink()));
    eval.set_capability_policy(Rc::new(crate::runtime::WorkerPolicy));
    eval.set_entry_script(&program.path.to_string_lossy());
    eval.set_file(program.path.to_string_lossy().to_string());
    eval.set_limits(mix::EvalLimits {time_limit:Some(Duration::from_secs(5)),max_string_len:Some(1_048_576),max_list_len:Some(1024),max_map_len:Some(1024),..Default::default()});
    let context=crate::runtime::context(program,0);
    eval.register("viu_context",mix::sync_ext(move |_|crate::runtime::from_json(context.clone())));
    let terminals=crate::terminal::sessions::Sessions::default();
    eval.register("viu_terminal",mix::sync_ext(move |args| {
        let [mix::value::Value::String(op),mix::value::Value::String(handle),body]=args.as_slice() else {
            return Err(mix::MixError::RuntimeError {msg:"startup terminal operation needs operation, handle and body".into(),span:None});
        };
        if !["config","font.parse"].contains(&op.as_str()) || !handle.is_empty() {
            return Err(mix::MixError::RuntimeError {msg:"startup terminal operation is not configuration".into(),span:None});
        }
        let value=terminals.call(op,handle,crate::runtime::to_json(body)?).map_err(|msg|mix::MixError::RuntimeError {msg,span:None})?;
        crate::runtime::from_json(value)
    }));
    let value=rt.block_on(async {
        tokio::time::timeout(Duration::from_secs(5),eval.execute_script_source(&program.source)).await
    }).map_err(|_|"startup deadline exceeded".to_owned())?.map_err(|e|e.to_string())?;
    let value=crate::runtime::to_json(&value).map_err(|e|e.to_string())?;
    let choice:Choice=serde_json::from_value(value).map_err(|e|format!("invalid startup result: {e}"))?;
    match &choice {
        Choice::Exit {stdout,stderr,..} if stdout.len()+stderr.len()>1_048_576=>return Err("startup output exceeds budget".into()),
        Choice::Window {context}|Choice::Headless {context,..} if !context.is_object()=>return Err("startup context must be a map".into()),
        _=>{}
    }
    Ok(choice)
}
pub(crate) fn prepare(bundle: &mut Bundle, choice:Choice) -> Result<Option<(u8,String,String)>,String> {
    match choice {
        Choice::Exit {code,stdout,stderr}=>Ok(Some((code,stdout,stderr))),
        Choice::Window {context}=>{bundle.startup_context(&context);Ok(None)},
        Choice::Headless {context,commands}=>{
            if commands.is_empty() || commands.len()>256 || commands.iter().any(|c|!bundle.manifest.commands.contains(c)) {
                return Err("headless startup commands must be declared bundle commands".into());
            }
            bundle.startup_context(&context);
            bundle.manifest.commands=commands;
            bundle.manifest.command_descriptors.retain(|d|bundle.manifest.commands.contains(&d.name));
            bundle.manifest.surface="headless".into();
            bundle.manifest.frame=None;
            bundle.manifest.native_debug |= bundle.manifest.native_drive;
            bundle.manifest.native_drive=false;
            bundle.manifest.native_ui=false;
            bundle.manifest.bus_surface=None;
            bundle.manifest.activation=None;
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn program(path:std::path::PathBuf,source:&str)->Program {
        Program {path,source:source.into(),scene:"startup-test".into(),citizen:"startup-test".into()}
    }
    #[test]
    fn early_result_is_bounded_typed_and_cannot_start_a_terminal() {
        let dir=tempfile::tempdir().unwrap();
        let path=dir.path().join("startup.mix");
        for source in [
            "return {mode:\"exit\",code:0,stdout:\"ok\\n\",stderr:\"\"}",
            "return {mode:\"window\",context:{text:\"literal \\${value}\"}}",
            "return {mode:\"headless\",context:{},commands:[\"HELP\"]}",
        ] { assert!(evaluate(&program(path.clone(),source)).is_ok(),"{source}"); }
        for source in [
            "return {mode:\"exit\",code:256,stdout:\"\",stderr:\"\"}",
            "return {mode:\"exit\",code:0.5,stdout:\"\",stderr:\"\"}",
            "return {mode:\"window\",context:[]}",
            "return {mode:\"headless\",context:{},commands:[],extra:true}",
            "return viu_terminal(\"create\",\"\",{})",
            "exit(0)",
        ] { assert!(evaluate(&program(path.clone(),source)).is_err(),"{source}"); }
    }
    #[test]
    fn real_authored_startup_precedes_every_native_owner() {
        let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/term");
        for (args,code) in [(vec!["--version","--unknown"],0),(vec!["--unknown"],2),(vec!["--unknown","--help"],0),(vec!["--print-config"],0)] {
            let bundle=Bundle::load(&root,&args.into_iter().map(String::from).collect::<Vec<_>>()).unwrap();
            match evaluate(bundle.startup.as_ref().unwrap()).unwrap() {
                Choice::Exit {code:actual,stdout,stderr}=>{
                    assert_eq!(actual,code);
                    if code==2 {assert!(stdout.is_empty());assert_eq!(stderr,"term: unknown option --unknown (see --help)\n");}
                    else {assert!(!stdout.is_empty());assert!(stderr.is_empty());}
                },
                choice=>panic!("expected early exit: {choice:?}"),
            }
        }
    }
    #[test]
    fn headless_choice_retains_macros_and_debug_but_excludes_window_commands() {
        let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/term");
        let mut bundle=Bundle::load(&root,&["--headless".into()]).unwrap();
        let choice=evaluate(bundle.startup.as_ref().unwrap()).unwrap();
        assert!(prepare(&mut bundle,choice).unwrap().is_none());
        assert_eq!(bundle.manifest.surface,"headless");
        assert_eq!(bundle.manifest.commands.len(),36);
        assert!(bundle.manifest.native_macros && bundle.manifest.native_debug);
        assert!(!bundle.manifest.native_ui && !bundle.manifest.native_drive);
        assert!(bundle.manifest.frame.is_none() && bundle.manifest.activation.is_none());
        for command in ["term.actions","term.show","term.window.state","term.ui.capture","viu.state"] {
            assert!(!bundle.manifest.commands.iter().any(|c|c==command));
        }
        assert!(bundle.program.source.contains("\\\"headless\\\":true"));
        let bad=Choice::Headless {context:serde_json::json!({}),commands:vec!["undeclared.command".into()]};
        assert!(prepare(&mut bundle,bad).is_err());
    }
}
