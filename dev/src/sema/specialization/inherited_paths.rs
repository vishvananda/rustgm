//! Sealed ancestry orders successful candidates; conformance reads only real parent members.
use super::{
    graph_pairs,
    registry_graph::{Forest, State},
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
        || header.source != Some(vertex.witness.policy.source)
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
    let count = forest.vertices.len();
    graph.budget.charge(
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
pub(in crate::sema) fn inherited_body(
    facts: &Declarations,
    owner: DefId,
    member: DefId,
) -> Result<bool, &'static str> {
    let registry = graph_pairs::registry(facts)?;
    let Some(graph) = registry.specialization_graph() else {
        return Ok(false);
    };
    graph.current(facts)?;
    let (State::Conforming(forest) | State::Sealed(forest)) = &graph.state else {
        return Ok(false);
    };
    let mut key = registry.entry_key(owner)?;
    loop {
        graph.budget.charge(1, 0)?;
        let Some(vertex) = checked(forest, registry, key)? else {
            return Ok(false);
        };
        let Some(edge) = vertex.parent else {
            return Ok(false);
        };
        let relation = forest
            .parents
            .get(edge.index())
            .ok_or("missing inherited method edge")?;
        if relation.child != key {
            return Err("inherited method edge changed child identity");
        }
        key = relation.parent;
        if registry.direct_method(key, member)?.is_some() {
            return Ok(true);
        }
    }
}
