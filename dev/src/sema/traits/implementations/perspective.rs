//! Coherence proves absence for the crate whose impls it compares.
use crate::{resolve::model::DefId, sema::model::Declarations};

/// The crate whose coherence a query proves. A consumer re-proves an imported
/// packet's own pair from that packet, and every other pair from itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sema) enum Perspective {
    Local,
    /// One origin-catalog artifact. A catalog holds at most
    /// `MAX_DEPENDENCIES` (1,024) artifacts, so 16 bits hold any ordinal; a
    /// larger one is an error, never a truncation.
    Packet(u16),
}

impl Perspective {
    /// Two impls imported from one artifact are that packet's pair. Any other
    /// pair, including one across two packets until a multi-packet profile
    /// records which is downstream, is proved for this crate, which owns the
    /// fewest of those packets' declarations.
    pub(in crate::sema) fn of(facts: &Declarations, owners: [DefId; 2]) -> Result<Self, String> {
        let [left, right] = owners;
        Ok(match (artifact(facts, left)?, artifact(facts, right)?) {
            (Some(left), Some(right)) if left == right => Self::Packet(
                u16::try_from(left).map_err(|_| "coherence perspective artifact overflow")?,
            ),
            _ => Self::Local,
        })
    }

    /// Whether this crate declares `definition`: one no artifact imported, or
    /// one this packet's own artifact did.
    pub(in crate::sema) fn owns(
        self,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<bool, String> {
        let artifact = artifact(facts, definition)?;
        Ok(match self {
            Self::Local => artifact.is_none(),
            Self::Packet(packet) => artifact == Some(u32::from(packet)),
        })
    }

    pub(in crate::sema) const fn packet(self) -> bool {
        matches!(self, Self::Packet(_))
    }
}

fn artifact(facts: &Declarations, definition: DefId) -> Result<Option<u32>, String> {
    let Some(catalog) = facts
        .crates
        .as_deref()
        .and_then(crate::sema::crates::Linkage::origin_catalog)
    else {
        return Ok(None);
    };
    catalog
        .declaration_artifact(definition)
        .map_err(str::to_string)
}
