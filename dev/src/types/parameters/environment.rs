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
    traits: Option<Box<TraitAssumptions>>,
    shape_assumptions: bool,
}
#[derive(Clone, Default)]
struct Assumptions {
    predicates: HashMap<TypeId, Abilities>,
    sized: std::collections::HashSet<TypeId>,
}
#[derive(Clone, Default)]
struct TraitAssumptions {
    predicates: Vec<Predicate>,
    unique: std::collections::HashSet<Predicate>,
    constants: std::collections::HashSet<ParameterId>,
}
impl Environment {
    /// Parameter identities and abilities contain no regions to substitute.
    /// Composite and trait predicates still need the ordinary binder mapping.
    pub(crate) fn has_only_parameters(&self) -> bool {
        self.assumptions.is_none()
            && self
                .traits
                .as_ref()
                .is_none_or(|traits| traits.predicates.is_empty())
    }

    /// Preserve the same immutable permissions while replacing their captured
    /// lifetime binder. Inference consumes free aliases, including in callables.
    pub(crate) fn substitute_regions(
        &self,
        types: &mut Store,
        fold: &mut crate::types::substitute::Bound,
    ) -> Result<Self, &'static str> {
        let mut result = self.clone();
        if let Some(assumptions) = &mut result.assumptions {
            // Substitution may allocate canonical types or open lifetimes. Its
            // encounter order cannot depend on a hash table's randomized seed.
            let mut predicates: Vec<_> = assumptions.predicates.drain().collect();
            predicates.sort_unstable_by_key(|(ty, _)| ty.0);
            assumptions.predicates = predicates
                .into_iter()
                .map(|(ty, abilities)| Ok((fold.apply(types, ty)?, abilities)))
                .collect::<Result<_, &'static str>>()?;
            let mut sized: Vec<_> = assumptions.sized.drain().collect();
            sized.sort_unstable_by_key(|ty| ty.0);
            assumptions.sized = sized
                .into_iter()
                .map(|ty| fold.apply(types, ty))
                .collect::<Result<_, _>>()?;
        }
        if let Some(traits) = &mut result.traits {
            for predicate in &mut traits.predicates {
                *predicate = fold.predicate(types, *predicate)?;
            }
            traits.unique = traits.predicates.iter().copied().collect();
        }
        Ok(result)
    }

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
            traits: None,
            shape_assumptions,
        })
    }

    /// Const parameters may occur in a trait application without granting a type ability.
    /// # Errors
    /// Rejects unknown/type parameters and excessive input before retaining identities.
    pub fn with_const_parameters(
        mut self,
        types: &Store,
        parameters: &[ParameterId],
    ) -> Result<Self, &'static str> {
        if parameters.len() > 1_000_000 {
            return Err("const parameter environment storage budget exceeded");
        }
        for &id in parameters {
            if !matches!(
                types.parameter(id).map(|p| p.kind),
                Some(ParameterKind::Const { .. })
            ) {
                return Err("const parameter environment requires declared const parameters");
            }
            let traits = self.traits.get_or_insert_with(Default::default);
            if traits.constants.len() >= 1_000_000 && !traits.constants.contains(&id) {
                return Err("const parameter environment storage budget exceeded");
            }
            traits.constants.insert(id);
        }
        Ok(self)
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
            uses.predicate(types, *predicate)?;
            if matches!(predicate.bound, Bound::Trait(_)) {
                let traits = self.traits.get_or_insert_with(Default::default);
                if traits.unique.len() >= 1_000_000 && !traits.unique.contains(predicate) {
                    return Err("trait assumption storage budget exceeded");
                }
                if traits.unique.insert(*predicate) {
                    traits.predicates.push(*predicate);
                }
                continue;
            }
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
                Bound::Sized | Bound::Outlives(_) | Bound::Trait(_) => {}
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
        if uses.constants.iter().any(|id| {
            !self
                .traits
                .as_ref()
                .is_some_and(|traits| traits.constants.contains(id))
        }) {
            return Err("bound const parameter is outside its environment");
        }
        Ok(self)
    }
    /// Exact retained assumptions, in declaration order. Supertrait elaboration
    /// adds its own proved predicates rather than granting arbitrary type abilities.
    pub fn trait_predicates(&self) -> impl Iterator<Item = &Predicate> {
        self.traits.iter().flat_map(|traits| &traits.predicates)
    }
    #[must_use]
    pub fn assumes_trait(
        &self,
        ty: TypeId,
        application: crate::types::traits::ApplicationId,
    ) -> bool {
        self.traits.as_ref().is_some_and(|traits| {
            traits.unique.contains(&Predicate {
                ty,
                bound: Bound::Trait(application),
            })
        })
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
