//! Rustc orders two applicable candidates by `specializes`, never by a finished
//! graph. Before closure one completed pairwise comparison orders them; a closed
//! forest orders them by rank. Finality and Min admission still close the graph.
use super::{
    graph_pairs::{self, Settled},
    member_paths::checked,
    registry_graph::{Comparison, Graph, PairKey, Relation, State},
    retention_work::Budget,
};
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::{
        model::Declarations,
        traits::{
            implementations::{EntryKey, Implementations, Polarity},
            selection::Dependency,
        },
    },
    support::hash::HashSet,
    types::GenericArgs,
};

pub(in crate::sema) enum Order {
    Prefer(DefId),
    Unordered,
    /// The pair's own comparison waits for this declaration fact.
    Await(Dependency),
}

/// A crate that requests specialization owns its graph before declarations are
/// checked, so their selections can order candidates as rustc's do.
pub(in crate::sema) fn open(facts: &mut Declarations, report: &Report) -> Result<(), String> {
    let Some(registry) = facts.generics.implementations.as_deref() else {
        return Ok(());
    };
    if registry.specialization_graph().is_some() {
        return Ok(());
    }
    let (minimal, full) = report.package.specialization_requests()?;
    if !minimal && !full {
        return Ok(());
    }
    let graph = Graph::new(facts, report)?;
    facts
        .generics
        .implementations
        .as_deref_mut()
        .ok_or("specialization registry disappeared")?
        .install_specialization_graph(graph)?;
    Ok(())
}

/// Order two candidates proven for one goal, either fully known or resolved to
/// identical inputs by both. `compare` lets this query complete the pair's own
/// comparison; other queries read only results.
pub(in crate::sema) fn order(
    facts: &mut Declarations,
    previous: DefId,
    next: DefId,
    compare: bool,
) -> Result<Order, String> {
    let registry = graph_pairs::registry(facts)?;
    let Some(graph) = registry.specialization_graph() else {
        return Ok(Order::Unordered);
    };
    graph.current(facts)?;
    graph.budget.charge(2, 0)?;
    let a = registry.entry_key(previous)?;
    let b = registry.entry_key(next)?;
    let key = match &graph.state {
        State::Sealed(forest) | State::Conforming(forest) => {
            let (Some(x), Some(y)) = (checked(forest, registry, a)?, checked(forest, registry, b)?)
            else {
                return Ok(Order::Unordered);
            };
            graph.budget.charge(2, 0)?;
            return Ok(if x.preorder < y.preorder && y.preorder <= x.subtree_end {
                Order::Prefer(next)
            } else if y.preorder < x.preorder && x.preorder <= y.subtree_end {
                Order::Prefer(previous)
            } else {
                Order::Unordered
            });
        }
        State::Building(builder) => {
            let key = PairKey::new(a, b)?;
            if let Some(Comparison::Ordered(edge)) = builder.attempts.get(&key) {
                let relation = builder
                    .relations
                    .get(edge.index())
                    .ok_or("specialization comparison lost its relation")?;
                return Ok(Order::Prefer(relation.child.owner()));
            }
            if !compare || !graph.requests.crate_request() || !local(facts, registry, [a, b])? {
                return Ok(Order::Unordered);
            }
            key
        }
        State::Failed => return Err("specialization graph validation previously failed".into()),
    };
    Ok(match graph_pairs::settle(facts, key)? {
        Settled::Ordered(relation) => Order::Prefer(relation.child.owner()),
        Settled::Waiting(dependency) => Order::Await(dependency),
        Settled::Undecided => Order::Unordered,
    })
}
/// Only positive local impls have source witnesses that closure authenticates.
fn local(
    facts: &Declarations,
    registry: &Implementations,
    entries: [EntryKey; 2],
) -> Result<bool, &'static str> {
    for entry in entries {
        if registry.checked_entry(entry)?.source.is_none()
            || registry.polarity(&facts.types, &facts.associated, entry.owner())?
                != Polarity::Positive
        {
            return Ok(false);
        }
    }
    Ok(true)
}
/// A client validates an imported impl on demand only outside every consumer
/// relation. `local` and witness capture keep imported impls out of them, so a
/// break of that property becomes this diagnostic, never a silent mis-order.
pub(in crate::sema) fn require_unordered_import(
    facts: &Declarations,
    owner: DefId,
) -> Result<(), &'static str> {
    let registry = graph_pairs::registry(facts)?;
    let Some(graph) = registry.specialization_graph() else {
        return Ok(());
    };
    if graph.relates(registry.entry_key(owner)?)? {
        return Err(
            "imported implementation validation before its specialization conformance order",
        );
    }
    Ok(())
}

/// Rustc's `assoc_def`: an impl that omits an associated type inherits its
/// nearest defining ancestor's, with arguments translated through each
/// completed relation. `None` keeps the selected impl's own definition.
pub(in crate::sema) fn defining_type(
    facts: &mut Declarations,
    owner: DefId,
    arguments: GenericArgs,
    item: DefId,
) -> Result<Option<(DefId, GenericArgs)>, &'static str> {
    if facts.associated.type_definition(owner, item).is_some() {
        return Ok(None);
    }
    let registry = graph_pairs::registry(facts)?;
    let Some(graph) = registry.specialization_graph() else {
        return Ok(None);
    };
    graph.current(facts)?;
    let budget = graph.budget.alias()?;
    let start = registry.entry_key(owner)?;
    let defines = |key: EntryKey| {
        facts
            .associated
            .type_definition(key.owner(), item)
            .is_some()
    };
    let path = match &graph.state {
        State::Sealed(forest) | State::Conforming(forest) => {
            let mut path = Vec::new();
            let mut current = start;
            loop {
                budget.charge(1, size_of::<Relation>())?;
                let Some(edge) =
                    checked(forest, registry, current)?.and_then(|vertex| vertex.parent)
                else {
                    break None;
                };
                let relation = *forest
                    .parents
                    .get(edge.index())
                    .ok_or("inherited type parent edge disappeared")?;
                path.push(relation);
                if defines(relation.parent) {
                    break Some(path);
                }
                current = relation.parent;
            }
        }
        State::Building(builder) => search(&builder.relations, start, defines, &budget)?,
        State::Failed => return Err("specialization graph validation previously failed"),
    };
    let Some(path) = path else {
        return Ok(None);
    };
    let mut arguments = arguments;
    let mut defining = owner;
    for relation in path {
        arguments = super::selected_member::translate(facts, relation, arguments, &budget)?;
        defining = relation.parent.owner();
    }
    Ok(Some((defining, arguments)))
}
/// Before closure, each contested selection recorded `child -> parent`
/// relations along its winners, so every applicable ancestor is reachable.
fn search(
    relations: &[Relation],
    start: EntryKey,
    defines: impl Fn(EntryKey) -> bool,
    budget: &Budget,
) -> Result<Option<Vec<Relation>>, &'static str> {
    let mut path: Vec<Relation> = Vec::new();
    let mut cursors = vec![0_usize];
    let mut visited = HashSet::default();
    visited.insert(start);
    while let Some(cursor) = cursors.last_mut() {
        let current = path.last().map_or(start, |relation| relation.parent);
        budget.charge(
            relations.len().saturating_sub(*cursor).max(1),
            size_of::<Relation>() + size_of::<usize>(),
        )?;
        let found = relations
            .iter()
            .enumerate()
            .skip(*cursor)
            .find(|(_, relation)| relation.child == current && !visited.contains(&relation.parent));
        let Some((index, &relation)) = found else {
            cursors.pop();
            path.pop();
            continue;
        };
        *cursor = index + 1;
        path.push(relation);
        if defines(relation.parent) {
            return Ok(Some(path));
        }
        visited.insert(relation.parent);
        cursors.push(0);
    }
    Ok(None)
}
