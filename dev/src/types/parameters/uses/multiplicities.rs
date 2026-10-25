//! Shared canonical nodes contribute capped path counts once in topological order.
use super::dag_parts::{Node, parts};
use crate::{
    support::{hash::HashMap, retained::Growth},
    types::{GenericArgs, Store, parameters::ParameterId},
};
use std::ops::Range;

#[derive(Default)]
struct Record {
    edges: Range<usize>,
    incoming: usize,
    paths: u8,
}
enum Edge {
    Child(usize),
    Parameter(ParameterId),
}
#[derive(Default)]
pub(crate) struct Multiplicities {
    indices: HashMap<Node, usize>,
    nodes: Vec<(Node, Record)>,
    edges: Vec<Edge>,
    parameters: HashMap<ParameterId, u8>,
}
impl Multiplicities {
    /// O(V + E + R + P) demanded-node, edge, root and parameter work. Repeated
    /// edges stay distinct; no per-parameter DAG walk or per-node parameter map.
    pub(crate) fn arguments(
        store: &Store,
        roots: GenericArgs,
        charge: impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        let mut result = Self::default();
        let root = result.node(Node::Arguments(roots), &charge)?;
        result.nodes[root].1.paths = 1;
        let mut cursor = 0;
        while cursor < result.nodes.len() {
            result.expand(store, cursor, &charge)?;
            cursor += 1;
        }
        result.propagate(&charge)?;
        Ok(result)
    }
    pub(crate) fn repeated(
        &self,
        charge: impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<bool, &'static str> {
        charge(self.parameters.len(), 0)?;
        Ok(self.parameters.values().any(|&count| count > 1))
    }
    fn node(
        &mut self,
        node: Node,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<usize, &'static str> {
        charge(1, 0)?;
        if let Some(&index) = self.indices.get(&node) {
            return Ok(index);
        }
        let mut growth = Growth::map::<Node, usize>(self.indices.len(), self.indices.capacity(), 1);
        growth.include(Growth::vector::<(Node, Record)>(
            self.nodes.len(),
            self.nodes.capacity(),
            1,
        ));
        charge(growth.work, growth.bytes)?;
        let index = self.nodes.len();
        self.nodes.push((node, Record::default()));
        self.indices.insert(node, index);
        Ok(index)
    }
    fn expand(
        &mut self,
        store: &Store,
        index: usize,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let mut children = Vec::new();
        let mut parameters = Vec::new();
        let mut visits = 0;
        let result = parts(
            store,
            self.nodes[index].0,
            &mut visits,
            |parameter| push(&mut parameters, parameter, charge),
            |child| push(&mut children, child, charge),
        );
        // Failed canonical identity and scratch-growth work remains charged.
        charge(1 + visits, 0)?;
        result?;
        let growth = Growth::vector::<Edge>(
            self.edges.len(),
            self.edges.capacity(),
            children.len().saturating_add(parameters.len()),
        );
        charge(growth.work, growth.bytes)?;
        let start = self.edges.len();
        for child in children {
            let child = self.node(child, charge)?;
            self.nodes[child].1.incoming = self.nodes[child]
                .1
                .incoming
                .checked_add(1)
                .ok_or("parameter path indegree overflow")?;
            self.edges.push(Edge::Child(child));
        }
        self.edges
            .extend(parameters.into_iter().map(Edge::Parameter));
        self.nodes[index].1.edges = start..self.edges.len();
        Ok(())
    }
    fn propagate(
        &mut self,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        charge(
            self.nodes.len(),
            self.nodes.len().saturating_mul(size_of::<usize>()),
        )?;
        let mut ready = Vec::with_capacity(self.nodes.len());
        ready.extend(
            self.nodes
                .iter()
                .enumerate()
                .filter_map(|(i, (_, node))| (node.incoming == 0).then_some(i)),
        );
        let mut cursor = 0;
        while let Some(&index) = ready.get(cursor) {
            cursor += 1;
            let node = &self.nodes[index].1;
            let paths = node.paths;
            let range = node.edges.clone();
            charge(1 + range.len(), 0)?;
            for edge in &self.edges[range] {
                match *edge {
                    Edge::Child(child) => {
                        let node = &mut self.nodes[child].1;
                        node.paths = node.paths.saturating_add(paths).min(2);
                        node.incoming = node
                            .incoming
                            .checked_sub(1)
                            .ok_or("parameter path indegree changed")?;
                        if node.incoming == 0 {
                            ready.push(child);
                        }
                    }
                    Edge::Parameter(parameter) => {
                        let growth = Growth::map::<ParameterId, u8>(
                            self.parameters.len(),
                            self.parameters.capacity(),
                            usize::from(!self.parameters.contains_key(&parameter)),
                        );
                        charge(growth.work, growth.bytes)?;
                        let count = self.parameters.entry(parameter).or_default();
                        *count = count.saturating_add(paths).min(2);
                    }
                }
            }
        }
        if cursor != self.nodes.len() {
            return Err("cyclic canonical parameter occurrence graph");
        }
        Ok(())
    }
}
fn push<T>(
    values: &mut Vec<T>,
    value: T,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    let growth = Growth::vector::<T>(values.len(), values.capacity(), 1);
    charge(growth.work, growth.bytes)?;
    values.push(value);
    Ok(())
}

#[cfg(test)]
#[path = "../../../../tests/invariants/specialization_occurrences.rs"]
mod tests;
