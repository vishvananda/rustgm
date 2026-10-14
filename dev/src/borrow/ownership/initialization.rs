//! Retained owned initialization, distinct from whether an immutable binding was written.
pub use super::cleanup::{Candidate, Cause, Event};
pub use super::consumption::{Consumption, Disposition, Effects, Moment};
use super::{
    BodyFacts, Error, Path,
    cleanup::{Flow, Owners},
    state,
};
use crate::{
    body::{
        ir::{BlockId, Body},
        storage::Location,
    },
    types::{
        ItemId, Store,
        aggregate::Definitions,
        copy::{Copies, Properties},
    },
};
use std::{
    collections::{HashMap, HashSet},
    hash::BuildHasher,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// No bit in the queried range is initialized on any incoming path.
    Never,
    /// Some bits may be initialized. A partial aggregate must be split further.
    Conditional,
    /// Every bit is initialized on every incoming path.
    Always,
}
pub struct Facts<'a> {
    body: &'a Body,
    pub ownership: BodyFacts,
    pub consumption: Effects,
    pub storage: crate::borrow::storage::Analysis,
    owners: Owners,
    incoming: Vec<state::Word<u64>>,
    words: usize,
}
impl Facts<'_> {
    /// Check non-Copy field moves against resolved custom destructor identities.
    /// Whole-object moves and Copy field reads remain valid.
    /// # Errors
    /// Rejects moves that would leave a custom destructor a partially moved object.
    pub fn check_destructor_moves<S: BuildHasher>(
        &self,
        types: &Store,
        custom: &HashSet<ItemId, S>,
    ) -> Result<(), Error> {
        super::destructors::check(types, custom, self.body, &self.ownership)
    }
    /// Inspect an owned path at a reachable block entry, before its first statement.
    /// Partial paths require examining their tracked children separately.
    /// # Errors
    /// Rejects unreachable blocks and invalid bit intervals.
    pub fn at_entry(&self, block: BlockId, path: &Path) -> Result<Status, &'static str> {
        validate_path(path, self.ownership.paths.bits, self.words)?;
        Ok(state::status(self.entry(block)?, path))
    }
    /// # Errors
    /// Rejects unreachable blocks and invalid initialization bit identities.
    pub fn bit_at_entry(&self, block: BlockId, bit: u32) -> Result<Status, &'static str> {
        if bit >= self.ownership.paths.bits {
            return Err("invalid initialization bit");
        }
        state::bit_status(self.entry(block)?, bit)
    }
    fn entry(&self, block: BlockId) -> Result<&[state::Word<u64>], &'static str> {
        if self.ownership.reachable.get(block.0 as usize).copied() != Some(true) {
            return Err("initialization block is unreachable");
        }
        let start = block.0 as usize * self.words;
        self.incoming
            .get(start..start + self.words)
            .ok_or("invalid initialization block extent")
    }
    /// Replay the proved transfers once, exposing bounded queries before each
    /// statement and at block end. Ordered scope exits and returns precede the
    /// callback; a statement consumes its operands after it.
    /// The callback cannot mutate initialization.
    /// # Errors
    /// Reports query/transfer budget exhaustion or a callback error.
    pub fn visit(
        &self,
        mut visitor: impl FnMut(Location, &mut Point<'_>) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        self.visit_events(|event, point| match event {
            Event::Point(location) => visitor(location, point),
            Event::Cleanup(_) => Ok(()),
        })
    }
    /// Inspect ordered cleanup candidates immediately before their state is
    /// cleared, together with ordinary statement points. CFG edges are replayed
    /// independently. Candidates can have Never state and need no runtime action.
    /// This does not prove destructor-induced loans or select aggregate drop glue.
    /// # Errors
    /// Reports query/transfer budget exhaustion or a callback error.
    pub fn visit_events(
        &self,
        mut visitor: impl FnMut(Event, &mut Point<'_>) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        state::replay(
            self.body,
            &self.ownership,
            &self.flow(),
            &self.incoming,
            self.words,
            |event, state| {
                visitor(
                    event,
                    &mut Point {
                        state,
                        bits: self.ownership.paths.bits,
                        words: self.words,
                    },
                )
            },
        )
    }
    fn flow(&self) -> Flow<'_> {
        Flow {
            body: self.body,
            storage: &self.storage,
            owners: &self.owners,
            consumption: &self.consumption,
        }
    }
}

pub struct Point<'a> {
    state: &'a mut state::State<u64>,
    bits: u32,
    words: usize,
}
impl Point<'_> {
    /// # Errors
    /// Rejects invalid bits and charges one word to the replay budget.
    pub fn bit(&mut self, bit: u32) -> Result<Status, &'static str> {
        if bit >= self.bits {
            return Err("invalid initialization bit");
        }
        self.state.classify_bit(bit)
    }
    /// # Errors
    /// Rejects invalid paths and charges each examined word to the replay budget.
    pub fn status(&mut self, path: &Path) -> Result<Status, &'static str> {
        validate_path(path, self.bits, self.words)?;
        self.state.classify(path)
    }
}
fn validate_path(path: &Path, bits: u32, words: usize) -> Result<(), &'static str> {
    if path.start > path.end || path.end > bits || (path.end as usize).div_ceil(64) > words {
        return Err("invalid initialization path");
    }
    Ok(())
}

/// Retain the same move/access proof with a separate may-be-initialized plane.
/// This does not prove destructor liveness or grant an executable token.
/// # Errors
/// Reports ownership/access errors and bounded dataflow resource exhaustion.
pub fn analyze<'a, S: BuildHasher>(
    types: &Store,
    definitions: &Definitions,
    derives: &HashMap<ItemId, Properties, S>,
    copies: &mut Copies,
    body: &'a Body,
) -> Result<Facts<'a>, Error> {
    let mut ownership = super::prepare(types, definitions, derives, copies, body, true)?;
    let consumption = Effects::build(body, types, &ownership.paths).map_err(|message| Error {
        node: body.source,
        message,
    })?;
    let storage = crate::borrow::storage::check(body).map_err(|error| Error {
        node: error.node,
        message: error.message,
    })?;
    let owners = Owners::build(body, &ownership.paths).map_err(|message| Error {
        node: body.source,
        message,
    })?;
    let flow = Flow {
        body,
        storage: &storage,
        owners: &owners,
        consumption: &consumption,
    };
    let incoming = state::verify::<u64>(body, &mut ownership, types, Some(&flow))?;
    super::guards::verify(body, &ownership, types)?;
    let words = (ownership.paths.bits as usize).div_ceil(64);
    Ok(Facts {
        body,
        ownership,
        consumption,
        storage,
        owners,
        incoming,
        words,
    })
}

pub(super) trait Maybe: Copy + Default + PartialEq {
    const TRACK_VALUES: bool;
    fn assign(&mut self, mask: u64, initialized: bool);
    fn join(self, other: Self) -> Self;
}
impl Maybe for () {
    const TRACK_VALUES: bool = false;
    fn assign(&mut self, _: u64, _: bool) {}
    fn join(self, (): Self) -> Self {}
}
impl Maybe for u64 {
    const TRACK_VALUES: bool = true;
    fn assign(&mut self, mask: u64, initialized: bool) {
        if initialized {
            *self |= mask;
        } else {
            *self &= !mask;
        }
    }
    fn join(self, other: Self) -> Self {
        self | other
    }
}
