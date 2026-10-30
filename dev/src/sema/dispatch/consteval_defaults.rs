//! Const default entry selects a real const impl, independently of runtime dispatch.
use super::{Declarations, Dispatcher, arguments::Arguments};
use crate::{
    resolve::model::DefId,
    support::retained,
    types::{
        Kind, RegionId,
        arguments::Argument,
        instance::Instance,
        parameters::{ParameterId, predicates::Bound},
    },
};

pub(crate) fn check_const_default(
    facts: &mut Declarations,
    instance: Instance,
) -> Result<(), &'static str> {
    let Some(Kind::FunctionItem {
        item, arguments, ..
    }) = facts.types.kind(instance.ty())
    else {
        return Err("const default lost its actual function item");
    };
    let definition = DefId(item.0);
    let owner = facts
        .associated
        .trait_owner(definition)
        .ok_or("const default lost its trait owner")?;
    let count = facts
        .types
        .arguments(arguments)
        .ok_or("const default lost its captured arguments")?
        .len();
    // Reconstruction, the Self goal and method predicates each use the existing
    // captured fold. Prepay their selected argument lists/maps before copying.
    let temporary = retained::new_map::<ParameterId, Argument>(count)
        .saturating_add(count.saturating_mul(
            size_of::<Argument>() + size_of::<(ParameterId, Argument)>() + size_of::<RegionId>(),
        ))
        .saturating_mul(3);
    facts.charge_caller_inputs(count.saturating_mul(4).saturating_add(16), temporary)?;
    if facts.types.contains_parameters(instance.ty()) != Some(false) {
        return Err("const default execution requires its concrete selected instance");
    }
    let values = facts
        .types
        .arguments(arguments)
        .ok_or("const default lost its captured arguments")?
        .collect::<Vec<_>>();
    let mut work = 0;
    let reconstructed = super::instantiate_callable(facts, definition, &values, &mut work);
    facts.charge_caller_inputs(work, 0)?;
    if reconstructed? != instance.ty() {
        return Err("const default changed its authenticated public signature or captures");
    }
    let arguments = Arguments::new(facts, instance.ty())?;
    let mut predicate = arguments.goal(facts, owner)?;
    let Bound::Trait(application) = predicate.bound else {
        return Err("const default lost its actual trait application");
    };
    predicate.bound = Bound::ConstTrait(application);
    let mut dispatch = Dispatcher::new(0, 1)?;
    let selected = dispatch.implementation(facts, predicate);
    facts.charge_caller_inputs(dispatch.context.steps.saturating_add(1), 0)?;
    let (implementation, arguments, proof) =
        selected.map_err(|_| "const default requires a completed concrete const implementation")?;
    if !proof.is_empty() || dispatch.provisional_size {
        return Err("const default execution has unresolved lifetime or size obligations");
    }
    facts.charge_caller_inputs(4, 0)?;
    let selected =
        crate::sema::specialization::selected_method(facts, implementation, arguments, definition)?;
    if selected.definition() != definition {
        return Err("const default call selected a different implementation method");
    }
    super::effect_requirements::check(&mut dispatch, facts, instance)?;
    Ok(())
}
