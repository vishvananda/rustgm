//! SSA projection bases retain graph dependencies and real execution availability.
use super::{Work, g, s};

#[derive(Clone, Copy)]
struct Visit {
    node: usize,
    next: usize,
}

pub(super) fn allocation(original: &s::Original) -> usize {
    original
        .values
        .len()
        .saturating_add(original.projections.len())
        .saturating_mul(4 * size_of::<Visit>() + 3)
        .saturating_add(original.blocks.len())
}

/// Caller prepays the complete cold scratch bound before any collection exists.
pub(super) fn validate(original: &s::Original, work: &mut Work) -> Result<(), &'static str> {
    let count = original
        .values
        .len()
        .saturating_add(original.projections.len());
    work.charge(
        count
            .saturating_mul(3)
            .saturating_add(original.blocks.len()),
    )?;
    let mut state = vec![0u8; count];
    let mut stack = Vec::with_capacity(count);
    for root in 0..count {
        work.charge(1)?;
        if state[root] != 0 {
            continue;
        }
        state[root] = 1;
        stack.push(Visit {
            node: root,
            next: 0,
        });
        while let Some(current) = stack.last_mut() {
            work.charge(1)?;
            if let Some(child) = dependency(original, current.node, current.next)? {
                current.next += 1;
                match state[child] {
                    0 => {
                        state[child] = 1;
                        stack.push(Visit {
                            node: child,
                            next: 0,
                        });
                    }
                    1 => return Err("source SSA place/value dependency graph is cyclic"),
                    _ => {}
                }
            } else {
                state[current.node] = 2;
                stack.pop();
            }
        }
    }
    work.charge(count)?;
    state.fill(0);
    for statement in &original.statements {
        work.charge(1)?;
        let Some(node) = definition(original, statement.action)? else {
            continue;
        };
        if std::mem::replace(&mut state[node], 1) != 0 {
            return Err("source SSA snapshot has duplicate evaluation/projection ownership");
        }
    }
    work.charge(count)?;
    if state.contains(&0) {
        return Err("source SSA snapshot has no actual evaluation/projection ownership");
    }
    work.charge(count)?;
    state.fill(0);
    execute(original, &mut state, work)
}

fn definition(original: &s::Original, action: s::Action) -> Result<Option<usize>, &'static str> {
    match action {
        s::Action::Evaluate(value) if (value as usize) < original.values.len() => {
            Ok(Some(value as usize))
        }
        s::Action::Project(projection) if (projection as usize) < original.projections.len() => {
            Ok(Some(original.values.len() + projection as usize))
        }
        s::Action::Evaluate(_) | s::Action::Project(_) => {
            Err("source SSA definition has a foreign actual value/projection")
        }
        _ => Ok(None),
    }
}

fn place(original: &s::Original, place: s::Place) -> Result<Option<usize>, &'static str> {
    match place {
        s::Place::Local(index) if (index as usize) < original.locals.len() => Ok(None),
        s::Place::Temporary(index) if (index as usize) < original.values.len() => {
            Ok(Some(index as usize))
        }
        s::Place::Projection(index) if (index as usize) < original.projections.len() => {
            Ok(Some(original.values.len() + index as usize))
        }
        _ => Err("source SSA place dependency belongs to another body"),
    }
}

fn dependency(
    original: &s::Original,
    node: usize,
    ordinal: usize,
) -> Result<Option<usize>, &'static str> {
    if node >= original.values.len() {
        return if ordinal == 0 {
            place(
                original,
                original.projections[node - original.values.len()].base,
            )
        } else {
            Ok(None)
        };
    }
    let operand = match original.values[node].kind {
        s::ValueKind::Read(target) | s::ValueKind::Borrow(target) if ordinal == 0 => {
            return place(original, target);
        }
        s::ValueKind::Constrain(value) | s::ValueKind::ScalarCast(value) if ordinal == 0 => {
            Some(value)
        }
        s::ValueKind::Call {
            callee, arguments, ..
        } => {
            if ordinal == 0 {
                Some(callee)
            } else {
                arguments
                    .slice(&original.arguments)?
                    .get(ordinal - 1)
                    .copied()
            }
        }
        _ => None,
    };
    match operand {
        Some(value) if (value as usize) < original.values.len() => Ok(Some(value as usize)),
        Some(_) => Err("source SSA value dependency belongs to another body"),
        None => Ok(None),
    }
}

fn require(state: &[u8], node: Option<usize>) -> Result<(), &'static str> {
    if node.is_some_and(|node| state.get(node).is_none_or(|state| *state == 0)) {
        return Err("source SSA snapshot use is not dominated by its actual evaluation");
    }
    Ok(())
}

// The current wire CFG has only one successor per block. Walk the actual entry
// chain once, including the first traversal of a loop; unavailable first-use
// values cannot be justified by a later iteration. Unreachable recipes still
// passed the complete graph and ownership checks above.
fn execute(original: &s::Original, state: &mut [u8], work: &mut Work) -> Result<(), &'static str> {
    let mut seen = vec![false; original.blocks.len()];
    let mut block = 0;
    loop {
        work.charge(1)?;
        let visited = seen
            .get_mut(block)
            .ok_or("source SSA block is foreign or absent")?;
        if std::mem::replace(visited, true) {
            return Ok(());
        }
        let current = &original.blocks[block];
        for statement in current.statements.slice(&original.statements)? {
            work.charge(1)?;
            if let Some(node) = definition(original, statement.action)? {
                let mut ordinal = 0;
                loop {
                    work.charge(1)?;
                    let Some(child) = dependency(original, node, ordinal)? else {
                        break;
                    };
                    require(state, Some(child))?;
                    ordinal += 1;
                }
                state[node] = 1;
            } else {
                match statement.action {
                    s::Action::Store {
                        place: target,
                        value,
                    } => {
                        require(state, place(original, target)?)?;
                        require(state, Some(value as usize))?;
                    }
                    s::Action::CheckInitialized(target) | s::Action::Mention(target) => {
                        require(state, place(original, target)?)?;
                    }
                    _ => {}
                }
            }
        }
        match current.terminator {
            g::Terminator::Jump(next) => block = next as usize,
            g::Terminator::Return(value) => return require(state, Some(value as usize)),
            g::Terminator::Unreachable => return Ok(()),
        }
    }
}

#[cfg(test)]
#[path = "../../../../../../tests/invariants/ssa_projection_ports.rs"]
mod tests;
