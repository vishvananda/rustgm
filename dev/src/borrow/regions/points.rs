//! Sparse CFG-point propagation over an already proved outlives graph.
use super::graph::{Region, Solution};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Point(pub u32);

/// Use-site seeds determine runtime liveness. The contract-only LOCAL sentinel
/// is never a runtime seed or an intermediary connecting unrelated lifetimes.
pub struct Liveness {
    points: u32,
    regions: usize,
    masks: HashMap<(Region, u32), u64>,
    pub steps: usize,
}
impl Liveness {
    /// Unordered nonzero words; an inspection view sorts by region then word.
    pub fn words(&self) -> impl Iterator<Item = (Region, u32, u64)> + '_ {
        self.masks
            .iter()
            .map(|(&(region, word), &bits)| (region, word, bits))
    }
    /// A missing sparse word contains no live points, not an unknown answer.
    ///
    /// # Errors
    /// Rejects foreign region/point identities and the contract-only LOCAL bound.
    pub fn contains(&self, region: Region, point: Point) -> Result<bool, &'static str> {
        self.validate(region, point)?;
        Ok(self
            .masks
            .get(&(region, point.0 / 64))
            .is_some_and(|bits| bits & (1 << (point.0 % 64)) != 0))
    }
    #[must_use]
    pub fn mask_count(&self) -> usize {
        self.masks.len()
    }
    fn validate(&self, region: Region, point: Point) -> Result<(), &'static str> {
        if region == Region::LOCAL || region.0 as usize >= self.regions || point.0 >= self.points {
            return Err("invalid runtime lifetime identity");
        }
        Ok(())
    }
}
struct Change {
    region: Region,
    word: u32,
    bits: u64,
}
struct Solver {
    result: Liveness,
    work: Vec<Change>,
    limit: usize,
}
impl Solver {
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.result.steps = self
            .result
            .steps
            .checked_add(count)
            .filter(|n| *n <= self.limit)
            .ok_or("runtime lifetime work budget exceeded")?;
        Ok(())
    }
    fn insert<const SEED: bool>(
        &mut self,
        region: Region,
        word: u32,
        bits: u64,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        let old = self.result.masks.get(&(region, word)).copied().unwrap_or(0);
        let added = bits & !old;
        if added == 0 {
            return Ok(());
        }
        // Budget both retained sparse words and pending deltas before growth.
        // Hash-table overhead is bounded separately from propagation steps.
        if self.result.masks.len().saturating_add(self.work.len()) >= 500_000 {
            return Err("runtime lifetime storage budget exceeded");
        }
        self.result.masks.insert((region, word), old | added);
        // Initial seeds are all collected before propagation. Queue each word
        // once, in first-seen order; subsequent propagation still carries deltas.
        if !SEED || old == 0 {
            self.work.push(Change {
                region,
                word,
                bits: added,
            });
        }
        Ok(())
    }
}
/// Propagate explicit use-site seeds backwards through retained outlives edges.
/// Callers supply CFG liveness and external-region extents; this operation does
/// not itself prove loan conflicts, storage lifetime or execution eligibility.
///
/// # Errors
/// Rejects malformed identities and exhausted storage/work budgets.
pub fn infer(
    proof: &Solution,
    points: u32,
    seeds: impl IntoIterator<Item = (Region, Point)>,
    limit: usize,
) -> Result<Liveness, &'static str> {
    let mut solver = Solver {
        result: Liveness {
            points,
            regions: proof.regions,
            masks: HashMap::new(),
            steps: 0,
        },
        work: Vec::new(),
        limit,
    };
    for (region, point) in seeds {
        solver.result.validate(region, point)?;
        solver.insert::<true>(region, point.0 / 64, 1 << (point.0 % 64))?;
    }
    solver.charge(solver.work.len())?;
    for change in &mut solver.work {
        change.bits = *solver
            .result
            .masks
            .get(&(change.region, change.word))
            .ok_or("missing runtime lifetime seed word")?;
    }
    while let Some(change) = solver.work.pop() {
        solver.charge(1)?;
        for relation in proof
            .requirements(change.region)
            .map_err(|_| "invalid proven lifetime graph")?
        {
            solver.charge(1)?;
            if relation.longer != Region::LOCAL {
                solver.insert::<false>(relation.longer, change.word, change.bits)?;
            }
        }
    }
    Ok(solver.result)
}
