//! Paired binder slots form a bijection; unused declared slots need no partner.
use super::super::{Error, RegionId, signature_slots::Owner};
use crate::types::closure::ClosureId;
use std::collections::HashMap;

struct Frame {
    scope: usize,
    counts: (u32, u32),
    rigid: bool,
    forward: HashMap<u16, u16>,
    reverse: HashMap<u16, u16>,
    owners: (Option<Owner>, Option<Owner>),
}

#[derive(Default)]
pub(super) struct Binders {
    frames: Vec<Frame>,
    next: usize,
    owners: HashMap<(ClosureId, bool), Vec<usize>>,
}

enum Region {
    Local(usize, u16),
    Outer(RegionId),
}

impl Binders {
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
    pub fn depth(&self) -> Result<u16, Error> {
        u16::try_from(self.frames.len()).map_err(|_| Error::Overflow)
    }
    pub fn scope(&self) -> usize {
        self.frames.last().map_or(0, |frame| frame.scope)
    }
    pub fn enter(&mut self, left: u32, right: u32) -> Result<(), Error> {
        self.push(left, right, false)
    }
    pub fn enter_owned(
        &mut self,
        left: u32,
        right: u32,
        owners: (Option<Owner>, Option<Owner>),
    ) -> Result<(), Error> {
        self.push(left, right, false)?;
        let index = self.frames.len() - 1;
        self.frames[index].owners = owners;
        for (owner, right) in [(owners.0, false), (owners.1, true)] {
            if let Some(owner) = owner.filter(|owner| owner.closed) {
                self.owners
                    .entry((owner.origin, right))
                    .or_default()
                    .push(index);
            }
        }
        Ok(())
    }
    pub fn owner_depth(&self, owner: Owner, right: bool) -> Result<Option<u16>, Error> {
        self.owners
            .get(&(owner.origin, right))
            .and_then(|stack| stack.last())
            .map(|index| u16::try_from(index + 1).map_err(|_| Error::Overflow))
            .transpose()
    }
    pub fn enter_rigid(&mut self, count: u32) -> Result<(), Error> {
        self.push(count, count, true)
    }
    fn push(&mut self, left: u32, right: u32, rigid: bool) -> Result<(), Error> {
        if self.frames.len() >= 0x7fff {
            return Err(Error::Overflow);
        }
        self.next = self.next.checked_add(1).ok_or(Error::Overflow)?;
        self.frames.push(Frame {
            scope: self.next,
            counts: (left, right),
            rigid,
            forward: HashMap::new(),
            reverse: HashMap::new(),
            owners: (None, None),
        });
        Ok(())
    }
    pub fn exit(&mut self) -> Result<(), Error> {
        let frame = self.frames.pop().ok_or(Error::InvalidIdentity)?;
        for (owner, right) in [(frame.owners.0, false), (frame.owners.1, true)] {
            if let Some(owner) = owner.filter(|owner| owner.closed) {
                self.owners
                    .get_mut(&(owner.origin, right))
                    .ok_or(Error::InvalidIdentity)?
                    .pop()
                    .ok_or(Error::InvalidIdentity)?;
            }
        }
        Ok(())
    }
    fn region(&self, region: RegionId) -> Result<Region, Error> {
        let Some((depth, slot)) = region.bound_parts() else {
            return Ok(Region::Outer(region));
        };
        let count = self.depth()?;
        Ok(if depth < count {
            Region::Local(usize::from(count - depth - 1), slot)
        } else {
            Region::Outer(RegionId::bound(depth - count, slot).map_err(|_| Error::Overflow)?)
        })
    }
    pub fn equate(
        &mut self,
        left: RegionId,
        right: RegionId,
        outer: &mut Vec<(RegionId, RegionId)>,
    ) -> Result<(), Error> {
        match (self.region(left)?, self.region(right)?) {
            (Region::Outer(a), Region::Outer(b)) => {
                if a != b {
                    outer.push((a, b));
                }
            }
            (Region::Local(a, x), Region::Local(b, y)) if a == b => {
                let frame = &mut self.frames[a];
                if u32::from(x) >= frame.counts.0 || u32::from(y) >= frame.counts.1 {
                    return Err(Error::InvalidIdentity);
                }
                if frame.rigid && x != y {
                    return Err(Error::Mismatch);
                }
                if frame.forward.get(&x).is_some_and(|&slot| slot != y)
                    || frame.reverse.get(&y).is_some_and(|&slot| slot != x)
                {
                    return Err(Error::Mismatch);
                }
                frame.forward.insert(x, y);
                frame.reverse.insert(y, x);
            }
            _ => return Err(Error::Mismatch),
        }
        Ok(())
    }
}
