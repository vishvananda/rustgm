//! Immutable assumptions for one symbolic body. Absence of a bound is not proof
//! that a concrete instantiation lacks it; it only withholds that operation here.
use super::{ParameterId, ParameterKind};
use crate::types::Store;
use std::collections::{HashMap, hash_map::Entry};

/// Language-provided abilities used by move checking. Copy entails Clone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Abilities {
    pub copy: bool,
    pub clone_: bool,
}

/// Completed parameter scope and its assumptions. Query caches must retain the
/// same environment for their entire lifetime, including cached failures.
pub struct Environment {
    parameters: HashMap<ParameterId, Abilities>,
}
impl Environment {
    /// Include every type parameter in scope, even when it has no bounds.
    /// Repeated assumptions are combined; parameter identity includes its owner.
    ///
    /// # Errors
    /// Rejects unknown/const parameters and excessive input before allocating.
    pub fn new(
        types: &Store,
        assumptions: &[(ParameterId, Abilities)],
    ) -> Result<Self, &'static str> {
        if assumptions.len() > 1_000_000 {
            return Err("parameter environment storage budget exceeded");
        }
        let mut parameters: HashMap<ParameterId, Abilities> = HashMap::new();
        for (id, bounds) in assumptions {
            if !matches!(
                types.parameter(*id).map(|p| p.kind),
                Some(ParameterKind::Type { .. })
            ) {
                return Err("parameter environment requires declared type parameters");
            }
            let bounds = Abilities {
                copy: bounds.copy,
                clone_: bounds.clone_ || bounds.copy,
            };
            match parameters.entry(*id) {
                Entry::Vacant(entry) => {
                    entry.insert(bounds);
                }
                Entry::Occupied(mut entry) => {
                    let existing = entry.get_mut();
                    existing.copy |= bounds.copy;
                    existing.clone_ |= bounds.clone_;
                }
            }
        }
        Ok(Self { parameters })
    }

    /// `None` means the parameter is outside this scope, not an unbounded type.
    #[must_use]
    pub fn abilities(&self, parameter: ParameterId) -> Option<Abilities> {
        self.parameters.get(&parameter).copied()
    }
}
