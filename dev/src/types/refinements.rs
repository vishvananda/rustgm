//! Closed scalar refinements keep target bits separate from logical type proof.
use super::{Constraint, Kind, Primitive, Store, TypeId, arguments::Constant};
use crate::support::intern_slice::Slices;
#[cfg(test)]
#[path = "../../tests/invariants/closed_intervals.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RangeId(u32);

/// Inclusive target-width endpoints. Signed intervals can wrap in raw bit order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScalarRange {
    pub base: TypeId,
    pub start: u128,
    pub end: u128,
}
impl Store {
    /// # Errors
    /// Rejects unknown/non-scalar bases, invalid endpoints, reversed source
    /// order, exhausted payment and compact arena overflow before publication.
    pub(crate) fn intern_scalar_range(
        &mut self,
        range: ScalarRange,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<RangeId, &'static str> {
        charge(5, 0)?;
        self.require_scalar_range(range)?;
        self.intern_canonical_scalar_range(range, charge)
    }
    /// Reinterns an already canonical interval; surface order belongs to its producer.
    pub(crate) fn intern_canonical_scalar_range(
        &mut self,
        range: ScalarRange,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<RangeId, &'static str> {
        charge(5, 0)?;
        self.require_scalar_endpoints(range)?;
        let probe = std::cell::Cell::new(0_usize);
        if let Some(id) = self.find_scalar_range(range, &|steps, bytes| {
            probe.set(probe.get().saturating_add(steps));
            charge(steps, bytes)
        })? {
            return Ok(id);
        }
        // The miss probe is repeated by intern; prepay its fixed-value hash and
        // all actual collision comparisons using the same immutable table.
        let storage = self.arguments.as_ref().and_then(|s| s.ranges.as_deref());
        let mut growth = storage.map_or_else(
            || Slices::<ScalarRange>::default().interning_growth(1),
            |storage| storage.interning_growth(1),
        );
        if self.arguments.is_none() {
            growth.bytes = growth
                .bytes
                .saturating_add(size_of::<super::arguments::Storage>());
        }
        if storage.is_none() {
            growth.bytes = growth
                .bytes
                .saturating_add(size_of::<Slices<ScalarRange>>());
        }
        charge(
            growth.work.saturating_add(probe.get()).saturating_add(2),
            growth.bytes,
        )?;
        self.arguments
            .get_or_insert_with(Default::default)
            .ranges
            .get_or_insert_with(Default::default)
            .intern(&[range])
            .map(RangeId)
    }
    pub(crate) fn find_scalar_range(
        &self,
        range: ScalarRange,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Option<RangeId>, &'static str> {
        let Some(storage) = self.arguments.as_ref().and_then(|s| s.ranges.as_deref()) else {
            charge(2, 0)?;
            return Ok(None);
        };
        storage
            .find_charged(&[range], |steps| charge(steps, 0))
            .map(|id| id.map(RangeId))
    }
    #[must_use]
    pub fn scalar_range(&self, id: RangeId) -> Option<ScalarRange> {
        let values = self.arguments.as_ref()?.ranges.as_deref()?.get(id.0)?;
        let [range] = values else {
            return None;
        };
        Some(*range)
    }
    pub(crate) fn range_allows(&self, id: RangeId, bits: u128) -> bool {
        self.scalar_range(id).is_some_and(|range| {
            self.valid_constant(Constant {
                ty: range.base,
                bits,
            }) && if range.start <= range.end {
                range.start <= bits && bits <= range.end
            } else {
                bits >= range.start || bits <= range.end
            }
        })
    }
    pub(super) fn valid_refinement(&self, base: TypeId, constraint: Constraint) -> bool {
        match constraint {
            Constraint::NonNull => matches!(self.kind(base), Some(Kind::Pointer { .. })),
            Constraint::ScalarRange(id) => self.scalar_range(id).is_some_and(|r| r.base == base),
        }
    }
    pub(crate) fn intern_scalar_or(
        &mut self,
        first: ScalarRange,
        second: ScalarRange,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<RangeId, &'static str> {
        charge(12, 0)?;
        self.require_scalar_range(first)?;
        self.require_scalar_range(second)?;
        if first.base != second.base {
            return Err("scalar alternatives have different base types");
        }
        let Some(Kind::Primitive(p)) = self.kind(first.base) else {
            return Err("scalar alternative base changed");
        };
        let Some((true, width)) = p.integer() else {
            return Err("scalar alternatives require a signed integer base");
        };
        let bias = 1_u128 << (width - 1);
        let maximum = bias - 1;
        let (first, second) = if first.start == bias {
            (first, second)
        } else if second.start == bias {
            (second, first)
        } else {
            return Err("disconnected scalar alternatives require their wider source profile");
        };
        if (first.end ^ bias) >= (second.start ^ bias) {
            return Err("scalar alternatives overlap");
        }
        if second.end != maximum {
            return Err("scalar alternatives must connect at signed overflow");
        }
        self.intern_canonical_scalar_range(
            ScalarRange {
                base: first.base,
                start: second.start,
                end: first.end,
            },
            charge,
        )
    }
    fn require_scalar_range(&self, range: ScalarRange) -> Result<(), &'static str> {
        self.require_scalar_endpoints(range)?;
        let Some(Kind::Primitive(primitive)) = self.kind(range.base) else {
            return Err("scalar pattern requires a concrete integer or char base");
        };
        let (signed, width) = if primitive == Primitive::Char {
            (false, 32)
        } else {
            primitive
                .integer()
                .ok_or("scalar pattern requires an integer or char base")?
        };
        let bias = if signed { 1_u128 << (width - 1) } else { 0 };
        if (range.start ^ bias) > (range.end ^ bias) {
            return Err("scalar pattern endpoints are reversed in source value order");
        }
        Ok(())
    }
    fn require_scalar_endpoints(&self, range: ScalarRange) -> Result<(), &'static str> {
        if !matches!(self.kind(range.base), Some(Kind::Primitive(p)) if p == Primitive::Char || p.integer().is_some())
        {
            return Err("scalar pattern requires an integer or char base");
        }
        if !self.valid_constant(Constant {
            ty: range.base,
            bits: range.start,
        }) || !self.valid_constant(Constant {
            ty: range.base,
            bits: range.end,
        }) {
            return Err("scalar pattern endpoint is invalid for its actual base");
        }
        Ok(())
    }
}
