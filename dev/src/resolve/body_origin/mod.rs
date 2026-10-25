//! Exact source/imported ownership is independent of diagnostic labels and proof.
mod catalog;
mod declarations;
mod locations;
mod publication;
use super::unit::NodeKey;
use crate::syntax::ast::NodeId;
pub(crate) use catalog::{Binding as BodyBinding, Catalog, CatalogSlot, Input, Installed, Limits};
pub(crate) use declarations::{
    DeclarationBinding, DeclarationOrigin, DeclarationSite, ImportedDeclarationId,
};
pub(crate) use locations::{ImportedDiagnostic, Label, LocationOwner, ProviderFile};
use std::{
    num::NonZeroU32,
    sync::atomic::{AtomicU32, Ordering},
};

static NEXT_CATALOG: AtomicU32 = AtomicU32::new(1);
#[derive(Clone, Copy, PartialEq, Eq)]
struct Epoch(NonZeroU32);
impl Epoch {
    fn allocate(counter: &AtomicU32) -> Result<Self, &'static str> {
        counter
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                NonZeroU32::new(value).and_then(|value| value.get().checked_add(1))
            })
            .ok()
            .and_then(NonZeroU32::new)
            .map(Self)
            .ok_or("imported body catalog identity space exhausted")
    }
}

/// Compiler-local rejection identity. Never serialize it or order symbols by it.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ImportedOwnerId {
    epoch: NonZeroU32,
    row: u32,
}
impl std::fmt::Debug for ImportedOwnerId {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str("ImportedOwnerId(..)")
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImportedBodyId(ImportedOwnerId);
impl std::fmt::Debug for ImportedBodyId {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str("ImportedBodyId(..)")
    }
}
/// Bounds-carrying location only; authorizing consumers still check the catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImportedBodyOrigin {
    id: ImportedBodyId,
    sites: NonZeroU32,
}
impl ImportedBodyOrigin {
    pub(crate) const fn id(self) -> ImportedBodyId {
        self.id
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BodyOrigin {
    Source(NodeKey),
    Imported(ImportedBodyOrigin),
}
impl BodyOrigin {
    #[must_use]
    pub const fn source(self) -> Option<NodeKey> {
        match self {
            Self::Source(source) => Some(source),
            Self::Imported(_) => None,
        }
    }
    pub(crate) const fn root_body_site(self) -> BodySite {
        match self {
            Self::Source(source) => BodySite(NonZeroU32::MIN.saturating_add(source.node.get())),
            Self::Imported(_) => BodySite(NonZeroU32::MIN),
        }
    }
    /// A real source root or the checked imported body's first retained site.
    #[must_use]
    pub const fn root_site(self) -> ProofSite {
        match self {
            Self::Source(source) => ProofSite::Source(source),
            Self::Imported(origin) => ProofSite::Imported(ImportedProofSite {
                owner: origin.id.0,
                ordinal: NonZeroU32::MIN,
            }),
        }
    }
    pub(crate) fn proof_site(self, site: BodySite) -> Result<ProofSite, &'static str> {
        match self {
            Self::Source(source) => Ok(ProofSite::Source(NodeKey {
                file: source.file,
                node: NodeId::checked(site.index())
                    .ok_or("source body site is not a syntax node")?,
            })),
            Self::Imported(origin) => {
                if site.index() >= origin.sites.get() {
                    return Err("operation site belongs outside its imported body");
                }
                Ok(ProofSite::Imported(ImportedProofSite {
                    owner: origin.id.0,
                    ordinal: site.0,
                }))
            }
        }
    }
    pub(crate) fn source_site(self, node: NodeId) -> Result<ProofSite, &'static str> {
        let mut source = self.source().ok_or("imported body has no syntax nodes")?;
        source.node = node;
        Ok(ProofSite::Source(source))
    }
}

/// An operation ordinal interpreted only together with its exact body origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BodySite(NonZeroU32);
impl BodySite {
    pub(crate) fn source(node: NodeId) -> Result<Self, &'static str> {
        Self::checked(node.get())
    }
    fn checked(index: u32) -> Result<Self, &'static str> {
        index
            .checked_add(1)
            .and_then(NonZeroU32::new)
            .map(Self)
            .ok_or("body operation site space exhausted")
    }
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0.get() - 1
    }
    #[must_use]
    pub const fn get(self) -> u32 {
        self.index()
    }
}
impl From<NodeId> for BodySite {
    fn from(node: NodeId) -> Self {
        // NodeId already stores index+1 in NonZeroU32, so this cannot saturate.
        Self(NonZeroU32::MIN.saturating_add(node.get()))
    }
}
impl From<NodeKey> for BodyOrigin {
    fn from(source: NodeKey) -> Self {
        Self::Source(source)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProofSite {
    Source(NodeKey),
    Imported(ImportedProofSite),
}
/// A checked location in the common owner table, not a body or proof handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImportedProofSite {
    owner: ImportedOwnerId,
    ordinal: NonZeroU32,
}
impl ProofSite {
    #[must_use]
    pub const fn source(self) -> Option<NodeKey> {
        match self {
            Self::Source(source) => Some(source),
            Self::Imported(_) => None,
        }
    }
}
impl From<BodyOrigin> for ProofSite {
    fn from(origin: BodyOrigin) -> Self {
        origin.root_site()
    }
}
impl From<NodeKey> for ProofSite {
    fn from(source: NodeKey) -> Self {
        Self::Source(source)
    }
}

pub(crate) use crate::support::edition::Edition;

#[cfg(test)]
#[path = "../../../tests/invariants/declaration_origins.rs"]
mod declaration_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/origin_layouts.rs"]
mod layout_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/transport_owners.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../tests/invariants/bounded_sites.rs"]
mod site_tests;

#[cfg(test)]
#[path = "../../../tests/invariants/location_errors.rs"]
mod diagnostic_tests;
