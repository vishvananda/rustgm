//! A transported call operator carries only its call trait, its caller-side
//! signature and its arguments. Every consumer re-derives the callee from the
//! receiver, re-proves the callee's own call-trait obligation in the original's
//! environment and checks that the signature's result is that callee's
//! declared `FnOnce::Output`. No provider selection or proof is trusted.
use crate::{
    body::{
        ir::{Body, Rvalue, ValueList},
        regions::Contract,
    },
    sema::model::Declarations,
    support::call_traits::CallTrait,
    types::{
        GenericArgs, Kind, TypeId,
        arguments::Argument,
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
    let mut goals = Vec::new();
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
        goals.push(goal(
            facts,
            (body, environment),
            (kind, signature, arguments),
            work,
        )?);
    }
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
    .map_err(|_| "source call operator receiver does not implement its actual call trait")?;
    if !proof.is_empty() {
        return Err("source call operator role proof requires its wider regional profile");
    }
    Ok(())
}

/// The receiver is the callee itself for `FnOnce`, and a unique or shared
/// borrow of it for `FnMut` or `Fn`.
fn goal(
    facts: &mut Declarations,
    (body, environment): (&Body, &Environment),
    (kind, signature, arguments): (CallTrait, TypeId, ValueList),
    work: &mut usize,
) -> Result<Predicate, &'static str> {
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
        .ok_or("source call operator lost its signature inputs")?
        .to_vec();
    if inputs.len() != (arguments.end - arguments.start - 1) as usize {
        return Err("source call operator changed its actual untupled inputs");
    }
    let callable = Callable {
        callee,
        inputs,
        once: crate::sema::compiler_items::callable_role(facts, CallTrait::FnOnce)?,
        output: crate::sema::compiler_items::callable_output_role(facts)?,
    };
    // Either the unnormalized projection, or the value an environment
    // premise in the `Output` bucket binds to exactly that projection.
    let output = crate::types::ItemId(callable.output.0);
    let arguments = callable
        .output_arguments(facts, result)
        .or_else(|| {
            environment
                .projection_candidates(output)
                .find_map(|clause| {
                    *work = work.saturating_add(1);
                    let predicate = vacuous(facts, clause)?;
                    matches!(predicate.bound, Bound::ProjectionEqual(value) if value == result)
                        .then(|| callable.output_arguments(facts, predicate.ty))
                        .flatten()
                })
        })
        .ok_or("source call operator result is not its callee's declared Output")?;
    let role = crate::sema::compiler_items::callable_role(facts, kind)?;
    let application = facts.types.intern_trait_application(Application {
        item: crate::types::ItemId(role.0),
        arguments,
    })?;
    Ok(Predicate {
        ty: callee,
        bound: Bound::Trait(application),
    })
}
