//! Source facts become body-owned requirements without changing their proof scope.
use crate::{
    body::ir::ProofSite,
    types::parameters::{environment::Environment, predicates::Predicate},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct BuiltinRequirements {
    pub(crate) environment: Option<Arc<Environment>>,
    pub(crate) predicates: Vec<(ProofSite, Predicate)>,
    pub(crate) source_copy: Option<Arc<crate::sema::model::SourceCopyRequirements>>,
    prepared_copy: Option<Box<super::super::copy_requirements::Prepared>>,
    defining: Option<Arc<super::super::defining_cleanups::State>>,
}
impl From<&crate::sema::model::BuiltinRequirements> for BuiltinRequirements {
    fn from(source: &crate::sema::model::BuiltinRequirements) -> Self {
        Self {
            environment: source.environment.clone(),
            predicates: source
                .predicates
                .iter()
                .map(|&(site, predicate)| (site.into(), predicate))
                .collect(),
            source_copy: source.source_copy.clone(),
            prepared_copy: None,
            defining: None,
        }
    }
}

impl BuiltinRequirements {
    pub(crate) fn imported(
        environment: Arc<Environment>,
        predicates: Vec<(ProofSite, Predicate)>,
    ) -> Self {
        Self::unpaid_imported(Some(environment), predicates, None)
    }
    /// Raw imported requests carry no selected Copy or defining-body permission.
    pub(crate) fn unpaid_imported(
        environment: Option<Arc<Environment>>,
        predicates: Vec<(ProofSite, Predicate)>,
        source_copy: Option<Arc<crate::sema::model::SourceCopyRequirements>>,
    ) -> Self {
        Self {
            environment,
            predicates,
            source_copy,
            prepared_copy: None,
            defining: None,
        }
    }

    pub(crate) fn is_unprepared_imported(&self) -> bool {
        self.prepared_copy.is_none() && self.defining.is_none()
    }

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
            .saturating_add(
                self.defining
                    .as_ref()
                    .map_or(0, |state| state.retained_bytes()),
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
            defining: None,
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
        self.defining = None;
    }
    pub(crate) fn empty() -> Self {
        Self {
            environment: None,
            predicates: Vec::new(),
            source_copy: None,
            prepared_copy: None,
            defining: None,
        }
    }
    pub(crate) fn set_defining(
        &mut self,
        state: Option<Arc<super::super::defining_cleanups::State>>,
    ) {
        self.defining = state;
    }
    pub(crate) fn has_defining(&self) -> bool {
        self.defining.is_some()
    }
    pub(crate) fn defining(&self) -> Option<&super::super::defining_cleanups::State> {
        self.defining.as_deref()
    }
    pub(super) fn replace_source(
        target: &mut Option<Arc<Self>>,
        source: &crate::sema::model::BuiltinRequirements,
    ) {
        let mut updated = Self::from(source);
        updated.defining = target
            .as_ref()
            .and_then(|previous| previous.defining.clone());
        *target = Some(Arc::new(updated));
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
