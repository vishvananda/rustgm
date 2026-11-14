//! A new temporary value replaces the pointer identity from a prior loop visit.
use super::{Depth, Flow, Index, Loan, Mode, Place, Set, ValueId};
use crate::support::hash::HashMap;

#[derive(Default)]
pub(super) struct Replacements {
    words: HashMap<ValueId, Vec<(usize, u64)>>,
    pub steps: usize,
}
impl Replacements {
    pub fn new(loans: &[Loan], places: &Index) -> Result<Self, &'static str> {
        if loans.len() > 500_000 {
            return Err("temporary loan replacement storage budget exceeded");
        }
        let mut result = Self::default();
        for (index, loan) in loans.iter().enumerate() {
            result.steps += 1;
            let Place::Temporary(value) = places.root(loan.place) else {
                continue;
            };
            let words = result.words.entry(value).or_default();
            let word = index / 64;
            let bit = 1 << (index % 64);
            if let Some((_, bits)) = words.last_mut().filter(|(last, _)| *last == word) {
                *bits |= bit;
            } else {
                words.push((word, bit));
            }
        }
        Ok(result)
    }
}
impl Flow<'_> {
    pub(super) fn replace(
        &mut self,
        state: &mut Set,
        value: ValueId,
        inspect: bool,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        if !self.replacements.words.contains_key(&value) {
            return Ok(());
        }
        if inspect {
            // Replacing a pointer does not access its old referent. Borrowing
            // the temporary's own storage still forbids overwriting that storage.
            self.access(
                state,
                Place::Temporary(value),
                Mode::Mutable,
                Depth::Shallow,
                None,
            )?;
        }
        let words = &self.replacements.words[&value];
        let cost = words.len();
        for &(word, mask) in words {
            state.retain(word, !mask);
        }
        self.charge(cost)
    }
}
