//! Nested DST alignment is a clamp of the terminal object's vtable alignment.

/// All bounds are powers of two. Prefix/explicit alignment raises the bounds;
/// packing caps them. A fixed interval needs no runtime metadata access.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Alignment {
    pub minimum: u32,
    pub maximum: u32,
}
impl Alignment {
    pub const OBJECT: Self = Self {
        minimum: 1,
        maximum: 1 << 29,
    };
    pub(super) const fn fixed(value: u32) -> Self {
        Self {
            minimum: value,
            maximum: value,
        }
    }
    pub(super) fn cap(self, value: Option<u32>) -> Self {
        value.map_or(self, |value| Self {
            minimum: self.minimum.min(value),
            maximum: self.maximum.min(value),
        })
    }
    pub(super) fn raise(self, value: u32) -> Self {
        Self {
            minimum: self.minimum.max(value),
            maximum: self.maximum.max(value),
        }
    }
    #[must_use]
    pub const fn is_dynamic(self) -> bool {
        self.minimum != self.maximum
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Offset {
    Fixed(u64),
    Aligned { prefix: u64, alignment: Alignment },
}
