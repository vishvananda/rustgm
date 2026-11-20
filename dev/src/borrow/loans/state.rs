//! Forward states keep dense CFG joins and index only occupied words for point checks.
use super::State;
use crate::borrow::active::Live;
use crate::borrow::regions::points::Point;

pub(super) struct Set {
    words: Vec<State>,
    occupied: Live,
    pub(super) valid_at: Option<Point>,
}
impl Set {
    pub fn new(words: usize) -> Self {
        Self {
            words: vec![State { live: 0, active: 0 }; words],
            occupied: Live::new(words.div_ceil(64)),
            valid_at: None,
        }
    }
    pub fn words(&self) -> &[State] {
        &self.words
    }
    pub fn is_empty(&self) -> bool {
        self.next(0).is_none()
    }
    pub fn depth(&self) -> usize {
        self.occupied.depth() + 1
    }
    pub fn next(&self, from: usize) -> Option<usize> {
        self.occupied.next_bit(from)
    }
    pub fn iter(&self) -> impl Iterator<Item = (usize, State)> + '_ {
        let mut from = 0;
        std::iter::from_fn(move || {
            let index = self.next(from)?;
            from = index + 1;
            Some((index, self.words[index]))
        })
    }
    pub fn copy_from(&mut self, source: &[State]) {
        self.valid_at = None;
        self.words.copy_from_slice(source);
        self.occupied.clear();
        for (index, word) in self.words.iter().enumerate() {
            if word.live != 0 {
                self.occupied.mark(index, true);
            }
        }
    }
    /// The first successor still has the post-block words and occupied index.
    /// Each successor starts without its predecessor's expiration evidence.
    pub fn restore_edge(&mut self, outgoing: &[State], first: bool) {
        if first {
            self.valid_at = None;
        } else {
            self.copy_from(outgoing);
        }
    }
    pub fn retain(&mut self, index: usize, mask: u64) {
        let word = &mut self.words[index];
        word.live &= mask;
        word.active &= mask;
        if word.live == 0 {
            self.occupied.mark(index, false);
        }
    }
    pub fn generate(&mut self, loan: usize, live_at_point: bool) {
        // A new loan outside its region needs the general check at the next
        // point; an expiration event alone cannot remove an already-dead loan.
        if !live_at_point {
            self.valid_at = None;
        }
        let word = &mut self.words[loan / 64];
        word.live |= 1 << (loan % 64);
        word.active &= !(1 << (loan % 64));
        self.occupied.mark(loan / 64, true);
    }
    pub fn activate(&mut self, loan: usize) {
        let word = &mut self.words[loan / 64];
        let mask = 1 << (loan % 64);
        word.active |= word.live & mask;
    }
}

#[cfg(test)]
mod tests {
    use super::{Point, Set, State};
    #[test]
    fn pruning_and_regeneration_keep_activation_and_occupied_words_in_sync() {
        let mut state = Set::new(66);
        for loan in [0, 63, 64, 4096] {
            state.generate(loan, true);
            state.activate(loan);
        }
        assert_eq!(
            state.iter().map(|(word, _)| word).collect::<Vec<_>>(),
            [0, 1, 64]
        );
        state.retain(0, 1);
        state.retain(1, 0);
        state.generate(0, true);
        let words = state
            .iter()
            .map(|(word, value)| (word, value.live, value.active))
            .collect::<Vec<_>>();
        assert_eq!(words, [(0, 1, 0), (64, 1, 1)]);
        let mut incoming = vec![State { live: 0, active: 0 }; 66];
        incoming[65] = State { live: 3, active: 2 };
        state.copy_from(&incoming);
        assert_eq!(
            state
                .iter()
                .map(|(word, value)| (word, value.live, value.active))
                .collect::<Vec<_>>(),
            [(65, 3, 2)]
        );
        state.retain(65, 0);
        assert!(state.iter().next().is_none());
    }

    #[test]
    fn edges_invalidate_adjacency_and_keep_the_original_branch_state() {
        let mut state = Set::new(3);
        state.generate(0, true);
        state.generate(65, true);
        state.activate(65);
        let outgoing = state.words().to_vec();
        let occupied = |state: &Set| {
            state
                .iter()
                .map(|(word, value)| (word, value.live, value.active))
                .collect::<Vec<_>>()
        };
        state.valid_at = Some(Point(63));
        state.restore_edge(&outgoing, true);
        assert_eq!(state.valid_at, None);
        assert_eq!(occupied(&state), [(0, 1, 0), (1, 2, 2)]);

        state.retain(0, 0);
        state.retain(1, 0);
        state.generate(130, true);
        state.activate(130);
        state.valid_at = Some(Point(81));
        state.restore_edge(&outgoing, false);
        assert_eq!(state.valid_at, None);
        assert_eq!(occupied(&state), [(0, 1, 0), (1, 2, 2)]);
    }
}
