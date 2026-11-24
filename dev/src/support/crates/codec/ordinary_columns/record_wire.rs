//! Version ten owns complete source struct fields independently of resolver IDs.
use super::{Input, Writer, id, range, s};
use crate::support::crates::{RecordRepr, generics::TypeRef, owner_catalog::DeclarationRef};
const FUNDAMENTAL: u8 = 128;
const INCOHERENT_INHERENT: u8 = 64;
const SIMD_LANE_LIMIT: u8 = 32;
const PACKED_SIMD: u8 = 16;
const NON_EXHAUSTIVE: u8 = 8;
const LAYOUT: u8 = 4;

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
    let repr = match row.repr {
        RecordRepr::Rust => 0,
        RecordRepr::C => 1,
        RecordRepr::Transparent => 2,
        RecordRepr::Simd => 3,
    };
    out.u8(repr
        | (u8::from(row.fundamental) * FUNDAMENTAL)
        | (u8::from(row.incoherent_inherent) * INCOHERENT_INHERENT)
        | (u8::from(row.simd_lane_limit.is_some()) * SIMD_LANE_LIMIT)
        | (u8::from(row.packed_simd) * PACKED_SIMD)
        | (u8::from(row.exhaustiveness == s::Exhaustiveness::Open) * NON_EXHAUSTIVE)
        | (u8::from(row.layout != s::NominalLayout::default()) * LAYOUT))?;
    if let Some(limit) = row.simd_lane_limit {
        out.bytes(&limit.to_le_bytes())?;
    }
    if row.layout != s::NominalLayout::default() {
        layout_write(out, row.layout)?;
    }
    Ok(())
}
pub(super) fn nominal_read(read: &mut Input<'_, '_>) -> Result<s::NominalHeader, &'static str> {
    let mut row = s::NominalHeader {
        contract: super::header_facts::row_read(read)?,
        declaration: DeclarationRef(read.id()?),
        scope: DeclarationRef(read.id()?),
        self_type: TypeRef(read.id()?),
        fields: read.range()?,
        inputs: read.range()?,
        declared_inputs: read.id()?,
        repr: RecordRepr::Rust,
        layout: s::NominalLayout::default(),
        fundamental: false,
        incoherent_inherent: false,
        packed_simd: false,
        exhaustiveness: s::Exhaustiveness::Closed,
        simd_lane_limit: None,
    };
    let flags = read.reader.u8()?;
    let repr = if read.profile.declaration_policies() {
        row.fundamental = flags & FUNDAMENTAL != 0;
        row.incoherent_inherent = flags & INCOHERENT_INHERENT != 0;
        row.packed_simd = flags & PACKED_SIMD != 0;
        if flags & NON_EXHAUSTIVE != 0 {
            row.exhaustiveness = s::Exhaustiveness::Open;
        }
        if flags & SIMD_LANE_LIMIT != 0 {
            row.simd_lane_limit = Some(read.reader.u64()?);
        }
        if flags & LAYOUT != 0 {
            row.layout = layout_read(read)?;
        }
        flags
            & !(FUNDAMENTAL
                | INCOHERENT_INHERENT
                | SIMD_LANE_LIMIT
                | PACKED_SIMD
                | NON_EXHAUSTIVE
                | LAYOUT)
    } else {
        flags
    };
    row.repr = match repr {
        0 => RecordRepr::Rust,
        1 => RecordRepr::C,
        2 if read.profile.body_nodes() => RecordRepr::Transparent,
        3 if read.profile.body_nodes() => RecordRepr::Simd,
        _ => return Err("unknown source nominal representation"),
    };
    Ok(row)
}
fn layout_write(out: &mut Writer, layout: s::NominalLayout) -> Result<(), &'static str> {
    let (kind, width) = match layout.modifier {
        s::LayoutModifier::Natural => (0, None),
        s::LayoutModifier::Aligned(width) => (1, Some(width)),
        s::LayoutModifier::Packed(width) => (2, Some(width)),
    };
    out.u8(kind | (u8::from(layout.indirect_arguments) << 2))?;
    if let Some(width) = width {
        id(out, width)?;
    }
    Ok(())
}
fn layout_read(read: &mut Input<'_, '_>) -> Result<s::NominalLayout, &'static str> {
    let flags = read.reader.u8()?;
    if flags == 0 || flags > 6 {
        return Err("invalid source nominal layout flags");
    }
    let modifier = match flags & 3 {
        0 => s::LayoutModifier::Natural,
        1 => s::LayoutModifier::Aligned(read.id()?),
        2 => s::LayoutModifier::Packed(read.id()?),
        _ => return Err("invalid source nominal layout modifier"),
    };
    Ok(s::NominalLayout {
        modifier,
        indirect_arguments: flags & 4 != 0,
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
