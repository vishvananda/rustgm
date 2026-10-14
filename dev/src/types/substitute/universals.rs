//! Abstract argument lifetimes into one instance's own proof context.
use super::{Action, Bound, RegionId};
use std::collections::HashMap;

pub(super) struct Mapping {
    first: u32,
    replacements: HashMap<RegionId, RegionId>,
}
impl Mapping {
    pub(super) fn region(
        &mut self,
        original: RegionId,
        depth: u16,
    ) -> Result<RegionId, &'static str> {
        if original == RegionId::STATIC {
            return Ok(original);
        }
        let free = match original.bound_parts() {
            Some((binder, _)) if binder < depth => return Ok(original),
            Some((binder, index)) => RegionId::bound(binder - depth, index)?,
            None => original,
        };
        let region = if let Some(&region) = self.replacements.get(&free) {
            region
        } else {
            let next = self.first.saturating_add(
                u32::try_from(self.replacements.len())
                    .map_err(|_| "too many instance lifetimes")?,
            );
            let index = u16::try_from(next).map_err(|_| "too many instance lifetimes")?;
            let region = RegionId::bound(0, index)?;
            self.replacements.insert(free, region);
            region
        };
        let (_, index) = region.bound_parts().ok_or("invalid instance lifetime")?;
        RegionId::bound(depth, index)
    }
}
impl Bound {
    /// Give each distinct free argument lifetime its own universal in a fresh
    /// instance context, preserving static and all nested callable binders.
    /// Reuse one folder for all arguments so repeated lifetimes remain equal.
    /// This is abstraction for proof, not runtime lifetime erasure.
    #[must_use]
    pub fn universals(first: u32) -> Self {
        let mut bound = Self::new(Vec::new());
        bound.action = Action::Universals(Box::new(Mapping {
            first,
            replacements: HashMap::new(),
        }));
        bound
    }
    #[must_use]
    pub fn universal_count(&self) -> usize {
        match &self.action {
            Action::Universals(mapping) => mapping.replacements.len(),
            _ => 0,
        }
    }
}
