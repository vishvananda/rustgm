//! Closed scalar values retain exact bits without implying storage or executable bodies.
use super::{ExternalDef, ExternalKind, Scalar};

impl Scalar {
    pub(crate) const fn bit_width(self) -> u32 {
        match self {
            Self::Unit => 0,
            Self::Bool => 1,
            Self::U8 | Self::I8 => 8,
            Self::U16 | Self::I16 => 16,
            Self::Char | Self::U32 | Self::I32 | Self::F32 => 32,
            Self::U64 | Self::I64 | Self::Usize | Self::Isize | Self::F64 => 64,
            Self::U128 | Self::I128 => 128,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScalarValue {
    ty: Scalar,
    bits: u128,
}
impl ScalarValue {
    /// Integers use their width's two's-complement bits, with zero upper padding.
    /// Floating values preserve every bit, including signed zero and NaN payloads.
    /// # Errors
    /// Rejects invalid bool/char values, nonzero unit and out-of-width padding.
    pub fn new(ty: Scalar, bits: u128) -> Result<Self, &'static str> {
        if ty == Scalar::Char && u32::try_from(bits).ok().and_then(char::from_u32).is_none() {
            return Err("invalid metadata character value");
        }
        let width = ty.bit_width();
        if width < 128 && bits >> width != 0 {
            return Err("metadata scalar value exceeds its type width");
        }
        Ok(Self { ty, bits })
    }
    #[must_use]
    pub const fn ty(self) -> Scalar {
        self.ty
    }
    #[must_use]
    pub const fn bits(self) -> u128 {
        self.bits
    }
}

/// Body literals use the source type's width without extending legacy scalar tags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScalarLiteral {
    Primitive(ScalarValue),
    F16(u16),
    F128(u128),
}
impl ScalarLiteral {
    pub(crate) fn bits(self) -> u128 {
        match self {
            Self::Primitive(value) => value.bits(),
            Self::F16(bits) => u128::from(bits),
            Self::F128(bits) => bits,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Constant {
    definition: ExternalDef,
    value: ScalarValue,
}
impl Constant {
    /// A constant exports a value, without runtime linkage or allocation identity.
    /// # Errors
    /// Rejects identities belonging to another declaration kind.
    pub fn new(definition: ExternalDef, value: ScalarValue) -> Result<Self, &'static str> {
        if definition.kind() != ExternalKind::Constant {
            return Err("metadata value entry is not a constant");
        }
        Ok(Self { definition, value })
    }
    #[must_use]
    pub const fn definition(&self) -> &ExternalDef {
        &self.definition
    }
    #[must_use]
    pub const fn value(&self) -> ScalarValue {
        self.value
    }
}
