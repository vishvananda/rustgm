//! A normalized result seal is minted only while retaining its caller's real debts.
use super::{Declarations, Instance, SourceComputation, TypeId};
use crate::sema::dispatch::Dispatcher;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Seal {
    selected: Instance,
    actual: TypeId,
    requested: TypeId,
    paid: Option<super::paid_inputs::Paid>,
}
impl Seal {
    pub(super) const fn result(self) -> TypeId {
        self.requested
    }
    pub(super) fn prepare(
        facts: &mut Declarations,
        dispatcher: &mut Dispatcher,
        selected: Instance,
        requested: TypeId,
    ) -> Result<Self, String> {
        let original = facts
            .initializer_index(selected.definition)
            .and_then(|n| facts.initializers.get(n))
            .ok_or("missing selected initializer result")?
            .semantic_source();
        let declared = SourceComputation::shape(facts, selected, original)?;
        let mut fold = selected.substitution(facts)?;
        let actual = fold.apply(&mut facts.types, declared)?;
        facts.body_work.canonical_steps += fold.steps;
        let normalized = dispatcher.normalize_override(facts, actual)?;
        let requested = dispatcher.normalize_override(facts, requested)?;
        if normalized != requested || !super::source_result(&facts.types, requested) {
            return Err(
                "source initializer result disagrees with the normalized caller request".into(),
            );
        }
        Ok(Self {
            selected,
            actual,
            requested,
            paid: None,
        })
    }
    pub(super) fn validate(
        self,
        facts: &mut Declarations,
        selected: Instance,
        requested: TypeId,
    ) -> Result<(), &'static str> {
        if self.selected != selected
            || self.requested != requested
            || !super::source_result(&facts.types, requested)
        {
            return Err("source result seal belongs to another instance or result");
        }
        let declared = facts
            .definitions
            .get(selected.definition.0 as usize)
            .copied()
            .flatten()
            .ok_or("source result seal lost its declaration")?;
        let mut fold = selected.substitution(facts)?;
        let actual = fold.apply(&mut facts.types, declared)?;
        facts.body_work.canonical_steps += fold.steps;
        if actual != self.actual {
            return Err("source result seal substituted declaration changed");
        }
        Ok(())
    }
    pub(super) fn with_paid(
        mut self,
        paid: Option<super::paid_inputs::Paid>,
    ) -> Result<Self, &'static str> {
        if paid.is_some_and(|paid| !paid.selects(self.selected)) {
            return Err("paid source premise belongs to another selected instance");
        }
        self.paid = paid;
        Ok(self)
    }
    pub(super) fn validate_paid(
        self,
        facts: &Declarations,
        caller: super::catalog::CallerId,
        key: super::catalog::CallerKey,
    ) -> Result<bool, &'static str> {
        let Some(paid) = self.paid else {
            return Ok(false);
        };
        paid.validate(facts, caller, key, self.selected)?;
        Ok(true)
    }
    pub(super) fn projected(self, facts: &Declarations) -> bool {
        facts.types.contains_projections(self.actual) == Some(true)
    }
}
