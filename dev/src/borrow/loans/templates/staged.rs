//! One initialized plan releases semantic borrows without publishing a loan proof.
use super::{Analysis, Error, Verifier, regions, symbolic};
use crate::{
    body::{ir::Body, regions::BuiltinRequirements},
    borrow::ownership::initialization,
    types::{TypeId, parameters::environment::Environment},
};
use std::sync::Arc;

pub(crate) struct Prepared<'a> {
    facts: initialization::Facts<'a>,
    plan: Option<symbolic::Plan>,
    parameters: Arc<Environment>,
    builtins: Option<Arc<BuiltinRequirements>>,
    seal: crate::borrow::loans::seal::Seal,
    defining_steps: usize,
}
impl<'a> Prepared<'a> {
    pub(super) fn new<S: std::hash::BuildHasher>(
        environment: &crate::borrow::Environment<'_, S>,
        facts: initialization::Facts<'a>,
        plan: Option<symbolic::Plan>,
        parameters: Arc<Environment>,
        contract: &regions::Body,
        owner: Option<Arc<()>>,
        defining_steps: usize,
    ) -> Self {
        let seal =
            crate::borrow::loans::seal::Seal::new(environment, facts.body(), contract, owner);
        Self {
            facts,
            plan,
            parameters,
            builtins: contract.builtins.clone(),
            seal,
            defining_steps,
        }
    }
    pub(crate) fn body(&self) -> &Body {
        self.facts.body()
    }
    pub(crate) fn active_const(&self) -> Result<bool, &'static str> {
        if self.eligibility().next().is_none() {
            return Ok(false);
        }
        if self.body().const_context {
            return Ok(true);
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
    pub(crate) fn eligibility(&self) -> impl Iterator<Item = Result<TypeId, &'static str>> + '_ {
        self.plan
            .iter()
            .flat_map(|plan| plan.schedule().cleanups())
            .filter_map(|cleanup| cleanup.live_type.map(Ok))
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
impl Verifier {
    pub(crate) fn finish<S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        prepared: Prepared<'_>,
        contract: &regions::Body,
    ) -> Result<Analysis, Error> {
        let parameters = crate::borrow::permissions::check(environment, contract)?
            .unwrap_or_else(|| Arc::clone(&prepared.parameters));
        self.complete(environment, prepared, contract, parameters)
    }
    pub(crate) fn finish_initial<S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        prepared: Prepared<'_>,
        contract: &regions::Body,
    ) -> Result<Analysis, Error> {
        let parameters = Arc::clone(&prepared.parameters);
        self.complete(environment, prepared, contract, parameters)
    }
    fn complete<S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        prepared: Prepared<'_>,
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
        let result = if let Some(plan) = &prepared.plan {
            let mut drops = self.drops.scoped(parameters);
            super::super::check_schedule(
                environment.types,
                environment.definitions,
                &mut self.components,
                &mut drops,
                &prepared.facts,
                contract,
                plan.schedule(),
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
        let result = result
            .map(|mut analysis| {
                analysis.steps = analysis.steps.saturating_add(prepared.defining_steps);
                analysis
            })
            .map_err(|e| Error {
                node: Some(e.node),
                message: e.message.into(),
            });
        drop(prepared);
        result
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
