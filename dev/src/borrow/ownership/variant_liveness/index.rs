//! One prepaid eligibility partition drives the existing initialization machine.
use super::{
    parts::Range,
    tags::{Coordinate, Tags},
    trans::Transfers,
};
use crate::{
    body::{
        drops::{MemberPosition, MemberQuery},
        ir::Body,
        storage::Location,
    },
    borrow::ownership::{
        Paths,
        initialization::{Maybe, Transition},
        state::State,
    },
    support::{hash::HashMap, retained::Growth},
    types::{ItemId, Store, TypeId, aggregate::Definitions},
};
#[derive(Clone, Copy)]
pub(super) struct Inputs<'a> {
    pub types: &'a Store,
    pub definitions: &'a Definitions,
    pub paths: &'a Paths,
    pub body: &'a Body,
    pub limit: usize,
}
struct Release {
    clear: Option<Range>,
    parent: Option<Coordinate>,
}
pub(super) struct Partition {
    pub tags: Tags,
    pub transfers: Transfers,
    releases: HashMap<(u32, u32), Release>,
}
impl Partition {
    pub(super) fn new(
        input: Inputs<'_>,
        custom: &dyn Fn(ItemId) -> bool,
        position: &impl Fn(TypeId, MemberQuery) -> Result<MemberPosition, &'static str>,
    ) -> Result<Self, &'static str> {
        let mut tags = Tags::new(
            input.types,
            input.definitions,
            input.paths,
            input.limit,
            custom,
        )?;
        tags.locate(input.paths, position)?;
        let transfers = Transfers::new(&mut tags, input.types, input.paths, input.body, position)?;
        let mut releases = HashMap::default();
        for (index, path) in input.paths.entries.iter().enumerate() {
            tags.parts.charge(1)?;
            if path.start == path.end {
                continue;
            }
            let Some(root) = tags
                .origins
                .get(&crate::borrow::ownership::PathId(super::parts::compact(
                    index,
                )?))
                .copied()
            else {
                continue;
            };
            if path.parent.is_none() && tags.roots[root as usize].path.0 as usize != index {
                return Err("cleanup qualifier root changed its actual path");
            }
            let clear = path
                .parent
                .is_none()
                .then_some(tags.roots[root as usize].tags);
            let mut parent = path.parent;
            let mut coordinate = None;
            while let Some(id) = parent {
                tags.parts.charge(1)?;
                if let Some(found) = tags.locations.get(&id) {
                    coordinate = Some(found.coordinate);
                    break;
                }
                parent = input.paths.entries[id.0 as usize].parent;
            }
            tags.parts.pay(Growth::map::<(u32, u32), Release>(
                releases.len(),
                releases.capacity(),
                1,
            ))?;
            releases
                .try_reserve(1)
                .map_err(|_| "cleanup qualifier release allocation failed")?;
            if releases
                .insert(
                    (path.start, path.end),
                    Release {
                        clear,
                        parent: coordinate,
                    },
                )
                .is_some()
            {
                return Err("cleanup qualifier release interval has multiple owners");
            }
        }
        let words = (tags.bits as usize).div_ceil(64);
        let entries = words
            .checked_mul(input.body.blocks.len().saturating_add(2))
            .ok_or("cleanup qualifier block table overflow")?;
        let bytes = entries
            .checked_mul(size_of::<crate::borrow::ownership::state::Word<u64>>())
            .ok_or("cleanup qualifier block table storage overflow")?;
        tags.parts.pay(Growth {
            bytes,
            work: entries,
        })?;
        Ok(Self {
            tags,
            transfers,
            releases,
        })
    }
    pub(super) fn scratch(&self) -> Result<Vec<u8>, &'static str> {
        let mut scratch = Vec::new();
        scratch
            .try_reserve_exact(self.transfers.scratch)
            .map_err(|_| "cleanup qualifier scratch allocation failed")?;
        scratch.resize(self.transfers.scratch, 0);
        Ok(scratch)
    }
    pub(super) fn before<I: Maybe>(
        &self,
        location: Location,
        state: &mut State<I>,
        scratch: &mut [u8],
    ) -> Result<(), &'static str> {
        let Some(row) = self.transfers.rows.get(&location) else {
            return Ok(());
        };
        for copy in &self.transfers.copies[row.copies.start as usize..row.copies.end as usize] {
            for (offset, bit) in (copy.source.start..copy.source.end).enumerate() {
                *scratch
                    .get_mut(copy.scratch + offset)
                    .ok_or("cleanup qualifier scratch changed extent")? =
                    u8::from(state.possible(bit)?);
            }
        }
        Ok(())
    }
    pub(super) fn after<I: Maybe>(
        &self,
        location: Location,
        state: &mut State<I>,
        scratch: &[u8],
    ) -> Result<(), &'static str> {
        let Some(row) = self.transfers.rows.get(&location) else {
            return Ok(());
        };
        if let Some(destination) = row.destination {
            let node = &self.tags.nodes[destination.coordinate.0 as usize];
            let root = &self.tags.roots[node.root as usize];
            if row.storage.is_some() {
                state.set_possible(root.exposed, root.exposed + 1, false)?;
            }
            if row.storage.is_none() && state.possible(root.exposed)? {
                state.set_possible(node.tags.start, node.tags.end, true)?;
            } else {
                if !destination.merged {
                    state.set_possible(node.tags.start, node.tags.end, false)?;
                }
                for fill in &self.transfers.fills[row.fills.start as usize..row.fills.end as usize]
                {
                    state.set_possible(fill.destination.start, fill.destination.end, fill.value)?;
                }
                for copy in
                    &self.transfers.copies[row.copies.start as usize..row.copies.end as usize]
                {
                    for (offset, bit) in (copy.destination.start..copy.destination.end).enumerate()
                    {
                        if *scratch
                            .get(copy.scratch + offset)
                            .ok_or("cleanup qualifier captured scratch changed extent")?
                            != 0
                        {
                            state.set_possible(bit, bit + 1, true)?;
                        }
                    }
                }
            }
        }
        if let Some(root) = row.expose {
            let root = &self.tags.roots[root as usize];
            state.set_possible(root.exposed, root.exposed + 1, true)?;
            state.set_possible(root.tags.start, root.tags.end, true)?;
        }
        Ok(())
    }
    pub(super) fn transition<I: Maybe>(
        &self,
        transition: Transition,
        state: &mut State<I>,
    ) -> Result<(), &'static str> {
        let Transition::Initialize {
            start,
            end,
            initialized: false,
            ..
        } = transition
        else {
            return Ok(());
        };
        state.charge(1)?;
        let Some(release) = self.releases.get(&(start, end)) else {
            return Ok(());
        };
        if let Some(range) = release.clear {
            state.set_possible(range.start, range.end, false)?;
        }
        let mut parent = release.parent;
        while let Some(id) = parent {
            state.charge(1)?;
            let node = &self.tags.nodes[id.0 as usize];
            state.set_possible(node.partial.start, node.partial.end, true)?;
            parent = node.parent;
        }
        Ok(())
    }
}

impl Partition {
    pub(super) fn edge<I: Maybe>(
        &self,
        path: crate::borrow::ownership::PathId,
        variant: u32,
        yes: bool,
        state: &mut State<I>,
    ) -> Result<(), &'static str> {
        let Some(located) = self.tags.locations.get(&path).filter(|p| !p.merged) else {
            return Ok(());
        };
        let node = &self.tags.nodes[located.coordinate.0 as usize];
        let own = node.own_tags;
        let selected = own
            .start
            .checked_add(variant)
            .filter(|&bit| bit < own.end)
            .ok_or("cleanup qualifier edge lost its actual variant")?;
        for bit in own.start..own.end {
            if (bit == selected) != yes {
                state.set_possible(bit, bit + 1, false)?;
            }
        }
        let mut possible = false;
        for bit in own.start..own.end {
            possible |= state.possible(bit)?;
        }
        if !possible {
            return state.set_possible(node.tags.start, node.tags.end, false);
        }
        for child in &self.tags.children[node.children.start as usize..node.children.end as usize] {
            let child = &self.tags.nodes[child.coordinate.0 as usize];
            if let super::tags::Step::Nominal {
                variant: actual, ..
            } = child.step
                && (actual == variant) != yes
            {
                state.set_possible(child.tags.start, child.tags.end, false)?;
            }
        }
        Ok(())
    }
}
