//! Explicit dynamic call-trait members use the selected callable slot's ABI.
use super::{Method, ShapeId, Virtual};
use crate::{
    sema::model::Declarations,
    support::call_traits::CallTrait,
    types::{
        Kind, TypeId, arguments::Argument, callable::Convention, instance::Instance,
        traits::ApplicationId,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Member {
    pub call: Virtual,
    pub signature: TypeId,
    pub execution: TypeId,
    pub tuple: TypeId,
}

pub(super) fn prepare(
    facts: &mut Declarations,
    instance: Instance,
    kind: CallTrait,
    shape: ShapeId,
    parent: ApplicationId,
) -> Result<bool, String> {
    // Owned dyn FnOnce needs a consuming slot and an owned unsized operand.
    // A borrowed callable plan provides neither.
    if kind == CallTrait::FnOnce {
        return Ok(false);
    }
    let object = facts
        .objects
        .shape(shape)
        .ok_or("missing callable member shape")?;
    let slot = object
        .indices
        .get(&(Method::Callable(kind), parent))
        .copied()
        .ok_or("explicit callable member differs from its object application")?;
    let object = object.object;
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(instance.ty()) else {
        return Err("explicit callable member requires an actual item".into());
    };
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST_CALL,
    }) = facts.types.kind(signature)
    else {
        return Err("explicit callable member requires its rust-call signature".into());
    };
    let Some(&[receiver, tuple]) = facts.types.list(parameters) else {
        return Err("explicit callable member requires its receiver and Args tuple".into());
    };
    let Some(Kind::Reference {
        referent, mutable, ..
    }) = facts.types.kind(receiver)
    else {
        return Err("explicit callable member requires a reference receiver".into());
    };
    let storage = facts
        .objects
        .storage
        .as_mut()
        .ok_or("missing callable member storage")?;
    let before = storage.erase.steps;
    let erased_referent = storage.erase.apply(&mut facts.types, referent)?;
    let erased_tuple = storage.erase.apply(&mut facts.types, tuple)?;
    storage.charge(storage.erase.steps.saturating_sub(before).saturating_add(1))?;
    if erased_referent != object || mutable != (kind == CallTrait::FnMut) {
        return Err("explicit callable member differs from its selected receiver".into());
    }
    let application = facts
        .types
        .trait_application(parent)
        .ok_or("missing callable member application")?;
    let arguments = facts
        .types
        .arguments(application.arguments)
        .ok_or("missing callable member arguments")?;
    if !arguments.eq([Argument::Type(erased_tuple)]) {
        return Err("explicit callable member Args differ from its selected application".into());
    }
    let Some(Kind::Tuple(fields)) = facts.types.kind(tuple) else {
        return Err("explicit callable member requires a tuple Args".into());
    };
    let fields = facts
        .types
        .list(fields)
        .ok_or("missing callable tuple fields")?;
    storage.charge(fields.len())?;
    let parameters = std::iter::once(receiver)
        .chain(fields.iter().copied())
        .collect::<Vec<_>>();
    let parameters = facts.types.intern_list(&parameters)?;
    let execution = facts.types.intern(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    })?;
    facts.aggregates.prepare(&mut facts.types, tuple)?;
    storage.members.insert(
        instance,
        Member {
            call: Virtual { shape, slot },
            signature,
            execution,
            tuple,
        },
    );
    Ok(true)
}
