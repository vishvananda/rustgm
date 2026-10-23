//! Exact source/imported ownership is independent of diagnostic labels and proof.
mod catalog;
mod declarations;
mod locations;
mod publication;
use super::unit::NodeKey;
use crate::syntax::ast::NodeId;
pub(crate) use catalog::{Catalog, Input, Installed};
pub(crate) use declarations::{
    DeclarationBinding, DeclarationOrigin, DeclarationSite, ImportedDeclarationId,
};
pub(crate) use locations::{Label, ProviderFile};
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
pub(crate) struct ImportedBodyId(ImportedOwnerId);
impl std::fmt::Debug for ImportedBodyId {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str("ImportedBodyId(..)")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BodyOrigin {
    Source(NodeKey),
    Imported(ImportedBodyId),
}
impl BodyOrigin {
    pub(crate) const fn source(self) -> Option<NodeKey> {
        match self {
            Self::Source(source) => Some(source),
            Self::Imported(_) => None,
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
pub(crate) struct BodySite(NonZeroU32);
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
    pub(crate) const fn index(self) -> u32 {
        self.0.get() - 1
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ProofSite {
    Source(NodeKey),
    Imported(ImportedProofSite),
}
/// A checked location in the common owner table, not a body or proof handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ImportedProofSite {
    owner: ImportedOwnerId,
    ordinal: NonZeroU32,
}
impl ProofSite {
    pub(crate) const fn source(self) -> Option<NodeKey> {
        match self {
            Self::Source(source) => Some(source),
            Self::Imported(_) => None,
        }
    }
}

/// Defining semantics are explicit; no wire/edition tag is assigned here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Edition {
    Rust2021,
    Rust2024,
}

#[cfg(test)]
#[path = "../../../tests/invariants/declaration_origins.rs"]
mod declaration_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/origin_layouts.rs"]
mod layout_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/transport_owners.rs"]
mod tests;
