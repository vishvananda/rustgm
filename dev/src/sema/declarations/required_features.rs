//! Required features consume the completed source request inventory.
use super::{Checker, Error};
use crate::{
    resolve::{expansion::Report, model::DefId, unit::NodeKey},
    types::parameters::environment::Environment,
};

fn payment(
    report: &Report,
    key: NodeKey,
) -> Option<impl Fn(usize, usize) -> Result<(), &'static str> + '_> {
    report.lint_producer().map(|producer| {
        move |steps, bytes| {
            producer
                .diagnostic_payload(key, steps, bytes)
                .map_err(|error| error.message)
        }
    })
}

impl crate::sema::model::Declarations {
    pub(crate) fn source_parameter_environment(
        &self,
        report: &Report,
        key: NodeKey,
        owner: DefId,
    ) -> Result<Environment, &'static str> {
        match payment(report, key) {
            Some(pay) => self
                .generics
                .parameter_environment_charged(&self.types, owner, &pay),
            None => self.generics.parameter_environment(&self.types, owner),
        }
    }
}

impl Checker<'_> {
    pub(super) fn complete_source_environment(
        &mut self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<(), Error> {
        let result = match payment(self.report, key) {
            Some(pay) => {
                self.facts
                    .generics
                    .complete_environment_charged(&mut self.facts.types, owner, &pay)
            }
            None => self
                .facts
                .generics
                .complete_environment(&mut self.facts.types, owner),
        };
        result.map_err(|e| self.error(key, e))
    }
    pub(super) fn elaborated_source_environment(
        &mut self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<Environment, Error> {
        let result = match payment(self.report, key) {
            Some(pay) => self.facts.generics.elaborated_environment_charged(
                &mut self.facts.types,
                owner,
                &pay,
            ),
            None => self
                .facts
                .generics
                .elaborated_environment(&mut self.facts.types, owner),
        };
        result.map_err(|e| self.error(key, e))
    }
    pub(super) fn prepare_feature_policy(&mut self) -> Result<(), Error> {
        let Some(library) = self.report.package.library_policy() else {
            return Ok(());
        };
        let requests = library.predicate_requests().map_err(Error::unlocated)?;
        let root = &self.report.package.modules[0];
        let key = crate::resolve::unit::NodeKey {
            file: root.file,
            node: root.root,
        };
        let producer = self
            .report
            .lint_producer()
            .ok_or_else(|| self.error(key, "source feature policy lost its attribute producer"))?;
        let pay = |steps, bytes| {
            producer
                .diagnostic_payload(key, steps, bytes)
                .map_err(|error| error.message)
        };
        pay(
            requests.len(),
            requests
                .len()
                .saturating_mul(size_of::<crate::types::parameters::features::FeatureId>()),
        )
        .map_err(|e| self.error(key, e))?;
        let mut enabled = Vec::with_capacity(requests.len());
        for request in requests {
            enabled.push(
                self.facts
                    .types
                    .intern_feature_charged(request, &pay)
                    .map_err(|e| self.error(key, e))?,
            );
        }
        self.facts
            .publish_feature_policy(&enabled, &pay)
            .map_err(|e| self.error(key, e))
    }
}
