//! Physical carrier facts retain pointer and integer validity as distinct encodings.
use crate::types::{Kind, Primitive, RangeId, Store, TypeId};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Carrier {
    NonNullPointer {
        leaf: TypeId,
    },
    /// An integer of either signedness whose one excluded bit pattern is zero.
    ZeroExcludedInteger {
        leaf: TypeId,
        primitive: Primitive,
    },
    Boolean {
        leaf: TypeId,
    },
}
impl Carrier {
    #[must_use]
    pub const fn leaf(self) -> TypeId {
        match self {
            Self::NonNullPointer { leaf }
            | Self::ZeroExcludedInteger { leaf, .. }
            | Self::Boolean { leaf } => leaf,
        }
    }
    #[must_use]
    pub fn bytes(self) -> u64 {
        match self {
            Self::NonNullPointer { .. } => 8,
            Self::Boolean { .. } => 1,
            Self::ZeroExcludedInteger { primitive, .. } => primitive
                .integer()
                .map_or(0, |(_, bits)| u64::from(bits / 8)),
        }
    }
    #[must_use]
    pub const fn is_pointer(self) -> bool {
        matches!(self, Self::NonNullPointer { .. })
    }
    /// The unused carrier value that encodes the companion variant.
    #[must_use]
    pub const fn empty_bits(self) -> u128 {
        match self {
            Self::Boolean { .. } => 2,
            Self::NonNullPointer { .. } | Self::ZeroExcludedInteger { .. } => 0,
        }
    }
}
impl Carrier {
    /// The carrier of a range that excludes only zero. An unsigned base spells
    /// it `1..`; a signed base spells it `..0 | 1..`, which is stored as the same
    /// wrapped interval of raw bits, one through the all-ones mask.
    pub(super) fn integer(
        types: &Store,
        leaf: TypeId,
        base: TypeId,
        id: RangeId,
        bytes: u64,
    ) -> Result<Option<Self>, &'static str> {
        let range = types
            .scalar_range(id)
            .ok_or("missing scalar carrier range")?;
        if range.base != base {
            return Err("scalar carrier range belongs to a different base");
        }
        let Some(Kind::Primitive(primitive)) = types.kind(base) else {
            return Err("scalar carrier requires its primitive base");
        };
        let Some((_, bits)) = primitive.integer() else {
            return Ok(None);
        };
        let mask = u128::MAX >> (128 - bits);
        if range.start != 1 || range.end != mask || bytes != u64::from(bits / 8) {
            return Ok(None);
        }
        Ok(Some(Self::ZeroExcludedInteger { leaf, primitive }))
    }
}

/// A valid but wider scalar range is distinct from proven physical ineligibility.
#[derive(Clone, Copy)]
pub(super) enum Answer {
    Absent,
    Carrier(Carrier),
    WiderScalar,
}
impl Answer {
    pub(super) const fn carrier(self) -> Option<Carrier> {
        match self {
            Self::Carrier(carrier) => Some(carrier),
            Self::Absent | Self::WiderScalar => None,
        }
    }
}
