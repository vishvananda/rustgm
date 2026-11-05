//! Abstract argument lifetimes into one instance's own proof context.
use super::{Action, Bound, RegionId};
use crate::support::hash::HashMap;

pub(super) struct Mapping {
    first: u32,
    locals_only: bool,
    replacements: HashMap<RegionId, RegionId>,
}
impl Mapping {
    pub(super) fn region_observed<const CHARGED: bool>(
        &mut self,
        original: RegionId,
        depth: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<RegionId, &'static str> {
        if original == RegionId::STATIC {
            return Ok(original);
        }
        if self.locals_only
            && original
                .bound_parts()
                .is_none_or(|(binder, _)| binder != depth)
        {
            return Ok(original);
        }
        if self.locals_only
            && original
                .bound_parts()
                .is_some_and(|(_, slot)| u32::from(slot) < self.first)
        {
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
            if CHARGED {
                let growth = crate::support::retained::Growth::map::<RegionId, RegionId>(
                    self.replacements.len(),
                    self.replacements.capacity(),
                    1,
                );
                charge(growth.work, growth.bytes)?;
            }
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
            locals_only: false,
            replacements: HashMap::default(),
        }));
        bound
    }

    /// Retain the same input mapping under the actual source allocation payer.
    pub(crate) fn universals_charged(
        first: u32,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        charge(1, size_of::<Self>() + size_of::<Mapping>())?;
        Ok(Self::universals(first))
    }

    /// Renumber appended locals while preserving a parent binder's fixed prefix.
    /// Captures and nested locals stay rigid; the result is not a proof clause.
    pub(crate) fn canonical_locals_preserving(first: u32) -> Self {
        let mut bound = Self::universals(first);
        if let Action::Universals(mapping) = &mut bound.action {
            mapping.locals_only = true;
        }
        bound
    }
    /// Inspect the universal already assigned to a free argument lifetime.
    /// Static and unused lifetimes have no imported universal. This does not
    /// allocate lifetimes or alter nested callable binders.
    #[must_use]
    pub fn universal_image(&self, original: RegionId) -> Option<RegionId> {
        let Action::Universals(mapping) = &self.action else {
            return None;
        };
        mapping.replacements.get(&original).copied()
    }

    /// Source lifetimes and the universal slots assigned by this abstraction.
    /// The order is unspecified; callers must preserve the assigned slot numbers.
    pub fn universal_pairs(&self) -> impl Iterator<Item = (RegionId, RegionId)> + '_ {
        let replacements = match &self.action {
            Action::Universals(mapping) => Some(&mapping.replacements),
            _ => None,
        };
        replacements.into_iter().flatten().map(|(&a, &b)| (a, b))
    }

    #[must_use]
    pub fn universal_count(&self) -> usize {
        match &self.action {
            Action::Universals(mapping) => mapping.replacements.len(),
            _ => 0,
        }
    }
}

impl Mapping {
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.map(&self.replacements);
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn paid_universal_import_keeps_warm_identity_and_rejects_unpaid_growth()
    -> Result<(), &'static str> {
        let bytes = Cell::new(0usize);
        let limit = Cell::new(usize::MAX);
        let pay = |_: usize, growth: usize| {
            if bytes.get().saturating_add(growth) > limit.get() {
                return Err("actual input mapping budget");
            }
            bytes.set(bytes.get().saturating_add(growth));
            Ok(())
        };
        let mut fold = Bound::universals_charged(3, &pay)?;
        let initial = bytes.get();
        limit.set(initial);
        assert!(fold.region_observed::<true>(RegionId(42), 0, &pay).is_err());
        assert_eq!(fold.universal_count(), 0);
        limit.set(usize::MAX);
        let first = fold.region_observed::<true>(RegionId(42), 0, &pay)?;
        assert_eq!(first, RegionId::bound(0, 3)?);
        let cold = bytes.get();
        assert!(cold > initial);
        assert_eq!(fold.region_observed::<true>(RegionId(42), 0, &pay)?, first);
        assert_eq!(bytes.get(), cold);
        assert_eq!(
            fold.region_observed::<true>(RegionId::bound(0, 0)?, 1, &pay)?,
            RegionId::bound(0, 0)?
        );
        assert_eq!(
            fold.region_observed::<true>(RegionId::STATIC, 0, &pay)?,
            RegionId::STATIC
        );
        assert_eq!(fold.universal_count(), 1);
        assert!(Bound::universals_charged(0, &|_, _| Err("reject before construction")).is_err());
        Ok(())
    }
}
