//! Only declaration roots enter the ordinary graph before an original is reached.
use super::{Error, error, s};
use crate::support::crates::generics::TypeRef;

pub(super) fn prepare(
    rows: s::ValidatedView<'_>,
    demand: &mut impl FnMut(TypeRef) -> Result<(), Error>,
) -> Result<(), Error> {
    for header in &rows.functions {
        for signature in [header.original, header.callable] {
            for &root in signature.parameters.slice(&rows.roots).map_err(error)? {
                demand(root)?;
            }
            demand(signature.result)?;
        }
        if let Some(receiver) = header.receiver {
            demand(receiver.base)?;
        }
    }
    for header in &rows.traits {
        demand(header.self_type)?;
    }
    for header in &rows.implementations {
        demand(header.self_type)?;
        if let Some(reference) = header.trait_application {
            application(rows, reference, demand)?;
        }
    }
    for clause in &rows.clauses {
        match &clause.requirement {
            s::Requirement::Trait {
                application: reference,
                ..
            } => application(rows, *reference, demand)?,
            s::Requirement::TypeOutlives { subject, .. } => demand(*subject)?,
            s::Requirement::Outlives(_) | s::Requirement::UnstableFeature(_) => {}
        }
    }
    Ok(())
}
fn application(
    rows: s::ValidatedView<'_>,
    reference: s::TraitApplicationRef,
    demand: &mut impl FnMut(TypeRef) -> Result<(), Error>,
) -> Result<(), Error> {
    let application = rows
        .trait_applications
        .get(reference.0 as usize)
        .ok_or_else(|| error("ordinary header lost its actual trait application"))?;
    demand(application.subject)?;
    for argument in application
        .arguments
        .slice(&rows.arguments)
        .map_err(error)?
    {
        if let s::Argument::Type(ty) = *argument {
            demand(ty)?;
        }
    }
    Ok(())
}
