//! Sparse execution entries retain canonical closure identity and receiver permission.
mod borrowed;
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
    adapter: Option<crate::types::closure::Adapter>,
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
    #[must_use]
    pub const fn adapter(self) -> Option<crate::types::closure::Adapter> {
        self.adapter
    }
    pub(crate) fn validate_adapter(
        self,
        facts: &mut Declarations,
    ) -> Result<
        (
            crate::sema::closures::Body,
            crate::sema::closures::Execution,
        ),
        &'static str,
    > {
        let adapter = self
            .adapter
            .ok_or("entry does not select a receiver adapter")?;
        adapter.validate(&facts.types)?;
        if adapter.instance() != self.instance || adapter.requested() != self.requested {
            return Err("receiver adapter differs from its selected instance");
        }
        let Some(Type::Closure { identity, .. }) = facts.types.kind(self.instance.ty()) else {
            return Err("receiver adapter does not select a closure");
        };
        let source = *facts
            .closures
            .identity(identity)
            .ok_or("missing adapter source")?;
        if source
            .owner
            .is_some_and(|owner| facts.generics.has_parameters(owner))
        {
            return Err("receiver adapter requires a nongeneric original body");
        }
        let Some(Type::Function { parameters, .. }) = facts.types.kind(self.execution) else {
            return Err("receiver adapter entry has no selected signature");
        };
        let receiver = *facts
            .types
            .list(parameters)
            .and_then(|p| p.first())
            .ok_or("receiver adapter entry has no receiver")?;
        if borrowed::subject(&facts.types, receiver, self.requested)? != adapter.receiver() {
            return Err("receiver adapter entry has a different physical receiver");
        }
        let borrowed::Matched { execution, .. } =
            borrowed::execution(&mut facts.types, source, adapter)?;
        matching::execution(&facts.types, execution, self.execution)?;
        Ok((source, execution))
    }
}
#[derive(Clone, Default)]
pub struct Entries {
    values: HashMap<ValueId, Entry>,
    adapters: bool,
}
impl Entries {
    pub(crate) const fn has_adapters(&self) -> bool {
        self.adapters
    }
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
        let subject = borrowed::subject(&facts.types, receiver, kind)?;
        let Some(ty) = borrowed::leaf(&facts.types, subject)? else {
            continue;
        };
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
        let instance = Instance::closure(&facts.types, ty)?;
        let adapter = borrowed::adapter(&facts.types, instance, subject, kind)?;
        let borrowed::Matched {
            execution,
            mut equalities,
        } = if let Some(adapter) = adapter {
            borrowed::execution(&mut facts.types, source, adapter)?
        } else {
            borrowed::Matched {
                execution: validate_execution(&facts.types, source)?,
                equalities: Vec::new(),
            }
        };
        let selected = call_signature(&mut facts.types, receiver, signature, value.ty)?;
        equalities.extend(matching::execution(&facts.types, execution, selected)?);
        contract
            .obligations
            .extend(equalities.into_iter().map(|pair| Obligation {
                source: body.source,
                kind: Kind::Equality(pair),
            }));
        let index = ValueId(u32::try_from(index).map_err(|_| "too many closure call values")?);
        let entry = Entry {
            instance,
            requested: kind,
            execution: selected,
            adapter,
        };
        let entries = body
            .closure_calls
            .get_or_insert_with(|| Box::new(Entries::default()));
        entries.adapters |= entry.adapter.is_some();
        entries.values.insert(index, entry);
    }
    Ok(())
}
pub(crate) fn requested_execution(
    types: &mut crate::types::Store,
    source: crate::sema::closures::Body,
    requested: CallTrait,
) -> Result<crate::sema::closures::Execution, &'static str> {
    let mut execution = validate_execution(types, source)?;
    let required = CallKind::from_trait(requested);
    if !execution.receiver.implements(required) {
        return Err("requested closure entry exceeds its call permission");
    }
    if execution.receiver == required {
        return Ok(execution);
    }
    let Some(Type::Function {
        parameters,
        result,
        convention,
        ..
    }) = types.kind(execution.signature)
    else {
        return Err("closure execution has no callable signature");
    };
    let mut parameters = types
        .list(parameters)
        .ok_or("missing receiver adapter parameters")?
        .to_vec();
    let receiver = if required == CallKind::Once {
        execution.self_ty
    } else {
        types.intern(Type::Reference {
            referent: execution.self_ty,
            mutable: required == CallKind::Mutable,
            region: crate::types::RegionId::bound(
                0,
                u16::try_from(source.universals)
                    .map_err(|_| "adapter receiver lifetime overflow")?,
            )?,
        })?
    };
    *parameters.first_mut().ok_or("adapter has no receiver")? = receiver;
    execution.universals = source
        .universals
        .checked_add(u32::from(required != CallKind::Once))
        .ok_or("adapter universal count overflow")?;
    let parameters = types.intern_list(&parameters)?;
    execution.signature = types.intern(Type::Function {
        parameters,
        result,
        bound_regions: execution.universals,
        convention,
        unsafe_: false,
    })?;
    execution.receiver = required;
    Ok(execution)
}
pub(crate) fn validate_receiver(
    types: &crate::types::Store,
    signature: TypeId,
    self_ty: TypeId,
    adapter: crate::types::closure::Adapter,
) -> Result<(), &'static str> {
    adapter.validate(types)?;
    let Some(Type::Function {
        parameters,
        bound_regions,
        ..
    }) = types.kind(signature)
    else {
        return Err("adapter receiver has no execution signature");
    };
    let receiver = *types
        .list(parameters)
        .and_then(|inputs| inputs.first())
        .ok_or("adapter receiver has no physical input")?;
    if borrowed::subject(types, receiver, adapter.requested())? != self_ty {
        return Err("adapter execution does not retain its receiver Self");
    }
    matching::instance(types, bound_regions, self_ty, adapter.receiver())?;
    Ok(())
}
pub(crate) fn validate_instance(
    types: &crate::types::Store,
    execution: TypeId,
    instance: Instance,
) -> Result<(), &'static str> {
    let Some(Type::Function {
        parameters,
        bound_regions,
        ..
    }) = types.kind(execution)
    else {
        return Err("adapter original source has no execution signature");
    };
    let receiver = *types
        .list(parameters)
        .and_then(|p| p.first())
        .ok_or("adapter original source has no receiver")?;
    let Some(Type::Reference { referent, .. }) = types.kind(receiver) else {
        return Err("adapter original source requires a borrowed receiver");
    };
    matching::instance(types, bound_regions, referent, instance.ty())?;
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
