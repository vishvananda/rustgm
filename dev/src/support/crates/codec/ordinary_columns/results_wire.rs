//! Version thirteen's associated-result header columns precede the optional
//! const-body trailer exactly when the declaration table names an associated
//! result.
use super::{Input, Writer, compiler_wire, g, id, s};
use crate::support::crates::{MAX_COMPONENTS, owner_catalog::DeclarationRef};

/// Presence is a function of rows already decoded, never of trailing bytes:
/// a packet without associated declarations keeps its exact earlier bytes.
pub(super) fn present(declarations: &[s::Declaration]) -> bool {
    declarations
        .iter()
        .any(|row| row.definition.kind == s::DefinitionKind::AssociatedType)
}
pub(super) fn encode(out: &mut Writer, rows: s::ValidatedView<'_>) -> Result<(), &'static str> {
    if !present(&rows.declarations) {
        return if rows.associated_types.is_empty()
            && rows.associated_definitions().is_empty()
            && rows.projections.is_empty()
        {
            Ok(())
        } else {
            Err("source associated result rows lack their declaration")
        };
    }
    super::write(out, &rows.associated_types, compiler_wire::associated_write)?;
    super::write_copy(out, rows.associated_definitions(), definition_write)?;
    super::write(out, &rows.projections, compiler_wire::projection_write)
}
pub(super) fn decode(
    read: &mut Input<'_, '_>,
    rows: &mut s::SourceRows,
) -> Result<(), &'static str> {
    read.reader.atoms(rows.declarations.len())?;
    if !present(&rows.declarations) {
        return Ok(());
    }
    rows.associated_types = read.list(MAX_COMPONENTS, 39, compiler_wire::associated_read)?;
    let definitions = read.list(MAX_COMPONENTS, 8, definition_read)?;
    if !definitions.is_empty() {
        read.cold(rows)?.associated_definitions = definitions;
    }
    rows.projections = read.list(MAX_COMPONENTS, 24, compiler_wire::projection_read)?;
    Ok(())
}
fn definition_write(out: &mut Writer, row: s::AssociatedDefinition) -> Result<(), &'static str> {
    id(out, row.declaration.0)?;
    id(out, row.value.0)
}
fn definition_read(read: &mut Input<'_, '_>) -> Result<s::AssociatedDefinition, &'static str> {
    Ok(s::AssociatedDefinition {
        declaration: DeclarationRef(read.id()?),
        value: g::TypeRef(read.id()?),
    })
}
