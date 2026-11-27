//! Account-specific preserve-on-omit semantics, before the substrate computes
//! its canonical record, fields_changed and audit digest. This is a decorator
//! over the existing store, not a second persistence path.
use ::props::{
    namespace::NamespaceName,
    record::{AuditEpoch, Nseq, Record, RecordEvent, RecordKey, Version},
    store::{
        CompleteCommit, DeleteCommit, MergeMode, PropertyStore, ReconcileCommit, SetCommit,
        Snapshot, StoreError, StoreFuture,
    },
    value::PropValue,
};
use std::sync::Arc;

pub(super) struct AccountGuardStore(pub Arc<dyn PropertyStore>);

impl PropertyStore for AccountGuardStore {
    fn get<'a>(&'a self, key: &'a RecordKey) -> StoreFuture<'a, Snapshot<Record>> {
        self.0.get(key)
    }
    fn list<'a>(&'a self, ns: &'a NamespaceName) -> StoreFuture<'a, Snapshot<Vec<Record>>> {
        self.0.list(ns)
    }
    fn commit_set<'a>(&'a self, mut op: SetCommit) -> StoreFuture<'a, (Record, RecordEvent)> {
        Box::pin(async move {
            if op.key.namespace == super::accounts::namespace_name()
                && op.merge == MergeMode::Replace
            {
                // Read the tombstone-aware anchor before the value. For an
                // absent account this fences a concurrent create/recreate.
                let anchor = self
                    .0
                    .version_anchor(&op.key)
                    .await?
                    .unwrap_or(Version::zero());
                let prior = match self.0.get(&op.key).await {
                    Ok(snap) => Some(snap.value),
                    Err(StoreError::NotFound) => None,
                    Err(error) => return Err(error),
                };
                let observed = prior.as_ref().map_or(anchor, |record| record.version);
                // Fence preservation even when an operator omits if_version.
                match op.expected_version {
                    Some(expected) if expected != observed => {
                        return Err(StoreError::VersionMismatch {
                            expected,
                            current: observed,
                        });
                    }
                    _ => op.expected_version = Some(observed),
                }
                if let Some(prior) = prior {
                    if let (PropValue::Object(new), PropValue::Object(old)) =
                        (&mut op.new_value, prior.value)
                    {
                        for field in ["allowed_recipients", "send_rate"] {
                            if !new.contains_key(field)
                                && let Some(value) = old.get(field)
                            {
                                new.insert(field.into(), value.clone());
                            }
                        }
                    }
                }
            }
            self.0.commit_set(op).await
        })
    }
    fn commit_delete<'a>(&'a self, op: DeleteCommit) -> StoreFuture<'a, RecordEvent> {
        self.0.commit_delete(op)
    }
    fn commit_complete<'a>(&'a self, op: CompleteCommit) -> StoreFuture<'a, (Record, RecordEvent)> {
        self.0.commit_complete(op)
    }
    fn commit_reconcile<'a>(
        &'a self,
        op: ReconcileCommit,
    ) -> StoreFuture<'a, (Record, RecordEvent)> {
        self.0.commit_reconcile(op)
    }
    fn events_since<'a>(
        &'a self,
        ns: &'a NamespaceName,
        since: Nseq,
    ) -> StoreFuture<'a, Vec<RecordEvent>> {
        self.0.events_since(ns, since)
    }
    fn audit_epoch<'a>(&'a self, ns: &'a NamespaceName) -> StoreFuture<'a, AuditEpoch> {
        self.0.audit_epoch(ns)
    }
    fn version_anchor<'a>(&'a self, key: &'a RecordKey) -> StoreFuture<'a, Option<Version>> {
        self.0.version_anchor(key)
    }
}
