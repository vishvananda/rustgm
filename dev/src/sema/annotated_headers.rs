//! Checked source and canonical imported header facts share diagnostic consumers.
#[path = "header_attributes/metadata_contracts.rs"]
mod imported;
#[path = "header_attributes/declared_origins.rs"]
mod local;
#[cfg(test)]
#[path = "../../tests/invariants/transported_annotations.rs"]
mod tests;
use crate::{
    resolve::{
        body_origin::{DeclarationSite, ImportedDeclarationId},
        expansion::Report,
        model::DefId,
        package::Error,
        unit::NodeKey,
    },
    sema::{inlining, model::Declarations, must_use::ContractView},
    support::{
        crates::{CrateIdentity, ExternalDef, owner_catalog::DeclarationRef},
        edition::Edition,
    },
    syntax::token_tree::Span,
};

#[derive(Clone, Copy)]
pub struct SourceContractSite {
    pub owner: NodeKey,
    pub declaration: NodeKey,
    pub span: Span,
}
#[derive(Clone, Copy)]
pub struct ImportedContractSite<'a> {
    coordinate: (&'a CrateIdentity, DeclarationRef, Edition),
    declaration: ImportedDeclarationId,
    site: Option<DeclarationSite>,
    external: Option<&'a ExternalDef>,
}
impl<'a> ImportedContractSite<'a> {
    #[must_use]
    pub const fn coordinate(self) -> (&'a CrateIdentity, DeclarationRef, Edition) {
        self.coordinate
    }
    #[must_use]
    pub const fn declaration(self) -> ImportedDeclarationId {
        self.declaration
    }
    #[must_use]
    pub const fn site(self) -> Option<DeclarationSite> {
        self.site
    }
    #[must_use]
    pub const fn external_definition(self) -> Option<&'a ExternalDef> {
        self.external
    }
}
#[derive(Clone, Copy)]
enum Site<'a> {
    Source(SourceContractSite),
    Imported(ImportedContractSite<'a>),
}
/// Constructors stay inside semantic producers; a source-free origin cannot
/// masquerade as an AST owner. This view grants no execution authority.
#[derive(Clone, Copy)]
pub struct ContractOrigin<'a>(Site<'a>);
impl ContractOrigin<'_> {
    pub(in crate::sema) const fn source_site(
        owner: NodeKey,
        declaration: NodeKey,
        span: Span,
    ) -> Self {
        Self(Site::Source(SourceContractSite {
            owner,
            declaration,
            span,
        }))
    }
    #[must_use]
    pub const fn source(self) -> Option<SourceContractSite> {
        match self.0 {
            Site::Source(site) => Some(site),
            Site::Imported(_) => None,
        }
    }
}
impl<'a> ContractOrigin<'a> {
    #[must_use]
    pub const fn imported(self) -> Option<ImportedContractSite<'a>> {
        match self.0 {
            Site::Imported(site) => Some(site),
            Site::Source(_) => None,
        }
    }
}
pub enum ContractKnowledge<T> {
    Unknown,
    KnownAbsent,
    KnownPresent(T),
}
pub struct HeaderAttributeView<'a> {
    pub origin: ContractOrigin<'a>,
    pub must_use: ContractKnowledge<ContractView<'a>>,
    pub inline: ContractKnowledge<inlining::View<'a>>,
}
#[derive(Default)]
pub(in crate::sema) struct Ledger {
    local: local::Headers,
    imported: Option<Box<imported::Headers>>,
}
impl Declarations {
    /// Complete header facts require positive producer state. Legacy imported
    /// rows and missing producers remain Unknown rather than checked absence.
    /// # Errors
    /// Rejects stale source incarnations, foreign catalogs and changed headers.
    pub fn header_attribute_contracts<'a>(
        &'a self,
        report: &Report,
        definition: DefId,
    ) -> Result<HeaderAttributeView<'a>, Error> {
        let actual = report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| Error::unlocated("unknown header attribute declaration"))?;
        if actual.node.is_none() {
            return imported::query(self, report, definition, actual.kind);
        }
        local::query(self, report, definition)
    }
}
pub(super) use imported::prepare as prepare_imported;
pub(super) use local::{bind_definition, finish, observe_owner};
pub(super) fn has_imported(facts: &Declarations, definition: DefId) -> bool {
    facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .and_then(|ledger| ledger.imported.as_deref())
        .is_some_and(|headers| headers.contains(definition))
}
