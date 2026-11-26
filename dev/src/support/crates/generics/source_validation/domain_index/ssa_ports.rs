//! SSA projection bases retain graph dependencies and real execution availability.
use super::{Work, g, s};

#[derive(Clone, Copy)]
struct Visit {
    node: usize,
    next: usize,
}

#[derive(Clone, Copy)]
pub(super) enum Scope {
    Linear,
    Block,
    Variant,
    Dominated,
}

impl Scope {
    pub(super) const fn for_body(
        branching: bool,
        tested_temporary: bool,
        declaration_policies: bool,
    ) -> Self {
        if declaration_policies {
            Self::Dominated
        } else if tested_temporary {
            Self::Variant
        } else if branching {
            Self::Block
        } else {
            Self::Linear
        }
    }
}

pub(super) fn allocation(original: &s::Original, scope: Scope) -> usize {
    let block_bytes = match scope {
        Scope::Dominated => {
            size_of::<(usize, usize)>() + 9 * size_of::<usize>() + size_of::<bool>()
        }
        Scope::Variant => size_of::<(usize, usize)>() + size_of::<u32>() + size_of::<bool>(),
        _ => size_of::<bool>(),
    };
    original
        .values
        .len()
        .saturating_add(original.projections.len())
        .saturating_mul(4 * size_of::<Visit>() + 3)
        .saturating_add(original.blocks.len().saturating_mul(block_bytes))
}

/// Caller prepays the complete cold scratch bound before any collection exists.
pub(super) fn validate(
    original: &s::Original,
    work: &mut Work,
    scope: Scope,
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
        if matches!(scope, Scope::Dominated)
            && let s::Action::Select { projection, anchor } = statement.action
        {
            mark_selection_chain(original, &mut state, projection, anchor, work)?;
            continue;
        }
        let Some(node) = definition(original, statement.action)? else {
            continue;
        };
        if std::mem::replace(&mut state[node], 1) != 0 {
            return Err("source SSA snapshot has duplicate evaluation/projection ownership");
        }
    }
    work.charge(count)?;
    let required = if matches!(scope, Scope::Dominated) {
        &state[..original.values.len()]
    } else {
        &state[..]
    };
    if required.contains(&0) {
        return Err("source SSA snapshot has no actual evaluation/projection ownership");
    }
    work.charge(count)?;
    state.fill(0);
    match scope {
        Scope::Dominated => dominated_blocks(original, &mut state, work),
        Scope::Variant => variant_blocks(original, &mut state, work),
        Scope::Block
            if original
                .blocks
                .iter()
                .any(|block| block.terminator.successors()[1].is_some()) =>
        {
            blocks(original, &mut state, work)
        }
        _ => execute(original, &mut state, work),
    }
}

/// Version thirteen: in a branching body each snapshot dependency must be
/// evaluated earlier in its user's own block, which then dominates the use.
/// Every block is walked once and forgets its definitions at its end.
fn blocks(original: &s::Original, state: &mut [u8], work: &mut Work) -> Result<(), &'static str> {
    let mut defined = Vec::new();
    for &current in &original.blocks {
        block(original, current, state, &mut defined, false, work)?;
        rewind(state, &mut defined, 0, work)?;
    }
    Ok(())
}

fn block(
    original: &s::Original,
    current: g::Block,
    state: &mut [u8],
    defined: &mut Vec<usize>,
    dominated: bool,
    work: &mut Work,
) -> Result<(), &'static str> {
    for statement in current.statements.slice(&original.statements)? {
        work.charge(1)?;
        if dominated && let s::Action::Select { projection, anchor } = statement.action {
            select_uses(original, state, projection, anchor, work)?;
            let node = original.values.len() + projection as usize;
            if state[node] == 0 {
                state[node] = 1;
                defined.push(node);
            }
            continue;
        }
        uses(original, state, statement.action, work)?;
        if let Some(node) = definition(original, statement.action)?
            && state[node] == 0
        {
            state[node] = 1;
            defined.push(node);
        }
    }
    match current.terminator {
        g::Terminator::Return(value)
        | g::Terminator::Branch {
            condition: value, ..
        } => require(state, Some(value as usize)),
        g::Terminator::Jump(_)
        | g::Terminator::FalseEdge { .. }
        | g::Terminator::Unreachable
        | g::Terminator::Panic => Ok(()),
    }
}

fn mark_selection_chain(
    original: &s::Original,
    state: &mut [u8],
    mut current: u32,
    anchor: s::Place,
    work: &mut Work,
) -> Result<(), &'static str> {
    let values = original.values.len();
    for _ in 0..=original.projections.len() {
        work.charge(1)?;
        let projection = original
            .projections
            .get(current as usize)
            .ok_or("source SSA selection projection belongs to another body")?;
        state[values + current as usize] = 1;
        if projection.base == anchor {
            return Ok(());
        }
        let s::Place::Projection(parent) = projection.base else {
            return Err("source pattern selection does not reach its anchor");
        };
        current = parent;
    }
    Err("source pattern selection chain is cyclic")
}

fn select_uses(
    original: &s::Original,
    state: &[u8],
    mut current: u32,
    anchor: s::Place,
    work: &mut Work,
) -> Result<(), &'static str> {
    require(state, place(original, anchor)?)?;
    for _ in 0..=original.projections.len() {
        work.charge(1)?;
        let projection = original
            .projections
            .get(current as usize)
            .ok_or("source SSA selection projection belongs to another body")?;
        if let s::ProjectionKind::Index { index, .. }
        | s::ProjectionKind::IndexParameter { index, .. }
        | s::ProjectionKind::SliceIndex { index } = projection.kind
        {
            require(state, Some(index as usize))?;
        }
        if projection.base == anchor {
            return Ok(());
        }
        let s::Place::Projection(parent) = projection.base else {
            return Err("source pattern selection does not reach its anchor");
        };
        current = parent;
    }
    Err("source pattern selection chain is cyclic")
}

fn rewind(
    state: &mut [u8],
    defined: &mut Vec<usize>,
    keep: usize,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(defined.len() - keep)?;
    for node in defined.drain(keep..) {
        state[node] = 0;
    }
    Ok(())
}

/// A tested temporary can reach the successful downcast along the exact
/// single-incoming-edge path. Entry and merges begin with no inherited values.
fn variant_blocks(
    original: &s::Original,
    state: &mut [u8],
    work: &mut Work,
) -> Result<(), &'static str> {
    let count = original.blocks.len();
    work.charge(count.saturating_mul(3))?;
    let mut incoming = vec![0_u32; count];
    for current in &original.blocks {
        for successor in current.terminator.successors().into_iter().flatten() {
            work.charge(1)?;
            let slot = incoming
                .get_mut(successor as usize)
                .ok_or("source SSA edge leaves its body")?;
            *slot = slot.saturating_add(1);
        }
    }
    let mut visited = vec![false; count];
    let mut pending = Vec::with_capacity(count);
    let mut defined = Vec::new();
    for (root, &edges) in incoming.iter().enumerate() {
        if root != 0 && edges == 1 {
            continue;
        }
        pending.push((root, 0));
        while let Some((index, keep)) = pending.pop() {
            work.charge(1)?;
            rewind(state, &mut defined, keep, work)?;
            visited[index] = true;
            let current = original.blocks[index];
            block(original, current, state, &mut defined, false, work)?;
            for successor in current.terminator.successors().into_iter().flatten() {
                work.charge(1)?;
                let child = successor as usize;
                if child != 0 && incoming[child] == 1 {
                    pending.push((child, defined.len()));
                }
            }
        }
        rewind(state, &mut defined, 0, work)?;
    }
    // A disconnected cycle gets the ordinary block-local check.
    for (&current, visited) in original.blocks.iter().zip(visited) {
        if !visited {
            block(original, current, state, &mut defined, false, work)?;
            rewind(state, &mut defined, 0, work)?;
        }
    }
    Ok(())
}

/// Version fourteen: exact dominator-tree DFS over reachable blocks so
/// temporaries and pattern selections survive arbitrary guard/match merges.
fn dominated_blocks(
    original: &s::Original,
    state: &mut [u8],
    work: &mut Work,
) -> Result<(), &'static str> {
    let count = original.blocks.len();
    if count == 0 {
        return Ok(());
    }
    let (rpo, rpo_index) = reverse_postorder(original, work)?;
    let idom = immediate_dominators(original, &rpo, &rpo_index, work)?;
    let mut dom_head = vec![usize::MAX; count];
    let mut dom_next = vec![usize::MAX; count];
    for child in 1..count {
        let parent = idom[child];
        if parent != usize::MAX {
            dom_next[child] = dom_head[parent];
            dom_head[parent] = child;
        }
    }
    let mut pending = Vec::with_capacity(count);
    let mut defined = Vec::new();
    pending.push((0_usize, 0_usize));
    while let Some((index, keep)) = pending.pop() {
        work.charge(1)?;
        rewind(state, &mut defined, keep, work)?;
        block(
            original,
            original.blocks[index],
            state,
            &mut defined,
            true,
            work,
        )?;
        let mut child = dom_head[index];
        while child != usize::MAX {
            work.charge(1)?;
            pending.push((child, defined.len()));
            child = dom_next[child];
        }
    }
    rewind(state, &mut defined, 0, work)?;
    for (&current, &pos) in original.blocks.iter().zip(&rpo_index) {
        if pos == usize::MAX {
            block(original, current, state, &mut defined, true, work)?;
            rewind(state, &mut defined, 0, work)?;
        }
    }
    Ok(())
}

fn reverse_postorder(
    original: &s::Original,
    work: &mut Work,
) -> Result<(Vec<usize>, Vec<usize>), &'static str> {
    let count = original.blocks.len();
    work.charge(count.saturating_mul(2))?;
    let mut seen = vec![false; count];
    let mut postorder = Vec::with_capacity(count);
    let mut stack = Vec::with_capacity(count);
    seen[0] = true;
    stack.push((0_usize, 0_usize));
    while let Some(&mut (node, ref mut next)) = stack.last_mut() {
        work.charge(1)?;
        let successors = original.blocks[node].terminator.successors();
        let mut child_to_push = None;
        while *next < successors.len() {
            let candidate = successors[*next];
            *next += 1;
            if let Some(target) = candidate {
                let child = target as usize;
                if child >= count {
                    return Err("source SSA edge leaves its body");
                }
                if !std::mem::replace(&mut seen[child], true) {
                    child_to_push = Some(child);
                    break;
                }
            }
        }
        if let Some(child) = child_to_push {
            stack.push((child, 0));
        } else {
            postorder.push(node);
            stack.pop();
        }
    }
    postorder.reverse();
    let mut rpo_index = vec![usize::MAX; count];
    for (pos, &node) in postorder.iter().enumerate() {
        rpo_index[node] = pos;
    }
    Ok((postorder, rpo_index))
}

fn immediate_dominators(
    original: &s::Original,
    rpo: &[usize],
    rpo_index: &[usize],
    work: &mut Work,
) -> Result<Vec<usize>, &'static str> {
    let count = original.blocks.len();
    work.charge(count.saturating_mul(3))?;
    let mut pred_head = vec![usize::MAX; count];
    let mut pred_next = Vec::with_capacity(count.saturating_mul(2));
    let mut pred_src = Vec::with_capacity(count.saturating_mul(2));
    for &src in rpo {
        for target in original.blocks[src]
            .terminator
            .successors()
            .into_iter()
            .flatten()
        {
            work.charge(1)?;
            let dst = target as usize;
            let edge = pred_src.len();
            pred_src.push(src);
            pred_next.push(pred_head[dst]);
            pred_head[dst] = edge;
        }
    }
    let mut idom = vec![usize::MAX; count];
    idom[0] = 0;
    let mut changed = true;
    while changed {
        changed = false;
        for &node in rpo.iter().skip(1) {
            work.charge(1)?;
            let mut candidate = usize::MAX;
            let mut edge = pred_head[node];
            while edge != usize::MAX {
                work.charge(1)?;
                let pred = pred_src[edge];
                if idom[pred] != usize::MAX {
                    candidate = if candidate == usize::MAX {
                        pred
                    } else {
                        intersect(&idom, rpo_index, pred, candidate, work)?
                    };
                }
                edge = pred_next[edge];
            }
            if candidate != usize::MAX && idom[node] != candidate {
                idom[node] = candidate;
                changed = true;
            }
        }
    }
    Ok(idom)
}

fn intersect(
    idom: &[usize],
    rpo_index: &[usize],
    mut a: usize,
    mut b: usize,
    work: &mut Work,
) -> Result<usize, &'static str> {
    while a != b {
        work.charge(1)?;
        while rpo_index[a] > rpo_index[b] {
            work.charge(1)?;
            a = idom[a];
        }
        while rpo_index[b] > rpo_index[a] {
            work.charge(1)?;
            b = idom[b];
        }
    }
    Ok(a)
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
        s::Place::Static(_) => Ok(None),
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
        let projection = &original.projections[node - original.values.len()];
        return match ordinal {
            0 => place(original, projection.base),
            1 => match projection.kind {
                s::ProjectionKind::Index { index, .. }
                | s::ProjectionKind::IndexParameter { index, .. }
                | s::ProjectionKind::SliceIndex { index }
                    if (index as usize) < original.values.len() =>
                {
                    Ok(Some(index as usize))
                }
                s::ProjectionKind::Index { .. }
                | s::ProjectionKind::IndexParameter { .. }
                | s::ProjectionKind::SliceIndex { .. } => {
                    Err("source SSA projection index belongs to another body")
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        };
    }
    // Every kind states its dependencies; a new kind cannot fall through.
    let operand = match original.values[node].kind {
        s::ValueKind::Read(target)
        | s::ValueKind::Borrow(target)
        | s::ValueKind::TwoPhaseBorrow(target)
        | s::ValueKind::UniqueBorrow(target)
        | s::ValueKind::Address(target)
        | s::ValueKind::Length(target)
        | s::ValueKind::Validate(target)
        | s::ValueKind::TextEquals { place: target, .. }
        | s::ValueKind::IsVariant { place: target, .. } => {
            return if ordinal == 0 {
                place(original, target)
            } else {
                Ok(None)
            };
        }
        s::ValueKind::AssemblyOutput { effect: value, .. }
        | s::ValueKind::Constrain(value)
        | s::ValueKind::ScalarCast(value)
        | s::ValueKind::PointerCast(value)
        | s::ValueKind::EnumCast(value)
        | s::ValueKind::DirectEnumCast(value)
        | s::ValueKind::AddressCast { operand: value, .. }
        | s::ValueKind::Unary { operand: value, .. }
        | s::ValueKind::Union { value, .. }
        | s::ValueKind::Repeat { value, .. }
        | s::ValueKind::Unsize { value, .. }
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
        | s::ValueKind::Constant(_)
        | s::ValueKind::ConstArgument(_)
        | s::ValueKind::AssociatedConstant { .. }
        | s::ValueKind::InlineConstant(_) => None,
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
