//! Only declared signature formation grants implied lifetime bounds to an override.
use super::{Contract, Declarations};
use crate::{
    sema::projections::families,
    types::{
        bound_uses::captures::Captures,
        outlives::Pair,
        parameters::clauses::Clause,
        substitute::{Bound, Request},
    },
};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Formation {
    clauses: Vec<Clause>,
    regions: Vec<Pair>,
    scopes: HashMap<u16, Scope>,
    steps: usize,
}
struct Scope {
    captures: Captures,
    lower: Bound,
}
impl Formation {
    pub fn record(
        &mut self,
        facts: &mut Declarations,
        request: Request,
    ) -> Result<(), &'static str> {
        let Some(required) = families::requirements(facts, request.application)? else {
            return Ok(());
        };
        self.steps = self.steps.saturating_add(required.steps);
        let mut scope = (request.depth != 0).then(|| {
            self.scopes.entry(request.depth).or_insert_with(|| Scope {
                captures: Captures::new(request.depth),
                lower: Bound::lower(request.depth),
            })
        });
        let before = scope
            .as_ref()
            .map_or(0, |s| s.captures.steps.saturating_add(s.lower.steps));
        for clause in required.clauses {
            if !clause.is_outlives(&facts.types)? {
                continue;
            }
            let clause = if let Some(scope) = &mut scope {
                // A surrounding callable's local lifetimes are not universally
                // true caller assumptions. Only captured conditions can escape.
                if scope.captures.clause(&facts.types, clause)? {
                    continue;
                }
                scope.lower.clause(&mut facts.types, clause)?
            } else {
                clause
            };
            self.clauses.push(clause);
        }
        for (a, b) in required.regions {
            let pair = if let Some(scope) = &mut scope {
                if [a, b].iter().any(|r| {
                    r.bound_parts()
                        .is_some_and(|(depth, _)| depth < request.depth)
                }) {
                    continue;
                }
                (scope.lower.region(a, 0)?, scope.lower.region(b, 0)?)
            } else {
                (a, b)
            };
            self.regions.push(pair);
        }
        let after = scope
            .as_ref()
            .map_or(0, |s| s.captures.steps.saturating_add(s.lower.steps));
        self.steps = self.steps.saturating_add(after.saturating_sub(before));
        if self
            .steps
            .saturating_add(self.clauses.len())
            .saturating_add(self.regions.len())
            > 1_000_000
        {
            return Err("signature formation implication work budget exceeded");
        }
        Ok(())
    }

    pub fn extend(self, contract: &mut Contract) {
        contract.assumptions.extend(self.clauses);
        contract.assumed_regions.extend(self.regions);
    }
}
