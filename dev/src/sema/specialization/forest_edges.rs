//! Immediate parents come only from already-paid directional comparisons.
use super::{
    registry_graph::{Builder, Comparison, EdgeId, Forest, PairKey, Relation, Vertex, VertexId},
    retention_work::Budget,
};
use crate::{
    sema::traits::implementations::EntryKey,
    support::{
        hash::HashMap,
        retained::{self, Growth},
    },
};

pub(super) fn build(builder: Builder, budget: &Budget) -> Result<Forest, &'static str> {
    let count = builder.sources.len();
    budget.charge(count, retained::new_map::<EntryKey, EdgeId>(count))?;
    let mut immediate = HashMap::default();
    for (index, relation) in builder.relations.iter().enumerate() {
        budget.charge(1, 0)?;
        let edge = EdgeId::new(index)?;
        if let Some(previous) = immediate.get(&relation.child).copied() {
            let selected = nearest(&builder, previous, edge, budget)?;
            immediate.insert(relation.child, selected);
        } else {
            immediate.insert(relation.child, edge);
        }
    }
    budget.charge(
        count.saturating_mul(count.max(1).ilog2() as usize + 1),
        count
            .saturating_mul(size_of::<super::registry_graph::Witness>())
            .saturating_add(count.saturating_mul(size_of::<Vertex>()))
            .saturating_add(retained::new_map::<EntryKey, VertexId>(count)),
    )?;
    let mut sources: Vec<_> = builder.sources.into_values().collect();
    sources.sort_unstable_by_key(|witness| witness.key.ordinal());
    let mut forest = Forest {
        vertices: Vec::with_capacity(count),
        parents: Vec::new(),
        by_entry: HashMap::default(),
        families: HashMap::default(),
    };
    for witness in sources {
        let parent = if let Some(old) = immediate.get(&witness.key) {
            let relation = *builder
                .relations
                .get(old.index())
                .ok_or("specialization edge disappeared")?;
            let growth =
                Growth::vector::<Relation>(forest.parents.len(), forest.parents.capacity(), 1);
            budget.charge(growth.work, growth.bytes)?;
            let new = EdgeId::new(forest.parents.len())?;
            forest.parents.push(relation);
            Some(new)
        } else {
            None
        };
        let id = VertexId::new(forest.vertices.len())?;
        forest.by_entry.insert(witness.key, id);
        forest.vertices.push(Vertex {
            witness,
            parent,
            depth: 0,
            preorder: 0,
            subtree_end: 0,
        });
    }
    Ok(forest)
}
fn nearest(
    builder: &Builder,
    old: EdgeId,
    new: EdgeId,
    budget: &Budget,
) -> Result<EdgeId, &'static str> {
    let old_parent = builder
        .relations
        .get(old.index())
        .ok_or("missing earlier parent relation")?
        .parent;
    let new_parent = builder
        .relations
        .get(new.index())
        .ok_or("missing later parent relation")?
        .parent;
    if old_parent == new_parent {
        return Ok(old);
    }
    budget.charge(1, 0)?;
    let pair = PairKey::new(old_parent, new_parent)?;
    let Some(Comparison::Ordered(edge)) = builder.attempts.get(&pair) else {
        return Err("specialization parents are incomparable or still unresolved");
    };
    let relation = builder
        .relations
        .get(edge.index())
        .ok_or("missing parent comparison relation")?;
    if relation.child == old_parent && relation.parent == new_parent {
        Ok(old)
    } else if relation.child == new_parent && relation.parent == old_parent {
        Ok(new)
    } else {
        Err("specialization parent comparison changed identity")
    }
}
