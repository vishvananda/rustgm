//! One depth-first walk per body yields reachability and reverse postorder.
use crate::body::ir::{BlockId, Body, Terminator};

/// Reachable blocks of a CFG whose successors are all valid identities.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Order {
    /// Reachable blocks in reverse postorder.
    pub blocks: Vec<usize>,
    pub reachable: Vec<bool>,
    /// No cycle is reachable from the entry, so reverse postorder visits every
    /// predecessor before its successors.
    pub acyclic: bool,
}
const NEW: u8 = 0;
const OPEN: u8 = 1;
const DONE: u8 = 2;
impl Order {
    /// `None` when the body has no entry or a reachable successor is invalid.
    pub(crate) fn new(body: &Body) -> Option<Self> {
        let count = body.blocks.len();
        if count == 0 {
            return None;
        }
        let mut state = vec![NEW; count];
        let mut blocks = Vec::with_capacity(count);
        let mut stack: Vec<(usize, [Option<BlockId>; 2], usize)> =
            Vec::with_capacity(count.min(64));
        let mut acyclic = true;
        state[0] = OPEN;
        stack.push((0, successors(body.blocks[0].terminator), 0));
        while let Some(top) = stack.last_mut() {
            let Some(&target) = top.1.get(top.2) else {
                let block = top.0;
                state[block] = DONE;
                blocks.push(block);
                stack.pop();
                continue;
            };
            top.2 += 1;
            let Some(target) = target else {
                continue;
            };
            let next = target.0 as usize;
            match *state.get(next)? {
                NEW => {
                    state[next] = OPEN;
                    stack.push((next, successors(body.blocks[next].terminator), 0));
                }
                OPEN => acyclic = false,
                _ => {}
            }
        }
        blocks.reverse();
        Some(Self {
            blocks,
            reachable: state.into_iter().map(|state| state == DONE).collect(),
            acyclic,
        })
    }
}
pub(crate) fn successors(terminator: Terminator) -> [Option<BlockId>; 2] {
    match terminator {
        Terminator::Jump(target) => [Some(target), None],
        Terminator::Branch { yes, no, .. }
        | Terminator::FalseEdge {
            real: yes,
            imaginary: no,
        } => [Some(yes), Some(no)],
        Terminator::Return(_) | Terminator::Unreachable | Terminator::Panic => [None, None],
    }
}
