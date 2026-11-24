//! An imported abstract callback needs a paid defining effect on its actual receiver.
use super::Work;
use crate::{
    body::ir::{Action, Body, Owner, Rvalue, Statement},
    resolve::expansion::Report,
    sema::model::Declarations,
    support::{
        call_traits::CallTrait,
        crates::generics::source::{CallContext, EnvironmentOrigin},
    },
    types::{
        Kind, TypeId,
        arguments::Argument,
        callable::Convention,
        instance::Instance,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};

pub(super) fn check(
    report: Option<&Report>,
    facts: &Declarations,
    body: &Body,
    statement: &Statement,
    work: &mut Work,
) -> Result<bool, &'static str> {
    work.charge(10)?;
    let Owner::Function {
        definition,
        signature: current,
        execution,
    } = body.owner
    else {
        return Ok(false);
    };
    if !facts.imported_iterator_fold(definition)? {
        return Ok(false);
    }
    if !body.const_context {
        return Err("imported const invocation requires its actual const body context");
    }
    let view = crate::sema::crates::source_original(facts, definition, &mut work.steps)?
        .ok_or("imported const invocation lost its actual original")?;
    work.charge(0)?;
    if view.source()? != body.source
        || view.raw().context != CallContext::ConstDefinition
        || !matches!(view.inputs().environment, EnvironmentOrigin::ConstTraitDefault { member, .. }
            if member == view.raw().declaration)
    {
        return Err("imported const invocation changed its defining body or effect");
    }
    body.source.proof_site(statement.source)?;
    if let Some(instance) = body.instance {
        if execution.is_none()
            || !matches!(facts.types.kind(instance.ty()), Some(Kind::FunctionItem { item, signature, .. })
                if item.0 == definition.0 && signature == current)
        {
            return Err("imported const invocation changed its actual fold instance");
        }
        concrete(report, facts, body, statement, current, work)?;
        return Ok(true);
    }
    if execution.is_some() {
        return Err("imported const invocation changed its original fold owner");
    }
    let signature = view
        .signature()
        .ok_or("imported const invocation lost its defining signature")?;
    let Some(Kind::Function {
        parameters, result, ..
    }) = facts.types.kind(signature)
    else {
        return Err("imported const invocation changed its defining callable");
    };
    let Some([_, accumulator, callback]) = facts.types.list(parameters) else {
        return Err("imported const invocation changed its defining formal arity");
    };
    if result != *accumulator {
        return Err("imported const invocation lost its actual accumulator result");
    }
    let (inputs, _) = operands(facts, body, statement, (*callback, *accumulator), work)?;
    let callable = crate::sema::compiler_items::callable_role(facts, CallTrait::FnMut)?;
    for clause in view.environment()?.trait_clauses()? {
        work.charge(3)?;
        let Some(Predicate {
            ty,
            bound: Bound::MaybeConstTrait(application),
        }) = predicate(facts, clause)
        else {
            continue;
        };
        if ty == *callback
            && facts
                .types
                .trait_application(application)
                .is_some_and(|application| {
                    application.item.0 == callable.0
                        && facts
                            .types
                            .arguments(application.arguments)
                            .is_some_and(|mut args| {
                                args.len() == 1
                                    && matches!(args.next(), Some(Argument::Type(tuple))
                        if matches!(facts.types.kind(tuple), Some(Kind::Tuple(elements))
                            if facts.types.list(elements) == Some(inputs)))
                            })
                })
        {
            return Ok(true);
        }
    }
    Err("imported const invocation has no actual conditional callable premise")
}

fn concrete(
    report: Option<&Report>,
    facts: &Declarations,
    body: &Body,
    statement: &Statement,
    signature: TypeId,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(8)?;
    let Some(Kind::Function {
        parameters, result, ..
    }) = facts.types.kind(signature)
    else {
        return Err("imported const invocation changed its instantiated signature");
    };
    let Some([_, accumulator, callback]) = facts.types.list(parameters) else {
        return Err("imported const invocation lost its instantiated callback");
    };
    if result != *accumulator {
        return Err("imported const invocation changed its instantiated accumulator result");
    }
    let (_, signature) = operands(facts, body, statement, (*callback, *accumulator), work)?;
    let Some(Kind::Closure {
        identity,
        signature: public,
        ..
    }) = facts.types.kind(*callback)
    else {
        return Err("imported const fold requires its actual checked closure callback");
    };
    if public != signature
        || !facts.types.closure_kind(identity).is_some_and(|kind| {
            kind.implements(crate::types::closure::CallKind::from_trait(
                CallTrait::FnMut,
            ))
        })
    {
        return Err("imported const fold changed its closure signature or capability");
    }
    let source = super::super::qualified_invocations::source(
        facts,
        Instance::closure(&facts.types, *callback)?,
    )?;
    if let Some(report) = report {
        source.check_const_source(report)?;
    }
    Ok(())
}

fn operands<'a>(
    facts: &'a Declarations,
    body: &Body,
    statement: &Statement,
    (callback, accumulator): (TypeId, TypeId),
    work: &mut Work,
) -> Result<(&'a [TypeId], TypeId), &'static str> {
    let Action::Evaluate(value) = statement.action else {
        return Err("imported const invocation lacks its actual operation site");
    };
    let value = body
        .values
        .get(value.0 as usize)
        .ok_or("imported const invocation value is unknown")?;
    let Rvalue::Invoke {
        kind: CallTrait::FnMut,
        signature,
        arguments,
    } = value.kind
    else {
        return Err("imported const invocation changed its actual mutable call operation");
    };
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_: false,
        bound_regions: 0,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err("imported const invocation requires its opened safe signature");
    };
    let Some(inputs @ [initial, _]) = facts.types.list(parameters) else {
        return Err("imported const invocation changed its actual input arity");
    };
    let Some([receiver, operands @ ..]) = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
    else {
        return Err("imported const invocation lost its actual operands");
    };
    let receiver = body
        .values
        .get(receiver.0 as usize)
        .ok_or("imported const invocation receiver is unknown")?;
    if *initial != accumulator
        || result != accumulator
        || value.ty != result
        || !matches!(facts.types.kind(receiver.ty), Some(Kind::Reference { referent, mutable: true, .. })
            if referent == callback)
        || operands.len() != inputs.len()
    {
        return Err("imported const invocation changed its callback, argument or result type");
    }
    for (&operand, &expected) in operands.iter().zip(inputs) {
        work.charge(1)?;
        if body
            .values
            .get(operand.0 as usize)
            .is_none_or(|value| value.ty != expected)
        {
            return Err("imported const invocation changed its actual operand type");
        }
    }
    Ok((inputs, signature))
}

fn predicate(facts: &Declarations, clause: Clause) -> Option<Predicate> {
    match clause {
        Clause::Predicate(predicate) => Some(predicate),
        Clause::ForAll(id) => facts
            .types
            .quantified(id)
            .filter(|clause| clause.bound_regions == 0)
            .map(|clause| clause.predicate),
        Clause::UnstableFeature(_) => None,
    }
}
