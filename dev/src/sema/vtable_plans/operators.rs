//! Object invocation plans consume checked source uses before selecting runtime slots.
use super::{Callable, Method, Virtual, shapes};
use crate::{
    resolve::expansion::Report,
    sema::model::Declarations,
    support::call_traits::CallTrait,
    types::{Kind, Store, TypeId, arguments::Argument, callable::Convention},
};

#[derive(Clone, Copy)]
pub(crate) struct Operator {
    pub call: Virtual,
    pub execution: TypeId,
    pub dereferences: u32,
}

pub(crate) fn prepare_operator(
    report: &Report,
    facts: &mut Declarations,
    receiver: TypeId,
    requested: CallTrait,
    signature: TypeId,
) -> Result<(), String> {
    facts
        .objects
        .storage
        .get_or_insert_with(Default::default)
        .charge(1)?;
    if facts
        .objects
        .operator(receiver, requested, signature)
        .is_some()
    {
        return Ok(());
    }
    let (object, kind, references) = subject(&facts.types, receiver, requested)?;
    let storage = facts.objects.storage.get_or_insert_with(Default::default);
    let before = storage.erase.steps;
    let object = storage.erase.apply(&mut facts.types, object)?;
    let erased_receiver = storage.erase.apply(&mut facts.types, receiver)?;
    let erased = storage.erase.apply(&mut facts.types, signature)?;
    storage.charge(
        storage
            .erase
            .steps
            .saturating_sub(before)
            .saturating_add(references as usize),
    )?;
    let shape = shapes::prepare(report, facts, object)?;
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(erased) else {
        return Err("object invocation lacks a selected signature".into());
    };
    let tuple = facts.types.intern(Kind::Tuple(parameters))?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(tuple)])?;
    let declaration = report
        .resolution
        .builtin_traits
        .callable(kind)
        .ok_or("missing canonical callable trait")?;
    let parent = facts
        .types
        .intern_trait_application(crate::types::traits::Application {
            item: crate::types::ItemId(declaration.definition.0),
            arguments,
        })?;
    let slots = facts
        .objects
        .shape(shape)
        .ok_or("missing object invocation shape")?;
    let slot = slots
        .indices
        .get(&(Method::Callable(kind), parent))
        .copied()
        .ok_or("object invocation differs from its complete callable application")?;
    let count = facts
        .types
        .list(parameters)
        .ok_or("missing object call arguments")?
        .len();
    let execution = prepend(&mut facts.types, erased_receiver, erased)?;
    let storage = facts
        .objects
        .storage
        .as_mut()
        .ok_or("missing object invocation plans")?;
    storage.charge(count.saturating_add(1))?;
    storage.operators.insert(
        (receiver, requested, signature),
        Operator {
            call: Virtual { shape, slot },
            execution,
            // Both &dyn Fn and &&dyn Fn pass the address of the fat reference.
            dereferences: references.saturating_sub(2),
        },
    );
    Ok(())
}

fn subject(
    types: &Store,
    receiver: TypeId,
    mut kind: CallTrait,
) -> Result<(TypeId, CallTrait, u32), &'static str> {
    let mut ty = types.storage_head(receiver)?;
    let mut references = 0u32;
    if kind != CallTrait::FnOnce {
        let Some(Kind::Reference {
            referent, mutable, ..
        }) = types.kind(ty)
        else {
            return Err("borrowed object callable requires a reference receiver");
        };
        if mutable != (kind == CallTrait::FnMut) {
            return Err("object callable receiver differs from its selected permission");
        }
        references += 1;
        ty = types.storage_head(referent)?;
    }
    while let Some(Kind::Reference {
        referent, mutable, ..
    }) = types.kind(ty)
    {
        references += 1;
        if references > 1_000_000 {
            return Err("object callable receiver exceeds its work budget");
        }
        kind = kind
            .referent(mutable)
            .ok_or("object receiver lacks its delegated callable permission")?;
        ty = types.storage_head(referent)?;
    }
    if !matches!(types.kind(ty), Some(Kind::Dynamic { .. })) || kind == CallTrait::FnOnce {
        return Err("object invocation requires a borrowed dynamic callable");
    }
    Ok((ty, kind, references))
}

pub(super) fn target(facts: &mut Declarations, concrete: TypeId) -> Result<Callable, &'static str> {
    let storage = facts
        .objects
        .storage
        .as_mut()
        .ok_or("missing callable target storage")?;
    storage.charge(1)?;
    if let Some(target) = storage.callable_targets.get(&concrete) {
        return Ok(*target);
    }
    let signature = match facts.types.kind(concrete) {
        Some(Kind::Function { .. }) => concrete,
        Some(
            Kind::FunctionItem { signature, .. }
            | Kind::Closure {
                signature,
                environment: TypeId::UNIT,
                ..
            },
        ) => signature,
        _ => return Err("captured callable object requires a checked conversion adapter"),
    };
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
        return Err("callable target lacks a function signature");
    };
    let count = facts
        .types
        .list(parameters)
        .ok_or("missing callable target parameters")?
        .len();
    storage.charge(count.saturating_add(1))?;
    let pointer = facts.types.intern(Kind::Pointer {
        mutable: false,
        pointee: concrete,
    })?;
    let execution = prepend(&mut facts.types, pointer, signature)?;
    let target = Callable {
        concrete,
        signature,
        execution,
    };
    storage.callable_targets.insert(concrete, target);
    Ok(target)
}

fn prepend(types: &mut Store, receiver: TypeId, signature: TypeId) -> Result<TypeId, &'static str> {
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
        ..
    }) = types.kind(signature)
    else {
        return Err("callable object adapter requires a safe Rust signature");
    };
    let values = types
        .list(parameters)
        .ok_or("missing callable object parameters")?;
    let mut parameters = Vec::with_capacity(values.len().saturating_add(1));
    parameters.push(receiver);
    parameters.extend_from_slice(values);
    let parameters = types.intern_list(&parameters)?;
    types.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    })
}
