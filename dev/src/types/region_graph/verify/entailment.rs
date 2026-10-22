//! Verify inferred lower bounds without promoting obligations into assumptions.
use super::{super::Solver, Relations};
use crate::support::hash::HashMap;
use crate::types::region_graph::{Error, Kind, Region};

struct Symbolic<'s, 'g> {
    solver: &'s mut Solver<'g>,
    lower: HashMap<Region, Vec<Region>>,
}
impl Relations for Symbolic<'_, '_> {
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.solver.charge_many(count)
    }
    fn outlives(&mut self, longer: Region, shorter: Region) -> Result<bool, Error> {
        if longer == shorter || longer == Region::STATIC {
            return Ok(true);
        }
        if self.solver.graph.kind(shorter)?.rigid() {
            return self.includes(longer, shorter);
        }
        let count = self.lower.get(&shorter).map_or(0, Vec::len);
        for index in 0..count {
            self.charge(1)?;
            let bound = self.lower[&shorter][index];
            if !self.includes(longer, bound)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
impl Symbolic<'_, '_> {
    fn includes(&mut self, longer: Region, bound: Region) -> Result<bool, Error> {
        let kind = self.solver.graph.kind(longer)?;
        let bound = if kind.universe() < self.solver.graph.kind(bound)?.universe() {
            Region::STATIC
        } else {
            bound
        };
        if kind.rigid() {
            return self.solver.entails(longer, bound);
        }
        let count = self.lower.get(&longer).map_or(0, Vec::len);
        for index in 0..count {
            self.charge(1)?;
            let candidate = self.lower[&longer][index];
            if self.solver.entails(candidate, bound)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
impl Solver<'_> {
    pub(in super::super) fn verify(&mut self) -> Result<Vec<Option<bool>>, Error> {
        let Some(tests) = &self.graph.verifications else {
            return Ok(Vec::new());
        };
        if tests.requests.is_empty() {
            return Ok(Vec::new());
        }
        self.charge_many(self.reached.len())?;
        let mut lower: HashMap<Region, Vec<Region>> = HashMap::default();
        for &(region, bound) in &self.reached {
            if matches!(self.graph.kind(region)?, Kind::Variable { .. }) {
                lower.entry(region).or_default().push(bound);
            }
        }
        // Stable order makes bounded proof work independent of hash iteration.
        for bounds in lower.values_mut() {
            self.charge_many(
                bounds
                    .len()
                    .saturating_mul(bounds.len().ilog2() as usize + 1),
            )?;
            bounds.sort_unstable_by_key(|region| region.0);
        }
        tests.check(
            None,
            &mut Symbolic {
                solver: self,
                lower,
            },
        )
    }
}
