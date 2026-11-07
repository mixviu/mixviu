// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded in-process handoff to Mix layout policy. Authority is a native
//! callback scoped to one evaluator event; JSON carries only application data.
use super::protocol::Reply;
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::{BTreeSet, VecDeque},
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Mutex, Weak},
};

type Guard = Arc<dyn Fn(&[u64]) -> Result<(), Reply> + Send + Sync>;
type Finish = Box<dyn FnOnce(&mut Reply, bool) + Send>;
struct Completion {
    answer: tokio::sync::oneshot::Sender<Reply>,
    finish: Finish,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn bridge() -> Bridge {
        Bridge::new(Arc::new(|| {}))
    }
    fn guard() -> Guard {
        Arc::new(|ids| {
            if ids == [1] {
                Ok(())
            } else {
                Err(Reply::error("FORBIDDEN"))
            }
        })
    }
    fn finish() -> Finish {
        Box::new(|_, _| {})
    }
    fn error(reply: &Reply) -> String {
        serde_json::from_str::<Value>(&reply.body).unwrap()["error_code"]
            .as_str()
            .unwrap()
            .into()
    }

    #[test]
    fn scope_is_native_owner_bound_and_commit_is_single_use() {
        let bridge = bridge();
        let other = Bridge::new(Arc::new(|| {}));
        let (job, answer) = bridge
            .enqueue(json!({"pane":1}), guard(), finish())
            .ok()
            .unwrap();
        assert!(
            bridge
                .call("control.inspect", json!({"pane":1,"token":1}))
                .is_err()
        );
        assert!(bridge.take().is_some());
        let scope = (job.context())();
        assert_eq!(
            bridge.call("control.inspect", json!({})).unwrap(),
            json!({"pane":1})
        );
        assert!(other.call("control.inspect", json!({})).is_err());
        assert_eq!(
            bridge
                .call("control.begin", json!({"affected":[1]}))
                .unwrap()["rc"],
            0
        );
        assert_eq!(
            bridge
                .call("control.begin", json!({"affected":[1]}))
                .unwrap()["rc"],
            10
        );
        bridge
            .call("control.finish", json!({"rc":0,"body":{"selected":1}}))
            .unwrap();
        drop(scope);
        let reply = answer.blocking_recv().unwrap();
        assert_eq!(reply.rc, 0);
        assert!(bridge.call("control.inspect", json!({})).is_err());
    }

    #[test]
    fn abandoned_cancelled_and_refused_callbacks_never_claim_success() {
        let bridge = bridge();
        let (job, answer) = bridge.enqueue(json!({}), guard(), finish()).ok().unwrap();
        job.cancel();
        let scope = (job.context())();
        assert_eq!(
            bridge
                .call("control.begin", json!({"affected":[1]}))
                .unwrap()["rc"],
            10
        );
        drop(scope);
        assert_eq!(error(&answer.blocking_recv().unwrap()), "FORBIDDEN");
        let committed = Arc::new(AtomicUsize::new(0));
        let seen = committed.clone();
        let (job, answer) = bridge
            .enqueue(
                json!({}),
                guard(),
                Box::new(move |_, begun| {
                    if begun {
                        seen.fetch_add(1, Ordering::SeqCst);
                    }
                }),
            )
            .ok()
            .unwrap();
        let scope = (job.context())();
        assert_eq!(
            bridge
                .call("control.begin", json!({"affected":[2]}))
                .unwrap()["rc"],
            10
        );
        assert_eq!(
            bridge
                .call("control.begin", json!({"affected":[1]}))
                .unwrap()["rc"],
            0
        );
        drop(scope);
        assert_eq!(error(&answer.blocking_recv().unwrap()), "UNKNOWN_OUTCOME");
        assert_eq!(committed.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn queue_is_bounded_and_cancelled_entries_release_capacity() {
        let bridge = bridge();
        let mut jobs = Vec::new();
        for _ in 0..32 {
            jobs.push(bridge.enqueue(json!({}), guard(), finish()).ok().unwrap().0);
        }
        assert_eq!(
            error(&bridge.enqueue(json!({}), guard(), finish()).err().unwrap()),
            "RESOURCE_LIMIT"
        );
        for job in jobs {
            job.cancel();
        }
        assert!(bridge.enqueue(json!({}), guard(), finish()).is_ok());
    }

    #[test]
    fn runtime_mailbox_context_cannot_be_supplied_by_an_ordinary_event() {
        use crate::runtime::{Completion, Program, Runtime, Services};
        fn next(runtime: &Runtime) -> Completion {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            loop {
                if let Some(done) = runtime.try_completion() {
                    return done;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "native callback did not complete"
                );
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
        let sessions = crate::terminal::sessions::Sessions::default();
        let bridge = sessions.layout_bridge();
        let mut runtime = Runtime::with_services(
            8,
            Arc::new(|| {}),
            None,
            Services {
                terminals: Some(sessions),
                ..Services::default()
            },
        );
        let source = r#"on protected
  $request=viu_terminal("control.inspect","",{})
  $begin=viu_terminal("control.begin","",{affected:[$request.pane]})
  if $begin.rc==0 then
    $done=viu_terminal("control.finish","",{rc:0,body:{selected:$request.pane}})
  else
    $done=viu_terminal("control.finish","",$begin)
  end
end
"#;
        runtime
            .load(Program {
                path: "/tmp/terminal-layout-test.mix".into(),
                source: source.into(),
                scene: "test".into(),
                citizen: "test".into(),
            })
            .unwrap();
        assert!(next(&runtime).error.is_none());
        runtime
            .event("protected".into(), json!({"pane":1,"token":1}))
            .unwrap();
        assert!(
            next(&runtime)
                .error
                .unwrap()
                .contains("native layout context required")
        );
        let (job, answer) = bridge
            .enqueue(json!({"pane":1}), guard(), finish())
            .ok()
            .unwrap();
        assert!(bridge.take().is_some());
        runtime
            .native_event("protected".into(), json!({"pane":999}), job.context())
            .unwrap();
        assert!(next(&runtime).error.is_none());
        let reply = answer.blocking_recv().unwrap();
        assert_eq!(reply.rc, 0);
        assert_eq!(
            serde_json::from_str::<Value>(&reply.body).unwrap(),
            json!({"selected":1})
        );
        runtime
            .event("protected".into(), json!({"pane":1}))
            .unwrap();
        assert!(
            next(&runtime)
                .error
                .unwrap()
                .contains("native layout context required")
        );
    }
}
#[derive(Default)]
struct State {
    begun: bool,
    done: bool,
    completion: Option<Completion>,
}
struct Inner {
    queue: Mutex<VecDeque<Arc<Job>>>,
    wake: crate::runtime::Wake,
}
#[derive(Clone)]
pub(crate) struct Bridge(Arc<Inner>);
pub(crate) struct Job {
    owner: Weak<Inner>,
    payload: Value,
    guard: Guard,
    state: Mutex<State>,
}
thread_local! {
    static CURRENT: RefCell<Option<Arc<Job>>> = const {RefCell::new(None)};
}
struct Scope {
    job: Arc<Job>,
    previous: Option<Arc<Job>>,
    _thread: PhantomData<Rc<()>>,
}
impl Drop for Scope {
    fn drop(&mut self) {
        CURRENT.with(|current| *current.borrow_mut() = self.previous.take());
        self.job.cancel();
    }
}

impl Bridge {
    pub fn new(wake: crate::runtime::Wake) -> Self {
        Self(Arc::new(Inner {
            queue: Mutex::default(),
            wake,
        }))
    }
    pub fn enqueue(
        &self,
        payload: Value,
        guard: Guard,
        finish: Finish,
    ) -> Result<(Arc<Job>, tokio::sync::oneshot::Receiver<Reply>), Reply> {
        let mut queue = self.0.queue.lock().unwrap();
        queue.retain(|job| !job.state.lock().unwrap().done);
        if queue.len() >= 32 {
            return Err(Reply::error("RESOURCE_LIMIT"));
        }
        let (answer, receive) = tokio::sync::oneshot::channel();
        let job = Arc::new(Job {
            owner: Arc::downgrade(&self.0),
            payload,
            guard,
            state: Mutex::new(State {
                completion: Some(Completion { answer, finish }),
                ..State::default()
            }),
        });
        queue.push_back(job.clone());
        drop(queue);
        (self.0.wake)();
        Ok((job, receive))
    }
    pub fn take(&self) -> Option<Arc<Job>> {
        let mut queue = self.0.queue.lock().unwrap();
        while let Some(job) = queue.pop_front() {
            if !job.state.lock().unwrap().done {
                return Some(job);
            }
        }
        None
    }
    pub fn call(&self, operation: &str, args: Value) -> Result<Value, String> {
        let job = CURRENT
            .with(|current| current.borrow().clone())
            .filter(|job| Weak::ptr_eq(&job.owner, &Arc::downgrade(&self.0)))
            .ok_or("native layout context required")?;
        match operation {
            "control.inspect" => Ok(job.payload.clone()),
            "control.begin" => {
                let answer = job.begin(&args);
                Ok(
                    json!({"rc":answer.rc,"body":serde_json::from_str::<Value>(&answer.body).map_err(|e|e.to_string())?}),
                )
            }
            "control.finish" => {
                let rc = crate::widgets::whole(&args["rc"])
                    .filter(|n| matches!(n, 0 | 10))
                    .ok_or("layout reply rc must be 0 or 10")? as u8;
                let mut reply = Reply {
                    rc,
                    body: args["body"].to_string(),
                };
                if reply.body.len() > 256 * 1024 {
                    reply = Reply::error("UNKNOWN_OUTCOME");
                }
                job.complete(reply);
                Ok(json!({"finished":true}))
            }
            _ => Err("unknown layout control operation".into()),
        }
    }
}

impl Job {
    pub fn context(self: &Arc<Self>) -> crate::runtime::NativeContext {
        let job = self.clone();
        Box::new(move || {
            let previous = CURRENT.with(|current| current.replace(Some(job.clone())));
            Box::new(Scope {
                job,
                previous,
                _thread: PhantomData,
            })
        })
    }
    fn begin(&self, args: &Value) -> Reply {
        let mut state = self.state.lock().unwrap();
        if state.done || state.begun {
            return Reply::error("FORBIDDEN");
        }
        let Some(affected) = args["affected"]
            .as_array()
            .filter(|ids| !ids.is_empty() && ids.len() <= 32)
        else {
            return Reply::error("FORBIDDEN");
        };
        let Some(ids) = affected
            .iter()
            .map(|id| crate::widgets::whole(id).filter(|n| *n > 0))
            .collect::<Option<Vec<_>>>()
        else {
            return Reply::error("FORBIDDEN");
        };
        if ids.iter().copied().collect::<BTreeSet<_>>().len() != ids.len() {
            return Reply::error("FORBIDDEN");
        }
        if let Err(reply) = (self.guard)(&ids) {
            return reply;
        }
        state.begun = true;
        Reply::ok(json!({"begun":true}))
    }
    pub fn cancel(&self) {
        self.settle(None);
    }
    pub fn complete(&self, reply: Reply) {
        self.settle(Some(reply));
    }
    fn settle(&self, supplied: Option<Reply>) {
        let mut reply;
        let (completion, begun) = {
            let mut state = self.state.lock().unwrap();
            if state.done {
                return;
            }
            reply = supplied.unwrap_or_else(|| {
                Reply::error(if state.begun {
                    "UNKNOWN_OUTCOME"
                } else {
                    "FORBIDDEN"
                })
            });
            if !state.begun && reply.rc == 0 {
                reply = Reply::error("FORBIDDEN");
            }
            state.done = true;
            (state.completion.take(), state.begun)
        };
        if let Some(completion) = completion {
            (completion.finish)(&mut reply, begun);
            let _ = completion.answer.send(reply);
        }
    }
}
