//! Source From identity only biases a live float obligation; it supplies no proof.
use crate::{
    resolve::body_origin::DeclarationOrigin,
    sema::{associated::OwnerKind, model::Declarations},
    support::bindings::LangItem,
    types::{
        ItemId, Primitive,
        infer::{Context, Domain, TermId, arguments::Argument, predicates::Predicate},
    },
};
#[cfg(test)]
#[path = "../../../../tests/invariants/conversion_owners.rs"]
mod tests;

pub(crate) fn float_default(
    facts: &Declarations,
    context: &mut Context,
    predicate: &Predicate,
) -> Result<Option<TermId>, String> {
    let Some((item, [Argument::Type(argument)])) = predicate.trait_arguments() else {
        return Ok(None);
    };
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(role) = catalog.role(LangItem::From).map_err(str::to_owned)? else {
        return Ok(None);
    };
    if role.definition.0 != item.0 {
        return Ok(None);
    }
    super::super::super::header_attributes::charge_source_contract_query(facts, 16)
        .map_err(|error| error.to_string())?;
    catalog
        .validated_role(LangItem::From)
        .map_err(str::to_owned)?;
    authenticate(facts, role).map_err(str::to_owned)?;
    if !context
        .is_primitive(predicate.subject(), Primitive::F32)
        .map_err(|error| error.to_string())?
        || context
            .unresolved_domain(*argument)
            .map_err(|error| error.to_string())?
            != Some(Domain::Float)
    {
        return Ok(None);
    }
    Ok(Some(*argument))
}

fn authenticate(facts: &Declarations, role: super::super::Role) -> Result<(), &'static str> {
    let DeclarationOrigin::Source(source) = role.origin else {
        return Err("imported From fallback requires authenticated role transport");
    };
    let owner = facts
        .associated
        .owner(role.definition)
        .and_then(|id| facts.associated.get(id))
        .ok_or("source From fallback lost its actual trait owner")?;
    if role.parent.is_some()
        || owner.source != Some(source)
        || !matches!(owner.kind, OwnerKind::Trait { definition, .. } if definition == role.definition)
        || facts
            .generics
            .parameters
            .get(&role.definition)
            .is_none_or(|parameters| parameters.len() != 1)
    {
        return Err("source From fallback changed its actual declaration");
    }
    let header = facts
        .generics
        .traits
        .as_deref()
        .and_then(|headers| headers.header(ItemId(role.definition.0)))
        .ok_or("source From fallback lacks its completed canonical header")?;
    if facts
        .associated
        .schema(role.definition)
        .is_none_or(|schema| schema.self_type != header.self_type)
    {
        return Err("source From fallback changed its canonical Self");
    }
    super::super::headers::require_self(&facts.types, header.self_type, owner.self_definition)
}
