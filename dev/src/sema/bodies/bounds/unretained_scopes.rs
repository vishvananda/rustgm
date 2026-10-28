//! Actual body checking retains completed empty scopes without bypassing body aliases.
use super::{Checker, DefId, Error, NodeId};
use crate::{support::retained, types::parameters::environment::Environment};
use std::sync::Arc;
const LIMIT: usize = 64 * 1024 * 1024;

impl Checker<'_> {
    pub(super) fn caller_declaration_environment(
        &mut self,
        node: NodeId,
        owner: DefId,
    ) -> Result<Option<Arc<Environment>>, Error> {
        if let Some(environment) = self.facts.generics.environments.get(&owner) {
            return Ok(Some(Arc::clone(environment)));
        }
        if !self.facts.generics.caller_scope_pending(owner) {
            return Ok(None);
        }
        let environment = self
            .facts
            .source_parameter_environment(self.report, self.key(node), owner)
            .map_err(|error| self.error(node, error))?;
        self.pay_scope_retention(node, 1, size_of::<Environment>() + 2 * size_of::<usize>())?;
        self.body_environment_capacity(node, owner)?;
        Ok(Some(Arc::new(environment)))
    }
    pub(super) fn retain_body_environment(
        &mut self,
        node: NodeId,
        owner: DefId,
        environment: Arc<Environment>,
    ) -> Result<(), Error> {
        self.pay_scope_retention(node, 1, 0)?;
        let growth = self.body_environment_capacity(node, owner)?;
        self.pay_scope_retention(node, growth.work, growth.bytes)?;
        self.facts
            .generics
            .body_environments
            .insert(owner, environment);
        Ok(())
    }
    fn body_environment_capacity(
        &self,
        node: NodeId,
        owner: DefId,
    ) -> Result<retained::Growth, Error> {
        let rows = &self.facts.generics.body_environments;
        let additional = usize::from(!rows.contains_key(&owner));
        let growth = retained::Growth::map::<DefId, Arc<Environment>>(
            rows.len(),
            rows.capacity(),
            additional,
        );
        let envelope = retained::map(rows)
            .saturating_add(growth.bytes)
            .saturating_add(
                rows.len()
                    .saturating_add(additional)
                    .saturating_mul(size_of::<Environment>() + 2 * size_of::<usize>()),
            );
        if envelope > LIMIT {
            return Err(self.error(
                node,
                "retained caller environment cache exceeds 64 MB budget",
            ));
        }
        Ok(growth)
    }
    fn pay_scope_retention(
        &mut self,
        node: NodeId,
        steps: usize,
        bytes: usize,
    ) -> Result<(), Error> {
        self.state.steps = self.state.steps.saturating_add(steps);
        self.facts.body_work.canonical_steps =
            self.facts.body_work.canonical_steps.saturating_add(steps);
        if let Some(producer) = self.report.lint_producer() {
            producer
                .diagnostic_payload(self.key(node), steps, bytes)
                .map_err(Error::from)?;
        }
        if self.state.steps > 1_000_000 {
            return Err(self.error(node, "body type work budget exceeded"));
        }
        Ok(())
    }
}
