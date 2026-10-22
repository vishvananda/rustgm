//! Index region exits once; consecutive CFG points visit only expiring loans.
use super::{Loan, Point, Set, points};
use crate::borrow::regions::graph::Region;
use crate::support::hash::HashMap;

#[cfg(test)]
#[path = "../../../tests/invariants/loan_expirations.rs"]
mod tests;

/// Expiring loan words grouped by the point that first excludes them. A
/// compressed row index per CFG point replaces a hash probe at every point.
#[derive(Default)]
pub(super) struct Expirations {
    starts: Vec<u32>,
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
        // Unsorted (point, region) rows; a counting sort keeps their order.
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
                    .filter(|&count| count <= 500_000)
                    .ok_or("loan expiration storage budget exceeded")?;
                rows.push((point, region));
            }
        }
        let points = regions.point_count() as usize + 1;
        let mut starts = vec![0_u32; points + 1];
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
        result.events = vec![(0, 0); result.entries];
        for (point, region) in rows {
            let loans = words.get(&region).map_or(&[][..], Vec::as_slice);
            let at = next[point as usize] as usize;
            result.events[at..at + loans.len()].copy_from_slice(loans);
            next[point as usize] += u32::try_from(loans.len())
                .map_err(|_| "loan expiration storage budget exceeded")?;
        }
        result.starts = starts;
        Ok(result)
    }
    /// CFG points that expire at least one loan word.
    #[cfg(test)]
    fn event_points(&self) -> usize {
        self.starts
            .windows(2)
            .filter(|row| row[0] != row[1])
            .count()
    }
    fn at(&self, point: Point) -> &[(usize, u64)] {
        let index = point.0 as usize;
        match (self.starts.get(index), self.starts.get(index + 1)) {
            (Some(&start), Some(&end)) => &self.events[start as usize..end as usize],
            _ => &[],
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
            let entries = self.at(point);
            steps += entries.len();
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
