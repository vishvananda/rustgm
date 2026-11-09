//! Version thirteen's constant header column follows the enum variant columns
//! exactly when the declaration table names a constant. It has no count and
//! no declaration field: row `k` belongs to the `k`-th constant declaration.
use super::{Input, Writer, g, id, recipe_columns, s};

/// Presence and length are functions of rows already decoded, never of
/// trailing bytes: a packet without constants keeps its exact earlier bytes.
fn count(declarations: &[s::Declaration]) -> usize {
    declarations
        .iter()
        .filter(|row| row.definition.kind == s::DefinitionKind::Constant)
        .count()
}
/// Each row is `ty | value`: value `0` is none, `1 | term` writes the
/// const term in place with the shared term codec.
pub(super) fn encode(out: &mut Writer, rows: s::ValidatedView<'_>) -> Result<(), &'static str> {
    let constants = rows.constants();
    if count(&rows.declarations) != constants.len() {
        return Err("source constant rows lack their declaration");
    }
    for row in constants {
        id(out, row.ty.0)?;
        match row.value {
            None => out.u8(0)?,
            Some(term) => {
                out.u8(1)?;
                recipe_columns::const_term_write(out, term)?;
            }
        }
    }
    Ok(())
}
pub(super) fn decode(
    read: &mut Input<'_, '_>,
    rows: &mut s::SourceRows,
) -> Result<(), &'static str> {
    read.reader.atoms(rows.declarations.len())?;
    let count = count(&rows.declarations);
    if count == 0 {
        return Ok(());
    }
    read.reserve(count.saturating_mul(size_of::<s::ConstantHeader>()))?;
    let mut constants = Vec::with_capacity(count);
    for _ in 0..count {
        let ty = g::TypeRef(read.id()?);
        let value = match read.reader.u8()? {
            0 => None,
            1 => Some(recipe_columns::const_term_read(read)?),
            _ => return Err("invalid source constant value presence"),
        };
        constants.push(s::ConstantHeader { ty, value });
    }
    read.cold(rows)?.constants = constants;
    Ok(())
}
