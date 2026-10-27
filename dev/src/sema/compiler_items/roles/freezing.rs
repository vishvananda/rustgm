//! A source Freeze role retains actual unsafe-auto syntax; no property is granted here.
use super::{Declaration, Declarations, Error, ItemKind, Report, item};
pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
    role: crate::support::bindings::LangItem,
) -> Result<(), Error> {
    let label = match role {
        crate::support::bindings::LangItem::Freeze => "Freeze",
        crate::support::bindings::LangItem::Send => "Send",
        crate::support::bindings::LangItem::Sync => "Sync",
        crate::support::bindings::LangItem::Unpin => "Unpin",
        _ => {
            return Err(Declarations::error(
                report,
                declaration.origin,
                "invalid source auto marker",
            ));
        }
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    let syntax = item(report, declaration)?;
    let ItemKind::Trait { bounds, .. } = syntax.kind else {
        return Err(error(format!(
            "source {label} requires its actual auto trait"
        )));
    };
    let ast = &report.package.unit(declaration.origin.file).configured.ast;
    let members = facts
        .associated
        .owner(declaration.definition)
        .and_then(|owner| facts.associated.owner_members(owner))
        .ok_or_else(|| error(format!("source {label} requires completed members")))?;
    if !ast.list(bounds).is_empty()
        || !ast.list(syntax.where_clause).is_empty()
        || !members.is_empty()
    {
        return Err(error(format!(
            "source {label} requires its empty pinned auto-trait contract"
        )));
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
    let application_item = facts
        .types
        .trait_application(application)
        .ok_or("invalid source auto-trait application")?
        .item;
    let mut selected = None;
    for role in [
        crate::support::bindings::LangItem::Freeze,
        crate::support::bindings::LangItem::Send,
        crate::support::bindings::LangItem::Sync,
        crate::support::bindings::LangItem::Unpin,
    ] {
        if catalog
            .language(role)
            .is_some_and(|d| d.definition.0 == application_item.0)
        {
            selected = Some(role);
        }
    }
    let Some(role) = selected else {
        return Ok(false);
    };
    let declaration = catalog
        .language(role)
        .ok_or("source auto trait disappeared")?;
    let application = facts
        .types
        .trait_application(application)
        .ok_or("invalid Freeze trait application")?;
    if application.item.0 != declaration.definition.0 {
        return Ok(false);
    }
    catalog.validated_language(role)?;
    Ok(facts
        .types
        .arguments(application.arguments)
        .is_some_and(|a| a.len() == 0))
}
