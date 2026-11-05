//! Actual initialized cleanup requests acquire ordinary scoped const trait proof.
use super::{Arc, Body, Contract, Declarations, Owner, Prepared, State, Work, claims};
use crate::{
    body::regions::{BuiltinRequirements, Proof},
    types::{
        GenericArgs, ItemId, TypeId,
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    },
};
pub(crate) fn select(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
    subjects: impl Iterator<Item = Result<TypeId, &'static str>>,
) -> Result<(), String> {
    let mut work = Work::new(1_000_000);
    let mut prepared = prepare(facts, body, contract, &mut work)?;
    let role = prepared.role;
    let environment = Arc::clone(&prepared.environment);
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(role.require()?.0),
        arguments: GenericArgs::EMPTY,
    })?;
    let effect = prepared.effect();
    let mut selected = Vec::new();
    let mut seen = crate::support::hash::HashSet::default();
    for subject in subjects {
        work.charge(1)?;
        let ty = subject?;
        if prepared.requirement(&facts.types, ty, &mut work).is_ok() {
            continue;
        }
        if let Some(crate::types::Kind::Adt(item, _)) = facts.types.kind(ty)
            && facts
                .aggregates
                .get(item)
                .is_some_and(|definition| definition.kind == crate::types::aggregate::Kind::Enum)
        {
            return Err("selected const enum cleanup leaves are not implemented".into());
        }
        if !seen.insert(ty) {
            continue;
        }
        if selected.len() >= crate::support::crates::MAX_COMPONENTS {
            return Err("active const cleanup request budget exceeded".into());
        }
        selected.push(ty);
    }
    if selected.is_empty() {
        return Ok(());
    }
    let first = crate::body::dispatch::copy_frontier(facts, contract)?;
    let goals = selected
        .iter()
        .map(|&ty| Predicate {
            ty,
            bound: Bound::trait_application(application, effect),
        })
        .collect::<Vec<_>>();
    let proof = crate::sema::compiler_items::prove_cleanup_requirements(
        facts,
        Some(environment),
        contract.universals,
        first,
        &goals,
    )?;
    for &ty in &selected {
        prepared
            .claims
            .select(&mut facts.types, ty, effect, &mut work)?;
    }
    let growth = crate::support::retained::Growth::vector::<Proof>(
        contract.proofs.len(),
        contract.proofs.capacity(),
        1,
    );
    work.charge(growth.work)?;
    if contract
        .retained_bytes()
        .saturating_add(growth.bytes)
        .saturating_add(proof.allocation_bytes())
        .saturating_add(prepared.claims.retained_bytes())
        .saturating_add(size_of::<State>())
        > 64 * 1024 * 1024
    {
        return Err("selected const cleanup proof exceeds storage budget".into());
    }
    contract
        .proofs
        .try_reserve(1)
        .map_err(|_| "selected const cleanup proof allocation failed")?;
    contract.proofs.push(Proof::new(body.source, proof));
    let builtins = Arc::make_mut(
        contract
            .builtins
            .get_or_insert_with(|| Arc::new(BuiltinRequirements::empty())),
    );
    builtins.set_defining(Some(Arc::new(State::Prepared(prepared))));
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work.steps);
    Ok(())
}

fn prepare(
    facts: &Declarations,
    body: &Body,
    contract: &Contract,
    work: &mut Work,
) -> Result<Prepared, String> {
    let Owner::Function {
        definition,
        signature,
        execution: None,
    } = body.owner
    else {
        return Err("const cleanup selection requires its actual source function".into());
    };
    if body.instance.is_some() {
        return Err("const cleanup specialization requires its original defining proof".into());
    }
    let default = facts
        .associated
        .trait_owner(definition)
        .is_some_and(|owner| facts.associated.constant_trait(owner) == Some(true));
    let actual = super::active_definition(facts, definition, body.source, &mut 0, default)?
        .ok_or("active const cleanup lost its actual source definition")?;
    super::validate_contract(actual, contract)?;
    if signature != actual.signature || contract.signature != Some(signature) {
        return Err("active const cleanup changed its actual signature".into());
    }
    let builtins = contract
        .builtins
        .as_deref()
        .ok_or("active const cleanup has no actual source policy")?;
    let environment = Arc::clone(
        builtins
            .environment
            .as_ref()
            .ok_or("active const cleanup lost its environment")?,
    );
    if let Some(expected) = facts.generics.body_environment(definition) {
        if !Arc::ptr_eq(expected, &environment) {
            return Err("active const cleanup changed its defining environment".into());
        }
    } else if facts.generics.caller_scope_pending(definition)
        || !contract.parameters.is_empty()
        || !contract.type_assumptions.is_empty()
    {
        return Err("active const cleanup requires its completed defining environment".into());
    }
    let role = claims::Role::current(facts, crate::support::bindings::LangItem::Destruct, work)?;
    let copy_role = claims::Role::current(facts, crate::support::bindings::LangItem::Copy, work)?;
    let prepared = Prepared {
        definition: actual,
        owned_signature: signature,
        const_context: body.const_context,
        environment: Arc::clone(&environment),
        role,
        copy_role,
        claims: claims::Claims::new(facts, &environment, role, copy_role, work)?,
    };
    Ok(prepared)
}
