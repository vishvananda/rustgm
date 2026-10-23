//! Generic originals are validated data; ordinary proof still precedes execution.
mod calls;
mod graph;
mod operands;
mod operations;
mod rows;
#[path = "schema_rows.rs"]
pub mod source;
mod validate;
use crate::support::{
    crates::{self, CrateIdentity, owner_catalog::BodyCatalog},
    retained,
};
pub use operations::Relation;
pub use rows::*;

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
/// Only complete, structurally valid rows can enter an Interface. This is not a
/// proof token: import authenticates real remaps and re-proves every original.
#[derive(Debug)]
pub struct Payload {
    owners: BodyCatalog,
    rows: Rows,
    bytes: usize,
    peak: usize,
    names: usize,
    atoms: usize,
    application_owners: Vec<Option<crate::support::crates::owner_catalog::DeclarationRef>>,
}
impl PartialEq for Payload {
    fn eq(&self, other: &Self) -> bool {
        self.owners == other.owners && self.rows == other.rows
    }
}
impl Eq for Payload {}
impl Payload {
    /// Work is charged even on failure; no partial payload is returned.
    /// # Errors
    /// Rejects malformed owners, roots, operations and bounded storage/work.
    pub fn new(owner: &CrateIdentity, rows: Rows, work: &mut Work) -> Result<Self, &'static str> {
        Self::with_base(owner, rows, work, 0)
    }
    pub(crate) fn with_base(
        owner: &CrateIdentity,
        rows: Rows,
        work: &mut Work,
        base: usize,
    ) -> Result<Self, &'static str> {
        let first = work.steps;
        work.charge(1)?;
        if rows.edition != crate::support::edition::Edition::Rust2024 {
            return Err("generic row version zero requires defining Rust2024 edition");
        }
        let (bytes, peak) = storage(&rows, work)?;
        if base.saturating_add(peak) > crates::MAX_BYTES {
            return Err("generic metadata exceeds combined retained and scratch budget");
        }
        let names = validate::declarations(owner, &rows, work)?;
        let mut graph = graph::validate(&rows, work)?;
        let owners = validate::functions(owner, &rows, &graph, work)?;
        let application_owners = calls::validate(&rows, &graph, work)?;
        graph.applications(&rows, &application_owners, work)?;
        operations::validate(&rows, &graph, work)?;
        Ok(Self {
            owners,
            rows,
            bytes,
            peak,
            names,
            atoms: work.steps.saturating_sub(first),
            application_owners,
        })
    }
    pub(crate) fn application_owner(
        &self,
        application: ApplicationRef,
    ) -> Result<Option<crate::support::crates::owner_catalog::DeclarationRef>, &'static str> {
        self.application_owners
            .get(application.0 as usize)
            .copied()
            .ok_or("generic application ownership is absent")
    }
    #[must_use]
    pub const fn rows(&self) -> &Rows {
        &self.rows
    }
    #[must_use]
    pub const fn owners(&self) -> &BodyCatalog {
        &self.owners
    }
    #[must_use]
    pub const fn retained_bytes(&self) -> usize {
        self.bytes
    }
    /// Required relations are recomputed from the exact immutable operations.
    /// # Errors
    /// Rejects a body reference outside this artifact.
    pub fn relations(
        &self,
        body: crate::support::crates::owner_catalog::BodyRef,
    ) -> Result<impl Iterator<Item = Relation> + '_, &'static str> {
        let original = self
            .rows
            .originals
            .get(body.0 as usize)
            .ok_or("generic relation body is unknown")?;
        Ok(operations::relations(original))
    }
    pub(crate) const fn peak_bytes(&self) -> usize {
        self.peak
    }
    pub(crate) const fn budget(&self) -> (usize, usize) {
        (self.atoms, self.names)
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
