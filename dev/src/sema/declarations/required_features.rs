//! Required features consume the completed source request inventory.
use super::{Checker, Error};

impl Checker<'_> {
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
