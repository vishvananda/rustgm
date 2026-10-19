//! Linear, iterative SCCs of assumptions. Required edges never grant permissions.
use super::{Edge, Error, Graph, Region, Solution};

impl Solution {
    /// Only a completed proof may add required edges to the equality relation.
    /// Their mutual reachability is evidence, not a choice of inference values.
    pub(crate) fn established_classes(&self) -> Result<Vec<Region>, Error> {
        let mut graph = Graph::new(self.graph.limit);
        graph.kinds.clone_from(&self.graph.kinds);
        graph.assumptions.clone_from(&self.graph.assumptions);
        graph.assumed_heads.clone_from(&self.graph.assumed_heads);
        for (shorter, &head) in self.graph.incoming.iter().enumerate() {
            if shorter == Region::LOCAL.0 as usize {
                continue;
            }
            let mut edge = head;
            while let Some(index) = edge {
                let requirement = self.graph.obligations[index];
                edge = requirement.next;
                // LOCAL is an abstract contract sentinel. Runtime borrow extents
                // remain independent and cannot become equal through this node.
                if requirement.target == Region::LOCAL {
                    continue;
                }
                let longer = requirement.target.0 as usize;
                graph.assumptions.push(Edge {
                    target: Region(u32::try_from(shorter).map_err(|_| Error::Budget)?),
                    source: requirement.source,
                    next: graph.assumed_heads[longer],
                });
                graph.assumed_heads[longer] = Some(graph.assumptions.len() - 1);
            }
        }
        graph.assumption_classes()
    }
}

struct Frame {
    node: usize,
    edge: Option<usize>,
}
struct Search<'a> {
    graph: &'a Graph,
    numbers: Vec<usize>,
    low: Vec<usize>,
    active: Vec<bool>,
    stack: Vec<usize>,
    frames: Vec<Frame>,
    classes: Vec<Region>,
    members: Vec<usize>,
    next: usize,
    static_target: usize,
    steps: usize,
}
impl Graph {
    /// Compute mutual entailment from declared/implied bounds only. Static
    /// intrinsically outlives all nodes, including unmentioned universals.
    pub(crate) fn assumption_classes(&self) -> Result<Vec<Region>, Error> {
        Search {
            graph: self,
            numbers: vec![0; self.kinds.len()],
            low: vec![0; self.kinds.len()],
            active: vec![false; self.kinds.len()],
            stack: Vec::new(),
            frames: Vec::new(),
            classes: vec![Region::STATIC; self.kinds.len()],
            members: Vec::new(),
            next: 0,
            static_target: 1,
            steps: 0,
        }
        .run()
    }
}
impl Search<'_> {
    fn enter(&mut self, node: usize) {
        self.next += 1;
        self.numbers[node] = self.next;
        self.low[node] = self.next;
        self.active[node] = true;
        self.stack.push(node);
        self.frames.push(Frame {
            node,
            edge: self.graph.assumed_heads[node],
        });
    }
    fn run(mut self) -> Result<Vec<Region>, Error> {
        // Static reaches every vertex; one traversal covers the entire graph.
        self.enter(0);
        while let Some(frame) = self.frames.last_mut() {
            self.steps = self.steps.saturating_add(1);
            if self.steps > self.graph.limit {
                return Err(Error::Budget);
            }
            let node = frame.node;
            let target = if let Some(edge) = frame.edge {
                let edge = self.graph.assumptions[edge];
                frame.edge = edge.next;
                Some(edge.target.0 as usize)
            } else if node == 0 && self.static_target < self.numbers.len() {
                let target = self.static_target;
                self.static_target += 1;
                Some(target)
            } else {
                None
            };
            if let Some(target) = target {
                if self.numbers[target] == 0 {
                    self.enter(target);
                } else if self.active[target] {
                    self.low[node] = self.low[node].min(self.numbers[target]);
                }
            } else {
                self.finish(node)?;
            }
        }
        Ok(self.classes)
    }
    fn finish(&mut self, node: usize) -> Result<(), Error> {
        self.frames.pop();
        if self.low[node] == self.numbers[node] {
            self.members.clear();
            let mut representative = node;
            loop {
                let member = self.stack.pop().ok_or(Error::Identity)?;
                self.active[member] = false;
                self.members.push(member);
                representative = representative.min(member);
                if member == node {
                    break;
                }
            }
            let representative = Region(u32::try_from(representative).map_err(|_| Error::Budget)?);
            for &member in &self.members {
                self.classes[member] = representative;
            }
        }
        if let Some(parent) = self.frames.last() {
            self.low[parent.node] = self.low[parent.node].min(self.low[node]);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/assumption_components.rs"]
mod tests;
