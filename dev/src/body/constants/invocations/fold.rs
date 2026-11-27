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
    match body.owner {
        Owner::Function {
            definition,
            signature: current,
            execution,
        } => {
            if !facts.imported_iterator_fold(definition)? {
                return imported_const_fn(
                    report,
                    facts,
                    body,
                    statement,
                    (definition, current, execution),
                    work,
                );
            }
            imported_iterator_fold(
                report,
                facts,
                body,
                statement,
                (definition, current, execution),
                work,
            )
        }
        Owner::Closure {
            identity,
            signature: current,
            execution,
        } => imported_const_closure(
            report,
            facts,
            body,
            statement,
            (identity, current, execution),
            work,
        ),
        _ => Ok(false),
    }
}

fn imported_iterator_fold(
    report: Option<&Report>,
    facts: &Declarations,
    body: &Body,
    statement: &Statement,
    (definition, current, execution): (crate::resolve::model::DefId, TypeId, Option<TypeId>),
    work: &mut Work,
) -> Result<bool, &'static str> {
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
    if !crate::support::sealed::self_check() {
        return Ok(true);
    }
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
    check_concrete_callback(
        report,
        facts,
        *callback,
        (CallTrait::FnMut, signature),
        work,
    )
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

fn imported_const_fn(
    report: Option<&Report>,
    facts: &Declarations,
    body: &Body,
    statement: &Statement,
    (definition, current, execution): (crate::resolve::model::DefId, TypeId, Option<TypeId>),
    work: &mut Work,
) -> Result<bool, &'static str> {
    if !body.const_context {
        return Ok(false);
    }
    let Some(view) = crate::sema::crates::source_original(facts, definition, &mut work.steps)?
    else {
        return Ok(false);
    };
    work.charge(2)?;
    if !view.rows().descriptor().profile().declaration_policies() {
        return Ok(false);
    }
    if view.source()? != body.source || view.raw().context != CallContext::ConstDefinition {
        return Err("imported const invocation changed its defining body or effect");
    }
    body.source.proof_site(statement.source)?;
    let (kind, signature, subject, inputs) = general_operands(facts, body, statement, work)?;
    if let Some(instance) = body.instance {
        if execution.is_none()
            || !matches!(
                facts.types.kind(instance.ty()),
                Some(Kind::FunctionItem { item, signature, .. })
                    if item.0 == definition.0 && signature == current
            )
        {
            return Err("imported const invocation changed its actual function instance");
        }
        check_concrete_callback(report, facts, subject, (kind, signature), work)?;
        return Ok(true);
    }
    if execution.is_some() {
        return Err("imported const invocation changed its original function owner");
    }
    if !crate::support::sealed::self_check() {
        return Ok(true);
    }
    check_callable_premise(facts, view.environment()?, subject, kind, inputs, work)?;
    Ok(true)
}

fn imported_const_closure(
    report: Option<&Report>,
    facts: &Declarations,
    body: &Body,
    statement: &Statement,
    (identity, current, execution): (crate::types::closure::ClosureId, TypeId, Option<TypeId>),
    work: &mut Work,
) -> Result<bool, &'static str> {
    if !body.const_context {
        return Ok(false);
    }
    work.charge(4)?;
    if facts.closures.origin(identity) != Some(body.source) {
        return Err("imported const closure invocation changed its defining body");
    }
    body.source.proof_site(statement.source)?;
    let (kind, signature, subject, _) = general_operands(facts, body, statement, work)?;
    if let Some(instance) = body.instance {
        if execution.is_none()
            || !matches!(
                facts.types.kind(instance.ty()),
                Some(Kind::Closure {
                    identity: actual,
                    signature: actual_signature,
                    ..
                }) if actual == identity && actual_signature == current
            )
        {
            return Err("imported const closure invocation changed its actual instance");
        }
        check_concrete_callback(report, facts, subject, (kind, signature), work)?;
        return Ok(true);
    }
    if !crate::support::sealed::self_check() {
        return Ok(true);
    }
    Ok(false)
}

fn general_operands<'a>(
    facts: &'a Declarations,
    body: &Body,
    statement: &Statement,
    work: &mut Work,
) -> Result<(CallTrait, TypeId, TypeId, &'a [TypeId]), &'static str> {
    let Action::Evaluate(value) = statement.action else {
        return Err("imported const invocation lacks its actual operation site");
    };
    let value = body
        .values
        .get(value.0 as usize)
        .ok_or("imported const invocation value is unknown")?;
    let Rvalue::Invoke {
        kind,
        signature,
        arguments,
    } = value.kind
    else {
        return Err("imported const invocation changed its actual call operation");
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
    let inputs = facts
        .types
        .list(parameters)
        .ok_or("imported const invocation lost its input list")?;
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
    if value.ty != result || operands.len() != inputs.len() {
        return Err("imported const invocation changed its argument or result type");
    }
    let subject = match kind {
        CallTrait::FnOnce => receiver.ty,
        CallTrait::Fn | CallTrait::FnMut => {
            let Some(Kind::Reference {
                referent, mutable, ..
            }) = facts.types.kind(receiver.ty)
            else {
                return Err("imported const invocation lost its borrowed receiver");
            };
            if mutable != (kind == CallTrait::FnMut) {
                return Err("imported const invocation changed its receiver borrow capability");
            }
            referent
        }
    };
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
    Ok((kind, signature, subject, inputs))
}

fn check_concrete_callback(
    report: Option<&Report>,
    facts: &Declarations,
    mut callback: TypeId,
    (kind, signature): (CallTrait, TypeId),
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(8)?;
    for _ in 0..16 {
        let Some(Kind::Reference {
            referent, mutable, ..
        }) = facts.types.kind(callback)
        else {
            break;
        };
        if !mutable && kind != CallTrait::Fn {
            break;
        }
        work.charge(1)?;
        callback = referent;
    }
    match facts.types.kind(callback) {
        Some(Kind::Closure {
            identity,
            signature: public,
            ..
        }) => {
            if public != signature
                || !facts
                    .types
                    .closure_kind(identity)
                    .is_some_and(|permission| {
                        permission.implements(crate::types::closure::CallKind::from_trait(kind))
                    })
            {
                return Err(
                    "imported const invocation changed its closure signature or capability",
                );
            }
            let _ = super::super::qualified_invocations::source(
                facts,
                Instance::closure(&facts.types, callback)?,
            )?;
            if let Some(report) = report
                && !facts.closures.check_const_source(report, identity)?
            {
                return Err("non-const closures cannot be called in a const context");
            }
            Ok(())
        }
        Some(Kind::FunctionItem {
            item,
            signature: public,
            ..
        }) => {
            if public != signature
                || facts.types.needs_region_context(callback) != Some(false)
                || facts.types.contains_parameters(callback) != Some(false)
                || facts.types.contains_projections(callback) != Some(false)
                || !facts.is_const_function(crate::resolve::model::DefId(item.0))
            {
                return Err(
                    "imported const invocation requires its actual checked const function callback",
                );
            }
            Ok(())
        }
        _ => Err("imported const invocation requires its actual checked const callback"),
    }
}

fn check_callable_premise(
    facts: &Declarations,
    environment: &crate::types::parameters::environment::Environment,
    subject: TypeId,
    kind: CallTrait,
    inputs: &[TypeId],
    work: &mut Work,
) -> Result<(), &'static str> {
    let required = crate::sema::compiler_items::callable_role(facts, kind)?;
    let admissible = match kind {
        CallTrait::Fn => [Some(required), None, None],
        CallTrait::FnMut => [
            Some(required),
            crate::sema::compiler_items::callable_role(facts, CallTrait::Fn).ok(),
            None,
        ],
        CallTrait::FnOnce => [
            Some(required),
            crate::sema::compiler_items::callable_role(facts, CallTrait::FnMut).ok(),
            crate::sema::compiler_items::callable_role(facts, CallTrait::Fn).ok(),
        ],
    };
    for clause in environment.trait_clauses()? {
        work.charge(3)?;
        let Some(Predicate { ty, bound }) = predicate(facts, clause) else {
            continue;
        };
        let (Bound::MaybeConstTrait(application) | Bound::ConstTrait(application)) = bound else {
            continue;
        };
        if ty == subject
            && facts
                .types
                .trait_application(application)
                .is_some_and(|application| {
                    admissible
                        .iter()
                        .flatten()
                        .any(|role| application.item.0 == role.0)
                        && facts
                            .types
                            .arguments(application.arguments)
                            .is_some_and(|mut args| {
                                args.len() == 1
                                    && matches!(
                                        args.next(),
                                        Some(Argument::Type(tuple))
                                            if (inputs.is_empty() && tuple == TypeId::UNIT)
                                                || matches!(
                                                    facts.types.kind(tuple),
                                                    Some(Kind::Tuple(elements))
                                                        if facts.types.list(elements) == Some(inputs)
                                                )
                                    )
                            })
                })
        {
            return Ok(());
        }
    }
    Err("imported const invocation has no actual conditional callable premise")
}
