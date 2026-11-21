//! The canonical String identity guides diagnostics for failed concatenations.
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
    types::{
        ItemId, Kind, Primitive, Store, TypeId, arguments::Argument,
        parameters::predicates::Predicate,
    },
};

pub(in crate::sema) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let role = LangItem::StringType;
    let Some(declaration) = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.language(role))
    else {
        return Ok(());
    };
    if super::roles::nominal_completed(report, facts, role, declaration)? {
        return Ok(());
    }
    let definition = facts.aggregates.get(ItemId(declaration.definition.0));
    if !definition
        .is_some_and(|definition| definition.kind == crate::types::aggregate::Kind::Struct)
    {
        return Err(Declarations::error(
            report,
            declaration.origin,
            "the String role requires a checked struct",
        ));
    }
    super::roles::complete_nominal(facts, role, declaration)
}

pub(in crate::sema) fn addition(facts: &Declarations, item: ItemId) -> Option<ItemId> {
    let catalog = facts.source_bindings.as_deref()?;
    if catalog.language(LangItem::Add)?.definition.0 != item.0 {
        return None;
    }
    identity(facts)
}

fn identity(facts: &Declarations) -> Option<ItemId> {
    let string = facts
        .source_bindings
        .as_deref()?
        .validated_language(LangItem::StringType)
        .ok()
        .flatten()?;
    Some(ItemId(string.definition.0))
}

pub(in crate::sema) fn borrowed_operands(facts: &Declarations, predicate: Predicate) -> bool {
    let Some(application) = predicate
        .bound
        .trait_parts()
        .and_then(|(id, _)| facts.types.trait_application(id))
    else {
        return false;
    };
    let Some(string) = addition(facts, application.item) else {
        return false;
    };
    let Some(Argument::Type(right)) = facts
        .types
        .arguments(application.arguments)
        .and_then(|mut arguments| arguments.next())
    else {
        return false;
    };
    borrowed(&facts.types, predicate.ty, string) && borrowed(&facts.types, right, string)
}

pub(in crate::sema) fn borrowed_types(facts: &Declarations, left: TypeId, right: TypeId) -> bool {
    identity(facts).is_some_and(|string| {
        borrowed(&facts.types, left, string) && borrowed(&facts.types, right, string)
    })
}

fn borrowed(types: &Store, ty: TypeId, string: ItemId) -> bool {
    let Some(Kind::Reference { referent, .. }) = types.kind(ty) else {
        return false;
    };
    match types.kind(referent) {
        Some(Kind::Primitive(Primitive::Str)) => true,
        Some(Kind::Adt(item, _)) => item == string,
        _ => false,
    }
}
