//! Ordinary imported impl headers supply checked destructor coverage.
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
        parameters::{
            ParameterKind,
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
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
    let effect = facts
        .associated
        .imported_implementation(&facts.types, owner, predicate)?;
    if effect != (false, ImplPolarity::Positive) {
        return Err(WIDER);
    }
    let Some((application, Constness::Ordinary)) = predicate.bound.trait_parts() else {
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
    if schema.self_type != target || schema.captures != 0 || !schema.bounds.is_empty() {
        return Err(WIDER);
    }
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
        || facts.is_const_function(method)
        || actual.universals != 1
    {
        return Err(WIDER);
    }
    receiver(facts, actual.signature, target)?;
    let coverage = coverage(facts, definition, method, item, arguments)?;
    Ok(Implementation {
        source: body.root_site(),
        self_definition: definition,
        method,
        target,
        item,
        parameters: 0,
        bounds: Vec::new(),
        coverage,
    })
}

fn coverage(
    facts: &mut Declarations,
    implementation: DefId,
    method: DefId,
    item: ItemId,
    arguments: GenericArgs,
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
    let bytes = count.saturating_mul(8 * size_of::<Argument>());
    crate::sema::crates::pay_source_headers(facts, count.saturating_mul(8), bytes)?;
    let coverage = super::coverage::target(facts, implementation, item, arguments, 0)?;
    let formals = facts
        .generics
        .parameters
        .get(&implementation)
        .map_or(&[][..], Vec::as_slice);
    let mappings = facts.types.arguments(coverage.arguments).ok_or(WIDER)?;
    for (formal, actual) in formals.iter().zip(mappings) {
        let kind = |argument| {
            let Argument::Type(ty) = argument else {
                return None;
            };
            let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
                return None;
            };
            facts.types.parameter(id).map(|parameter| parameter.kind)
        };
        let expected = kind(formal.argument);
        if !matches!(expected, Some(ParameterKind::Type { .. })) || kind(actual) != expected {
            return Err("provider Drop type formals changed their nominal kinds or sizing");
        }
    }
    sizing_premises(facts, implementation, item, coverage.arguments)?;
    Ok(coverage)
}

fn sizing_premises(
    facts: &mut Declarations,
    implementation: DefId,
    item: ItemId,
    coverage: GenericArgs,
) -> Result<(), &'static str> {
    let inputs = |facts: &Declarations, owner| {
        declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), owner)
            .map(declaration_inputs::Inputs::len)
    };
    let (required, nominal) = (
        inputs(facts, implementation)?,
        inputs(facts, DefId(item.0))?,
    );
    if required == 0 {
        return Ok(());
    }
    crate::sema::crates::pay_source_headers(
        facts,
        required.saturating_add(nominal).saturating_mul(4),
        retained::new_map::<Clause, ()>(nominal),
    )?;
    let catalog = facts.source_bindings.as_deref().ok_or(WIDER)?;
    let nominal =
        declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), DefId(item.0))?
            .iter()
            .map(|input| input.value)
            .collect::<crate::support::hash::HashSet<_>>();
    let required =
        declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), implementation)?;
    for input in required.iter() {
        let Clause::Predicate(Predicate { ty, bound }) = input.value else {
            return Err(WIDER);
        };
        let sizing_bound = match bound {
            Bound::Sized => true,
            Bound::Trait(id) => {
                let application = facts.types.trait_application(id).ok_or(WIDER)?;
                application.arguments == GenericArgs::EMPTY
                    && catalog.sizing_kind(DefId(application.item.0))?.is_some()
            }
            _ => false,
        };
        let Some(Kind::Parameter(index)) = facts.types.kind(ty) else {
            return Err(WIDER);
        };
        let parameter = facts.types.parameter(index).ok_or(WIDER)?;
        let Some(Argument::Type(ty)) = facts
            .types
            .arguments(coverage)
            .and_then(|mut arguments| arguments.nth(parameter.index as usize))
        else {
            return Err(WIDER);
        };
        if !sizing_bound
            || parameter.owner.0 != implementation.0
            || !nominal.contains(&Clause::Predicate(Predicate { ty, bound }))
        {
            return Err("provider Drop implementation adds a requirement to its nominal type");
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
    let receiver = if matches!(facts.types.kind(*receiver), Some(Kind::Adt(..))) {
        crate::sema::compiler_items::drop_pin_pointer(facts, *receiver)?
    } else {
        *receiver
    };
    if !matches!(facts.types.kind(receiver), Some(Kind::Reference { mutable: true, referent, region })
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
