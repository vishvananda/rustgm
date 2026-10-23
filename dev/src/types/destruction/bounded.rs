//! Source-query budgets cover owned logical slots before registry or work growth.
use super::{Definitions, HashSet, ItemId, NeedsDrop, Scope, State, Store, TypeId, catalogue};
use crate::support::retained;

impl NeedsDrop {
    /// Preflight the cold query and immutable registry before consuming its entries.
    #[allow(dead_code)] // The source needs_drop consumer awaits validated source roles.
    pub(crate) fn concrete_bounded(
        custom: impl ExactSizeIterator<Item = ItemId>,
        limit: usize,
    ) -> Result<Self, &'static str> {
        let count = custom.len();
        let bytes = retained::sum([
            size_of::<Self>(),
            size_of::<Scope>(),
            size_of::<HashSet<ItemId>>(),
            retained::new_map::<ItemId, ()>(count),
        ]);
        let steps = count
            .checked_add(1)
            .filter(|n| *n <= 1_000_000)
            .ok_or("destruction query work budget exceeded")?;
        if bytes > limit {
            return Err("destruction query storage budget exceeded");
        }
        let mut query = Self::concrete_sparse(custom);
        query.steps = steps;
        Ok(query)
    }
    /// Exact source kernels charge each attempt and retain no symbolic permissions.
    #[allow(dead_code)] // The source needs_drop consumer awaits validated source roles.
    pub(crate) fn demand_concrete_bounded(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
        limit: usize,
    ) -> Result<bool, &'static str> {
        if !self.exact_heads() {
            return Err("bounded destruction requires an exact sparse query");
        }
        self.require_catalogue_environment(types, definitions)?;
        self.charge::<true>()?;
        self.retain::<true>(0, 0, limit)?;
        self.demand_inside::<true>(types, definitions, ty, limit)
    }
    /// Logical owned records and capacity; shared registry payload is charged once.
    #[allow(dead_code)] // The source needs_drop consumer awaits validated source roles.
    pub(crate) fn retained_bytes(&self) -> usize {
        self.allocation(
            self.scope
                .as_ref()
                .map_or(0, |scope| scope.states.capacity()),
            self.work.capacity(),
        )
    }
    fn allocation(&self, states: usize, work: usize) -> usize {
        retained::sum([
            size_of::<Self>(),
            self.scope.as_ref().map_or(0, |_| size_of::<Scope>()),
            self.scope
                .as_ref()
                .and_then(|s| s.binding.as_ref())
                .map_or(0, |_| catalogue::Binding::allocation_bytes()),
            size_of::<HashSet<ItemId>>(),
            self.custom.capacity().saturating_mul(size_of::<ItemId>()),
            states.saturating_mul(size_of::<(TypeId, State)>()),
            work.saturating_mul(size_of::<(TypeId, bool)>()),
            retained::vector(&self.states),
        ])
    }
    pub(super) fn retain<const BOUNDED: bool>(
        &self,
        states: usize,
        work: usize,
        limit: usize,
    ) -> Result<(), &'static str> {
        if !BOUNDED {
            return Ok(());
        }
        let scope = self
            .scope
            .as_deref()
            .ok_or("missing exact destruction scope")?;
        let state_capacity = if scope.states.len().saturating_add(states) > scope.states.capacity()
        {
            // Both old and replacement payloads can be live during table growth.
            scope
                .states
                .capacity()
                .saturating_add(scope.states.len().saturating_add(states).saturating_mul(4))
        } else {
            scope.states.capacity()
        };
        let work_capacity = if self.work.len().saturating_add(work) > self.work.capacity() {
            self.work.capacity().saturating_add(
                self.work
                    .capacity()
                    .saturating_mul(2)
                    .max(4)
                    .max(self.work.len().saturating_add(work)),
            )
        } else {
            self.work.capacity()
        };
        if self.allocation(state_capacity, work_capacity) > limit {
            return Err("destruction query storage budget exceeded");
        }
        Ok(())
    }
    pub(super) fn push<const BOUNDED: bool>(
        &mut self,
        value: (TypeId, bool),
        limit: usize,
    ) -> Result<(), &'static str> {
        self.retain::<BOUNDED>(0, 1, limit)?;
        self.work.push(value);
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/concrete_destruction/budgets.rs"]
mod tests;
