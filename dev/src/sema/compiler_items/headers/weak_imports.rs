//! Weak runtime-symbol declarations retain ordinary foreign headers, not operations.
use super::super::Catalog;
use crate::{
    resolve::{bindings::Declaration, expansion::Report, model::DefKind, package::Error},
    sema::{function_headers::FunctionHeader, model::Declarations},
    support::{
        bindings::LangItem,
        linkage::{Linkage, Naming},
    },
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::Kind,
};

#[cfg(test)]
#[path = "../../../../tests/invariants/weak_headers.rs"]
mod tests;

pub(in crate::sema) const ROLES: [LangItem; 6] = [
    LangItem::MemCpy,
    LangItem::MemMove,
    LangItem::MemSet,
    LangItem::MemCmp,
    LangItem::Bcmp,
    LangItem::StrLen,
];

pub(in crate::sema::compiler_items) fn complete(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    role: LangItem,
) -> Result<Option<LangItem>, Error> {
    let Some(declaration) = catalog.language(role) else {
        return Ok(None);
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    if !ROLES.contains(&role) {
        return Err(error(
            "foreign runtime header belongs to another role family",
        ));
    }
    crate::sema::header_attributes::charge_source_contract_query(facts, 8)?;
    catalog.require_provider_report(report).map_err(error)?;
    let header = facts
        .function_header(declaration.definition)
        .map_err(error)?
        .ok_or_else(|| error("foreign runtime declaration lacks its completed header"))?;
    current(report, declaration, &header).map_err(error)?;
    let work = registered(facts, declaration, &header).map_err(error)?;
    crate::sema::header_attributes::charge_source_contract_query(facts, work.saturating_add(1))?;
    Ok(Some(role))
}

fn current(
    report: &Report,
    declaration: Declaration,
    header: &FunctionHeader<'_>,
) -> Result<(), &'static str> {
    let actual = report
        .resolution
        .definitions
        .get(declaration.definition.0 as usize)
        .ok_or("foreign runtime declaration lost its actual definition")?;
    if actual.kind != DefKind::Function
        || actual.node != Some(declaration.origin)
        || header.declaration.source() != Some(declaration.origin)
        || header.body.is_some()
    {
        return Err("foreign runtime header changed its actual declaration or body");
    }
    let owner = report
        .resolution
        .foreign_owners
        .get(&declaration.definition)
        .copied()
        .filter(|owner| report.resolution.foreign_blocks.contains(owner))
        .ok_or("foreign runtime declaration lacks its actual external owner")?;
    let ast = &report.package.unit(owner.file).configured.ast;
    if !matches!(ast.node(owner.node).kind,
        Syntax::Item(index) if matches!(ast.items[index as usize].kind, ItemKind::ExternBlock{..}))
    {
        return Err("foreign runtime declaration changed its actual external block");
    }
    let ast = &report.package.unit(declaration.origin.file).configured.ast;
    if !matches!(ast.node(declaration.origin.node).kind,
        Syntax::Item(index) if matches!(ast.items[index as usize].kind, ItemKind::Function{body:None,..}))
    {
        return Err("foreign runtime declaration requires its actual body-free function");
    }
    Ok(())
}

fn registered(
    facts: &Declarations,
    declaration: Declaration,
    header: &FunctionHeader<'_>,
) -> Result<usize, &'static str> {
    let registry = facts
        .foreign
        .as_deref()
        .ok_or("foreign runtime registry is missing")?;
    let foreign = registry
        .function(declaration.definition)
        .ok_or("foreign runtime declaration lacks its actual registered import")?;
    if foreign.signature != header.signature || foreign.linkage != Linkage::Import {
        return Err("foreign runtime import differs from its actual signature or linkage");
    }
    let Some(Kind::Function { unsafe_, .. }) = facts.types.kind(header.signature) else {
        return Err("foreign runtime declaration lost its actual callable signature");
    };
    if unsafe_ != (foreign.safety != crate::syntax::item::Safety::Safe) {
        return Err("foreign runtime import differs from its actual safety");
    }
    let Naming::Exact(name) = foreign.naming else {
        return Err("foreign runtime import lacks its actual exact symbol identity");
    };
    let name = registry
        .names()
        .get(name)
        .ok_or("foreign runtime symbol belongs to another pool")?;
    let Some(Kind::FunctionItem {
        item,
        signature,
        arguments,
    }) = facts
        .definitions
        .get(declaration.definition.0 as usize)
        .copied()
        .flatten()
        .and_then(|ty| facts.types.kind(ty))
    else {
        return Err("foreign runtime declaration lacks its canonical function item");
    };
    if item.0 != declaration.definition.0
        || signature != header.signature
        || facts.types.arguments(arguments).is_none()
    {
        return Err("foreign runtime declaration changed its canonical signature or arguments");
    }
    Ok(name.len())
}
