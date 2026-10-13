//! Flat indexes share scope membership and preserve event order at equal CFG points.
use super::state::{self, State};
use crate::body::{
    ir::{BlockId, Body, Place, ProjectionKind},
    scopes,
    storage::{End, Location},
};
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(super) enum EventKind {
    Birth(u32),
    End(End),
}
pub(super) struct Event {
    pub block: BlockId,
    pub position: u32,
    pub order: u64,
    pub kind: EventKind,
}
pub(super) struct Edge {
    pub block: BlockId,
    pub target: BlockId,
    pub end: End,
}
pub(super) struct Mask {
    pub word: u32,
    pub bits: u64,
}
pub(super) struct Index {
    pub values: Vec<Option<u32>>,
    roots: Vec<Option<u32>>,
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
    pub(super) fn new(body: &Body, state: &mut State) -> Result<Self, &'static str> {
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
        for (i, scope) in scopes.iter().enumerate() {
            if (i == 0) != scope.parent.is_none() || scope.parent.is_some_and(|p| p.0 as usize >= i)
            {
                return Err("invalid storage scope ancestry");
            }
        }
        if scopes.is_empty() {
            return Err("missing root storage scope");
        }
        let mut index = Self {
            values: vec![None; body.values.len()],
            roots: Vec::with_capacity(body.projections.len()),
            permanent: Vec::with_capacity(body.temporaries.len()),
            permanent_masks: vec![0; body.temporaries.len().div_ceil(64)],
            members: Vec::new(),
            scope_starts: Vec::new(),
            events: Vec::with_capacity(body.temporaries.len() + storage.exits.len()),
            event_starts: Vec::new(),
            edges: Vec::with_capacity(
                storage
                    .exits
                    .iter()
                    .filter(|e| matches!(e.location, Location::Edge { .. }))
                    .count(),
            ),
            edge_starts: Vec::new(),
        };
        index.births(body, state)?;
        index.exits(body, state)?;
        for projection in &body.projections {
            let base = index.root(projection.base)?;
            let root = if matches!(projection.kind, ProjectionKind::Dereference { .. }) {
                None
            } else {
                base
            };
            index.roots.push(root);
        }
        let count = index.events.len() + index.edges.len();
        state.charge(count.saturating_mul(count.max(1).ilog2() as usize + 1))?;
        index
            .events
            .sort_by_key(|e| (e.block.0, e.position, e.order));
        index.edges.sort_by_key(|e| e.block.0);
        index.event_starts = starts(
            body.blocks.len(),
            index.events.iter().map(|e| e.block.0 as usize),
        );
        index.edge_starts = starts(
            body.blocks.len(),
            index.edges.iter().map(|e| e.block.0 as usize),
        );
        Ok(index)
    }
    fn births(&mut self, body: &Body, state: &mut State) -> Result<(), &'static str> {
        let storage = body.storage.as_ref().ok_or("missing body storage facts")?;
        let mut members = Vec::new();
        for (bit, temporary) in body.temporaries.iter().enumerate() {
            state.charge(1)?;
            let bit = u32::try_from(bit).map_err(|_| "too many storage identities")?;
            let slot = self
                .values
                .get_mut(temporary.value.0 as usize)
                .ok_or("invalid materialized value")?;
            if slot.replace(bit).is_some() {
                return Err("repeated temporary materialization");
            }
            validate_point(body, temporary.block, temporary.before)?;
            if temporary.exits_before as usize > storage.exits.len() {
                return Err("invalid materialization event order");
            }
            let promoted = body
                .promotions
                .as_ref()
                .is_some_and(|p| p.get(temporary.value).is_some());
            self.permanent
                .push(promoted || temporary.lifetime == scopes::Lifetime::Static);
            if self.permanent[bit as usize] {
                self.permanent_masks[bit as usize / 64] |= 1 << (bit % 64);
            }
            if let scopes::Lifetime::Scope(scope) = temporary.lifetime {
                if scope.0 as usize >= storage.scopes.len() {
                    return Err("invalid temporary scope");
                }
                if !self.permanent[bit as usize] {
                    members.push((scope.0, bit));
                }
            }
            self.events.push(Event {
                block: temporary.block,
                position: temporary.before,
                order: u64::from(temporary.exits_before) * 2,
                kind: EventKind::Birth(bit),
            });
        }
        state.charge(
            members
                .len()
                .saturating_mul(members.len().max(1).ilog2() as usize + 1),
        )?;
        members.sort_unstable();
        self.scope_starts = vec![0; storage.scopes.len() + 1];
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
    fn exits(&mut self, body: &Body, state: &mut State) -> Result<(), &'static str> {
        let storage = body.storage.as_ref().ok_or("missing body storage facts")?;
        let mut ranges = HashSet::new();
        for (ordinal, exit) in storage.exits.iter().enumerate() {
            match exit.end {
                End::Scopes(range) => {
                    if range.inner.0 as usize >= storage.scopes.len()
                        || range
                            .outer
                            .is_some_and(|id| id.0 as usize >= storage.scopes.len())
                    {
                        return Err("invalid storage exit scope");
                    }
                    if ranges.insert((range.inner, range.outer)) {
                        let mut next = Some(range.inner);
                        while next != range.outer {
                            state.charge(1)?;
                            let scope = next
                                .and_then(|id| storage.scopes.get(id.0 as usize))
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
                Location::At { block, before } => {
                    validate_point(body, block, before)?;
                    self.events.push(Event {
                        block,
                        position: before,
                        order: u64::try_from(ordinal).map_err(|_| "too many storage events")? * 2
                            + 1,
                        kind: EventKind::End(exit.end),
                    });
                }
                Location::Edge { block, target } => {
                    let data = body
                        .blocks
                        .get(block.0 as usize)
                        .ok_or("invalid storage edge source")?;
                    if !state::successors(data.terminator).contains(&Some(target)) {
                        return Err("storage exit has no matching CFG edge");
                    }
                    self.edges.push(Edge {
                        block,
                        target,
                        end: exit.end,
                    });
                }
            }
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
            Place::Projection(id) => self
                .roots
                .get(id.0 as usize)
                .copied()
                .ok_or("invalid storage projection ancestry"),
            Place::Local(_) | Place::Static(_) => Ok(None),
        }
    }
    pub(super) fn members(&self, scope: scopes::Id) -> &[Mask] {
        &self.members[self.scope_starts[scope.0 as usize]..self.scope_starts[scope.0 as usize + 1]]
    }
    pub(super) fn events(&self, block: usize) -> &[Event] {
        &self.events[self.event_starts[block]..self.event_starts[block + 1]]
    }
    pub(super) fn edges(&self, block: usize) -> &[Edge] {
        &self.edges[self.edge_starts[block]..self.edge_starts[block + 1]]
    }
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
fn starts(count: usize, indices: impl Iterator<Item = usize>) -> Vec<usize> {
    let mut starts = vec![0; count + 1];
    for index in indices {
        starts[index + 1] += 1;
    }
    prefix_sums(&mut starts);
    starts
}
fn prefix_sums(values: &mut [usize]) {
    for i in 1..values.len() {
        values[i] += values[i - 1];
    }
}
