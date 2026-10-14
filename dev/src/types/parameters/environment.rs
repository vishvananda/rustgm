//! Immutable assumptions for one symbolic body. Absence of a bound is not proof
//! that a concrete instantiation lacks it; it only withholds that operation here.
use super::predicates::{Bound, Predicate};
use super::{ParameterId, ParameterKind};
use crate::types::Store;
use crate::types::{Kind, TypeId};
use std::collections::{HashMap, hash_map::Entry};

/// Language-provided abilities used by move checking. Copy entails Clone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Abilities {
    pub copy: bool,
    pub clone_: bool,
}

/// Completed parameter scope and its assumptions. Query caches must retain the
/// same environment for their entire lifetime, including cached failures.
#[derive(Clone)]
pub struct Environment {
    parameters: HashMap<ParameterId, Abilities>,
    assumptions: Option<Box<Assumptions>>,
    shape_assumptions: bool,
}
#[derive(Clone, Default)]
struct Assumptions {
    predicates: HashMap<TypeId, Abilities>,
    sized: std::collections::HashSet<TypeId>,
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
        let mut shape_assumptions = false;
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
            shape_assumptions |= bounds.clone_;
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
        Ok(Self {
            parameters,
            assumptions: None,
            shape_assumptions,
        })
    }

    /// Freeze additional assumptions without deriving reverse implications from
    /// composite types: `(T,U): Copy` does not establish `T: Copy`.
    /// # Errors
    /// Rejects malformed subjects, foreign type parameters and excessive work.
    pub fn with_predicates(
        mut self,
        types: &Store,
        predicates: &[Predicate],
    ) -> Result<Self, &'static str> {
        if predicates.len() > 1_000_000 {
            return Err("parameter predicate budget exceeded");
        }
        let mut uses = super::uses::Uses::default();
        for predicate in predicates {
            uses.collect(types, predicate.ty)?;
            if matches!(predicate.bound, Bound::Outlives(_)) {
                continue;
            }
            let assumptions = self.assumptions.get_or_insert_with(Default::default);
            assumptions.sized.insert(predicate.ty);
            let properties = assumptions.predicates.entry(predicate.ty).or_default();
            match predicate.bound {
                Bound::Copy => {
                    properties.copy = true;
                    properties.clone_ = true;
                }
                Bound::Clone => properties.clone_ = true,
                Bound::Sized | Bound::Outlives(_) => {}
            }
            if let Some(Kind::Parameter(id)) = types.kind(predicate.ty) {
                let parameter = self
                    .parameters
                    .get_mut(&id)
                    .ok_or("bound parameter is outside its environment")?;
                parameter.copy |= properties.copy;
                parameter.clone_ |= properties.clone_;
            }
        }
        if uses
            .parameters
            .iter()
            .any(|id| !self.parameters.contains_key(id))
        {
            return Err("bound parameter is outside its environment");
        }
        Ok(self)
    }
    #[must_use]
    pub fn assumed(&self, ty: TypeId) -> Abilities {
        self.assumptions
            .as_ref()
            .and_then(|a| a.predicates.get(&ty))
            .copied()
            .unwrap_or_default()
    }
    #[must_use]
    pub fn assumes_sized(&self, ty: TypeId) -> bool {
        self.assumptions
            .as_ref()
            .is_some_and(|a| a.sized.contains(&ty))
    }
    #[must_use]
    pub fn has_shape_assumptions(&self) -> bool {
        self.shape_assumptions || self.assumptions.is_some()
    }

    /// `None` means the parameter is outside this scope, not an unbounded type.
    #[must_use]
    pub fn abilities(&self, parameter: ParameterId) -> Option<Abilities> {
        self.parameters.get(&parameter).copied()
    }
}
