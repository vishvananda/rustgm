//! Version thirteen's enum variant columns follow the associated-result columns
//! exactly when the declaration table names an enum. A dense name column
//! follows the field rows exactly when some variant row is a record variant:
//! each record variant's field names, in variant-row and ordinal order, with
//! no count, as the record variants' field ranges imply it.
use super::{Input, Writer, g, id, range, s};
use crate::support::crates::{MAX_COMPONENTS, owner_catalog::DeclarationRef};

/// Presence is a function of rows already decoded, never of trailing bytes:
/// a packet without enum declarations keeps its exact earlier bytes.
fn present(declarations: &[s::Declaration]) -> bool {
    declarations
        .iter()
        .any(|row| row.definition.kind == s::DefinitionKind::Enum)
}
pub(super) fn encode(out: &mut Writer, rows: s::ValidatedView<'_>) -> Result<(), &'static str> {
    if !present(&rows.declarations) {
        return if rows.enums().is_empty() {
            Ok(())
        } else {
            Err("source enum variant rows lack their enum declaration")
        };
    }
    let enums = rows.enums();
    super::write_copy(out, &enums.headers, header_write)?;
    super::write_copy(out, &enums.variants, variant_write)?;
    super::write_copy(out, &enums.fields, field_write)?;
    for variant in &enums.variants {
        for name in enums.record_names(variant)?.unwrap_or_default() {
            out.string(name)?;
        }
    }
    Ok(())
}
pub(super) fn decode(
    read: &mut Input<'_, '_>,
    rows: &mut s::SourceRows,
) -> Result<(), &'static str> {
    read.reader.atoms(rows.declarations.len())?;
    if !present(&rows.declarations) {
        return Ok(());
    }
    let headers = read.list(MAX_COMPONENTS, 13, header_read)?;
    let variants = read.list(MAX_COMPONENTS, 33, variant_read)?;
    let fields = read.list(MAX_COMPONENTS, 16, field_read)?;
    let names = names_read(read, &variants, fields.len())?;
    if !(headers.is_empty() && variants.is_empty() && fields.is_empty()) {
        read.cold(rows)?.enums = s::EnumColumns {
            headers,
            variants,
            fields,
            names,
        };
    }
    Ok(())
}
/// Presence and length derive from the decoded variant rows: one name slot
/// per field row once any variant is a record variant, filled from the wire
/// for record variants' fields and empty for positional ones.
fn names_read(
    read: &mut Input<'_, '_>,
    variants: &[s::VariantHeader],
    fields: usize,
) -> Result<Vec<String>, &'static str> {
    if !variants
        .iter()
        .any(|variant| variant.shape == s::VariantShape::Record)
    {
        return Ok(Vec::new());
    }
    read.reserve(fields.saturating_mul(size_of::<String>()))?;
    let mut names = vec![String::new(); fields];
    for variant in variants {
        if variant.shape != s::VariantShape::Record {
            continue;
        }
        let slots = names
            .get_mut(variant.fields.start as usize..variant.fields.end as usize)
            .ok_or("source record variant names have incomplete field coverage")?;
        read.reader.atoms(slots.len())?;
        for slot in slots {
            *slot = read.string()?;
        }
    }
    Ok(names)
}
fn header_write(out: &mut Writer, row: s::EnumHeader) -> Result<(), &'static str> {
    id(out, row.declaration.0)?;
    // Tag zero is the default representation; otherwise the integer Scalar's
    // permanent byte follows.
    match row.tag {
        None => out.u8(0)?,
        Some(tag) => out.bytes(&[1, tag as u8])?,
    }
    range(out, row.variants)
}
fn header_read(read: &mut Input<'_, '_>) -> Result<s::EnumHeader, &'static str> {
    Ok(s::EnumHeader {
        declaration: DeclarationRef(read.id()?),
        tag: match read.reader.u8()? {
            0 => None,
            1 => Some(read.reader.scalar()?),
            _ => return Err("unknown source enum representation"),
        },
        variants: read.range()?,
    })
}
fn variant_write(out: &mut Writer, row: s::VariantHeader) -> Result<(), &'static str> {
    id(out, row.declaration.0)?;
    id(out, row.ordinal)?;
    out.u8(match row.shape {
        s::VariantShape::Unit => 0,
        s::VariantShape::Tuple => 1,
        s::VariantShape::Record => 2,
    })?;
    out.bytes(&row.discriminant.to_le_bytes())?;
    range(out, row.fields)
}
fn variant_read(read: &mut Input<'_, '_>) -> Result<s::VariantHeader, &'static str> {
    Ok(s::VariantHeader {
        declaration: DeclarationRef(read.id()?),
        ordinal: read.id()?,
        shape: match read.reader.u8()? {
            0 => s::VariantShape::Unit,
            1 => s::VariantShape::Tuple,
            2 => s::VariantShape::Record,
            _ => return Err("unknown source variant shape"),
        },
        discriminant: read.reader.u128()?,
        fields: read.range()?,
    })
}
fn field_write(out: &mut Writer, row: s::VariantField) -> Result<(), &'static str> {
    for value in [row.owner.0, row.ordinal, row.ty.0, row.site] {
        id(out, value)?;
    }
    Ok(())
}
fn field_read(read: &mut Input<'_, '_>) -> Result<s::VariantField, &'static str> {
    Ok(s::VariantField {
        owner: DeclarationRef(read.id()?),
        ordinal: read.id()?,
        ty: g::TypeRef(read.id()?),
        site: read.id()?,
    })
}
