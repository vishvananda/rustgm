//! Implied bounds exclude whole predicates mentioning quantified or callable locals.
use super::{Budget, Pair, Store, free};
use crate::support::hash::HashMap;
use crate::types::{
    bound_uses::captures::Captures,
    parameters::{
        clauses::{Clause, QuantifiedId},
        predicates::{Bound, Predicate},
    },
    substitute,
};
#[cfg(test)]
#[path = "../../../tests/invariants/implied_clauses.rs"]
mod tests;

/// Formation may imply only complete lifetime clauses independent of local binders.
/// This structural collector grants no permission until its caller publishes it.
#[derive(Default)]
pub(crate) struct Implied {
    declarations: Declarations,
    scopes: HashMap<u16, Scope>,
    budget: Budget,
}
struct Scope {
    captures: Captures,
    lowering: substitute::Bound,
}
impl Scope {
    fn new(depth: u16) -> Self {
        Self {
            captures: Captures::new(depth),
            lowering: substitute::Bound::lower(depth),
        }
    }

    fn lift(
        &mut self,
        types: &mut Store,
        predicate: Predicate,
        budget: &mut Budget,
    ) -> Result<Option<Predicate>, &'static str> {
        let before = self.captures.steps;
        let captured = self.captures.predicate(types, predicate);
        budget.work(self.captures.steps.saturating_sub(before))?;
        if captured? {
            // Filter the whole predicate before decomposing its subject. An
            // independent component cannot escape a dependent requirement.
            return Ok(None);
        }
        let before = self.lowering.steps;
        let result = self.lowering.predicate(types, predicate);
        budget.work(self.lowering.steps.saturating_sub(before))?;
        result.map(Some)
    }
}
impl Implied {
    pub(crate) fn outlives(
        &mut self,
        types: &mut Store,
        clause: Clause,
        depth: u16,
    ) -> Result<Option<Predicate>, &'static str> {
        self.budget.charge()?;
        let Some(predicate) = self
            .declarations
            .outlives(types, clause, &mut self.budget)?
        else {
            return Ok(None);
        };
        if depth == 0 {
            return Ok(Some(predicate));
        }
        self.scopes
            .entry(depth)
            .or_insert_with(|| Scope::new(depth))
            .lift(types, predicate, &mut self.budget)
    }

    pub(crate) fn regions(&mut self, pair: Pair, depth: u16) -> Result<Option<Pair>, &'static str> {
        self.budget.charge()?;
        Ok(free(pair.0, depth)?.zip(free(pair.1, depth)?))
    }

    #[must_use]
    pub(crate) const fn steps(&self) -> usize {
        self.budget.0
    }
}

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
            Clause::UnstableFeature(_) => return Ok(None),
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
    scope: Scope,
}
impl Default for Scoped {
    fn default() -> Self {
        Self {
            completed: HashMap::default(),
            scope: Scope::new(1),
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
        // Unused and empty quantifiers still add a structural binder. Remove
        // it without capturing the locals of any nested callable.
        let result = self.scope.lift(types, predicate, budget)?;
        self.completed.insert(id, result);
        Ok(result)
    }
}
