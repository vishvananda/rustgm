//! Independent tests share assumption reachability and never introduce proof debt.
use super::{Error, Graph, Region, Relations, VerificationId};
use crate::types::region_graph::Kind;
use std::collections::{HashMap, HashSet};

pub(crate) struct Answers {
    pub values: Vec<bool>,
    pub steps: usize,
}

struct Assumptions<'a> {
    graph: &'a Graph,
    reachable: HashMap<Region, HashSet<Region>>,
    work: Vec<Region>,
    steps: usize,
}
impl Relations for Assumptions<'_> {
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > self.graph.limit {
            return Err(Error::Budget);
        }
        Ok(())
    }

    fn outlives(&mut self, longer: Region, shorter: Region) -> Result<bool, Error> {
        self.charge(1)?;
        if longer == shorter || longer == Region::STATIC {
            return Ok(true);
        }
        let shorter = if self.graph.kind(longer)?.universe() < self.graph.kind(shorter)?.universe()
        {
            Region::STATIC
        } else {
            shorter
        };
        if !self.reachable.contains_key(&longer) {
            let mut reached = HashSet::from([longer]);
            self.work.push(longer);
            while let Some(region) = self.work.pop() {
                self.charge(1)?;
                let mut edge = self.graph.assumed_heads[region.0 as usize];
                while let Some(index) = edge {
                    self.charge(1)?;
                    let relation = self.graph.assumptions[index];
                    edge = relation.next;
                    if reached.insert(relation.target) {
                        self.work.push(relation.target);
                    }
                }
            }
            self.reachable.insert(longer, reached);
        }
        let reached = &self.reachable[&longer];
        Ok(reached.contains(&shorter) || reached.contains(&Region::STATIC))
    }
}

impl Graph {
    /// Evaluate independent queries under a rigid declaration environment.
    /// Unknown relationships return false; malformed or unfinished proofs fail.
    pub(crate) fn test_verifications(&self, roots: &[VerificationId]) -> Result<Answers, Error> {
        if self.members.is_some()
            || !self.obligations.is_empty()
            || self
                .kinds
                .iter()
                .any(|kind| matches!(kind, Kind::Variable { .. }))
            || self
                .verifications
                .as_ref()
                .is_some_and(|v| !v.requests.is_empty())
        {
            return Err(Error::Identity);
        }
        if roots.is_empty() {
            return Ok(Answers {
                values: Vec::new(),
                steps: 0,
            });
        }
        let tests = self.verifications.as_ref().ok_or(Error::Identity)?;
        let mut context = Assumptions {
            graph: self,
            reachable: HashMap::new(),
            work: Vec::new(),
            steps: 0,
        };
        context.charge(
            self.kinds
                .len()
                .saturating_add(tests.nodes.len())
                .saturating_add(roots.len()),
        )?;
        let mut values = vec![None; tests.nodes.len()];
        let mut work = Vec::new();
        let mut answers = Vec::with_capacity(roots.len());
        for &root in roots {
            tests.evaluate_root(root, None, &mut values, &mut work, &mut context)?;
            answers.push(values[root.0 as usize].ok_or(Error::Identity)?);
        }
        Ok(Answers {
            values: answers,
            steps: context.steps,
        })
    }
}
