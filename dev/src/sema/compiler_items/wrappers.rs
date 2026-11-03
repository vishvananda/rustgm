//! Actual source wrapper declarations publish separate drop and pointer policies.
use crate::{
    resolve::{bindings::Declaration, expansion::Report, model::DefKind, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::{
        ItemId, Kind,
        aggregate::{self, FieldDestruction},
    },
};
const ROLES: [LangItem; 2] = [LangItem::ManuallyDrop, LangItem::MaybeDangling];
fn claim(
    report: &Report,
    facts: &Declarations,
    item: ItemId,
    role: LangItem,
) -> Result<Option<Declaration>, Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(declaration) = catalog.language(role).filter(|d| d.definition.0 == item.0) else {
        return Ok(None);
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    crate::sema::header_attributes::charge_source_contract_query(facts, 4)?;
    catalog.require_provider_report(report).map_err(error)?;
    let actual = report
        .resolution
        .definitions
        .get(item.0 as usize)
        .ok_or_else(|| error("source wrapper lost its actual definition"))?;
    let ast = &report.package.unit(declaration.origin.file).configured.ast;
    let Syntax::Item(index) = ast.node(declaration.origin.node).kind else {
        return Err(error("source wrapper lost its actual item"));
    };
    let written = ast.items[index as usize];
    if actual.kind != DefKind::Struct
        || actual.node != Some(declaration.origin)
        || !matches!(written.kind, ItemKind::Struct { .. })
        || ast.list(written.generics).len() != 1
    {
        return Err(error(
            "source wrapper changed its actual structure or generic arity",
        ));
    }
    Ok(Some(declaration))
}
/// Called after the real field templates are installed, before drop/union queries.
pub(in crate::sema) fn register(
    report: &Report,
    facts: &mut Declarations,
    item: ItemId,
) -> Result<(), Error> {
    for role in ROLES {
        let Some(declaration) = claim(report, facts, item, role)? else {
            continue;
        };
        let error = |message| Declarations::error(report, declaration.origin, message);
        if facts
            .aggregates
            .get(item)
            .is_none_or(|shape| shape.kind != aggregate::Kind::Struct)
        {
            return Err(error(
                "source wrapper requires its completed structure fields",
            ));
        }
        let growth = facts.aggregates.nominal_policy_growth(item);
        crate::sema::header_attributes::reserve_source_contracts(
            facts,
            1 + growth.work,
            growth.bytes,
        )?;
        match role {
            LangItem::ManuallyDrop => facts.aggregates.suppress_destruction(item),
            LangItem::MaybeDangling => facts.aggregates.permit_dangling_pointers(item),
            _ => Err("invalid source wrapper role"),
        }
        .map_err(error)?;
    }
    Ok(())
}
/// Role completion is distinct from constructors, fields and executable bodies.
pub(in crate::sema) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    for role in ROLES {
        let Some(declaration) = facts
            .source_bindings
            .as_deref()
            .and_then(|c| c.language(role))
        else {
            continue;
        };
        let item = ItemId(declaration.definition.0);
        let error = |message| Declarations::error(report, declaration.origin, message);
        claim(report, facts, item, role)?
            .ok_or_else(|| error("source wrapper claim disappeared"))?;
        if !matches!(facts.definitions.get(item.0 as usize).copied().flatten().and_then(|ty|facts.types.kind(ty)),Some(Kind::Adt(owner,_)) if owner==item)
            || !facts
                .generics
                .nominal_contracts
                .contains(&declaration.definition)
            || facts
                .aggregates
                .get(item)
                .is_none_or(|d| d.kind != aggregate::Kind::Struct)
            || match role {
                LangItem::ManuallyDrop => {
                    facts.aggregates.destruction(item) != Some(FieldDestruction::Never)
                }
                LangItem::MaybeDangling => {
                    facts.aggregates.permits_dangling_pointers(item) != Some(true)
                }
                _ => true,
            }
        {
            return Err(error(
                "source wrapper requires its actual completed nominal policy",
            ));
        }
        if !super::roles::nominal_completed(report, facts, role, declaration)? {
            super::roles::complete_nominal(facts, role, declaration)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/wrapper_rules.rs"]
mod tests;
