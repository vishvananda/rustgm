//! Write checked deprecation notices and their effective declaration mappings.
use super::{Input, Writer, availability_wire::text, id, s, write};
use crate::support::crates::{MAX_COMPONENTS, owner_catalog::DeclarationRef};

pub(super) fn encode(
    out: &mut Writer,
    columns: &s::DeprecationColumns,
) -> Result<(), &'static str> {
    write(out, &columns.notices, |out, row| {
        id(out, row.owner.0)?;
        id(out, row.site)?;
        since_write(out, &row.since)?;
        string_option_write(out, row.note.as_deref())?;
        string_option_write(out, row.suggestion.as_deref())
    })?;
    write(out, &columns.declarations, |out, row| {
        id(out, row.declaration.0)?;
        id(out, row.owner.0)
    })
}

pub(super) fn decode(read: &mut Input<'_, '_>) -> Result<s::DeprecationColumns, &'static str> {
    read.reserve(size_of::<s::DeprecationColumns>())?;
    Ok(s::DeprecationColumns {
        notices: read.list(MAX_COMPONENTS, 11, |read| {
            Ok(s::DeprecationNotice {
                owner: DeclarationRef(read.id()?),
                site: read.id()?,
                since: since_read(read)?,
                note: string_option_read(read)?,
                suggestion: string_option_read(read)?,
            })
        })?,
        declarations: read.list(MAX_COMPONENTS, 8, |read| {
            Ok(s::DeprecatedDeclaration {
                declaration: DeclarationRef(read.id()?),
                owner: DeclarationRef(read.id()?),
            })
        })?,
    })
}

fn since_write(out: &mut Writer, since: &s::DeprecationSince) -> Result<(), &'static str> {
    match since {
        s::DeprecationSince::Unspecified => out.u8(0),
        s::DeprecationSince::Future => out.u8(1),
        s::DeprecationSince::Version(s::AvailabilitySince::Current) => out.u8(2),
        s::DeprecationSince::Version(s::AvailabilitySince::Version {
            major,
            minor,
            patch,
        }) => {
            out.u8(3)?;
            out.u16(*major)?;
            out.u16(*minor)?;
            out.u16(*patch)
        }
        s::DeprecationSince::Nonstandard(value) => {
            out.u8(4)?;
            out.string(value)
        }
    }
}

fn since_read(read: &mut Input<'_, '_>) -> Result<s::DeprecationSince, &'static str> {
    Ok(match read.reader.u8()? {
        0 => s::DeprecationSince::Unspecified,
        1 => s::DeprecationSince::Future,
        2 => s::DeprecationSince::Version(s::AvailabilitySince::Current),
        3 => s::DeprecationSince::Version(s::AvailabilitySince::Version {
            major: read.reader.u16()?,
            minor: read.reader.u16()?,
            patch: read.reader.u16()?,
        }),
        4 => s::DeprecationSince::Nonstandard(text(read)?.into_boxed_str()),
        _ => return Err("unknown source deprecation since kind"),
    })
}

fn string_option_write(out: &mut Writer, value: Option<&str>) -> Result<(), &'static str> {
    out.u8(u8::from(value.is_some()))?;
    if let Some(value) = value {
        out.string(value)?;
    }
    Ok(())
}

fn string_option_read(read: &mut Input<'_, '_>) -> Result<Option<Box<str>>, &'static str> {
    if read.boolean()? {
        Ok(Some(text(read)?.into_boxed_str()))
    } else {
        Ok(None)
    }
}
