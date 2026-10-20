//! Retain only report-owned indexes and cleared storage between evaluation batches.
use super::{Declarations, Evaluator, Queries, Report};

/// This cache must stay with the report and declaration result that populated it.
/// Active queries, caller permissions and per-expression values are never retained.
#[derive(Default)]
pub(crate) struct Scratch(super::super::Scratch);
impl Scratch {
    pub(crate) const fn preparation_steps(&self) -> usize {
        self.0.preparation_steps
    }
}
impl<'a> Evaluator<'a> {
    pub(crate) fn with_scratch(
        report: &'a Report,
        facts: &'a mut Declarations,
        scratch: Scratch,
    ) -> Self {
        Self {
            queries: Queries::with_scratch(report, facts, None, scratch.0),
        }
    }
    pub(crate) fn into_scratch(self) -> Scratch {
        Scratch(self.queries.into_scratch())
    }
}
