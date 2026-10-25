//! Structural owner indexes cannot mint descriptors, proof or frontend remaps.
#[path = "source_validation/regional_frames.rs"]
mod binder_domains;
#[path = "source_validation/lexical_forest.rs"]
mod declaration_graph;
#[path = "source_validation/domain_index.rs"]
mod domains;
#[path = "source_validation/completeness.rs"]
mod header_ledger;
#[path = "source_validation/table_storage.rs"]
mod table_storage;
#[path = "source_validation/recipe_domains.rs"]
mod type_domains;
#[path = "source_validation/unit_layout.rs"]
mod unit_layout;
use super::{DefinitionKind, ScopeIndex, SourceRows};
use crate::support::crates::{generics::Work, owner_catalog::DeclarationRef};
pub use domains::{
    DomainIndex, DomainWork, InputWork, OperationPort, OriginalInputPlan, RequiredInput,
    RequiredKind,
};
pub use unit_layout::NumberingUnits;
#[cfg(test)]
#[path = "../../../../tests/invariants/numbered_units.rs"]
mod numbered_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/transport_domains.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderRef {
    Trait(u32),
    Implementation(u32),
    Function(u32),
}

/// One borrowed ownership/header view. Type, operation and semantic validation
/// remain separate mandatory passes; this index is not `ValidatedRows`.
pub struct HeaderIndex<'a> {
    rows: &'a SourceRows,
    headers: Vec<Option<HeaderRef>>,
    scopes: ScopeIndex<'a>,
    bytes: usize,
    peak: usize,
}
impl<'a> HeaderIndex<'a> {
    /// Check complete raw numbering data without constructing frontend regions,
    /// defining environments, imported identities or any paid proof.
    /// # Errors
    /// Rejects incomplete units, invalid allocation/alias relations and budgets.
    pub fn numbering(&self, work: &mut Work) -> Result<NumberingUnits<'a>, &'static str> {
        unit_layout::validate(self, work)
    }
    /// Inspect actual binder/type/application/operation ownership. This is
    /// structural data validation, not a descriptor or a semantic proof.
    /// # Errors
    /// Rejects malformed domains, missing ports and attempted budget exhaustion.
    pub fn domains<'index>(
        &'index self,
        work: &mut Work,
    ) -> Result<DomainIndex<'index, 'a>, &'static str> {
        DomainIndex::new(self, work)
    }
    /// Index the complete original declaration/header/body association once.
    /// # Errors
    /// Rejects malformed ownership/coverage and aggregate storage/work limits.
    pub fn new(
        rows: &'a SourceRows,
        base_bytes: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let budget = table_storage::preflight(rows, base_bytes, work)?;
        let definitions = declaration_graph::validate(rows, work)?;
        let headers = header_ledger::validate(rows, work)?;
        let scopes = ScopeIndex::new(rows, &definitions, budget.scope_base, work)?;
        for header in &rows.traits {
            scopes.complete_slots(header.scope, work)?;
            scopes.complete_slots(header.self_definition, work)?;
            if scopes.parent(header.scope, work)? != Some(header.self_definition) {
                return Err("source trait scope inherited from another actual Self declaration");
            }
        }
        for header in &rows.implementations {
            scopes.complete_slots(header.scope, work)?;
        }
        for header in &rows.functions {
            scopes.complete_slots(header.scope, work)?;
            if scopes.parent(header.scope, work)? != header.parent {
                return Err(
                    "source function generic inheritance differs from its actual associated owner",
                );
            }
        }
        let bytes = budget
            .rows
            .saturating_add(size_of::<Self>())
            .saturating_add(
                headers
                    .capacity()
                    .saturating_mul(size_of::<Option<HeaderRef>>()),
            )
            .saturating_add(
                scopes
                    .retained_bytes()
                    .saturating_sub(size_of::<ScopeIndex<'_>>()),
            );
        Ok(Self {
            rows,
            headers,
            scopes,
            bytes,
            peak: budget.peak,
        })
    }

    pub(super) fn into_header_rows(self) -> Result<(Vec<Option<HeaderRef>>, usize), &'static str> {
        let scope_payload = self
            .scopes
            .retained_bytes()
            .saturating_sub(size_of::<ScopeIndex<'_>>());
        let indexes = size_of::<Self>()
            .saturating_add(scope_payload)
            .saturating_add(crate::support::retained::vector(&self.headers));
        let rows = self
            .bytes
            .checked_sub(indexes)
            .ok_or("source header storage decomposition is inconsistent")?;
        Ok((self.headers, rows))
    }

    /// # Errors
    /// Rejects unknown declarations and charges rejected lookups as well.
    pub fn header(
        &self,
        declaration: DeclarationRef,
        work: &mut Work,
    ) -> Result<Option<HeaderRef>, &'static str> {
        work.charge(1)?;
        self.headers
            .get(declaration.0 as usize)
            .copied()
            .ok_or("source header query has an unknown declaration")
    }
    #[must_use]
    pub const fn rows(&self) -> &'a SourceRows {
        self.rows
    }
    #[must_use]
    pub const fn scopes(&self) -> &ScopeIndex<'a> {
        &self.scopes
    }
    #[must_use]
    /// Combined borrowed row/base storage and this index's owned payload.
    pub const fn retained_bytes(&self) -> usize {
        self.bytes
    }
    #[must_use]
    pub const fn peak_bytes(&self) -> usize {
        self.peak
    }
}

fn declaration(
    rows: &SourceRows,
    reference: DeclarationRef,
    kind: DefinitionKind,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(1)?;
    if rows
        .declarations
        .get(reference.0 as usize)
        .is_none_or(|row| row.definition.kind != kind)
    {
        return Err("source header has an unknown or differently typed owner");
    }
    Ok(())
}
