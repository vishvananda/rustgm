//! Interpret nominal guards per query side, leaving shared structural facts immutable.
use super::{Budget, Language, LayoutId, automata::Node, budget::push};
use crate::support::{hash::HashMap, retained::Growth};
pub(super) type Memo = HashMap<(LayoutId, bool), LayoutId>;
pub(super) fn interpret(
    language: &mut Language,
    root: LayoutId,
    enforce: bool,
    ready: &mut Memo,
    budget: &mut Budget<'_>,
) -> Result<LayoutId, &'static str> {
    if !language.info(root)?.guarded {
        budget.charge(1, 0)?;
        return Ok(root);
    }
    let mut stack = Vec::new();
    push(&mut stack, (root, false), budget)?;
    while let Some((id, finish)) = stack.pop() {
        budget.charge(1, 0)?;
        if ready.contains_key(&(id, enforce)) {
            continue;
        }
        let node = language.node(id)?;
        if !finish {
            push(&mut stack, (id, true), budget)?;
            match node {
                Node::Sequence(a, b) | Node::Choice(a, b) => {
                    push(&mut stack, (a, false), budget)?;
                    push(&mut stack, (b, false), budget)?;
                }
                Node::Repeat(a, _) => push(&mut stack, (a, false), budget)?,
                Node::Guard(a) if !enforce => push(&mut stack, (a, false), budget)?,
                _ => {}
            }
            continue;
        }
        let get = |child| {
            ready
                .get(&(child, enforce))
                .copied()
                .ok_or("missing interpreted layout child")
        };
        let value = match node {
            Node::Sequence(a, b) => language.sequence(get(a)?, get(b)?, budget)?,
            Node::Choice(a, b) => language.either(get(a)?, get(b)?, budget)?,
            Node::Repeat(a, n) => language.repeat(get(a)?, n, budget)?,
            Node::Guard(a) if enforce => language.empty(language.info(a)?.size, budget)?,
            Node::Guard(a) => get(a)?,
            _ => id,
        };
        let growth = Growth::map::<(LayoutId, bool), LayoutId>(ready.len(), ready.capacity(), 1);
        budget.charge(growth.work, growth.bytes)?;
        ready
            .try_reserve(1)
            .map_err(|_| "layout interpretation allocation failed")?;
        ready.insert((id, enforce), value);
    }
    ready
        .get(&(root, enforce))
        .copied()
        .ok_or("missing interpreted layout root")
}
