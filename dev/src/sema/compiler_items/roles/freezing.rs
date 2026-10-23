//! A source Freeze role retains actual unsafe-auto syntax; no property is granted here.
use super::{Declaration, Declarations, Error, ItemKind, Report, item};
pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let syntax = item(report, declaration)?;
    let ItemKind::Trait { bounds, .. } = syntax.kind else {
        return Err(error("source Freeze requires its actual auto trait"));
    };
    let ast = &report.package.unit(declaration.origin.file).configured.ast;
    let members = facts
        .associated
        .owner(declaration.definition)
        .and_then(|owner| facts.associated.owner_members(owner))
        .ok_or_else(|| error("source Freeze requires completed members"))?;
    if !ast.list(bounds).is_empty()
        || !ast.list(syntax.where_clause).is_empty()
        || !members.is_empty()
    {
        return Err(error(
            "source Freeze requires its empty pinned auto-trait contract",
        ));
    }
    Ok(())
}

/// A retained structural proof remains tied to the actual ordinary source role.
pub(crate) fn predicate(
    facts: &Declarations,
    predicate: crate::types::parameters::predicates::Predicate,
) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some((application, crate::support::constness::Constness::Ordinary)) =
        predicate.bound.trait_parts()
    else {
        return Ok(false);
    };
    let Some(declaration) = catalog.language(crate::support::bindings::LangItem::Freeze) else {
        return Ok(false);
    };
    let application = facts
        .types
        .trait_application(application)
        .ok_or("invalid Freeze trait application")?;
    if application.item.0 != declaration.definition.0 {
        return Ok(false);
    }
    catalog.validated_language(crate::support::bindings::LangItem::Freeze)?;
    Ok(facts
        .types
        .arguments(application.arguments)
        .is_some_and(|a| a.len() == 0))
}
