//! A transported call operator carries only its call trait, its caller-side
//! signature and its arguments. Source-bound calls re-prove the callee's
//! call-trait obligation and its declared `FnOnce::Output`. A bootstrap
//! closure call uses the remapped closure's own receiver permission and
//! callable signature.
use crate::{
    body::{
        ir::{Body, Rvalue, ValueList},
        regions::Contract,
    },
    sema::{closures::matching, model::Declarations},
    support::call_traits::CallTrait,
    types::{
        GenericArgs, Kind, TypeId,
        arguments::Argument,
        callable::Convention,
        closure::CallKind,
        parameters::{
            clauses::Clause,
            environment::Environment,
            predicates::{Bound, Predicate},
        },
        traits::Application,
    },
};
use std::sync::Arc;

struct Callable {
    callee: TypeId,
    inputs: Vec<TypeId>,
    once: crate::resolve::model::DefId,
    output: crate::resolve::model::DefId,
}
impl Callable {
    /// The `FnOnce` arguments when `ty` is `<callee as FnOnce<(inputs..)>>::Output`.
    fn output_arguments(&self, facts: &Declarations, ty: TypeId) -> Option<GenericArgs> {
        let Some(Kind::Projection(projection)) = facts.types.kind(ty) else {
            return None;
        };
        let application = facts
            .types
            .trait_application(projection.trait_application)?;
        let mut arguments = facts.types.arguments(application.arguments)?;
        let (Some(Argument::Type(tuple)), None) = (arguments.next(), arguments.next()) else {
            return None;
        };
        let Some(Kind::Tuple(elements)) = facts.types.kind(tuple) else {
            return None;
        };
        (projection.item.0 == self.output.0
            && projection.subject == self.callee
            && application.item.0 == self.once.0
            && facts.types.arguments(projection.arguments)?.len() == 0
            && facts.types.list(elements)? == self.inputs.as_slice())
        .then_some(application.arguments)
    }
}
/// Parenthesized callable sugar opens a clause binder that binds no region.
fn vacuous(facts: &Declarations, clause: Clause) -> Option<Predicate> {
    match clause {
        Clause::Predicate(predicate) => Some(predicate),
        Clause::ForAll(id) => facts
            .types
            .quantified(id)
            .filter(|scoped| scoped.bound_regions == 0)
            .map(|scoped| scoped.predicate),
        Clause::UnstableFeature(_) => None,
    }
}

/// Charged as canonical body work: one step per call operator and per
/// `Output` premise it consults. Bodies without call operators charge nothing.
pub(super) fn check(
    facts: &mut Declarations,
    body: &Body,
    contract: &Contract,
    environment: &Arc<Environment>,
) -> Result<(), &'static str> {
    let mut work = 0;
    let result = proved(facts, body, contract, environment, &mut work);
    if work != 0 {
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    }
    result
}

fn proved(
    facts: &mut Declarations,
    body: &Body,
    contract: &Contract,
    environment: &Arc<Environment>,
    work: &mut usize,
) -> Result<(), &'static str> {
    let mut trait_goals = Vec::new();
    let mut equality_goals = Vec::new();
    for value in &body.values {
        let Rvalue::Invoke {
            kind,
            signature,
            arguments,
        } = value.kind
        else {
            continue;
        };
        *work = work.saturating_add(1);
        collect_goals(
            facts,
            (body, environment),
            (kind, signature, arguments, value.ty),
            (&mut trait_goals, &mut equality_goals),
            work,
        )?;
    }
    prove_batch(
        facts,
        contract,
        environment,
        trait_goals,
        "source call operator receiver does not implement its actual call trait",
    )?;
    prove_batch(
        facts,
        contract,
        environment,
        equality_goals,
        "source call operator result is not its callee's declared Output",
    )
}

fn prove_batch(
    facts: &mut Declarations,
    contract: &Contract,
    environment: &Arc<Environment>,
    goals: Vec<Predicate>,
    error: &'static str,
) -> Result<(), &'static str> {
    if goals.is_empty() {
        return Ok(());
    }
    let first = crate::body::dispatch::copy_frontier(facts, contract)?;
    let proof = crate::sema::compiler_items::prove_body_requirements(
        facts,
        Some(Arc::clone(environment)),
        contract.universals,
        first,
        goals.into_iter(),
    )
    .map_err(|_| error)?;
    let pure_aliases =
        proof.outlives.is_empty() && proof.types.is_empty() && !proof.has_quantified();
    if !pure_aliases {
        return Err("source call operator role proof requires its wider regional profile");
    }
    Ok(())
}

/// The receiver is the callee itself for `FnOnce`, and a unique or shared
/// borrow of it for `FnMut` or `Fn`.
fn collect_goals(
    facts: &mut Declarations,
    (body, environment): (&Body, &Environment),
    (kind, signature, arguments, value_type): (CallTrait, TypeId, ValueList, TypeId),
    (trait_goals, equality_goals): (&mut Vec<Predicate>, &mut Vec<Predicate>),
    work: &mut usize,
) -> Result<(), &'static str> {
    let receiver = body
        .arguments
        .get(arguments.start as usize)
        .and_then(|value| body.values.get(value.0 as usize))
        .filter(|_| arguments.start < arguments.end)
        .ok_or("source call operator lost its actual receiver")?
        .ty;
    let callee = if kind == CallTrait::FnOnce {
        receiver
    } else {
        match facts.types.kind(receiver) {
            Some(Kind::Reference {
                mutable, referent, ..
            }) if mutable == (kind == CallTrait::FnMut) => referent,
            _ => return Err("source call operator receiver changed its call trait borrow"),
        }
    };
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("source call operator signature requires its wider regional profile");
    };
    let inputs = facts
        .types
        .list(parameters)
        .ok_or("source call operator lost its signature inputs")?;
    if inputs.len() != (arguments.end - arguments.start - 1) as usize {
        return Err("source call operator changed its actual untupled inputs");
    }
    if facts.source_bindings.is_none() {
        bootstrap_operands(body, arguments, inputs, (value_type, result), work)?;
        return bootstrap_closure(facts, callee, signature, kind, work);
    }
    let callable = Callable {
        callee,
        inputs: inputs.to_vec(),
        once: crate::sema::compiler_items::callable_role(facts, CallTrait::FnOnce)?,
        output: crate::sema::compiler_items::callable_output_role(facts)?,
    };
    let (arguments, equality) =
        output_goal(facts, environment, &callable, parameters, result, work)?;
    let role = crate::sema::compiler_items::callable_role(facts, kind)?;
    let application = facts.types.intern_trait_application(Application {
        item: crate::types::ItemId(role.0),
        arguments,
    })?;
    trait_goals.push(Predicate {
        ty: callee,
        bound: Bound::Trait(application),
    });
    if let Some(equality) = equality {
        equality_goals.push(equality);
    }
    Ok(())
}

fn bootstrap_operands(
    body: &Body,
    arguments: ValueList,
    parameters: &[TypeId],
    (value, result): (TypeId, TypeId),
    work: &mut usize,
) -> Result<(), &'static str> {
    let actual = body
        .arguments
        .get((arguments.start as usize + 1)..arguments.end as usize)
        .ok_or("bootstrap closure call lost its actual operands")?;
    *work = work.saturating_add(actual.len());
    if value != result
        || actual.iter().zip(parameters).any(|(argument, &ty)| {
            body.values
                .get(argument.0 as usize)
                .is_none_or(|arg| arg.ty != ty)
        })
    {
        return Err("bootstrap closure call changed its actual operand or result type");
    }
    Ok(())
}

fn bootstrap_closure(
    facts: &mut Declarations,
    callee: TypeId,
    signature: TypeId,
    kind: CallTrait,
    work: &mut usize,
) -> Result<(), &'static str> {
    let Some(Kind::Closure {
        identity,
        signature: declared,
        ..
    }) = facts.types.kind(callee)
    else {
        return Err("bootstrap call operator requires its actual closure receiver");
    };
    if facts.types.closure_kind(identity) != Some(CallKind::from_trait(kind)) {
        return Err("bootstrap closure call changed its actual receiver permission");
    }
    let Some(Kind::Function {
        bound_regions,
        unsafe_: false,
        convention: Convention::RUST,
        ..
    }) = facts.types.kind(declared)
    else {
        return Err("bootstrap closure lost its actual safe Rust signature");
    };
    if declared == signature {
        return Ok(());
    }
    matching::prepare_heads(facts, declared, signature)?;
    let (bindings, steps) =
        matching::signature_bindings(&facts.types, declared, bound_regions, signature, 0)?;
    *work = work.saturating_add(steps);
    if !bindings.equalities.is_empty() {
        return Err("bootstrap closure signature requires its regional proof");
    }
    Ok(())
}

/// Either the unnormalized projection, a vacuous `Output` premise, or a
/// normalization proof of `<callee as FnOnce<(inputs..)>>::Output == result`.
fn output_goal(
    facts: &mut Declarations,
    environment: &Environment,
    callable: &Callable,
    parameters: crate::types::ListId,
    result: TypeId,
    work: &mut usize,
) -> Result<(GenericArgs, Option<Predicate>), &'static str> {
    let output = crate::types::ItemId(callable.output.0);
    if let Some(arguments) = callable.output_arguments(facts, result).or_else(|| {
        environment
            .projection_candidates(output)
            .find_map(|clause| {
                *work = work.saturating_add(1);
                let predicate = vacuous(facts, clause)?;
                matches!(predicate.bound, Bound::ProjectionEqual(value) if value == result)
                    .then(|| callable.output_arguments(facts, predicate.ty))
                    .flatten()
            })
    }) {
        return Ok((arguments, None));
    }
    let has_quantified_output = environment.projection_candidates(output).any(|clause| {
        *work = work.saturating_add(1);
        let Clause::ForAll(id) = clause else {
            return false;
        };
        facts.types.quantified(id).is_some_and(|scoped| {
            scoped.bound_regions > 0
                && matches!(scoped.predicate.bound, Bound::ProjectionEqual(_))
                && matches!(
                    facts.types.kind(scoped.predicate.ty),
                    Some(Kind::Projection(projection))
                        if projection.item == output && projection.subject == callable.callee
                )
        })
    });
    let is_closure = matches!(
        facts.types.kind(callable.callee),
        Some(Kind::Closure { .. })
    );
    if !has_quantified_output && !is_closure {
        return Err("source call operator result is not its callee's declared Output");
    }
    let tuple = facts.types.intern(Kind::Tuple(parameters))?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(tuple)])?;
    let once_application = facts.types.intern_trait_application(Application {
        item: crate::types::ItemId(callable.once.0),
        arguments,
    })?;
    let empty = facts.types.intern_arguments(&[])?;
    let projection =
        facts
            .types
            .intern(Kind::Projection(crate::types::projection::Application {
                item: output,
                subject: callable.callee,
                trait_application: once_application,
                arguments: empty,
            }))?;
    Ok((
        arguments,
        Some(Predicate {
            ty: projection,
            bound: Bound::ProjectionEqual(result),
        }),
    ))
}
