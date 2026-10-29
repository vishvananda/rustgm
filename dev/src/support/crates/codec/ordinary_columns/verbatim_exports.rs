//! Explicit version-five exact callable definitions; no local arena IDs on the wire.
use super::{Input, Writer, id, s, write};
use crate::support::crates::{
    MAX_FUNCTIONS,
    owner_catalog::{BodyRef, DeclarationRef},
};
pub(super) fn encode(
    out: &mut Writer,
    rows: &[s::ExactCallableExport],
) -> Result<(), &'static str> {
    write(out, rows, |out, row| {
        id(out, row.declaration.0)?;
        id(out, row.body.0)?;
        out.string(&row.symbol)
    })
}
pub(super) fn decode(
    read: &mut Input<'_, '_>,
) -> Result<Vec<s::ExactCallableExport>, &'static str> {
    read.list(MAX_FUNCTIONS, 13, |read| {
        let declaration = DeclarationRef(read.id()?);
        let body = BodyRef(read.id()?);
        let name = read.reader.text(4096)?;
        read.reserve(name.len())?;
        Ok(s::ExactCallableExport {
            declaration,
            body,
            symbol: name.into(),
        })
    })
}
