//! Early and final coherence queues share one retained comparison per actual pair.
use super::{
    proof_attempt::{Attempt, Progress},
    registry_graph::{Comparison, EdgeId, Graph, PairKey, Relation, State},
};
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::{
        model::Declarations,
        traits::implementations::{Implementations, Polarity},
    },
    support::retained::Growth,
};
pub(super) fn registry(facts: &Declarations) -> Result<&Implementations, &'static str> {
    facts
        .generics
        .implementations
        .as_deref()
        .ok_or("specialization registry disappeared")
}
fn pair(facts: &Declarations, owners: [DefId; 2]) -> Result<PairKey, &'static str> {
    let registry = registry(facts)?;
    PairKey::new(
        registry.entry_key(owners[0])?,
        registry.entry_key(owners[1])?,
    )
}
pub(super) fn active_graph(facts: &mut Declarations) -> Result<&mut Graph, &'static str> {
    facts
        .generics
        .implementations
        .as_deref_mut()
        .and_then(Implementations::specialization_graph_mut)
        .ok_or("specialization graph disappeared")
}
pub(in crate::sema) fn requested(
    report: &Report,
    owners: [DefId; 2],
    steps: &mut usize,
) -> Result<bool, String> {
    let requests = crate::resolve::specialization_authority::Requests::new(report)
        .map_err(|error| error.to_string())?;
    let mut session = report.macro_allowances();
    for owner in owners {
        if requests
            .permits_comparison(report, &mut session, owner, |work, _| {
                *steps = steps.saturating_add(work);
                if *steps > 1_000_000 {
                    return Err("trait implementation overlap work budget exceeded");
                }
                Ok(())
            })
            .map_err(|error| error.to_string())?
        {
            return Ok(true);
        }
    }
    Ok(false)
}
pub(in crate::sema) fn remembered(
    facts: &mut Declarations,
    report: &Report,
    owners: [DefId; 2],
) -> Result<bool, String> {
    let Some(existing) = registry(facts)?.specialization_graph() else {
        return Ok(false);
    };
    existing.current(facts)?;
    let key = pair(facts, owners)?;
    let State::Building(builder) = &existing.state else {
        return Err("coherence comparison arrived after specialization closure".into());
    };
    if !builder.attempts.contains_key(&key) {
        return Ok(false);
    }
    authenticate(facts, report, key)?;
    advance(facts, key)?;
    Ok(true)
}
pub(in crate::sema) fn compare(
    facts: &mut Declarations,
    report: &Report,
    owners: [DefId; 2],
) -> Result<(), String> {
    if registry(facts)?.specialization_graph().is_none() {
        let graph = Graph::new(facts, report)?;
        facts
            .generics
            .implementations
            .as_deref_mut()
            .ok_or("specialization registry disappeared")?
            .install_specialization_graph(graph)?;
    }
    let key = pair(facts, owners)?;
    authenticate(facts, report, key)?;
    let graph = active_graph(facts)?;
    let budget = graph.budget.alias()?;
    let builder = graph.building()?;
    if !builder.attempts.contains_key(&key) {
        let growth = Growth::map::<PairKey, Comparison>(
            builder.attempts.len(),
            builder.attempts.capacity(),
            1,
        );
        budget.charge(growth.work, growth.bytes)?;
        let attempt = Attempt::new(key, budget.alias()?)?;
        builder.attempts.insert(
            key,
            Comparison::Pending {
                attempt,
                dependency: None,
            },
        );
    }
    advance(facts, key)
}
fn authenticate(facts: &mut Declarations, report: &Report, key: PairKey) -> Result<(), String> {
    for entry in key.entries() {
        if registry(facts)?.polarity(&facts.types, &facts.associated, entry.owner())?
            != Polarity::Positive
        {
            return Err("negative implementation cannot participate in specialization".into());
        }
        let witness = registry(facts)?
            .specialization_graph()
            .ok_or("specialization graph disappeared")?
            .witness(facts, report, entry)?;
        if let Some(witness) = witness {
            active_graph(facts)?
                .building()?
                .sources
                .insert(entry, witness);
        }
    }
    Ok(())
}
fn advance(facts: &mut Declarations, key: PairKey) -> Result<(), String> {
    let graph = active_graph(facts)?;
    let comparison = graph
        .building()?
        .attempts
        .get_mut(&key)
        .ok_or("specialization comparison was not retained")?;
    let mut attempt = match std::mem::replace(comparison, Comparison::Running) {
        Comparison::Pending {
            attempt,
            dependency: _,
        } => attempt,
        completed @ Comparison::Ordered(_) => {
            *comparison = completed;
            return Ok(());
        }
        Comparison::Running => {
            return Err("recursive specialization comparison is unresolved".into());
        }
    };
    // The Graph stays installed while inference mutates Declarations. The one
    // owned attempt carries its inputs, dependency and already-paid work.
    let result = attempt.advance(facts);
    let graph = active_graph(facts)?;
    let budget = graph.budget.alias()?;
    let builder = graph.building()?;
    match result {
        Ok(Progress::Pending(dependency)) => {
            builder.attempts.insert(
                key,
                Comparison::Pending {
                    attempt,
                    dependency: Some(dependency),
                },
            );
        }
        Ok(Progress::Complete(Some(relation))) => {
            // A selection may order its pair before coherence authenticates
            // the sources; closure authenticates every child before admission.
            if builder
                .sources
                .get(&relation.child)
                .is_some_and(|witness| !witness.policy.may_specialize())
            {
                graph.fail();
                return Err(
                    "specializing implementation requires its actual feature authority".into(),
                );
            }
            let growth = Growth::vector::<Relation>(
                builder.relations.len(),
                builder.relations.capacity(),
                1,
            );
            budget.charge(growth.work, growth.bytes)?;
            let edge = EdgeId::new(builder.relations.len())?;
            builder.relations.push(relation);
            builder.attempts.insert(key, Comparison::Ordered(edge));
        }
        Ok(Progress::Complete(None)) => {
            graph.fail();
            return Err(
                "overlapping implementation headers have no specialization ordering".into(),
            );
        }
        Err(error) => {
            graph.fail();
            return Err(error);
        }
    }
    Ok(())
}
pub(in crate::sema) fn drain(facts: &mut Declarations, report: &Report) -> Result<(), String> {
    let Some(graph) = registry(facts)?.specialization_graph() else {
        return Ok(());
    };
    graph.current(facts)?;
    let State::Building(builder) = &graph.state else {
        return Ok(());
    };
    graph.budget.charge(
        builder.attempts.len(),
        builder.attempts.len().saturating_mul(size_of::<PairKey>()),
    )?;
    let keys: Vec<_> = builder
        .attempts
        .iter()
        .map(|(&key, value)| (key, matches!(value, Comparison::Pending { .. })))
        .collect();
    for (key, pending) in keys {
        authenticate(facts, report, key)?;
        if pending {
            advance(facts, key)?;
        }
    }
    let graph = active_graph(facts)?;
    let budget = graph.budget.alias()?;
    let builder = graph.building()?;
    budget.charge(builder.relations.len(), 0)?;
    if builder.relations.iter().any(|relation| {
        builder
            .sources
            .get(&relation.child)
            .is_none_or(|witness| !witness.policy.may_specialize())
    }) {
        graph.fail();
        return Err("specializing implementation requires its actual feature authority".into());
    }
    if graph
        .building()?
        .attempts
        .values()
        .any(|value| matches!(value, Comparison::Pending { .. } | Comparison::Running))
    {
        let pending = graph
            .building()?
            .attempts
            .values()
            .find_map(|value| match value {
                Comparison::Pending { dependency, .. } => Some(*dependency),
                _ => None,
            });
        return Err(format!(
            "specialization implication still awaits a declaration dependency: {pending:?}"
        ));
    }
    Ok(())
}
pub(super) enum Settled {
    Ordered(Relation),
    Waiting(crate::sema::traits::selection::Dependency),
    Undecided,
}
/// Complete one pair's comparison for a selection that proved both candidates.
/// Closure authenticates its source witnesses before any relation is admitted;
/// a recursive or ambiguous comparison leaves the pair undecided.
pub(super) fn settle(facts: &mut Declarations, key: PairKey) -> Result<Settled, String> {
    let graph = active_graph(facts)?;
    let budget = graph.budget.alias()?;
    let builder = graph.building()?;
    match builder.attempts.get(&key) {
        Some(
            Comparison::Running
            | Comparison::Pending {
                dependency: None, ..
            },
        ) => {
            return Ok(Settled::Undecided);
        }
        Some(Comparison::Ordered(_) | Comparison::Pending { .. }) => {}
        None => {
            let growth = Growth::map::<PairKey, Comparison>(
                builder.attempts.len(),
                builder.attempts.capacity(),
                1,
            );
            budget.charge(growth.work, growth.bytes)?;
            let attempt = Attempt::new(key, budget.alias()?)?;
            builder.attempts.insert(
                key,
                Comparison::Pending {
                    attempt,
                    dependency: None,
                },
            );
        }
    }
    advance(facts, key)?;
    let builder = active_graph(facts)?.building()?;
    Ok(match builder.attempts.get(&key) {
        Some(Comparison::Ordered(edge)) => Settled::Ordered(
            *builder
                .relations
                .get(edge.index())
                .ok_or("specialization comparison lost its relation")?,
        ),
        Some(Comparison::Pending {
            dependency: Some(dependency),
            ..
        }) => Settled::Waiting(*dependency),
        _ => Settled::Undecided,
    })
}
