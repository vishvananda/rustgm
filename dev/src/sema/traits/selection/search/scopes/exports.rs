//! Access to a free lifetime follows proof ownership, including suspended children.
use super::{Query, RegionId, ScopeId, Scopes};
use std::ops::Range;

impl Scopes {
    pub(in crate::sema::traits::selection::search) fn export_source(
        &self,
        parent: ScopeId,
        from: ScopeId,
        region: RegionId,
        first: u32,
        work: &mut usize,
    ) -> Result<bool, &'static str> {
        if region.0 >= first {
            return Err("scoped witness lies beyond the allocated frontier");
        }
        let index = self.reservations.partition_point(|reservation| {
            *work += 1;
            reservation.regions.end <= region.0
        });
        let Some(found) = self
            .reservations
            .get(index)
            .filter(|row| row.regions.contains(&region.0))
        else {
            // Root and caller allocations do not have a local reservation.
            return Ok(false);
        };
        if self.descendant(parent, found.scope, work)? {
            return Ok(false);
        }
        if !found.alias && self.descendant(found.scope, from, work)? {
            return Ok(true);
        }
        Err("witness export contains a universal or unowned lifetime")
    }

    fn descendant(
        &self,
        mut scope: ScopeId,
        ancestor: ScopeId,
        work: &mut usize,
    ) -> Result<bool, &'static str> {
        while scope != ancestor && scope != ScopeId::ROOT {
            *work += 1;
            scope = self
                .universes
                .get(scope.0 as usize - 1)
                .ok_or("invalid witness proof scope")?
                .parent;
        }
        Ok(scope == ancestor)
    }
}

impl Query {
    /// These regions belong to the selected scope's continuing inputs.
    /// A single proof node need not own the allocation.
    pub(in crate::sema::traits::selection::search) fn retain_scoped_regions(
        &mut self,
        owner: ScopeId,
        regions: Range<u32>,
    ) -> Result<(), &'static str> {
        if regions.is_empty() || owner == ScopeId::ROOT {
            return Ok(());
        }
        self.reserve(owner, regions.clone())?;
        let scope = self
            .scopes
            .as_mut()
            .ok_or("missing scoped lifetime allocations")?;
        let reservation = scope
            .reservations
            .last_mut()
            .ok_or("missing retained scoped region reservation")?;
        if reservation.scope != owner || reservation.regions != regions {
            return Err("retained scoped region reservation changed ownership");
        }
        reservation.retained_with_scope = true;
        Ok(())
    }
}
