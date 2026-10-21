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
    if arguments
        .iter()
        .any(|arg| matches!(arg, Argument::Lifetime(_)))
    {
        return Err("generic closure inherited lifetime specialization is not implemented");
    }
    if arguments.len() != parameters.len() {
        return Err("closure instance argument count differs from its enclosing template");
    }
    let bindings = parameters
        .iter()
        .copied()
        .zip(arguments)
        .collect::<Vec<_>>();
    let mut selected = Bound::parameters(&facts.types, &bindings)?;
    let expected = selected.apply(&mut facts.types, body.signature)?;
    let projected = facts.types.contains_projections(expected) == Some(true);
    if !projected && expected != signature {
        return Err("closure signature differs from substituted body template");
    }
    Ok(Inputs {
        instance,
        bindings,
        comparison: projected.then_some((expected, signature)),
    })
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
