//! Version-one complete columns share the ordinary decoder's retained envelope.
use super::{Input, Writer, id, s, write};
use crate::support::crates::{MAX_FUNCTIONS, owner_catalog::DeclarationRef};

pub(super) fn encode(
    out: &mut Writer,
    columns: &s::HeaderAttributeColumns,
) -> Result<(), &'static str> {
    write(out, &columns.functions, row_write)?;
    write(out, &columns.traits, row_write)
}
pub(super) fn decode(read: &mut Input<'_, '_>) -> Result<s::HeaderAttributeColumns, &'static str> {
    read.reserve(size_of::<s::HeaderAttributeColumns>())?;
    Ok(s::HeaderAttributeColumns {
        functions: read.list(MAX_FUNCTIONS, 10, row_read)?,
        traits: read.list(MAX_FUNCTIONS, 10, row_read)?,
    })
}
fn row_write(out: &mut Writer, row: &s::HeaderContract) -> Result<(), &'static str> {
    id(out, row.declaration.0)?;
    id(out, row.declaration_site)?;
    out.u8(u8::from(row.must_use.is_some()))?;
    if let Some(value) = &row.must_use {
        id(out, value.owner_site)?;
        out.u8(u8::from(value.reason.is_some()))?;
        if let Some(reason) = &value.reason {
            out.string(reason)?;
        }
    }
    out.u8(u8::from(row.inline.is_some()))?;
    if let Some(value) = row.inline {
        id(out, value.owner_site)?;
        out.u8(match value.hint {
            s::InlineHint::Hint => 0,
            s::InlineHint::Always => 1,
            s::InlineHint::Never => 2,
        })?;
        out.u8(match value.applicability {
            s::InlineApplicability::FunctionBody => 0,
            s::InlineApplicability::DiagnosedNoBody => 1,
        })?;
    }
    Ok(())
}
fn row_read(read: &mut Input<'_, '_>) -> Result<s::HeaderContract, &'static str> {
    Ok(s::HeaderContract {
        declaration: DeclarationRef(read.id()?),
        declaration_site: read.id()?,
        must_use: if read.boolean()? {
            Some(s::MustUseContract {
                owner_site: read.id()?,
                reason: if read.boolean()? {
                    Some(reason_read(read)?)
                } else {
                    None
                },
            })
        } else {
            None
        },
        inline: if read.boolean()? {
            Some(s::InlineContract {
                owner_site: read.id()?,
                hint: match read.reader.u8()? {
                    0 => s::InlineHint::Hint,
                    1 => s::InlineHint::Always,
                    2 => s::InlineHint::Never,
                    _ => return Err("unknown transported inline hint"),
                },
                applicability: match read.reader.u8()? {
                    0 => s::InlineApplicability::FunctionBody,
                    1 => s::InlineApplicability::DiagnosedNoBody,
                    _ => return Err("unknown transported inline applicability"),
                },
            })
        } else {
            None
        },
    })
}
fn reason_read(read: &mut Input<'_, '_>) -> Result<String, &'static str> {
    let length = read.reader.u32()?;
    if length > s::MAX_REASON_BYTES {
        return Err("source must_use reason exceeds transport profile limit");
    }
    read.reader.atoms(length)?;
    read.reserve(length)?;
    let value = std::str::from_utf8(read.reader.bytes(length)?)
        .map_err(|_| "source must_use reason is not UTF-8")?;
    Ok(value.to_owned())
}
