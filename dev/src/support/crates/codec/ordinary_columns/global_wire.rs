//! The v14 static table stores each canonical initializer once.
use super::{DeclarationRef, Input, Writer, crates, g, id, range, s, write};

// Scalar payloads are bounded far below this reserved starting offset.
const PROMOTION: u32 = u32::MAX;

pub(super) fn encode(out: &mut Writer, statics: &s::StaticColumns) -> Result<(), &'static str> {
    write(out, &statics.headers, |out, row| {
        id(out, row.declaration.0)?;
        id(out, row.ty.0)?;
        range(
            out,
            match row.initializer {
                s::StaticInitializer::Scalars(values) => values,
                s::StaticInitializer::Promotion(anchor) => g::Range {
                    start: PROMOTION,
                    end: anchor.0,
                },
            },
        )
    })?;
    out.u32(statics.values.len())?;
    out.bytes(&statics.values)
}

pub(super) fn decode(read: &mut Input<'_, '_>) -> Result<Box<s::StaticColumns>, &'static str> {
    let headers = read.list(crates::MAX_COMPONENTS, 16, |read| {
        let declaration = DeclarationRef(read.id()?);
        let ty = g::TypeRef(read.id()?);
        let values = read.range()?;
        let initializer = if values.start == PROMOTION {
            s::StaticInitializer::Promotion(DeclarationRef(values.end))
        } else {
            s::StaticInitializer::Scalars(values)
        };
        Ok(s::StaticHeader {
            declaration,
            ty,
            initializer,
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
