//! Existing row0/1 construction and storage stay independent of source packets.
use super::{ApplicationRef, Relation, Rows, Work, calls, graph, operations, storage, validate};
use crate::support::crates::{self, CrateIdentity, owner_catalog::BodyCatalog};
/// Only complete, structurally valid rows can enter an Interface. This is not a
/// proof token: import authenticates real remaps and re-proves every original.
#[derive(Debug)]
pub(super) struct Legacy {
    owners: BodyCatalog,
    rows: Rows,
    bytes: usize,
    peak: usize,
    names: usize,
    atoms: usize,
    application_owners: Vec<Option<crate::support::crates::owner_catalog::DeclarationRef>>,
}
impl PartialEq for Legacy {
    fn eq(&self, other: &Self) -> bool {
        self.owners == other.owners && self.rows == other.rows
    }
}
impl Eq for Legacy {}
impl Legacy {
    /// Work is charged even on failure; no partial payload is returned.
    /// # Errors
    /// Rejects malformed owners, roots, operations and bounded storage/work.
    pub(super) fn new(
        owner: &CrateIdentity,
        rows: Rows,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::with_base(owner, rows, work, 0)
    }
    pub(super) fn with_base(
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
    pub(super) fn application_owner(
        &self,
        application: ApplicationRef,
    ) -> Result<Option<crate::support::crates::owner_catalog::DeclarationRef>, &'static str> {
        self.application_owners
            .get(application.0 as usize)
            .copied()
            .ok_or("generic application ownership is absent")
    }
    #[must_use]
    pub(super) const fn rows(&self) -> &Rows {
        &self.rows
    }
    #[must_use]
    pub(super) const fn owners(&self) -> &BodyCatalog {
        &self.owners
    }
    #[must_use]
    pub(super) const fn retained_bytes(&self) -> usize {
        self.bytes
    }
    /// Required relations are recomputed from the exact immutable operations.
    /// # Errors
    /// Rejects a body reference outside this artifact.
    pub(super) fn relations(
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
    pub(super) const fn peak_bytes(&self) -> usize {
        self.peak
    }
    pub(super) const fn budget(&self) -> (usize, usize) {
        (self.atoms, self.names)
    }
}
