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
    let receiver = types.storage_head(receiver)?;
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
    types.storage_head(referent)
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
    let canonical = super::validate_execution(types, source)?;
    prepared(types, canonical, source.universals, adapter, false)
}
/// A specialized source keeps its obsolete receiver slot before generic slots.
pub(super) fn prepared(
    types: &mut Store,
    canonical: Execution,
    original_receiver: u32,
    adapter: Adapter,
    preserve_slots: bool,
) -> Result<Matched, &'static str> {
    adapter.validate(types)?;
    if adapter.receiver() == adapter.instance().ty() {
        return Ok(Matched {
            execution: super::shape_execution(
                types,
                canonical,
                original_receiver,
                adapter.requested(),
                preserve_slots,
            )?,
            equalities: Vec::new(),
        });
    }
    let base = if preserve_slots {
        canonical.universals
    } else {
        original_receiver
    };
    let prefix = reference_self(types, canonical.self_ty, adapter, base)?;
    let self_ty = prefix.self_ty;
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

struct Receiver {
    self_ty: TypeId,
    universals: u32,
    equalities: Vec<crate::types::outlives::Pair>,
}
fn reference_self(
    types: &mut Store,
    canonical: TypeId,
    adapter: Adapter,
    base: u32,
) -> Result<Receiver, &'static str> {
    let mut chain = Vec::new();
    let mut actual = adapter.receiver();
    while actual != adapter.instance().ty() {
        if chain.len() >= 1_000_000 {
            return Err("adapter reference path exceeds its work budget");
        }
        let Some(Type::Reference {
            referent,
            mutable,
            region,
        }) = types.kind(actual)
        else {
            return Err("reference adapter selects another closure Self");
        };
        chain.push((mutable, region));
        actual = referent;
    }
    let regions = chain.iter().map(|&(_, region)| region).collect::<Vec<_>>();
    let prefix = matching::bind_instance(types, base, canonical, actual)?.prefixes(&regions)?;
    let mut self_ty = canonical;
    for (&(mutable, _), &region) in chain.iter().zip(&prefix.regions).rev() {
        self_ty = types.intern(Type::Reference {
            referent: self_ty,
            mutable,
            region,
        })?;
    }
    Ok(Receiver {
        self_ty,
        universals: prefix.universals,
        equalities: prefix.equalities,
    })
}
