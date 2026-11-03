//! Product/repetition factoring precedes the general interval/reference comparison.
use super::{
    Assumptions, Budget, Language, Reference,
    automata::Node,
    budget::push,
    propositions::{ConditionId, Conditions, Obligation},
    relation::{Pair, Task, byte_run},
    transitions::Transitions,
};

fn conjunction(pairs: &[Pair], budget: &mut Budget<'_>) -> Result<Vec<Task>, &'static str> {
    let mut result = Vec::new();
    for &pair in pairs {
        push(&mut result, Task::Pair(pair), budget)?;
    }
    push(
        &mut result,
        Task::Fold {
            all: true,
            count: pairs.len(),
        },
        budget,
    )?;
    Ok(result)
}
fn with_uninitialized_tail(
    language: &Language,
    pair: Pair,
    tail: super::LayoutId,
    budget: &mut Budget<'_>,
) -> Result<Vec<Task>, &'static str> {
    let mut result = conjunction(&[pair], budget)?;
    push(
        &mut result,
        Task::Value(if language.info(tail)?.uninit {
            Conditions::YES
        } else {
            Conditions::NO
        }),
        budget,
    )?;
    push(
        &mut result,
        Task::Fold {
            all: true,
            count: 2,
        },
        budget,
    )?;
    Ok(result)
}
pub(super) fn structural(
    language: &mut Language,
    (source, destination): Pair,
    budget: &mut Budget<'_>,
) -> Result<Vec<Task>, &'static str> {
    let source_node = language.node(source)?;
    let destination_node = language.node(destination)?;
    let source_size = language.info(source)?.size;
    let destination_size = language.info(destination)?.size;
    if let Node::Sequence(left, _) = source_node
        && language.info(left)?.size >= destination_size
    {
        return conjunction(&[(left, destination)], budget);
    }
    if let Node::Sequence(left, right) = destination_node {
        let size = language.info(left)?.size;
        if source_size <= size {
            return with_uninitialized_tail(language, (source, left), right, budget);
        }
    }
    match (source_node, destination_node) {
        (
            Node::Sequence(source_prefix, source_suffix),
            Node::Sequence(destination_prefix, destination_suffix),
        ) if language.info(source_prefix)?.size == language.info(destination_prefix)?.size => {
            return conjunction(
                &[
                    (source_prefix, destination_prefix),
                    (source_suffix, destination_suffix),
                ],
                budget,
            );
        }
        (
            Node::Repeat(source_prefix, source_count),
            Node::Repeat(source_suffix, destination_count),
        ) if language.info(source_prefix)?.size == language.info(source_suffix)?.size => {
            let mut pairs = Vec::new();
            push(&mut pairs, (source_prefix, source_suffix), budget)?;
            if source_count < destination_count {
                let empty = language.epsilon(budget)?;
                push(&mut pairs, (empty, source_suffix), budget)?;
            }
            return conjunction(&pairs, budget);
        }
        (Node::Sequence(source_prefix, source_suffix), Node::Repeat(unit, source_count)) => {
            let size = language.info(unit)?.size;
            let width = language.info(source_prefix)?.size;
            if size != 0 && width % size == 0 && width / size < source_count {
                let first = language.repeat(unit, width / size, budget)?;
                let rest = language.repeat(unit, source_count - width / size, budget)?;
                return conjunction(&[(source_prefix, first), (source_suffix, rest)], budget);
            }
        }
        (Node::Repeat(unit, source_count), Node::Sequence(source_prefix, source_suffix)) => {
            let size = language.info(unit)?.size;
            let width = language.info(source_prefix)?.size;
            if size != 0 && width % size == 0 && width / size < source_count {
                let first = language.repeat(unit, width / size, budget)?;
                let rest = language.repeat(unit, source_count - width / size, budget)?;
                return conjunction(&[(first, source_prefix), (rest, source_suffix)], budget);
            }
        }
        _ => {}
    }
    if let (Some((range, count)), Node::Repeat(element, source_count)) =
        (byte_run(language, source)?, destination_node)
    {
        let width = language.info(element)?.size;
        if width > 1 && count % width == 0 {
            let byte = language.byte(range, budget)?;
            let unit = language.repeat(byte, width, budget)?;
            return if count / width < source_count {
                with_uninitialized_tail(language, (unit, element), element, budget)
            } else {
                conjunction(&[(unit, element)], budget)
            };
        }
    }
    Ok(Vec::new())
}
fn reference_conditions(
    a: Reference,
    b: Reference,
    options: Assumptions,
    conditions: &mut Conditions,
    budget: &mut Budget<'_>,
) -> Result<ConditionId, &'static str> {
    if !a.mutable && b.mutable || !options.alignment && a.align < b.align || b.size > a.size {
        return Ok(Conditions::NO);
    }
    let mut result = conditions.obligation(
        Obligation::Transmutable {
            source: a.referent,
            destination: b.referent,
        },
        budget,
    )?;
    if !options.lifetimes {
        let value = conditions.obligation(
            Obligation::Outlives {
                long: a.region,
                short: b.region,
            },
            budget,
        )?;
        result = conditions.combine(true, result, value, budget)?;
    }
    if b.mutable {
        let value = conditions.obligation(
            Obligation::Transmutable {
                source: b.referent,
                destination: a.referent,
            },
            budget,
        )?;
        result = conditions.combine(true, result, value, budget)?;
        if !options.lifetimes {
            let value = conditions.obligation(
                Obligation::Outlives {
                    long: b.region,
                    short: a.region,
                },
                budget,
            )?;
            result = conditions.combine(true, result, value, budget)?;
        }
    } else {
        let value = conditions.obligation(Obligation::Freeze(b.referent), budget)?;
        result = conditions.combine(true, result, value, budget)?;
    }
    Ok(result)
}
pub(super) fn derivatives(
    transitions: &Transitions,
    (a, b): Pair,
    options: Assumptions,
    conditions: &mut Conditions,
    budget: &mut Budget<'_>,
) -> Result<Vec<Task>, &'static str> {
    let a = transitions.get(a)?;
    let b = transitions.get(b)?;
    let mut result = Vec::new();
    let mut branches = 0;
    let mut j = 0;
    for source in &a.bytes {
        let mut start = source.range.start;
        while start < source.range.end {
            budget.charge(1, 0)?;
            while b.bytes.get(j).is_some_and(|edge| edge.range.end <= start) {
                j += 1;
            }
            if let Some(edge) = b.bytes.get(j).filter(|edge| edge.range.start <= start) {
                push(&mut result, Task::Pair((source.next, edge.next)), budget)?;
                start = source.range.end.min(edge.range.end);
            } else {
                if !options.validity {
                    let mut stopped = Vec::new();
                    push(&mut stopped, Task::Value(Conditions::NO), budget)?;
                    return Ok(stopped);
                }
                push(&mut result, Task::Value(Conditions::NO), budget)?;
                start = b.bytes.get(j).map_or(source.range.end, |edge| {
                    source.range.end.min(edge.range.start)
                });
            }
            branches += 1;
        }
    }
    for &(source, next) in &a.references {
        for &(destination, rest) in &b.references {
            let value = reference_conditions(source, destination, options, conditions, budget)?;
            push(&mut result, Task::Value(value), budget)?;
            if value == Conditions::NO {
                continue;
            }
            push(&mut result, Task::Pair((next, rest)), budget)?;
            push(
                &mut result,
                Task::Fold {
                    all: true,
                    count: 2,
                },
                budget,
            )?;
        }
        push(
            &mut result,
            Task::Fold {
                all: false,
                count: b.references.len(),
            },
            budget,
        )?;
        branches += 1;
    }
    push(
        &mut result,
        Task::Fold {
            all: !options.validity,
            count: branches,
        },
        budget,
    )?;
    Ok(result)
}
