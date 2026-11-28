//! Writes can replace the reference identity used to reach a loan.
use super::{Depth, Flow, Id, Index, Loan, Mode, Place, Set, ValueId};
use crate::support::hash::HashMap;

#[derive(Default)]
pub(super) struct Replacements {
    temporaries: HashMap<ValueId, Vec<(usize, u64)>>,
    slots: HashMap<Id, Vec<(usize, u64)>>,
    entries: usize,
    pub steps: usize,
}
impl Replacements {
    pub fn new(loans: &[Loan], places: &mut Index) -> Result<Self, &'static str> {
        if loans.len() > 500_000 {
            return Err("temporary loan replacement storage budget exceeded");
        }
        let mut result = Self::default();
        for (index, loan) in loans.iter().enumerate() {
            result.steps += 1;
            if let Place::Temporary(value) = places.root(loan.place) {
                append(
                    result.temporaries.entry(value).or_default(),
                    index,
                    &mut result.entries,
                )?;
            }
            let mut slot = places.reference_slot(loan.place)?;
            while let Some(place) = slot {
                result.steps += 1;
                append(
                    result.slots.entry(place).or_default(),
                    index,
                    &mut result.entries,
                )?;
                slot = places.parent(place);
            }
        }
        Ok(result)
    }
}
fn append(
    words: &mut Vec<(usize, u64)>,
    index: usize,
    entries: &mut usize,
) -> Result<(), &'static str> {
    let word = index / 64;
    let bit = 1 << (index % 64);
    if let Some((_, bits)) = words.last_mut().filter(|(last, _)| *last == word) {
        *bits |= bit;
    } else {
        if *entries >= 500_000 {
            return Err("loan replacement storage budget exceeded");
        }
        *entries += 1;
        words.push((word, bit));
    }
    Ok(())
}

fn retire(state: &mut Set, words: &[(usize, u64)]) -> usize {
    for &(word, mask) in words {
        state.retain(word, !mask);
    }
    words.len()
}

impl Flow<'_> {
    pub(super) fn replace(
        &mut self,
        state: &mut Set,
        value: ValueId,
        inspect: bool,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        if !self.replacements.temporaries.contains_key(&value) {
            return Ok(());
        }
        if inspect
            && !self
                .body
                .promotions
                .as_ref()
                .is_some_and(|promotions| promotions.contains(value))
        {
            // Replacing a pointer does not access its old referent. Borrowing
            // the temporary's own storage still forbids overwriting that storage,
            // except when promotion placed it in immutable static storage.
            self.access(
                state,
                Place::Temporary(value),
                Mode::Mutable,
                Depth::Shallow,
                None,
            )?;
        }
        let cost = retire(state, &self.replacements.temporaries[&value]);
        self.charge(cost)
    }

    pub(super) fn replace_slot(
        &mut self,
        state: &mut Set,
        place: Place,
    ) -> Result<(), &'static str> {
        if self.replacements.slots.is_empty() {
            return Ok(());
        }
        let place = self.places.place(place)?;
        self.charge(1)?;
        if let Some(words) = self.replacements.slots.get(&place) {
            let cost = retire(state, words);
            self.charge(cost)?;
        }
        Ok(())
    }
}
