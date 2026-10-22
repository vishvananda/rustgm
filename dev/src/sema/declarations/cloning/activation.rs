//! Only the reserved canonical iterator implementation may activate a Clone body.
use super::{Checker, Error, State};
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::model::Declarations,
};
pub(in crate::sema) fn owns(report: &Report, owner: DefId) -> bool {
    report
        .resolution
        .builtin_traits
        .cloning()
        .and_then(|c| c.implementation)
        .is_some_and(|i| [i.definition, i.lifetime, i.parameter, i.method].contains(&owner))
}
impl Checker<'_> {
    pub(in crate::sema::declarations) fn activate_iterator_clone(
        &mut self,
        owner: DefId,
    ) -> Result<(), Error> {
        prepare(self.report, &mut self.facts, owner, false)?;
        let implementation = self
            .report
            .resolution
            .builtin_traits
            .cloning()
            .and_then(|c| c.implementation)
            .ok_or_else(|| {
                Error::at(
                    super::super::FileId(0),
                    0,
                    "missing iterator Clone identities",
                )
            })?;
        for id in [
            implementation.definition,
            implementation.lifetime,
            implementation.parameter,
            implementation.method,
        ] {
            self.states[id.0 as usize] = State::Complete;
        }
        Ok(())
    }
    pub(in crate::sema::declarations) fn validate_iterator_clone(
        &mut self,
        owner: DefId,
    ) -> Result<bool, Error> {
        if !self
            .facts
            .cloning
            .as_deref()
            .and_then(|r| r.original)
            .is_some_and(|o| o.declaration.definition == owner)
        {
            return Ok(false);
        }
        complete(self.report, &mut self.facts)?;
        Ok(true)
    }
}
pub(in crate::sema) fn prepare(
    report: &Report,
    facts: &mut Declarations,
    owner: DefId,
    validated: bool,
) -> Result<(), Error> {
    if !owns(report, owner) {
        return Err(Error::at(
            super::super::FileId(0),
            0,
            "unregistered iterator Clone demand",
        ));
    }
    let canonical = report
        .resolution
        .builtin_traits
        .cloning()
        .ok_or_else(|| Error::at(super::super::FileId(0), 0, "missing Clone declaration"))?;
    let registry = facts
        .cloning
        .as_deref()
        .ok_or_else(|| Declarations::error(report, canonical.origin, "missing Clone contract"))?;
    if registry.canonical != canonical {
        return Err(Declarations::error(
            report,
            canonical.origin,
            "Clone demand differs from registered declaration",
        ));
    }
    if registry.complete {
        return Ok(());
    }
    let collection = report
        .resolution
        .builtin_traits
        .collections()
        .ok_or_else(|| Declarations::error(report, canonical.origin, "missing canonical iterator"))?
        .declarations[crate::support::iteration::Collection::SliceIterator as usize]
        .definition;
    if !collection_complete(facts) {
        super::super::iteration::collections::prepare(report, facts, collection, validated)?;
    }
    if facts.cloning.as_deref().and_then(|r| r.original).is_none() {
        let original = super::super::comparisons::LibraryContracts { report, facts }
            .iterator_clone_original(canonical)?;
        facts
            .cloning
            .as_deref_mut()
            .ok_or_else(|| Declarations::error(report, canonical.origin, "missing Clone registry"))?
            .original = Some(original);
    }
    complete(report, facts)
}
fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let registry = facts
        .cloning
        .as_deref()
        .ok_or_else(|| Error::at(super::super::FileId(0), 0, "missing Clone registry"))?;
    if registry.complete {
        return Ok(());
    }
    let canonical = registry.canonical;
    let original = registry.original.ok_or_else(|| {
        Declarations::error(report, canonical.origin, "missing iterator Clone original")
    })?;
    super::super::iteration::collections::conformance::check_overlap(
        report,
        facts,
        canonical.origin,
    )?;
    super::super::comparisons::LibraryContracts { report, facts }
        .check_iterator_clone(canonical, original)?;
    facts
        .cloning
        .as_deref_mut()
        .ok_or_else(|| {
            Declarations::error(report, canonical.origin, "missing Clone completion owner")
        })?
        .complete = true;
    Ok(())
}

// A real selected/reified member records executable demand. Resolver names and
// unsuccessful applicability candidates never call this facade.
pub(in crate::sema) fn request_member(
    report: &Report,
    facts: &mut Declarations,
    owner: DefId,
    ty: crate::types::TypeId,
) -> Result<(), Error> {
    let Some(registry) = facts.cloning.as_deref_mut() else {
        return Ok(());
    };
    if owner != registry.canonical.method {
        return Ok(());
    }
    if !matches!(facts.types.kind(ty), Some(crate::types::Kind::FunctionItem { item, .. }) if item.0 == owner.0)
    {
        return Err(Declarations::error(
            report,
            registry.canonical.origin,
            "selected Clone member has a different callable identity",
        ));
    }
    registry.requested = true;
    prepare_requested(report, facts)
}
pub(in crate::sema) fn prepare_requested(
    report: &Report,
    facts: &mut Declarations,
) -> Result<(), Error> {
    let Some(registry) = facts.cloning.as_deref() else {
        return Ok(());
    };
    if !registry.requested || registry.complete || !collection_complete(facts) {
        return Ok(());
    }
    let Some(implementation) = registry.canonical.implementation else {
        return Ok(());
    };
    let validated = facts
        .generics
        .implementations
        .as_ref()
        .is_some_and(|r| r.headers_validated());
    prepare(report, facts, implementation.definition, validated)
}
fn collection_complete(facts: &Declarations) -> bool {
    facts
        .iteration
        .as_deref()
        .and_then(|r| r.collections.as_deref())
        .is_some_and(|r| r.status == crate::sema::iteration::collections::Status::Complete)
}
