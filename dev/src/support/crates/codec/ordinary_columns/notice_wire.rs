//! Write checked deprecation notices and their effective declaration mappings.
use super::{Input, Writer, id, s, write};
use crate::support::crates::codec::notice_fields::{
    since_read, since_write, string_option_read, string_option_write,
};
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
            let owner = DeclarationRef(read.id()?);
            let site = read.id()?;
            let base = read.base;
            let mut reserve = |bytes| super::reserve_decoded(base, &mut read.bytes, bytes);
            Ok(s::DeprecationNotice {
                owner,
                site,
                since: since_read(read.reader, &mut reserve)?,
                note: string_option_read(read.reader, &mut reserve)?,
                suggestion: string_option_read(read.reader, &mut reserve)?,
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
