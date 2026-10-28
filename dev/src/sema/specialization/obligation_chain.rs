//! Member-specific projection proof uses the existing counted suspension engine.
use super::{SelectedMethod, header_conditions::Conditions};
use crate::{
    sema::{
        model::Declarations,
        traits::{implementations::Availability, obligations::Obligations, selection},
    },
    support::retained::Growth,
    types::{
        infer::{Context, Instantiation, predicates::Predicate as Goal},
        parameters::clauses::Clause,
    },
};

pub(in crate::sema) struct ProjectionRequirements {
    method: SelectedMethod,
    conditions: Conditions,
    cursor: usize,
    active: Option<Active>,
}
struct Active {
    goal: Goal,
    bound_regions: Option<u32>,
    pending: Option<Box<selection::Pending>>,
    work: usize,
    paid: usize,
}
pub(in crate::sema) struct ProofInputs<'a> {
    pub context: &'a mut Context,
    pub proof: &'a mut Obligations,
    pub caller: selection::OwnedCaller,
    pub availability: Availability,
    pub frontier: u32,
}
impl ProjectionRequirements {
    pub(in crate::sema) fn new(
        facts: &mut Declarations,
        method: SelectedMethod,
    ) -> Result<Option<Self>, &'static str> {
        let Some(conditions) = method.projection_conditions(facts)? else {
            return Ok(None);
        };
        method.charge(1, size_of::<Self>())?;
        Ok(Some(Self {
            method,
            conditions,
            cursor: 0,
            active: None,
        }))
    }
    pub(in crate::sema) fn allocation_end(&self, first: u32) -> u32 {
        self.active
            .as_ref()
            .and_then(|active| active.pending.as_ref())
            .map_or(first, |pending| first.max(pending.allocation_end()))
    }
    pub(in crate::sema) fn charge(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        self.method.charge(steps, bytes)
    }
    #[cfg(test)]
    pub(in crate::sema) fn storage_capacities(&self) -> (usize, usize) {
        (
            self.conditions.clauses.capacity(),
            self.conditions.outlives.capacity(),
        )
    }
    pub(in crate::sema) fn advance(
        &mut self,
        facts: &mut Declarations,
        inputs: &mut ProofInputs<'_>,
    ) -> Result<Option<selection::Dependency>, String> {
        self.method.current(facts)?;
        while let Some(&clause) = self.conditions.clauses.get(self.cursor) {
            self.method.charge(1, 0)?;
            if self.active.is_none() {
                self.active = Some(Active::new(facts, inputs.context, clause, &self.method)?);
            }
            let active = self
                .active
                .as_mut()
                .ok_or("missing defining condition query")?;
            match active.advance(facts, inputs, &self.method)? {
                selection::Progress::Pending(dependency, pending) => {
                    active.pending = Some(pending);
                    return Ok(Some(dependency));
                }
                selection::Progress::Ready(selection::Selection::Proven(evidence)) => {
                    if evidence.provisional_size || !evidence.sized.is_empty() {
                        return Err(
                            "inherited result requires finalized formation contracts".into()
                        );
                    }
                    let allocation = evidence.proof.retained_allocation();
                    let growth = join_growth(inputs.proof, &evidence.proof);
                    self.method.charge(
                        allocation.visits.saturating_add(growth.work),
                        allocation.bytes.saturating_add(growth.bytes),
                    )?;
                    inputs.proof.extend(evidence.proof)?;
                    self.active = None;
                    self.cursor += 1;
                }
                selection::Progress::Ready(selection::Selection::Refuted(refutation)) => {
                    refutation.reject()?;
                    return Err("inherited method defining-header condition is refuted".into());
                }
                selection::Progress::Ready(
                    selection::Selection::Absent | selection::Selection::Deferred,
                ) => return Err("inherited method defining-header condition is not proven".into()),
                selection::Progress::Inferred { .. } => {
                    return Err("inherited result proof cannot refine the selected member".into());
                }
            }
        }
        let regions = &mut self.conditions.outlives;
        let growth = Growth::vector::<crate::types::outlives::Pair>(
            inputs.proof.outlives.len(),
            inputs.proof.outlives.capacity(),
            regions.len(),
        );
        self.method
            .charge(regions.len().saturating_add(growth.work), growth.bytes)?;
        inputs.proof.outlives.append(regions);
        Ok(None)
    }
}
impl Active {
    fn new(
        facts: &mut Declarations,
        context: &mut Context,
        clause: Clause,
        method: &SelectedMethod,
    ) -> Result<Self, String> {
        let (predicate, bound_regions) = match clause {
            Clause::UnstableFeature(_) => {
                return Err(
                    "specialization feature condition requires its non-type caller proof".into(),
                );
            }
            Clause::Predicate(predicate) => (predicate, None),
            Clause::ForAll(id) => {
                let value = facts
                    .types
                    .quantified(id)
                    .ok_or("missing defining condition binder")?;
                (value.predicate, Some(value.bound_regions))
            }
        };
        let before = context.steps;
        let result = (|| {
            let mut substitution = Instantiation::new(context, &facts.types, &[])?;
            context.instantiate_predicate(&mut facts.types, predicate, &mut substitution)
        })();
        method.charge(context.steps.saturating_sub(before), 0)?;
        Ok(Self {
            goal: result.map_err(|error| error.to_string())?,
            bound_regions,
            pending: None,
            work: 0,
            paid: 0,
        })
    }
    fn advance(
        &mut self,
        facts: &mut Declarations,
        inputs: &mut ProofInputs<'_>,
        method: &SelectedMethod,
    ) -> Result<selection::Progress, String> {
        let result = if let Some(pending) = self.pending.take() {
            let first = pending.allocation_end().max(inputs.frontier);
            pending.resume_counted(facts, inputs.context, &self.goal, first, &mut self.work)
        } else {
            // The counted engine owns its existing search storage. Reserve the
            // additional retained suspension shell before it can be published.
            method.charge(1, size_of::<selection::Pending>())?;
            let caller = selection::OwnedCaller {
                first_region: inputs.proof.regions.end,
                assumptions: std::sync::Arc::clone(&inputs.caller.assumptions),
                environment: inputs.caller.environment.clone(),
                retain_sized: inputs.caller.retain_sized,
            };
            selection::begin_canonical_counted(
                facts,
                inputs.context,
                &self.goal,
                caller,
                (inputs.availability, self.bound_regions),
                &mut self.work,
            )
        };
        let delta = self.work.saturating_sub(self.paid);
        self.paid = self.work;
        method.charge(delta, 0)?;
        result
    }
}

fn join_growth(proof: &Obligations, child: &Obligations) -> Growth {
    let mut growth = Growth::vector::<crate::types::outlives::Pair>(
        proof.equalities.len(),
        proof.equalities.capacity(),
        child.equalities.len(),
    );
    growth.include(Growth::vector::<crate::types::outlives::Pair>(
        proof.outlives.len(),
        proof.outlives.capacity(),
        child.outlives.len(),
    ));
    growth.include(Growth::vector::<(
        crate::types::TypeId,
        crate::types::RegionId,
    )>(
        proof.types.len(),
        proof.types.capacity(),
        child.types.len(),
    ));
    growth
}
