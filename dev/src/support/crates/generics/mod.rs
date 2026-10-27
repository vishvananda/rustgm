//! Generic originals are validated data; ordinary proof still precedes execution.
mod attachment;
pub(crate) use source::canonical_order;
mod calls;
mod graph;
mod legacy_payload;
mod operands;
mod operations;
mod rows;
#[path = "schema_rows.rs"]
pub mod source;
mod validate;
use crate::support::{
    crates::{self, owner_catalog::BodyCatalog},
    retained,
};
pub use attachment::Payload;
pub use operations::Relation;
pub use rows::*;

/// Readers distinguish supported representations without inventing legacy rows.
pub enum PayloadView<'a> {
    Legacy(&'a Rows),
    Source(&'a source::ValidatedRows),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Work {
    pub steps: usize,
    pub roots: usize,
    pub operations: usize,
}
impl Work {
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > crates::MAX_COMPONENTS {
            return Err("generic metadata work budget exceeded");
        }
        Ok(())
    }
}

fn storage(rows: &Rows, work: &mut Work) -> Result<(usize, usize), &'static str> {
    work.charge(1)?;
    let mut bytes = size_of::<Payload>().saturating_add(retained::sum([
        retained::vector(&rows.declarations),
        retained::vector(&rows.declaration_sites),
        retained::vector(&rows.binders),
        retained::vector(&rows.types),
        retained::vector(&rows.roots),
        retained::vector(&rows.parameters),
        retained::vector(&rows.clauses),
        retained::vector(&rows.functions),
        retained::vector(&rows.originals),
    ]));
    if bytes > crates::MAX_BYTES {
        return Err("generic metadata storage budget exceeded");
    }
    for declaration in &rows.declarations {
        work.charge(1)?;
        match declaration {
            Declaration::Function { .. } => {}
            Declaration::TypeParameter { name, .. }
            | Declaration::ValueParameter { name, .. }
            | Declaration::Local { name, .. } => bytes = bytes.saturating_add(name.capacity()),
        }
    }
    let mut arguments = 0_usize;
    for original in &rows.originals {
        work.charge(1)?;
        bytes = bytes.saturating_add(retained::sum([
            retained::vector(&original.locals),
            retained::vector(&original.parameter_bindings),
            retained::vector(&original.values),
            retained::vector(&original.statements),
            retained::vector(&original.blocks),
            retained::vector(&original.scopes),
            retained::vector(&original.exits),
            retained::vector(&original.temporaries),
            retained::vector(&original.arguments),
        ]));
        arguments = arguments.saturating_add(original.arguments.len());
        if arguments > crates::MAX_COMPONENTS {
            return Err("generic value argument table exceeds profile budget");
        }
        if rows.calls.is_none() && !original.arguments.is_empty() {
            return Err("generic value lists require row version one");
        }
    }
    let application_scratch = call_storage(rows, work, &mut bytes)?;
    // All validation scratch and the derived owner forest coexist with the rows.
    // Four hash slots per node covers minimum/geometric capacity before growth.
    let body_scratch = rows
        .originals
        .iter()
        .map(|original| {
            original
                .locals
                .len()
                .saturating_mul(size_of::<Option<u32>>() + size_of::<bool>())
                .saturating_add(
                    original
                        .values
                        .len()
                        .saturating_mul(size_of::<Option<u32>>()),
                )
                .saturating_add(original.statements.len().saturating_mul(size_of::<bool>()))
        })
        .max()
        .unwrap_or(0);
    let scratch = application_scratch
        .saturating_add(rows.calls.as_ref().map_or(0, |c| c.applications.len()))
        .saturating_add(graph::scratch(rows.types.len()))
        .saturating_add(rows.roots.len().saturating_mul(size_of::<bool>()))
        .saturating_add(body_scratch)
        .saturating_add(
            rows.declarations
                .len()
                .saturating_mul(2 * size_of::<u32>() + 4 * size_of::<bool>()),
        )
        .saturating_add(
            rows.originals
                .len()
                .saturating_mul(2 * size_of::<crate::support::crates::owner_catalog::BodyOwner>()),
        )
        .saturating_add(size_of::<BodyCatalog>() + 2 * size_of::<usize>());
    if bytes.saturating_add(scratch) > crates::MAX_BYTES {
        return Err("generic metadata storage budget exceeded");
    }
    Ok((bytes, bytes.saturating_add(scratch)))
}

#[cfg(test)]
#[path = "../../../../tests/invariants/original_rows.rs"]
mod tests;

fn call_storage(rows: &Rows, work: &mut Work, bytes: &mut usize) -> Result<usize, &'static str> {
    let scratch = if let Some(calls) = &rows.calls {
        work.charge(
            calls
                .applications
                .len()
                .saturating_add(calls.arguments.len()),
        )?;
        *bytes = bytes
            .saturating_add(size_of::<Calls>())
            .saturating_add(retained::vector(&calls.applications))
            .saturating_add(retained::vector(&calls.arguments))
            .saturating_add(calls.applications.len().saturating_mul(size_of::<
                Option<crate::support::crates::owner_catalog::DeclarationRef>,
            >()));
        crate::support::retained::new_map::<
            (
                crate::support::crates::owner_catalog::DeclarationRef,
                &[TypeRef],
            ),
            (),
        >(calls.applications.len())
    } else {
        0
    };
    Ok(scratch)
}
