//! Marker syntax validates an empty source trait; it supplies no overlap proof.
use crate::{
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
    },
};

pub(super) fn consume(
    report: &Report,
    facts: &mut Declarations,
    declaration: NodeKey,
    attribute: Attribute,
    allowances: &mut MacroAllowanceSession<'_>,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error(
            "marker attribute requires word form without an unsafe wrapper",
        ));
    }
    if !report.package.requested_marker_traits().map_err(error)?
        && !allowances.allows_at(
            declaration.file,
            attribute.span,
            &FeatureName::new("marker_trait_attr").map_err(error)?,
        )?
    {
        return Err(error(
            "marker trait requires marker_trait_attr or exact token allowance",
        ));
    }
    let unit = report.package.unit(declaration.file);
    let Kind::Item(index) = unit.configured.ast.node(declaration.node).kind else {
        return Err(error("marker attribute requires an actual trait"));
    };
    let ItemKind::Trait { items, .. } = unit.configured.ast.items[index as usize].kind else {
        return Err(error("marker attribute requires an actual trait"));
    };
    for &member in unit.configured.ast.list(items) {
        facts.reserve_must_use_source(declaration, attribute.span)?;
        if unit.configured.active[member.get() as usize] {
            return Err(error("marker traits cannot have associated items"));
        }
    }
    Ok(())
}

/// This documentary attribute grants neither overlap nor trait/call evidence.
pub(super) fn derived_impl(
    report: &Report,
    facts: &mut Declarations,
    declaration: NodeKey,
    attribute: Attribute,
) -> Result<(), Error> {
    facts.reserve_must_use_source(declaration, attribute.span)?;
    let error = |message| Error::at(declaration.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || attribute.input != Input::Empty {
        return Err(error("automatically_derived requires word form"));
    }
    let unit = report.package.unit(declaration.file);
    if !matches!(unit.configured.ast.node(declaration.node).kind,
        Kind::Item(index) if matches!(unit.configured.ast.items[index as usize].kind, ItemKind::Impl { trait_path: Some(_), .. }))
    {
        return Err(error(
            "automatically_derived requires an actual trait implementation",
        ));
    }
    Ok(())
}
