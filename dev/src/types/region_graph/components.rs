//! Linear, iterative SCCs of assumptions. Required edges never grant permissions.
use super::{Edge, Error, Graph, Region, Solution};

impl Solution {
    /// Runtime propagation follows only requirements, excluding the abstract LOCAL
    /// sentinel. Component ranks decrease along every inter-component edge.
    pub(crate) fn runtime_classes(
        &self,
        limit: usize,
    ) -> Result<(Vec<Region>, Vec<usize>, usize), Error> {
        let mut search = Search::new(
            &self.graph,
            &self.graph.incoming,
            &self.graph.obligations,
            false,
            true,
        );
        search.limit = search.limit.min(limit);
        search.exclude_local = true;
        search.run()
    }

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
    heads: &'a [Option<usize>],
    edges: &'a [Edge],
    limit: usize,
    numbers: Vec<usize>,
    low: Vec<usize>,
    active: Vec<bool>,
    stack: Vec<usize>,
    frames: Vec<Frame>,
    classes: Vec<Region>,
    ranks: Vec<usize>,
    components: usize,
    members: Vec<usize>,
    next: usize,
    static_target: Option<usize>,
    steps: usize,
    exclude_local: bool,
}
impl Graph {
    /// Compute mutual entailment from declared/implied bounds only. Static
    /// intrinsically outlives all nodes, including unmentioned universals.
    pub(crate) fn assumption_classes(&self) -> Result<Vec<Region>, Error> {
        self.classes(&self.assumed_heads, &self.assumptions, true, false)
            .map(|(classes, _, _)| classes)
    }

    pub(super) fn required_classes(&self) -> Result<(Vec<Region>, Vec<usize>, usize), Error> {
        // Reversing every required edge preserves its strongly connected classes.
        self.classes(&self.incoming, &self.obligations, false, true)
    }

    pub(super) fn classes(
        &self,
        heads: &[Option<usize>],
        edges: &[Edge],
        static_reaches_all: bool,
        ranked: bool,
    ) -> Result<(Vec<Region>, Vec<usize>, usize), Error> {
        Search::new(self, heads, edges, static_reaches_all, ranked).run()
    }
}

impl<'a> Search<'a> {
    fn new(
        graph: &Graph,
        heads: &'a [Option<usize>],
        edges: &'a [Edge],
        static_reaches_all: bool,
        ranked: bool,
    ) -> Self {
        Self {
            heads,
            edges,
            limit: graph.limit,
            numbers: vec![0; graph.kinds.len()],
            low: vec![0; graph.kinds.len()],
            active: vec![false; graph.kinds.len()],
            stack: Vec::new(),
            frames: Vec::new(),
            classes: vec![Region::STATIC; graph.kinds.len()],
            ranks: if ranked {
                vec![0; graph.kinds.len()]
            } else {
                Vec::new()
            },
            components: 0,
            members: Vec::new(),
            next: 0,
            static_target: static_reaches_all.then_some(1),
            steps: 0,
            exclude_local: false,
        }
    }
    fn enter(&mut self, node: usize) {
        self.next += 1;
        self.numbers[node] = self.next;
        self.low[node] = self.next;
        self.active[node] = true;
        self.stack.push(node);
        self.frames.push(Frame {
            node,
            edge: if self.exclude_local && node == Region::LOCAL.0 as usize {
                None
            } else {
                self.heads[node]
            },
        });
    }
    fn run(mut self) -> Result<(Vec<Region>, Vec<usize>, usize), Error> {
        for root in 0..self.numbers.len() {
            if self.numbers[root] == 0 {
                self.enter(root);
                self.traverse()?;
            }
        }
        Ok((self.classes, self.ranks, self.steps))
    }
    fn traverse(&mut self) -> Result<(), Error> {
        while let Some(frame) = self.frames.last_mut() {
            self.steps = self.steps.saturating_add(1);
            if self.steps > self.limit {
                return Err(Error::Budget);
            }
            let node = frame.node;
            let target = if let Some(edge) = frame.edge {
                let edge = self.edges[edge];
                frame.edge = edge.next;
                Some(edge.target.0 as usize)
            } else if node == 0 {
                self.static_target.and_then(|target| {
                    if target < self.numbers.len() {
                        self.static_target = Some(target + 1);
                        Some(target)
                    } else {
                        None
                    }
                })
            } else {
                None
            };
            if let Some(target) = target {
                if self.exclude_local && target == Region::LOCAL.0 as usize {
                    continue;
                }
                if self.numbers[target] == 0 {
                    self.enter(target);
                } else if self.active[target] {
                    self.low[node] = self.low[node].min(self.numbers[target]);
                }
            } else {
                self.finish(node)?;
            }
        }
        Ok(())
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
                if !self.ranks.is_empty() {
                    self.ranks[member] = self.components;
                }
            }
            self.components += 1;
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
