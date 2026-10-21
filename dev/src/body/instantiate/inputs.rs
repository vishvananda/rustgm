//! Select exact callable arguments without rebuilding or inferring source bodies.
use super::{Bound, Declarations};
use crate::{
    body::ir::{Body, Owner},
    resolve::model::DefId,
    types::{ItemId, Kind, TypeId, arguments::Argument, instance::Instance},
};

type Binding = (crate::types::parameters::ParameterId, Argument);
pub(super) struct Inputs {
    pub(super) instance: Instance,
    pub(super) bindings: Vec<Binding>,
    pub(super) captures: Vec<crate::types::RegionId>,
    pub(super) comparison: Option<(crate::types::TypeId, crate::types::TypeId)>,
}
pub(super) fn select(facts: &mut Declarations, instance: Instance) -> Result<Inputs, &'static str> {
    if let Some(Kind::Closure { .. }) = facts.types.kind(instance.ty()) {
        return closure(facts, instance);
    }
    let Some(Kind::FunctionItem {
        item,
        signature,
        arguments,
    }) = facts.types.kind(instance.ty())
    else {
        return Err("invalid callable instance");
    };
    let Some(callable) = facts.lifetimes.callables.get(item) else {
        let owner = crate::resolve::model::DefId(item.0);
        if arguments != crate::types::GenericArgs::EMPTY
            || facts.generics.has_parameters(owner)
            || facts.definitions.get(item.0 as usize).copied().flatten() != Some(instance.ty())
        {
            return Err("missing callable template");
        }
        return Ok(Inputs {
            instance,
            bindings: Vec::new(),
            captures: Vec::new(),
            comparison: None,
        });
    };
    let parameters = facts
        .lifetimes
        .callables
        .generic_parameters(item)
        .ok_or("missing callable parameters")?;
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("invalid callable arguments")?
        .collect::<Vec<_>>();
    let early = callable.parameters as usize;
    if arguments.len() != early.saturating_add(parameters.len()) {
        return Err("callable instance argument count differs from its template");
    }
    let mut regions = Vec::with_capacity(early);
    for argument in &arguments[..early] {
        let Argument::Lifetime(region) = *argument else {
            return Err("callable early argument is not a lifetime");
        };
        regions.push(region);
    }
    let bindings = parameters
        .iter()
        .copied()
        .zip(arguments[early..].iter().copied())
        .collect::<Vec<_>>();
    let mut selected = Bound::parameters_captured(&facts.types, &bindings, regions)?;
    let expected = selected.apply(&mut facts.types, callable.signature)?;
    let projected = facts.types.contains_projections(expected) == Some(true);
    if !projected && expected != signature {
        return Err("instance signature differs from substituted template");
    }
    Ok(Inputs {
        instance,
        bindings,
        captures: Vec::new(),
        comparison: projected.then_some((expected, signature)),
    })
}

fn closure(facts: &mut Declarations, instance: Instance) -> Result<Inputs, &'static str> {
    let Some(Kind::Closure {
        identity,
        signature,
        arguments,
        environment: TypeId::UNIT,
    }) = facts.types.kind(instance.ty())
    else {
        return Err("closure instance requires an implemented environment adapter");
    };
    let body = *facts
        .closures
        .identity(identity)
        .ok_or("missing original closure body")?;
    let owner = body.owner.ok_or("generic closure has no enclosing owner")?;
    let parameters = facts
        .lifetimes
        .callables
        .generic_parameters(ItemId(owner.0))
        .ok_or("missing enclosing closure parameters")?;
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("invalid closure arguments")?
        .collect::<Vec<_>>();
    let inherited = body.inherited as usize;
    if arguments.len() != inherited.saturating_add(parameters.len()) {
        return Err("closure instance argument count differs from its enclosing template");
    }
    let captures = arguments[..inherited]
        .iter()
        .map(|argument| match argument {
            Argument::Lifetime(region) => Ok(*region),
            _ => Err("closure inherited argument is not a lifetime"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let bindings = parameters
        .iter()
        .copied()
        .zip(arguments[inherited..].iter().copied())
        .collect::<Vec<_>>();
    let expected = inherited_signature(&mut facts.types, body, &captures, signature)?;
    let mut selected = Bound::parameters(&facts.types, &bindings)?;
    let expected = selected.apply(&mut facts.types, expected)?;
    let projected = facts.types.contains_projections(expected) == Some(true);
    if !projected && expected != signature {
        return Err("closure signature differs from substituted body template");
    }
    Ok(Inputs {
        instance,
        bindings,
        captures,
        comparison: projected.then_some((expected, signature)),
    })
}

/// Reconstruct the source call binder from the body's distinct proof universe.
fn inherited_signature(
    types: &mut crate::types::Store,
    body: crate::sema::closures::Body,
    captures: &[crate::types::RegionId],
    actual: TypeId,
) -> Result<TypeId, &'static str> {
    use crate::types::RegionId;
    if captures.is_empty() {
        return Ok(body.signature);
    }
    let Some(Kind::Function { bound_regions, .. }) = types.kind(actual) else {
        return Err("closure instance has no call signature");
    };
    if body.universals.checked_sub(body.inherited) != Some(bound_regions) {
        return Err("closure signature lifetime interface differs from its body");
    }
    let mut mapping = std::collections::HashMap::new();
    for (index, &capture) in captures.iter().enumerate() {
        let index = u16::try_from(index).map_err(|_| "too many inherited closure lifetimes")?;
        let capture = match capture.bound_parts() {
            Some((depth, slot)) => RegionId::bound(
                depth
                    .checked_add(1)
                    .ok_or("closure lifetime binder depth exhausted")?,
                slot,
            )?,
            None => capture,
        };
        mapping.insert(RegionId::bound(0, index)?, capture);
    }
    for index in 0..bound_regions {
        let source =
            u16::try_from(body.inherited + index).map_err(|_| "too many closure body lifetimes")?;
        let target = u16::try_from(index).map_err(|_| "too many closure call lifetimes")?;
        mapping.insert(RegionId::bound(0, source)?, RegionId::bound(0, target)?);
    }
    Bound::restore_regions(mapping).signature(types, body.signature, bound_regions)
}

pub(super) fn owner(
    facts: &Declarations,
    source: &Body,
    instance: Instance,
) -> Result<(Owner, DefId), &'static str> {
    if source.instance.is_some() {
        return Err("instance does not select an original body template");
    }
    match (source.owner, facts.types.kind(instance.ty())) {
        (
            Owner::Function { definition, .. },
            Some(Kind::FunctionItem {
                item, signature, ..
            }),
        ) if definition.0 == item.0 => Ok((
            Owner::Function {
                definition,
                signature,
            },
            definition,
        )),
        (
            Owner::Closure { identity, .. },
            Some(Kind::Closure {
                identity: selected,
                signature,
                ..
            }),
        ) if identity == selected => {
            let closure = facts
                .closures
                .identity(identity)
                .ok_or("missing closure template identity")?;
            if closure.body != source.source {
                return Err("closure instance selects a different source body");
            }
            let owner = closure
                .owner
                .ok_or("missing generic closure environment owner")?;
            Ok((
                Owner::Closure {
                    identity,
                    signature,
                },
                owner,
            ))
        }
        _ => Err("instance does not select its original body owner"),
    }
}
