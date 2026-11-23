//! A checked declared Output premise retains the actual closure execution requirements.
use super::{Clause, Declarations, Obligations, PredicateBound, RegionId};
use crate::{
    resolve::model::DefId,
    sema::closures::{Execution, actual, condition_slots, matching},
    types::{
        Kind, TypeId, arguments::Argument, bound_uses::captures::Captures, instance::Instance,
        lifetime_proof::ScopeId, substitute::Bound,
    },
};

pub(super) fn append(
    facts: &mut Declarations,
    predicates: &[Clause],
    proof: &mut Obligations,
) -> Result<(), &'static str> {
    let mut captures = Captures::new(1);
    let mut lowering = None;
    for &clause in predicates {
        if !clause.has_free_regions(&facts.types)? {
            continue;
        }
        let Some(subject) = callable_subject(facts, clause, &mut captures, &mut lowering)? else {
            continue;
        };
        // This instance's substitution may have interned the subject's opaque
        // application only now, so its storage head is prepared before the read.
        if facts.types.contains_opaques(subject) != Some(false) {
            facts.prepare_opaque_storage()?;
        }
        let (leaf, visits) = crate::types::closure::reference_leaf(&facts.types, subject)?;
        facts.body_work.canonical_steps += visits;
        let Some(leaf) = leaf else {
            continue;
        };
        let Some(Kind::Closure { identity, .. }) = facts.types.kind(leaf) else {
            return Err("declared callable premise lost its closure leaf");
        };
        let Some(conditions) = facts.closures.conditions(identity) else {
            continue;
        };
        let source = facts
            .closures
            .identity(identity)
            .ok_or("declared callable premise has no closure source")?;
        let Some(execution) = source.execution else {
            if !conditions.is_empty() {
                return Err("declared callable premise has no checked execution");
            }
            continue;
        };
        let call = if matches!(facts.types.kind(subject), Some(Kind::Opaque(..))) {
            closed_call(&mut facts.types, clause, &mut lowering)?
        } else {
            None
        };
        // The public Output may already be concrete. The authored closure's
        // Self can still need projection evidence to establish that same type.
        if call.is_none()
            && conditions.is_empty()
            && facts.types.contains_projections(execution.self_ty) == Some(false)
            && facts.types.contains_projections(leaf) == Some(false)
        {
            continue;
        }
        let conditions = conditions.to_vec();
        facts.body_work.canonical_steps += conditions.len();
        let instance = Instance::closure(&facts.types, leaf)?;
        let execution = actual::execution(facts, instance, execution)?;
        let bindings = bind(facts, execution, leaf, call, proof)?;
        let work = condition_slots::visit(
            &conditions,
            |slot| bindings.slots.get(&slot).copied(),
            "declared callable condition has no actual source slot",
            |pair| proof.outlives.push(pair),
        )?;
        facts.body_work.canonical_steps += work;
        proof.equalities.extend(bindings.equalities);
    }
    facts.body_work.canonical_steps += captures.steps + lowering.map_or(0, |fold| fold.steps);
    Ok(())
}

fn closed_call(
    types: &mut crate::types::Store,
    clause: Clause,
    lowering: &mut Option<Bound>,
) -> Result<Option<(TypeId, TypeId)>, &'static str> {
    let Clause::Predicate(predicate) =
        crate::sema::traits::selection::vacuous(types, lowering, clause)?
    else {
        return Ok(None);
    };
    let Some(Kind::Projection(application)) = types.kind(predicate.ty) else {
        return Err("declared callable output lost its exact projection");
    };
    let PredicateBound::ProjectionEqual(result) = predicate.bound else {
        return Err("missing declared callable result");
    };
    let application = types
        .trait_application(application.trait_application)
        .ok_or("missing declared callable application")?;
    let mut arguments = types
        .arguments(application.arguments)
        .ok_or("missing declared callable inputs")?;
    let (Some(Argument::Type(inputs)), None) = (arguments.next(), arguments.next()) else {
        return Err("declared callable application requires one input tuple");
    };
    Ok(Some((inputs, result)))
}

fn bind_call(
    facts: &mut Declarations,
    mut execution: Execution,
    formal: TypeId,
    leaf: TypeId,
    (inputs, result): (TypeId, TypeId),
    (declared, proof): (&mut Obligations, &mut Obligations),
) -> Result<(matching::Bindings, usize), &'static str> {
    // The exact declared callable promise has already been checked. Open its
    // physical invocation together with Self so repeated ports share a witness.
    if facts.types.contains_projections(execution.signature) == Some(true) {
        let mut session = crate::sema::projections::concrete(declared, ScopeId::ROOT)?;
        execution.signature =
            actual::normalize_signature(facts, &mut session, execution.signature)?;
    }
    execution.self_ty = formal;
    let inputs = normalize(facts, proof, inputs)?;
    let result = normalize(facts, proof, result)?;
    matching::prepare_heads(facts, execution.signature, inputs)?;
    matching::prepare_heads(facts, formal, result)?;
    matching::bind_callable_counted(&facts.types, execution, leaf, inputs, result)
}

fn callable_subject(
    facts: &mut Declarations,
    clause: Clause,
    captures: &mut Captures,
    lowering: &mut Option<Bound>,
) -> Result<Option<TypeId>, &'static str> {
    let (predicate, quantified) = match clause {
        Clause::Predicate(predicate) => (predicate, false),
        Clause::ForAll(id) => (
            facts
                .types
                .quantified(id)
                .ok_or("missing quantified callable premise")?
                .predicate,
            true,
        ),
        Clause::UnstableFeature(_) => return Ok(None),
    };
    let Some(Kind::Projection(application)) = facts.types.kind(predicate.ty) else {
        return Ok(None);
    };
    if !matches!(predicate.bound, PredicateBound::ProjectionEqual(_))
        || facts.associated.callable_output() != Some(DefId(application.item.0))
    {
        return Ok(None);
    }
    // A higher-ranked argument may name its binder even when the closure's
    // environment is fixed. Only transport the execution of that fixed Self;
    // selection has already proved the complete quantified clause in scope.
    let subject = application.subject;
    if !quantified {
        return Ok(Some(subject));
    }
    if captures.collect(&facts.types, subject)? {
        return Ok(None);
    }
    lowering
        .get_or_insert_with(|| Bound::lower(1))
        .apply(&mut facts.types, subject)
        .map(Some)
}

fn bind(
    facts: &mut Declarations,
    execution: Execution,
    leaf: TypeId,
    call: Option<(TypeId, TypeId)>,
    proof: &mut Obligations,
) -> Result<matching::Bindings, &'static str> {
    // Formal normalization owns source slots. Retain its debts separately until
    // complete Self matching supplies their actual caller coordinates.
    let mut declared = Obligations::new(1);
    let formal = normalize(facts, &mut declared, execution.self_ty)?;
    let leaf = normalize(facts, proof, leaf)?;
    // The actual execution's substitution may have interned an opaque
    // application a capture names, such as `&mut F` for an opaque `F`, only now.
    matching::prepare_heads(facts, formal, leaf)?;
    let (bindings, work) = if let Some(call) = call {
        bind_call(facts, execution, formal, leaf, call, (&mut declared, proof))?
    } else {
        matching::bind_instance_counted(&facts.types, execution.universals, formal, leaf)?
    };
    facts.body_work.canonical_steps += work;
    if !declared.is_empty() {
        let work = declared.rebase(&mut facts.types, proof.regions.end)?;
        let mapping = bindings
            .slots
            .iter()
            .map(|(&slot, &actual)| Ok((RegionId::bound(0, slot)?, actual)))
            .collect::<Result<_, &'static str>>()?;
        let mut restore = Bound::restore_owned_regions(mapping, declared.regions.clone());
        declared.substitute(&mut facts.types, &mut restore)?;
        facts.body_work.canonical_steps += work + restore.steps;
        proof.extend(declared)?;
    }
    Ok(bindings)
}

fn normalize(
    facts: &mut Declarations,
    proof: &mut Obligations,
    ty: TypeId,
) -> Result<TypeId, &'static str> {
    if facts.types.contains_projections(ty) == Some(false) {
        return Ok(ty);
    }
    let (ty, work) =
        crate::sema::projections::concrete(proof, ScopeId::ROOT)?.normalize(facts, ty)?;
    facts.body_work.canonical_steps += work;
    Ok(ty)
}
