//! Literal bits as the pinned text reader gives the writer's spelling of each
//! literal in the context of its use.
//!
//! `RustGM`'s writer spells an integer as its unsigned decimal value, a null
//! pointer as `nullptr` and a float as `nan`, `inf`, `-inf` or a decimal with
//! eighteen significant digits. The reader decides the bits by the type the
//! use reads the operand in: an integer in an `iN`, `uN` or `ptr` context is
//! wrapped to 64 bits and sign-extended, in an `i128` context or one the use
//! decides later (call arguments, `index` offsets) it keeps all 128 bits, and
//! in a float context it is that float's value. A float spelling is rounded
//! once to the float context's kind, and is an `f64` elsewhere; in an integer
//! context the reader refuses it.
use crate::records::{KIND_F32, KIND_F64, Literal as Bits, VALUE_FLOAT, VALUE_INTEGER};
use rustgm::lowir::{program::Literal, vocabulary::Type};

/// The type a use reads its operand in.
#[derive(Clone, Copy, Debug)]
pub enum Context {
    /// The use decides the type later: all 128 bits of an integer are kept.
    Open,
    /// A scalar of this type.
    Scalar(Type),
    /// An object: integers are wrapped and floats read as `f64`.
    Object,
}
impl Context {
    /// The context of a use typed `ty`; `void` is the open context.
    pub fn of(ty: Type) -> Self {
        if ty == Type::Void {
            Self::Open
        } else {
            Self::Scalar(ty)
        }
    }
    fn integer(self) -> bool {
        matches!(self, Self::Scalar(ty) if ty != Type::F32 && ty != Type::F64)
    }
    fn wide(self) -> bool {
        matches!(self, Self::Open | Self::Scalar(Type::I128))
    }
    /// The float kind of a float spelling: the context's, else `f64`.
    fn float_kind(self) -> FloatKind {
        match self {
            Self::Scalar(Type::F32) => FloatKind::F32,
            _ => FloatKind::F64,
        }
    }
    fn float(self) -> Option<FloatKind> {
        match self {
            Self::Scalar(Type::F32) => Some(FloatKind::F32),
            Self::Scalar(Type::F64) => Some(FloatKind::F64),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FloatKind {
    F32,
    F64,
}
impl FloatKind {
    fn type_kind(self) -> u8 {
        match self {
            Self::F32 => KIND_F32,
            Self::F64 => KIND_F64,
        }
    }
}

/// One literal as the pool and its operand state it.
#[derive(Clone, Copy, Debug)]
pub struct Read {
    pub kind: u8,
    pub float_kind: u8,
    pub bits: Bits,
}
impl Read {
    fn integer(low: u64, high: u64) -> Self {
        Self {
            kind: VALUE_INTEGER,
            float_kind: 0,
            bits: Bits { low, high },
        }
    }
    fn float(kind: FloatKind, low: u64) -> Self {
        Self {
            kind: VALUE_FLOAT,
            float_kind: kind.type_kind(),
            bits: Bits { low, high: 0 },
        }
    }
}

/// The bits the reader gives `literal` in `context`, or the form it refuses
/// or that this mirror does not decide exactly.
pub fn read(literal: Literal, context: Context) -> Result<Read, &'static str> {
    match literal {
        Literal::Null => Ok(Read::integer(0, 0)),
        Literal::Integer(value) => integer(value, context),
        Literal::F32(bits) => float(f64::from(f32::from_bits(bits)), context),
        Literal::F64(bits) => float(f64::from_bits(bits), context),
    }
}

fn split(value: u128) -> (u64, u64) {
    let [low, high] = [value, value >> 64].map(|half| {
        let bytes = half.to_le_bytes();
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    });
    (low, high)
}

fn integer(value: u128, context: Context) -> Result<Read, &'static str> {
    let (low, high) = split(value);
    if let Some(kind) = context.float() {
        // The 128-bit two's complement value, rounded once to the kind.
        let negative = high >> 63 != 0;
        let magnitude = if negative {
            value.wrapping_neg()
        } else {
            value
        };
        let (magnitude, beyond) = split(magnitude);
        if beyond != 0 {
            return Err("integer literal of more than 64 bits in a float context");
        }
        return Ok(match kind {
            FloatKind::F32 => {
                let value = rounded_f32(magnitude);
                let value = if negative { -value } else { value };
                Read::float(kind, u64::from(value.to_bits()))
            }
            FloatKind::F64 => {
                let value = rounded_f64(magnitude);
                let value = if negative { -value } else { value };
                Read::float(kind, value.to_bits())
            }
        });
    }
    if context.wide() {
        return Ok(Read::integer(low, high));
    }
    let extended = if low >> 63 == 0 { 0 } else { u64::MAX };
    Ok(Read::integer(low, extended))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "the reader rounds the integer's value once to the float kind, as this conversion does"
)]
fn rounded_f32(magnitude: u64) -> f32 {
    magnitude as f32
}

#[expect(
    clippy::cast_precision_loss,
    reason = "the reader rounds the integer's value once to the float kind, as this conversion does"
)]
fn rounded_f64(magnitude: u64) -> f64 {
    magnitude as f64
}

fn float(value: f64, context: Context) -> Result<Read, &'static str> {
    if context.integer() {
        return Err("float literal in an integer context");
    }
    let kind = context.float_kind();
    if value.is_nan() {
        // `nan`: the kind's positive quiet NaN without a payload.
        return Ok(match kind {
            FloatKind::F32 => Read::float(kind, 0x7fc0_0000),
            FloatKind::F64 => Read::float(kind, 0x7ff8_0000_0000_0000),
        });
    }
    match kind {
        // Eighteen significant digits read back as the same `f64`.
        FloatKind::F64 => Ok(Read::float(kind, value.to_bits())),
        FloatKind::F32 => {
            narrowed(value).map(|value| Read::float(kind, u64::from(value.to_bits())))
        }
    }
}

/// The `f32` the reader's `strtof` gives the writer's spelling of `value`.
fn narrowed(value: f64) -> Result<f32, &'static str> {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a value that the cast keeps exactly is the one any correct rounding gives"
    )]
    let exact = value as f32;
    if f64::from(exact).to_bits() == value.to_bits() || value.is_infinite() {
        return Ok(exact);
    }
    // Round the decimal itself, once and correctly, as `strtof` does; the
    // writer's spelling is not the `f64` it was written from.
    let spelled = format!("{:.17e}", value.abs());
    let rounded: f32 = spelled
        .parse()
        .map_err(|_| "float literal the bridge cannot read")?;
    if rounded.is_infinite() {
        return Err("float literal beyond the range of f32");
    }
    Ok(if value.is_sign_negative() {
        -rounded
    } else {
        rounded
    })
}
