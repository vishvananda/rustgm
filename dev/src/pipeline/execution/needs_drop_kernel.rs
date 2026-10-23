//! The source bool kernel retains ordinary original and specialized caller proofs.
use super::{Declarations, Engine, Error, FileId, Report, const_eval};
use crate::types::instance::Instance;
impl Engine {
    pub(super) fn source_needs_drop(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: Instance,
        state: Option<&const_eval::State>,
    ) -> Result<bool, Error> {
        let error = |message| Error::at(FileId(0), 0, message);
        let call = state
            .ok_or_else(|| error("source intrinsic requires a reached checked caller"))?
            .source_needs_drop_call(facts, &self.catalog, instance)
            .map_err(error)?
            .ok_or_else(|| error("source needs_drop caller lost its validated target"))?;
        if call.selection().fallback && !self.function(report, facts, instance)? {
            return Ok(false);
        }
        self.catalog
            .evaluate_needs_drop(facts, &call)
            .map_err(error)?;
        Ok(true)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/kernel_reuse.rs"]
mod tests;
