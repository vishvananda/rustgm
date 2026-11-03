//! Iterative paired derivatives publish only completed conditional answers.
use super::{
    Assumptions, Budget, Language, LayoutId, Proof,
    automata::{ByteRange, Node},
    budget::push,
    interpretation::{Memo, interpret},
    planning,
    propositions::{ConditionId, Conditions},
    transitions::Transitions,
};
use crate::support::{hash::HashMap, retained::Growth};
pub(super) type Pair = (LayoutId, LayoutId);
pub(super) enum Task {
    Pair(Pair),
    Value(ConditionId),
    Fold { all: bool, count: usize },
    Publish(Pair),
}

/// Produce a boolean/obligation DAG for all values of the source layout.
/// # Errors
/// Rejects invalid layout identities, exhausted payment/limits and allocation failure.
pub fn decide(
    language: &mut Language,
    source: LayoutId,
    destination: LayoutId,
    assumptions: Assumptions,
    budget: &mut Budget<'_>,
) -> Result<Proof, &'static str> {
    let mut interpreted = Memo::default();
    let source = interpret(language, source, false, &mut interpreted, budget)?;
    let destination = interpret(
        language,
        destination,
        !assumptions.safety,
        &mut interpreted,
        budget,
    )?;
    let mut conditions = Conditions::new(budget)?;
    let mut cache = HashMap::default();
    let mut transitions = Transitions::default();
    let mut work = Vec::new();
    let mut values = Vec::new();
    push(&mut work, Task::Pair((source, destination)), budget)?;
    while let Some(task) = work.pop() {
        budget.charge(1, 0)?;
        match task {
            Task::Value(value) => push(&mut values, value, budget)?,
            Task::Fold { all, count } => {
                if values.len() < count {
                    return Err("incomplete layout condition stack");
                }
                let mut result = if all { Conditions::YES } else { Conditions::NO };
                for _ in 0..count {
                    let value = values.pop().ok_or("missing layout condition child")?;
                    result = conditions.combine(all, result, value, budget)?;
                }
                push(&mut values, result, budget)?;
            }
            Task::Publish(pair) => {
                let value = *values.last().ok_or("missing completed layout condition")?;
                let growth = Growth::map::<Pair, ConditionId>(cache.len(), cache.capacity(), 1);
                budget.charge(growth.work, growth.bytes)?;
                cache
                    .try_reserve(1)
                    .map_err(|_| "layout relation cache allocation failed")?;
                cache.insert(pair, value);
            }
            Task::Pair(pair) => {
                if let Some(&value) = cache.get(&pair) {
                    push(&mut values, value, budget)?;
                    continue;
                }
                push(&mut work, Task::Publish(pair), budget)?;
                if let Some(value) = basic(language, pair, assumptions)? {
                    push(&mut work, Task::Value(value), budget)?;
                    continue;
                }
                let mut plan = planning::structural(language, pair, budget)?;
                if plan.is_empty() {
                    transitions.prepare(language, pair.0, budget)?;
                    transitions.prepare(language, pair.1, budget)?;
                    plan = planning::derivatives(
                        &transitions,
                        pair,
                        assumptions,
                        &mut conditions,
                        budget,
                    )?;
                }
                for task in plan.into_iter().rev() {
                    push(&mut work, task, budget)?;
                }
            }
        }
    }
    if values.len() != 1 {
        return Err("layout relation retained unrelated conditions");
    }
    Ok(conditions.finish(values[0]))
}
pub(super) fn byte_run(
    language: &Language,
    id: LayoutId,
) -> Result<Option<(ByteRange, u64)>, &'static str> {
    Ok(match language.node(id)? {
        Node::Byte(range) => Some((range, 1)),
        Node::Repeat(child, count) => {
            if let Node::Byte(range) = language.node(child)? {
                Some((range, count))
            } else {
                None
            }
        }
        _ => None,
    })
}
fn basic(
    language: &Language,
    (source, destination): Pair,
    assumptions: Assumptions,
) -> Result<Option<ConditionId>, &'static str> {
    let a = language.info(source)?;
    let b = language.info(destination)?;
    let answer = if !a.inhabited {
        Some(true)
    } else if !b.inhabited {
        Some(false)
    } else if b.size == 0 {
        Some(true)
    } else if a.size == 0 {
        Some(b.uninit)
    } else if source == destination && a.bytes_only {
        Some(true)
    } else if let (Some((x, n)), Some((y, m))) = (
        byte_run(language, source)?,
        byte_run(language, destination)?,
    ) {
        let overlap = x.start < y.end && y.start < x.end;
        let compatible = if assumptions.validity {
            overlap
        } else {
            x.start >= y.start && x.end <= y.end
        };
        Some(compatible && (n >= m || y.end == 257))
    } else if let Some((range, _)) = byte_run(language, destination)? {
        if a.bytes_only
            && range.start == 0
            && (range.end == 257
                || range.end == 256
                    && a.size >= b.size
                    && if assumptions.validity {
                        a.can_initialize
                    } else {
                        a.initialized
                    })
        {
            Some(true)
        } else {
            None
        }
    } else {
        None
    };
    Ok(answer.map(|yes| if yes { Conditions::YES } else { Conditions::NO }))
}
