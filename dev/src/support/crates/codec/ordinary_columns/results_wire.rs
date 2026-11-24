//! Version thirteen's associated-result header columns precede the optional
//! const-body trailer exactly when the declaration table names an associated
//! result.
use super::{Input, Writer, compiler_wire, g, id, s};
use crate::support::{
    crates::{MAX_COMPONENTS, owner_catalog::DeclarationRef},
    retained::Growth,
};

// Type indices are bounded well below this bit. Earlier readers reject the
// marked index, and unmarked definition rows retain their exact old bytes.
const HAS_INPUTS: u32 = 1 << 31;

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
            && rows.cold().associated_definition_inputs.is_empty()
            && rows.projections.is_empty()
        {
            Ok(())
        } else {
            Err("source associated result rows lack their declaration")
        };
    }
    super::write(out, &rows.associated_types, compiler_wire::associated_write)?;
    let definitions = rows.associated_definitions();
    out.u32(definitions.len())?;
    let mut inputs = rows.cold().associated_definition_inputs.iter().peekable();
    for (index, &definition) in (0_u32..).zip(definitions) {
        let inputs = inputs
            .next_if(|row| row.index == index)
            .map(|row| row.inputs);
        definition_write(out, definition, inputs)?;
    }
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
    let mut inputs = Vec::new();
    let mut index = 0;
    let definitions = read.list(MAX_COMPONENTS, 8, |read| {
        let definition = definition_read(read, index, &mut inputs)?;
        index += 1;
        Ok(definition)
    })?;
    if !definitions.is_empty() {
        let cold = read.cold(rows)?;
        cold.associated_definitions = definitions;
        cold.associated_definition_inputs = inputs;
    }
    rows.projections = read.list(MAX_COMPONENTS, 24, compiler_wire::projection_read)?;
    Ok(())
}
fn definition_write(
    out: &mut Writer,
    row: s::AssociatedDefinition,
    inputs: Option<g::Range>,
) -> Result<(), &'static str> {
    id(out, row.declaration.0)?;
    if let Some(inputs) = inputs {
        id(out, row.value.0 | HAS_INPUTS)?;
        super::range(out, inputs)
    } else {
        id(out, row.value.0)
    }
}
fn definition_read(
    read: &mut Input<'_, '_>,
    index: u32,
    inputs: &mut Vec<s::AssociatedDefinitionInputs>,
) -> Result<s::AssociatedDefinition, &'static str> {
    let declaration = DeclarationRef(read.id()?);
    let raw = read.id()?;
    if raw & HAS_INPUTS != 0 {
        if !read.profile.declaration_policies() {
            return Err("associated definition inputs require version fourteen");
        }
        let range = read.range()?;
        let growth =
            Growth::vector::<s::AssociatedDefinitionInputs>(inputs.len(), inputs.capacity(), 1);
        read.reader.atoms(growth.work)?;
        read.reserve(growth.bytes)?;
        inputs.push(s::AssociatedDefinitionInputs {
            index,
            inputs: range,
        });
    }
    Ok(s::AssociatedDefinition {
        declaration,
        value: g::TypeRef(raw & !HAS_INPUTS),
    })
}
