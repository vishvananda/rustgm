//! Version ten owns complete source struct fields independently of resolver IDs.
use super::{Input, Writer, id, range, s};
use crate::support::crates::{RecordRepr, generics::TypeRef, owner_catalog::DeclarationRef};
pub(super) fn encode(out: &mut Writer, rows: s::ValidatedView<'_>) -> Result<(), &'static str> {
    // Version thirteen alone carries a transparent or SIMD struct header.
    if !rows.descriptor().profile().body_nodes()
        && rows
            .nominals
            .iter()
            .any(|row| matches!(row.repr, RecordRepr::Transparent | RecordRepr::Simd))
    {
        return Err("source nominal representation requires its wider complete contract");
    }
    super::write(out, &rows.nominals, nominal_write)?;
    super::write(out, &rows.fields, field_write)
}
pub(super) fn nominal_write(out: &mut Writer, row: &s::NominalHeader) -> Result<(), &'static str> {
    super::header_facts::row_write(out, &row.contract)?;
    id(out, row.declaration.0)?;
    id(out, row.scope.0)?;
    id(out, row.self_type.0)?;
    range(out, row.fields)?;
    range(out, row.inputs)?;
    id(out, row.declared_inputs)?;
    out.u8(match row.repr {
        RecordRepr::Rust => 0,
        RecordRepr::C => 1,
        RecordRepr::Transparent => 2,
        RecordRepr::Simd => 3,
    })
}
pub(super) fn nominal_read(read: &mut Input<'_, '_>) -> Result<s::NominalHeader, &'static str> {
    Ok(s::NominalHeader {
        contract: super::header_facts::row_read(read)?,
        declaration: DeclarationRef(read.id()?),
        scope: DeclarationRef(read.id()?),
        self_type: TypeRef(read.id()?),
        fields: read.range()?,
        inputs: read.range()?,
        declared_inputs: read.id()?,
        repr: match read.reader.u8()? {
            0 => RecordRepr::Rust,
            1 => RecordRepr::C,
            2 if read.profile.body_nodes() => RecordRepr::Transparent,
            3 if read.profile.body_nodes() => RecordRepr::Simd,
            _ => return Err("unknown source nominal representation"),
        },
    })
}
fn field_write(out: &mut Writer, row: &s::NominalField) -> Result<(), &'static str> {
    id(out, row.owner.0)?;
    id(out, row.ordinal)?;
    out.string(&row.name)?;
    id(out, row.ty.0)?;
    match row.visibility {
        s::ModuleVisibility::Public => out.u8(0)?,
        s::ModuleVisibility::Within(owner) => {
            out.u8(1)?;
            id(out, owner.0)?;
        }
    }
    id(out, row.site)
}
pub(super) fn field_read(read: &mut Input<'_, '_>) -> Result<s::NominalField, &'static str> {
    Ok(s::NominalField {
        owner: DeclarationRef(read.id()?),
        ordinal: read.id()?,
        name: read.string()?,
        ty: TypeRef(read.id()?),
        visibility: match read.reader.u8()? {
            0 => s::ModuleVisibility::Public,
            1 => s::ModuleVisibility::Within(DeclarationRef(read.id()?)),
            _ => return Err("invalid source nominal field visibility"),
        },
        site: read.id()?,
    })
}
