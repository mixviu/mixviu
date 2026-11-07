// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native admission and retained outcomes around the Mix-owned layout callback.
use super::*;

impl Recipient {
    pub(super) async fn layout(
        &self,
        connection: &VerifiedConnection,
        parent: &SessionRecord,
        own: &(Hello, Deadline),
        admitted: Admission,
    ) -> Reply {
        let target = admitted.request.target.clone();
        let permit = Arc::new(Permit {
            connection: Arc::new(connection.client().connection_liveness()),
            live: AtomicBool::new(true),
            parent: own.1.clone(),
            hello: own.0.clone(),
            until: Instant::now() + Duration::from_secs(2),
            pane: self.native.pane_guard(target.pane_id.0, target.pane_generation.0),
            written: AtomicU64::new(0),
            notice: Mutex::new(None),
        });
        if !permit.valid() || (self.pane)(target.pane_id.0).is_none() {
            return Reply::error("FORBIDDEN");
        }
        self.permits.lock().unwrap().push(Arc::downgrade(&permit));
        let payload = json!({"operation":admitted.operation,"pane":target.pane_id.0,"dir":admitted.request.dir,"target":target});
        let identity = admitted.identity.clone();
        let sequence = admitted.request.request_id;
        let digest = admitted.digest;
        let replay = self.replay.clone();
        let native = self.native.clone();
        let parent = parent.clone();
        let admitted = Mutex::new(admitted);
        let guard = Arc::new(move |affected: &[u64]| {
            let mut admitted = admitted.lock().unwrap();
            if !permit.valid() || !affected.contains(&admitted.request.target.pane_id.0) {
                return Err(Reply::error("FORBIDDEN"));
            }
            for id in affected {
                let candidate = if *id == admitted.request.target.pane_id.0 {
                    Some(&admitted.request.target)
                } else {
                    admitted.request.affected.iter().find(|target| target.pane_id.0 == *id)
                };
                if candidate.is_none_or(|target| {
                    !super::super::protocol::target_matches(&parent, target)
                        || !native.pane_guard(target.pane_id.0, target.pane_generation.0)()
                }) {
                    return Err(Reply::error("FORBIDDEN"));
                }
            }
            admitted.mutation_epoch()?;
            let sequence = admitted.request.request_id.map(|id| id.0).filter(|id| *id > 0)
                .ok_or_else(|| Reply::error("INVALID_ARGUMENT"))?;
            replay.lock().unwrap().reserve_request_id(&admitted.identity, sequence, admitted.digest)?;
            if !permit.valid() {return Err(Reply::error("FORBIDDEN"));}
            Ok(())
        });
        let replay = self.replay.clone();
        let finish = Box::new(move |reply: &mut Reply, begun: bool| {
            if begun {
                // Every begun request has reserved a positive native sequence.
                replay.lock().unwrap().record_local_outcome(&identity, &target, sequence.unwrap().0, digest, reply);
            }
        });
        let (job, answer) = match self.layout.enqueue(payload, guard, finish) {
            Ok(job) => job,
            Err(reply) => return reply,
        };
        // A late callback cannot commit after cancellation. If commitment has
        // begun, cancellation retains UNKNOWN_OUTCOME and prevents reexecution.
        match tokio::time::timeout(Duration::from_secs(22), answer).await {
            Ok(Ok(reply)) => reply,
            _ => {job.cancel(); Reply::error("UNKNOWN_OUTCOME")}
        }
    }
}
