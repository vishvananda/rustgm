//! Implementation polarity is independent of the const effect.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ImplPolarity {
    Positive = 0,
    Negative = 1,
    Reservation = 2,
}

impl ImplPolarity {
    pub(crate) const fn from_negative(negative: bool) -> Self {
        if negative {
            Self::Negative
        } else {
            Self::Positive
        }
    }

    pub(crate) const fn is_positive(self) -> bool {
        matches!(self, Self::Positive)
    }

    pub(crate) const fn is_negative(self) -> bool {
        matches!(self, Self::Negative)
    }
}
