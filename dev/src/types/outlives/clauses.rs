//! Implied nominal bounds exclude whole predicates mentioning their own quantifier.
use super::{Budget, Store};
use crate::types::{
    bound_uses::captures::Captures,
    parameters::{
        clauses::{Clause, QuantifiedId},
        predicates::{Bound, Predicate},
    },
    substitute,
};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Declarations {
    scoped: Option<Box<Scoped>>,
}
impl Declarations {
    pub fn outlives(
        &mut self,
        types: &mut Store,
        clause: Clause,
        budget: &mut Budget,
    ) -> Result<Option<Predicate>, &'static str> {
        let id = match clause {
            Clause::Predicate(predicate) => {
                return Ok(matches!(predicate.bound, Bound::Outlives(_)).then_some(predicate));
            }
            Clause::ForAll(id) => id,
        };
        let value = types
            .quantified(id)
            .ok_or("invalid nominal outlives clause")?;
        if !matches!(value.predicate.bound, Bound::Outlives(_)) {
            return Ok(None);
        }
        self.scoped
            .get_or_insert_with(Default::default)
            .lift(types, id, value.predicate, budget)
    }
}

struct Scoped {
    completed: HashMap<QuantifiedId, Option<Predicate>>,
    captures: Captures,
    lowering: substitute::Bound,
}
impl Default for Scoped {
    fn default() -> Self {
        Self {
            completed: HashMap::new(),
            captures: Captures::new(1),
            lowering: substitute::Bound::lower(1),
        }
    }
}
impl Scoped {
    fn lift(
        &mut self,
        types: &mut Store,
        id: QuantifiedId,
        predicate: Predicate,
        budget: &mut Budget,
    ) -> Result<Option<Predicate>, &'static str> {
        if let Some(&result) = self.completed.get(&id) {
            return Ok(result);
        }
        let before = self.captures.steps;
        let captured = self.captures.predicate(types, predicate);
        budget.work(self.captures.steps.saturating_sub(before))?;
        let result = if captured? {
            // Filter before decomposing the subject. Independent components of a
            // dependent nominal contract are not implied caller assumptions.
            None
        } else {
            // Unused and empty quantifiers still add a structural binder. Remove
            // it without capturing the locals of any nested callable.
            let before = self.lowering.steps;
            let lifted = self.lowering.predicate(types, predicate);
            budget.work(self.lowering.steps.saturating_sub(before))?;
            Some(lifted?)
        };
        self.completed.insert(id, result);
        Ok(result)
    }
}
