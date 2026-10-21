//! Reference Self keeps its own ABI and lifetime slots beside the source closure contract.
use super::{CallKind, CallTrait, Type, TypeId, matching};
use crate::{
    sema::closures::{Body, Execution},
    types::{RegionId, Store, closure::Adapter},
};
pub(super) fn subject(
    types: &Store,
    receiver: TypeId,
    requested: CallTrait,
) -> Result<TypeId, &'static str> {
    if requested == CallTrait::FnOnce {
        return Ok(receiver);
    }
    let Some(Type::Reference {
        referent, mutable, ..
    }) = types.kind(receiver)
    else {
        return Err("borrowed callable receiver is not a reference");
    };
    if mutable != (requested == CallTrait::FnMut) {
        return Err("callable receiver differs from its requested borrow capability");
    }
    Ok(referent)
}
pub(super) fn leaf(types: &Store, mut subject: TypeId) -> Result<Option<TypeId>, &'static str> {
    let mut references = 0usize;
    while let Some(Type::Reference { referent, .. }) = types.kind(subject) {
        references += 1;
        if references > 1_000_000 {
            return Err("callable reference path exceeds its work budget");
        }
        subject = referent;
    }
    let Some(Type::Closure { environment, .. }) = types.kind(subject) else {
        return Ok(None);
    };
    if references > 1 && environment != TypeId::UNIT {
        return Err("nested reference receiver adapters are not implemented");
    }
    Ok(Some(subject))
}
pub(super) fn adapter(
    types: &Store,
    instance: crate::types::instance::Instance,
    subject: TypeId,
    requested: CallTrait,
) -> Result<Option<Adapter>, &'static str> {
    let Some(Type::Closure { identity, .. }) = types.kind(instance.ty()) else {
        return Err("callable adapter has no leaf closure");
    };
    let strongest = types
        .closure_kind(identity)
        .ok_or("captured closure permission is unresolved")?;
    if subject != instance.ty() {
        Adapter::for_receiver(types, instance, subject, requested).map(Some)
    } else if strongest == CallKind::from_trait(requested) {
        Ok(None)
    } else {
        Adapter::new(types, instance, requested).map(Some)
    }
}
pub(super) struct Matched {
    pub execution: Execution,
    pub equalities: Vec<crate::types::outlives::Pair>,
}
pub(super) fn execution(
    types: &mut Store,
    source: Body,
    adapter: Adapter,
) -> Result<Matched, &'static str> {
    adapter.validate(types)?;
    if adapter.receiver() == adapter.instance().ty() {
        return Ok(Matched {
            execution: super::requested_execution(types, source, adapter.requested())?,
            equalities: Vec::new(),
        });
    }
    let canonical = super::validate_execution(types, source)?;
    let Some(Type::Reference {
        referent,
        mutable,
        region,
    }) = types.kind(adapter.receiver())
    else {
        return Err("reference adapter has no reference Self");
    };
    if referent != adapter.instance().ty() {
        return Err("reference adapter selects another closure Self");
    }
    let prefix = matching::bind_instance(types, source.universals, canonical.self_ty, referent)?
        .prefix(region)?;
    let self_ty = types.intern(Type::Reference {
        referent: canonical.self_ty,
        mutable,
        region: prefix.region,
    })?;
    let receiver = CallKind::from_trait(adapter.requested());
    let physical = if receiver == CallKind::Once {
        self_ty
    } else {
        types.intern(Type::Reference {
            referent: self_ty,
            mutable: receiver == CallKind::Mutable,
            region: RegionId::bound(
                0,
                u16::try_from(prefix.universals)
                    .map_err(|_| "too many adapter receiver lifetimes")?,
            )?,
        })?
    };
    let universals = prefix
        .universals
        .checked_add(u32::from(receiver != CallKind::Once))
        .ok_or("adapter lifetime overflow")?;
    let Type::Function {
        parameters,
        result,
        convention,
        ..
    } = types
        .kind(canonical.signature)
        .ok_or("missing source execution")?
    else {
        return Err("source execution is not callable");
    };
    let mut parameters = types
        .list(parameters)
        .ok_or("missing source execution inputs")?
        .to_vec();
    *parameters
        .first_mut()
        .ok_or("source execution has no receiver")? = physical;
    let parameters = types.intern_list(&parameters)?;
    let signature = types.intern(Type::Function {
        parameters,
        result,
        convention,
        unsafe_: false,
        bound_regions: universals,
    })?;
    Ok(Matched {
        execution: Execution {
            self_ty,
            signature,
            universals,
            receiver,
        },
        equalities: prefix.equalities,
    })
}
