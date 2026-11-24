//! Resumable declaration identities grant no generated body or state machine.
use super::Catalog;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
    },
    sema::model::Declarations,
    support::bindings::LangItem,
    types::ItemId,
};

/// Readiness for async result formation, before the final source role ledger joins.
/// No method body, poll ABI, pinning or executable future is supplied here.
pub(in crate::sema) fn signature_roles(
    report: &Report,
    facts: &Declarations,
) -> Result<(DefId, DefId), Error> {
    crate::sema::header_attributes::charge_source_contract_query(facts, 3)?;
    let catalog = facts.source_bindings.as_deref().ok_or_else(|| {
        Error::unlocated("async signatures require genuine source Future declarations")
    })?;
    catalog
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    let future = catalog
        .language(LangItem::Future)
        .ok_or_else(|| Error::unlocated("async signatures require the actual Future trait"))?;
    let output = catalog
        .language(LangItem::FutureOutput)
        .ok_or_else(|| Error::unlocated("async signatures require the actual Future Output"))?;
    let error = |message| Declarations::error(report, future.origin, message);
    super::headers::Trait::source(report, facts, future).map_err(error)?;
    if output.parent != Some(future.definition)
        || facts
            .types
            .projection_schema(ItemId(output.definition.0))
            .is_none_or(|schema| schema.trait_item.0 != future.definition.0)
    {
        return Err(error("async Future Output belongs to another actual trait"));
    }
    Ok((future.definition, output.definition))
}

/// Validate independently present source declarations without inventing a poll ABI.
/// Alternative unused language members remain ordinary checked source declarations.
pub(super) fn complete_future(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
) -> Result<[Option<LangItem>; 3], Error> {
    complete_roles(
        report,
        facts,
        catalog,
        [
            LangItem::Future,
            LangItem::FutureOutput,
            LangItem::FuturePoll,
        ],
    )
}

pub(super) fn complete_coroutine(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
) -> Result<[Option<LangItem>; 3], Error> {
    complete_roles(
        report,
        facts,
        catalog,
        [
            LangItem::CoroutineYield,
            LangItem::CoroutineReturn,
            LangItem::CoroutineResume,
        ],
    )
}

fn complete_roles<const N: usize>(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    roles: [LangItem; N],
) -> Result<[Option<LangItem>; N], Error> {
    let mut completed = [None; N];
    for (slot, role) in roles.into_iter().enumerate() {
        let Some(declaration) = catalog.language(role) else {
            continue;
        };
        crate::sema::header_attributes::charge_source_contract_query(facts, 2)?;
        let error = |message| Declarations::error(report, declaration.origin, message);
        catalog.require_provider_report(report).map_err(error)?;
        let definition = report
            .resolution
            .definitions
            .get(declaration.definition.0 as usize)
            .ok_or_else(|| error("source resumable member lost its declaration"))?;
        if definition.node != Some(declaration.origin) {
            return Err(error(
                "source resumable member changed its declaration identity",
            ));
        }
        match role {
            LangItem::Future => {
                super::headers::Trait::source(report, facts, declaration).map_err(error)?;
            }
            LangItem::FutureOutput | LangItem::CoroutineYield | LangItem::CoroutineReturn => {
                let schema = facts
                    .types
                    .projection_schema(ItemId(declaration.definition.0))
                    .ok_or_else(|| {
                        error("source associated result lacks its actual projection schema")
                    })?;
                if definition.kind != DefKind::TypeAlias
                    || declaration.parent != Some(DefId(schema.trait_item.0))
                {
                    return Err(error(
                        "source associated result changed its actual associated owner",
                    ));
                }
            }
            LangItem::FuturePoll | LangItem::CoroutineResume => {
                let function = facts
                    .function_header(declaration.definition)
                    .map_err(error)?
                    .ok_or_else(|| {
                        error("source resume/poll lacks its actual callable declaration")
                    })?;
                if function.declaration.source() != Some(declaration.origin)
                    || definition.kind != DefKind::Function
                {
                    return Err(error(
                        "source resume/poll changed its actual callable identity",
                    ));
                }
            }
            _ => return Err(error("invalid source resumable role completion")),
        }
        completed[slot] = Some(role);
    }
    Ok(completed)
}

/// Re-prove the imported owners and associated declarations. Their ordinary
/// clause environments and call signatures remain the source of usable proof.
pub(super) fn imported_declarations(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    steps: &mut usize,
    complete: &mut super::roles::RoleSet,
) -> Result<(), &'static str> {
    for role in [
        LangItem::AsyncFn,
        LangItem::AsyncFnMut,
        LangItem::AsyncFnOnce,
        LangItem::AsyncFnKindHelper,
        LangItem::Coroutine,
        LangItem::AsyncIterator,
    ] {
        *steps = steps.saturating_add(1);
        if let Some(owner) = catalog.role(role)? {
            let header = super::headers::Trait::current(report, facts, owner, steps)?;
            header.canonical_inputs(facts)?;
            complete.insert(role);
        }
    }
    for (role, owner) in LangItem::RESUMABLE_ASSOCIATED {
        *steps = steps.saturating_add(2);
        if let Some(member) = catalog.role(role)? {
            let actual = catalog
                .role(owner)?
                .ok_or("imported resumable associated result requires its actual trait role")?;
            if !complete.contains(owner)
                || member.parent != Some(actual.definition)
                || facts.associated.trait_owner(member.definition) != Some(actual.definition)
                || facts.generics.parents.get(&member.definition) != Some(&actual.definition)
                || facts
                    .types
                    .projection_schema(ItemId(member.definition.0))
                    .is_none_or(|schema| schema.trait_item.0 != actual.definition.0)
            {
                return Err("imported resumable associated result changed owner or projection");
            }
            complete.insert(role);
        }
    }
    for role in LangItem::RESUMABLE_REQUIRED {
        *steps = steps.saturating_add(4);
        if let Some(member) = catalog.role(role)? {
            imported_required(facts, catalog, role, member)?;
            complete.insert(role);
        }
    }
    Ok(())
}

fn imported_required(
    facts: &Declarations,
    catalog: &Catalog,
    role: LangItem,
    member: super::Role,
) -> Result<(), &'static str> {
    let owner = member
        .parent
        .ok_or("imported resumable method lost its actual trait")?;
    let expected = match role {
        LangItem::CoroutineResume => Some(LangItem::Coroutine),
        LangItem::AsyncIteratorPollNext => Some(LangItem::AsyncIterator),
        _ => None,
    };
    if let Some(expected) = expected
        && catalog
            .role(expected)?
            .is_none_or(|actual| actual.definition != owner)
    {
        return Err("imported resumable method changed role owner");
    }
    let function = facts
        .declaration_header(member.definition)?
        .ok_or("imported resumable method lost its actual callable")?;
    if facts.associated.trait_owner(member.definition) != Some(owner)
        || facts.generics.parents.get(&member.definition) != Some(&owner)
        || function.declaration != member.origin
        || function.body.is_some()
    {
        return Err("imported resumable method changed its actual required callable");
    }
    Ok(())
}
