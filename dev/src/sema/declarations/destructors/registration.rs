//! Ordinary imported impl headers supply exact closed destructor obligations.
use crate::{
    resolve::{body_origin::BodyOrigin, model::DefId},
    sema::{
        associated::{ImplPolarity, OwnerKind},
        declaration_inputs,
        destructors::{Coverage, Implementation},
        model::Declarations,
        traits::implementations::Implementation as TraitImpl,
    },
    support::{bindings::LangItem, constness::Constness, retained},
    types::{ItemId, Kind, TypeId, aggregate, callable::Convention},
};

const WIDER: &str = "provider-owned Drop implementation requires its wider destructor contract";

pub(super) fn register(facts: &mut Declarations, drop: DefId) -> Result<(), &'static str> {
    let item = ItemId(drop.0);
    let count = facts
        .generics
        .implementations
        .as_deref()
        .map_or(0, |registry| registry.imported_headers(item).count());
    if count == 0 {
        return Ok(());
    }
    let steps = count.saturating_mul(18);
    let bytes = count
        .saturating_mul(4 * size_of::<Implementation>())
        .saturating_add(retained::new_map::<DefId, usize>(count).saturating_mul(3))
        .saturating_add(retained::new_map::<ItemId, ()>(count));
    crate::sema::crates::pay_source_headers(facts, steps, bytes)?;
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(steps);
    let copy = facts
        .source_bindings
        .as_deref()
        .ok_or("imported destructors lost their actual role catalog")?
        .validated_role(LangItem::Copy)?
        .map(|role| ItemId(role.definition.0));
    let registry = facts.generics.implementations.as_deref().ok_or(WIDER)?;
    let mut prepared = Vec::with_capacity(count);
    for header in registry.imported_headers(item) {
        let implementation = prepare(facts, header, drop)?;
        if copy.is_some_and(|copy| registry.imported_impl_for_nominal(copy, implementation.item)) {
            return Err("a type cannot implement both provider Copy and Drop");
        }
        prepared.push(implementation);
    }
    for implementation in prepared {
        facts.destructors.insert(implementation)?;
    }
    Ok(())
}

fn prepare(
    facts: &Declarations,
    header: &TraitImpl,
    drop: DefId,
) -> Result<Implementation, &'static str> {
    let owner_id = facts.associated.owner(header.owner).ok_or(WIDER)?;
    let owner = facts.associated.get(owner_id).copied().ok_or(WIDER)?;
    let OwnerKind::ImportedImpl(id) = owner.kind else {
        return Err(WIDER);
    };
    let effect = facts
        .associated
        .imported_implementation(&facts.types, owner, header.predicate)?;
    if effect != (false, ImplPolarity::Positive) {
        return Err(WIDER);
    }
    let Some((application, Constness::Ordinary)) = header.predicate.bound.trait_parts() else {
        return Err(WIDER);
    };
    let application = facts.types.trait_application(application).ok_or(WIDER)?;
    if application.item.0 != drop.0
        || facts
            .types
            .arguments(application.arguments)
            .is_none_or(|args| args.len() != 0)
    {
        return Err(WIDER);
    }
    let target = header.predicate.ty;
    let Some(Kind::Adt(item, arguments)) = facts.types.kind(target) else {
        return Err(WIDER);
    };
    let origins = facts
        .crates
        .as_deref()
        .and_then(|linkage| linkage.origin_catalog())
        .ok_or(WIDER)?;
    let provider = origins.declaration_artifact(header.owner)?.ok_or(WIDER)?;
    origins.validate_declaration(id, header.owner, crate::resolve::model::DefKind::SelfType)?;
    if origins.declaration_artifact(DefId(item.0))? != Some(provider)
        || facts
            .types
            .arguments(arguments)
            .is_none_or(|args| args.len() != 0)
        || facts.definitions.get(item.0 as usize).copied().flatten() != Some(target)
        || facts.aggregates.get(item).is_none_or(|shape| {
            !matches!(shape.kind, aggregate::Kind::Struct | aggregate::Kind::Enum)
        })
    {
        return Err(WIDER);
    }
    let schema = facts.associated.schema(header.owner).ok_or(WIDER)?;
    if schema.self_type != target || schema.captures != 0 || !schema.bounds.is_empty() {
        return Err(WIDER);
    }
    let [method] = facts.associated.owner_members(owner_id).ok_or(WIDER)? else {
        return Err(WIDER);
    };
    require_closed(facts, [header.owner, *method])?;
    let actual = facts.declaration_header(*method)?.ok_or(WIDER)?;
    let Some(body @ BodyOrigin::Imported(_)) = actual.body else {
        return Err(WIDER);
    };
    if actual.declaration.source().is_some()
        || origins.declaration_artifact(*method)? != Some(provider)
        || facts.is_const_function(*method)
        || actual.universals != 1
    {
        return Err(WIDER);
    }
    receiver(facts, actual.signature, target)?;
    Ok(Implementation {
        source: body.root_site(),
        self_definition: header.owner,
        method: *method,
        target,
        item,
        parameters: 0,
        bounds: Vec::new(),
        coverage: Coverage {
            arguments,
            regions: Vec::new(),
        },
    })
}

fn require_closed(facts: &Declarations, owners: [DefId; 2]) -> Result<(), &'static str> {
    for owner in owners {
        if facts
            .generics
            .parameters
            .get(&owner)
            .is_some_and(|params| !params.is_empty())
            || declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), owner)?
                .len()
                != 0
        {
            return Err(WIDER);
        }
    }
    Ok(())
}

fn receiver(facts: &Declarations, signature: TypeId, target: TypeId) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err(WIDER);
    };
    let Some([receiver]) = facts.types.list(parameters) else {
        return Err(WIDER);
    };
    if !matches!(facts.types.kind(*receiver), Some(Kind::Reference { mutable: true, referent, region })
        if referent == target && region.bound_parts() == Some((0, 0)))
    {
        return Err(WIDER);
    }
    Ok(())
}

pub(super) fn complete(facts: &Declarations) -> Result<(), &'static str> {
    let mut rows = facts
        .destructors
        .implementations()
        .iter()
        .filter(|implementation| implementation.source.source().is_none())
        .peekable();
    if rows.peek().is_none() {
        return Ok(());
    }
    let drop = facts
        .source_bindings
        .as_deref()
        .ok_or(WIDER)?
        .validated_role(LangItem::Drop)?
        .ok_or(WIDER)?;
    let owner = facts.associated.owner(drop.definition).ok_or(WIDER)?;
    let [required] = facts.associated.owner_members(owner).ok_or(WIDER)? else {
        return Err(WIDER);
    };
    let registry = facts.generics.implementations.as_deref().ok_or(WIDER)?;
    for implementation in rows {
        if registry.method(implementation.self_definition, *required) != Some(implementation.method)
        {
            return Err("provider destructor lost its checked Drop member mapping");
        }
    }
    Ok(())
}
