//! Universal obligations retain private selection witnesses inside their own scope.
use super::{
    Assumptions, Availability, Context, Declarations, Obligations, State, select_requirement,
    selection,
};
use crate::types::{
    RegionId,
    lifetime_proof::ScopeId,
    parameters::{
        clauses::{Clause, Quantified, QuantifiedId},
        predicates::Predicate,
    },
    substitute::Bound,
};
use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

#[derive(Default)]
pub(super) struct Queries {
    /// Every entry's pending lifetime evidence is already retained by this State.
    /// Caller environment and outer substitution are immutable for its lifetime.
    completed: HashSet<(QuantifiedId, bool, bool)>,
    universes: HashMap<(u32, bool, bool), Universe>,
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
        let group = (value.bound_regions, key.1, key.2);
        let prepared = self
            .quantified
            .get_or_insert_with(Default::default)
            .prepare(
                facts,
                &mut self.context,
                group,
                value,
                self.obligations.regions.end,
            )?;
        let mut context = Context::default();
        let empty = Assumptions::default();
        let selected = select_requirement(
            facts,
            &mut context,
            prepared.requirement,
            selection::Caller {
                assumptions: if closed { &empty } else { &self.assumptions },
                environment: if closed {
                    None
                } else {
                    self.environment.as_ref()
                },
                first_region: prepared.next,
                retain_sized: false,
            },
            self.availability,
            false,
        );
        self.context
            .account_work(context.steps)
            .map_err(|_| "quantified premise selection work exhausted")?;
        let selection::Selection::Proven(evidence) = selected? else {
            return Ok(false);
        };
        if !evidence.sized.is_empty() {
            return Err("quantified selection retained unscoped sized requirements");
        }
        let scope = self
            .quantified
            .as_mut()
            .ok_or("missing quantified query group")?
            .commit(&mut self.obligations, group, prepared.destination)?;
        self.obligations.graft(evidence.proof, scope)?;
        let mut equations = Obligations::new(self.obligations.regions.end);
        equations.equalities = context.region_equalities;
        self.obligations.append_at(equations, scope)?;
        self.quantified
            .get_or_insert_with(Default::default)
            .completed
            .insert(key);
        Ok(true)
    }
}

// Conjunctive universal requirements of the same arity may use the same fresh
// placeholders. These scopes introduce requirements only, never local hypotheses.
// Every implementation selection still has isolated inference and fresh witnesses.
struct Universe {
    scope: ScopeId,
    opening: Box<Bound>,
}
enum Destination {
    Shared(ScopeId),
    Fresh {
        regions: Range<u32>,
        opening: Box<Bound>,
    },
}
struct Prepared {
    requirement: Predicate,
    destination: Destination,
    next: u32,
}
impl Queries {
    fn prepare(
        &mut self,
        facts: &mut Declarations,
        context: &mut Context,
        key: (u32, bool, bool),
        value: Quantified,
        first: u32,
    ) -> Result<Prepared, &'static str> {
        if let Some(universe) = self.universes.get_mut(&key) {
            let before = universe.opening.steps;
            let requirement = universe
                .opening
                .predicate(&mut facts.types, value.predicate);
            context
                .account_work(universe.opening.steps.saturating_sub(before))
                .map_err(|_| "quantified premise opening work exhausted")?;
            return Ok(Prepared {
                requirement: requirement?,
                destination: Destination::Shared(universe.scope),
                next: first,
            });
        }
        let end = first
            .checked_add(value.bound_regions)
            .filter(|end| *end < 0x8000_0000)
            .ok_or("quantified premise region identity space exhausted")?;
        context
            .account_work(value.bound_regions as usize)
            .map_err(|_| "quantified premise work exhausted")?;
        let mut opening = Box::new(Bound::new((first..end).map(RegionId).collect()));
        let requirement = opening.predicate(&mut facts.types, value.predicate);
        context
            .account_work(opening.steps)
            .map_err(|_| "quantified premise opening work exhausted")?;
        Ok(Prepared {
            requirement: requirement?,
            destination: Destination::Fresh {
                regions: first..end,
                opening,
            },
            next: end,
        })
    }

    fn commit(
        &mut self,
        proof: &mut Obligations,
        key: (u32, bool, bool),
        destination: Destination,
    ) -> Result<ScopeId, &'static str> {
        let (regions, opening) = match destination {
            Destination::Shared(scope) => return Ok(scope),
            Destination::Fresh { regions, opening } => (regions, opening),
        };
        let scope = proof.scope(ScopeId::ROOT, key.0)?;
        let mut aliases = Obligations::new(regions.start);
        aliases.regions.end = regions.end;
        for (slot, region) in regions.enumerate() {
            let slot = u16::try_from(slot).map_err(|_| "too many quantified premise lifetimes")?;
            aliases
                .equalities
                .push((RegionId(region), RegionId::bound(0, slot)?));
        }
        proof.append_at(aliases, scope)?;
        if self
            .universes
            .insert(key, Universe { scope, opening })
            .is_some()
        {
            return Err("quantified universe was committed twice");
        }
        Ok(scope)
    }
}
