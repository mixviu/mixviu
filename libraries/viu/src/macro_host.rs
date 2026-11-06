// SPDX-License-Identifier: MIT OR Apache-2.0
//! Isolated Mix macro mode for the exact client artifact. The retained native
//! macro runner owns its process group, output tail and stop/drop lifecycle.
//! No installed interpreter or GUI is launched by this entry point.
use crate::native_bus::Endpoint;
use mix::{
    BusHandler, MixError, MixResult,
    evaluator::{BusFuture, CapabilityPolicy, Evaluator, IncomingEvent},
    value::Value,
};
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc, time::Duration};

struct MacroPolicy;
impl CapabilityPolicy for MacroPolicy {
    fn check_builtin(&self, _: &str) -> Result<(), String> {
        Ok(())
    }
    fn check_class(&self, class: mix::CapabilityClass) -> Result<(), String> {
        if class == mix::CapabilityClass::Process {
            Err(
                "macro mode requires native argv operations and refuses process-global export"
                    .into(),
            )
        } else {
            Ok(())
        }
    }
}
struct LazyBus {
    endpoint: RefCell<Option<Endpoint>>,
}
impl LazyBus {
    async fn handler(&self) -> MixResult<Rc<dyn BusHandler>> {
        let fail = |message: String| MixError::RuntimeError {
            msg: message,
            span: None,
        };
        if self.endpoint.borrow().is_none() {
            let url = std::env::var("MIXOS_NODED_URL")
                .map_err(|_| fail("macro Bus address is not configured".into()))?;
            *self.endpoint.borrow_mut() = Some(Endpoint::start(
                format!("viu-macro-{}", std::process::id()),
                url,
                Vec::new(),
                String::new(),
                Arc::new(|| {}),
            ));
        }
        let end = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            let connected = self.endpoint.borrow().as_ref().unwrap().client().is_some();
            if connected {
                let factory = self.endpoint.borrow().as_ref().unwrap().backend();
                return Ok(factory());
            }
            if tokio::time::Instant::now() >= end {
                return Err(fail("macro native Bus connection timed out".into()));
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}
impl BusHandler for LazyBus {
    fn send<'a>(
        &'a self,
        t: &'a str,
        c: &'a str,
        a: &'a Value,
    ) -> BusFuture<'a, MixResult<(i32, Value)>> {
        Box::pin(async move { self.handler().await?.send(t, c, a).await })
    }
    fn send_with_reply<'a>(
        &'a self,
        t: &'a str,
        c: &'a str,
        a: &'a Value,
    ) -> BusFuture<'a, MixResult<(i32, Value, Value)>> {
        Box::pin(async move { self.handler().await?.send_with_reply(t, c, a).await })
    }
    fn emit<'a>(&'a self, t: &'a str, c: &'a str, a: &'a Value) -> BusFuture<'a, MixResult<()>> {
        Box::pin(async move { self.handler().await?.emit(t, c, a).await })
    }
    fn port_exists<'a>(&'a self, t: &'a str) -> BusFuture<'a, MixResult<bool>> {
        Box::pin(async move { self.handler().await?.port_exists(t).await })
    }
    fn next_incoming<'a>(&'a self) -> BusFuture<'a, Option<IncomingEvent>> {
        Box::pin(async { None })
    }
}
pub fn run(path: PathBuf, arguments: Vec<String>) -> i32 {
    let child = std::thread::Builder::new()
        .name("viu-macro-eval".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let source = match std::fs::read_to_string(&path) {
                Ok(source) => source,
                Err(error) => {
                    eprintln!("{}: {error}", path.display());
                    return 1;
                }
            };
            mix::set_script_argv(arguments);
            let mut eval = Evaluator::default();
            eval.register("viu_files", crate::file_data::binding());
            eval.set_entry_script(&path.to_string_lossy());
            eval.set_file(path.to_string_lossy().to_string());
            eval.set_capability_policy(Rc::new(MacroPolicy));
            eval.set_bus_handler(Rc::new(LazyBus {
                endpoint: RefCell::new(None),
            }));
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("macro executor");
            let tasks = tokio::task::LocalSet::new();
            let result = runtime.block_on(tasks.run_until(eval.execute_script_source(&source)));
            eval.close_native_events();
            match result {
                Ok(_) => 0,
                Err(MixError::ExitRequest { code }) => code,
                Err(error) => {
                    eprintln!("{error}");
                    1
                }
            }
        });
    match child {
        Ok(child) => child.join().unwrap_or(101),
        Err(error) => {
            eprintln!("macro evaluator: {error}");
            101
        }
    }
}
