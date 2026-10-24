//! Only an actual materialization can enter dispatch with an instance Copy scope.
use super::{Declarations, Materialized};

impl Materialized {
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
