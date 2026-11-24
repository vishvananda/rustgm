//! Exact native linkage keeps the actual original or runtime import.
use super::{Input, Writer, id, s, write};
use crate::support::crates::{
    MAX_FUNCTIONS,
    owner_catalog::{BodyRef, DeclarationRef},
};
// This value was never a valid actual source body owner.
const IMPORT: u32 = u32::MAX;

pub(super) fn encode(out: &mut Writer, rows: &[s::ExactCallable]) -> Result<(), &'static str> {
    write(out, rows, |out, row| {
        id(out, row.declaration.0)?;
        id(out, row.body.map_or(IMPORT, |body| body.0))?;
        out.string(&row.symbol)
    })
}
pub(super) fn decode(read: &mut Input<'_, '_>) -> Result<Vec<s::ExactCallable>, &'static str> {
    read.list(MAX_FUNCTIONS, 13, |read| {
        let declaration = DeclarationRef(read.id()?);
        let body = read.id()?;
        let name = read.reader.text(4096)?;
        read.reserve(name.len())?;
        Ok(s::ExactCallable {
            declaration,
            body: (body != IMPORT).then_some(BodyRef(body)),
            symbol: name.into(),
        })
    })
}
