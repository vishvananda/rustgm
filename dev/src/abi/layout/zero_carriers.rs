//! Physical carrier facts retain pointer and integer validity as distinct encodings.
use crate::types::{Kind, Primitive, RangeId, Store, TypeId};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Carrier {
    NonNullPointer { leaf: TypeId },
    ZeroExcludedUnsigned { leaf: TypeId, primitive: Primitive },
}
impl Carrier {
    #[must_use]
    pub const fn leaf(self) -> TypeId {
        match self {
            Self::NonNullPointer { leaf } | Self::ZeroExcludedUnsigned { leaf, .. } => leaf,
        }
    }
    #[must_use]
    pub fn bytes(self) -> u64 {
        match self {
            Self::NonNullPointer { .. } => 8,
            Self::ZeroExcludedUnsigned { primitive, .. } => match primitive.integer() {
                Some((false, bits)) => u64::from(bits / 8),
                _ => 0,
            },
        }
    }
    #[must_use]
    pub const fn is_pointer(self) -> bool {
        matches!(self, Self::NonNullPointer { .. })
    }
}
impl Carrier {
    pub(super) fn unsigned(
        types: &Store,
        leaf: TypeId,
        base: TypeId,
        id: RangeId,
        bytes: u64,
    ) -> Result<Option<Self>, &'static str> {
        let range = types
            .scalar_range(id)
            .ok_or("missing scalar carrier range")?;
        let Some(Kind::Primitive(primitive)) = types.kind(base) else {
            return Err("scalar carrier requires its primitive base");
        };
        let Some((false, bits)) = primitive.integer() else {
            return Ok(None);
        };
        let mask = u128::MAX >> (128 - bits);
        if range.base != base {
            return Err("scalar carrier range belongs to a different base");
        }
        if range.start != 1 || range.end != mask || bytes != u64::from(bits / 8) {
            return Ok(None);
        }
        Ok(Some(Self::ZeroExcludedUnsigned { leaf, primitive }))
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
