//! Existential body regions, numbered densely within one numbering unit.
//!
//! A unit is an item, initializer or constant body together with every nested
//! closure and inline body that shares its inference graph. Identities
//! `1 + ordinal` restart in each unit, so identical bodies open identical
//! canonical types and type-keyed memos are shared between them. Units never
//! exchange body regions: each fact naming a region also names its body, and
//! every consumer interprets the identity inside that body's unit. The
//! crate-wide log keeps allocation order for contract collection and views.
//!
//! Every allocation, frontier and identity query names its body; the active
//! unit only caches the most recent one. Disposable probes that never publish
//! their identities start beyond every unit, so they cannot capture a caller
//! region whatever body requested them.
use super::model::BodyRegion;
use crate::resolve::unit::NodeKey;
use crate::support::hash::HashMap;
use crate::types::RegionId;

/// One allocated region with its unit-local identity.
#[derive(Clone, Copy)]
pub struct AllocatedRegion {
    pub id: RegionId,
    pub region: BodyRegion,
}

/// Log positions of one unit's identities, by ordinal. A unit usually
/// allocates in one uninterrupted run.
#[derive(Default)]
enum Positions {
    #[default]
    Empty,
    Run {
        start: u32,
        len: u32,
    },
    List(Vec<u32>),
}
impl Positions {
    fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Run { len, .. } => *len as usize,
            Self::List(positions) => positions.len(),
        }
    }
    fn get(&self, index: usize) -> Option<u32> {
        match self {
            Self::Empty => None,
            Self::Run { start, len } => {
                let index = u32::try_from(index).ok().filter(|index| index < len)?;
                Some(start + index)
            }
            Self::List(positions) => positions.get(index).copied(),
        }
    }
    fn push(&mut self, position: u32) {
        match self {
            Self::Empty => {
                *self = Self::Run {
                    start: position,
                    len: 1,
                };
            }
            Self::Run { start, len } if *start + *len == position => *len += 1,
            Self::Run { start, len } => {
                let mut positions: Vec<u32> = (*start..*start + *len).collect();
                positions.push(position);
                *self = Self::List(positions);
            }
            Self::List(positions) => positions.push(position),
        }
    }
}

#[derive(Default)]
pub struct BodyRegions {
    log: Vec<AllocatedRegion>,
    active: Positions,
    unit: Option<NodeKey>,
    suspended: HashMap<NodeKey, Positions>,
    /// Unit links. Insertion uses the current root; later empty-unit nesting
    /// retains that old root as an alias, so queries must still follow links.
    parents: HashMap<NodeKey, NodeKey>,
    /// Most identities allocated in any unit.
    widest: usize,
}
impl BodyRegions {
    /// The next identity of `body`'s unit.
    /// # Errors
    /// Rejects an exhausted identity space.
    pub fn frontier(&self, body: NodeKey) -> Result<u32, &'static str> {
        next_identity(self.positions(body).map_or(0, Positions::len))
    }
    /// A first identity beyond every unit, for disposable probes whose regions
    /// are never published into a body.
    /// # Errors
    /// Rejects an exhausted identity space.
    pub fn probe_frontier(&self) -> Result<u32, &'static str> {
        next_identity(self.widest)
    }
    /// `body`'s region with identity `id`.
    #[must_use]
    pub fn get(&self, body: NodeKey, id: RegionId) -> Option<&BodyRegion> {
        let position = self.position(body, id)?;
        self.log.get(position as usize).map(|entry| &entry.region)
    }
    /// `body`'s region with identity `id`.
    pub fn get_mut(&mut self, body: NodeKey, id: RegionId) -> Option<&mut BodyRegion> {
        let position = self.position(body, id)?;
        self.log
            .get_mut(position as usize)
            .map(|entry| &mut entry.region)
    }
    /// `body`'s unit regions from identity `first + 1` on, in identity order.
    pub fn regions_after(
        &self,
        body: NodeKey,
        first: usize,
    ) -> impl Iterator<Item = (RegionId, &BodyRegion)> + '_ {
        let positions = self.positions(body);
        let count = positions.map_or(0, Positions::len);
        (first..count)
            .filter_map(move |index| positions?.get(index))
            .filter_map(|position| self.log.get(position as usize))
            .map(|entry| (entry.id, &entry.region))
    }
    /// Every allocated region in allocation order, with its unit-local identity.
    #[must_use]
    pub fn log(&self) -> &[AllocatedRegion] {
        &self.log
    }
    /// Allocate the next identity of the region's unit.
    /// # Errors
    /// Rejects an exhausted identity space.
    pub fn allocate(&mut self, region: BodyRegion) -> Result<RegionId, &'static str> {
        self.activate(region.body);
        self.push(region)
    }
    /// Allocate `count` identities from an interval reserved at `first`.
    /// # Errors
    /// Rejects a moved frontier and an exhausted identity space.
    pub fn allocate_reserved(
        &mut self,
        region: BodyRegion,
        first: u32,
        count: usize,
    ) -> Result<(), &'static str> {
        if count == 0 {
            return Ok(());
        }
        self.activate(region.body);
        if next_identity(self.active.len())? != first {
            return Err("body region allocation frontier changed");
        }
        for _ in 0..count {
            self.push(region)?;
        }
        Ok(())
    }
    /// Record that `body` shares the identities of `parent`'s unit.
    /// # Errors
    /// Rejects a body that already belongs to, or allocated in, another unit.
    pub fn nest(&mut self, body: NodeKey, parent: NodeKey) -> Result<(), &'static str> {
        let unit = self.unit_of(parent);
        if body == unit {
            return Ok(());
        }
        if let Some(&current) = self.parents.get(&body) {
            if self.unit_of(current) == unit {
                return Ok(());
            }
            return Err("nested body changed region numbering unit");
        }
        if self
            .positions(body)
            .is_some_and(|positions| positions.len() != 0)
        {
            return Err("nested body already numbered its own regions");
        }
        self.parents.insert(body, unit);
        // The check above proves this cached unit has no identities. Its old
        // root now aliases `unit`, so even read-only queries must resolve again.
        if self.unit == Some(body) {
            self.unit = None;
        }
        Ok(())
    }
    /// The unit whose identities `body` uses.
    #[must_use]
    pub fn unit_of(&self, mut body: NodeKey) -> NodeKey {
        if self.parents.is_empty() {
            return body;
        }
        while let Some(&parent) = self.parents.get(&body) {
            #[cfg(test)]
            tests::followed_parent();
            body = parent;
        }
        body
    }
    /// Cache `body`'s unit for the queries of a starting body session.
    pub fn activate(&mut self, body: NodeKey) {
        if self.unit == Some(body) {
            return;
        }
        let unit = self.unit_of(body);
        if self.unit == Some(unit) {
            return;
        }
        let active = std::mem::take(&mut self.active);
        if let Some(previous) = self.unit
            && active.len() != 0
        {
            self.suspended.insert(previous, active);
        }
        self.active = self.suspended.remove(&unit).unwrap_or_default();
        self.unit = Some(unit);
    }
    fn positions(&self, body: NodeKey) -> Option<&Positions> {
        if self.unit == Some(body) {
            return Some(&self.active);
        }
        let unit = self.unit_of(body);
        if self.unit == Some(unit) {
            Some(&self.active)
        } else {
            self.suspended.get(&unit)
        }
    }
    fn position(&self, body: NodeKey, id: RegionId) -> Option<u32> {
        let index = (id.0 as usize).checked_sub(1)?;
        self.positions(body)?.get(index)
    }
    fn push(&mut self, region: BodyRegion) -> Result<RegionId, &'static str> {
        let id = RegionId(next_identity(self.active.len())?);
        let position =
            u32::try_from(self.log.len()).map_err(|_| "body region log exceeds 32 bits")?;
        self.log.push(AllocatedRegion { id, region });
        self.active.push(position);
        self.widest = self.widest.max(self.active.len());
        Ok(id)
    }
}
fn next_identity(count: usize) -> Result<u32, &'static str> {
    u32::try_from(count)
        .ok()
        .and_then(|n| n.checked_add(1))
        .filter(|n| *n < 0x8000_0000)
        .ok_or("body region identity space exhausted")
}

#[cfg(test)]
#[path = "../../tests/invariants/numbering_units.rs"]
mod tests;
