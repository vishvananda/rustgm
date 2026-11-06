//! Only an actual materialization can enter dispatch with an instance Copy scope.
use super::{Declarations, Materialized};

impl Materialized {
    pub(in crate::body) fn into_empty_async_parts(
        self,
    ) -> Result<(super::Body, super::super::regions::Contract), &'static str> {
        if self.copy_scope.is_some()
            || self.destruction.locals.iter().any(|&needed| needed)
            || self.destruction.values.iter().any(|&needed| needed)
            || self.destruction.projections.iter().any(|&needed| needed)
        {
            return Err(
                "future poll requires its actual scoped inputs and owned cleanup transform",
            );
        }
        Ok((self.body, self.contract))
    }

    #[cfg(test)]
    pub(crate) fn has_instance_copy_scope_for_test(&self) -> bool {
        self.copy_scope.is_some()
    }

    #[cfg(test)]
    pub(in crate::body) fn instance_copy_scope_for_test(
        &self,
    ) -> Option<&super::super::copy_requirements::materialized::InstanceScope> {
        self.copy_scope.as_deref()
    }

    pub(crate) fn prepare_source_copy(
        &self,
        facts: &mut Declarations,
        body: &super::Body,
        contract: &mut super::super::regions::Contract,
    ) -> Result<(), String> {
        super::super::copy_requirements::prepare_scoped(
            facts,
            body,
            contract,
            self.copy_scope.as_deref(),
        )
    }

    pub(crate) fn resolve(&mut self, facts: &mut Declarations) -> Result<(), String> {
        super::super::dispatch::materialized(
            facts,
            &mut self.body,
            &mut self.contract,
            self.copy_scope.as_deref_mut(),
        )
    }
}
