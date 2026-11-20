//! The v14 static table stores each canonical initializer once.
use super::{DeclarationRef, Input, Writer, crates, g, id, range, s, write};

pub(super) fn encode(out: &mut Writer, statics: &s::StaticColumns) -> Result<(), &'static str> {
    write(out, &statics.headers, |out, row| {
        id(out, row.declaration.0)?;
        id(out, row.ty.0)?;
        range(out, row.initializer)
    })?;
    out.u32(statics.values.len())?;
    out.bytes(&statics.values)
}

pub(super) fn decode(read: &mut Input<'_, '_>) -> Result<Box<s::StaticColumns>, &'static str> {
    let headers = read.list(crates::MAX_COMPONENTS, 16, |read| {
        Ok(s::StaticHeader {
            declaration: DeclarationRef(read.id()?),
            ty: g::TypeRef(read.id()?),
            initializer: read.range()?,
        })
    })?;
    if headers.is_empty() {
        return Err("source static trailer must contain an actual declaration");
    }
    let bytes = read.reader.count(crates::MAX_BYTES, 1)?;
    read.reserve(size_of::<s::StaticColumns>().saturating_add(bytes))?;
    let values = read.reader.bytes(bytes)?.to_vec();
    Ok(Box::new(s::StaticColumns { headers, values }))
}
