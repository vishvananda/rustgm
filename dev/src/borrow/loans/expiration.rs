//! Index region exits once; consecutive CFG points visit only expiring loans.
use super::{Loan, Point, Set, points};
use crate::borrow::regions::graph::Region;
use crate::support::hash::HashMap;

#[cfg(test)]
#[path = "../../../tests/invariants/loan_expirations.rs"]
mod tests;

#[derive(Default)]
pub(super) struct Expirations {
    events: HashMap<Point, Vec<(usize, u64)>>,
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
        for (region, word, mut ends) in regions.endings() {
            result.charge(1)?;
            let Some(loans) = words.get(&region) else {
                continue;
            };
            while ends != 0 {
                result.charge(loans.len().saturating_add(1))?;
                let point = Point(word * 64 + ends.trailing_zeros() + 1);
                ends &= ends - 1;
                result.entries = result
                    .entries
                    .checked_add(loans.len())
                    .filter(|&count| count <= 500_000)
                    .ok_or("loan expiration storage budget exceeded")?;
                result
                    .events
                    .entry(point)
                    .or_default()
                    .extend_from_slice(loans);
            }
        }
        Ok(result)
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
            if let Some(entries) = self.events.get(&point) {
                steps += entries.len();
                for &(word, mask) in entries {
                    state.retain(word, !mask);
                }
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
