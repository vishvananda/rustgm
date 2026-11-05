//! A future application owes its actual function inputs, independently of body assumptions.
use crate::{
    resolve::model::DefId,
    sema::{declaration_inputs, model::Declarations, nominals::traits::State},
    types::{TypeId, lifetime_proof::Obligations, parameters::clauses::Clause, substitute::Bound},
};

impl Declarations {
    /// Return a private proof unit, never insert these debts under callee assumptions.
    pub(crate) fn authored_future_requirements(
        &mut self,
        future: TypeId,
        fold: &mut Bound,
        universals: u32,
    ) -> Result<Obligations, &'static str> {
        let owner = self.checked_future_definition(future)?;
        let clauses = inputs(self, owner)?;
        let count = self
            .lifetimes
            .functions
            .get(&owner)
            .ok_or("future formation lost its actual function lifetime schema")?
            .bounds
            .len();
        crate::sema::header_attributes::reserve_source_contracts(
            self,
            clauses.len().saturating_add(count),
            count.saturating_mul(size_of::<crate::types::outlives::Pair>()),
        )
        .map_err(|_| "future formation source storage budget exhausted")?;
        let bounds = self
            .lifetimes
            .functions
            .get(&owner)
            .ok_or("future formation lost its actual function lifetime schema")?
            .bounds
            .clone();
        let mut state = if clauses.is_empty() {
            None
        } else {
            Some(State::instance(self, universals)?)
        };
        let before = fold.steps;
        let result = (|| {
            for clause in clauses {
                let clause = fold.clause(&mut self.types, clause)?;
                let state = state
                    .as_mut()
                    .ok_or("future formation lost its canonical selection")?;
                let before = state.selection_steps();
                let proven = state.prove_clause(self, clause, true);
                self.body_work.canonical_steps = self
                    .body_work
                    .canonical_steps
                    .saturating_add(state.selection_steps().saturating_sub(before));
                if !proven? {
                    return Err(
                        "future application does not satisfy its actual defining input clause",
                    );
                }
            }
            let mut proof = state.map_or_else(|| Obligations::new(1), State::finish);
            for (longer, shorter) in bounds {
                let longer = fold.region(longer, 0)?;
                let shorter = fold.region(shorter, 0)?;
                if longer != shorter && longer != crate::types::RegionId::STATIC {
                    proof.outlives.push((longer, shorter));
                }
            }
            Ok(proof)
        })();
        self.body_work.canonical_steps = self
            .body_work
            .canonical_steps
            .saturating_add(fold.steps.saturating_sub(before));
        result
    }
}

fn inputs(facts: &mut Declarations, owner: DefId) -> Result<Vec<Clause>, &'static str> {
    let count =
        declaration_inputs::declared(&facts.generics, facts.crates.as_deref(), owner)?.count();
    crate::sema::header_attributes::reserve_source_contracts(
        facts,
        count,
        count.saturating_mul(size_of::<Clause>()),
    )
    .map_err(|_| "future formation source input budget exhausted")?;
    Ok(
        declaration_inputs::declared(&facts.generics, facts.crates.as_deref(), owner)?
            .map(|input| input.value)
            .collect(),
    )
}
