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
pub(super) fn validate(
    original: &s::Original,
    work: &mut Work,
    branching: bool,
) -> Result<(), &'static str> {
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
    if branching
        && original
            .blocks
            .iter()
            .any(|block| block.terminator.successors()[1].is_some())
    {
        return blocks(original, &mut state, work);
    }
    execute(original, &mut state, work)
}

/// Version thirteen: in a branching body each snapshot dependency must be
/// evaluated earlier in its user's own block, which then dominates the use.
/// Every block is walked once and forgets its definitions at its end.
fn blocks(original: &s::Original, state: &mut [u8], work: &mut Work) -> Result<(), &'static str> {
    let mut defined = Vec::new();
    for current in &original.blocks {
        for statement in current.statements.slice(&original.statements)? {
            work.charge(1)?;
            uses(original, state, statement.action, work)?;
            if let Some(node) = definition(original, statement.action)? {
                state[node] = 1;
                defined.push(node);
            }
        }
        match current.terminator {
            g::Terminator::Return(value)
            | g::Terminator::Branch {
                condition: value, ..
            } => {
                require(state, Some(value as usize))?;
            }
            g::Terminator::Jump(_)
            | g::Terminator::FalseEdge { .. }
            | g::Terminator::Unreachable
            | g::Terminator::Panic => {}
        }
        work.charge(defined.len())?;
        for node in defined.drain(..) {
            state[node] = 0;
        }
    }
    Ok(())
}

/// One statement's dependencies must already be available.
fn uses(
    original: &s::Original,
    state: &[u8],
    action: s::Action,
    work: &mut Work,
) -> Result<(), &'static str> {
    if let Some(node) = definition(original, action)? {
        let mut ordinal = 0;
        loop {
            work.charge(1)?;
            let Some(child) = dependency(original, node, ordinal)? else {
                return Ok(());
            };
            require(state, Some(child))?;
            ordinal += 1;
        }
    }
    match action {
        s::Action::Store {
            place: target,
            value,
        } => {
            require(state, place(original, target)?)?;
            require(state, Some(value as usize))
        }
        s::Action::CheckInitialized(target)
        | s::Action::Mention(target)
        | s::Action::BindView { place: target, .. }
        | s::Action::Select { anchor: target, .. } => require(state, place(original, target)?),
        _ => Ok(()),
    }
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
    // Every kind states its dependencies; a new kind cannot fall through.
    let operand = match original.values[node].kind {
        s::ValueKind::Read(target)
        | s::ValueKind::Borrow(target)
        | s::ValueKind::TwoPhaseBorrow(target)
        | s::ValueKind::IsVariant { place: target, .. } => {
            return if ordinal == 0 {
                place(original, target)
            } else {
                Ok(None)
            };
        }
        s::ValueKind::Constrain(value)
        | s::ValueKind::ScalarCast(value)
        | s::ValueKind::EnumCast(value)
        | s::ValueKind::Reify(value) => (ordinal == 0).then_some(value),
        s::ValueKind::Binary { left, right, .. } => [left, right].get(ordinal).copied(),
        s::ValueKind::Aggregate { fields }
        | s::ValueKind::VariantAggregate { fields, .. }
        | s::ValueKind::Invoke {
            arguments: fields, ..
        }
        | s::ValueKind::Assembly {
            arguments: fields, ..
        } => fields.slice(&original.arguments)?.get(ordinal).copied(),
        s::ValueKind::Call {
            callee, arguments, ..
        }
        | s::ValueKind::PointerCall {
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
        // A constant read names a declaration's header row, not a value.
        s::ValueKind::Unit
        | s::ValueKind::Scalar(_)
        | s::ValueKind::Text(_)
        | s::ValueKind::Function(_)
        | s::ValueKind::Constant(_) => None,
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

// Snapshot bodies admit only one successor per block. Walk the actual entry
// chain once, including the first traversal of a loop; unavailable first-use
// values cannot be justified by a later iteration. Unreachable recipes still
// passed the complete graph and ownership checks above. A two-way exit would
// need dominance, not this chain, so it is refused before the walk (the block
// scan is prepaid by `validate`).
fn execute(original: &s::Original, state: &mut [u8], work: &mut Work) -> Result<(), &'static str> {
    if original.blocks.iter().any(|block| {
        matches!(
            block.terminator,
            g::Terminator::Branch { .. } | g::Terminator::FalseEdge { .. }
        )
    }) {
        return Err("source SSA snapshots across two-way exits require a wider dominance proof");
    }
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
            g::Terminator::Unreachable | g::Terminator::Panic => return Ok(()),
            g::Terminator::Branch { .. } | g::Terminator::FalseEdge { .. } => {
                return Err("source SSA snapshot walk reached an actual two-way exit");
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../../../tests/invariants/ssa_projection_ports.rs"]
mod tests;
