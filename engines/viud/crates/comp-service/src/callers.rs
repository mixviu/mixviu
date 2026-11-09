// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

//! Bounded, audit-only caller inventory. Never retains payloads or headers.
use bus::SupervisedDelivery;
use comp_model::catalogue::{all_commands, command_class};
use serde_json::{Value, json};
use std::collections::{BTreeMap, VecDeque};

const RECENT: usize = 128;

#[derive(Default)]
pub struct CallerAudit {
    total: u64,
    legacy: u64,
    classes: BTreeMap<&'static str, u64>,
    commands: BTreeMap<&'static str, u64>,
    recent: VecDeque<Value>,
}

impl CallerAudit {
    pub fn record(&mut self, delivery: &SupervisedDelivery) {
        let command = delivery.command();
        if command.is_topic_delivery() {
            return;
        }
        let class = command_class(&command.command).map_or("other", |c| c.as_str());
        // Unknown names share one bucket; hostile names cannot grow the map.
        let name = all_commands().find(|&c| c == command.command).unwrap_or(
            match command.command.as_str() {
                "HELP" => "HELP",
                "app.describe" => "app.describe",
                "viud.truth" => "viud.truth",
                _ => "unknown",
            },
        );
        self.total = self.total.saturating_add(1);
        let counter = self.classes.entry(class).or_default();
        *counter = counter.saturating_add(1);
        let counter = self.commands.entry(name).or_default();
        *counter = counter.saturating_add(1);
        let caller = match delivery.principal() {
            Some(p) => json!({"assurance": p.assurance, "uid": p.unix_uid, "pid": p.peer_pid,
                "session_role": p.session.as_ref().map(|s| s.role),
                "registered_name": command.from.chars().take(64).collect::<String>()}),
            None => {
                self.legacy = self.legacy.saturating_add(1);
                json!("legacy/no principal")
            }
        };
        if self.recent.len() == RECENT {
            self.recent.pop_front();
        }
        self.recent.push_back(json!({"sequence": self.total, "class": class, "command": name, "caller": caller,
            "protected_recipient": delivery.protected_recipient(), "generation": command.generation}));
    }

    pub fn snapshot(&self, protected_recipient: bool) -> Value {
        json!({"mode": "observe", "protected_recipient": protected_recipient,
            "total": self.total, "legacy": self.legacy, "classes": self.classes,
            "commands": self.commands, "recent_capacity": RECENT, "recent": self.recent})
    }
}
