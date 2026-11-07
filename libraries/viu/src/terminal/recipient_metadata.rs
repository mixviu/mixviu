// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Gold protected session readback using Mix-authored tab metadata.
use super::*;
use std::collections::BTreeMap;

impl Recipient {
    pub(super) fn describe(
        &self,
        admitted: &Admission,
        parent: &SessionRecord,
        permit: &Permit,
        foreground: u64,
    ) -> Reply {
        let Some(metadata) = (self.metadata)() else {
            return Reply::error("FORBIDDEN");
        };
        if metadata.tab_of(admitted.request.target.pane_id.0).is_none() {
            return Reply::error("FORBIDDEN");
        }
        let mut ids = metadata
            .tabs
            .iter()
            .flat_map(|t| t.panes.iter().copied())
            .collect::<Vec<_>>();
        ids.sort_unstable();
        let mut native = BTreeMap::new();
        let mut panes = Vec::new();
        for id in ids {
            let Some(generation) = self.native.pane_generation(id) else {
                continue;
            };
            let Some(pane) = (self.pane)(id) else {
                continue;
            };
            let (cols, rows, pid) = {
                let pane = pane.lock().unwrap();
                let (cols, rows) = pane.dimensions();
                (cols, rows, pane.pid)
            };
            if self.native.pane_generation(id) != Some(generation) {
                continue;
            }
            let target = Target {
                pane_id: DecimalU64(id),
                pane_generation: DecimalU64(generation),
                ..admitted.request.target.clone()
            };
            if !crate::terminal::protocol::target_matches(parent, &target) {
                continue;
            }
            panes.push(json!({"target":target,"tab_id":metadata.tab_of(id),"cols":cols,"rows":rows,"child_pid":pid}));
            native.insert(id, (cols, rows, pid));
        }
        let tabs=metadata.tabs.iter().filter(|t|t.panes.iter().all(|id|native.contains_key(id))).map(|t| {
            let (cols,rows,pid)=native[&t.selected];
            json!({"id":t.id,"active":t.active,"title":t.title,"cols":cols,"rows":rows,"child_pid":pid})
        }).collect::<Vec<_>>();
        if !permit.valid() {
            return Reply::error("FORBIDDEN");
        }
        Reply::ok(json!({
            "instance_id":parent.instance_id,"incarnation":parent.incarnation,
            "target":admitted.request.target,"policy":parent.policy,
            "request_epoch":admitted.epoch,
            "request_high_water":DecimalU64(self.replay.lock().unwrap().high_water(&admitted.identity)),
            "panes":panes,"tabs":tabs,
            "binding":self.native.status()["panes"][admitted.request.target.pane_id.0.to_string()],
            "foreground_generation":foreground,"retention_seconds":900,
            "requests_per_actor":1024,"instance_request_limit":4096,
        }))
    }
}
