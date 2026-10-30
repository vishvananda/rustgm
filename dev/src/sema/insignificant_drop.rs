//! Insignificance describes advisory drop-order diagnostics, never actual cleanup.
use crate::{
    resolve::{
        expansion::{MacroAllowanceSession, Report},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input, Style},
        item::ItemKind,
        token_tree::Forest,
    },
};

pub(super) fn consume(
    report: &Report,
    facts: &mut Declarations,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    allowances: &mut MacroAllowanceSession<'_>,
    seen: &mut bool,
) -> Result<(), Error> {
    facts.require_lint_context(report)?;
    facts.reserve_must_use_source(key, attribute.span)?;
    let error = |message| Error::at(key.file, attribute.span.start, message);
    if attribute.style != Style::Outer
        || attribute.input != Input::Empty
        || attribute.unsafe_wrapper
    {
        return Err(error(
            "rustc_insignificant_dtor requires an outer word attribute",
        ));
    }
    crate::resolve::bindings::root_options::require_internal(
        report, allowances, key.file, attribute, forest,
    )?;
    let unit = report.package.unit(key.file);
    if !matches!(unit.configured.ast.node(key.node).kind, Kind::Item(index)
        if matches!(unit.configured.ast.items[index as usize].kind,
            ItemKind::Struct { .. } | ItemKind::Enum { .. } | ItemKind::ForeignType))
    {
        return Err(error(
            "rustc_insignificant_dtor requires a struct, enum or foreign type",
        ));
    }
    if *seen {
        return Err(error("duplicate rustc_insignificant_dtor attribute"));
    }
    *seen = true;
    // The retained configured source already owns this annotation. Current
    // compilation has no significant-drop advisory producer; ordinary drop,
    // capture, move and borrow consumers keep their actual existing proofs.
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/invariants/insignificant_drops.rs"]
mod tests;
