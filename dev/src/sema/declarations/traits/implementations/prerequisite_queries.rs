//! Retain exact executable prerequisite queries while normal declaration work advances.
use super::{
    Checker, DefId, Error, Work,
    eligibility_profile::{Plan, Unsupported},
    scalar_readiness::{Pending, Phase},
};
use crate::{
    sema::traits::{assumptions::Assumptions, selection},
    types::infer::{Context, Instantiation, predicates::Predicate as Goal},
};
use std::sync::Arc;

pub(super) struct Attempt {
    plan: Plan,
    cursor: usize,
    context: Context,
    goal: Option<Goal>,
    query: Option<Box<selection::Pending>>,
    waiting: Option<selection::Dependency>,
    query_context_start: usize,
    query_work: usize,
    charged_work: usize,
}
enum Step {
    Complete,
    Wait(selection::Dependency),
    Unsupported(Unsupported),
}
impl Attempt {
    pub(super) fn new(plan: Plan) -> Self {
        Self {
            plan,
            cursor: 0,
            context: Context::default(),
            goal: None,
            query: None,
            waiting: None,
            query_context_start: 0,
            query_work: 0,
            charged_work: 0,
        }
    }

    fn advance(
        &mut self,
        facts: &mut crate::sema::model::Declarations,
        owner: DefId,
    ) -> Result<Step, String> {
        self.plan.require_current(facts, owner)?;
        while let Some(&requirement) = self.plan.requirements.get(self.cursor) {
            if self.goal.is_none() {
                let mut substitution = Instantiation::new(&mut self.context, &facts.types, &[])
                    .map_err(|error| error.to_string())?;
                self.goal = Some(
                    self.context
                        .instantiate_predicate(&mut facts.types, requirement, &mut substitution)
                        .map_err(|error| error.to_string())?,
                );
            }
            if self.query.is_none() {
                self.query_context_start = self.context.steps;
                self.query_work = 0;
            }
            let goal = self
                .goal
                .as_ref()
                .ok_or("missing scalar conformance prerequisite")?;
            let progress = if let Some(query) = self.query.take() {
                let end = query.allocation_end();
                query.resume_counted(facts, &mut self.context, goal, end, &mut self.query_work)?
            } else {
                let environment = self.plan.environment.clone();
                selection::begin_executable_counted(
                    facts,
                    &mut self.context,
                    goal,
                    selection::OwnedCaller {
                        assumptions: Arc::new(Assumptions::default()),
                        environment,
                        first_region: 1,
                        retain_sized: false,
                    },
                    &mut self.query_work,
                )?
            };
            match progress {
                selection::Progress::Pending(dependency, query) => {
                    self.query = Some(query);
                    self.waiting = Some(dependency);
                    return Ok(Step::Wait(dependency));
                }
                selection::Progress::Inferred { .. }
                | selection::Progress::Ready(selection::Selection::Deferred) => {
                    return Ok(Step::Unsupported(Unsupported::ProofDebt));
                }
                selection::Progress::Ready(selection::Selection::Proven(evidence)) => {
                    if evidence.provisional_size
                        || !evidence.sized.is_empty()
                        || !evidence.proof.is_empty()
                        || !self.context.region_equalities.is_empty()
                    {
                        return Ok(Step::Unsupported(Unsupported::ProofDebt));
                    }
                    self.cursor += 1;
                    self.goal = None;
                    self.waiting = None;
                }
                selection::Progress::Ready(
                    selection::Selection::Absent | selection::Selection::Refuted(_),
                ) => {
                    return Err(
                        "trait implementation does not satisfy the trait's requirements".into(),
                    );
                }
            }
        }
        Ok(Step::Complete)
    }
}

impl Checker<'_> {
    pub(super) fn scalar_impl_requirements(
        &mut self,
        owner: DefId,
        pending: &mut Pending,
    ) -> Result<(), Error> {
        pending
            .charge(&mut self.facts, 1)
            .map_err(Error::unlocated)?;
        let mut attempt = pending
            .active
            .remove(&owner)
            .ok_or_else(|| Error::unlocated("missing scalar conformance continuation"))?;
        if matches!(attempt.waiting, Some(selection::Dependency::ImplementationValidated(child))
            if pending.skipped.contains_key(&child))
        {
            pending
                .skip(&mut self.facts, owner, Unsupported::Requirements)
                .map_err(Error::unlocated)?;
            return Ok(());
        }
        pending
            .charge(&mut self.facts, 4)
            .map_err(|e| self.error(attempt.plan.source, e))?;
        let progress = attempt.advance(&mut self.facts, owner);
        let total = attempt.context.steps.max(
            attempt
                .query_context_start
                .saturating_add(attempt.query_work),
        );
        let work = total.saturating_sub(attempt.charged_work);
        attempt.charged_work = total;
        let charged = pending.charge(&mut self.facts, work);
        // Charge even a failed query, while preserving its original semantic error.
        let progress = progress.map_err(|error| self.error(attempt.plan.source, error))?;
        charged.map_err(|error| self.error(attempt.plan.source, error))?;
        match progress {
            Step::Complete => self.complete_scalar_impl(owner, &attempt.plan, pending),
            Step::Unsupported(reason) => {
                pending
                    .skip(&mut self.facts, owner, reason)
                    .map_err(Error::unlocated)?;
                Ok(())
            }
            Step::Wait(dependency) => {
                if let selection::Dependency::ImplementationValidated(child) = dependency {
                    if pending.active.contains_key(&child) {
                        return Err(self.error(
                            attempt.plan.source,
                            "cycle in implementation validation dependencies",
                        ));
                    }
                    if !self.ensure_scalar_plan(child, pending)? {
                        pending
                            .skip(&mut self.facts, owner, Unsupported::Requirements)
                            .map_err(Error::unlocated)?;
                        return Ok(());
                    }
                }
                pending.active.insert(owner, attempt);
                self.work
                    .push(Work::ValidateImpl(owner, Phase::Requirements));
                self.work.push(Work::selection_dependency(dependency));
                Ok(())
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/early_conformance.rs"]
mod tests;
