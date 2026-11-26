//! Only declaration roots enter the ordinary graph before an original is reached.
use super::{Error, error, s};
use crate::support::crates::generics::TypeRef;

pub(super) enum Root {
    Type(TypeRef),
    Region(s::RegionRef),
}

pub(super) fn prepare(
    rows: s::ValidatedView<'_>,
    demand: &mut impl FnMut(Root) -> Result<(), Error>,
) -> Result<(), Error> {
    for header in &rows.nominals {
        demand(Root::Type(header.self_type))?;
        for field in header.fields.slice(&rows.fields).map_err(error)? {
            demand(Root::Type(field.ty))?;
        }
    }
    for field in &rows.enums().fields {
        demand(Root::Type(field.ty))?;
    }
    // Version thirteen: a constant's declared type is its declaration's cell.
    for header in rows.constants() {
        demand(Root::Type(header.ty))?;
    }
    for alias in rows.type_aliases() {
        demand(Root::Type(alias.target))?;
    }
    for normalization in &rows.cold().alias_normalizations {
        demand(Root::Type(normalization.target))?;
    }
    for alias in rows.trait_aliases() {
        demand(Root::Type(alias.self_type))?;
    }
    if let Some(statics) = rows.statics() {
        for header in &statics.headers {
            demand(Root::Type(header.ty))?;
        }
    }
    for header in &rows.functions {
        for signature in [header.original, header.callable] {
            for &root in signature.parameters.slice(&rows.roots).map_err(error)? {
                demand(Root::Type(root))?;
            }
            demand(Root::Type(signature.result))?;
        }
        if let Some(receiver) = header.receiver {
            demand(Root::Type(receiver.base))?;
        }
    }
    for header in &rows.traits {
        demand(Root::Type(header.self_type))?;
    }
    for slot in &rows.slots {
        if let Some(default) = slot.default {
            demand(Root::Type(default))?;
        }
    }
    for header in &rows.implementations {
        demand(Root::Type(header.self_type))?;
        if let Some(reference) = header.trait_application {
            application(rows, reference, demand)?;
        }
    }
    for clause in &rows.clauses {
        match &clause.requirement {
            s::Requirement::Bootstrap { subject, .. } => demand(Root::Type(*subject))?,
            s::Requirement::Trait {
                application: reference,
                ..
            } => application(rows, *reference, demand)?,
            s::Requirement::TypeOutlives { subject, shorter } => {
                demand(Root::Type(*subject))?;
                demand(Root::Region(*shorter))?;
            }
            s::Requirement::ProjectionEqual { projection, value } => {
                demand(Root::Type(*projection))?;
                demand(Root::Type(*value))?;
            }
            s::Requirement::Outlives(pair) => {
                demand(Root::Region(pair.longer))?;
                demand(Root::Region(pair.shorter))?;
            }
            s::Requirement::UnstableFeature(_) => {}
        }
    }
    // Both sets supply independent declaration cells. Their recipes can occur
    // only in these headers, with no function signature or clause using them.
    for header in &rows.associated_types {
        demand(Root::Type(header.projection))?;
    }
    for definition in rows.associated_definitions() {
        demand(Root::Type(definition.value))?;
    }
    for opaque in &rows.cold().opaques.declarations {
        demand(Root::Type(opaque.application))?;
        if let Some(hidden) = opaque.hidden {
            demand(Root::Type(hidden))?;
        }
    }
    if let Some(closures) = rows.closures() {
        for header in &closures.headers {
            demand(Root::Type(header.signature))?;
            if let Some(execution) = header.execution {
                demand(Root::Type(execution.self_ty))?;
                demand(Root::Type(execution.signature))?;
            }
        }
    }
    Ok(())
}
fn application(
    rows: s::ValidatedView<'_>,
    reference: s::TraitApplicationRef,
    demand: &mut impl FnMut(Root) -> Result<(), Error>,
) -> Result<(), Error> {
    let application = rows
        .trait_applications
        .get(reference.0 as usize)
        .ok_or_else(|| error("ordinary header lost its actual trait application"))?;
    demand(Root::Type(application.subject))?;
    for argument in application
        .arguments
        .slice(&rows.arguments)
        .map_err(error)?
    {
        match *argument {
            s::Argument::Type(ty) => demand(Root::Type(ty))?,
            s::Argument::Region(region) => demand(Root::Region(region))?,
            s::Argument::Const(term) => match rows.consts().get(term.0 as usize) {
                Some(s::ConstTerm::Value(value)) => demand(Root::Type(value.ty))?,
                Some(s::ConstTerm::Variant(value)) => demand(Root::Type(value.ty))?,
                Some(s::ConstTerm::Parameter(_)) => {}
                None => return Err(error("source trait argument lost its const term")),
            },
        }
    }
    Ok(())
}
