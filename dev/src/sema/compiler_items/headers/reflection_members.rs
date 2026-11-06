//! Unused reflection members retain their real ordinary trait or impl declarations.
use crate::{
    resolve::{bindings::Declaration, expansion::Report, model::DefKind, package::Error},
    sema::{associated::OwnerKind, header_attributes, model::Declarations},
    support::bindings::LangItem,
    types::ItemId,
};

pub(in crate::sema::compiler_items) const ROLES: [LangItem; 3] = [
    LangItem::FieldBase,
    LangItem::FieldType,
    LangItem::FieldOffset,
];

pub(in crate::sema) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    for role in ROLES {
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(());
        };
        let Some(declaration) = catalog.language(role) else {
            continue;
        };
        let error = |message| Declarations::error(report, declaration.origin, message);
        header_attributes::charge_source_contract_query(facts, 10)?;
        catalog.require_provider_report(report).map_err(error)?;
        member(report, facts, declaration, role).map_err(error)?;
        super::super::roles::complete_reflection_member(facts, role, declaration)?;
    }
    Ok(())
}

fn member(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
    role: LangItem,
) -> Result<(), &'static str> {
    let actual = report
        .resolution
        .definitions
        .get(declaration.definition.0 as usize)
        .ok_or("reflection member lost its actual source definition")?;
    let kind = if role == LangItem::FieldOffset {
        DefKind::Const
    } else {
        DefKind::TypeAlias
    };
    if actual.node != Some(declaration.origin) || actual.kind != kind {
        return Err("reflection member changed its actual declaration identity");
    }
    let owner = owner(report, facts, declaration)?;
    if role != LangItem::FieldOffset
        && let OwnerKind::Trait { definition, .. } = owner
    {
        if facts
            .types
            .projection_schema(ItemId(declaration.definition.0))
            .is_none_or(|schema| schema.trait_item.0 != definition.0)
        {
            return Err("reflection associated type lacks its actual trait projection schema");
        }
    } else if facts
        .definitions
        .get(declaration.definition.0 as usize)
        .copied()
        .flatten()
        .and_then(|ty| facts.types.kind(ty))
        .is_none()
    {
        return Err("reflection member lacks its completed ordinary declared type");
    }
    if role != LangItem::FieldOffset
        && matches!(owner, OwnerKind::Impl { .. })
        && facts.associated.type_definition(
            declaration
                .parent
                .ok_or("reflection alias lost its parent")?,
            declaration.definition,
        ) != Some(declaration.definition)
    {
        return Err("reflection alias differs from its actual implementation member");
    }
    Ok(())
}

fn owner(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
) -> Result<OwnerKind, &'static str> {
    let id = facts
        .associated
        .owner(declaration.definition)
        .ok_or("reflection member lacks its actual associated owner")?;
    let owner = facts.associated.source_owner(id)?;
    let parent = match owner.kind {
        OwnerKind::Trait { definition, .. } => definition,
        OwnerKind::Impl { .. } => owner.self_definition,
        OwnerKind::ImportedImpl(_) => {
            return Err("source reflection member cannot use imported ownership");
        }
    };
    if declaration.parent != Some(parent)
        || facts.associated.owner(parent) != Some(id)
        || facts.associated.schema(parent).is_none()
    {
        return Err("reflection member changed its actual parent or prepared owner schema");
    }
    let definitions = &report.resolution.definitions;
    let self_definition = definitions
        .get(owner.self_definition.0 as usize)
        .ok_or("reflection owner lost its actual Self definition")?;
    if self_definition.kind != DefKind::SelfType
        || self_definition.node != Some(owner.source)
        || definitions
            .get(declaration.definition.0 as usize)
            .is_none_or(|member| member.scope != self_definition.scope)
    {
        return Err("reflection member differs from its actual source owner scope");
    }
    if matches!(owner.kind, OwnerKind::Trait { .. }) {
        super::Trait::source(
            report,
            facts,
            Declaration {
                definition: parent,
                parent: None,
                origin: owner.source,
            },
        )?
        .canonical_inputs(facts)?;
    }
    Ok(owner.kind)
}
