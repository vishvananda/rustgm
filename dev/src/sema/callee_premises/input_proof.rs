//! Actual caller input assumptions stay separate from every selected body debt.
use crate::{
    resolve::unit::NodeKey,
    sema::model::Declarations,
    types::{
        Kind, RegionId, TypeId,
        lifetime_proof::Obligations,
        outlives::{Pair, Queries},
        parameters::{ParameterId, clauses::Clause},
        relations::Constraints,
        substitute::RegionEquations,
    },
};
use std::collections::HashSet;
const LIMIT: usize = 1_000_000;
const BYTES: usize = 64 * 1024 * 1024;

/// A borrow of an actual body header, not proof authority. Construction and
/// caller/source matching remain the responsibility of its typed owning path.
#[derive(Clone, Copy)]
pub(crate) struct Inputs<'a> {
    pub source: NodeKey,
    pub universals: u32,
    pub signature: Option<TypeId>,
    pub parameters: &'a [ParameterId],
    pub assumptions: &'a [Pair],
    pub type_assumptions: &'a [Clause],
    pub require_result: bool,
}
impl Inputs<'_> {
    pub(crate) fn prepare(self, facts: &mut Declarations) -> Result<(), &'static str> {
        if let Some(signature) = self.signature {
            Self::prepare_type(facts, signature)?;
        }
        Ok(())
    }
    pub(crate) fn prepare_type(facts: &mut Declarations, ty: TypeId) -> Result<(), &'static str> {
        let before = facts.lifetimes.nominal_requirements.preparation_steps();
        let result = facts
            .lifetimes
            .nominal_requirements
            .prepare(&mut facts.types, ty);
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(1)
            .saturating_add(
                facts
                    .lifetimes
                    .nominal_requirements
                    .preparation_steps()
                    .saturating_sub(before),
            );
        result
    }
    fn seed<'a>(
        self,
        facts: &'a Declarations,
        queries: &mut Queries<'a>,
    ) -> Result<Constraints<'a>, &'static str> {
        let mut proof = Constraints::new(
            &facts.types,
            &facts.lifetimes.variances,
            self.universals,
            LIMIT,
        )
        .map_err(|_| "source caller input graph exhausted")?;
        for &parameter in self.parameters {
            proof
                .parameter(parameter)
                .map_err(|_| "invalid source caller input parameter")?;
        }
        for &(longer, shorter) in self.assumptions {
            proof
                .assume(longer, shorter)
                .map_err(|_| "invalid source caller input bound")?;
        }
        if let Some(signature) = self.signature {
            if self.require_result {
                let Some(Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
                    return Err("source caller has no input signature");
                };
                for &ty in facts
                    .types
                    .list(parameters)
                    .ok_or("missing source caller input types")?
                {
                    proof
                        .assume_well_formed(queries, ty)
                        .map_err(|_| "invalid source caller implied input")?;
                }
            } else {
                proof
                    .assume_signature(queries, signature)
                    .map_err(|_| "invalid source caller input signature")?;
            }
        }
        proof
            .assume_outlives_clauses(queries, self.type_assumptions.iter().copied())
            .map_err(|_| "invalid source caller declared type premise")?;
        Ok(proof)
    }
    /// Require original alias equations once before the identity fold can be
    /// used to justify any declared-premise specialization. No body outlives,
    /// coercion or selected-initializer requirement becomes an assumption.
    pub(crate) fn aliases(
        self,
        facts: &Declarations,
        equalities: impl IntoIterator<Item = Pair>,
        work: &mut usize,
    ) -> Result<RegionEquations, &'static str> {
        let mut queries = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        *work = work
            .saturating_add(self.parameters.len())
            .saturating_add(self.assumptions.len())
            .saturating_add(self.type_assumptions.len())
            .saturating_add(1);
        let seeded = self.seed(facts, &mut queries);
        let seeded_steps = queries.steps();
        *work = work.saturating_add(seeded_steps);
        let mut constraints = seeded?;
        let mut variables = HashSet::new();
        let mut equations = RegionEquations::default();
        let result = (|| {
            for (left, right) in equalities {
                *work = work.saturating_add(1);
                if *work > LIMIT {
                    return Err("source caller alias work exhausted");
                }
                for id in [left, right] {
                    if id == RegionId::STATIC || id.bound_parts().is_some() {
                        continue;
                    }
                    if facts
                        .body_regions
                        .get(id.0 as usize - 1)
                        .is_none_or(|region| region.body != self.source)
                    {
                        return Err("source caller alias has a foreign body identity");
                    }
                    if !variables.contains(&id) {
                        if crate::support::retained::new_map::<RegionId, ()>(
                            variables.len().saturating_add(1),
                        ) > BYTES
                        {
                            return Err("source caller alias storage exhausted");
                        }
                        variables.insert(id);
                        constraints
                            .variable(id)
                            .map_err(|_| "invalid source caller alias variable")?;
                    }
                }
                constraints
                    .outlives(left, right, 0)
                    .map_err(|_| "invalid source caller alias equation")?;
                constraints
                    .outlives(right, left, 0)
                    .map_err(|_| "invalid source caller alias equation")?;
                equations.extend_counted([(left, right)], work)?;
            }
            if let Ok(steps) = constraints.check_aliases() {
                *work = work.saturating_add(steps);
                Ok(())
            } else {
                // The consuming solver exposes exact successful work only.
                // Charge its full bounded allowance on failure rather than
                // lose rejected work; this is a conservative logical charge.
                *work = work.saturating_add(LIMIT);
                Err("source caller inputs do not prove its lifetime alias equations")
            }
        })();
        *work = work.saturating_add(queries.steps().saturating_sub(seeded_steps));
        result?;
        Ok(equations)
    }
    /// Check one exact pre-append selection and its raw declared-premise proof.
    /// The fixed opening is required, never assumed from the existence of a map.
    pub(crate) fn selection(
        self,
        facts: &Declarations,
        opening: &Obligations,
        selected: &Obligations,
        premises: Option<&Obligations>,
        work: &mut usize,
    ) -> Result<(), &'static str> {
        let mut queries = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        *work = work
            .saturating_add(self.parameters.len())
            .saturating_add(self.assumptions.len())
            .saturating_add(self.type_assumptions.len())
            .saturating_add(1);
        let seeded = self.seed(facts, &mut queries);
        let seeded_steps = queries.steps();
        *work = work.saturating_add(seeded_steps);
        let mut constraints = seeded?;
        let result = (|| {
            for proof in [Some(opening), Some(selected), premises]
                .into_iter()
                .flatten()
            {
                *work = work.saturating_add(proof.retained_allocation().visits);
                proof
                    .require(&mut constraints, &mut queries)
                    .map_err(|_| "invalid source caller selected premise proof")?;
            }
            if let Ok(solution) = constraints.solve() {
                *work = work.saturating_add(solution.steps);
                Ok(())
            } else {
                *work = work.saturating_add(LIMIT);
                Err("source initializer declared lifetime premise is not paid by caller inputs")
            }
        })();
        *work = work.saturating_add(queries.steps().saturating_sub(seeded_steps));
        result
    }
}
