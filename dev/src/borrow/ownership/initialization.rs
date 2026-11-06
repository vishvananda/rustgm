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
        ir::{BlockId, Body, ValueId},
        storage::{End, Location},
    },
    types::{
        ItemId, Store,
        aggregate::Definitions,
        copy::{Copies, Declaration},
    },
};
use std::hash::BuildHasher;

/// Physical cleanup refinement and const eligibility have independent activation.
#[derive(Clone, Copy)]
pub(crate) enum Refinement {
    Physical,
    Defining,
    EligibilityOnly,
}
impl Refinement {
    pub(super) const fn physical(self) -> bool {
        !matches!(self, Self::EligibilityOnly)
    }
    pub(super) const fn eligibility(self) -> bool {
        !matches!(self, Self::Physical)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// No bit in the queried range is initialized on any incoming path.
    Never,
    /// Some bits may be initialized. A partial aggregate must be split further.
    Conditional,
    /// Every bit is initialized on every incoming path.
    Always,
}
/// The ordered state transitions used for runtime flag elaboration. Observation
/// happens before mutation; Statement is the actual body action, after operand
/// consumption and replacement cleanup. Its initialization writes follow it.
#[derive(Clone, Copy, Debug)]
pub enum Transition {
    Observe(Event),
    Statement(Location),
    /// Materialized storage exists, even when it needs no ownership path.
    Birth {
        location: Location,
        value: ValueId,
    },
    /// A single scope or initializer temporary ends after its own cleanup.
    StorageEnd {
        location: Location,
        end: End,
    },
    Initialize {
        location: Location,
        start: u32,
        end: u32,
        initialized: bool,
    },
}
pub struct Facts<'a> {
    body: &'a Body,
    pub ownership: BodyFacts,
    pub consumption: Effects,
    /// Proved for this body, or shared by every instance of its template.
    pub storage: std::borrow::Cow<'a, crate::borrow::storage::Analysis>,
    owners: Owners,
    incoming: Vec<state::Word<u64>>,
    words: usize,
    refinements: Option<Box<super::variant_liveness::Refinements>>,
}
impl<'a> Facts<'a> {
    /// Check non-Copy field moves against resolved custom destructor identities.
    /// Whole-object moves and Copy field reads remain valid.
    /// # Errors
    /// Rejects moves that would leave a custom destructor a partially moved object.
    pub fn check_destructor_moves<S: BuildHasher>(
        &self,
        types: &Store,
        custom: &std::collections::HashSet<ItemId, S>,
    ) -> Result<(), Error> {
        self.destructor_moves(types, |item| custom.contains(&item))
    }
    pub(crate) const fn body(&self) -> &'a Body {
        self.body
    }
    pub(crate) fn destructor_moves(
        &self,
        types: &Store,
        custom: impl Fn(ItemId) -> bool,
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
        self.visit_transfers(|transition, point| match transition {
            Transition::Observe(event) => visitor(event, point),
            _ => Ok(()),
        })
    }
    /// Replay exact initialization writes, including scope exits, materialized
    /// births and ownership transfers, in their evaluation order. Intervals use
    /// sparse path bits; array lengths do not expand them. CFG edges are independent.
    /// # Errors
    /// Reports query/transfer budget exhaustion or a callback error.
    pub fn visit_transfers(
        &self,
        mut visitor: impl FnMut(Transition, &mut Point<'_>) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        state::replay(
            self.body,
            &self.ownership,
            &self.flow(),
            &self.incoming,
            self.words,
            self.refinements.as_deref(),
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
    pub(crate) fn visit_selected(
        &self,
        mut visitor: impl FnMut(
            Transition,
            &mut Point<'_>,
        ) -> Result<
            Option<crate::borrow::destruction::schedule::Cleanup>,
            &'static str,
        >,
    ) -> Result<Option<Box<super::Selected>>, &'static str> {
        if let Some(refinements) = &self.refinements {
            return refinements.visit_selected(self, visitor);
        }
        self.visit_transfers(|event, point| visitor(event, point).map(|_| ()))?;
        Ok(None)
    }
    pub(crate) fn install_selected(
        &mut self,
        selected: Option<Box<super::Selected>>,
    ) -> Result<(), &'static str> {
        if let Some(selected) = selected {
            self.refinements
                .as_mut()
                .ok_or("selected cleanup has no refinement owner")?
                .install_selected(self.body, selected)?;
        }
        Ok(())
    }
    pub(crate) fn resolve_selected(
        &mut self,
        types: &mut Store,
        definitions: &Definitions,
    ) -> Result<(), &'static str> {
        if let Some(refinements) = &mut self.refinements {
            refinements.resolve_selected(types, definitions, self.body)?;
        }
        Ok(())
    }
    pub(crate) fn selected(&self) -> Option<&super::Selected> {
        self.refinements.as_deref()?.selected()
    }
    /// Refine possible payload cleanup without changing read or move permission.
    pub(crate) fn refine_variants(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        custom: &dyn Fn(ItemId) -> bool,
        mode: Refinement,
        position: &impl Fn(
            crate::types::TypeId,
            crate::body::drops::MemberQuery,
        ) -> Result<crate::body::drops::MemberPosition, &'static str>,
    ) -> Result<usize, &'static str> {
        let refinements = if let Some(mut pending) = self.refinements.take() {
            pending.apply_queries()?;
            *pending
        } else {
            self.prepare_variants(types, definitions, custom, (mode, None), position)?
        };
        self.finish_variants(types, refinements)
    }
    pub(crate) fn begin_qualification(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        custom: &dyn Fn(ItemId) -> bool,
        (mode, requirements): (Refinement, Option<&crate::body::drops::Requirements>),
        position: &impl Fn(
            crate::types::TypeId,
            crate::body::drops::MemberQuery,
        ) -> Result<crate::body::drops::MemberPosition, &'static str>,
    ) -> Result<(), &'static str> {
        if let Some(requirements) = requirements
            && (requirements.locals.len() != self.body.locals.len()
                || requirements.values.len() != self.body.values.len()
                || requirements.projections.len() != self.body.projections.len())
        {
            return Err("const qualification requirements belong to another body extent");
        }
        let refinements =
            self.prepare_variants(types, definitions, custom, (mode, requirements), position)?;
        if refinements.has_partition() {
            self.refinements = Some(Box::new(refinements));
        }
        Ok(())
    }
    pub(crate) fn qualification_questions(
        &mut self,
        types: &mut Store,
        definitions: &Definitions,
    ) -> Result<Option<&[crate::types::TypeId]>, &'static str> {
        self.refinements
            .as_deref_mut()
            .map(|refinements| {
                refinements.prepare_queries(types, definitions, self.body, &self.ownership.paths)
            })
            .transpose()
            .map(Option::flatten)
    }
    pub(crate) fn accept_qualification(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        regions: &crate::borrow::regions::Body,
        answers: &crate::body::defining_cleanups::Qualification<'_>,
    ) -> Result<(), &'static str> {
        self.refinements
            .as_deref_mut()
            .ok_or("const qualification lost its initialized owner")?
            .accept_queries(self.body, types, definitions, regions, answers)
    }
    fn prepare_variants(
        &self,
        types: &Store,
        definitions: &Definitions,
        custom: &dyn Fn(ItemId) -> bool,
        (mode, requirements): (Refinement, Option<&crate::body::drops::Requirements>),
        position: &impl Fn(
            crate::types::TypeId,
            crate::body::drops::MemberQuery,
        ) -> Result<crate::body::drops::MemberPosition, &'static str>,
    ) -> Result<super::variant_liveness::Refinements, &'static str> {
        if self.refinements.is_some() {
            return Err("cleanup variant refinement was repeated");
        }
        let occupied = self
            .incoming
            .capacity()
            .saturating_add(self.words.saturating_mul(2))
            .saturating_mul(size_of::<state::Word<u64>>());
        let available = 64_000_000_usize.saturating_sub(occupied);
        super::variant_liveness::Refinements::new(
            self.body,
            &self.ownership.paths,
            types,
            position,
            available,
            mode.eligibility()
                .then_some(super::variant_liveness::EligibilityInput {
                    definitions,
                    custom,
                    requirements,
                }),
            mode.physical(),
        )
    }
    fn finish_variants(
        &mut self,
        types: &Store,
        refinements: super::variant_liveness::Refinements,
    ) -> Result<usize, &'static str> {
        if refinements.is_empty() {
            return Ok(refinements.steps);
        }
        let flow = Flow {
            body: self.body,
            storage: &self.storage,
            owners: &self.owners,
            consumption: &self.consumption,
        };
        // Release the replaced table before computing its refined image.
        drop(std::mem::take(&mut self.incoming));
        self.incoming = state::verify::<u64>(
            self.body,
            &mut self.ownership,
            types,
            Some(&flow),
            None,
            Some(&refinements),
        )
        .map_err(|e| e.message)?;
        self.words = (refinements.bits(self.ownership.paths.bits) as usize).div_ceil(64);
        let steps = refinements.steps;
        self.refinements = Some(Box::new(refinements));
        Ok(steps)
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
    pub(super) fn possible(&mut self, bit: u32) -> Result<bool, &'static str> {
        self.state.possible(bit)
    }
    /// Eligibility history distinguishes an actually partial value from a
    /// conditional whole value. This reads physical facts and grants no access.
    pub(super) fn eligibility_partial(&mut self, path: &Path) -> Result<bool, &'static str> {
        if path.start == path.end {
            return Ok(false);
        }
        Ok(self.state.classify_bit(path.start)? == Status::Always
            && !self.state.initialized(path)?)
    }
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
    derives: &std::collections::HashMap<ItemId, Declaration, S>,
    copies: &mut Copies,
    body: &'a Body,
) -> Result<Facts<'a>, Error> {
    let order = crate::borrow::order::Order::new(body);
    analyze_ordered(
        types,
        definitions,
        derives,
        &mut crate::borrow::copy_view::CopyView::ordinary(copies),
        body,
        order.as_ref(),
    )
}
/// `analyze`, reusing a reverse-postorder walk already computed for the body.
#[inline]
pub(crate) fn analyze_ordered<'a, S: BuildHasher>(
    types: &Store,
    definitions: &Definitions,
    derives: &std::collections::HashMap<ItemId, Declaration, S>,
    copies: &mut crate::borrow::copy_view::CopyView<'_>,
    body: &'a Body,
    order: Option<&crate::borrow::order::Order>,
) -> Result<Facts<'a>, Error> {
    analyze_with(types, definitions, derives, copies, body, order, None)
}
/// `analyze`, reusing the order walk and storage proof an instance shares with
/// every other instance of its checked template (see `borrow::check::structure`).
#[inline]
pub(crate) fn analyze_shared<'a, S: BuildHasher>(
    types: &Store,
    definitions: &Definitions,
    derives: &std::collections::HashMap<ItemId, Declaration, S>,
    copies: &mut crate::borrow::copy_view::CopyView<'_>,
    body: &'a Body,
    order: Option<&crate::borrow::order::Order>,
    storage: &'a crate::borrow::storage::Analysis,
) -> Result<Facts<'a>, Error> {
    analyze_with(
        types,
        definitions,
        derives,
        copies,
        body,
        order,
        Some(storage),
    )
}
fn analyze_with<'a, S: BuildHasher>(
    types: &Store,
    definitions: &Definitions,
    derives: &std::collections::HashMap<ItemId, Declaration, S>,
    copies: &mut crate::borrow::copy_view::CopyView<'_>,
    body: &'a Body,
    order: Option<&crate::borrow::order::Order>,
    shared: Option<&'a crate::borrow::storage::Analysis>,
) -> Result<Facts<'a>, Error> {
    let mut ownership = super::prepare(types, definitions, derives, copies, body, true)?;
    let consumption = Effects::build(body, types, &ownership.paths).map_err(|message| Error {
        node: body.source.into(),
        message,
    })?;
    let storage = if let Some(shared) = shared {
        std::borrow::Cow::Borrowed(shared)
    } else {
        std::borrow::Cow::Owned(crate::borrow::storage::check_ordered(body, order).map_err(
            |error| Error {
                node: error.node,
                message: error.message,
            },
        )?)
    };
    let owners = Owners::build(body, &ownership.paths).map_err(|message| Error {
        node: body.source.into(),
        message,
    })?;
    let flow = Flow {
        body,
        storage: &storage,
        owners: &owners,
        consumption: &consumption,
    };
    let incoming = state::verify::<u64>(body, &mut ownership, types, Some(&flow), order, None)?;
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
        refinements: None,
    })
}

pub(super) trait Maybe: Copy + Default + PartialEq {
    const TRACK_VALUES: bool;
    fn assign(&mut self, mask: u64, initialized: bool);
    fn join(self, other: Self) -> Self;
    fn bits(self) -> u64;
}
impl Maybe for () {
    const TRACK_VALUES: bool = false;
    fn assign(&mut self, _: u64, _: bool) {}
    fn join(self, (): Self) -> Self {}
    fn bits(self) -> u64 {
        0
    }
}
impl Maybe for u64 {
    const TRACK_VALUES: bool = true;
    fn bits(self) -> u64 {
        self
    }
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
