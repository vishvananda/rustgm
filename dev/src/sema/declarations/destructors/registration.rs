//! Imported impl headers supply the destructor's exact declaration contract.
use crate::{
    resolve::{body_origin::BodyOrigin, model::DefId},
    sema::{
        associated::{ImplPolarity, OwnerKind},
        declaration_inputs,
        destructors::Implementation,
        model::Declarations,
    },
    support::{bindings::LangItem, constness::Constness, retained},
    types::{
        GenericArgs, ItemId, Kind, TypeId, aggregate,
        arguments::Argument,
        callable::Convention,
        parameters::{ParameterKind, predicates::Predicate},
    },
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
        .saturating_mul(4 * (size_of::<Implementation>() + size_of::<(DefId, Predicate)>()))
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
    let headers = registry
        .imported_headers(item)
        .map(|header| (header.owner, header.predicate))
        .collect::<Vec<_>>();
    let mut prepared = Vec::with_capacity(count);
    for (owner, predicate) in headers {
        let implementation = prepare(facts, owner, predicate, drop)?;
        if let Some(copy) = copy
            && facts
                .generics
                .implementations
                .as_deref()
                .ok_or(WIDER)?
                .imported_impl_for_nominal(copy, implementation.item)
        {
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
    facts: &mut Declarations,
    definition: DefId,
    predicate: Predicate,
    drop: DefId,
) -> Result<Implementation, &'static str> {
    let owner_id = facts.associated.owner(definition).ok_or(WIDER)?;
    let owner = facts.associated.get(owner_id).copied().ok_or(WIDER)?;
    let OwnerKind::ImportedImpl(id) = owner.kind else {
        return Err(WIDER);
    };
    let (constant, polarity) =
        facts
            .associated
            .imported_implementation(&facts.types, owner, predicate)?;
    if polarity != ImplPolarity::Positive {
        return Err(WIDER);
    }
    let Some((application, constness)) = predicate.bound.trait_parts() else {
        return Err(WIDER);
    };
    if constness
        != if constant {
            Constness::Const
        } else {
            Constness::Ordinary
        }
    {
        return Err(WIDER);
    }
    let application = facts.types.trait_application(application).ok_or(WIDER)?;
    if application.item.0 != drop.0
        || facts
            .types
            .arguments(application.arguments)
            .is_none_or(|args| args.len() != 0)
    {
        return Err(WIDER);
    }
    let target = predicate.ty;
    let Some(Kind::Adt(item, arguments)) = facts.types.kind(target) else {
        return Err(WIDER);
    };
    let origins = facts
        .crates
        .as_deref()
        .and_then(|linkage| linkage.origin_catalog())
        .ok_or(WIDER)?;
    let provider = origins.declaration_artifact(definition)?.ok_or(WIDER)?;
    origins.validate_declaration(id, definition, crate::resolve::model::DefKind::SelfType)?;
    if origins.declaration_artifact(DefId(item.0))? != Some(provider)
        || facts.aggregates.get(item).is_none_or(|shape| {
            !matches!(shape.kind, aggregate::Kind::Struct | aggregate::Kind::Enum)
        })
    {
        return Err(WIDER);
    }
    let schema = facts.associated.schema(definition).ok_or(WIDER)?;
    if schema.self_type != target {
        return Err(WIDER);
    }
    let captures = schema.captures;
    let [method] = facts.associated.owner_members(owner_id).ok_or(WIDER)? else {
        return Err(WIDER);
    };
    let method = *method;
    let actual = facts.declaration_header(method)?.ok_or(WIDER)?;
    let Some(body @ BodyOrigin::Imported(_)) = actual.body else {
        return Err(WIDER);
    };
    if actual.declaration.source().is_some()
        || origins.declaration_artifact(method)? != Some(provider)
        || facts.is_const_function(method) != constant
        || captures.checked_add(1) != Some(actual.universals)
    {
        return Err(WIDER);
    }
    receiver(facts, actual.signature, target, captures)?;
    let bound_count = schema.bounds.len();
    crate::sema::crates::pay_source_headers(
        facts,
        bound_count,
        bound_count.saturating_mul(size_of::<crate::types::outlives::Pair>()),
    )?;
    let bounds = facts
        .associated
        .schema(definition)
        .ok_or(WIDER)?
        .bounds
        .clone();
    let coverage = coverage(facts, definition, method, item, arguments, captures)?;
    Ok(Implementation {
        source: body.root_site(),
        self_definition: definition,
        method,
        target,
        item,
        parameters: captures,
        bounds,
        coverage,
    })
}

fn coverage(
    facts: &mut Declarations,
    implementation: DefId,
    method: DefId,
    item: ItemId,
    arguments: GenericArgs,
    captures: u32,
) -> Result<crate::sema::destructors::Coverage, &'static str> {
    if declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), method)?.len()
        != 0
        || facts
            .generics
            .parameters
            .get(&method)
            .is_some_and(|parameters| !parameters.is_empty())
    {
        return Err("provider Drop method introduces an independent generic premise");
    }
    let count = facts
        .generics
        .parameters
        .get(&implementation)
        .map_or(0, Vec::len);
    let count = count.saturating_add(captures as usize);
    let bytes = count.saturating_mul(8 * size_of::<Argument>());
    crate::sema::crates::pay_source_headers(facts, count.saturating_mul(8), bytes)?;
    let coverage = super::coverage::target(facts, implementation, item, arguments, captures)?;
    let formals = facts
        .generics
        .parameters
        .get(&implementation)
        .map_or(&[][..], Vec::as_slice);
    let mappings = facts.types.arguments(coverage.arguments).ok_or(WIDER)?;
    for (formal, actual) in formals.iter().zip(mappings) {
        if matches!(
            (formal.argument, actual),
            (Argument::Lifetime(_), Argument::Lifetime(_))
        ) {
            continue;
        }
        let kind = |argument| {
            let id = crate::sema::generics::parameter_id(&facts.types, argument)?;
            facts
                .types
                .parameter(id)
                .map(|parameter| parameter.kind)
                .ok_or(WIDER)
        };
        let expected = kind(formal.argument)?;
        let actual = kind(actual)?;
        if !matches!(
            (expected, actual),
            (ParameterKind::Type { .. }, ParameterKind::Type { .. })
        ) && actual != expected
        {
            return Err("provider Drop formals changed their nominal kinds");
        }
    }
    Ok(coverage)
}

fn receiver(
    facts: &Declarations,
    signature: TypeId,
    target: TypeId,
    captures: u32,
) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err(WIDER);
    };
    let receiver_slot = u16::try_from(captures).map_err(|_| WIDER)?;
    if captures.checked_add(1) != Some(bound_regions) {
        return Err(WIDER);
    }
    let Some([receiver]) = facts.types.list(parameters) else {
        return Err(WIDER);
    };
    let receiver = if matches!(facts.types.kind(*receiver), Some(Kind::Adt(..))) {
        crate::sema::compiler_items::drop_pin_pointer(facts, *receiver)?
    } else {
        *receiver
    };
    if !matches!(facts.types.kind(receiver), Some(Kind::Reference { mutable: true, referent, region })
        if referent == target && region.bound_parts() == Some((0, receiver_slot)))
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
    let registry = facts.generics.implementations.as_deref().ok_or(WIDER)?;
    for implementation in rows {
        let owner = implementation.self_definition;
        if !registry.is_validated(owner) {
            return Err("provider destructor lost its checked Drop implementation");
        }
        let header = registry.header(owner).ok_or(WIDER)?;
        let mut selected = None;
        for &(member, actual) in &header.methods {
            if facts.associated.trait_owner(member) != Some(drop.definition)
                || actual.is_some_and(|actual| selected.replace(actual).is_some())
            {
                return Err("provider destructor changed its actual Drop declaration or override");
            }
        }
        if selected != Some(implementation.method) {
            return Err("provider destructor lost its checked Drop member mapping");
        }
    }
    Ok(())
}
