//! Historical wire data cannot supply a removed implicit auto-trait premise.
use super::{Error, error};
use crate::support::{bindings::LangItem, crates::generics::source as s};

/// Run once before either ordinary or private canonical materialization.
pub(super) fn require(
    rows: s::ValidatedView<'_>,
    mut charge: impl FnMut(usize) -> Result<(), Error>,
) -> Result<(), Error> {
    charge(1)?;
    if rows.descriptor().profile() == s::Profile::Source2024Provided {
        return Err(error(
            "legacy Provided-v0 rows require the current semantic profile",
        ));
    }
    charge(rows.roles.len().saturating_add(rows.traits.len()))?;
    let meta = rows
        .roles
        .iter()
        .find(|binding| binding.role == LangItem::MetaSized)
        .map(|binding| binding.declaration);
    for header in &rows.traits {
        let inputs = header.inputs.slice(&rows.clauses).map_err(error)?;
        auto_prefix(header, inputs, &rows.trait_applications, meta, &mut charge)?;
    }
    Ok(())
}
fn auto_prefix(
    header: &s::TraitHeader,
    inputs: &[s::Clause],
    applications: &[s::TraitApplication],
    meta: Option<s::DeclarationAddress>,
    charge: &mut impl FnMut(usize) -> Result<(), Error>,
) -> Result<(), Error> {
    if header.flags.bits() & s::TraitFlags::AUTO == 0 || header.canonical_inputs == 0 {
        return Ok(());
    }
    let count = header.canonical_inputs as usize;
    charge(count)?;
    let prefix = inputs
        .get(..count)
        .ok_or_else(|| error("auto-trait canonical prefix is truncated"))?;
    for input in prefix {
        let s::Requirement::Trait { application, .. } = input.requirement else {
            continue;
        };
        let application = applications
            .get(application.0 as usize)
            .ok_or_else(|| error("auto-trait canonical application is unknown"))?;
        if application.subject != header.self_type {
            continue;
        }
        if Some(application.declaration) == meta {
            return Err(error(
                "legacy implicit auto-trait Self bounds cannot supply semantic premises",
            ));
        }
        if application.declaration.artifact.0 != 0 {
            return Err(error(
                "external auto-trait Self bounds require complete role provenance",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../tests/invariants/epoch_inputs.rs"]
mod tests;
