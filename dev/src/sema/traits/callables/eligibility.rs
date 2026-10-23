//! Callable equations carry no const permission without the exact declaration.
use super::{Context, Declarations, Predicate, infer};
use crate::{
    resolve::model::DefId,
    sema::compiler_items::CallEffect,
    support::{bindings::LangItem, call_traits::CallTrait, constness::Constness},
    types::{Kind, instance::Instance},
};

pub(in crate::sema) enum Eligibility {
    Allowed,
    Rejected,
    Pending,
    Unpaid(Instance),
    Unsupported(&'static str),
}

pub(super) fn for_goal(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Predicate,
) -> Result<Eligibility, String> {
    if goal.constness() == Some(Constness::Ordinary) {
        return Ok(Eligibility::Allowed);
    }
    actual_role(facts, goal)?;
    match context.head(&facts.types, goal.subject()).map_err(error)? {
        Some(infer::Constructor::Function { .. }) => Ok(Eligibility::Rejected),
        Some(infer::Constructor::FunctionItem(..) | infer::Constructor::FunctionItemArgs(..)) => {
            function_item(facts, context, goal)
        }
        Some(infer::Constructor::Closure(..)) => Ok(Eligibility::Unsupported(
            "source const closure eligibility is not implemented",
        )),
        None => Ok(Eligibility::Pending),
        _ => Err("const callable proof has no direct callable subject".into()),
    }
}

fn actual_role(facts: &Declarations, goal: &Predicate) -> Result<(), &'static str> {
    let (item, _) = goal
        .trait_arguments()
        .ok_or("const callable requires its actual trait application")?;
    let role = match facts
        .associated
        .callable(DefId(item.0))
        .ok_or("missing callable trait identity")?
    {
        CallTrait::FnOnce => LangItem::FnOnce,
        CallTrait::FnMut => LangItem::FnMut,
        CallTrait::Fn => LangItem::Fn,
    };
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("const callable requires actual source declarations")?
        .validated_language(role)?
        .ok_or("const callable source role is absent")?;
    if declaration.definition != DefId(item.0) {
        return Err("const callable trait differs from its validated source role");
    }
    Ok(())
}

fn function_item(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Predicate,
) -> Result<Eligibility, String> {
    let ty = match context.resolved(&mut facts.types, goal.subject()) {
        Ok(ty) => ty,
        Err(infer::Error::Unresolved | infer::Error::Normalization) => {
            return Ok(Eligibility::Pending);
        }
        Err(e) => return Err(error(e)),
    };
    let instance = Instance::new(&facts.types, ty)?;
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(ty) else {
        return Err("const callable lost its function item".into());
    };
    let definition = DefId(item.0);
    let Some(header) = facts.function_header(definition)? else {
        return Ok(Eligibility::Pending);
    };
    if header.declaration.source().is_none() {
        return Ok(Eligibility::Unsupported(
            "imported const callable eligibility is not implemented",
        ));
    }
    let Some(declared) = facts
        .definitions
        .get(definition.0 as usize)
        .copied()
        .flatten()
    else {
        return Ok(Eligibility::Pending);
    };
    if !matches!(facts.types.kind(declared), Some(Kind::FunctionItem { item: actual, signature, .. }) if actual == item && signature == header.signature)
    {
        return Err(
            "const callable declaration changed its canonical identity or signature".into(),
        );
    }
    // A const-trait member requires its actual Self/implementation const proof;
    // its signature or abstract declaration cannot supply that permission.
    if facts
        .associated
        .owner(definition)
        .and_then(|id| facts.associated.get(id))
        .is_some_and(|owner| {
            matches!(
                owner.kind,
                crate::sema::associated::OwnerKind::Trait { constant: true, .. }
            )
        })
    {
        return Ok(Eligibility::Unpaid(instance));
    }
    if !facts.is_const_function(definition) {
        return Ok(Eligibility::Rejected);
    }
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("const callable source catalog disappeared")?;
    if catalog.intrinsic_kind(definition)?.is_some() {
        return Ok(Eligibility::Unsupported(
            "source intrinsic const-callable eligibility is not implemented",
        ));
    }
    let Some(effect) = catalog.prepared_call_effect(definition)? else {
        return Ok(Eligibility::Pending);
    };
    if facts.lifetimes.callables.get(item).is_none()
        && (facts.generics.has_parameters(definition)
            || facts
                .lifetimes
                .functions
                .get(&definition)
                .is_some_and(|scheme| scheme.captures != 0 || !scheme.early.is_empty()))
    {
        return Ok(Eligibility::Pending);
    }
    if let Some(boundary) = validate_signature(facts, context, instance)? {
        return Ok(Eligibility::Unsupported(boundary));
    }
    Ok(match effect {
        CallEffect::Ordinary => Eligibility::Allowed,
        CallEffect::Conditional => Eligibility::Unpaid(instance),
    })
}

fn validate_signature(
    facts: &mut Declarations,
    context: &mut Context,
    instance: Instance,
) -> Result<Option<&'static str>, String> {
    let ty = instance.ty();
    let Some(Kind::FunctionItem {
        item, arguments, ..
    }) = facts.types.kind(ty)
    else {
        return Err("const callable signature has no full function item".into());
    };
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("const callable has invalid arguments")?
        .collect::<Vec<_>>();
    context
        .account_work(arguments.len().saturating_add(1))
        .map_err(error)?;
    let mut work = 0;
    let expected =
        crate::sema::dispatch::instantiate_callable(facts, DefId(item.0), &arguments, &mut work);
    context.account_work(work).map_err(error)?;
    let expected = expected?;
    if expected == ty {
        return Ok(None);
    }
    if facts.types.contains_projections(expected) != Some(false) {
        return Ok(Some(
            "const callable projected-signature validation is not implemented",
        ));
    }
    Err("const callable instance differs from its exact substituted declaration".into())
}

fn error(error: infer::Error) -> String {
    error.to_string()
}

#[cfg(test)]
#[path = "../../../../tests/invariants/host_calls.rs"]
mod tests;
