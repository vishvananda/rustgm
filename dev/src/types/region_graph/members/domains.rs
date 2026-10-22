//! Canonical candidate sets share intersection, filtering and minimum queries.
use super::{ChoicesId, Error, Members, Region};
use crate::support::hash::{HashMap, HashSet};
use crate::{support::intern_slice::Slices, types::region_graph::Solver};

#[derive(Default)]
pub(super) struct Domains {
    sets: Slices<Region>,
    order: Vec<usize>,
    predecessors: Vec<Vec<Region>>,
    ancestors: HashMap<Region, HashSet<Region>>,
    choices: HashMap<ChoicesId, u32>,
    names: HashMap<(ChoicesId, Region), Region>,
    intersections: HashMap<(u32, u32), u32>,
    indexed: HashMap<u32, HashSet<Region>>,
    filtered: HashMap<(u32, Region), u32>,
    minima: HashMap<u32, Option<Region>>,
    reachable: HashMap<Region, Option<u32>>,
}
impl Domains {
    pub(super) fn new(
        classes: &[Region],
        order: Vec<usize>,
        solver: &mut Solver<'_>,
    ) -> Result<Self, Error> {
        let mut predecessors = vec![Vec::new(); classes.len()];
        for (longer, &head) in solver.graph.assumed_heads.iter().enumerate() {
            let longer = classes[longer];
            let mut edge = head;
            while let Some(index) = edge {
                solver.charge()?;
                let relation = solver.graph.assumptions[index];
                edge = relation.next;
                let shorter = classes[relation.target.0 as usize];
                if shorter != longer {
                    predecessors[shorter.0 as usize].push(longer);
                }
            }
        }
        Ok(Self {
            order,
            predecessors,
            ..Self::default()
        })
    }

    pub(super) fn choices(
        &mut self,
        id: ChoicesId,
        members: &Members,
        classes: &[Region],
        solver: &mut Solver<'_>,
    ) -> Result<u32, Error> {
        if let Some(&domain) = self.choices.get(&id) {
            return Ok(domain);
        }
        let choices = members.choices.get(id.0).ok_or(Error::Identity)?;
        solver.charge_many(choices.len())?;
        let mut canonical = Vec::with_capacity(choices.len());
        for &choice in choices {
            let class = classes[choice.0 as usize];
            canonical.push(class);
            self.names.entry((id, class)).or_insert(choice);
        }
        canonical.sort_unstable_by_key(|r| r.0);
        canonical.dedup();
        let domain = self.sets.intern(&canonical).map_err(|_| Error::Budget)?;
        self.choices.insert(id, domain);
        Ok(domain)
    }

    pub(super) fn name(&self, choices: ChoicesId, selected: Region) -> Result<Region, Error> {
        self.names
            .get(&(choices, selected))
            .copied()
            .ok_or(Error::Identity)
    }

    pub(super) fn intersect(
        &mut self,
        a: u32,
        b: u32,
        solver: &mut Solver<'_>,
    ) -> Result<u32, Error> {
        if a == b {
            return Ok(a);
        }
        let key = (a.min(b), a.max(b));
        if let Some(&domain) = self.intersections.get(&key) {
            return Ok(domain);
        }
        let av = self.sets.get(a).ok_or(Error::Identity)?;
        let bv = self.sets.get(b).ok_or(Error::Identity)?;
        // Index each larger shared set once; singleton upper bounds cost O(1).
        let (small, large_id, large) = if av.len() <= bv.len() {
            (av, b, bv)
        } else {
            (bv, a, av)
        };
        if let std::collections::hash_map::Entry::Vacant(entry) = self.indexed.entry(large_id) {
            solver.charge_many(large.len())?;
            entry.insert(large.iter().copied().collect());
        }
        let index = &self.indexed[&large_id];
        let mut result = Vec::new();
        for &region in small {
            solver.charge()?;
            if index.contains(&region) {
                result.push(region);
            }
        }
        let domain = self.sets.intern(&result).map_err(|_| Error::Budget)?;
        self.intersections.insert(key, domain);
        Ok(domain)
    }

    pub(super) fn upper(
        &mut self,
        region: Region,
        classes: &[Region],
        solver: &mut Solver<'_>,
    ) -> Result<Option<u32>, Error> {
        if region == Region::STATIC {
            return Ok(None);
        }
        if let Some(&domain) = self.reachable.get(&region) {
            return Ok(domain);
        }
        let mut reached = HashSet::from_iter([region]);
        let mut work = vec![region];
        let mut all = false;
        while let Some(node) = work.pop() {
            solver.charge()?;
            if node == Region::STATIC {
                all = true;
                break;
            }
            let mut edge = solver.graph.assumed_heads[node.0 as usize];
            while let Some(index) = edge {
                solver.charge()?;
                let relation = solver.graph.assumptions[index];
                edge = relation.next;
                if reached.insert(relation.target) {
                    work.push(relation.target);
                }
            }
        }
        let domain = if all {
            None
        } else {
            let mut regions: Vec<_> = reached.into_iter().map(|r| classes[r.0 as usize]).collect();
            regions.sort_unstable_by_key(|r| r.0);
            regions.dedup();
            Some(self.sets.intern(&regions).map_err(|_| Error::Budget)?)
        };
        self.reachable.insert(region, domain);
        Ok(domain)
    }

    pub(super) fn lower(
        &mut self,
        domain: u32,
        bound: Region,
        solver: &mut Solver<'_>,
    ) -> Result<u32, Error> {
        if let Some(&result) = self.filtered.get(&(domain, bound)) {
            return Ok(result);
        }
        if bound == Region::LOCAL {
            // Every permitted capture is a root universal and includes the body.
            return Ok(domain);
        }
        let mut result = Vec::new();
        let count = self.sets.get(domain).ok_or(Error::Identity)?.len();
        for index in 0..count {
            let candidate = self.sets.get(domain).ok_or(Error::Identity)?[index];
            let holds = if solver.graph.kind(bound)?.universe() == 0 {
                self.outlives(candidate, bound, solver)?
            } else {
                solver.entails(candidate, bound)?
            };
            if holds {
                result.push(candidate);
            }
        }
        let result = self.sets.intern(&result).map_err(|_| Error::Budget)?;
        self.filtered.insert((domain, bound), result);
        Ok(result)
    }

    pub(super) fn minimum(
        &mut self,
        domain: u32,
        solver: &mut Solver<'_>,
    ) -> Result<Option<Region>, Error> {
        if let Some(&minimum) = self.minima.get(&domain) {
            return Ok(minimum);
        }
        let mut values = self.sets.get(domain).ok_or(Error::Identity)?.to_vec();
        solver.charge_many(values.len())?;
        // A DAG order puts the least qualifying candidate first. Try it once,
        // instead of testing every larger comparable candidate against the list.
        values.sort_unstable_by_key(|r| self.order[r.0 as usize]);
        let mut minimum = None;
        for &candidate in &values {
            let mut comparable = true;
            for &other in &values {
                let (longer, shorter) =
                    if self.order[other.0 as usize] < self.order[candidate.0 as usize] {
                        (candidate, other)
                    } else {
                        (other, candidate)
                    };
                if !self.outlives(longer, shorter, solver)? {
                    comparable = false;
                    break;
                }
            }
            if comparable {
                minimum = Some(candidate);
                break;
            }
        }
        self.minima.insert(domain, minimum);
        Ok(minimum)
    }

    fn outlives(
        &mut self,
        longer: Region,
        shorter: Region,
        solver: &mut Solver<'_>,
    ) -> Result<bool, Error> {
        solver.charge()?;
        if longer == shorter || longer == Region::STATIC {
            return Ok(true);
        }
        if !self.ancestors.contains_key(&shorter) {
            let mut seen = HashSet::from_iter([shorter]);
            let mut work = vec![shorter];
            while let Some(region) = work.pop() {
                solver.charge()?;
                for &parent in &self.predecessors[region.0 as usize] {
                    solver.charge()?;
                    if seen.insert(parent) {
                        work.push(parent);
                    }
                }
            }
            self.ancestors.insert(shorter, seen);
        }
        Ok(self.ancestors[&shorter].contains(&longer))
    }
}
