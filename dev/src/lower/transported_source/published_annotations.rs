//! Lowering delegates source contract completion to the one semantic producer.
use super::{Declarations, Report, Source, s};
use crate::resolve::package::Error;
impl Source {
    pub(super) fn collect_header_columns(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<s::HeaderAttributeColumns, Error> {
        self.stage.collect_header_columns(report, facts)
    }
}
