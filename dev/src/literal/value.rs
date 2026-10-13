//! Typed literal facts. No host-sized arithmetic or inferred-type defaults.
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value<'source> {
    Integer(Integer),
    Float(Float<'source>),
    Character(char),
    Byte(u8),
    String(Cow<'source, [u8]>),
    ByteString(Cow<'source, [u8]>),
    /// Content excludes the implicit terminator and cannot contain NUL.
    CString(Cow<'source, [u8]>),
    Boolean(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,
    Usize,
    Isize,
}

impl IntegerType {
    #[must_use]
    pub fn from_suffix(suffix: &str) -> Option<Self> {
        Some(match suffix {
            "u8" => Self::U8,
            "i8" => Self::I8,
            "u16" => Self::U16,
            "i16" => Self::I16,
            "u32" => Self::U32,
            "i32" => Self::I32,
            "u64" => Self::U64,
            "i64" => Self::I64,
            "u128" => Self::U128,
            "i128" => Self::I128,
            "usize" => Self::Usize,
            "isize" => Self::Isize,
            _ => return None,
        })
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::U8 => "u8",
            Self::I8 => "i8",
            Self::U16 => "u16",
            Self::I16 => "i16",
            Self::U32 => "u32",
            Self::I32 => "i32",
            Self::U64 => "u64",
            Self::I64 => "i64",
            Self::U128 => "u128",
            Self::I128 => "i128",
            Self::Usize => "usize",
            Self::Isize => "isize",
        }
    }

    /// The initial course target is x86-64, independent of the compiler host.
    #[must_use]
    pub const fn bits(self) -> u32 {
        match self {
            Self::U8 | Self::I8 => 8,
            Self::U16 | Self::I16 => 16,
            Self::U32 | Self::I32 => 32,
            Self::U64 | Self::I64 | Self::Usize | Self::Isize => 64,
            Self::U128 | Self::I128 => 128,
        }
    }

    #[must_use]
    pub const fn signed(self) -> bool {
        matches!(
            self,
            Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::I128 | Self::Isize
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Integer {
    pub magnitude: u128,
    pub suffix: Option<IntegerType>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetInteger {
    /// Two's-complement bits, zero-extended to u128.
    pub bits: u128,
    /// The literal magnitude exceeds the range permitted by this context.
    pub overflow: bool,
}

impl Integer {
    /// Materialize after inference and unary-minus context are known. The
    /// caller owns suffix/type agreement and the `overflowing_literals` lint.
    /// This helper does not implement arbitrary expression arithmetic.
    ///
    /// # Errors
    /// Unary negation is not a Rust operation on unsigned integers.
    pub fn materialize(
        self,
        ty: IntegerType,
        negative: bool,
    ) -> Result<TargetInteger, &'static str> {
        if negative && !ty.signed() {
            return Err("cannot negate an unsigned literal");
        }
        let mask = u128::MAX >> (128 - ty.bits());
        let limit = if ty.signed() {
            (mask >> 1) + u128::from(negative)
        } else {
            mask
        };
        Ok(TargetInteger {
            bits: if negative {
                0_u128.wrapping_sub(self.magnitude) & mask
            } else {
                self.magnitude & mask
            },
            overflow: self.magnitude > limit,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatType {
    F32,
    F64,
}

impl FloatType {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Float<'source> {
    /// Valid decimal spelling, without separators or a suffix. Borrowed when
    /// already clean; retained to round directly to the eventual target type.
    pub decimal: Cow<'source, str>,
    pub suffix: Option<FloatType>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FloatBits {
    F32(u32),
    F64(u64),
}

impl Float<'_> {
    /// Round directly as required by expr.literal.float.value. Never round to
    /// f64 first and narrow to f32: that can introduce double rounding.
    ///
    /// # Errors
    /// Returns an error if manually constructed with invalid decimal syntax.
    pub fn round(&self, ty: FloatType) -> Result<FloatBits, &'static str> {
        match ty {
            FloatType::F32 => self
                .decimal
                .parse::<f32>()
                .map(|v| FloatBits::F32(v.to_bits())),
            FloatType::F64 => self
                .decimal
                .parse::<f64>()
                .map(|v| FloatBits::F64(v.to_bits())),
        }
        .map_err(|_| "invalid floating literal")
    }
}
