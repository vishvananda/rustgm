//! Version thirteen's constant header column follows the enum variant columns
//! exactly when the declaration table names a constant. It has no count and
//! no declaration field: row `k` belongs to the `k`-th constant declaration.
use super::{Input, Writer, g, id, recipe_columns, s};
use crate::support::crates::owner_catalog::{BodyRef, DeclarationRef};

/// Presence and length are functions of rows already decoded, never of
/// trailing bytes: a packet without constants keeps its exact earlier bytes.
fn count(declarations: &[s::Declaration]) -> usize {
    declarations
        .iter()
        .filter(|row| row.definition.kind == s::DefinitionKind::Constant)
        .count()
}
/// Each row has its type followed by the required-declaration tag, an
/// evaluated term, or the v14 original initializer's body identity.
pub(super) fn encode(out: &mut Writer, rows: s::ValidatedView<'_>) -> Result<(), &'static str> {
    let constants = rows.constants();
    if count(&rows.declarations) != constants.len() {
        return Err("source constant rows lack their declaration");
    }
    for row in constants {
        id(out, row.ty.0)?;
        match row.body {
            s::ConstantBody::Required => out.u8(0)?,
            s::ConstantBody::Evaluated(term) => {
                out.u8(1)?;
                recipe_columns::const_term_write(out, term)?;
            }
            s::ConstantBody::Original(body) => {
                if !rows.descriptor().profile().declaration_policies() {
                    return Err("source constant original requires the version-fourteen profile");
                }
                out.u8(2)?;
                id(out, body.0)?;
            }
            s::ConstantBody::PromotedReference { anchor, handlers } => {
                if !rows.descriptor().profile().declaration_policies() {
                    return Err("source promoted reference requires the version-fourteen profile");
                }
                out.u8(3)?;
                id(out, anchor.0)?;
                let targets = handlers.slice(&rows.cold().promoted_handlers)?;
                out.u32(targets.len())?;
                for target in targets {
                    id(out, target.identity)?;
                    id(out, target.signature.0)?;
                }
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
    let mut handlers = Vec::new();
    for _ in 0..count {
        let ty = g::TypeRef(read.id()?);
        let body = match read.reader.u8()? {
            0 => s::ConstantBody::Required,
            1 => s::ConstantBody::Evaluated(recipe_columns::const_term_read(read)?),
            2 if read.profile.declaration_policies() => {
                s::ConstantBody::Original(BodyRef(read.id()?))
            }
            3 if read.profile.declaration_policies() => promotion(read, &mut handlers)?,
            _ => return Err("invalid source constant value presence"),
        };
        constants.push(s::ConstantHeader { ty, body });
    }
    let cold = read.cold(rows)?;
    cold.constants = constants;
    cold.promoted_handlers = handlers;
    Ok(())
}

fn promotion(
    read: &mut Input<'_, '_>,
    handlers: &mut Vec<s::PromotedHandler>,
) -> Result<s::ConstantBody, &'static str> {
    let anchor = DeclarationRef(read.id()?);
    let start = offset(handlers.len())?;
    let targets = read.list(crate::support::crates::MAX_COMPONENTS, 8, |read| {
        Ok(s::PromotedHandler {
            identity: read.id()?,
            signature: g::TypeRef(read.id()?),
        })
    })?;
    read.reserve(
        targets
            .len()
            .saturating_mul(4 * size_of::<s::PromotedHandler>()),
    )?;
    handlers.extend(targets);
    Ok(s::ConstantBody::PromotedReference {
        anchor,
        handlers: g::Range {
            start,
            end: offset(handlers.len())?,
        },
    })
}

fn offset(length: usize) -> Result<u32, &'static str> {
    u32::try_from(length).map_err(|_| "source promoted handler ordinal overflow")
}
