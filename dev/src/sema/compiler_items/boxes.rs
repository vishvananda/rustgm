//! `owned_box` makes its struct the built-in owning pointer, as rustc 1.98.0
//! does for alloc's `Box<T, A>(Unique<T>, A)`: `*b` is an owned place behind
//! the pointer that the struct's leading fields hold, moving out of it leaves
//! the allocation for the box's own destructor to free, and dropping a box
//! drops its content before that destructor runs.
use crate::{
    sema::model::Declarations,
    support::bindings::LangItem,
    types::{ItemId, Kind, Store, TypeId, aggregate::Definitions, arguments::Argument},
};

/// A box type's content: the first type argument of the declared box struct.
/// # Errors
/// Rejects a box application without a leading type argument.
pub(crate) fn content(
    types: &Store,
    definitions: &Definitions,
    ty: TypeId,
) -> Result<Option<TypeId>, &'static str> {
    let (Some(owner), Some(Kind::Adt(item, arguments))) = (definitions.owned_box(), types.kind(ty))
    else {
        return Ok(None);
    };
    if item != owner {
        return Ok(None);
    }
    let first = types.arguments(arguments).and_then(|mut arguments| {
        arguments.find(|argument| !matches!(argument, Argument::Lifetime(_)))
    });
    match first {
        Some(Argument::Type(content)) => Ok(Some(content)),
        _ => Err("owned box requires its leading content type argument"),
    }
}

/// The struct `#[lang = "owned_box"]` names, before its role completes. A
/// method's receiver shape keeps such a box whole; the completed receiver
/// chain then steps through it.
pub(in crate::sema) fn declared(facts: &Declarations) -> Option<ItemId> {
    let declaration = facts
        .source_bindings
        .as_deref()?
        .language(LangItem::OwnedBox)?;
    Some(ItemId(declaration.definition.0))
}

/// Validate the declared box struct's shape, record it as the session's
/// owning pointer and complete its role: an unpacked struct whose first
/// generic parameter is its content type and whose leading field leads to
/// the content pointer. Lowering checks that field path for each box type.
/// # Errors
/// Rejects any other declaration of the role.
pub(in crate::sema) fn complete(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
) -> Result<(), crate::resolve::package::Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    let owned_box = catalog.language(LangItem::OwnedBox);
    if let Some(declaration) = owned_box {
        catalog
            .require_provider_report(report)
            .map_err(|message| Declarations::error(report, declaration.origin, message))?;
    }
    if let Some(declaration) = catalog.language(LangItem::GlobalAlloc) {
        complete_global_allocator(report, facts, declaration)?;
    }
    let Some(declaration) = owned_box else {
        return Ok(());
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    crate::sema::header_attributes::charge_source_contract_query(facts, 8)?;
    let item = ItemId(declaration.definition.0);
    shape(facts, item).map_err(error)?;
    facts.aggregates.declare_owned_box(item).map_err(error)?;
    if !super::roles::nominal_completed(report, facts, LangItem::OwnedBox, declaration)? {
        super::roles::complete_nominal(facts, LangItem::OwnedBox, declaration)?;
    }
    Ok(())
}

fn complete_global_allocator(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
    declaration: crate::resolve::bindings::Declaration,
) -> Result<(), crate::resolve::package::Error> {
    let role = LangItem::GlobalAlloc;
    if !super::roles::nominal_completed(report, facts, role, declaration)? {
        let item = ItemId(declaration.definition.0);
        facts
            .aggregates
            .declare_global_allocator(item)
            .map_err(|message| Declarations::error(report, declaration.origin, message))?;
        super::roles::complete_nominal(facts, role, declaration)?;
    }
    Ok(())
}

fn shape(facts: &Declarations, owner: ItemId) -> Result<(), &'static str> {
    let definition = facts
        .aggregates
        .get(owner)
        .ok_or("`owned_box` declaration is pending")?;
    if definition.kind != crate::types::aggregate::Kind::Struct || definition.repr.packed.is_some()
    {
        return Err("`owned_box` lang item must be applied to an unpacked struct");
    }
    let [variant] = facts.members.variants(owner) else {
        return Err("`owned_box` lang item must be applied to a struct");
    };
    let parameters = facts
        .generics
        .parameter_ids(&facts.types, crate::resolve::model::DefId(owner.0))?;
    let first = parameters.first().and_then(|&id| facts.types.parameter(id));
    if !first.is_some_and(|parameter| {
        matches!(
            parameter.kind,
            crate::types::parameters::ParameterKind::Type { .. }
        )
    }) {
        return Err(
            "`owned_box` lang item must be applied to a struct with at least 1 generic argument",
        );
    }
    if facts.members.fields(*variant).is_none_or(<[_]>::is_empty) {
        return Err("`owned_box` struct requires its leading pointer field");
    }
    Ok(())
}
