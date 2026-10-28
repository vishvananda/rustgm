//! Extract the actual parent application from the one completed callable order.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        GenericArgs, ItemId,
        arguments::Argument,
        parameters::{
            ParameterKind,
            predicates::{Bound, Predicate},
        },
        traits::Application,
    },
};

pub(super) fn select(
    facts: &Declarations,
    definition: DefId,
    arguments: GenericArgs,
    charge: &mut impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<Predicate, &'static str> {
    charge(12)?;
    let parent = facts
        .associated
        .trait_owner(definition)
        .ok_or("abstract const call has no actual source trait owner")?;
    let canonical = facts.const_self_predicate(parent)?;
    let Bound::Trait(application) = canonical.bound else {
        return Err("const function member lost its canonical trait application");
    };
    let prototype = facts
        .types
        .trait_application(application)
        .ok_or("const function member has no complete trait application")?;
    let prototype_arguments = facts
        .types
        .arguments(prototype.arguments)
        .ok_or("const function member has invalid parent arguments")?;
    let (template, formals) = facts
        .lifetimes
        .callables
        .registration(ItemId(definition.0))
        .ok_or("const function member has no callable formal order")?;
    let captures = facts
        .lifetimes
        .functions
        .get(&definition)
        .ok_or("const function member has no lifetime scheme")?
        .captures as usize;
    let early = template.parameters as usize;
    let mut actual = facts
        .types
        .arguments(arguments)
        .ok_or("const function member has invalid complete arguments")?;
    charge(3 + 3 * prototype_arguments.len() + 2 * actual.len())?;
    if captures > early
        || captures > prototype_arguments.len()
        || actual.len() != early + formals.len()
    {
        return Err("const function member changed its complete argument arity");
    }
    let mut parent_arguments = Vec::new();
    parent_arguments
        .try_reserve_exact(prototype_arguments.len())
        .map_err(|_| "const function member argument allocation failed")?;
    for slot in 0..early {
        let Some(argument @ Argument::Lifetime(_)) = actual.next() else {
            return Err("const function member changed an early lifetime kind");
        };
        if slot < captures {
            parent_arguments.push(argument);
        }
    }
    let Some(Argument::Type(subject)) = actual.next() else {
        return Err("const function member has no actual Self application");
    };
    let parent_count = prototype_arguments.len() - captures;
    for _ in 0..parent_count {
        let Some(argument @ (Argument::Type(_) | Argument::Const(_))) = actual.next() else {
            return Err("const function member changed a parent argument kind");
        };
        parent_arguments.push(argument);
    }
    let own = formals
        .get(1 + parent_count..)
        .ok_or("const function member lost its parent formal prefix")?;
    check_method_arguments(facts, own, actual)?;
    // Canonical lookup cannot create evidence or intern an invented premise.
    // The existing immutable environment performs an indexed exact query; no
    // unrelated clause scan is repeated at each call site.
    let arguments = canonical_arguments(&facts.types, &parent_arguments)?
        .ok_or("const function member lacks its current conditional trait premise")?;
    let application = facts
        .types
        .find_trait_application(Application {
            arguments,
            ..prototype
        })
        .ok_or("const function member lacks its current conditional trait premise")?;
    Ok(Predicate {
        ty: subject,
        bound: Bound::MaybeConstTrait(application),
    })
}

/// Match the interner's lifetime-only representation, including its empty list.
/// This lookup cannot publish a new list or create an assumption.
fn canonical_arguments(
    types: &crate::types::Store,
    arguments: &[Argument],
) -> Result<Option<GenericArgs>, &'static str> {
    if arguments.is_empty() {
        return Ok(Some(GenericArgs::EMPTY));
    }
    if arguments
        .iter()
        .all(|arg| matches!(arg, Argument::Lifetime(_)))
    {
        let mut regions = Vec::new();
        regions
            .try_reserve_exact(arguments.len())
            .map_err(|_| "conditional parent lifetime allocation failed")?;
        for &argument in arguments {
            let Argument::Lifetime(region) = argument else {
                return Err("conditional parent changed its lifetime-only representation");
            };
            regions.push(region);
        }
        return Ok(types.find_regions(&regions));
    }
    Ok(types.find_mixed_arguments(arguments))
}

/// Parent extraction must leave exactly the method's own declared slot kinds.
/// The caller prepays every formal and argument before entering this check.
fn check_method_arguments(
    facts: &Declarations,
    own: &[crate::types::parameters::ParameterId],
    mut actual: crate::types::arguments::Arguments<'_>,
) -> Result<(), &'static str> {
    for &id in own {
        let kind = facts
            .types
            .parameter(id)
            .ok_or("invalid const function member formal")?
            .kind;
        if !matches!(
            (kind, actual.next()),
            (ParameterKind::Type { .. }, Some(Argument::Type(_)))
                | (ParameterKind::Const { .. }, Some(Argument::Const(_)))
        ) {
            return Err("const function member changed a method argument kind");
        }
    }
    if actual.next().is_some() {
        return Err("const function member has extra method arguments");
    }
    Ok(())
}
