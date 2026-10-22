//! Bounded structural recipes preserve repeats, byte payloads and typed element paths.
use super::{Profile, Reader, Writer, ordinal};
use crate::support::crates::{
    MAX_COMPONENTS, ScalarValue, StaticRef, StoredKind, StoredProjection, ValueRef,
};

pub(super) fn encode(out: &mut Writer, value: &StoredKind) -> Result<(), &'static str> {
    match value {
        StoredKind::Scalar(scalar) => {
            out.bytes(&[0, scalar.ty() as u8])?;
            out.bytes(&scalar.bits().to_le_bytes())?;
        }
        StoredKind::Product(fields) => {
            out.u8(1)?;
            out.u32(fields.len())?;
            for field in fields {
                out.u32(field.0 as usize)?;
            }
        }
        StoredKind::Reference { target, fields } => {
            out.u8(2)?;
            out.u32(target.0 as usize)?;
            out.u32(fields.len())?;
            for &field in fields {
                out.u32(field as usize)?;
            }
        }
        StoredKind::Repeat(child) => {
            out.u8(3)?;
            out.u32(child.0 as usize)?;
        }
        StoredKind::Bytes(bytes) => {
            out.u8(4)?;
            out.u32(bytes.len())?;
            out.bytes(bytes)?;
        }
        StoredKind::ProjectedReference { target, path } => {
            out.u8(5)?;
            out.u32(target.0 as usize)?;
            out.u32(path.len())?;
            for step in path {
                match *step {
                    StoredProjection::Field(index) => {
                        out.u8(0)?;
                        out.u32(index as usize)?;
                    }
                    StoredProjection::Element(index) => {
                        out.u8(1)?;
                        out.bytes(&index.to_le_bytes())?;
                    }
                }
            }
        }
    }
    Ok(())
}
pub(super) fn decode(input: &mut Reader<'_>, profile: Profile) -> Result<StoredKind, &'static str> {
    match input.u8()? {
        0 => {
            input.atoms(1)?;
            Ok(StoredKind::Scalar(ScalarValue::new(
                input.scalar()?,
                input.u128()?,
            )?))
        }
        1 => Ok(StoredKind::Product(
            ordinals(input)?.into_iter().map(ValueRef).collect(),
        )),
        2 => {
            input.atoms(1)?;
            let target = StaticRef(ordinal(input)?);
            Ok(StoredKind::Reference {
                target,
                fields: ordinals(input)?.into_boxed_slice(),
            })
        }
        3 if profile.aggregates => {
            input.atoms(1)?;
            Ok(StoredKind::Repeat(ValueRef(ordinal(input)?)))
        }
        4 if profile.aggregates => {
            let count = input.count(MAX_COMPONENTS, 1)?;
            input.atoms(count)?;
            Ok(StoredKind::Bytes(input.bytes(count)?.into()))
        }
        5 if profile.aggregates => {
            input.atoms(1)?;
            let target = StaticRef(ordinal(input)?);
            let count = input.count(MAX_COMPONENTS, 5)?;
            input.atoms(count)?;
            let mut path = Vec::with_capacity(count);
            for _ in 0..count {
                path.push(match input.u8()? {
                    0 => StoredProjection::Field(ordinal(input)?),
                    1 => StoredProjection::Element(input.u64()?),
                    _ => return Err("unknown metadata static projection kind"),
                });
            }
            Ok(StoredKind::ProjectedReference {
                target,
                path: path.into_boxed_slice(),
            })
        }
        _ => Err("unknown metadata stored value kind"),
    }
}
fn ordinals(input: &mut Reader<'_>) -> Result<Vec<u32>, &'static str> {
    let count = input.count(MAX_COMPONENTS, 4)?;
    input.atoms(count)?;
    (0..count).map(|_| ordinal(input)).collect()
}
