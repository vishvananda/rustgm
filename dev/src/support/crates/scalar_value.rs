//! Closed scalar values retain exact bits without implying storage or executable bodies.
use super::{ExternalDef, ExternalKind, Scalar};

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
        let width = match ty {
            Scalar::Unit => 0,
            Scalar::Bool => 1,
            Scalar::Char => {
                u32::try_from(bits)
                    .ok()
                    .and_then(char::from_u32)
                    .ok_or("invalid metadata character value")?;
                32
            }
            Scalar::U8 | Scalar::I8 => 8,
            Scalar::U16 | Scalar::I16 => 16,
            Scalar::U32 | Scalar::I32 | Scalar::F32 => 32,
            Scalar::U64 | Scalar::I64 | Scalar::Usize | Scalar::Isize | Scalar::F64 => 64,
            Scalar::U128 | Scalar::I128 => 128,
        };
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
