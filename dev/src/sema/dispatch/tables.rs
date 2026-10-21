//! Vtable entries instantiate checked declarations after the conversion proof.
use super::{Dispatcher, arguments};
use crate::{
    resolve::model::DefId,
    sema::{model::Declarations, vtable_plans::Slot},
    types::{
        ItemId, Kind, RegionId, TypeId, arguments::Argument, instance::Instance, outlives::Queries,
        parameters::argument_id, relations::Constraints,
    },
};
use std::collections::HashMap;

pub(crate) fn table_method(
    facts: &mut Declarations,
    concrete: TypeId,
    slot: Slot,
) -> Result<Option<Instance>, String> {
    let crate::sema::vtable_plans::Method::Declared(method) = slot.method else {
        return Err("language callable requires its checked execution adapter".into());
    };
    let arguments = bindings(facts, concrete, slot)?;
    let source = arguments::instantiate(facts, method, &arguments)?;
    let Some(conditions) = conditions(facts, method, source)? else {
        return Ok(None);
    };
    let mut dispatch = Dispatcher::new(0, 1)?;
    let target = dispatch.select(facts, source)?;
    let proof = dispatch.finish();
    let mut restore = proof.body_restoration()?;
    let target = restore.apply(&mut facts.types, target)?;
    // Conditions and dispatch own separate private witness namespaces. Each
    // closes independently before the selected instance may be demanded.
    certify(facts, &conditions)?;
    certify(facts, &proof)?;
    Instance::new(&facts.types, target)
        .map(Some)
        .map_err(Into::into)
}

fn conditions(
    facts: &mut Declarations,
    method: DefId,
    source: TypeId,
) -> Result<Option<super::Obligations>, String> {
    let predicates = facts.generics.declared_predicates(method).to_vec();
    if predicates.is_empty() {
        return Ok(Some(super::Obligations::new(1)));
    }
    let mut substitution = arguments::Arguments::new(facts, source)?.substitution(facts)?;
    let mut state = crate::sema::nominals::traits::State::instance(facts, 0)?;
    for predicate in predicates {
        let predicate = substitution.clause(&mut facts.types, predicate.value)?;
        if !state.prove_clause(facts, predicate, true)? {
            return Ok(None);
        }
    }
    Ok(Some(state.finish()))
}

fn certify(facts: &Declarations, proof: &super::Obligations) -> Result<(), String> {
    if proof.is_empty() {
        return Ok(());
    }
    let mut constraints = Constraints::new(&facts.types, &facts.lifetimes.variances, 0, 1_000_000)
        .map_err(|e| e.to_string())?;
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    proof.require(&mut constraints, &mut queries)?;
    constraints
        .solve()
        .map(|_| ())
        .map_err(|e| format!("vtable entry lifetime requirement is not satisfied: {e}"))
}

fn bindings(
    facts: &Declarations,
    concrete: TypeId,
    slot: Slot,
) -> Result<Vec<Argument>, &'static str> {
    let application = facts
        .types
        .trait_application(slot.parent)
        .ok_or("missing vtable entry parent")?;
    let owner = DefId(application.item.0);
    let self_type = facts
        .associated
        .schema(owner)
        .ok_or("missing vtable entry owner schema")?
        .self_type;
    let Some(Kind::Parameter(self_id)) = facts.types.kind(self_type) else {
        return Err("vtable entry owner lacks its Self parameter");
    };
    let formal = facts
        .generics
        .parameters
        .get(&owner)
        .ok_or("missing vtable owner parameters")?;
    let actual = facts
        .types
        .arguments(application.arguments)
        .ok_or("invalid vtable owner arguments")?;
    if formal.len() != actual.len() {
        return Err("vtable parent argument arity mismatch");
    }
    let mut bindings = HashMap::from([(self_id, Argument::Type(concrete))]);
    for (parameter, argument) in formal.iter().zip(actual) {
        if !matches!(parameter.argument, Argument::Lifetime(_)) {
            bindings.insert(argument_id(&facts.types, parameter.argument)?, argument);
        }
    }
    let crate::sema::vtable_plans::Method::Declared(method) = slot.method else {
        return Err("language callable has no declared method bindings");
    };
    let item = ItemId(method.0);
    let template = facts
        .lifetimes
        .callables
        .get(item)
        .ok_or("missing vtable callable template")?;
    // Source universals were proved before this runtime plan. Callable-local
    // late binders remain in the template; only its captured/early inputs erase.
    let mut arguments = vec![Argument::Lifetime(RegionId::STATIC); template.parameters as usize];
    for id in facts
        .lifetimes
        .callables
        .generic_parameters(item)
        .ok_or("missing vtable callable parameters")?
    {
        arguments.push(*bindings.get(id).ok_or("unbound vtable method parameter")?);
    }
    Ok(arguments)
}
