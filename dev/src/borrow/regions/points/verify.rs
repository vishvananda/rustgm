//! Retained type tests must use one alternative across symbolic and CFG extents.
use super::{HashMap, Region, Solution, Solver};
use crate::types::region_graph::Error;

struct Extents<'a> {
    solver: &'a mut Solver,
    words: Option<HashMap<Region, Vec<(u32, u64)>>>,
}
impl Extents<'_> {
    fn index(&mut self) -> Result<(), Error> {
        if self.words.is_some() {
            return Ok(());
        }
        self.solver
            .charge(self.solver.result.masks.len())
            .map_err(|_| Error::Budget)?;
        let mut words: HashMap<Region, Vec<(u32, u64)>> = HashMap::default();
        for (&(region, word), &bits) in &self.solver.result.masks {
            words.entry(region).or_default().push((word, bits));
        }
        for entries in words.values_mut() {
            let work = entries
                .len()
                .saturating_mul(entries.len().ilog2() as usize + 1);
            self.solver.charge(work).map_err(|_| Error::Budget)?;
            entries.sort_unstable_by_key(|entry| entry.0);
        }
        self.words = Some(words);
        Ok(())
    }
    fn contains(&mut self, longer: Region, shorter: Region) -> Result<bool, Error> {
        self.solver
            .result
            .validate_region(longer)
            .map_err(|_| Error::Identity)?;
        self.solver
            .result
            .validate_region(shorter)
            .map_err(|_| Error::Identity)?;
        self.solver.charge(1).map_err(|_| Error::Budget)?;
        if longer == shorter || self.solver.result.full.contains(&longer) {
            return Ok(true);
        }
        self.index()?;
        let words = self.words.as_ref().ok_or(Error::Identity)?;
        if self.solver.result.full.contains(&shorter) {
            let count = self.solver.result.points.div_ceil(64);
            let entries = words.get(&longer).map_or(&[][..], Vec::as_slice);
            if entries.len() != count as usize {
                return Ok(false);
            }
            self.solver
                .charge(entries.len())
                .map_err(|_| Error::Budget)?;
            let tail = self.solver.result.points % 64;
            return Ok(entries.iter().all(|&(word, bits)| {
                let expected = if word + 1 == count && tail != 0 {
                    (1 << tail) - 1
                } else {
                    u64::MAX
                };
                bits == expected
            }));
        }
        let entries = words.get(&shorter).map_or(&[][..], Vec::as_slice);
        self.solver
            .charge(entries.len())
            .map_err(|_| Error::Budget)?;
        Ok(entries.iter().all(|&(word, bits)| {
            self.solver
                .result
                .masks
                .get(&(longer, word))
                .copied()
                .unwrap_or(0)
                & bits
                == bits
        }))
    }
}
impl Solver {
    pub(super) fn verify(&mut self, proof: &Solution) -> Result<(), &'static str> {
        let limit = self.limit.saturating_sub(self.result.steps);
        let steps = {
            let mut context = Extents {
                solver: self,
                words: None,
            };
            proof.verify_regions(|a, b| context.contains(a, b), limit)
        }
        .map_err(|error| match error {
            Error::Budget => "runtime lifetime verification work budget exceeded",
            Error::Identity => "invalid runtime lifetime verification identity",
            Error::Verification { .. } => "required type does not outlive its live uses",
            Error::UnsupportedOutput | Error::Unsatisfied { .. } | Error::Membership { .. } => {
                "invalid symbolic lifetime verification"
            }
        })?;
        self.charge(steps)
    }
}
