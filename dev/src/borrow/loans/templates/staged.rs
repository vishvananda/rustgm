//! One initialized plan releases semantic borrows without publishing a loan proof.
use super::{Analysis, Error, Payload, Verifier, regions, symbolic};
use crate::{
    body::{ir::Body, regions::BuiltinRequirements},
    borrow::ownership::initialization,
    types::{TypeId, parameters::environment::Environment},
};
use std::sync::Arc;

/// An actual physical initializer is not a selected plan or a completed loan proof.
pub(crate) struct Initial<'a> {
    pub(super) facts: initialization::Facts<'a>,
    pub(super) parameters: Arc<Environment>,
    builtins: Option<Arc<BuiltinRequirements>>,
    seal: crate::borrow::loans::seal::Seal,
    defining_steps: usize,
}
impl<'a> Initial<'a> {
    pub(super) fn new<S: std::hash::BuildHasher>(
        environment: &crate::borrow::Environment<'_, S>,
        facts: initialization::Facts<'a>,
        parameters: Arc<Environment>,
        contract: &regions::Body,
        owner: Option<Arc<()>>,
        defining_steps: usize,
    ) -> Self {
        let seal =
            crate::borrow::loans::seal::Seal::new(environment, facts.body(), contract, owner);
        Self {
            facts,
            parameters,
            builtins: contract.builtins.clone(),
            seal,
            defining_steps,
        }
    }
    pub(crate) fn body(&self) -> &'a Body {
        self.facts.body()
    }
    pub(crate) fn questions(
        &mut self,
        types: &mut crate::types::Store,
        definitions: &crate::types::aggregate::Definitions,
    ) -> Result<Option<&[TypeId]>, &'static str> {
        self.facts.qualification_questions(types, definitions)
    }
    pub(super) fn require<S: std::hash::BuildHasher>(
        &self,
        environment: &crate::borrow::Environment<'_, S>,
        contract: &regions::Body,
        owner: Option<&Arc<()>>,
    ) -> Result<(), &'static str> {
        self.seal
            .validate(environment, self.body(), contract, owner)
    }
    pub(super) fn select<P: Payload>(self, plan: Option<P>) -> Prepared<'a, P> {
        Prepared {
            facts: self.facts,
            plan,
            parameters: self.parameters,
            builtins: self.builtins,
            seal: self.seal,
            defining_steps: self.defining_steps,
        }
    }
}
pub(crate) struct Prepared<'a, P: Payload = symbolic::Plan> {
    facts: initialization::Facts<'a>,
    plan: Option<P>,
    parameters: Arc<Environment>,
    builtins: Option<Arc<BuiltinRequirements>>,
    seal: crate::borrow::loans::seal::Seal,
    defining_steps: usize,
}
impl<P: Payload> Prepared<'_, P> {
    pub(crate) fn body(&self) -> &Body {
        self.facts.body()
    }
    pub(crate) fn active_const(&self) -> Result<bool, &'static str> {
        let eligible = self.eligibility().next().is_some();
        if self.body().const_context {
            return Ok(eligible || self.active().next().is_some());
        }
        if !eligible {
            return Ok(false);
        }
        let Some(requirements) = self.builtins.as_deref() else {
            return Ok(false);
        };
        let Some(state) = requirements.defining() else {
            return Ok(false);
        };
        let prepared = state.checked(
            self.body(),
            requirements.environment.as_ref(),
            &mut crate::body::defining_cleanups::Work::new(1_000_000),
        )?;
        Ok(prepared.checks_cleanup())
    }
    pub(crate) fn resolve_selected(
        &mut self,
        types: &mut crate::types::Store,
        definitions: &crate::types::aggregate::Definitions,
    ) -> Result<(), &'static str> {
        self.facts.resolve_selected(types, definitions)
    }
    pub(crate) fn eligibility(
        &self,
    ) -> impl Iterator<Item = Result<crate::body::defining_cleanups::Subject, &'static str>> + '_
    {
        let selected = self.facts.selected();
        let subjects = selected.map(crate::borrow::ownership::Selected::subjects);
        let full = subjects
            .and_then(Result::ok)
            .into_iter()
            .flatten()
            .copied()
            .map(Ok);
        let errors = subjects.and_then(Result::err).into_iter().map(Err);
        let whole = self
            .plan
            .iter()
            .flat_map(|plan| plan.schedule().cleanups())
            .filter(move |_| selected.is_none())
            .filter_map(|cleanup| {
                cleanup.live_type.map(|ty| {
                    Ok(crate::body::defining_cleanups::Subject {
                        purpose: crate::body::defining_cleanups::Purpose::Body,
                        ty,
                        access: cleanup.access,
                    })
                })
            });
        full.chain(errors).chain(whole)
    }
    pub(crate) fn active(&self) -> impl Iterator<Item = Result<TypeId, &'static str>> + '_ {
        self.plan
            .iter()
            .flat_map(|plan| plan.schedule().cleanups())
            .filter(|cleanup| cleanup.access)
            .map(|cleanup| {
                cleanup
                    .live_type
                    .ok_or("active const cleanup has no initialized subject")
            })
    }
    fn validate<S: std::hash::BuildHasher>(
        &self,
        verifier: &Verifier,
        environment: &crate::borrow::Environment<'_, S>,
        contract: &regions::Body,
    ) -> Result<(), &'static str> {
        self.seal
            .validate(environment, self.body(), contract, verifier.owner.as_ref())
    }
}

/// Only successful same-owner loan completion can move out the concrete plan.
/// The source body remains borrowed until the owned execution snapshot is built.
pub(crate) struct Execution<'a> {
    body: &'a Body,
    plan: Option<crate::borrow::destruction::Plan>,
    seal: crate::borrow::loans::seal::Seal,
}
impl<'body> Execution<'body> {
    pub(crate) fn active(&self) -> bool {
        self.plan
            .as_ref()
            .is_some_and(|plan| plan.cleanups.iter().any(|cleanup| cleanup.node.is_some()))
    }
    pub(crate) fn planning_steps(&self) -> usize {
        self.plan.as_ref().map_or(0, |plan| plan.steps)
    }
    pub(crate) fn require_current<S: std::hash::BuildHasher>(
        &self,
        environment: &crate::borrow::Environment<'_, S>,
        body: &Body,
        regions: &regions::Body,
    ) -> Result<(), &'static str> {
        if !std::ptr::eq(self.body, body) {
            return Err("completed cleanup belongs to another borrowed source body");
        }
        self.seal.validate_context(environment, body, regions)
    }
    pub(crate) fn into_plan(self) -> (&'body Body, Option<crate::borrow::destruction::Plan>) {
        (self.body, self.plan)
    }
}

const _: [(); 784] = [(); size_of::<Prepared<'_>>()];
const _: [(); 904] = [(); size_of::<Prepared<'_, crate::borrow::destruction::Plan>>()];
const _: [(); 224] = [(); size_of::<Execution<'_>>()];
impl Verifier {
    pub(crate) fn begin_qualification<S: std::hash::BuildHasher>(
        &self,
        environment: &crate::borrow::Environment<'_, S>,
        initial: &mut Initial<'_>,
        contract: &regions::Body,
        (requirements, mode): (
            &crate::body::drops::Requirements,
            initialization::Refinement,
        ),
        position: &impl Fn(
            TypeId,
            crate::body::drops::MemberQuery,
        ) -> Result<crate::body::drops::MemberPosition, &'static str>,
    ) -> Result<(), &'static str> {
        initial.require(environment, contract, self.owner.as_ref())?;
        initial.facts.begin_qualification(
            environment.types,
            environment.definitions,
            &|item| self.custom.contains(&item),
            (mode, Some(requirements)),
            position,
        )
    }
    pub(crate) fn accept_qualification<S: std::hash::BuildHasher>(
        &self,
        environment: &crate::borrow::Environment<'_, S>,
        initial: &mut Initial<'_>,
        contract: &regions::Body,
        answers: &crate::body::defining_cleanups::Qualification<'_>,
    ) -> Result<(), &'static str> {
        initial.require(environment, contract, self.owner.as_ref())?;
        initial.facts.accept_qualification(
            environment.types,
            environment.definitions,
            contract,
            answers,
        )?;
        initial.builtins.clone_from(&contract.builtins);
        Ok(())
    }

    pub(crate) fn finish<S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        prepared: Prepared<'_>,
        contract: &regions::Body,
    ) -> Result<Analysis, Error> {
        let parameters = crate::borrow::permissions::check(environment, contract)?
            .unwrap_or_else(|| Arc::clone(&prepared.parameters));
        let result = self.complete(environment, &prepared, contract, parameters);
        drop(prepared);
        result
    }
    pub(crate) fn finish_initial<S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        prepared: Prepared<'_>,
        contract: &regions::Body,
    ) -> Result<Analysis, Error> {
        let parameters = Arc::clone(&prepared.parameters);
        let result = self.complete(environment, &prepared, contract, parameters);
        drop(prepared);
        result
    }
    pub(crate) fn finish_execution<'a, S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        prepared: Prepared<'a, crate::borrow::destruction::Plan>,
        contract: &regions::Body,
        refreshed: bool,
    ) -> Result<Execution<'a>, Error> {
        let parameters = if refreshed {
            crate::borrow::permissions::check(environment, contract)?
                .unwrap_or_else(|| Arc::clone(&prepared.parameters))
        } else {
            Arc::clone(&prepared.parameters)
        };
        self.complete(environment, &prepared, contract, parameters)?;
        Ok(Execution {
            body: prepared.facts.body(),
            plan: prepared.plan,
            seal: prepared.seal,
        })
    }
    pub(crate) fn require_execution<S: std::hash::BuildHasher>(
        &self,
        environment: &crate::borrow::Environment<'_, S>,
        execution: &Execution<'_>,
        contract: &regions::Body,
    ) -> Result<(), &'static str> {
        execution
            .seal
            .validate(environment, execution.body, contract, self.owner.as_ref())
    }
    fn complete<S: std::hash::BuildHasher, P: Payload>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        prepared: &Prepared<'_, P>,
        contract: &regions::Body,
        parameters: Arc<Environment>,
    ) -> Result<Analysis, Error> {
        let error = |message: &str| Error {
            node: Some(prepared.body().source.into()),
            message: message.into(),
        };
        prepared
            .validate(self, environment, contract)
            .map_err(error)?;
        let schedule = prepared
            .plan
            .as_ref()
            .map(|plan| {
                plan.schedule()
                    .with_selected(prepared.body(), prepared.facts.selected())
            })
            .transpose()
            .map_err(error)?;
        let result = if let Some(schedule) = schedule {
            let mut drops = self.drops.scoped(parameters);
            super::super::check_schedule(
                environment.types,
                environment.definitions,
                &mut self.components,
                &mut drops,
                &prepared.facts,
                contract,
                schedule,
            )
        } else {
            super::super::check(
                environment.types,
                &mut self.components,
                prepared.body(),
                contract,
                &prepared.facts.ownership,
                &prepared.facts.storage,
            )
        };
        result
            .map(|mut analysis| {
                analysis.steps = analysis.steps.saturating_add(prepared.defining_steps);
                analysis
            })
            .map_err(|e| Error {
                node: Some(e.node),
                message: e.message.into(),
            })
    }
}
pub(super) fn require_arenas<S: std::hash::BuildHasher>(
    environment: &crate::borrow::Environment<'_, S>,
    body: &Body,
) -> Result<(), Error> {
    crate::borrow::loans::seal::Seal::require_arenas(environment).map_err(|message| Error {
        node: Some(body.source.into()),
        message: message.into(),
    })
}
