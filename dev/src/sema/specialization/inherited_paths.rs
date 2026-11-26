//! Closed ancestry orders conformance; inherited members are read only from real parents.
use super::{
    graph_pairs,
    registry_graph::{Forest, Graph, Relation, State},
    retention_work::Budget,
};
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    sema::{
        model::Declarations,
        traits::implementations::{EntryKey, Implementations},
    },
    types::parameters::predicates::Predicate,
};
type Header = (DefId, Option<NodeKey>, Predicate);
pub(super) fn checked<'a>(
    forest: &'a Forest,
    registry: &Implementations,
    key: EntryKey,
) -> Result<Option<&'a super::registry_graph::Vertex>, &'static str> {
    let Some(id) = forest.by_entry.get(&key) else {
        return Ok(None);
    };
    let vertex = forest
        .vertices
        .get(id.index())
        .ok_or("specialization vertex index changed")?;
    let header = registry.checked_entry(key)?;
    if vertex.witness.key != key
        || header.predicate != vertex.witness.predicate
        || header.source != vertex.witness.policy.as_ref().map(|policy| policy.source)
    {
        return Err("specialization candidate header changed after closure");
    }
    Ok(Some(vertex))
}
pub(in crate::sema) fn conformance_order(
    facts: &Declarations,
    headers: &mut [Header],
) -> Result<(), &'static str> {
    let registry = graph_pairs::registry(facts)?;
    let Some(graph) = registry.specialization_graph() else {
        return Ok(());
    };
    graph.current(facts)?;
    let State::Conforming(forest) = &graph.state else {
        return Err("conformance order requires closed specialization comparisons");
    };
    if let Some(imported) = graph.imported.as_deref() {
        order_headers(imported, registry, headers, &graph.budget)?;
    }
    order_headers(forest, registry, headers, &graph.budget)
}

fn order_headers(
    forest: &Forest,
    registry: &Implementations,
    headers: &mut [Header],
    budget: &Budget,
) -> Result<(), &'static str> {
    let count = forest.vertices.len();
    budget.charge(
        headers
            .len()
            .saturating_add(count.saturating_mul(count.max(1).ilog2() as usize + 1)),
        count.saturating_mul(size_of::<usize>() + size_of::<(u32, u32, Header)>()),
    )?;
    let mut positions = Vec::with_capacity(count);
    let mut selected = Vec::with_capacity(count);
    for (index, &header) in headers.iter().enumerate() {
        if let Some(vertex) = checked(forest, registry, registry.entry_key(header.0)?)? {
            positions.push(index);
            selected.push((vertex.depth, vertex.witness.key.ordinal(), header));
        }
    }
    if selected.len() != count {
        return Err("specialization conformance inventory lost a header");
    }
    selected.sort_unstable_by_key(|&(depth, ordinal, _)| (depth, ordinal));
    // Only actual graph slots move; unrelated headers keep their original slots.
    for (position, (_, _, header)) in positions.into_iter().zip(selected) {
        headers[position] = header;
    }
    Ok(())
}
fn conformance_forest(graph: &Graph, entry: EntryKey) -> Option<&Forest> {
    graph.imported_forest(entry).or(match &graph.state {
        State::Conforming(forest) | State::Sealed(forest) => Some(forest),
        _ => None,
    })
}

fn parent(
    forest: &Forest,
    registry: &Implementations,
    entry: EntryKey,
) -> Result<Option<Relation>, &'static str> {
    let Some(edge) = checked(forest, registry, entry)?.and_then(|vertex| vertex.parent) else {
        return Ok(None);
    };
    let relation = *forest
        .parents
        .get(edge.index())
        .ok_or("inherited parent edge disappeared")?;
    if relation.child != entry {
        return Err("inherited parent edge changed child identity");
    }
    Ok(Some(relation))
}

pub(in crate::sema) fn imported_parent(
    facts: &Declarations,
    owner: DefId,
) -> Result<Option<EntryKey>, &'static str> {
    let registry = graph_pairs::registry(facts)?;
    let Some(graph) = registry.specialization_graph() else {
        return Ok(None);
    };
    graph.current(facts)?;
    let key = registry.entry_key(owner)?;
    graph.imported_forest(key).map_or(Ok(None), |forest| {
        parent(forest, registry, key).map(|relation| relation.map(|edge| edge.parent))
    })
}

pub(in crate::sema) fn unfinished_imported_ancestors(
    facts: &Declarations,
    owner: DefId,
) -> Result<Vec<(DefId, Predicate)>, &'static str> {
    let registry = graph_pairs::registry(facts)?;
    let Some(graph) = registry.specialization_graph() else {
        return Ok(Vec::new());
    };
    graph.current(facts)?;
    let mut current = registry.entry_key(owner)?;
    let Some(forest) = graph.imported_forest(current) else {
        return Ok(Vec::new());
    };
    let mut ancestors = Vec::new();
    while let Some(edge) = parent(forest, registry, current)? {
        graph.budget.charge(1, 0)?;
        current = edge.parent;
        if registry.is_validated(current.owner()) {
            break;
        }
        let header = registry.checked_entry(current)?;
        let growth = crate::support::retained::Growth::vector::<(DefId, Predicate)>(
            ancestors.len(),
            ancestors.capacity(),
            1,
        );
        graph.budget.charge(growth.work, growth.bytes)?;
        ancestors.push((header.owner, header.predicate));
    }
    Ok(ancestors)
}

fn inherited_member(
    facts: &Declarations,
    owner: DefId,
    resolves: impl Fn(&Implementations, EntryKey) -> Result<bool, &'static str>,
) -> Result<bool, &'static str> {
    let registry = graph_pairs::registry(facts)?;
    let Some(graph) = registry.specialization_graph() else {
        return Ok(false);
    };
    graph.current(facts)?;
    let mut current = registry.entry_key(owner)?;
    let Some(forest) = conformance_forest(graph, current) else {
        return Ok(false);
    };
    while let Some(edge) = parent(forest, registry, current)? {
        graph.budget.charge(1, 0)?;
        current = edge.parent;
        if resolves(registry, current)? {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(in crate::sema) fn inherited_body(
    facts: &Declarations,
    owner: DefId,
    member: DefId,
) -> Result<bool, &'static str> {
    inherited_member(facts, owner, |registry, parent| {
        Ok(registry.direct_method(parent, member)?.is_some())
    })
}

/// Under `min_specialization` an omitted associated type is final in the
/// nearest defining ancestor; conformance checks it there.
pub(in crate::sema) fn inherited_type(
    facts: &Declarations,
    owner: DefId,
    member: DefId,
) -> Result<bool, &'static str> {
    inherited_member(facts, owner, |_, parent| {
        Ok(facts
            .associated
            .type_definition(parent.owner(), member)
            .is_some())
    })
}
