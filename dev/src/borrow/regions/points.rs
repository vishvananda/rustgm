//! Sparse CFG-point propagation over an already proved outlives graph.
mod propagate;
mod verify;
use super::graph::{Region, Solution};
use crate::support::hash::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Point(pub u32);

/// Use-site seeds determine runtime liveness. The contract-only LOCAL sentinel
/// is never a runtime seed or an intermediary connecting unrelated lifetimes.
pub struct Liveness {
    points: u32,
    regions: usize,
    masks: HashMap<(Region, u32), u64>,
    full: HashSet<Region>,
    word_count: usize,
    pub steps: usize,
}
impl Liveness {
    /// Sparse word-local boundaries whose following point is no longer live.
    /// Full extents have no boundaries. Adjacent words are checked once, so
    /// consumers need not enumerate every live point of every region.
    pub(crate) fn endings(&self) -> impl Iterator<Item = (Region, u32, u64)> + '_ {
        self.masks.iter().map(|(&(region, word), &bits)| {
            let mut ends = bits & !(bits >> 1);
            if ends >> 63 != 0
                && self
                    .masks
                    .get(&(region, word + 1))
                    .is_some_and(|next| next & 1 != 0)
            {
                ends &= !(1 << 63);
            }
            let last = self.points.saturating_sub(1);
            if word == last / 64 {
                ends &= !(1 << (last % 64));
            }
            (region, word, ends)
        })
    }
    /// Nonzero words in region/word order. Sort only the retained representation;
    /// full extents expand lazily without allocating a region-by-word matrix.
    pub fn words(&self) -> impl Iterator<Item = (Region, u32, u64)> + '_ {
        let words = self.points.div_ceil(64);
        let tail = self.points % 64;
        let last = if tail == 0 { u64::MAX } else { (1 << tail) - 1 };
        let mut entries: Vec<_> = self
            .masks
            .iter()
            .map(|(&(region, word), &bits)| (region, word, Some(bits)))
            .chain(self.full.iter().map(|&region| (region, 0, None)))
            .collect();
        entries.sort_unstable_by_key(|&(region, word, _)| (region.0, word));
        entries.into_iter().flat_map(move |(region, word, bits)| {
            let range = if bits.is_some() {
                word..word + 1
            } else {
                0..words
            };
            range.map(move |word| {
                (
                    region,
                    word,
                    bits.unwrap_or(if word + 1 == words { last } else { u64::MAX }),
                )
            })
        })
    }
    /// A missing sparse word contains no live points, not an unknown answer.
    ///
    /// # Errors
    /// Rejects foreign region/point identities and the contract-only LOCAL bound.
    pub fn contains(&self, region: Region, point: Point) -> Result<bool, &'static str> {
        self.validate(region, point)?;
        Ok(self.full.contains(&region)
            || self
                .masks
                .get(&(region, point.0 / 64))
                .is_some_and(|bits| bits & (1 << (point.0 % 64)) != 0))
    }
    #[must_use]
    pub fn mask_count(&self) -> usize {
        self.word_count
    }
    fn validate_region(&self, region: Region) -> Result<(), &'static str> {
        if region == Region::LOCAL || region.0 as usize >= self.regions {
            return Err("invalid runtime lifetime identity");
        }
        Ok(())
    }
    fn validate(&self, region: Region, point: Point) -> Result<(), &'static str> {
        self.validate_region(region)?;
        if point.0 >= self.points {
            return Err("invalid runtime lifetime identity");
        }
        Ok(())
    }
}
struct Solver {
    result: Liveness,
    limit: usize,
}
impl Solver {
    fn full_extents(
        &mut self,
        proof: &Solution,
        extents: impl IntoIterator<Item = Region>,
    ) -> Result<(), &'static str> {
        let mut work = Vec::new();
        for region in extents {
            self.cover(region, &mut work)?;
        }
        while let Some(region) = work.pop() {
            self.charge(1)?;
            for relation in proof
                .requirements(region)
                .map_err(|_| "invalid proven lifetime graph")?
            {
                self.charge(1)?;
                if relation.longer != Region::LOCAL {
                    self.cover(relation.longer, &mut work)?;
                }
            }
        }
        Ok(())
    }
    fn cover(&mut self, region: Region, work: &mut Vec<Region>) -> Result<(), &'static str> {
        self.charge(1)?;
        self.result.validate_region(region)?;
        if self.result.points == 0 || self.result.full.contains(&region) {
            return Ok(());
        }
        if self.result.full.len().saturating_add(work.len()) >= 500_000 {
            return Err("runtime lifetime storage budget exceeded");
        }
        self.result.word_count = self
            .result
            .word_count
            .checked_add(self.result.points.div_ceil(64) as usize)
            .ok_or("runtime lifetime word count overflow")?;
        self.result.full.insert(region);
        work.push(region);
        Ok(())
    }
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.result.steps = self
            .result
            .steps
            .checked_add(count)
            .filter(|n| *n <= self.limit)
            .ok_or("runtime lifetime work budget exceeded")?;
        Ok(())
    }
    fn insert(&mut self, region: Region, word: u32, bits: u64) -> Result<bool, &'static str> {
        self.charge(1)?;
        if self.result.full.contains(&region) {
            return Ok(false);
        }
        let old = self.result.masks.get(&(region, word)).copied().unwrap_or(0);
        let added = bits & !old;
        if added == 0 {
            return Ok(false);
        }
        // Budget retained sparse words before growth. Component propagation
        // additionally budgets its temporary graph and word index.
        // Hash-table overhead is bounded separately from propagation steps.
        if self
            .result
            .masks
            .len()
            .saturating_add(self.result.full.len())
            >= 500_000
        {
            return Err("runtime lifetime storage budget exceeded");
        }
        if old == 0 {
            self.result.word_count = self
                .result
                .word_count
                .checked_add(1)
                .ok_or("runtime lifetime word count overflow")?;
        }
        self.result.masks.insert((region, word), old | added);
        Ok(old == 0)
    }
}
/// Propagate explicit use-site seeds backwards through retained outlives edges.
/// Full extents are an exact shorthand for seeding every point of the region.
/// Complete their closure before sparse seeds so the two representations stay
/// disjoint. No region kind or declared assumption supplies an implicit extent.
/// Callers supply CFG liveness and external-region extents; this operation does
/// not itself prove loan conflicts, storage lifetime or execution eligibility.
///
/// # Errors
/// Rejects malformed identities and exhausted storage/work budgets.
pub fn infer(
    proof: &Solution,
    points: u32,
    seeds: impl IntoIterator<Item = (Region, Point)>,
    full_extents: impl IntoIterator<Item = Region>,
    limit: usize,
) -> Result<Liveness, &'static str> {
    let mut solver = Solver {
        result: Liveness {
            points,
            regions: proof.regions,
            masks: HashMap::default(),
            full: HashSet::default(),
            word_count: 0,
            steps: 0,
        },
        limit,
    };
    solver.full_extents(proof, full_extents)?;
    for (region, point) in seeds {
        solver.result.validate(region, point)?;
        solver.insert(region, point.0 / 64, 1 << (point.0 % 64))?;
    }
    solver.propagate(proof)?;
    solver.verify(proof)?;
    Ok(solver.result)
}
