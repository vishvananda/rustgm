//! Only declared signature formation grants implied lifetime bounds to an override.
use super::{Contract, Declarations};
use crate::{
    sema::projections::families,
    types::{
        outlives::{Implied, Pair},
        parameters::clauses::Clause,
        substitute::Request,
    },
};

#[derive(Default)]
pub(super) struct Formation {
    clauses: Vec<Clause>,
    regions: Vec<Pair>,
    implied: Implied,
    steps: usize,
}
impl Formation {
    pub fn record(
        &mut self,
        facts: &mut Declarations,
        request: Request,
    ) -> Result<(), &'static str> {
        let Some(application) = request.alias.associated() else {
            return Ok(());
        };
        let Some(required) = families::requirements(facts, application)? else {
            return Ok(());
        };
        self.steps = self.steps.saturating_add(required.steps);
        for clause in required.clauses {
            if let Some(predicate) =
                self.implied
                    .outlives(&mut facts.types, clause, request.depth)?
            {
                self.clauses.push(predicate.into());
            }
        }
        for pair in required.regions {
            if let Some(pair) = self.implied.regions(pair, request.depth)? {
                self.regions.push(pair);
            }
        }
        if self
            .steps
            .saturating_add(self.implied.steps())
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
