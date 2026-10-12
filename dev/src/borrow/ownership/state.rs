//! Must-initialized / may-written fixed point over sparse path intervals.
use super::{BodyFacts, Error, Path};
use crate::body::ir::{Body, Terminator};
use std::collections::VecDeque;

#[derive(Clone, Copy, Default)]
struct Word {
    must: u64,
    may: u64,
}
pub(super) struct State {
    words: Vec<Word>,
    remaining: usize,
}
impl State {
    fn charge(&mut self, cost: usize) -> Result<(), &'static str> {
        self.remaining = self
            .remaining
            .checked_sub(cost)
            .ok_or("ownership work budget exceeded")?;
        Ok(())
    }
    pub(super) fn fill(&mut self, path: &Path, initialized: bool) -> Result<(), &'static str> {
        self.charge(((path.end - 1) / 64 - path.start / 64 + 1) as usize)?;
        range(path, |index, mask| {
            if initialized {
                self.words[index].must |= mask;
            } else {
                self.words[index].must &= !mask;
            }
        });
        Ok(())
    }
    pub(super) fn initialized(&mut self, path: &Path) -> Result<bool, &'static str> {
        self.charge(((path.end - 1) / 64 - path.start / 64 + 1) as usize)?;
        let mut result = true;
        range(path, |index, mask| {
            result &= self.words[index].must & mask == mask;
        });
        Ok(result)
    }
    pub(super) fn shallow(&self, bit: u32) -> bool {
        self.words[bit as usize / 64].must & (1 << (bit % 64)) != 0
    }
    pub(super) fn written(&mut self, bit: u32, value: bool) {
        let word = &mut self.words[bit as usize / 64];
        if value {
            word.may |= 1 << (bit % 64);
        } else {
            word.may &= !(1 << (bit % 64));
        }
    }
    pub(super) fn was_written(&self, bit: u32) -> bool {
        self.words[bit as usize / 64].may & (1 << (bit % 64)) != 0
    }
}
fn range(path: &Path, mut visit: impl FnMut(usize, u64)) {
    let mut bit = path.start;
    while bit < path.end {
        let offset = bit % 64;
        let width = (path.end - bit).min(64 - offset);
        let mask = (u64::MAX >> (64 - width)) << offset;
        visit(bit as usize / 64, mask);
        bit += width;
    }
}
pub(super) fn verify(body: &Body, facts: &mut BodyFacts) -> Result<(), Error> {
    let error = |message| Error {
        node: body.source,
        message,
    };
    if body.blocks.is_empty() {
        return Err(error("body has no ownership entry"));
    }
    let words = facts.paths.entries.len().div_ceil(64);
    let size = words
        .checked_mul(body.blocks.len())
        .filter(|n| *n <= 4_000_000)
        .ok_or_else(|| error("ownership state exceeds 64 MB budget"))?;
    let mut incoming = vec![Word::default(); size];
    let mut state = State {
        words: vec![Word::default(); words],
        remaining: 32_000_000,
    };
    let mut reached = vec![false; body.blocks.len()];
    let mut queued = vec![false; body.blocks.len()];
    let mut work = VecDeque::from([0_usize]);
    reached[0] = true;
    queued[0] = true;
    while let Some(index) = work.pop_front() {
        queued[index] = false;
        let block = &body.blocks[index];
        // Copy/joins plus dispatch; interval operations charge the words they
        // actually touch, so many unrelated locals do not cause quadratic charges.
        state
            .charge(words * 3 + (block.end - block.start) as usize + 1)
            .map_err(error)?;
        state
            .words
            .copy_from_slice(&incoming[index * words..(index + 1) * words]);
        for statement in body.statements(block) {
            facts
                .transfer(body, &mut state, statement.action)
                .map_err(error)?;
        }
        let successors = match block.terminator {
            Terminator::Jump(target) => [Some(target), None],
            Terminator::Branch { yes, no, .. } => [Some(yes), Some(no)],
            _ => [None, None],
        };
        for target in successors.into_iter().flatten() {
            let next = target.0 as usize;
            let target = &mut incoming[next * words..(next + 1) * words];
            let mut changed = !reached[next];
            if reached[next] {
                for (old, new) in target.iter_mut().zip(&state.words) {
                    let must = old.must & new.must;
                    let may = old.may | new.may;
                    changed |= must != old.must || may != old.may;
                    *old = Word { must, may };
                }
            } else {
                target.copy_from_slice(&state.words);
            }
            reached[next] = true;
            if changed && !queued[next] {
                queued[next] = true;
                work.push_back(next);
            }
        }
    }
    for (index, block) in body.blocks.iter().enumerate() {
        if !reached[index] {
            continue;
        }
        state
            .charge(words + (block.end - block.start) as usize + 1)
            .map_err(error)?;
        state
            .words
            .copy_from_slice(&incoming[index * words..(index + 1) * words]);
        for statement in body.statements(block) {
            let error = |message| {
                let mut node = body.source;
                node.node = statement.source;
                Error { node, message }
            };
            facts
                .inspect(body, &mut state, statement.action)
                .map_err(error)?;
            facts
                .transfer(body, &mut state, statement.action)
                .map_err(error)?;
        }
    }
    facts.reachable = reached;
    Ok(())
}
