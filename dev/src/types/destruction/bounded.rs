//! Source-query budgets cover owned logical slots before registry or work growth.
use super::{Definitions, HashSet, ItemId, NeedsDrop, Scope, State, Store, TypeId, catalogue};
use crate::support::retained;

impl NeedsDrop {
    /// Qualification shares the ordinary scoped law, with prepaid sparse storage.
    pub(crate) fn scoped_bounded(
        custom: impl ExactSizeIterator<Item = ItemId>,
        environment: std::sync::Arc<super::Environment>,
        limit: usize,
    ) -> Result<Self, &'static str> {
        let mut query = Self::concrete_bounded(custom, limit)?;
        query
            .scope
            .as_deref_mut()
            .ok_or("missing scoped destruction query")?
            .environment = Some(environment);
        Ok(query)
    }
    /// Seed this fresh runtime-property cache only from completed scoped Copy decisions.
    pub(crate) fn seed_scoped_copy(
        &mut self,
        types: &Store,
        environment: &std::sync::Arc<super::Environment>,
        choices: &[(TypeId, bool)],
        limit: usize,
    ) -> Result<(), &'static str> {
        let scope = self
            .scope
            .as_deref()
            .ok_or("Copy qualification requires its scoped query")?;
        if scope
            .environment
            .as_ref()
            .is_none_or(|actual| !std::sync::Arc::ptr_eq(actual, environment))
            || !scope.states.is_empty()
            || !self.work.is_empty()
            || scope.builtins.is_some()
            || scope.builtin_bytes != 0
        {
            return Err("Copy qualification requires the same fresh scoped query");
        }
        let mut count = 0_usize;
        for &(ty, copy) in choices {
            self.charge::<true>()?;
            types
                .kind(ty)
                .ok_or("Copy qualification has an invalid subject")?;
            count = count.saturating_add(usize::from(copy));
        }
        self.retain::<true>(count, 0, limit)?;
        self.scope
            .as_mut()
            .ok_or("Copy qualification lost its scope")?
            .states
            .try_reserve(count)
            .map_err(|_| "Copy qualifier cache allocation failed")?;
        for &(ty, copy) in choices {
            if copy {
                self.record::<true>(ty, State::Ready(false), limit)?;
            }
        }
        Ok(())
    }
    pub(crate) fn demand_scoped_bounded(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
        limit: usize,
    ) -> Result<bool, &'static str> {
        if !self.symbolic() {
            return Err("scoped destruction requires its actual parameter environment");
        }
        self.require_paid_builtins()?;
        self.charge::<true>()?;
        self.retain::<true>(0, 0, limit)?;
        self.demand_inside::<true>(types, definitions, ty, limit)
    }
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
        self.require_paid_builtins()?;
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
        self.own_allocation(states, work)
            .saturating_add(self.scope.as_ref().map_or(0, |s| s.builtin_bytes))
    }
    fn own_allocation(&self, states: usize, work: usize) -> usize {
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
    fn require_paid_builtins(&self) -> Result<(), &'static str> {
        if self.scope.as_ref().is_some_and(|s| {
            s.builtins.is_some() && (s.builtin_bytes == 0 || s.builtin_bytes == usize::MAX)
        }) {
            return Err("bounded destruction cannot reuse an unpaid builtin query");
        }
        Ok(())
    }
    pub(super) fn assumed_bounded(
        &mut self,
        types: &Store,
        ty: TypeId,
        limit: usize,
    ) -> Result<Option<super::super::parameters::environment::Abilities>, &'static str> {
        use std::cell::Cell;
        self.require_paid_builtins()?;
        let own = self.own_allocation(
            self.scope.as_ref().map_or(0, |s| s.states.capacity()),
            self.work.capacity(),
        );
        let Some(scope) = &mut self.scope else {
            return Ok(None);
        };
        let Some(environment) = &scope.environment else {
            return Ok(None);
        };
        let steps = Cell::new(self.steps);
        let paid = Cell::new(scope.builtin_bytes);
        let charge = |work: usize, bytes: usize| {
            let next_steps = steps.get().saturating_add(work);
            steps.set(next_steps);
            if next_steps > 1_000_000 {
                return Err("destruction query work budget exceeded");
            }
            let next_paid = paid
                .get()
                .checked_add(bytes)
                .ok_or("destruction query storage budget exceeded")?;
            if own.saturating_add(next_paid) > limit {
                return Err("destruction query storage budget exceeded");
            }
            paid.set(next_paid);
            Ok(())
        };
        let result = environment.assumed_with_charged(types, ty, &mut scope.builtins, &charge);
        self.steps = steps.get();
        scope.builtin_bytes = paid.get();
        result.map(Some)
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
