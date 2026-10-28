//! Must-initialized / may-written fixed point over sparse path intervals.
use super::initialization::{Maybe, Status, Transition};
use super::{
    BodyFacts, Error, Path,
    cleanup::{Event, Flow},
};
use crate::body::{
    ir::{BlockId, Body, Terminator},
    storage::Location,
};
use std::collections::VecDeque;

#[derive(Clone, Copy, Default)]
pub(super) struct Word<I: Maybe> {
    must: u64,
    may: u64,
    initialized: I,
}
pub(super) struct State<I: Maybe> {
    words: Vec<Word<I>>,
    remaining: usize,
}
pub(super) trait Mutations {
    fn fill(&mut self, path: &Path, initialized: bool) -> Result<(), &'static str>;
    fn written(&mut self, bit: u32, value: bool);
}
impl<I: Maybe> Mutations for State<I> {
    fn fill(&mut self, path: &Path, initialized: bool) -> Result<(), &'static str> {
        Self::fill(self, path, initialized)
    }
    fn written(&mut self, bit: u32, value: bool) {
        Self::written(self, bit, value);
    }
}
pub(super) fn change<I: Maybe>(
    state: &mut State<I>,
    location: Location,
    path: &Path,
    initialized: bool,
    mut visitor: impl FnMut(Transition, &mut State<I>) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    visitor(
        Transition::Initialize {
            location,
            start: path.start,
            end: path.end,
            initialized,
        },
        state,
    )?;
    state.fill(path, initialized)
}
struct Replay<'a, V> {
    state: &'a mut State<u64>,
    location: Location,
    visitor: V,
}
impl<V: FnMut(Transition, &mut State<u64>) -> Result<(), &'static str>> Mutations
    for Replay<'_, V>
{
    fn fill(&mut self, path: &Path, initialized: bool) -> Result<(), &'static str> {
        change(
            self.state,
            self.location,
            path,
            initialized,
            &mut self.visitor,
        )
    }
    fn written(&mut self, bit: u32, value: bool) {
        self.state.written(bit, value);
    }
}
impl<I: Maybe> State<I> {
    pub(super) fn charge(&mut self, cost: usize) -> Result<(), &'static str> {
        self.remaining = self
            .remaining
            .checked_sub(cost)
            .ok_or("ownership work budget exceeded")?;
        Ok(())
    }
    pub(super) fn fill(&mut self, path: &Path, initialized: bool) -> Result<(), &'static str> {
        if path.start == path.end {
            return Ok(());
        }
        self.charge(((path.end - 1) / 64 - path.start / 64 + 1) as usize)?;
        range(path, |index, mask| {
            self.words[index].initialized.assign(mask, initialized);
            if initialized {
                self.words[index].must |= mask;
            } else {
                self.words[index].must &= !mask;
            }
        });
        Ok(())
    }
    /// Exclude inactive payloads from possible destruction, preserving read permission.
    pub(super) fn exclude(&mut self, start: u32, end: u32) -> Result<(), &'static str> {
        if start > end {
            return Err("invalid cleanup refinement interval");
        }
        let mut bit = start;
        while bit < end {
            self.charge(1)?;
            let offset = bit % 64;
            let width = (end - bit).min(64 - offset);
            let mask = (u64::MAX >> (64 - width)) << offset;
            self.words
                .get_mut(bit as usize / 64)
                .ok_or("invalid cleanup refinement bit")?
                .initialized
                .assign(mask, false);
            bit += width;
        }
        Ok(())
    }
    pub(super) fn initialized(&mut self, path: &Path) -> Result<bool, &'static str> {
        if path.start == path.end {
            return Ok(true);
        }
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
pub(super) fn verify<I: Maybe>(
    body: &Body,
    facts: &mut BodyFacts,
    types: &crate::types::Store,
    flow: Option<&Flow<'_>>,
    order: Option<&crate::borrow::order::Order>,
    refinements: Option<&super::variant_liveness::Refinements>,
) -> Result<Vec<Word<I>>, Error> {
    let flow = if I::TRACK_VALUES { flow } else { None };
    let error = |message| Error {
        node: body.source.into(),
        message,
    };
    if body.blocks.is_empty() {
        return Err(error("body has no ownership entry"));
    }
    let words = (facts.paths.bits as usize).div_ceil(64);
    let size = words
        .checked_mul(body.blocks.len())
        .filter(|n| *n <= 64_000_000 / std::mem::size_of::<Word<I>>())
        .ok_or_else(|| error("ownership state exceeds 64 MB budget"))?;
    let computed;
    let order = if order.is_some() {
        order
    } else {
        computed = crate::borrow::order::Order::new(body);
        computed.as_ref()
    };
    if let Some(order) = order.filter(|order| order.acyclic)
        && let Ok(incoming) = verify_acyclic(
            body,
            facts,
            types,
            flow,
            (words, size),
            (order, refinements),
        )
    {
        return Ok(incoming);
    }
    verify_iterative(body, facts, types, flow, (words, size), refinements)
}
/// Canonical fixed point followed by one inspection pass in block order.
fn verify_iterative<I: Maybe>(
    body: &Body,
    facts: &mut BodyFacts,
    types: &crate::types::Store,
    flow: Option<&Flow<'_>>,
    (words, size): (usize, usize),
    refinements: Option<&super::variant_liveness::Refinements>,
) -> Result<Vec<Word<I>>, Error> {
    let error = |message| Error {
        node: body.source.into(),
        message,
    };
    let mut incoming = vec![Word::<I>::default(); size];
    let mut outgoing = vec![Word::<I>::default(); if flow.is_some() { words } else { 0 }];
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
        if flow.is_some() {
            transfer_block(body, facts, &mut state, index, None, flow)?;
        } else {
            transfer_plain(body, facts, &mut state, index, None)?;
        }
        let edges = flow.filter(|f| f.storage.has_edge_ends(index));
        let refined = refinements.is_some_and(|r| r.has(index));
        if edges.is_some() || refined {
            state.charge(words).map_err(error)?;
            outgoing.copy_from_slice(&state.words);
        }
        for target in successors(block.terminator).into_iter().flatten() {
            transfer_edge(
                facts,
                &mut state,
                (index, target),
                (edges.is_some() || refined).then_some(outgoing.as_slice()),
                edges,
                refinements,
            )
            .map_err(error)?;
            let next = target.0 as usize;
            let target = &mut incoming[next * words..(next + 1) * words];
            let mut changed = !reached[next];
            if reached[next] {
                for (old, new) in target.iter_mut().zip(&state.words) {
                    let must = old.must & new.must;
                    let may = old.may | new.may;
                    let initialized = old.initialized.join(new.initialized);
                    changed |= must != old.must || may != old.may || initialized != old.initialized;
                    *old = Word {
                        must,
                        may,
                        initialized,
                    };
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
        if flow.is_some() {
            transfer_block(body, facts, &mut state, index, Some(types), flow)?;
        } else {
            transfer_plain(body, facts, &mut state, index, Some(types))?;
        }
    }
    facts.reachable = reached;
    Ok(incoming)
}

/// In reverse postorder of an acyclic CFG every predecessor's final state
/// precedes a block, so one inspected pass computes the fixed point. Any
/// failure is discarded: the caller reruns the canonical passes, which report
/// the first error in block order.
fn verify_acyclic<I: Maybe>(
    body: &Body,
    facts: &mut BodyFacts,
    types: &crate::types::Store,
    flow: Option<&Flow<'_>>,
    (words, size): (usize, usize),
    (order, refinements): (
        &crate::borrow::order::Order,
        Option<&super::variant_liveness::Refinements>,
    ),
) -> Result<Vec<Word<I>>, Error> {
    let error = |message| Error {
        node: body.source.into(),
        message,
    };
    let mut incoming = vec![Word::<I>::default(); size];
    let mut outgoing = vec![Word::<I>::default(); if flow.is_some() { words } else { 0 }];
    let mut state = State {
        words: vec![Word::default(); words],
        remaining: 32_000_000,
    };
    let mut reached = vec![false; body.blocks.len()];
    reached[0] = true;
    for &index in &order.blocks {
        let block = &body.blocks[index];
        state
            .charge(words * 2 + (block.end - block.start) as usize + 1)
            .map_err(error)?;
        state
            .words
            .copy_from_slice(&incoming[index * words..(index + 1) * words]);
        // Inspection only reads the state, so the inspected transfer leaves
        // exactly the state of the uninspected fixed-point transfer.
        if flow.is_some() {
            transfer_block(body, facts, &mut state, index, Some(types), flow)?;
        } else {
            transfer_plain(body, facts, &mut state, index, Some(types))?;
        }
        let edges = flow.filter(|f| f.storage.has_edge_ends(index));
        let refined = refinements.is_some_and(|r| r.has(index));
        if edges.is_some() || refined {
            state.charge(words).map_err(error)?;
            outgoing.copy_from_slice(&state.words);
        }
        for target in successors(block.terminator).into_iter().flatten() {
            transfer_edge(
                facts,
                &mut state,
                (index, target),
                (edges.is_some() || refined).then_some(outgoing.as_slice()),
                edges,
                refinements,
            )
            .map_err(error)?;
            let next = target.0 as usize;
            let target = &mut incoming[next * words..(next + 1) * words];
            if reached[next] {
                for (old, new) in target.iter_mut().zip(&state.words) {
                    *old = Word {
                        must: old.must & new.must,
                        may: old.may | new.may,
                        initialized: old.initialized.join(new.initialized),
                    };
                }
            } else {
                target.copy_from_slice(&state.words);
                reached[next] = true;
            }
        }
    }
    facts.reachable = reached;
    Ok(incoming)
}

/// Each successor starts from the same outgoing initialization image.
fn transfer_edge<I: Maybe>(
    facts: &BodyFacts,
    state: &mut State<I>,
    (index, target): (usize, BlockId),
    outgoing: Option<&[Word<I>]>,
    flow: Option<&Flow<'_>>,
    refinements: Option<&super::variant_liveness::Refinements>,
) -> Result<(), &'static str> {
    if let Some(outgoing) = outgoing {
        state.charge(outgoing.len())?;
        state.words.copy_from_slice(outgoing);
    }
    if let Some(refinements) = refinements {
        refinements.apply(&facts.paths, index, target, state)?;
    }
    if let Some(flow) = flow {
        flow.edge(
            &facts.paths,
            state,
            Location::Edge {
                block: BlockId(u32::try_from(index).map_err(|_| "too many ownership blocks")?),
                target,
            },
            |_, _| Ok(()),
        )?;
    }
    Ok(())
}
/// The transfer of a block without cleanup flow, optionally inspecting each
/// statement against its incoming state first.
fn transfer_plain<I: Maybe>(
    body: &Body,
    facts: &BodyFacts,
    state: &mut State<I>,
    index: usize,
    types: Option<&crate::types::Store>,
) -> Result<(), Error> {
    let block = &body.blocks[index];
    for statement in &body.statements[block.start as usize..block.end as usize] {
        let error = |message| {
            let node = body.diagnostic_site(statement.source);
            Error { node, message }
        };
        if let Some(types) = types {
            facts
                .inspect(body, types, state, statement.action)
                .map_err(error)?;
        }
        facts
            .transfer(body, state, statement.action)
            .map_err(error)?;
    }
    Ok(())
}
fn transfer_block<I: Maybe>(
    body: &Body,
    facts: &BodyFacts,
    state: &mut State<I>,
    index: usize,
    types: Option<&crate::types::Store>,
    flow: Option<&Flow<'_>>,
) -> Result<(), Error> {
    let block = &body.blocks[index];
    let error = |source, message| {
        let node = body.diagnostic_site(source);
        Error { node, message }
    };
    let block_id = BlockId(
        u32::try_from(index).map_err(|_| error(block.source, "too many ownership blocks"))?,
    );
    let mut cursor = flow.map(|flow| flow.cursor(index));
    for before in block.start..=block.end {
        let source = if before < block.end {
            body.statements[before as usize].source
        } else {
            block.source
        };
        let error = |message| error(source, message);
        let location = Location::At {
            block: block_id,
            before,
        };
        if let Some(cursor) = &mut cursor {
            cursor
                .point(
                    &facts.paths,
                    state,
                    location,
                    types.is_some(),
                    |_, _| Ok(()),
                )
                .map_err(error)?;
        }
        if before == block.end {
            break;
        }
        let action = body.statements[before as usize].action;
        if let Some(types) = types {
            facts.inspect(body, types, state, action).map_err(error)?;
        }
        if let Some(flow) = flow {
            flow.statement(
                &facts.paths,
                state,
                location,
                types.is_some(),
                |_, _| Ok(()),
            )
            .map_err(error)?;
        }
        facts.transfer(body, state, action).map_err(error)?;
    }
    Ok(())
}

pub(super) fn status(words: &[Word<u64>], path: &Path) -> Status {
    let mut must = true;
    let mut may = false;
    range(path, |index, mask| {
        let initialized = words[index].initialized & mask;
        // Excluded inactive fields can remain initialized for read permission.
        // Cleanup is uniformly live only when both planes cover this interval.
        must &= words[index].must & mask == mask && initialized == mask;
        may |= initialized != 0;
    });
    if !may {
        Status::Never
    } else if must {
        Status::Always
    } else {
        Status::Conditional
    }
}

impl State<u64> {
    pub(super) fn classify_bit(&mut self, bit: u32) -> Result<Status, &'static str> {
        self.charge(1)?;
        bit_status(&self.words, bit)
    }
    pub(super) fn classify(&mut self, path: &Path) -> Result<Status, &'static str> {
        if path.start != path.end {
            self.charge(((path.end - 1) / 64 - path.start / 64 + 1) as usize)?;
        }
        Ok(status(&self.words, path))
    }
}

pub(super) fn bit_status(words: &[Word<u64>], bit: u32) -> Result<Status, &'static str> {
    let word = words
        .get(bit as usize / 64)
        .ok_or("invalid initialization bit")?;
    let mask = 1_u64 << (bit % 64);
    Ok(if word.initialized & mask == 0 {
        Status::Never
    } else if word.must & mask != 0 {
        Status::Always
    } else {
        Status::Conditional
    })
}

pub(super) fn replay(
    body: &Body,
    facts: &BodyFacts,
    flow: &Flow<'_>,
    incoming: &[Word<u64>],
    words: usize,
    refinements: Option<&super::variant_liveness::Refinements>,
    mut visitor: impl FnMut(Transition, &mut State<u64>) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    let mut state = State {
        words: vec![Word::default(); words],
        remaining: 32_000_000,
    };
    let mut outgoing = vec![Word::default(); words];
    for (index, block) in body.blocks.iter().enumerate() {
        if !facts.reachable[index] {
            continue;
        }
        state.charge(words + (block.end - block.start) as usize + 1)?;
        state
            .words
            .copy_from_slice(&incoming[index * words..(index + 1) * words]);
        let block_id = crate::body::ir::BlockId(
            u32::try_from(index).map_err(|_| "too many initialization blocks")?,
        );
        let mut cursor = flow.cursor(index);
        for before in block.start..=block.end {
            let location = Location::At {
                block: block_id,
                before,
            };
            cursor.point(&facts.paths, &mut state, location, false, &mut visitor)?;
            visitor(Transition::Observe(Event::Point(location)), &mut state)?;
            if before < block.end {
                flow.statement(&facts.paths, &mut state, location, false, &mut visitor)?;
                visitor(Transition::Statement(location), &mut state)?;
                facts.transfer(
                    body,
                    &mut Replay {
                        state: &mut state,
                        location,
                        visitor: &mut visitor,
                    },
                    body.statements[before as usize].action,
                )?;
            }
        }
        if flow.storage.has_edge_ends(index) {
            state.charge(words)?;
            outgoing.copy_from_slice(&state.words);
            for target in successors(block.terminator).into_iter().flatten() {
                state.charge(words)?;
                state.words.copy_from_slice(&outgoing);
                if let Some(refinements) = refinements {
                    refinements.apply(&facts.paths, index, target, &mut state)?;
                }
                flow.edge(
                    &facts.paths,
                    &mut state,
                    Location::Edge {
                        block: block_id,
                        target,
                    },
                    &mut visitor,
                )?;
            }
        }
    }
    Ok(())
}

fn successors(terminator: Terminator) -> [Option<BlockId>; 2] {
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
