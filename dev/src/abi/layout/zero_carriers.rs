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
    /// A zero-based integer interval; each enclosing enum consumes one value
    /// from its unused suffix. The original range stays in the type store.
    UpperBoundedInteger {
        leaf: TypeId,
        primitive: Primitive,
        consumed: u32,
    },
    Boolean {
        leaf: TypeId,
        consumed: u8,
    },
}
impl Carrier {
    #[must_use]
    pub const fn leaf(self) -> TypeId {
        match self {
            Self::NonNullPointer { leaf }
            | Self::ZeroExcludedInteger { leaf, .. }
            | Self::UpperBoundedInteger { leaf, .. }
            | Self::Boolean { leaf, .. } => leaf,
        }
    }
    #[must_use]
    pub fn bytes(self) -> u64 {
        match self {
            Self::NonNullPointer { .. } => 8,
            Self::Boolean { .. } => 1,
            Self::ZeroExcludedInteger { primitive, .. }
            | Self::UpperBoundedInteger { primitive, .. } => primitive
                .integer()
                .map_or(0, |(_, bits)| u64::from(bits / 8)),
        }
    }
    #[must_use]
    pub const fn is_pointer(self) -> bool {
        matches!(self, Self::NonNullPointer { .. })
    }
    /// The unused carrier value that encodes the companion variant.
    /// # Errors
    /// Rejects a stale range identity or an exhausted scalar carrier.
    pub fn empty_bits(self, types: &Store) -> Result<u128, &'static str> {
        match self {
            Self::Boolean { consumed, .. } => suffix_niche(1, 255, u128::from(consumed)),
            Self::UpperBoundedInteger {
                leaf,
                primitive,
                consumed,
            } => {
                let Kind::Pattern {
                    base,
                    constraint: crate::types::Constraint::ScalarRange(id),
                } = types.kind(leaf).ok_or("missing bounded carrier type")?
                else {
                    return Err("bounded carrier lacks its scalar range");
                };
                let range = types
                    .scalar_range(id)
                    .ok_or("missing bounded carrier range")?;
                let (_, bits) = primitive
                    .integer()
                    .ok_or("bounded carrier is not an integer")?;
                if base != TypeId::primitive(primitive) || range.base != base || range.start != 0 {
                    return Err("bounded carrier differs from its scalar range");
                }
                suffix_niche(range.end, u128::MAX >> (128 - bits), u128::from(consumed))
            }
            Self::NonNullPointer { .. } | Self::ZeroExcludedInteger { .. } => Ok(0),
        }
    }
    /// The payload's next available invalid value after one enum uses this one.
    pub(super) fn next(self, types: &Store) -> Result<Option<Self>, &'static str> {
        Ok(match self {
            Self::Boolean { leaf, consumed } if consumed < 253 => Some(Self::Boolean {
                leaf,
                consumed: consumed + 1,
            }),
            Self::UpperBoundedInteger {
                leaf,
                primitive,
                consumed,
            } => {
                let (_, bits) = primitive
                    .integer()
                    .ok_or("bounded carrier is not an integer")?;
                let Kind::Pattern {
                    constraint: crate::types::Constraint::ScalarRange(id),
                    ..
                } = types
                    .kind(self.leaf())
                    .ok_or("missing bounded carrier type")?
                else {
                    return Err("bounded carrier lacks its scalar range");
                };
                let range = types
                    .scalar_range(id)
                    .ok_or("missing bounded carrier range")?;
                self.empty_bits(types)?;
                if u128::from(consumed) + 1 == (u128::MAX >> (128 - bits)) - range.end {
                    None
                } else {
                    Some(Self::UpperBoundedInteger {
                        leaf,
                        primitive,
                        consumed: consumed
                            .checked_add(1)
                            .ok_or("bounded carrier count overflow")?,
                    })
                }
            }
            Self::NonNullPointer { .. }
            | Self::ZeroExcludedInteger { .. }
            | Self::Boolean { .. } => None,
        })
    }
}
impl Carrier {
    /// Supports one zero niche or the contiguous unused suffix of a zero-based
    /// integer interval. Other intervals retain their explicit wider ABI fence.
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
        if bytes != u64::from(bits / 8) {
            return Ok(None);
        }
        Ok(if range.start == 1 && range.end == mask {
            Some(Self::ZeroExcludedInteger { leaf, primitive })
        } else if range.start == 0 && range.end < mask {
            Some(Self::UpperBoundedInteger {
                leaf,
                primitive,
                consumed: 0,
            })
        } else {
            None
        })
    }
}

/// A valid but wider scalar range is distinct from proven physical ineligibility.
#[derive(Clone, Copy)]
pub(super) enum Answer {
    Absent,
    Carrier(Carrier),
    PlainScalar(TypeId),
    WiderScalar,
}
impl Answer {
    pub(super) const fn carrier(self) -> Option<Carrier> {
        match self {
            Self::Carrier(carrier) => Some(carrier),
            Self::Absent | Self::PlainScalar(_) | Self::WiderScalar => None,
        }
    }
}

/// Rank the suffix's bit patterns by signed distance from zero, negative first
/// on ties. This is the pinned source ABI's one-value reservation policy.
/// The exact 0..=1 range reserves 2 first, including integer pattern types.
/// Computing the rank directly keeps nested layout demands linear in depth.
fn suffix_niche(mut end: u128, mask: u128, mut consumed: u128) -> Result<u128, &'static str> {
    if end >= mask || consumed >= mask - end {
        return Err("bounded carrier has no unused scalar value");
    }
    if end == 1 {
        if consumed == 0 {
            return Ok(2);
        }
        end = 2;
        consumed -= 1;
    }
    if end >= mask / 2 || consumed <= end {
        return Ok(mask - consumed);
    }
    let alternating = consumed - end - 1;
    let magnitude = end + 1 + alternating / 2;
    Ok(if alternating & 1 == 0 {
        magnitude
    } else {
        mask - magnitude
    })
}
