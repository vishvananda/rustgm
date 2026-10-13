//! A bounded must-live fixed point, with separate successor-specific scope exits.
use super::{Index, end_storage, transfer};
use crate::body::ir::{Body, Terminator};
use std::collections::VecDeque;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Word {
    pub must: u64,
    pub may: u64,
}
pub(super) struct State {
    pub words: Vec<Word>,
    pub steps: usize,
}
impl State {
    pub(super) fn new(bits: usize) -> Self {
        Self {
            words: vec![Word::default(); bits.div_ceil(64)],
            steps: 0,
        }
    }
    pub(super) fn charge(&mut self, amount: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(amount)
            .filter(|n| *n <= 32_000_000)
            .ok_or("storage liveness work budget exceeded")?;
        Ok(())
    }
    pub(super) fn start(&mut self, bit: u32) -> Result<(), &'static str> {
        self.charge(1)?;
        let word = self
            .words
            .get_mut(bit as usize / 64)
            .ok_or("invalid storage bit")?;
        let mask = 1 << (bit % 64);
        word.must |= mask;
        word.may |= mask;
        Ok(())
    }
    pub(super) fn clear(&mut self, word: u32, mask: u64) -> Result<(), &'static str> {
        self.charge(1)?;
        let word = self
            .words
            .get_mut(word as usize)
            .ok_or("invalid storage mask")?;
        word.must &= !mask;
        word.may &= !mask;
        Ok(())
    }
    pub(super) fn live(&self, bit: u32) -> bool {
        self.words[bit as usize / 64].must & (1 << (bit % 64)) != 0
    }
    pub(super) fn maybe_live(&self, bit: u32) -> bool {
        self.words[bit as usize / 64].may & (1 << (bit % 64)) != 0
    }
}
pub(super) fn successors(terminator: Terminator) -> [Option<crate::body::ir::BlockId>; 2] {
    match terminator {
        Terminator::Jump(target) => [Some(target), None],
        Terminator::Branch { yes, no, .. }
        | Terminator::FalseEdge {
            real: yes,
            imaginary: no,
        } => [Some(yes), Some(no)],
        Terminator::Return(_) | Terminator::Unreachable => [None, None],
    }
}
pub(super) fn solve(
    body: &Body,
    index: &Index,
    state: &mut State,
) -> Result<(Vec<Word>, Vec<bool>), &'static str> {
    if body.blocks.is_empty() {
        return Err("body has no storage entry");
    }
    let words = state.words.len();
    let size = words
        .checked_mul(body.blocks.len())
        .filter(|n| *n <= 4_000_000)
        .ok_or("storage liveness state exceeds 64 MB budget")?;
    let mut incoming = vec![Word::default(); size];
    let mut outgoing = vec![Word::default(); words];
    let mut reached = vec![false; body.blocks.len()];
    let mut queued = vec![false; body.blocks.len()];
    let mut pending = VecDeque::from([0_usize]);
    reached[0] = true;
    queued[0] = true;
    while let Some(block) = pending.pop_front() {
        queued[block] = false;
        state.charge(words * 2 + 1)?;
        state
            .words
            .copy_from_slice(&incoming[block * words..(block + 1) * words]);
        transfer(body, index, state, block, false)?;
        outgoing.copy_from_slice(&state.words);
        for target in successors(body.blocks[block].terminator)
            .into_iter()
            .flatten()
        {
            state.charge(words * 2 + 1)?;
            state.words.copy_from_slice(&outgoing);
            for edge in index.edges(block) {
                if edge.target == target {
                    end_storage(body, index, state, edge.end)?;
                }
            }
            let next = target.0 as usize;
            let entry = &mut incoming[next * words..(next + 1) * words];
            let mut changed = !reached[next];
            if reached[next] {
                for (old, value) in entry.iter_mut().zip(&state.words) {
                    let new = Word {
                        must: old.must & value.must,
                        may: old.may | value.may,
                    };
                    changed |= *old != new;
                    *old = new;
                }
            } else {
                entry.copy_from_slice(&state.words);
            }
            reached[next] = true;
            if changed && !queued[next] {
                pending.push_back(next);
                queued[next] = true;
            }
        }
    }
    Ok((incoming, reached))
}
