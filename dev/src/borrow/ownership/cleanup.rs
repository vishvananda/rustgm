//! Ordered owner endpoints share the validated storage timeline and scope ancestry.
use super::{
    PathId, Paths, Root,
    consumption::{Disposition, Effects, Moment},
    initialization::{Maybe, Transition},
    paths::Access,
    state::{self, State},
};
use crate::{
    body::{
        ir::{Action, Body, LocalId, LocalKind, Place, Temporary},
        scopes::{self, Lifetime},
        storage::{End, Location, Range},
    },
    borrow::storage::{Analysis, Event as StorageEvent, EventKind},
};
use std::{cmp::Reverse, iter::Peekable, slice};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    Scope(scopes::Id),
    Initializer,
    Storage,
    Replacement,
    Discard,
}
/// A cleanup opportunity, before changing initialization. Never-initialized paths
/// need no drop. Mixed aggregates require field/segment decisions, not one flag.
#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub location: Location,
    pub cause: Cause,
    pub place: Place,
    /// Borrowed or dynamically indexed replacement destinations have no owned path.
    pub path: Option<PathId>,
}
#[derive(Clone, Copy, Debug)]
pub enum Event {
    Point(Location),
    Cleanup(Candidate),
}
pub(super) struct Owners {
    members: Vec<PathId>,
    starts: Vec<usize>,
}
impl Owners {
    pub(super) fn build(body: &Body, paths: &Paths) -> Result<Self, &'static str> {
        let storage = body
            .storage
            .as_ref()
            .ok_or("missing cleanup storage facts")?;
        let count = body.locals.len()
            + body.temporaries.len()
            + body.owned.as_ref().map_or(0, |o| o.created.len());
        count
            .checked_mul(48)
            .and_then(|n| n.checked_add(storage.scopes.len().checked_mul(size_of::<usize>())?))
            .filter(|n| *n <= 64_000_000)
            .ok_or("cleanup owner index exceeds 64 MB budget")?;
        let work = body.statements.len() + storage.scopes.len();
        work.checked_add(count.saturating_mul(count.max(1).ilog2() as usize + 3))
            .filter(|n| *n <= 32_000_000)
            .ok_or("cleanup owner index work budget exceeded")?;
        let mut members = Vec::with_capacity(count);
        let mut declared = vec![false; body.locals.len()];
        for (position, statement) in body.statements.iter().enumerate() {
            if let Action::StorageLive(local) = statement.action {
                let entry = body
                    .locals
                    .get(local.0 as usize)
                    .ok_or("invalid cleanup local")?;
                if std::mem::replace(&mut declared[local.0 as usize], true) {
                    return Err("repeated local storage declaration");
                }
                if !matches!(entry.kind, LocalKind::Guard(_)) {
                    members.push((
                        entry.scope.0,
                        Reverse(position as u64 * 2 + 1),
                        PathId(local.0),
                    ));
                }
            }
        }
        if declared.contains(&false) {
            return Err("cleanup local has no storage declaration");
        }
        if let Some(owned) = &body.owned {
            for temporary in &owned.created {
                let block = body
                    .blocks
                    .get(temporary.block.0 as usize)
                    .ok_or("invalid owned creation block")?;
                if temporary.before <= block.start
                    || temporary.before > block.end
                    || temporary.exits_before as usize > storage.exits.len()
                    || !matches!(body.statements[(temporary.before - 1) as usize].action,
                        Action::Evaluate(value) if value == temporary.value)
                {
                    return Err("invalid owned creation point");
                }
                let path = paths
                    .temporary_path(temporary.value)
                    .ok_or("owned cleanup temporary lacks an initialization path")?;
                Self::temporary(body, &mut members, temporary, path);
            }
        }
        for temporary in &body.temporaries {
            if body
                .owned
                .as_ref()
                .is_none_or(|o| o.creation(temporary.value).is_none())
                && let Some(path) = paths.temporary_path(temporary.value)
            {
                Self::temporary(body, &mut members, temporary, path);
            }
        }
        if members
            .iter()
            .any(|&(scope, _, _)| scope as usize >= storage.scopes.len())
        {
            return Err("invalid cleanup owner scope");
        }
        members.sort_unstable_by_key(|&(scope, order, path)| (scope, order, path.0));
        let mut starts = vec![0; storage.scopes.len() + 1];
        for &(scope, _, _) in &members {
            starts[scope as usize + 1] += 1;
        }
        for i in 1..starts.len() {
            starts[i] += starts[i - 1];
        }
        Ok(Self {
            members: members.into_iter().map(|(_, _, path)| path).collect(),
            starts,
        })
    }
    fn temporary(
        body: &Body,
        members: &mut Vec<(u32, Reverse<u64>, PathId)>,
        temporary: &Temporary,
        path: PathId,
    ) {
        if let Lifetime::Scope(scope) = temporary.lifetime
            && body
                .promotions
                .as_ref()
                .is_none_or(|p| p.get(temporary.value).is_none())
        {
            members.push((scope.0, Reverse(u64::from(temporary.before) * 2), path));
        }
    }
    fn members(&self, scope: scopes::Id) -> &[PathId] {
        &self.members[self.starts[scope.0 as usize]..self.starts[scope.0 as usize + 1]]
    }
}

pub(super) struct Flow<'a> {
    pub body: &'a Body,
    pub storage: &'a Analysis,
    pub owners: &'a Owners,
    pub consumption: &'a Effects,
}
impl Flow<'_> {
    pub(super) fn cursor(&self, block: usize) -> Cursor<'_> {
        Cursor {
            flow: self,
            events: self.storage.events(block).iter().peekable(),
        }
    }
    pub(super) fn edge<I: Maybe>(
        &self,
        paths: &Paths,
        state: &mut State<I>,
        location: Location,
        mut visitor: impl FnMut(Transition, &mut State<I>) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let Location::Edge { block, target } = location else {
            return Err("cleanup edge requires a successor");
        };
        for end in self.storage.edge_ends(block.0 as usize, target) {
            self.end(paths, state, location, end, &mut visitor)?;
        }
        Ok(())
    }
    fn end<I: Maybe>(
        &self,
        paths: &Paths,
        state: &mut State<I>,
        location: Location,
        end: End,
        mut visitor: impl FnMut(Transition, &mut State<I>) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        match end {
            End::Scopes(range) => {
                let scopes = &self
                    .body
                    .storage
                    .as_ref()
                    .ok_or("missing cleanup scopes")?
                    .scopes;
                let mut next = Some(range.inner);
                while next != range.outer {
                    state.charge(1)?;
                    let scope = next.ok_or("invalid cleanup ancestry")?;
                    for &path in self.owners.members(scope) {
                        clear(
                            paths,
                            state,
                            location,
                            Cause::Scope(scope),
                            path,
                            &mut visitor,
                        )?;
                    }
                    next = scopes[scope.0 as usize].parent;
                    visitor(
                        Transition::StorageEnd {
                            location,
                            end: End::Scopes(Range {
                                inner: scope,
                                outer: next,
                            }),
                        },
                        state,
                    )?;
                }
            }
            End::Temporaries { start, end } => {
                for index in (start..end).rev() {
                    let temporary = &self.body.temporaries[index as usize];
                    state.charge(1)?;
                    if self.storage.temporary(temporary.value).is_some()
                        && let Some(path) = paths.temporary_path(temporary.value)
                    {
                        clear(
                            paths,
                            state,
                            location,
                            Cause::Initializer,
                            path,
                            &mut visitor,
                        )?;
                    }
                    visitor(
                        Transition::StorageEnd {
                            location,
                            end: End::Temporaries {
                                start: index,
                                end: index + 1,
                            },
                        },
                        state,
                    )?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn statement<I: Maybe>(
        &self,
        paths: &Paths,
        state: &mut State<I>,
        location: Location,
        inspect: bool,
        mut visitor: impl FnMut(Transition, &mut State<I>) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let Location::At { before, .. } = location else {
            return Err("cleanup statement requires a point");
        };
        let mut local = |id: LocalId| {
            if !matches!(self.body.locals[id.0 as usize].kind, LocalKind::Guard(_)) {
                clear(
                    paths,
                    state,
                    location,
                    Cause::Storage,
                    PathId(id.0),
                    &mut visitor,
                )?;
            }
            Ok(())
        };
        match self.body.statements[before as usize].action {
            Action::StorageDead(id) => local(id)?,
            Action::StorageEnd { first, last } => {
                for id in self.body.storage_range(first, last) {
                    local(id?)?;
                }
            }
            Action::Store { place, .. } => {
                let info = paths.place(place)?;
                let path = if info.access == Access::Owned && !info.indexed {
                    info.path
                } else {
                    None
                };
                state.charge(1)?;
                visitor(
                    Transition::Observe(Event::Cleanup(Candidate {
                        location,
                        cause: Cause::Replacement,
                        place,
                        path,
                    })),
                    state,
                )?;
            }
            _ => {}
        }
        for effect in self.consumption.at(location) {
            if effect.moment != Moment::Statement {
                continue;
            }
            state.charge(1)?;
            if effect.disposition == Disposition::Discard {
                visitor(
                    Transition::Observe(Event::Cleanup(Candidate {
                        location,
                        cause: Cause::Discard,
                        place: Place::Temporary(effect.value),
                        path: Some(effect.path),
                    })),
                    state,
                )?;
            }
            effect.transfer(paths, state, inspect, &mut visitor)?;
        }
        Ok(())
    }
}

pub(super) struct Cursor<'a> {
    flow: &'a Flow<'a>,
    events: Peekable<slice::Iter<'a, StorageEvent>>,
}
impl Cursor<'_> {
    pub(super) fn point<I: Maybe>(
        &mut self,
        paths: &Paths,
        state: &mut State<I>,
        location: Location,
        inspect: bool,
        mut visitor: impl FnMut(Transition, &mut State<I>) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let Location::At { before, .. } = location else {
            return Err("cleanup cursor requires a point");
        };
        for effect in self.flow.consumption.at(location) {
            let Moment::Return(order) = effect.moment else {
                break;
            };
            while self
                .events
                .peek()
                .is_some_and(|e| e.position == before && e.order <= u64::from(order) * 2)
            {
                self.event(paths, state, location, &mut visitor)?;
            }
            state.charge(1)?;
            effect.transfer(paths, state, inspect, &mut visitor)?;
        }
        while self.events.peek().is_some_and(|e| e.position == before) {
            self.event(paths, state, location, &mut visitor)?;
        }
        Ok(())
    }
    fn event<I: Maybe>(
        &mut self,
        paths: &Paths,
        state: &mut State<I>,
        location: Location,
        mut visitor: impl FnMut(Transition, &mut State<I>) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        state.charge(1)?;
        let event = self.events.next().ok_or("missing cleanup storage event")?;
        match event.kind {
            EventKind::End(end) => self.flow.end(paths, state, location, end, visitor),
            EventKind::Birth(index) => {
                let value = self.flow.body.temporaries[index as usize].value;
                visitor(Transition::Birth { location, value }, state)?;
                // Materializing a borrowed place can follow another scope's end
                // at this same point. An already-owned value must never be revived.
                if self
                    .flow
                    .body
                    .owned
                    .as_ref()
                    .is_none_or(|o| o.creation(value).is_none())
                    && let Some(path) = paths.temporary_path(value)
                {
                    state::change(
                        state,
                        location,
                        &paths.entries[path.0 as usize],
                        true,
                        &mut visitor,
                    )?;
                }
                Ok(())
            }
        }
    }
}
fn clear<I: Maybe>(
    paths: &Paths,
    state: &mut State<I>,
    location: Location,
    cause: Cause,
    path: PathId,
    mut visitor: impl FnMut(Transition, &mut State<I>) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    state.charge(1)?;
    let entry = &paths.entries[path.0 as usize];
    let place = match entry.root {
        Root::Local(id) => Place::Local(id),
        Root::Temporary(id) => Place::Temporary(id),
    };
    visitor(
        Transition::Observe(Event::Cleanup(Candidate {
            location,
            cause,
            place,
            path: Some(path),
        })),
        state,
    )?;
    state::change(state, location, entry, false, &mut visitor)?;
    if matches!(entry.root, Root::Local(_)) {
        state.written(entry.start, false);
    }
    Ok(())
}
