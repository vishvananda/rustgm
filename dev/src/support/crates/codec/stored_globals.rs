//! Profile five transports storage identities and typed initializer graphs.
mod values;
use super::{Profile, Reader, Writer};
use crate::support::crates::{
    CrateIdentity, ExternalDef, ExternalKind, MAX_COMPONENTS, MAX_FUNCTIONS, Static, Storage,
    StorageOrigin, StoredValue, Symbol, TypeRef, ValueRef,
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
        values::encode(out, &value.kind)?;
    }
    Ok(())
}

pub(super) fn decode(
    input: &mut Reader<'_>,
    owner: &CrateIdentity,
    profile: Profile,
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
        let kind = values::decode(input, profile)?;
        values.push(StoredValue { ty, kind });
    }
    Storage::new(entries, values)
}
fn ordinal(input: &mut Reader<'_>) -> Result<u32, &'static str> {
    u32::try_from(input.u32()?).map_err(|_| "metadata storage ordinal exceeds 32 bits")
}
