//! One bounded forest walk seals ranks and ancestry without copying ancestor bodies.
use super::{
    registry_graph::{Forest, VertexId},
    retention_work::Budget,
};
use crate::support::retained::Growth;

pub(super) fn number(forest: &mut Forest, budget: &Budget) -> Result<(), &'static str> {
    let count = forest.vertices.len();
    budget.charge(count, count.saturating_mul(size_of::<Vec<VertexId>>()))?;
    let mut children = vec![Vec::new(); count];
    let mut roots = Vec::new();
    for (index, vertex) in forest.vertices.iter().enumerate() {
        let id = VertexId::new(index)?;
        if let Some(edge) = vertex.parent {
            let relation = forest
                .parents
                .get(edge.index())
                .ok_or("missing immediate parent edge")?;
            if relation.child != vertex.witness.key {
                return Err("immediate parent child changed");
            }
            let parent = forest
                .by_entry
                .get(&relation.parent)
                .ok_or("missing immediate parent vertex")?;
            push(&mut children[parent.index()], id, budget)?;
        } else {
            push(&mut roots, id, budget)?;
        }
    }
    let mut stack = Vec::new();
    for root in roots.into_iter().rev() {
        push(&mut stack, (root, false), budget)?;
    }
    let mut next = 0_u32;
    while let Some((id, finish)) = stack.pop() {
        budget.charge(1, 0)?;
        let vertex = &mut forest.vertices[id.index()];
        if finish {
            vertex.subtree_end = next;
            continue;
        }
        if vertex.subtree_end != 0 || vertex.preorder != 0 {
            return Err("specialization forest visits a vertex twice");
        }
        next = next
            .checked_add(1)
            .ok_or("specialization ancestry rank overflow")?;
        vertex.preorder = next;
        let depth = vertex
            .depth
            .checked_add(1)
            .ok_or("specialization depth overflow")?;
        push(&mut stack, (id, true), budget)?;
        for &child in children[id.index()].iter().rev() {
            forest.vertices[child.index()].depth = depth;
            push(&mut stack, (child, false), budget)?;
        }
    }
    if next as usize != count {
        return Err("cyclic specialization parent graph");
    }
    Ok(())
}
fn push<T>(values: &mut Vec<T>, value: T, budget: &Budget) -> Result<(), &'static str> {
    let growth = Growth::vector::<T>(values.len(), values.capacity(), 1);
    budget.charge(1 + growth.work, growth.bytes)?;
    values.push(value);
    Ok(())
}
