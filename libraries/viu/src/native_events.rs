// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded native publication queue. Loss stays in FIFO order across topics.
use serde_json::{Value, json};
use std::collections::VecDeque;
struct Item {
    topic: String,
    body: Value,
    carried: u64,
}
#[derive(Default)]
pub struct Events {
    queue: VecDeque<Item>,
    lost: u64,
    trailing: u64,
}
impl Events {
    pub fn push(&mut self, topic: String, body: Value, lost: u64) {
        let mut carried = std::mem::take(&mut self.trailing) + lost;
        if self.queue.len() >= 256 {
            if let Some(old) = self.queue.pop_front() {
                if let Some(next) = self.queue.front_mut() {
                    next.carried += old.carried + 1;
                } else {
                    carried += old.carried + 1;
                }
            }
        }
        self.queue.push_back(Item {
            topic,
            body,
            carried,
        });
    }
    pub fn lost_batch(&mut self, lost: u64) {
        self.trailing += lost;
    }
    pub fn front(&self) -> Option<(String, Value)> {
        let item = self.queue.front()?;
        let mut body = item.body.clone();
        let key = if body["dropped"].is_boolean() {
            "event_dropped"
        } else {
            "dropped"
        };
        body[key] = json!(self.lost + item.carried);
        Some((item.topic.clone(), body))
    }
    pub fn accepted(&mut self) {
        self.queue.pop_front();
        self.lost = 0;
    }
    pub fn failed(&mut self, body: &Value) {
        self.lost += 1 + body
            .get("event_dropped")
            .or_else(|| body.get("dropped"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn macro_loss_preserves_boolean_output_flag_and_action_order() {
        let mut queue = Events::default();
        queue.push("app.macros.finished".into(), json!({"dropped":false}), 0);
        let (_, body) = queue.front().unwrap();
        assert_eq!(body["event_dropped"], 0);
        assert_eq!(body["dropped"], false);
        queue.accepted();
        queue.failed(&body);
        queue.push("app.action.executed".into(), json!({"seq":2}), 0);
        assert_eq!(queue.front().unwrap().1["dropped"], 1);
        for i in 3..260 {
            queue.push("app.action.executed".into(), json!({"seq":i}), 0);
        }
        let (_, body) = queue.front().unwrap();
        assert_eq!(body["seq"], 4);
        assert_eq!(body["dropped"], 3);
    }
}
