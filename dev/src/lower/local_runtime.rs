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
    let (facts, prepared) = crate::pipeline::prepare_local_runtime(report, roots)?;
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
        let program = super::instances::build_local_prepared(
            self.report,
            &mut self.facts,
            entry,
            self.prepared,
        )?;
        #[cfg(test)]
        eprintln!(
            "local-runtime actual CFG functions={} typed bodies={} canonical steps={}",
            self.facts.body_work.cfg_functions.get(),
            self.facts.typed_function_bodies.len(),
            self.facts.body_work.canonical_steps
        );
        Ok(program)
    }
}
