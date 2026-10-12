//! Definite initialization and write permission for the scalar executable profile.
// Joins intersect definitely initialized bits and union possibly written bits.
// Storage markers reset both sets, including locals recreated on loop iterations.
use super::ir::{Action, Body, LocalId, LocalKind, Place, Program, Rvalue, Terminator};
use crate::resolve::unit::NodeKey;
use std::collections::VecDeque;

pub struct Error {
    pub node: NodeKey,
    pub message: &'static str,
}
pub struct Checked<'a> {
    program: &'a Program,
}
impl Checked<'_> {
    #[must_use]
    pub const fn program(&self) -> &Program {
        self.program
    }
}
/// Verify scalar reads/writes before executable lowering, without proving moves or regions.
///
/// # Errors
/// Rejects uninitialized reachable reads, immutable reassignment, and dataflow budgets.
pub fn check(program: &Program) -> Result<Checked<'_>, Error> {
    for body in &program.bodies {
        verify(body)?;
    }
    Ok(Checked { program })
}
#[derive(Clone, Copy, Default)]
struct Word {
    must: u64,
    may: u64,
}
fn cell(state: &mut [Word], id: LocalId) -> (&mut Word, u64) {
    (&mut state[id.0 as usize / 64], 1_u64 << (id.0 % 64))
}
fn transfer(state: &mut [Word], action: Action, body: &Body) {
    match action {
        Action::StorageLive(id)
            if matches!(body.locals[id.0 as usize].kind, LocalKind::Parameter(_)) =>
        {
            let (word, bit) = cell(state, id);
            word.must |= bit;
            word.may |= bit;
        }
        Action::StorageLive(id) | Action::StorageDead(id) => {
            let (word, bit) = cell(state, id);
            word.must &= !bit;
            word.may &= !bit;
        }
        Action::Store {
            place: Place::Local(id),
            ..
        } => {
            let (word, bit) = cell(state, id);
            word.must |= bit;
            word.may |= bit;
        }
        _ => {}
    }
}
fn verify(body: &Body) -> Result<(), Error> {
    let error = |message| Error {
        node: body.source,
        message,
    };
    if body.blocks.is_empty() {
        return Err(error("body has no initialization entry"));
    }
    let words = body.locals.len().div_ceil(64);
    let size = words
        .checked_mul(body.blocks.len())
        .filter(|n| *n <= 4_000_000)
        .ok_or_else(|| error("scalar initialization state exceeds 64 MB budget"))?;
    let mut incoming = vec![Word::default(); size];
    let mut scratch = vec![Word::default(); words];
    let mut reached = vec![false; body.blocks.len()];
    let mut queued = vec![false; body.blocks.len()];
    let mut work = VecDeque::from([0_usize]);
    reached[0] = true;
    queued[0] = true;
    let mut budget = 32_000_000_usize;
    while let Some(index) = work.pop_front() {
        queued[index] = false;
        let block = &body.blocks[index];
        let cost = words * 3 + (block.end - block.start) as usize + 1;
        budget = budget
            .checked_sub(cost)
            .ok_or_else(|| error("scalar initialization work budget exceeded"))?;
        scratch.copy_from_slice(&incoming[index * words..(index + 1) * words]);
        for statement in &body.statements[block.start as usize..block.end as usize] {
            transfer(&mut scratch, statement.action, body);
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
                for (old, new) in target.iter_mut().zip(&scratch) {
                    let must = old.must & new.must;
                    let may = old.may | new.may;
                    changed |= must != old.must || may != old.may;
                    *old = Word { must, may };
                }
            } else {
                target.copy_from_slice(&scratch);
            }
            reached[next] = true;
            if changed && !queued[next] {
                queued[next] = true;
                work.push_back(next);
            }
        }
    }
    inspect(body, &incoming, &reached, &mut scratch, words)
}
fn inspect(
    body: &Body,
    incoming: &[Word],
    reached: &[bool],
    scratch: &mut [Word],
    words: usize,
) -> Result<(), Error> {
    // Inspect only converged states, never a transient worklist approximation.
    for (index, block) in body.blocks.iter().enumerate() {
        if !reached[index] {
            continue;
        }
        scratch.copy_from_slice(&incoming[index * words..(index + 1) * words]);
        for statement in &body.statements[block.start as usize..block.end as usize] {
            let fault = match statement.action {
                Action::Evaluate(id) => {
                    if let Rvalue::Read(Place::Local(local)) = body.values[id.0 as usize].kind {
                        let (word, bit) = cell(scratch, local);
                        (word.must & bit == 0).then_some("local may be uninitialized at this read")
                    } else {
                        None
                    }
                }
                Action::Store {
                    place: Place::Local(local),
                    ..
                } => {
                    let (word, bit) = cell(scratch, local);
                    (!body.locals[local.0 as usize].mutable && word.may & bit != 0)
                        .then_some("cannot assign twice to an immutable local")
                }
                _ => None,
            };
            if let Some(message) = fault {
                return Err(Error {
                    node: NodeKey {
                        file: body.source.file,
                        node: statement.source,
                    },
                    message,
                });
            }
            transfer(scratch, statement.action, body);
        }
    }
    Ok(())
}
