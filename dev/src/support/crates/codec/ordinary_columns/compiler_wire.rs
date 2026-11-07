//! Version eleven appends semantic operations; it never encodes executable CFGs.
use super::{Input, Writer, address_read, address_write, g, id, range, s};
use crate::support::crates::{
    MAX_COMPONENTS,
    owner_catalog::{BodyRef, DeclarationRef},
};

pub(super) fn encode(out: &mut Writer, rows: s::ValidatedView<'_>) -> Result<(), &'static str> {
    super::write(out, &rows.associated_types, associated_write)?;
    super::write(out, &rows.projections, projection_write)?;
    super::write(out, &rows.opaque_futures, opaque_write)?;
    super::write(out, &rows.compiler_operations, operation_write)
}
pub(super) fn decode(
    read: &mut Input<'_, '_>,
    rows: &mut s::SourceRows,
) -> Result<(), &'static str> {
    rows.associated_types = read.list(MAX_COMPONENTS, 39, associated_read)?;
    rows.projections = read.list(MAX_COMPONENTS, 24, projection_read)?;
    rows.opaque_futures = read.list(MAX_COMPONENTS, 37, opaque_read)?;
    rows.compiler_operations = read.list(MAX_COMPONENTS, 21, operation_read)?;
    Ok(())
}
pub(super) fn associated_write(
    out: &mut Writer,
    row: &s::AssociatedTypeHeader,
) -> Result<(), &'static str> {
    for value in [
        row.declaration.0,
        row.parent.0,
        row.scope.0,
        row.projection.0,
    ] {
        id(out, value)?;
    }
    range(out, row.inputs)?;
    id(out, row.declared_inputs)?;
    range(out, row.bounds)?;
    out.bytes(&[
        u8::from(row.sized),
        u8::from(row.copy),
        u8::from(row.clone_),
    ])
}
pub(super) fn associated_read(
    read: &mut Input<'_, '_>,
) -> Result<s::AssociatedTypeHeader, &'static str> {
    Ok(s::AssociatedTypeHeader {
        declaration: DeclarationRef(read.id()?),
        parent: DeclarationRef(read.id()?),
        scope: DeclarationRef(read.id()?),
        projection: g::TypeRef(read.id()?),
        inputs: read.range()?,
        declared_inputs: read.id()?,
        bounds: read.range()?,
        sized: read.boolean()?,
        copy: read.boolean()?,
        clone_: read.boolean()?,
    })
}
pub(super) fn projection_write(
    out: &mut Writer,
    row: &s::ProjectionRecipe,
) -> Result<(), &'static str> {
    address_write(out, row.declaration)?;
    id(out, row.subject.0)?;
    id(out, row.trait_application.0)?;
    range(out, row.arguments)
}
pub(super) fn projection_read(
    read: &mut Input<'_, '_>,
) -> Result<s::ProjectionRecipe, &'static str> {
    Ok(s::ProjectionRecipe {
        declaration: address_read(read)?,
        subject: g::TypeRef(read.id()?),
        trait_application: s::TraitApplicationRef(read.id()?),
        arguments: read.range()?,
    })
}
fn opaque_write(out: &mut Writer, row: &s::OpaqueFuture) -> Result<(), &'static str> {
    id(out, row.declaration.0)?;
    id(out, row.ordinal)?;
    range(out, row.captures)?;
    id(out, row.output.0)?;
    address_write(out, row.future_trait)?;
    address_write(out, row.output_member)?;
    out.u8(u8::from(row.trait_result))
}
fn opaque_read(read: &mut Input<'_, '_>) -> Result<s::OpaqueFuture, &'static str> {
    Ok(s::OpaqueFuture {
        declaration: DeclarationRef(read.id()?),
        ordinal: read.id()?,
        captures: read.range()?,
        output: g::TypeRef(read.id()?),
        future_trait: address_read(read)?,
        output_member: address_read(read)?,
        trait_result: read.boolean()?,
    })
}
fn operation_write(out: &mut Writer, row: &s::CompilerOperation) -> Result<(), &'static str> {
    for value in [row.body.0, row.declaration.0, row.future.0, row.output.0] {
        id(out, value)?;
    }
    match row.kind {
        s::CompilerOperationKind::AsyncDropInPlace { target_formal } => {
            out.u8(0)?;
            id(out, target_formal)
        }
    }
}
fn operation_read(read: &mut Input<'_, '_>) -> Result<s::CompilerOperation, &'static str> {
    let body = BodyRef(read.id()?);
    let declaration = DeclarationRef(read.id()?);
    let future = g::TypeRef(read.id()?);
    let output = g::TypeRef(read.id()?);
    let kind = match read.reader.u8()? {
        0 => s::CompilerOperationKind::AsyncDropInPlace {
            target_formal: read.id()?,
        },
        _ => return Err("unknown deferred compiler-operation opcode"),
    };
    Ok(s::CompilerOperation {
        body,
        declaration,
        future,
        output,
        kind,
    })
}
