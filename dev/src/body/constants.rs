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
    let mut seen = HashSet::new();
    let mut demands = Vec::new();
    let mut demand = |id, static_| {
        if seen.insert(id) {
            demands.push((id, static_));
        }
    };
    for body in &bodies.bodies {
        for value in &body.values {
            match value.kind {
                Rvalue::Constant(id) => demand(id, false),
                Rvalue::Read(Place::Static(id))
                | Rvalue::Borrow {
                    place: Place::Static(id),
                    ..
                } => demand(id, true),
                _ => {}
            }
        }
        for projection in &body.projections {
            if let Place::Static(id) = projection.base {
                demand(id, true);
            }
        }
        for statement in &body.statements {
            if let super::ir::Action::Store {
                place: Place::Static(id),
                ..
            } = statement.action
            {
                demand(id, true);
            }
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
    Ok(())
}
