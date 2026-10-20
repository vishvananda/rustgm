//! Constant pointer casts preserve backing identity and explicit length metadata.
use super::{Value, arithmetic, values::Failure};
use crate::{
    sema::model::{Cast, Scalar},
    types::Primitive,
};

pub(super) fn cast(
    kind: Cast,
    source: Option<Primitive>,
    value: Value,
) -> Option<Result<Value, Failure>> {
    Some(match kind {
        Cast::Pointer | Cast::PointerThin => match value {
            Value::Reference(mut pointer) => {
                if kind == Cast::PointerThin {
                    pointer.length = None;
                }
                Ok(Value::Reference(pointer))
            }
            Value::Address { bits, length } => Ok(Value::Address {
                bits,
                length: if kind == Cast::PointerThin {
                    None
                } else {
                    length
                },
            }),
            Value::BytePointer { text, .. } if kind == Cast::PointerThin => Ok(Value::Bytes(text)),
            value @ Value::BytePointer { .. } => Ok(value),
            Value::Function(_)
            | Value::Text(_)
            | Value::Bytes(_)
            | Value::Scalar(Scalar::Integer(_)) => Ok(value),
            _ => Err(Failure::Invalid("pointer cast lacks an address value")),
        },
        Cast::IntegerToPointer => source
            .ok_or(Failure::Invalid(
                "integer address cast lacks a scalar source",
            ))
            .and_then(|source| {
                arithmetic::cast(
                    Cast::IntegerToInteger,
                    source,
                    Primitive::Usize,
                    value.scalar().map_err(Failure::Invalid)?,
                )
            })
            .and_then(|value| {
                let Scalar::Integer(bits) = value else {
                    return Err(Failure::Invalid(
                        "integer address cast produced a noninteger",
                    ));
                };
                u64::try_from(bits)
                    .map(|bits| Value::Address { bits, length: None })
                    .map_err(|_| {
                        Failure::Invalid("integer address exceeds the target pointer width")
                    })
            }),
        Cast::PointerToInteger => Err(Failure::Invalid(
            "pointers cannot be cast to integers during const eval",
        )),
        _ => return None,
    })
}

pub(super) fn unsize(value: Value, length: u64) -> Result<Value, Failure> {
    Ok(match value {
        Value::Reference(mut pointer) => {
            pointer.length = Some(length);
            Value::Reference(pointer)
        }
        Value::Address { bits, .. } => Value::Address {
            bits,
            length: Some(length),
        },
        Value::Text(text) | Value::Bytes(text) | Value::BytePointer { text, .. } => {
            Value::BytePointer { text, length }
        }
        _ => return Err(Failure::Invalid("unsizing constant lacks an address")),
    })
}
