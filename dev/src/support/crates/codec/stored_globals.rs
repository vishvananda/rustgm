//! Profile five transports storage identities and typed initializer graphs.
use super::{Reader, Writer};
use crate::support::crates::{
    CrateIdentity, ExternalDef, ExternalKind, MAX_COMPONENTS, MAX_FUNCTIONS, ScalarValue, Static,
    StaticRef, Storage, StorageOrigin, StoredKind, StoredValue, Symbol, TypeRef, ValueRef,
};

pub(super) fn encode(out: &mut Writer, storage: &Storage) -> Result<(), &'static str> {
    out.u32(storage.statics().len())?;
    for entry in storage.statics() {
        out.u32(0)?;
        out.bytes(&[1, 4])?; // Value namespace, static identity.
        out.u8(u8::from(entry.public()) | (u8::from(entry.mutable()) << 1))?;
        out.u16(entry.ty().0)?;
        out.path(entry.definition().path())?;
        match entry.symbol() {
            Symbol::Rust => out.u8(0)?,
            Symbol::Exact(name) => {
                out.u8(1)?;
                out.string(name)?;
            }
        }
        match entry.origin() {
            StorageOrigin::Defined(root) => {
                out.u8(0)?;
                out.u32(root.0 as usize)?;
            }
            StorageOrigin::Foreign { unsafe_ } => out.u8(if unsafe_ { 2 } else { 1 })?,
        }
    }
    out.u32(storage.values().len())?;
    for value in storage.values() {
        out.u16(value.ty.0)?;
        match &value.kind {
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
        }
    }
    Ok(())
}

pub(super) fn decode(
    input: &mut Reader<'_>,
    owner: &CrateIdentity,
) -> Result<Storage, &'static str> {
    let count = input.count(MAX_FUNCTIONS, 19)?;
    input.atoms(count.saturating_mul(2))?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        input.owner(0)?;
        if input.bytes(2)? != [1, 4] {
            return Err("invalid metadata static identity");
        }
        let flags = input.u8()?;
        if flags > 3 {
            return Err("unknown metadata static flags");
        }
        let ty = TypeRef(input.u16()?);
        let definition = ExternalDef::new(owner.clone(), input.path()?, ExternalKind::Static)?;
        let symbol = match input.u8()? {
            0 => Symbol::Rust,
            1 => Symbol::Exact(input.string(4096)?.into()),
            _ => return Err("unknown metadata static symbol policy"),
        };
        let origin = match input.u8()? {
            0 => StorageOrigin::Defined(ValueRef(ordinal(input)?)),
            1 => StorageOrigin::Foreign { unsafe_: false },
            2 => StorageOrigin::Foreign { unsafe_: true },
            _ => return Err("unknown metadata static origin"),
        };
        entries.push(Static::new(
            definition,
            ty,
            symbol,
            flags & 1 != 0,
            flags & 2 != 0,
            origin,
        )?);
    }
    let count = input.count(MAX_COMPONENTS, 7)?;
    input.atoms(count)?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        let ty = TypeRef(input.u16()?);
        let kind = match input.u8()? {
            0 => {
                input.atoms(1)?;
                StoredKind::Scalar(ScalarValue::new(input.scalar()?, input.u128()?)?)
            }
            1 => StoredKind::Product(ordinals(input)?.into_iter().map(ValueRef).collect()),
            2 => {
                input.atoms(1)?;
                let target = StaticRef(ordinal(input)?);
                StoredKind::Reference {
                    target,
                    fields: ordinals(input)?.into_boxed_slice(),
                }
            }
            _ => return Err("unknown metadata stored value kind"),
        };
        values.push(StoredValue { ty, kind });
    }
    Storage::new(entries, values)
}
fn ordinal(input: &mut Reader<'_>) -> Result<u32, &'static str> {
    u32::try_from(input.u32()?).map_err(|_| "metadata storage ordinal exceeds 32 bits")
}
fn ordinals(input: &mut Reader<'_>) -> Result<Vec<u32>, &'static str> {
    let count = input.count(MAX_COMPONENTS, 4)?;
    input.atoms(count)?;
    (0..count).map(|_| ordinal(input)).collect()
}
