// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Current settingsd policy, shared with the registry filter.
use std::sync::{LazyLock, RwLock};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub default: bool,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            default: false,
            allow: vec!["viu".into()],
            deny: vec![],
        }
    }
}
impl Policy {
    /// An unknown executable follows the global default, without list matching.
    pub fn allows(&self, executable: Option<&str>) -> bool {
        let Some(name) = executable else {
            return self.default;
        };
        if self.deny.iter().any(|entry| entry == name) {
            return false;
        }
        self.allow.iter().any(|entry| entry == name) || self.default
    }
}
static CURRENT: LazyLock<RwLock<Policy>> = LazyLock::new(|| RwLock::new(Policy::default()));
pub fn install(policy: Policy) -> bool {
    let mut current = CURRENT.write().unwrap();
    if *current == policy {
        return false;
    }
    *current = policy;
    true
}
pub fn refusal_settings(executable: Option<&str>) -> Vec<&'static str> {
    let policy = CURRENT.read().unwrap();
    if executable.is_some_and(|name| policy.deny.iter().any(|entry| entry == name)) {
        vec!["agent_seat.deny"]
    } else if executable.is_some_and(|name| policy.allow.iter().any(|entry| entry == name)) {
        vec!["agent_seat.allow"]
    } else {
        vec!["agent_seat.allow", "agent_seat.default"]
    }
}
pub fn allows(executable: Option<&str>) -> bool {
    CURRENT.read().unwrap().allows(executable)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_and_deny_precedence() {
        let mut policy = Policy::default();
        assert!(policy.allows(Some("viu")));
        assert!(!policy.allows(Some("browser")));
        assert!(!policy.allows(None));
        policy.default = true;
        assert!(policy.allows(None));
        policy.deny.push("viu".into());
        assert!(!policy.allows(Some("viu")));
        assert!(policy.allows(Some("browser")));
    }
}
