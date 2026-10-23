//! Declaration/clause ownership never requires a body or invented syntax node.
use super::{Catalog, Edition, Label, ProofSite, catalog::Artifact};
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
pub(crate) struct ImportedDeclarationId {
    epoch: NonZeroU32,
    definition: DefId,
}
impl std::fmt::Debug for ImportedDeclarationId {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str("ImportedDeclarationId(..)")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DeclarationOrigin {
    Source(NodeKey),
    Imported(ImportedDeclarationId),
}
impl DeclarationOrigin {
    pub(crate) const fn source(self) -> Option<NodeKey> {
        match self {
            Self::Source(source) => Some(source),
            Self::Imported(_) => None,
        }
    }
}
/// Clause/annotation ordinal in one declaration, distinct from a body operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct DeclarationSite(NonZeroU32);
impl DeclarationSite {
    fn checked(index: u32) -> Result<Self, &'static str> {
        index
            .checked_add(1)
            .and_then(NonZeroU32::new)
            .map(Self)
            .ok_or("declaration site space exhausted")
    }
    pub(crate) const fn index(self) -> u32 {
        self.0.get() - 1
    }
}
pub(crate) struct DeclarationBinding {
    pub kind: DefKind,
    pub sites: Range<u32>,
}
impl Catalog {
    pub(crate) fn declaration(
        &self,
        definition: DefId,
    ) -> Result<ImportedDeclarationId, &'static str> {
        self.charge(1)?;
        if !self.definitions.contains_key(&definition) {
            return Err("imported declaration is absent from this catalog");
        }
        Ok(ImportedDeclarationId {
            epoch: self.epoch.0,
            definition,
        })
    }
    fn declaration_row(&self, id: ImportedDeclarationId) -> Result<(&Artifact, u32), &'static str> {
        self.charge(1)?;
        if id.epoch != self.epoch.0 {
            return Err("imported declaration belongs to a foreign or retired catalog");
        }
        let &(artifact, ordinal) = self
            .definitions
            .get(&id.definition)
            .ok_or("imported declaration remap is missing or stale")?;
        Ok((&self.artifacts[artifact as usize], ordinal))
    }
    pub(crate) fn validate_declaration(
        &self,
        id: ImportedDeclarationId,
        definition: DefId,
        kind: DefKind,
    ) -> Result<super::DeclarationOrigin, &'static str> {
        let (artifact, ordinal) = self.declaration_row(id)?;
        if id.definition != definition || artifact.declarations[ordinal as usize].kind != kind {
            return Err("imported declaration changed its exact identity or kind");
        }
        Ok(DeclarationOrigin::Imported(id))
    }
    pub(crate) fn declaration_coordinate(
        &self,
        id: ImportedDeclarationId,
    ) -> Result<(&CrateIdentity, DeclarationRef, Edition), &'static str> {
        let (artifact, ordinal) = self.declaration_row(id)?;
        Ok((&artifact.owner, DeclarationRef(ordinal), artifact.edition))
    }
    pub(crate) fn declaration_site(
        &self,
        id: ImportedDeclarationId,
        index: u32,
    ) -> Result<DeclarationSite, &'static str> {
        let (artifact, ordinal) = self.declaration_row(id)?;
        self.charge(1)?;
        let sites = &artifact.declarations[ordinal as usize].sites;
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
        Ok(ProofSite::Declaration { declaration, site })
    }
    pub(super) fn declaration_label(
        &self,
        id: ImportedDeclarationId,
        site: DeclarationSite,
    ) -> Result<(&Artifact, Option<Label>), &'static str> {
        let (artifact, ordinal) = self.declaration_row(id)?;
        self.charge(1)?;
        let sites = &artifact.declarations[ordinal as usize].sites;
        if site.index() >= sites.end - sites.start {
            return Err("diagnostic site belongs outside its imported declaration");
        }
        Ok((
            artifact,
            artifact.declaration_labels[(sites.start + site.index()) as usize],
        ))
    }
}
