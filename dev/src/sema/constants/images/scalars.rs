//! Scalar codecs use the fixed x86-64 little-endian storage contract.
use crate::{literal::value::FloatBits, sema::model::Scalar, types::Primitive};
pub(in crate::sema::constants) fn width(ty: Primitive) -> Result<usize, &'static str> {
    Ok(match ty {
        Primitive::Bool => 1,
        Primitive::Char | Primitive::F32 => 4,
        Primitive::F64 => 8,
        _ => usize::from(
            ty.integer()
                .ok_or("union image field is not a primitive scalar")?
                .1
                / 8,
        ),
    })
}
pub(in crate::sema::constants) fn encode(
    ty: Primitive,
    value: Scalar,
) -> Result<(u128, usize), &'static str> {
    let size = width(ty)?;
    let bits = match (ty, value) {
        (Primitive::Bool, Scalar::Boolean(value)) => u128::from(value),
        (Primitive::Char, Scalar::Character(value)) => u128::from(u32::from(value)),
        (Primitive::F32, Scalar::Float(FloatBits::F32(bits))) => u128::from(bits),
        (Primitive::F64, Scalar::Float(FloatBits::F64(bits))) => u128::from(bits),
        (ty, Scalar::Integer(bits)) if ty.integer().is_some() => bits,
        _ => return Err("union scalar differs from its checked field type"),
    };
    if size < 16 && bits >> (size * 8) != 0 {
        return Err("union scalar bits exceed their storage width");
    }
    Ok((bits, size))
}
pub(in crate::sema::constants) fn decode(
    ty: Primitive,
    bytes: &[u8],
    initialized: &[u64],
) -> Result<Scalar, &'static str> {
    let size = width(ty)?;
    if bytes.len() < size {
        return Err("union scalar exceeds image bounds");
    }
    let mut bits = 0u128;
    for (index, &byte) in bytes[..size].iter().enumerate() {
        if initialized
            .get(index / 64)
            .is_none_or(|word| word & (1u64 << (index % 64)) == 0)
        {
            return Err("constant union scalar read uses uninitialized bytes");
        }
        bits |= u128::from(byte) << (index * 8);
    }
    Ok(match ty {
        Primitive::Bool => Scalar::Boolean(match bits {
            0 => false,
            1 => true,
            _ => return Err("constant union scalar read has invalid bool representation"),
        }),
        Primitive::Char => Scalar::Character(
            u32::try_from(bits)
                .ok()
                .and_then(char::from_u32)
                .ok_or("constant union scalar read has invalid char representation")?,
        ),
        Primitive::F32 => Scalar::Float(FloatBits::F32(
            u32::try_from(bits).map_err(|_| "invalid f32 image width")?,
        )),
        Primitive::F64 => Scalar::Float(FloatBits::F64(
            u64::try_from(bits).map_err(|_| "invalid f64 image width")?,
        )),
        ty if ty.integer().is_some() => Scalar::Integer(bits),
        _ => return Err("union image field is not a primitive scalar"),
    })
}
