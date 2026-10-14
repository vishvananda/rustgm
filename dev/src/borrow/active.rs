//! Dense dataflow words with a bounded summary tree for sparse point enumeration.
const LEVELS: usize = usize::BITS.div_ceil(6) as usize;

#[derive(Clone)]
pub(super) struct Live {
    words: Vec<u64>,
    // Each bit records whether a child word is nonzero. The root has one word.
    summaries: Vec<Vec<u64>>,
}
impl Live {
    pub fn new(words: usize) -> Self {
        let mut summaries = Vec::new();
        let mut count = words;
        while count > 1 {
            count = count.div_ceil(64);
            summaries.push(vec![0; count]);
            if count == 1 {
                break;
            }
        }
        Self {
            words: vec![0; words],
            summaries,
        }
    }
    pub fn words(&self) -> &[u64] {
        &self.words
    }
    pub fn len(&self) -> usize {
        self.words.len()
    }
    pub fn depth(&self) -> usize {
        self.summaries.len().max(1)
    }
    pub fn clear(&mut self) {
        self.words.fill(0);
        for level in &mut self.summaries {
            level.fill(0);
        }
    }
    pub fn mark(&mut self, entity: usize, used: bool) {
        let index = entity / 64;
        let old = self.words[index];
        let mask = 1 << (entity % 64);
        let new = if used { old | mask } else { old & !mask };
        self.words[index] = new;
        if (old == 0) != (new == 0) {
            self.present(index, new != 0);
        }
    }
    fn present(&mut self, mut index: usize, present: bool) {
        for level in &mut self.summaries {
            let word = &mut level[index / 64];
            let old = *word;
            let mask = 1 << (index % 64);
            if present {
                *word |= mask;
            } else {
                *word &= !mask;
            }
            if (old == 0) == (*word == 0) {
                break;
            }
            index /= 64;
        }
    }
    pub fn copy_from(&mut self, source: &[u64]) {
        self.words.copy_from_slice(source);
        self.rebuild();
    }
    pub fn union(&mut self, source: &[u64]) {
        for (target, bits) in self.words.iter_mut().zip(source) {
            *target |= bits;
        }
        self.rebuild();
    }
    fn rebuild(&mut self) {
        let mut previous = self.words.as_slice();
        for level in &mut self.summaries {
            level.fill(0);
            for (index, &word) in previous.iter().enumerate() {
                if word != 0 {
                    level[index / 64] |= 1 << (index % 64);
                }
            }
            previous = level;
        }
    }
    pub fn active(&self) -> Active<'_> {
        let mut stack = [Frame::default(); LEVELS];
        let depth = self.depth();
        if let Some(root) = self.summaries.last() {
            stack[depth - 1].bits = root[0];
        } else if let Some(&word) = self.words.first() {
            stack[0].bits = u64::from(word != 0);
        }
        Active {
            live: self,
            stack,
            level: depth.checked_sub(1),
        }
    }
    /// Find the next set bit without retaining a borrow across caller mutation.
    pub fn next_bit(&self, mut from: usize) -> Option<usize> {
        for level in 0..=self.summaries.len() {
            let words = if level == 0 {
                &self.words
            } else {
                &self.summaries[level - 1]
            };
            let word = from / 64;
            let bits = *words.get(word)? & (u64::MAX << (from % 64));
            if bits != 0 {
                let mut found = word * 64 + bits.trailing_zeros() as usize;
                for child in (0..level).rev() {
                    let words = if child == 0 {
                        &self.words
                    } else {
                        &self.summaries[child - 1]
                    };
                    found = found * 64 + words[found].trailing_zeros() as usize;
                }
                return Some(found);
            }
            from = word + 1;
        }
        None
    }
}
#[derive(Clone, Copy, Default)]
struct Frame {
    base: usize,
    bits: u64,
}
pub(super) struct Active<'a> {
    live: &'a Live,
    stack: [Frame; LEVELS],
    level: Option<usize>,
}
impl Iterator for Active<'_> {
    type Item = (usize, u64);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let level = self.level?;
            let frame = &mut self.stack[level];
            if frame.bits == 0 {
                self.level = (level + 1 < self.live.depth()).then_some(level + 1);
                continue;
            }
            let index = frame.base + frame.bits.trailing_zeros() as usize;
            frame.bits &= frame.bits - 1;
            if level == 0 {
                return Some((index, self.live.words[index]));
            }
            self.stack[level - 1] = Frame {
                base: index * 64,
                bits: self.live.summaries[level - 1][index],
            };
            self.level = Some(level - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Live;
    #[test]
    fn summaries_preserve_order_across_word_and_tree_boundaries() {
        for words in [0, 1, 2, 64, 65, 4096, 4097] {
            let mut live = Live::new(words);
            let mut expected = vec![0_u64; words];
            for step in 0..1000_usize {
                if words == 0 {
                    break;
                }
                let entity = (step * 7919) % (words * 64);
                let used = step % 3 != 0;
                live.mark(entity, used);
                if used {
                    expected[entity / 64] |= 1 << (entity % 64);
                } else {
                    expected[entity / 64] &= !(1 << (entity % 64));
                }
                assert_eq!(live.words(), expected);
                let from = (step * 6271) % (words * 64);
                let next =
                    (from..words * 64).find(|&bit| expected[bit / 64] & (1 << (bit % 64)) != 0);
                assert_eq!(live.next_bit(from), next);
                assert_eq!(
                    live.active().collect::<Vec<_>>(),
                    expected
                        .iter()
                        .copied()
                        .enumerate()
                        .filter(|(_, word)| *word != 0)
                        .collect::<Vec<_>>()
                );
            }
            let mut copied = Live::new(words);
            copied.copy_from(&expected);
            live.clear();
            assert_eq!(live.active().next(), None);
            assert_eq!(live.next_bit(0), None);
            live.union(copied.words());
            assert_eq!(
                live.active().collect::<Vec<_>>(),
                copied.active().collect::<Vec<_>>()
            );
            for entity in (0..words * 64).rev() {
                live.mark(entity, false);
            }
            assert_eq!(live.active().next(), None);
        }
    }
}
