//! Complete named values required by typed bodies before the read-only lowerer.
mod invocations;
mod provided_calls;
use super::ir::{Place, Program, Rvalue};
use crate::support::hash::HashSet;
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::{model::Declarations, values::Evaluator},
};

/// Evaluate each demanded definition once, retaining values in semantic arenas.
/// # Errors
/// Preserves required-constant diagnostics and unsupported evaluation boundaries.
pub fn analyze(report: &Report, facts: &mut Declarations, bodies: &Program) -> Result<(), Error> {
    prepare(report, facts, bodies.bodies.iter(), &[])
}
pub(crate) fn analyze_bodies<'a>(
    report: &Report,
    facts: &mut Declarations,
    bodies: impl Iterator<Item = &'a super::ir::Body>,
    globals: &[crate::resolve::model::DefId],
) -> Result<(), Error> {
    prepare(report, facts, bodies, globals)
}
/// Complete constants for one newly demanded body without rescanning the program.
/// # Errors
/// Reports the same required-constant failures as `analyze`.
pub fn instance(
    report: &Report,
    facts: &mut Declarations,
    body: &super::ir::Body,
) -> Result<(), Error> {
    prepare(report, facts, std::iter::once(body), &[])
}
fn prepare<'a>(
    report: &Report,
    facts: &mut Declarations,
    bodies: impl Iterator<Item = &'a super::ir::Body>,
    globals: &[crate::resolve::model::DefId],
) -> Result<(), Error> {
    let mut seen = HashSet::default();
    let mut demands = Vec::new();
    let mut inline_seen = HashSet::default();
    let mut inline = Vec::new();
    let mut instances = Vec::new();
    let mut instance_seen = HashSet::default();
    let mut source_seen = HashSet::default();
    let mut sources = Vec::new();
    let mut associated_seen = HashSet::default();
    let mut associated = Vec::new();
    let mut demand = |id, static_| {
        if seen.insert(id) {
            demands.push((id, static_));
        }
    };
    for &id in globals {
        demand(id, true);
    }
    for body in bodies {
        eligibility_charged(report, facts, body)?;
        if matches!(body.owner, super::ir::Owner::AnonymousConstant)
            && facts.inline_constants.as_deref().is_some_and(|registry| {
                body.source
                    .source()
                    .is_some_and(|source| registry.get(source).is_some())
            })
        {
            continue;
        }
        for statement in &body.statements {
            if let super::ir::Action::Evaluate(id) = statement.action
                && let Some(value) = body.values.get(id.0 as usize)
                && let Rvalue::InlineInstance(instance) = value.kind
                && instance_seen.insert((instance, value.ty))
            {
                instances.push((instance, value.ty));
            }
        }
        for value in &body.values {
            if let Rvalue::SourceAssociated(source) = value.kind
                && source_seen.insert(source)
            {
                sources.push(source);
            }
            if let Rvalue::InlineConstant(source) = value.kind
                && inline_seen.insert(source)
            {
                inline.push(source);
            }
            if let Rvalue::AssociatedConstant(instance) = value.kind
                && associated_seen.insert(instance)
            {
                associated.push(instance);
            }
        }
        for (id, static_) in references(body) {
            if static_
                && facts
                    .foreign
                    .as_deref()
                    .is_some_and(|r| r.imports_global(id))
            {
                continue;
            }
            demand(id, static_);
        }
    }
    if demands.is_empty()
        && associated.is_empty()
        && sources.is_empty()
        && inline.is_empty()
        && instances.is_empty()
    {
        return Ok(());
    }
    let mut evaluator = Evaluator::new(report, facts);
    for (id, static_) in demands {
        if static_ {
            evaluator.initializer(id)?;
        } else {
            evaluator.definition(id)?;
        }
    }
    for source in inline {
        evaluator.inline(source)?;
    }
    for (instance, result) in instances {
        evaluator.inline_instance(instance, result)?;
    }
    for source in sources {
        evaluator.source_associated(source)?;
    }
    for instance in associated {
        evaluator.associated(instance)?;
    }
    evaluator.complete_static_storage()?;
    Ok(())
}
pub(crate) fn eligibility(
    report: &Report,
    facts: &Declarations,
    body: &super::ir::Body,
) -> Result<(), Error> {
    eligibility_counted(report, facts, body, &mut invocations::Work::new(1_000_000))
}
/// Mutable owners retain attempted preparation work even when validation fails.
pub(crate) fn eligibility_charged(
    report: &Report,
    facts: &mut Declarations,
    body: &super::ir::Body,
) -> Result<(), Error> {
    let mut work = invocations::Work::new(1_000_000);
    let result = eligibility_counted(report, facts, body, &mut work);
    if work.steps != 0 {
        facts.body_work.canonical_steps =
            facts.body_work.canonical_steps.saturating_add(work.steps);
    }
    result
}
fn eligibility_counted(
    report: &Report,
    facts: &Declarations,
    body: &super::ir::Body,
    work: &mut invocations::Work,
) -> Result<(), Error> {
    if !body.const_context
        && !provided_calls::defining(facts, body, work)
            .map_err(|e| report.diagnostic(report.body_error(body.source, e)))?
    {
        return Ok(());
    }
    let function = matches!(body.owner, super::ir::Owner::Function { .. });
    // The CFG retains both sides of a conditional, but not source after an
    // unconditional transfer. Const permission is checked before value demand.
    let mut operators = 0;
    let mut members = 0;
    for value in &body.values {
        if matches!(value.kind, Rvalue::Invoke { .. }) {
            operators += 1;
        }
        let message = match value.kind {
            Rvalue::Intrinsic { intrinsic, .. }
                if intrinsic.descriptor().constant
                    == crate::support::intrinsics::ConstPermission::RuntimeOnly =>
            {
                Some("runtime-only intrinsic is not allowed during constant evaluation")
            }
            Rvalue::Invoke { .. } if facts.source_bindings.is_none() => {
                if function {
                    Some("non-const calls are not allowed in const functions")
                } else {
                    Some("non-const calls are not allowed in constant initializers")
                }
            }
            Rvalue::Call { .. } => match function_call(facts, body, value.kind) {
                CallPermission::Allowed => None,
                CallPermission::Member if function => {
                    members += 1;
                    None
                }
                CallPermission::Member => {
                    Some("source const-trait member execution is not implemented")
                }
                CallPermission::NonConst => Some(if function {
                    "non-const calls are not allowed in const functions"
                } else {
                    "non-const calls are not allowed in constant initializers"
                }),
            },
            Rvalue::Cast {
                kind: crate::sema::model::Cast::PointerToInteger,
                ..
            } => Some("pointers cannot be cast to integers during const eval"),
            Rvalue::Binary { left, .. }
                if matches!(
                    facts.types.kind(body.values[left.0 as usize].ty),
                    Some(crate::types::Kind::Pointer { .. } | crate::types::Kind::Function { .. })
                ) =>
            {
                Some("pointers cannot be compared during const eval")
            }
            _ => None,
        };
        if let Some(message) = message {
            return Err(report.diagnostic(report.body_error(body.source, message)));
        }
    }
    if operators != 0 || members != 0 {
        invocations::check_calls(facts, body, (operators, members), work)
            .map_err(|e| report.diagnostic(report.body_error(body.source, e)))?;
    }
    Ok(())
}
enum CallPermission {
    Allowed,
    NonConst,
    Member,
}
fn function_call(facts: &Declarations, body: &super::ir::Body, value: Rvalue) -> CallPermission {
    let Rvalue::Call { callee, .. } = value else {
        return CallPermission::Allowed;
    };
    if let Some(crate::types::Kind::FunctionItem { item, .. }) = body
        .values
        .get(callee.0 as usize)
        .and_then(|value| facts.types.kind(value.ty))
    {
        let definition = crate::resolve::model::DefId(item.0);
        if facts.is_const_function(definition)
            || facts.members.constructors.contains_key(&definition)
        {
            return CallPermission::Allowed;
        }
        if facts
            .associated
            .trait_owner(definition)
            .is_some_and(|owner| facts.associated.constant_trait(owner) == Some(true))
        {
            return CallPermission::Member;
        }
    }
    CallPermission::NonConst
}
/// Named values used by this CFG, with static storage distinguished from constants.
pub fn references(
    body: &super::ir::Body,
) -> impl Iterator<Item = (crate::resolve::model::DefId, bool)> + '_ {
    let values = body.values.iter().filter_map(|value| match value.kind {
        Rvalue::Constant(id) => Some((id, false)),
        Rvalue::Read(Place::Static(id))
        | Rvalue::Address(Place::Static(id))
        | Rvalue::Borrow {
            place: Place::Static(id),
            ..
        } => Some((id, true)),
        _ => None,
    });
    let projections = body.projections.iter().filter_map(|projection| {
        if let Place::Static(id) = projection.base {
            Some((id, true))
        } else {
            None
        }
    });
    let statements = body.statements.iter().filter_map(|statement| {
        if let super::ir::Action::Store {
            place: Place::Static(id),
            ..
        } = statement.action
        {
            Some((id, true))
        } else {
            None
        }
    });
    values.chain(projections).chain(statements)
}

/// Closed, trivial aliases are checked before demand by the pinned Rust profile.
/// Branching and generic initializers retain demand-only value evaluation.
pub(crate) fn alias_cycles(
    report: &Report,
    facts: &Declarations,
    templates: &[super::templates::Template],
) -> Result<(), Error> {
    use super::ir::{Owner, Terminator};
    let mut aliases = crate::support::hash::HashMap::default();
    let mut order = Vec::new();
    for template in templates {
        let body = &template.body;
        let Owner::Initializer { definition, .. } = body.owner else {
            continue;
        };
        if facts.generics.has_parameters(definition) {
            continue;
        }
        let mut returns = body.blocks.iter().filter_map(|block| {
            if let Terminator::Return(value) = block.terminator {
                Some(value)
            } else {
                None
            }
        });
        let Some(mut value) = returns.next() else {
            continue;
        };
        if returns.next().is_some() {
            continue;
        }
        let mut steps = 0;
        loop {
            steps += 1;
            if steps > body.values.len() {
                return Err(report.body_error(body.source, "cycle in constant alias value graph"));
            }
            match body.values[value.0 as usize].kind {
                Rvalue::Constrain(source) => value = source,
                Rvalue::AssociatedConstant(crate::sema::values::Instance {
                    definition: target,
                    ..
                })
                | Rvalue::Constant(target)
                    if !facts.generics.has_parameters(target) =>
                {
                    aliases.insert(definition, target);
                    order.push((definition, body.source));
                    break;
                }
                _ => break,
            }
        }
    }
    let mut done = HashSet::default();
    let mut active = HashSet::default();
    for (mut current, source) in order {
        active.clear();
        while let Some(&next) = aliases.get(&current) {
            if done.contains(&current) {
                break;
            }
            if !active.insert(current) {
                return Err(report.body_error(source, "cycle in closed constant aliases"));
            }
            current = next;
        }
        done.extend(active.iter().copied());
    }
    Ok(())
}
