//! Mutable contracts remain absent until exact typed mutable collection demand.
use super::{DefId, Error};
use crate::{
    resolve::expansion::Report,
    sema::{
        iteration::collections::{MutableRegistry, Status},
        model::Declarations,
    },
    support::library_types::LibraryType,
};
pub(super) fn prepare(
    report: &Report,
    facts: &mut Declarations,
    owner: DefId,
) -> Result<(), Error> {
    let Some(canonical) = report.resolution.builtin_traits.collections() else {
        return Ok(());
    };
    if !canonical.declarations().any(|d| {
        d.kind.mutable()
            && [Some(d.definition), Some(d.method), d.item, d.into_iter].contains(&Some(owner))
    }) {
        return Ok(());
    }
    if facts
        .iteration
        .as_deref()
        .and_then(|r| r.collections.as_deref())
        .and_then(|r| r.mutable.as_deref())
        .is_some()
    {
        return Ok(());
    }
    let canonical = canonical.mutable.as_deref().cloned().ok_or_else(|| {
        Error::at(
            canonical.origin.file,
            0,
            "missing mutable collection catalog",
        )
    })?;
    let iteration = report
        .resolution
        .builtin_traits
        .iteration()
        .ok_or_else(|| {
            Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "missing iteration contracts",
            )
        })?;
    let mut builder = super::super::super::comparisons::LibraryContracts { report, facts };
    builder.collection_nominal(LibraryType::SliceIterMut)?;
    builder.intrinsic_signature(canonical.read)?;
    let mut originals = [None; 4];
    for declaration in canonical.declarations {
        originals[declaration.kind.index()] =
            Some(builder.collection_contract(iteration, declaration)?);
    }
    builder
        .facts
        .iteration
        .as_deref_mut()
        .and_then(|r| r.collections.as_deref_mut())
        .ok_or_else(|| Error::at(iteration.origin.file, 0, "missing collection owner"))?
        .mutable = Some(Box::new(MutableRegistry {
        canonical,
        originals,
        status: Status::Prepared,
    }));
    Ok(())
}
