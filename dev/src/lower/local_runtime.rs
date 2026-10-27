//! Explicit local execution roots; this result cannot enter whole-source publication.
use crate::pipeline::Prepared;
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
};

#[cfg(test)]
#[path = "../../tests/invariants/local_runtime_roots.rs"]
mod tests;

/// No public facts/prepared conversion: unrelated source bodies can remain Pending.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "private selected local runtime awaits the complete source panic consumer; it cannot publish a whole-source artifact"
    )
)]
pub(crate) struct LocalAnalysis<'a> {
    report: &'a Report,
    facts: Declarations,
    prepared: Prepared,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "private selected local runtime awaits the complete source panic consumer; public all-body analysis is unchanged"
    )
)]
pub(crate) fn analyze<'a>(report: &'a Report, roots: &[DefId]) -> Result<LocalAnalysis<'a>, Error> {
    let (facts, prepared) = crate::pipeline::local_runtime::prepare(report, roots)?;
    Ok(LocalAnalysis {
        report,
        facts,
        prepared,
    })
}
impl LocalAnalysis<'_> {
    /// Consumes only real checked originals; a demanded Pending callee is an error.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private selected local runtime awaits the complete source panic consumer; no artifact/profile authority is granted"
        )
    )]
    pub(crate) fn build(mut self, entry: DefId) -> Result<crate::lowir::program::Program, Error> {
        if !self.facts.typed_function_bodies.contains(&entry) {
            return Err(Error::unlocated(
                "local runtime entry lacks its actual completed body",
            ));
        }
        super::instances::build_local_prepared(self.report, &mut self.facts, entry, self.prepared)
    }
}
