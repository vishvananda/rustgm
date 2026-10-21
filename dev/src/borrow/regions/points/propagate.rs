//! Complete each required component before sending its point words along the DAG.
use super::{HashMap, HashSet, Region, Solution, Solver};

struct Edge {
    target: Region,
    next: Option<usize>,
}
struct Word {
    offset: u32,
    next: Option<usize>,
}
struct Plan {
    classes: Vec<Region>,
    ranks: Vec<usize>,
    order: Vec<Region>,
    edges: Vec<Edge>,
    heads: Vec<Option<usize>>,
    words: Vec<Word>,
    word_heads: Vec<Option<usize>>,
}
impl Plan {
    fn space(&self, solver: &Solver, extra: usize) -> Result<(), &'static str> {
        let entries = self
            .classes
            .len()
            .saturating_mul(5)
            .saturating_add(self.edges.len())
            .saturating_add(self.words.len())
            .saturating_add(solver.result.masks.len())
            .saturating_add(solver.result.full.len())
            .saturating_add(extra);
        if entries > 500_000 {
            return Err("runtime lifetime storage budget exceeded");
        }
        Ok(())
    }
    fn insert(
        &mut self,
        solver: &mut Solver,
        region: Region,
        word: u32,
        bits: u64,
    ) -> Result<(), &'static str> {
        self.space(solver, 2)?;
        if solver.insert(region, word, bits)? {
            let head = &mut self.word_heads[region.0 as usize];
            self.words.push(Word {
                offset: word,
                next: *head,
            });
            *head = Some(self.words.len() - 1);
        }
        Ok(())
    }
    fn index_edges(&mut self, solver: &mut Solver, proof: &Solution) -> Result<(), &'static str> {
        let mut seen = HashSet::new();
        for (region, _) in proof.nodes() {
            solver.charge(1)?;
            if region == Region::LOCAL || solver.result.full.contains(&region) {
                continue;
            }
            let source = self.classes[region.0 as usize];
            for edge in proof
                .requirements(region)
                .map_err(|_| "invalid proven lifetime graph")?
            {
                solver.charge(1)?;
                if edge.longer == Region::LOCAL || solver.result.full.contains(&edge.longer) {
                    continue;
                }
                let target = self.classes[edge.longer.0 as usize];
                if source == target || seen.contains(&(source, target)) {
                    continue;
                }
                self.space(solver, seen.len().saturating_add(2))?;
                if self.ranks[source.0 as usize] <= self.ranks[target.0 as usize] {
                    return Err("invalid runtime lifetime component order");
                }
                seen.insert((source, target));
                let head = &mut self.heads[source.0 as usize];
                self.edges.push(Edge {
                    target,
                    next: *head,
                });
                *head = Some(self.edges.len() - 1);
            }
        }
        Ok(())
    }
    fn propagate(&mut self, solver: &mut Solver) -> Result<(), &'static str> {
        for index in (0..self.order.len()).rev() {
            solver.charge(1)?;
            let source = self.order[index];
            let mut edge = self.heads[source.0 as usize];
            while let Some(index) = edge {
                solver.charge(1)?;
                let target = self.edges[index].target;
                edge = self.edges[index].next;
                let mut word = self.word_heads[source.0 as usize];
                while let Some(index) = word {
                    let offset = self.words[index].offset;
                    word = self.words[index].next;
                    let bits = solver.result.masks[&(source, offset)];
                    self.insert(solver, target, offset, bits)?;
                }
            }
        }
        Ok(())
    }
    fn expand(&self, solver: &mut Solver) -> Result<(), &'static str> {
        // Only the completed public result expands equal regions. The solver
        // never revisits a component's outgoing edges for a newly arrived bit.
        for (index, &source) in self.classes.iter().enumerate() {
            solver.charge(1)?;
            let target = Region(u32::try_from(index).map_err(|_| "too many runtime lifetimes")?);
            if source == target || target == Region::LOCAL {
                continue;
            }
            let mut word = self.word_heads[source.0 as usize];
            while let Some(index) = word {
                self.space(solver, 1)?;
                let offset = self.words[index].offset;
                word = self.words[index].next;
                let bits = solver.result.masks[&(source, offset)];
                solver.insert(target, offset, bits)?;
            }
        }
        Ok(())
    }
}
impl Solver {
    pub(super) fn propagate(&mut self, proof: &Solution) -> Result<(), &'static str> {
        if self.result.masks.is_empty() {
            return Ok(());
        }
        // Account for the component search arrays before allocating them. Full
        // extents keep their existing compact representation and need no words.
        let storage = proof
            .regions
            .saturating_mul(10)
            .saturating_add(self.result.masks.len())
            .saturating_add(self.result.full.len());
        if storage > 500_000 {
            return Err("runtime lifetime storage budget exceeded");
        }
        self.charge(proof.regions)?;
        let (classes, ranks, steps) = proof
            .runtime_classes(self.limit.saturating_sub(self.result.steps))
            .map_err(|error| match error {
                crate::types::region_graph::Error::Budget => {
                    "runtime lifetime work budget exceeded"
                }
                _ => "invalid proven lifetime components",
            })?;
        self.charge(steps)?;
        let mut order = vec![Region::STATIC; ranks.iter().max().map_or(0, |rank| rank + 1)];
        for (index, &rank) in ranks.iter().enumerate() {
            order[rank] = classes[index];
        }
        let mut plan = Plan {
            classes,
            ranks,
            order,
            edges: Vec::new(),
            heads: vec![None; proof.regions],
            words: Vec::new(),
            word_heads: vec![None; proof.regions],
        };
        plan.index_edges(self, proof)?;
        let seeds: HashMap<_, _> = std::mem::take(&mut self.result.masks);
        self.result.word_count -= seeds.len();
        for ((region, word), bits) in seeds {
            plan.insert(self, plan.classes[region.0 as usize], word, bits)?;
        }
        plan.propagate(self)?;
        plan.expand(self)
    }
}
