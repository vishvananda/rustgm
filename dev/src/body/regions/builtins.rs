//! Source facts become body-owned requirements without changing their proof scope.
use crate::{
    resolve::unit::NodeKey,
    types::parameters::{environment::Environment, predicates::Predicate},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct BuiltinRequirements {
    pub(crate) environment: Option<Arc<Environment>>,
    pub(crate) predicates: Vec<(NodeKey, Predicate)>,
    pub(crate) source_copy: Option<Arc<crate::sema::model::SourceCopyRequirements>>,
    prepared_copy: Option<Box<super::super::copy_requirements::Prepared>>,
}
impl From<&crate::sema::model::BuiltinRequirements> for BuiltinRequirements {
    fn from(source: &crate::sema::model::BuiltinRequirements) -> Self {
        Self {
            environment: source.environment.clone(),
            predicates: source.predicates.clone(),
            source_copy: source.source_copy.clone(),
            prepared_copy: None,
        }
    }
}

impl BuiltinRequirements {
    #[cfg(test)]
    pub(crate) fn prepared_for_test(
        &self,
    ) -> Result<&super::super::copy_requirements::Prepared, &'static str> {
        self.prepared_copy
            .as_deref()
            .ok_or("missing prepared source Copy")
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(crate::support::retained::vector(&self.predicates))
            .saturating_add(self.source_copy.as_ref().map_or(0, |requirements| {
                size_of_val(requirements.as_ref())
                    .saturating_add(crate::support::retained::vector(&requirements.goals))
            }))
            .saturating_add(
                self.prepared_copy
                    .as_ref()
                    .map_or(0, |prepared| prepared.retained_bytes()),
            )
    }

    pub(crate) fn source(
        environment: Option<Arc<Environment>>,
        requirements: Arc<crate::sema::model::SourceCopyRequirements>,
        prepared: super::super::copy_requirements::Prepared,
    ) -> Self {
        Self {
            environment,
            predicates: Vec::new(),
            source_copy: Some(requirements),
            prepared_copy: Some(Box::new(prepared)),
        }
    }
    pub(crate) fn set_source_copy(
        &mut self,
        requirements: Arc<crate::sema::model::SourceCopyRequirements>,
        prepared: super::super::copy_requirements::Prepared,
    ) {
        self.source_copy = Some(requirements);
        self.prepared_copy = Some(Box::new(prepared));
    }
    pub(crate) fn clear_source_copy(&mut self) {
        self.prepared_copy = None;
    }
    pub(crate) fn copy_decision(
        &self,
        body: &crate::body::ir::Body,
        ty: crate::types::TypeId,
    ) -> Result<Option<bool>, &'static str> {
        let Some(requirements) = &self.source_copy else {
            return Ok(None);
        };
        let prepared = self
            .prepared_copy
            .as_deref()
            .ok_or("source Copy requires prepared body-local selection")?;
        prepared
            .decision(body, requirements, self.environment.as_ref(), ty)
            .map(Some)
    }
}
