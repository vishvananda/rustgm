//! Closed source publication checks retained must-use and inline contracts.
use super::{Declarations, Report, Source};
use crate::resolve::{model::DefId, package::Error, unit::NodeKey};

impl Source {
    pub(super) fn require_plain_attributes(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(), Error> {
        self.require_attribute_context(report, facts)?;
        for index in 0..self.stage.definitions.len() {
            self.stage.charge(1).map_err(Error::unlocated)?;
            let Some((definition, declaration)) =
                selected_header(&self.stage, report, facts, index)?
            else {
                continue;
            };
            require_defaultness_transport(report, declaration)?;
            facts.reject_specialization_transport(report, definition)?;
            self.stage.charge(2).map_err(Error::unlocated)?;
            if let Some(contract) = facts.must_use_contract(report, definition)? {
                let source = contract.origin.source().ok_or_else(|| {
                    Error::unlocated("source publication borrowed an imported contract")
                })?;
                if source.declaration != declaration || source.owner.file != declaration.file {
                    return Err(Error::at(
                        source.owner.file,
                        source.span.start,
                        "source must_use contract names another actual declaration",
                    ));
                }
                return Err(report.diagnostic(Error::at(
                    source.owner.file,
                    source.span.start,
                    "source must_use attribute contract transport is not implemented",
                )));
            }
            if let Some(hint) = facts.inline_hint_for_definition(report, definition)? {
                let source = hint.origin.source().ok_or_else(|| {
                    Error::unlocated("source publication borrowed an imported hint")
                })?;
                if source.declaration != declaration || source.owner.file != declaration.file {
                    return Err(Error::at(
                        source.owner.file,
                        source.span.start,
                        "source inline hint names another actual declaration",
                    ));
                }
                return Err(report.diagnostic(Error::at(
                    source.owner.file,
                    source.span.start,
                    "source inline attribute contract transport is not implemented",
                )));
            }
        }
        Ok(())
    }
    pub(super) fn require_attribute_context(
        &self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(), Error> {
        self.stage.require_attribute_context(report, facts)
    }
}

pub(super) fn selected_header(
    stage: &crate::sema::crates::authored_export::Stage,
    report: &Report,
    facts: &Declarations,
    index: usize,
) -> Result<Option<(DefId, NodeKey)>, Error> {
    stage.selected_header(report, facts, index)
}

pub(super) fn require_defaultness_transport(
    report: &Report,
    declaration: NodeKey,
) -> Result<(), Error> {
    crate::sema::crates::authored_export::require_defaultness_transport(report, declaration)
}
