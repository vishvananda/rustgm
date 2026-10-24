//! Declaration/clause ownership never requires a body or invented syntax node.
use super::{
    Catalog, Edition, ImportedOwnerId, ImportedProofSite, ProofSite,
    catalog::{DeclarationRow, OwnerRow},
};
use crate::{
    resolve::{
        model::{DefId, DefKind},
        unit::NodeKey,
    },
    support::crates::{CrateIdentity, owner_catalog::DeclarationRef},
};
use std::{num::NonZeroU32, ops::Range};

/// Catalog-local rejection identity; no stable symbol or serialized ordinal.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImportedDeclarationId(pub(super) ImportedOwnerId);
impl std::fmt::Debug for ImportedDeclarationId {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str("ImportedDeclarationId(..)")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeclarationOrigin {
    Source(NodeKey),
    Imported(ImportedDeclarationId),
}
impl DeclarationOrigin {
    pub const fn source(self) -> Option<NodeKey> {
        match self {
            Self::Source(source) => Some(source),
            Self::Imported(_) => None,
        }
    }
}
impl From<NodeKey> for DeclarationOrigin {
    fn from(source: NodeKey) -> Self {
        Self::Source(source)
    }
}
/// Clause/annotation ordinal in one declaration, distinct from a body operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeclarationSite(NonZeroU32);
impl DeclarationSite {
    fn checked(index: u32) -> Result<Self, &'static str> {
        index
            .checked_add(1)
            .and_then(NonZeroU32::new)
            .map(Self)
            .ok_or("declaration site space exhausted")
    }
    pub const fn index(self) -> u32 {
        self.0.get() - 1
    }
}
pub(crate) struct DeclarationBinding {
    pub kind: DefKind,
    pub sites: Range<u32>,
}
impl Catalog {
    /// Read one sealed current declaration without fabricating a source node or
    /// accepting an equal-shaped row from another catalog.
    pub(crate) fn declaration_identity(
        &self,
        id: ImportedDeclarationId,
        kind: DefKind,
    ) -> Result<DefId, &'static str> {
        let row = self.declaration_row(id)?;
        if row.kind != kind {
            return Err("imported declaration handle changed its actual kind");
        }
        Ok(row.definition)
    }
    pub(crate) fn declaration(
        &self,
        definition: DefId,
    ) -> Result<ImportedDeclarationId, &'static str> {
        self.charge(1)?;
        self.definitions
            .get(&definition)
            .copied()
            .ok_or("imported declaration is absent from this catalog")
    }
    pub(super) fn declaration_row(
        &self,
        id: ImportedDeclarationId,
    ) -> Result<&DeclarationRow, &'static str> {
        match self.owner(id.0)? {
            OwnerRow::Declaration(row) => Ok(row),
            OwnerRow::Body(_) => Err("imported declaration handle names a body row"),
        }
    }
    pub(crate) fn validate_declaration(
        &self,
        id: ImportedDeclarationId,
        definition: DefId,
        kind: DefKind,
    ) -> Result<super::DeclarationOrigin, &'static str> {
        let row = self.declaration_row(id)?;
        if row.definition != definition || row.kind != kind {
            return Err("imported declaration changed its exact identity or kind");
        }
        Ok(DeclarationOrigin::Imported(id))
    }
    pub(crate) fn declaration_coordinate(
        &self,
        id: ImportedDeclarationId,
    ) -> Result<(&CrateIdentity, DeclarationRef, Edition), &'static str> {
        let row = self.declaration_row(id)?;
        let artifact = &self.artifacts[row.artifact as usize];
        Ok((
            &artifact.owner,
            DeclarationRef(row.ordinal),
            artifact.edition,
        ))
    }
    pub(crate) fn declaration_site(
        &self,
        id: ImportedDeclarationId,
        index: u32,
    ) -> Result<super::DeclarationSite, &'static str> {
        let row = self.declaration_row(id)?;
        self.charge(1)?;
        let sites = &row.sites;
        if index >= sites.end - sites.start {
            return Err("declaration site belongs outside its imported owner");
        }
        DeclarationSite::checked(index)
    }
    pub(crate) fn declaration_proof_site(
        &self,
        declaration: ImportedDeclarationId,
        site: DeclarationSite,
    ) -> Result<ProofSite, &'static str> {
        let site = self.declaration_site(declaration, site.index())?;
        Ok(ProofSite::Imported(ImportedProofSite {
            owner: declaration.0,
            ordinal: site.0,
        }))
    }
}
