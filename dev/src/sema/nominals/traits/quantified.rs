//! Universal obligations retain private selection witnesses inside their own scope.
use super::{
    Assumptions, Availability, Context, Declarations, Obligations, State, select_requirement,
    selection,
};
use crate::types::{
    RegionId,
    lifetime_proof::ScopeId,
    parameters::clauses::{Clause, QuantifiedId},
    substitute::Bound,
};
use std::collections::HashSet;

#[derive(Default)]
pub(super) struct Queries {
    /// Every entry's pending lifetime evidence is already retained by this State.
    /// Caller environment and outer substitution are immutable for its lifetime.
    completed: HashSet<(QuantifiedId, bool, bool)>,
}

impl State {
    pub(super) fn prove_quantified(
        &mut self,
        facts: &mut Declarations,
        id: QuantifiedId,
        closed: bool,
    ) -> Result<bool, &'static str> {
        if self.retain_sized {
            return Err("quantified alias premises require scoped sized requirements");
        }
        let fold = self
            .fold
            .as_mut()
            .ok_or("quantified premises require an explicit caller scope")?;
        let before = fold.steps;
        let Clause::ForAll(id) = fold.clause(&mut facts.types, Clause::ForAll(id))? else {
            return Err("quantified substitution lost its binder");
        };
        self.context
            .account_work(fold.steps.saturating_sub(before).saturating_add(1))
            .map_err(|_| "quantified premise work exhausted")?;
        let key = (
            id,
            closed,
            matches!(self.availability, Availability::Validated),
        );
        if self
            .quantified
            .as_ref()
            .is_some_and(|queries| queries.completed.contains(&key))
        {
            return Ok(true);
        }
        let value = facts
            .types
            .quantified(id)
            .ok_or("missing quantified premise")?;
        let first = self.obligations.regions.end;
        let end = first
            .checked_add(value.bound_regions)
            .filter(|end| *end < 0x8000_0000)
            .ok_or("quantified premise region identity space exhausted")?;
        self.context
            .account_work(value.bound_regions as usize)
            .map_err(|_| "quantified premise work exhausted")?;
        let regions = (first..end).map(RegionId).collect::<Vec<_>>();
        let mut opening = Bound::new(regions);
        let requirement = opening.predicate(&mut facts.types, value.predicate)?;
        let mut context = Context::default();
        let empty = Assumptions::default();
        let selected = select_requirement(
            facts,
            &mut context,
            requirement,
            selection::Caller {
                assumptions: if closed { &empty } else { &self.assumptions },
                environment: if closed {
                    None
                } else {
                    self.environment.as_ref()
                },
                first_region: end,
                retain_sized: false,
            },
            self.availability,
            false,
        );
        self.context
            .account_work(opening.steps.saturating_add(context.steps))
            .map_err(|_| "quantified premise selection work exhausted")?;
        let selection::Selection::Proven(evidence) = selected? else {
            return Ok(false);
        };
        if !evidence.sized.is_empty() {
            return Err("quantified selection retained unscoped sized requirements");
        }
        let mut proof = Obligations::new(first);
        let scope = proof.scope(ScopeId::ROOT, value.bound_regions)?;
        let mut aliases = Obligations::new(first);
        aliases.regions.end = end;
        for (slot, region) in (first..end).enumerate() {
            let slot = u16::try_from(slot).map_err(|_| "too many quantified premise lifetimes")?;
            aliases
                .equalities
                .push((RegionId(region), RegionId::bound(0, slot)?));
        }
        proof.append_at(aliases, scope)?;
        proof.graft(evidence.proof, scope)?;
        let mut equations = Obligations::new(proof.regions.end);
        equations.equalities = context.region_equalities;
        proof.append_at(equations, scope)?;
        self.obligations.extend(proof)?;
        self.quantified
            .get_or_insert_with(Default::default)
            .completed
            .insert(key);
        Ok(true)
    }
}
