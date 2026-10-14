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
    prepare(report, facts, bodies.bodies.iter())
}
/// Complete constants for one newly demanded body without rescanning the program.
/// # Errors
/// Reports the same required-constant failures as `analyze`.
pub fn instance(
    report: &Report,
    facts: &mut Declarations,
    body: &super::ir::Body,
) -> Result<(), Error> {
    prepare(report, facts, std::iter::once(body))
}
fn prepare<'a>(
    report: &Report,
    facts: &mut Declarations,
    bodies: impl Iterator<Item = &'a super::ir::Body>,
) -> Result<(), Error> {
    let mut seen = HashSet::new();
    let mut demands = Vec::new();
    let mut demand = |id, static_| {
        if seen.insert(id) {
            demands.push((id, static_));
        }
    };
    for body in bodies {
        for (id, static_) in references(body) {
            demand(id, static_);
        }
    }
    if demands.is_empty() {
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
    evaluator.complete_static_storage()?;
    Ok(())
}
/// Named values used by this CFG, with static storage distinguished from constants.
pub fn references(
    body: &super::ir::Body,
) -> impl Iterator<Item = (crate::resolve::model::DefId, bool)> + '_ {
    let values = body.values.iter().filter_map(|value| match value.kind {
        Rvalue::Constant(id) => Some((id, false)),
        Rvalue::Read(Place::Static(id))
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
