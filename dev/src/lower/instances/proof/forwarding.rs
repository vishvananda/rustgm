//! Only proved adapters and factories may forward or omit a source caller scope.
use super::{Builder, borrow};
use crate::{
    body::ir::{Body, BodyOrigin},
    sema::model::Declarations,
    support::retained::Growth,
    types::Store,
};

pub(super) use crate::borrow::incremental::Bridge;
impl Builder {
    pub(super) fn prepay_forwarding(
        &self,
        facts: &mut Declarations,
        budget: &mut usize,
        source: BodyOrigin,
    ) -> Result<(), borrow::Error> {
        let growth =
            Growth::map::<usize, Bridge>(self.forwarding.len(), self.forwarding.capacity(), 1);
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(growth.work.saturating_add(3));
        let handoff = if self.forwarding.is_empty() {
            size_of::<crate::borrow::incremental::Forwarding>()
                + size_of::<Option<&crate::borrow::incremental::Forwarding>>()
        } else {
            0
        };
        *budget = budget.saturating_add(growth.bytes).saturating_add(handoff);
        if *budget > 64_000_000 || self.forwarding.contains_key(&self.bodies.len()) {
            return Err(borrow::Error {
                node: Some(source.into()),
                message: "caller forwarding exceeds its actual instance budget or repeats a body"
                    .into(),
            });
        }
        Ok(())
    }
    pub(in crate::lower::instances) fn caller_body(
        &self,
        types: &Store,
        index: usize,
    ) -> Result<Option<&Body>, &'static str> {
        match self.forwarding.get(&index) {
            None => self.body(index).map(Some),
            Some(bridge) => bridge.caller(types, index, |index| self.body(index)),
        }
    }
}
