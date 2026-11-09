// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
use slots::storage::token::base::{Token, TokenMut};
use uuid::Uuid;

/// Opaque, RENDERER-AGNOSTIC identity of a draw component. DrawOrder tracks only
/// this id + position; the owning system maps it back and skips foreign ids.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ComponentId(pub Uuid);

/// Fractional z-position within a layer; lower = further back. Gaps let a
/// component be restacked between two others without renumbering the rest.
pub type OrderKey = f64;

/// Coarse stacking tier. LOWER draws further back; within a tier `OrderKey`
/// decides. The ONE place layer affects z-order — draw pass and hit-test both
/// read it via `ordered()`, so they can never disagree.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct DrawLayer(pub i32);

impl DrawLayer {
    /// Default tier: windows + ordinary world UI (placeholders, launcher).
    pub const CONTENT: DrawLayer = DrawLayer(0);
    /// Group frames sit beneath the windows they contain.
    pub const GROUP: DrawLayer = DrawLayer(-100);
    /// World-space overlays ABOVE all windows (e.g. the selection toolbar).
    pub const OVERLAY: DrawLayer = DrawLayer(100);
}

/// The spatial world's NON-DESTRUCTIVE draw-order registry and single z-order
/// authority, superseding smithay `Space` stacking. Spawn appends to the tier
/// top; raise restacks; close removes.
#[derive(Default)]
pub struct DrawOrder {
    entries: Vec<(ComponentId, DrawLayer, OrderKey)>,
    next: OrderKey,
    z: std::collections::HashMap<ComponentId, i32>,
}

impl DrawOrder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next: 1.0,
            z: Default::default(),
        }
    }

    /// Insert `id` at the top of `layer` (re-insert re-raises AND updates tier).
    pub fn insert_top(&mut self, id: ComponentId, layer: DrawLayer) -> OrderKey {
        let key = self.next;
        self.next += 1.0;
        match self.entries.iter_mut().find(|(c, _, _)| *c == id) {
            Some(entry) => (entry.1, entry.2) = (layer, key),
            None => self.entries.push((id, layer, key)),
        }
        key
    }

    pub fn remove(&mut self, id: ComponentId) {
        self.entries.retain(|(c, _, _)| *c != id);
        self.z.remove(&id);
    }

    /// Teardown after Space unmap must resolve ownership from every stack.
    pub fn remove_from_all<'a>(id: ComponentId, orders: impl IntoIterator<Item = &'a mut Self>) {
        for order in orders {
            order.remove(id);
        }
    }

    /// Hand an existing entry's slot (tier + `OrderKey`) to a successor, keeping
    /// its z-position — a restored window appears WHERE THE PLACEHOLDER SAT,
    /// not popped to the top. Drops stale `new` first; `false` if `old` is absent.
    pub fn reassign(&mut self, old: ComponentId, new: ComponentId) -> bool {
        if old == new {
            return self.key(old).is_some();
        }
        self.entries.retain(|(c, _, _)| *c != new);
        let Some(entry) = self.entries.iter_mut().find(|(c, _, _)| *c == old) else {
            return false;
        };
        entry.0 = new;
        true
    }

    /// Move `id` above every component in its OWN tier. Lazily registers an
    /// unknown id at CONTENT (raise is a window/content op).
    pub fn raise(&mut self, id: ComponentId) {
        if self.entries.iter().any(|(c, _, _)| *c == id) {
            let key = self.next;
            self.next += 1.0;
            if let Some(entry) = self.entries.iter_mut().find(|(c, _, _)| *c == id) {
                entry.2 = key;
            }
        } else {
            self.insert_top(id, DrawLayer::CONTENT);
        }
    }

    /// Stacking request within the existing workspace tier; never changes tier.
    pub fn set_z(&mut self, id: ComponentId, z: i32) { self.z.insert(id, z); }

    pub fn key(&self, id: ComponentId) -> Option<OrderKey> {
        self.entries
            .iter()
            .find(|(c, _, _)| *c == id)
            .map(|(_, _, k)| *k)
    }

    /// Components back-to-front: by tier ascending, then `OrderKey`. Reverse for
    /// front-to-back input hit-testing.
    pub fn ordered(&self) -> Vec<(ComponentId, OrderKey)> {
        let mut v = self.entries.clone();
        v.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.total_cmp(&b.2)));
        // Only window slots move. Compositor-marked drawables keep their exact
        // place and remain barriers; z cannot promote a client across one.
        let mut start = 0;
        while start < v.len() {
            if !self.z.contains_key(&v[start].0) { start += 1; continue; }
            let mut end = start + 1;
            while end < v.len() && v[end].1 == v[start].1 && self.z.contains_key(&v[end].0) { end += 1; }
            v[start..end].sort_by(|a,b| self.z[&a.0].cmp(&self.z[&b.0]).then(a.2.total_cmp(&b.2)));
            start = end;
        }
        v.into_iter().map(|(c, _, k)| (c, k)).collect()
    }
}

pub static DRAW_ORDER: Token<DrawOrder> = Token::new();
pub static DRAW_ORDER_MUT: TokenMut<DrawOrder> = TokenMut::new(&DRAW_ORDER);

#[cfg(test)]
mod visual_tests {
    use super::*;

    #[test]
    fn z_reorders_windows_but_never_crosses_compositor_barriers_or_layers() {
        let ids = (1..=5).map(|n| ComponentId(Uuid::from_u128(n))).collect::<Vec<_>>();
        let mut order = DrawOrder::new();
        for id in &ids { order.insert_top(*id, DrawLayer::CONTENT); }
        // Third entry is compositor-marked: it is never registered as a window.
        for (id, z) in [(ids[0], 9), (ids[1], -9), (ids[3], i32::MAX), (ids[4], 0)] { order.set_z(id,z); }
        let ordered = || order.ordered().iter().map(|(id,_)| *id).collect::<Vec<_>>();
        assert_eq!(ordered(), vec![ids[1],ids[0],ids[2],ids[4],ids[3]]);
        order.insert_top(ids[0], DrawLayer(-50));
        assert_eq!(order.ordered()[0].0, ids[0], "z never promotes out of its layer");
        order.remove(ids[0]);
        assert!(!order.z.contains_key(&ids[0]));
    }
}
