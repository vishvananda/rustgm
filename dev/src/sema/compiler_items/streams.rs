//! Ordinary async-iterator declarations provide no generated coroutine implementation.
use super::Catalog;
use crate::{
    resolve::{bindings::Declaration, expansion::Report, model::DefKind, package::Error},
    sema::{associated::OwnerKind, model::Declarations},
    support::bindings::LangItem,
};

/// Preserve unused alternative declarations while authenticating their real
/// canonical owner, member and callable/value identity. Ordinary selection and
/// body proof remain mandatory; generated async-gen consumers need separate proof.
pub(super) fn complete(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
) -> Result<[Option<LangItem>; 2], Error> {
    let mut completed = [None; 2];
    for (slot, role) in [
        LangItem::AsyncIteratorPollNext,
        LangItem::IntoAsyncIterIntoIter,
    ]
    .into_iter()
    .enumerate()
    {
        let Some(declaration) = catalog.language(role) else {
            continue;
        };
        member(report, facts, catalog, declaration, role)?;
        completed[slot] = Some(role);
    }
    Ok(completed)
}

/// Inherent binders and associated constant types are genuine late producers.
/// The narrow role ledger join is called only after normal signature completion.
pub(in crate::sema) fn complete_helpers(
    report: &Report,
    facts: &mut Declarations,
) -> Result<(), Error> {
    if report.package.source_core().is_none() {
        return Ok(());
    }
    for role in [
        LangItem::AsyncGenReady,
        LangItem::AsyncGenPending,
        LangItem::AsyncGenFinished,
    ] {
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(());
        };
        let Some(declaration) = catalog.language(role) else {
            continue;
        };
        member(report, facts, catalog, declaration, role)?;
        super::roles::complete_stream_helper(facts, role, declaration)?;
    }
    Ok(())
}

fn member(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    role: LangItem,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    crate::sema::header_attributes::charge_source_contract_query(facts, 5)?;
    catalog.require_provider_report(report).map_err(error)?;
    let definition = report
        .resolution
        .definitions
        .get(declaration.definition.0 as usize)
        .ok_or_else(|| error("source async iterator member lost its declaration"))?;
    let owner_id = facts
        .associated
        .owner(declaration.definition)
        .ok_or_else(|| error("source async iterator member lacks its actual owner"))?;
    let owner = facts
        .associated
        .get(owner_id)
        .ok_or_else(|| error("source async iterator member lost its actual owner"))?;
    crate::sema::header_attributes::charge_source_contract_query(
        facts,
        facts
            .associated
            .owner_members(owner_id)
            .map_or(0, <[_]>::len),
    )?;
    if definition.node != Some(declaration.origin)
        || facts.associated.schema(declaration.definition).is_none()
        || !facts
            .associated
            .owner_members(owner_id)
            .is_some_and(|members| members.contains(&declaration.definition))
    {
        return Err(error(
            "source async iterator member changed its actual schema",
        ));
    }
    require_owner(report, facts, declaration, role, owner)?;
    if matches!(role, LangItem::AsyncGenPending | LangItem::AsyncGenFinished) {
        if definition.kind != DefKind::Const
            || facts
                .definitions
                .get(declaration.definition.0 as usize)
                .copied()
                .flatten()
                .is_none()
        {
            return Err(error(
                "source async-gen constant lacks its actual value type",
            ));
        }
    } else {
        let function = facts
            .function_header(declaration.definition)
            .map_err(error)?
            .ok_or_else(|| error("source async iterator member lacks its callable header"))?;
        if definition.kind != DefKind::Function
            || function.declaration.source() != Some(declaration.origin)
        {
            return Err(error(
                "source async iterator method changed its callable identity",
            ));
        }
    }
    Ok(())
}

fn require_owner(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
    role: LangItem,
    owner: &crate::sema::associated::Owner,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    if matches!(
        role,
        LangItem::AsyncIteratorPollNext | LangItem::IntoAsyncIterIntoIter
    ) {
        let parent = facts
            .associated
            .trait_owner(declaration.definition)
            .ok_or_else(|| error("source async iterator method lacks its actual trait"))?;
        if declaration.parent != Some(parent) {
            return Err(error(
                "source async iterator method changed its actual trait",
            ));
        }
        let source = report
            .resolution
            .definitions
            .get(parent.0 as usize)
            .and_then(|d| d.node)
            .ok_or_else(|| error("source async iterator trait lost its source"))?;
        super::headers::Trait::source(
            report,
            facts,
            Declaration {
                definition: parent,
                parent: None,
                origin: source,
            },
        )
        .map_err(error)?
        .canonical_inputs(facts)
        .map_err(error)?;
    } else if declaration.parent != Some(owner.self_definition)
        || !matches!(
            owner.kind,
            OwnerKind::Impl {
                trait_type: None,
                ..
            }
        )
    {
        return Err(error(
            "source async-gen helper changed its actual inherent owner",
        ));
    }
    Ok(())
}
