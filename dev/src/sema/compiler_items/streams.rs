//! Ordinary future/async-iterator members provide no generated coroutine implementation.
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
) -> Result<[Option<LangItem>; 3], Error> {
    let mut completed = [None; 3];
    for (slot, role) in [
        LangItem::AsyncIteratorPollNext,
        LangItem::IntoAsyncIterIntoIter,
        LangItem::IntoFutureIntoFuture,
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
    for role in LangItem::ASYNC_GENERATOR_HELPERS {
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
        LangItem::AsyncIteratorPollNext
            | LangItem::IntoAsyncIterIntoIter
            | LangItem::IntoFutureIntoFuture
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

/// Retain only the actual inherent helpers on Poll<Option<T>>.
pub(super) fn imported_helpers(
    facts: &Declarations,
    catalog: &Catalog,
    steps: &mut usize,
    complete: &mut super::roles::RoleSet,
) -> Result<(), &'static str> {
    use crate::types::{Kind, TypeId, arguments::Argument};

    for role in LangItem::ASYNC_GENERATOR_HELPERS {
        *steps = steps.saturating_add(4);
        let Some(member) = catalog.role(role)? else {
            continue;
        };
        let owner = facts
            .associated
            .owner(member.definition)
            .and_then(|id| facts.associated.get(id))
            .ok_or("imported async generator helper lost its inherent owner")?;
        if !matches!(owner.kind, OwnerKind::ImportedInherent(_))
            || member.parent != Some(owner.self_definition)
        {
            return Err("imported async generator helper changed its inherent owner");
        }
        let schema = facts
            .associated
            .schema(owner.self_definition)
            .ok_or("imported async generator helper lost its actual Self schema")?;
        let single_argument = |ty: TypeId, role| -> Result<TypeId, &'static str> {
            let actual = catalog
                .role(role)?
                .ok_or("async generator helper lacks its actual Poll or Option role")?;
            let Some(Kind::Adt(item, arguments)) = facts.types.kind(ty) else {
                return Err("async generator helper has a non-nominal result");
            };
            let mut arguments = facts
                .types
                .arguments(arguments)
                .ok_or("async generator helper lost its nominal arguments")?;
            match (
                item.0 == actual.definition.0,
                arguments.next(),
                arguments.next(),
            ) {
                (true, Some(Argument::Type(ty)), None) => Ok(ty),
                _ => Err("async generator helper changed its nominal result"),
            }
        };
        let option = single_argument(schema.self_type, LangItem::Poll)?;
        let value = single_argument(option, LangItem::Option)?;
        if role == LangItem::AsyncGenReady {
            imported_async_ready(facts, member, (value, schema.self_type))?;
        } else if facts
            .definitions
            .get(member.definition.0 as usize)
            .copied()
            .flatten()
            != Some(schema.self_type)
        {
            return Err("imported async generator constant changed its actual Self type");
        }
        complete.insert(role);
    }
    Ok(())
}

fn imported_async_ready(
    facts: &Declarations,
    member: super::Role,
    (input, result): (crate::types::TypeId, crate::types::TypeId),
) -> Result<(), &'static str> {
    use crate::types::{Kind, callable::Convention};

    let header = facts
        .declaration_header(member.definition)?
        .ok_or("imported async generator ready helper lost its actual callable header")?;
    if header.declaration != member.origin
        || header.body.is_none()
        || !matches!(facts.types.kind(header.signature),
            Some(Kind::Function { parameters, result: actual, bound_regions: 0,
                unsafe_: false, convention: Convention::RUST })
            if actual == result && facts.types.list(parameters) == Some(&[input][..]))
    {
        return Err("imported async generator ready helper changed its actual defined signature");
    }
    Ok(())
}
