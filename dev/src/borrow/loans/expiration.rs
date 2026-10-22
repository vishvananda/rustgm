//! Index region exits once; consecutive CFG points visit only expiring loans.
use super::{Loan, Point, Set, points};
use crate::borrow::regions::graph::Region;
use crate::support::hash::HashMap;

#[cfg(test)]
#[path = "../../../tests/invariants/loan_expirations.rs"]
mod tests;

/// Dense CFGs use direct rows; sparse point domains retain only actual exits.
#[derive(Default)]
enum Rows {
    #[default]
    Empty,
    Dense(Vec<u32>),
    Sparse(Vec<(Point, u32)>),
}

const MAX_ENTRIES: usize = 500_000;

/// Expiring loan words grouped by the point that first excludes them.
#[derive(Default)]
pub(super) struct Expirations {
    rows: Rows,
    events: Vec<(usize, u64)>,
    entries: usize,
    pub steps: usize,
}
impl Expirations {
    pub fn new(loans: &[Loan], regions: &points::Liveness) -> Result<Self, &'static str> {
        let mut result = Self::default();
        if loans.is_empty() {
            return Ok(result);
        }
        let mut words: HashMap<Region, Vec<(usize, u64)>> = HashMap::default();
        for (loan, data) in loans.iter().enumerate() {
            result.charge(1)?;
            let entries = words.entry(data.region).or_default();
            let word = loan / 64;
            let bit = 1 << (loan % 64);
            if let Some((_, bits)) = entries.last_mut().filter(|(last, _)| *last == word) {
                *bits |= bit;
            } else {
                entries.push((word, bit));
            }
        }
        // Unsorted (point, region) rows, bounded by the retained event masks.
        let mut rows = Vec::new();
        for (region, word, mut ends) in regions.endings() {
            result.charge(1)?;
            let Some(loans) = words.get(&region) else {
                continue;
            };
            while ends != 0 {
                result.charge(loans.len().saturating_add(1))?;
                let point = word * 64 + ends.trailing_zeros() + 1;
                ends &= ends - 1;
                result.entries = result
                    .entries
                    .checked_add(loans.len())
                    .filter(|&count| count <= MAX_ENTRIES)
                    .ok_or("loan expiration storage budget exceeded")?;
                rows.push((point, region));
            }
        }
        if rows.is_empty() {
            return Ok(result);
        }
        let slots = usize::try_from(regions.point_count())
            .ok()
            .and_then(|points| points.checked_add(2))
            .filter(|&slots| slots <= MAX_ENTRIES && slots <= rows.len() * 8 + 8);
        if let Some(slots) = slots {
            result.dense(rows, &words, slots)?;
        } else {
            result.sparse(rows, &words)?;
        }
        Ok(result)
    }
    fn dense(
        &mut self,
        rows: Vec<(u32, Region)>,
        words: &HashMap<Region, Vec<(usize, u64)>>,
        slots: usize,
    ) -> Result<(), &'static str> {
        // Bound both simultaneous index buffers and charge their initialization,
        // prefix walk and clone, plus event initialization and row counting.
        self.charge(slots * 3 + self.entries + rows.len())?;
        let mut starts = vec![0_u32; slots];
        for &(point, region) in &rows {
            let slot = starts
                .get_mut(point as usize + 1)
                .ok_or("invalid loan expiration point")?;
            *slot += u32::try_from(words.get(&region).map_or(0, Vec::len))
                .map_err(|_| "loan expiration storage budget exceeded")?;
        }
        for index in 1..starts.len() {
            starts[index] += starts[index - 1];
        }
        let mut next = starts.clone();
        self.events = vec![(0, 0); self.entries];
        for (point, region) in rows {
            let loans = words.get(&region).map_or(&[][..], Vec::as_slice);
            let at = next[point as usize] as usize;
            self.events[at..at + loans.len()].copy_from_slice(loans);
            next[point as usize] += u32::try_from(loans.len())
                .map_err(|_| "loan expiration storage budget exceeded")?;
        }
        self.rows = Rows::Dense(starts);
        Ok(())
    }
    fn sparse(
        &mut self,
        mut rows: Vec<(u32, Region)>,
        words: &HashMap<Region, Vec<(usize, u64)>>,
    ) -> Result<(), &'static str> {
        self.charge(rows.len())?;
        let mut comparisons = 0;
        rows.sort_unstable_by(|a, b| {
            comparisons += 1;
            a.0.cmp(&b.0)
        });
        self.charge(comparisons)?;
        let mut starts = Vec::with_capacity(rows.len());
        self.events = Vec::with_capacity(self.entries);
        for (point, region) in rows {
            if starts.last().is_none_or(|&(last, _)| last != Point(point)) {
                starts.push((
                    Point(point),
                    u32::try_from(self.events.len())
                        .map_err(|_| "loan expiration storage budget exceeded")?,
                ));
            }
            self.events
                .extend_from_slice(words.get(&region).map_or(&[][..], Vec::as_slice));
        }
        self.rows = Rows::Sparse(starts);
        Ok(())
    }
    /// CFG points that expire at least one loan word.
    #[cfg(test)]
    fn event_points(&self) -> usize {
        match &self.rows {
            Rows::Empty => 0,
            Rows::Dense(starts) => starts.windows(2).filter(|row| row[0] != row[1]).count(),
            Rows::Sparse(starts) => starts.len(),
        }
    }
    fn at(&self, point: Point) -> (&[(usize, u64)], usize) {
        match &self.rows {
            Rows::Empty => (&[], 1),
            Rows::Dense(starts) => {
                let index = point.0 as usize;
                match (starts.get(index), starts.get(index + 1)) {
                    (Some(&start), Some(&end)) => (&self.events[start as usize..end as usize], 1),
                    _ => (&[], 1),
                }
            }
            Rows::Sparse(starts) => {
                let mut steps = 0;
                let found = starts.binary_search_by(|&(candidate, _)| {
                    steps += 1;
                    candidate.0.cmp(&point.0)
                });
                if let Ok(index) = found {
                    let start = starts[index].1 as usize;
                    let end = starts
                        .get(index + 1)
                        .map_or(self.events.len(), |&(_, end)| end as usize);
                    (&self.events[start..end], steps)
                } else {
                    (&[], steps.max(1))
                }
            }
        }
    }
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|&steps| steps <= 32_000_000)
            .ok_or("loan expiration work budget exceeded")?;
        Ok(())
    }
    /// Block entries, joins, jumps and unvalidated births use the exact full
    /// membership check. Only a known consecutive state can consume exit events.
    pub fn prune(
        &self,
        state: &mut Set,
        point: Point,
        loans: &[Loan],
        regions: &points::Liveness,
    ) -> Result<usize, &'static str> {
        let mut steps = 1;
        if state
            .valid_at
            .is_some_and(|previous| previous.0.checked_add(1) == Some(point.0))
        {
            let (entries, lookup) = self.at(point);
            steps = lookup + entries.len();
            for &(word, mask) in entries {
                state.retain(word, !mask);
            }
        } else {
            let mut from = 0;
            while let Some(word) = state.next(from) {
                steps += state.depth();
                let mut bits = state.words()[word].live;
                let mut retained = bits;
                while bits != 0 {
                    steps += 1;
                    let bit = bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    if !regions.contains(loans[word * 64 + bit].region, point)? {
                        retained &= !(1 << bit);
                    }
                }
                state.retain(word, retained);
                from = word + 1;
            }
        }
        state.valid_at = Some(point);
        Ok(steps)
    }
}
