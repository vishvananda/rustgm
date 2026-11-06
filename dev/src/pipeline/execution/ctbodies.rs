//! An actual cleanup event demands normal const method Code without inventing a Call.
use super::{Declarations, Engine, Error, Report, const_eval};
impl Engine {
    pub(super) fn prepare_cleanup_body(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: crate::types::instance::Instance,
        state: Option<&mut const_eval::State>,
    ) -> Result<bool, Error> {
        let error = |message| Error::at(crate::resolve::unit::FileId(0), 0, message);
        let state =
            state.ok_or_else(|| error("cleanup body requires its actual reached caller"))?;
        let mut work = crate::body::defining_cleanups::Work::new(self.available_limits()?.steps);
        let reached = state.reached_cleanup(facts, &self.catalog, instance, &mut work);
        facts.body_work.canonical_steps =
            facts.body_work.canonical_steps.saturating_add(work.steps);
        let reached = reached.map_err(error)?;
        if !self.function_body(report, facts, instance, None)? {
            return Ok(false);
        }
        let mut work = crate::body::defining_cleanups::Work::new(self.available_limits()?.steps);
        let checked = state.require_cleanup(facts, &self.catalog, &reached, &mut work);
        facts.body_work.canonical_steps =
            facts.body_work.canonical_steps.saturating_add(work.steps);
        checked.map_err(error)?;
        Ok(true)
    }
}
