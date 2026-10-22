//! Flat indexes share scope membership and preserve event order at equal CFG points.
//!
//! The body builder establishes every metadata invariant the index relies on,
//! so release proofs of builder output do not re-check them:
//! - scope records are appended with an already existing parent (the root
//!   `Body` scope alone has none), so each parent precedes its child
//!   (`sema::scopes::Walker::scope`, `Facts::with_scratch`);
//! - statements are appended only to the single open block, which is opened
//!   once (`body::build::Builder::{switch, statement}`), so block statement
//!   ranges are disjoint and every terminator targets a block made by
//!   `new_block`;
//! - temporaries and scope exits are recorded at the current statement count
//!   of the open block, with the number of exits recorded so far
//!   (`body::build::Builder::materialize_place` and `body/build/storage.rs`),
//!   so within each block their emission order is also their point order;
//! - an edge exit names a successor of the terminator its block receives
//!   immediately afterwards (`Work::Branch` and the conditional failures).
//!
//! Instances clone these records unchanged (`body::instantiate::materialize`).
//! Debug builds and [`super::validate`] still check every invariant; the
//! validating entry point additionally accepts arbitrary event order.
use super::state::{self, State};
use crate::body::{
    ir::{BlockId, Body, Place},
    scopes,
    storage::{End, Location},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EventKind {
    Birth(u32),
    End(End),
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Event {
    pub position: u32,
    pub order: u64,
    pub kind: EventKind,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Edge {
    pub target: BlockId,
    pub end: End,
}
#[derive(Clone, PartialEq, Eq)]
pub(super) struct Mask {
    pub word: u32,
    pub bits: u64,
}
/// Whether the metadata comes unmodified from the body builder.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Trust {
    Builder,
    Validate,
}
#[derive(Clone, PartialEq, Eq)]
pub(super) struct Index {
    /// Empty when the body materializes no temporary: every lookup then
    /// reports the same missing materialization as an explicit `None` slot.
    pub values: Vec<Option<u32>>,
    roots: Vec<Option<u32>>,
    /// Leading projections whose roots are untracked (no temporary storage).
    untracked: usize,
    pub permanent: Vec<bool>,
    pub permanent_masks: Vec<u64>,
    members: Vec<Mask>,
    scope_starts: Vec<usize>,
    events: Vec<Event>,
    event_starts: Vec<usize>,
    edges: Vec<Edge>,
    edge_starts: Vec<usize>,
}
impl Index {
    pub(super) fn new(body: &Body, state: &mut State, trust: Trust) -> Result<Self, &'static str> {
        let storage = body.storage.as_ref().ok_or("missing body storage facts")?;
        let scopes = &storage.scopes;
        let estimates = [
            (
                body.values.len() + body.projections.len(),
                size_of::<Option<u32>>(),
            ),
            (body.temporaries.len(), 48 + size_of::<Event>()),
            (
                storage.exits.len(),
                size_of::<Event>() + size_of::<Edge>() + 32,
            ),
            (scopes.len() + body.blocks.len() * 2 + 3, size_of::<usize>()),
        ];
        estimates
            .into_iter()
            .try_fold(0_usize, |total, (count, width)| {
                total.checked_add(count.checked_mul(width)?)
            })
            .filter(|n| *n <= 64_000_000)
            .ok_or("storage indexes exceed 64 MB budget")?;
        state.charge(
            body.values.len() + body.projections.len() + scopes.len() + storage.exits.len(),
        )?;
        if trust == Trust::Validate || cfg!(debug_assertions) {
            validate(body, state)?;
        }
        let tracked = !body.temporaries.is_empty();
        let mut index = Self {
            values: if tracked {
                vec![None; body.values.len()]
            } else {
                Vec::new()
            },
            roots: Vec::with_capacity(if tracked { body.projections.len() } else { 0 }),
            untracked: 0,
            permanent: Vec::with_capacity(body.temporaries.len()),
            permanent_masks: vec![0; body.temporaries.len().div_ceil(64)],
            members: Vec::new(),
            scope_starts: Vec::new(),
            events: Vec::new(),
            event_starts: Vec::new(),
            edges: Vec::new(),
            edge_starts: Vec::new(),
        };
        if tracked {
            index.births(body, state)?;
        }
        index.timeline(body, state, trust)?;
        for projection in &body.projections {
            let base = index.root(projection.base)?;
            if !tracked {
                // Without temporaries every valid base is untracked storage.
                index.untracked += 1;
                continue;
            }
            let root = if projection.kind.is_dereference() {
                None
            } else {
                base
            };
            index.roots.push(root);
        }
        Ok(index)
    }
    fn births(&mut self, body: &Body, state: &mut State) -> Result<(), &'static str> {
        let mut members = Vec::new();
        for (bit, temporary) in body.temporaries.iter().enumerate() {
            state.charge(1)?;
            let bit = u32::try_from(bit).map_err(|_| "too many storage identities")?;
            *self
                .values
                .get_mut(temporary.value.0 as usize)
                .ok_or("invalid materialized value")? = Some(bit);
            let promoted = body
                .promotions
                .as_ref()
                .is_some_and(|p| p.contains(temporary.value));
            let permanent = promoted || temporary.lifetime == scopes::Lifetime::Static;
            self.permanent.push(permanent);
            if permanent {
                self.permanent_masks[bit as usize / 64] |= 1 << (bit % 64);
            } else if let scopes::Lifetime::Scope(scope) = temporary.lifetime {
                members.push((scope.0, bit));
            }
        }
        state.charge(
            members
                .len()
                .saturating_mul(members.len().max(1).ilog2() as usize + 1),
        )?;
        members.sort_unstable();
        let scope_count = body.storage.as_ref().map_or(0, |s| s.scopes.len());
        self.scope_starts = vec![0; scope_count + 1];
        let mut previous = None;
        for (scope, bit) in members {
            let key = (scope, bit / 64);
            let mask = 1 << (bit % 64);
            if previous == Some(key) {
                self.members
                    .last_mut()
                    .ok_or("missing storage scope mask")?
                    .bits |= mask;
            } else {
                self.members.push(Mask {
                    word: bit / 64,
                    bits: mask,
                });
                self.scope_starts[scope as usize + 1] += 1;
                previous = Some(key);
            }
        }
        prefix_sums(&mut self.scope_starts);
        Ok(())
    }
    /// Bucket births and point exits by block in their emission order, and
    /// edge exits by source block in exit order. Emission order interleaves a
    /// birth before exit `j` exactly when it saw at most `j` earlier exits.
    fn timeline(
        &mut self,
        body: &Body,
        state: &mut State,
        trust: Trust,
    ) -> Result<(), &'static str> {
        let storage = body.storage.as_ref().ok_or("missing body storage facts")?;
        let blocks = body.blocks.len();
        state.charge(body.temporaries.len() + storage.exits.len() + blocks)?;
        let mut event_starts = vec![0; blocks + 1];
        let mut edge_starts = vec![0; blocks + 1];
        for temporary in &body.temporaries {
            event_starts[temporary.block.0 as usize + 1] += 1;
        }
        for exit in &storage.exits {
            match exit.location {
                Location::At { block, .. } => event_starts[block.0 as usize + 1] += 1,
                Location::Edge { block, .. } => edge_starts[block.0 as usize + 1] += 1,
            }
        }
        prefix_sums(&mut event_starts);
        prefix_sums(&mut edge_starts);
        let placeholder = Event {
            position: 0,
            order: 0,
            kind: EventKind::Birth(0),
        };
        let mut events = vec![placeholder; event_starts[blocks]];
        // Edge exits keep exit order within each source block, exactly as a
        // stable sort by block would.
        let mut edges = vec![
            Edge {
                target: BlockId(0),
                end: End::Temporaries { start: 0, end: 0 },
            };
            edge_starts[blocks]
        ];
        let mut event_cursor = event_starts.clone();
        let mut edge_cursor = edge_starts.clone();
        let mut births = body.temporaries.iter().enumerate().peekable();
        for (ordinal, exit) in storage.exits.iter().enumerate() {
            while let Some((bit, temporary)) =
                births.next_if(|(_, t)| t.exits_before as usize <= ordinal)
            {
                place_birth(&mut events, &mut event_cursor, bit, temporary)?;
            }
            let order = u64::try_from(ordinal).map_err(|_| "too many storage events")? * 2 + 1;
            match exit.location {
                Location::At { block, before } => {
                    let slot = &mut event_cursor[block.0 as usize];
                    events[*slot] = Event {
                        position: before,
                        order,
                        kind: EventKind::End(exit.end),
                    };
                    *slot += 1;
                }
                Location::Edge { block, target } => {
                    let slot = &mut edge_cursor[block.0 as usize];
                    edges[*slot] = Edge {
                        target,
                        end: exit.end,
                    };
                    *slot += 1;
                }
            }
        }
        for (bit, temporary) in births {
            place_birth(&mut events, &mut event_cursor, bit, temporary)?;
        }
        self.events = events;
        self.event_starts = event_starts;
        self.edges = edges;
        self.edge_starts = edge_starts;
        if trust == Trust::Validate || cfg!(debug_assertions) {
            self.order_points(state, trust)?;
        }
        Ok(())
    }
    /// Each block's events must be ordered by point, then emission order.
    /// Builder output already is; validated input may record events in any
    /// order and is ordered canonically, exactly as one stable sort would.
    fn order_points(&mut self, state: &mut State, trust: Trust) -> Result<(), &'static str> {
        for block in 0..self.event_starts.len().saturating_sub(1) {
            let bucket = &mut self.events[self.event_starts[block]..self.event_starts[block + 1]];
            if bucket
                .windows(2)
                .all(|pair| (pair[0].position, pair[0].order) <= (pair[1].position, pair[1].order))
            {
                continue;
            }
            if trust == Trust::Builder {
                return Err("storage events violate builder construction order");
            }
            state.charge(
                bucket
                    .len()
                    .saturating_mul(bucket.len().ilog2() as usize + 1),
            )?;
            bucket.sort_by_key(|e| (e.position, e.order));
        }
        Ok(())
    }
    pub(super) fn root(&self, place: Place) -> Result<Option<u32>, &'static str> {
        match place {
            Place::Temporary(id) => self
                .values
                .get(id.0 as usize)
                .copied()
                .flatten()
                .map(Some)
                .ok_or("place lacks a materialization record"),
            Place::Projection(id) if (id.0 as usize) < self.untracked => Ok(None),
            Place::Projection(id) => self
                .roots
                .get(id.0 as usize)
                .copied()
                .ok_or("invalid storage projection ancestry"),
            Place::Local(_) | Place::Static(_) => Ok(None),
        }
    }
    pub(super) fn members(&self, scope: scopes::Id) -> &[Mask] {
        if self.members.is_empty() {
            return &[];
        }
        &self.members[self.scope_starts[scope.0 as usize]..self.scope_starts[scope.0 as usize + 1]]
    }
    pub(super) fn events(&self, block: usize) -> &[Event] {
        &self.events[self.event_starts[block]..self.event_starts[block + 1]]
    }
    pub(super) fn edges(&self, block: usize) -> &[Edge] {
        &self.edges[self.edge_starts[block]..self.edge_starts[block + 1]]
    }
}
fn place_birth(
    events: &mut [Event],
    cursor: &mut [usize],
    bit: usize,
    temporary: &crate::body::ir::Temporary,
) -> Result<(), &'static str> {
    let slot = &mut cursor[temporary.block.0 as usize];
    events[*slot] = Event {
        position: temporary.before,
        order: u64::from(temporary.exits_before) * 2,
        kind: EventKind::Birth(u32::try_from(bit).map_err(|_| "too many storage identities")?),
    };
    *slot += 1;
    Ok(())
}
/// Re-check every builder invariant the index relies on: scope ancestry,
/// materialization identities and points, exit ranges and locations, and the
/// block statement ranges and successors that the dataflow walks.
fn validate(body: &Body, state: &mut State) -> Result<(), &'static str> {
    let storage = body.storage.as_ref().ok_or("missing body storage facts")?;
    let scopes = &storage.scopes;
    for (i, scope) in scopes.iter().enumerate() {
        if (i == 0) != scope.parent.is_none() || scope.parent.is_some_and(|p| p.0 as usize >= i) {
            return Err("invalid storage scope ancestry");
        }
    }
    if scopes.is_empty() {
        return Err("missing root storage scope");
    }
    for block in &body.blocks {
        if block.start > block.end || block.end as usize > body.statements.len() {
            return Err("invalid storage block statements");
        }
        if state::successors(block.terminator)
            .into_iter()
            .flatten()
            .any(|id| id.0 as usize >= body.blocks.len())
        {
            return Err("invalid storage successor");
        }
    }
    let mut materialized = vec![false; body.values.len()];
    for temporary in &body.temporaries {
        let seen = materialized
            .get_mut(temporary.value.0 as usize)
            .ok_or("invalid materialized value")?;
        if std::mem::replace(seen, true) {
            return Err("repeated temporary materialization");
        }
        validate_point(body, temporary.block, temporary.before)?;
        if temporary.exits_before as usize > storage.exits.len() {
            return Err("invalid materialization event order");
        }
        if let scopes::Lifetime::Scope(scope) = temporary.lifetime
            && scope.0 as usize >= scopes.len()
        {
            return Err("invalid temporary scope");
        }
    }
    // Transfer walks every scope exit again when it ends storage; repeated
    // adjacent ranges, such as successive failure exits, are walked once.
    let mut previous = None;
    for exit in &storage.exits {
        match exit.end {
            End::Scopes(range) => {
                if range.inner.0 as usize >= scopes.len()
                    || range.outer.is_some_and(|id| id.0 as usize >= scopes.len())
                {
                    return Err("invalid storage exit scope");
                }
                if previous.replace(range) != Some(range) {
                    let mut next = Some(range.inner);
                    while next != range.outer {
                        state.charge(1)?;
                        let scope = next
                            .and_then(|id| scopes.get(id.0 as usize))
                            .ok_or("storage exit is not an ancestor range")?;
                        next = scope.parent;
                    }
                }
            }
            End::Temporaries { start, end } => {
                if start > end || end as usize > body.temporaries.len() {
                    return Err("invalid temporary exit range");
                }
            }
        }
        match exit.location {
            Location::At { block, before } => validate_point(body, block, before)?,
            Location::Edge { block, target } => {
                let data = body
                    .blocks
                    .get(block.0 as usize)
                    .ok_or("invalid storage edge source")?;
                if !state::successors(data.terminator).contains(&Some(target)) {
                    return Err("storage exit has no matching CFG edge");
                }
            }
        }
    }
    Ok(())
}
fn validate_point(body: &Body, block: BlockId, point: u32) -> Result<(), &'static str> {
    let block = body
        .blocks
        .get(block.0 as usize)
        .ok_or("invalid storage event block")?;
    if point < block.start || point > block.end {
        return Err("storage event outside its block");
    }
    Ok(())
}
fn prefix_sums(values: &mut [usize]) {
    for i in 1..values.len() {
        values[i] += values[i - 1];
    }
}
