//! Selected input equalities restore a symbolic application before opaque publication.
use crate::types::{Store, TypeId, lifetime_proof::Obligations, substitute::Alias};

pub(super) fn restore(
    types: &mut Store,
    proof: &Obligations,
    alias: Alias,
    work: &mut usize,
) -> Result<Option<TypeId>, &'static str> {
    if alias.associated().is_none() || proof.equalities.is_empty() {
        return Ok(None);
    }
    let original = types.intern(alias.kind())?;
    let mut restore = proof.restoration_counted(work)?;
    let result = restore.apply(types, original);
    *work += restore.steps;
    let restored = result?;
    // An unchanged application stays opaque, terminating the normalizer's
    // ordinary replacement traversal. The selected equalities remain debts.
    Ok((restored != original).then_some(restored))
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/application_anchors.rs"]
mod tests;
