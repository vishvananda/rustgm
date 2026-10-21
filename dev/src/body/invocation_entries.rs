//! Sparse execution entries retain canonical closure identity and receiver permission.
mod matching;
#[cfg(test)]
#[path = "../../tests/invariants/invocation_entries.rs"]
mod tests;
use super::{
    ir::{Body, Rvalue, ValueId},
    regions::{Contract, Kind, Obligation},
};
use crate::{
    sema::model::Declarations,
    support::call_traits::CallTrait,
    types::{Kind as Type, TypeId, closure::CallKind, instance::Instance},
};
use std::collections::HashMap;

/// Construction is private to canonical selection and complete structural matching.
#[derive(Clone, Copy)]
pub struct Entry {
    instance: Instance,
    requested: CallTrait,
    execution: TypeId,
}
impl Entry {
    #[must_use]
    pub const fn instance(self) -> Instance {
        self.instance
    }
    #[must_use]
    pub const fn requested(self) -> CallTrait {
        self.requested
    }
    #[must_use]
    pub const fn execution(self) -> TypeId {
        self.execution
    }
}
#[derive(Clone, Default)]
pub struct Entries {
    values: HashMap<ValueId, Entry>,
}
impl Entries {
    #[must_use]
    pub fn get(&self, value: ValueId) -> Option<Entry> {
        self.values.get(&value).copied()
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(crate::support::retained::map(&self.values))
    }
}
pub(crate) fn select(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
) -> Result<(), &'static str> {
    if body.closure_calls.is_some() {
        return Err("closure entries were selected twice");
    }
    for (index, value) in body.values.iter().enumerate() {
        let Rvalue::Invoke {
            kind,
            signature,
            arguments,
        } = value.kind
        else {
            continue;
        };
        let receiver = *body
            .arguments(arguments)
            .first()
            .ok_or("closure call has no receiver")?;
        let receiver = body
            .values
            .get(receiver.0 as usize)
            .ok_or("invalid closure call receiver")?
            .ty;
        let (ty, permission) = receiver_type(&facts.types, receiver)?;
        let Some(Type::Closure {
            identity,
            environment,
            ..
        }) = facts.types.kind(ty)
        else {
            continue;
        };
        if environment == TypeId::UNIT {
            continue;
        }
        let required = match kind {
            CallTrait::Fn => CallKind::Shared,
            CallTrait::FnMut => CallKind::Mutable,
            CallTrait::FnOnce => CallKind::Once,
        };
        if permission != required || facts.types.closure_kind(identity) != Some(required) {
            return Err("captured closure requires a checked receiver conversion adapter");
        }
        let source = *facts
            .closures
            .identity(identity)
            .ok_or("missing captured closure source")?;
        if source
            .owner
            .is_some_and(|owner| facts.generics.has_parameters(owner))
        {
            return Err(
                "generic captured closure execution requires environment lifetime substitution",
            );
        }
        let execution = validate_execution(&facts.types, source)?;
        if execution.receiver != required {
            return Err("closure execution permission differs from its declaration");
        }
        let selected = call_signature(&mut facts.types, receiver, signature, value.ty)?;
        let equalities = matching::execution(&facts.types, execution, selected)?;
        contract
            .obligations
            .extend(equalities.into_iter().map(|pair| Obligation {
                source: body.source,
                kind: Kind::Equality(pair),
            }));
        let index = ValueId(u32::try_from(index).map_err(|_| "too many closure call values")?);
        let entry = Entry {
            instance: Instance::closure(&facts.types, ty)?,
            requested: kind,
            execution: selected,
        };
        body.closure_calls
            .get_or_insert_with(|| Box::new(Entries::default()))
            .values
            .insert(index, entry);
    }
    Ok(())
}
pub(crate) fn validate_execution(
    types: &crate::types::Store,
    source: crate::sema::closures::Body,
) -> Result<crate::sema::closures::Execution, &'static str> {
    let execution = source
        .execution
        .ok_or("captured closure has no checked execution contract")?;
    if types.closure_kind(source.identity) != Some(execution.receiver) {
        return Err("closure execution permission differs from declaration");
    }
    let Some(Type::Closure {
        identity,
        environment,
        ..
    }) = types.kind(execution.self_ty)
    else {
        return Err("closure execution has no canonical Self");
    };
    if identity != source.identity || environment == TypeId::UNIT {
        return Err("closure execution selects a different or empty environment");
    }
    let Some(Type::Function {
        parameters,
        result,
        bound_regions,
        unsafe_: false,
        convention,
    }) = types.kind(execution.signature)
    else {
        return Err("closure execution has no safe function signature");
    };
    if bound_regions != execution.universals
        || convention != crate::types::callable::Convention::RUST
    {
        return Err("closure execution binder or convention differs");
    }
    let parameters = types
        .list(parameters)
        .ok_or("invalid closure execution parameters")?;
    let (&receiver, tail) = parameters
        .split_first()
        .ok_or("closure execution has no hidden receiver")?;
    let expected = if execution.receiver == CallKind::Once {
        source.universals
    } else {
        source
            .universals
            .checked_add(1)
            .ok_or("closure execution binder overflow")?
    };
    if expected != execution.universals {
        return Err("closure receiver lifetime is not appended after body universals");
    }
    match (execution.receiver, types.kind(receiver)) {
        (CallKind::Once, _) if receiver == execution.self_ty => {}
        (
            CallKind::Shared | CallKind::Mutable,
            Some(Type::Reference {
                referent,
                mutable,
                region,
            }),
        ) if referent == execution.self_ty
            && mutable == (execution.receiver == CallKind::Mutable)
            && region.bound_parts()
                == Some((
                    0,
                    u16::try_from(source.universals)
                        .map_err(|_| "closure receiver lifetime overflow")?,
                )) => {}
        _ => return Err("closure execution receiver differs from its permission"),
    }
    let Some(Type::Function {
        parameters: public,
        result: output,
        unsafe_: false,
        convention: cv,
        ..
    }) = types.kind(source.signature)
    else {
        return Err("closure body has no public proof signature");
    };
    if types.list(public) != Some(tail) || result != output || cv != convention {
        return Err("closure execution changes its public inputs or result");
    }
    Ok(execution)
}
fn receiver_type(
    types: &crate::types::Store,
    ty: TypeId,
) -> Result<(TypeId, CallKind), &'static str> {
    let ty = types.storage_head(ty)?;
    Ok(match types.kind(ty) {
        Some(Type::Reference {
            referent, mutable, ..
        }) => (
            types.storage_head(referent)?,
            if mutable {
                CallKind::Mutable
            } else {
                CallKind::Shared
            },
        ),
        _ => (ty, CallKind::Once),
    })
}

fn call_signature(
    types: &mut crate::types::Store,
    receiver: TypeId,
    signature: TypeId,
    output: TypeId,
) -> Result<TypeId, &'static str> {
    let Some(Type::Function {
        parameters,
        result,
        bound_regions: 0,
        unsafe_: false,
        convention,
    }) = types.kind(signature)
    else {
        return Err("closure call requires an opened safe signature");
    };
    if convention != crate::types::callable::Convention::RUST || result != output {
        return Err("closure call signature differs from its typed result");
    }
    let mut parameters = types
        .list(parameters)
        .ok_or("invalid closure call parameters")?
        .to_vec();
    parameters.insert(0, receiver);
    let parameters = types.intern_list(&parameters)?;
    types.intern(Type::Function {
        parameters,
        result,
        bound_regions: 0,
        unsafe_: false,
        convention,
    })
}
