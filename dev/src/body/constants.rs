//! Complete named values required by typed bodies before the read-only lowerer.
use super::ir::{Place, Program, Rvalue};
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::{model::Declarations, values::Evaluator},
};
use std::collections::HashSet;

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
    let mut seen = HashSet::new();
    let mut demands = Vec::new();
    let mut associated_seen = HashSet::new();
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
        eligibility(report, facts, body)?;
        for value in &body.values {
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
    if demands.is_empty() && associated.is_empty() {
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
    if !body.const_context {
        return Ok(());
    }
    let function = matches!(body.owner, super::ir::Owner::Function { .. });
    // The CFG retains both sides of a conditional, but not source after an
    // unconditional transfer. Const permission is checked before value demand.
    for value in &body.values {
        let message = match value.kind {
            Rvalue::Intrinsic { intrinsic, .. }
                if intrinsic.descriptor().constant
                    == crate::support::intrinsics::ConstPermission::RuntimeOnly =>
            {
                Some("runtime-only intrinsic is not allowed during constant evaluation")
            }
            Rvalue::Call { .. } | Rvalue::Invoke { .. }
                if matches!(value.kind, Rvalue::Invoke { .. })
                    || function_call(facts, body, value.kind) =>
            {
                if function {
                    Some("non-const calls are not allowed in const functions")
                } else {
                    Some("non-const calls are not allowed in constant initializers")
                }
            }
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
            return Err(report.diagnostic(Declarations::error(report, body.source, message)));
        }
    }
    Ok(())
}
fn function_call(facts: &Declarations, body: &super::ir::Body, value: Rvalue) -> bool {
    let Rvalue::Call { callee, .. } = value else {
        return false;
    };
    let Some(crate::types::Kind::FunctionItem { item, .. }) = body
        .values
        .get(callee.0 as usize)
        .and_then(|value| facts.types.kind(value.ty))
    else {
        return true;
    };
    let definition = crate::resolve::model::DefId(item.0);
    !facts.is_const_function(definition) && !facts.members.constructors.contains_key(&definition)
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
    let mut aliases = std::collections::HashMap::new();
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
                return Err(Declarations::error(
                    report,
                    body.source,
                    "cycle in constant alias value graph",
                ));
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
    let mut done = HashSet::new();
    let mut active = HashSet::new();
    for (mut current, source) in order {
        active.clear();
        while let Some(&next) = aliases.get(&current) {
            if done.contains(&current) {
                break;
            }
            if !active.insert(current) {
                return Err(Declarations::error(
                    report,
                    source,
                    "cycle in closed constant aliases",
                ));
            }
            current = next;
        }
        done.extend(active.iter().copied());
    }
    Ok(())
}
