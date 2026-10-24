//! Formatting identity comes from an actual nominal declaration, not an invented layout.
use crate::{
    resolve::{bindings::Declaration, expansion::Report, model::DefKind},
    sema::model::Declarations,
    support::bindings::LangItem,
    types::ItemId,
};
/// Structural header validation supplies no caller, trait or executable permission.
/// Ordinary nominal formation and the complete constructor call remain required.
pub(in crate::sema) fn declaration(
    report: &Report,
    facts: &Declarations,
    role: LangItem,
) -> Result<Option<Declaration>, &'static str> {
    if !matches!(role, LangItem::FormatArguments | LangItem::FormatArgument) {
        return Err("invalid formatting nominal role");
    }
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    if report.package.source_core().is_none() {
        return Err("foreign formatting roles require actual imported contract support");
    }
    let Some(declaration) = catalog.language(role) else {
        return Ok(None);
    };
    require_shape(report, declaration)?;
    let parameters = facts
        .lifetimes
        .parameters
        .get(&declaration.definition)
        .ok_or("formatting nominal lifetime header is not prepared")?;
    let [parameter] = parameters.as_slice() else {
        return Err("formatting nominal requires its actual single lifetime parameter");
    };
    let ast = &report.package.unit(declaration.origin.file).configured.ast;
    let crate::syntax::ast::Kind::Item(item) = ast.node(declaration.origin.node).kind else {
        return Err("missing formatting item");
    };
    let generic = ast.list(ast.items[item as usize].generics)[0];
    if report
        .resolution
        .definitions
        .get(parameter.0 as usize)
        .is_none_or(|binding| {
            binding.kind != DefKind::Generic
                || binding.namespace != crate::resolve::model::Namespace::Lifetime
                || binding.node
                    != Some(crate::resolve::unit::NodeKey {
                        node: generic,
                        ..declaration.origin
                    })
        })
    {
        return Err("formatting lifetime belongs to another declaration");
    }
    let shape = facts
        .aggregates
        .get(ItemId(declaration.definition.0))
        .ok_or("formatting nominal fields are not prepared")?;
    if shape.kind != crate::types::aggregate::Kind::Struct
        || shape.item.0 != declaration.definition.0
    {
        return Err("formatting nominal has a different actual shape owner");
    }
    Ok(Some(declaration))
}

pub(super) fn require_shape(report: &Report, declaration: Declaration) -> Result<(), &'static str> {
    let actual = report
        .resolution
        .definitions
        .get(declaration.definition.0 as usize)
        .ok_or("missing actual formatting nominal")?;
    if declaration.parent.is_some()
        || actual.kind != DefKind::Struct
        || actual.node != Some(declaration.origin)
    {
        return Err("formatting role requires its actual module-owned struct");
    }
    let ast = &report.package.unit(declaration.origin.file).configured.ast;
    let crate::syntax::ast::Kind::Item(item) = ast.node(declaration.origin.node).kind else {
        return Err("formatting role has no actual item");
    };
    let item = ast.items[item as usize];
    if !matches!(item.kind, crate::syntax::item::ItemKind::Struct { .. }) {
        return Err("formatting role is not its declared struct");
    }
    let [parameter] = ast.list(item.generics) else {
        return Err("formatting nominal requires one lifetime parameter");
    };
    let crate::syntax::ast::Kind::Generic(index) = ast.node(*parameter).kind else {
        return Err("missing formatting lifetime syntax");
    };
    if ast.generics[index as usize].flavor != crate::syntax::item::GenericFlavor::Lifetime {
        return Err("formatting nominal requires a lifetime parameter");
    }
    Ok(())
}
