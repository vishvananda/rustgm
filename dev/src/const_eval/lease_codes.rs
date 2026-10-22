//! Cold conditional indexes remain unusable after their actual owner retires.
use crate::support::hash::HashMap;
use crate::{sema::inline_constants::InlineInstance, types::TypeId};
type Key = (InlineInstance, TypeId);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Scope(u64);
#[derive(Default)]
pub(super) struct Scoped {
    next: u64,
    rows: Vec<Row>,
    codes: HashMap<usize, Scope>,
    lookup: HashMap<(Scope, Key), usize>,
}
struct Row {
    scope: Scope,
    keys: Vec<Key>,
}

impl Scoped {
    pub(super) fn enter(&mut self, bytes: &mut usize) -> Result<Scope, &'static str> {
        let retained = bytes.saturating_add(4 * size_of::<Row>());
        if retained > 64 * 1024 * 1024 || self.rows.len() >= 128 {
            return Err("conditional code scopes exceed storage budget");
        }
        self.next = self
            .next
            .checked_add(1)
            .ok_or("conditional code scope identity exhausted")?;
        let scope = Scope(self.next);
        self.rows.push(Row {
            scope,
            keys: Vec::new(),
        });
        *bytes = retained;
        Ok(scope)
    }
    pub(super) fn lookup(&self, key: Key) -> Option<&usize> {
        self.lookup.get(&(self.rows.last()?.scope, key))
    }
    pub(super) fn current(&self, scope: Scope) -> bool {
        self.rows.last().is_some_and(|row| row.scope == scope)
    }
    pub(super) fn validate(&self, index: usize) -> Result<(), &'static str> {
        if self
            .codes
            .get(&index)
            .is_some_and(|scope| !self.current(*scope))
        {
            return Err("conditional code requires its exact live parent scope");
        }
        Ok(())
    }
    pub(super) fn insert(
        &mut self,
        scope: Scope,
        key: Key,
        index: usize,
        bytes: &mut usize,
    ) -> Result<(), &'static str> {
        if !self.current(scope) || self.lookup.contains_key(&(scope, key)) {
            return Err("conditional code belongs to another or completed scope");
        }
        let retained = bytes
            .saturating_add(crate::support::retained::new_map::<usize, Scope>(1))
            .saturating_add(crate::support::retained::new_map::<(Scope, Key), usize>(1))
            .saturating_add(4 * size_of::<Key>());
        if retained > 64 * 1024 * 1024 {
            return Err("conditional code index exceeds storage budget");
        }
        let row = self
            .rows
            .last_mut()
            .ok_or("conditional code scope disappeared")?;
        row.keys.push(key);
        self.lookup.insert((scope, key), index);
        self.codes.insert(index, scope);
        *bytes = retained;
        Ok(())
    }
    pub(super) fn retire(&mut self, scope: Scope) -> Result<(), &'static str> {
        if !self.current(scope) {
            return Err("conditional code retirement has another owner");
        }
        let row = self
            .rows
            .pop()
            .ok_or("conditional code scope disappeared")?;
        for key in row.keys {
            self.lookup.remove(&(scope, key));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/lease_indexes.rs"]
mod tests;
