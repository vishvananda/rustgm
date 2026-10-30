//! Source From identity only biases a live float obligation; it supplies no proof.
use crate::{
    resolve::body_origin::DeclarationOrigin,
    sema::{associated::OwnerKind, generics::Parameter, model::Declarations},
    support::bindings::LangItem,
    types::{
        ItemId, Kind, Primitive,
        infer::{Context, Domain, TermId, arguments::Argument, predicates::Predicate},
        parameters::ParameterKind,
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
    // A fixed borrowed identity/header check, including the selected mapping's
    // current Store/catalog witnesses; no rows or inference variables are added.
    match role.origin {
        DeclarationOrigin::Source(_) => {
            super::super::super::header_attributes::charge_source_contract_query(facts, 16)
        }
        DeclarationOrigin::Imported(_) => {
            crate::sema::crates::charge_source_header_query(facts, 64)
        }
    }
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
    let owner = facts
        .associated
        .owner(role.definition)
        .and_then(|id| facts.associated.get(id))
        .ok_or("source From fallback lost its actual trait owner")?;
    if role.parent.is_some()
        || !matches!(owner.kind, OwnerKind::Trait { definition, .. } if definition == role.definition)
    {
        return Err("source From fallback changed its actual declaration");
    }
    let parameters = facts
        .generics
        .parameters
        .get(&role.definition)
        .ok_or("From fallback lost its actual type formal")?;
    formal(facts, role.definition, parameters)?;
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
    match role.origin {
        DeclarationOrigin::Source(source) => {
            if owner.source != Some(source) {
                return Err("source From fallback changed its source owner");
            }
            header.inputs.source_ref()?;
        }
        DeclarationOrigin::Imported(_) => {
            facts.associated.require_imported_role(role)?;
            let imported = facts
                .associated
                .imported_owner()
                .ok_or("From fallback lost its actual imported owner")?;
            imported.conversion_role(&facts.types, role)?;
            imported.owner(*owner)?;
            let catalog = facts
                .crates
                .as_deref()
                .and_then(|linkage| linkage.origin_catalog())
                .ok_or("From fallback lost its current imported catalog")?;
            header.inputs.require_catalog(catalog, role.definition)?;
        }
    }
    super::super::headers::require_self(&facts.types, header.self_type, owner.self_definition)
}

/// Imported completion retains the supported one-type-formal contract.
/// Members, safety and constness remain ordinary declaration/call contracts.
pub(super) fn header(
    facts: &Declarations,
    header: &super::super::headers::Trait<'_>,
) -> Result<(), &'static str> {
    header.profile(&facts.types, header.flags, 1)?;
    formal(facts, header.definition, header.parameters)
}

fn formal(
    facts: &Declarations,
    definition: crate::resolve::model::DefId,
    parameters: &[Parameter],
) -> Result<(), &'static str> {
    let [parameter] = parameters else {
        return Err("From requires exactly one actual type formal");
    };
    let crate::types::arguments::Argument::Type(ty) = parameter.argument else {
        return Err("From requires an actual type formal");
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
        return Err("From type formal lost its actual parameter identity");
    };
    if !facts.types.parameter(id).is_some_and(|formal| {
        formal.owner == ItemId(definition.0)
            && formal.index == 0
            && matches!(formal.kind, ParameterKind::Type { .. })
    }) {
        return Err("From type formal belongs to another actual owner or slot");
    }
    Ok(())
}
