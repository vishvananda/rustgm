//! Immutable assumptions for one symbolic body. Absence of a bound is not proof
//! that a concrete instantiation lacks it; it only withholds that operation here.
mod equalities;
mod owned_payload;
pub(crate) use owned_payload::Retained;
mod instantiation;
mod permissions;
mod premises;
pub(crate) use permissions::Query as BuiltinQuery;
#[cfg(test)]
#[path = "../../../tests/invariants/builtin_permissions.rs"]
mod builtin_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/premise_scopes.rs"]
mod scope_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/mapped_environment_sharing.rs"]
mod sharing_tests;
use super::{ParameterId, ParameterKind};
use super::{
    clauses::Clause,
    predicates::{Bound, Predicate},
};
use crate::support::hash::HashMap;
use crate::types::Store;
use crate::types::{Kind, TypeId};
use premises::TraitAssumptions;
use std::collections::hash_map::Entry;
use std::sync::Arc;

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
    parameters: Arc<HashMap<ParameterId, Abilities>>,
    assumptions: Option<Arc<Assumptions>>,
    builtins: Option<Arc<permissions::Index>>,
    regions: Option<Arc<crate::types::relations::RegionEqualities>>,
    traits: Option<Arc<TraitAssumptions>>,
    equalities: Option<Arc<equalities::Equalities>>,
    projection_promises:
        Option<std::sync::Arc<HashMap<crate::types::ItemId, crate::types::projection::Builtins>>>,
    shape_assumptions: bool,
    region_mapping: bool,
}
#[derive(Clone, Default)]
struct Assumptions {
    predicates: HashMap<TypeId, Abilities>,
    sized: crate::support::hash::HashSet<TypeId>,
    region_mapping: bool,
}
impl Environment {
    pub(crate) fn with_projection_promises(
        mut self,
        promises: Option<
            std::sync::Arc<HashMap<crate::types::ItemId, crate::types::projection::Builtins>>,
        >,
    ) -> Self {
        self.projection_promises = promises;
        self
    }
    /// Declaration metadata supplies no evidence for forming an application.
    pub(crate) fn projection_promises(
        &self,
        item: crate::types::ItemId,
    ) -> crate::types::projection::Builtins {
        self.projection_promises
            .as_ref()
            .and_then(|promises| promises.get(&item))
            .copied()
            .unwrap_or_default()
    }
    /// A false value certifies no outer-region mapping work. The store's region
    /// summary conservatively includes static lifetimes as free regions.
    pub(crate) const fn requires_region_mapping(&self) -> bool {
        self.region_mapping
    }

    /// Parameter identities and abilities contain no regions to substitute.
    /// Composite and trait predicates still need the ordinary binder mapping.
    pub(crate) fn has_only_parameters(&self) -> bool {
        self.assumptions.is_none()
            && self.builtins.is_none()
            && self.regions.is_none()
            && self.equalities.is_none()
            && self.traits.as_ref().is_none_or(|traits| traits.is_empty())
    }

    /// Preserve the same immutable permissions while replacing their captured
    /// lifetime binder. Inference consumes free aliases, including in callables.
    pub(crate) fn substitute_regions(
        &self,
        types: &mut Store,
        fold: &mut crate::types::substitute::Bound,
    ) -> Result<Self, &'static str> {
        let mut result = self.clone();
        if let Some(assumptions) = &mut result.assumptions
            && (assumptions.region_mapping || !fold.regions_only())
        {
            let assumptions = Arc::make_mut(assumptions);
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
            assumptions.region_mapping = false;
            for &ty in assumptions
                .sized
                .iter()
                .chain(assumptions.predicates.keys())
            {
                assumptions.region_mapping |= types
                    .has_free_regions(ty, 0)
                    .ok_or("invalid mapped bound subject")?;
            }
        }
        if let Some(traits) = &mut result.traits
            && (traits.region_mapping || !fold.regions_only())
        {
            Arc::make_mut(traits).substitute(types, fold)?;
        }
        if let Some(builtins) = &result.builtins
            && (builtins.region_mapping || !fold.regions_only())
        {
            result.builtins = Some(Arc::new(builtins.substitute(types, fold)?));
        }
        if let Some(regions) = &result.regions {
            result.regions = Some(Arc::new(regions.substitute(fold)?));
        }
        if let Some(equalities) = &mut result.equalities
            && (equalities.region_mapping || !fold.regions_only())
        {
            Arc::make_mut(equalities).substitute(types, fold)?;
        }
        result.region_mapping = result.mapped_regions();
        Ok(result)
    }

    fn mapped_regions(&self) -> bool {
        self.regions.is_some()
            || self.traits.as_ref().is_some_and(|part| part.region_mapping)
            || self
                .builtins
                .as_ref()
                .is_some_and(|part| part.region_mapping)
            || self
                .equalities
                .as_ref()
                .is_some_and(|part| part.region_mapping)
            || self
                .assumptions
                .as_ref()
                .is_some_and(|part| part.region_mapping)
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
        let mut parameters: HashMap<ParameterId, Abilities> = HashMap::default();
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
            parameters: Arc::new(parameters),
            assumptions: None,
            builtins: None,
            regions: None,
            traits: None,
            equalities: None,
            projection_promises: None,
            shape_assumptions,
            region_mapping: false,
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
            let traits = Arc::make_mut(self.traits.get_or_insert_with(Default::default));
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
        self,
        types: &Store,
        predicates: &[Predicate],
    ) -> Result<Self, &'static str> {
        self.with_requirements(types, predicates.iter().copied().map(Clause::Predicate))
    }

    /// Retain whole quantified trait and builtin assumptions beside ordinary predicates.
    /// Lifetime clauses validate membership here; the caller retains their whole
    /// scopes for its separate directed lifetime proof, as for ordinary outlives.
    /// # Errors
    /// Rejects unsupported quantified bounds, malformed or foreign identities,
    /// and exhausted storage or traversal limits before publishing an environment.
    pub fn with_clauses(self, types: &Store, clauses: &[Clause]) -> Result<Self, &'static str> {
        self.with_requirements(types, clauses.iter().copied())
    }

    fn with_requirements(
        mut self,
        types: &Store,
        clauses: impl ExactSizeIterator<Item = Clause>,
    ) -> Result<Self, &'static str> {
        if clauses.len() > 1_000_000 {
            return Err("parameter predicate budget exceeded");
        }
        let mut uses = super::uses::Uses::default();
        for clause in clauses {
            match clause {
                Clause::Predicate(predicate) => {
                    uses.predicate(types, predicate)?;
                    if !matches!(predicate.bound, Bound::Outlives(_)) {
                        self.region_mapping |= predicate.has_free_regions(types)?;
                    }
                    self.retain_predicate(types, predicate)?;
                }
                Clause::ForAll(id) => {
                    let value = types
                        .quantified(id)
                        .ok_or("invalid quantified assumption")?;
                    // Quantification binds lifetimes only; type/const membership
                    // still uses the shared occurrence walk for the whole batch.
                    uses.predicate(types, value.predicate)?;
                    if matches!(value.predicate.bound, Bound::Outlives(_)) {
                        continue;
                    }
                    self.region_mapping |= clause.has_free_regions(types)?;
                    if matches!(
                        value.predicate.bound,
                        Bound::Copy | Bound::Clone | Bound::Sized | Bound::Auto(_)
                    ) {
                        Arc::make_mut(self.builtins.get_or_insert_with(Default::default))
                            .insert(types, clause)?;
                        continue;
                    }
                    if matches!(value.predicate.bound, Bound::ProjectionEqual(_)) {
                        Arc::make_mut(self.equalities.get_or_insert_with(Default::default))
                            .insert(types, clause)?;
                    } else {
                        Arc::make_mut(self.traits.get_or_insert_with(Default::default))
                            .insert(types, clause)?;
                    }
                }
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
    fn retain_predicate(
        &mut self,
        types: &Store,
        predicate: Predicate,
    ) -> Result<(), &'static str> {
        if matches!(predicate.bound, Bound::ProjectionEqual(_)) {
            return Arc::make_mut(self.equalities.get_or_insert_with(Default::default))
                .insert(types, Clause::Predicate(predicate));
        }
        if matches!(predicate.bound, Bound::Trait(_)) {
            return Arc::make_mut(self.traits.get_or_insert_with(Default::default))
                .insert(types, Clause::Predicate(predicate));
        }
        if matches!(predicate.bound, Bound::Outlives(_)) {
            return Ok(());
        }
        if matches!(predicate.bound, Bound::Auto(_)) {
            return Arc::make_mut(self.builtins.get_or_insert_with(Default::default))
                .insert(types, predicate.into());
        }
        if types
            .has_free_regions(predicate.ty, 0)
            .ok_or("invalid builtin bound subject")?
        {
            Arc::make_mut(self.builtins.get_or_insert_with(Default::default))
                .insert(types, predicate.into())?;
        }
        let assumptions = Arc::make_mut(self.assumptions.get_or_insert_with(Default::default));
        assumptions.region_mapping |= types
            .has_free_regions(predicate.ty, 0)
            .ok_or("invalid assumed bound subject")?;
        assumptions.sized.insert(predicate.ty);
        let properties = assumptions.predicates.entry(predicate.ty).or_default();
        match predicate.bound {
            Bound::Copy => {
                properties.copy = true;
                properties.clone_ = true;
            }
            Bound::Clone => properties.clone_ = true,
            Bound::Sized
            | Bound::Auto(_)
            | Bound::Outlives(_)
            | Bound::Trait(_)
            | Bound::ProjectionEqual(_) => {}
        }
        if let Some(Kind::Parameter(id)) = types.kind(predicate.ty) {
            let parameter = Arc::make_mut(&mut self.parameters)
                .get_mut(&id)
                .ok_or("bound parameter is outside its environment")?;
            parameter.copy |= properties.copy;
            parameter.clone_ |= properties.clone_;
        }
        Ok(())
    }

    /// Share structural lookup data; inference and proof remain query-local.
    pub(crate) fn trait_index(
        &self,
        types: &Store,
    ) -> Result<(Option<Arc<super::premise_index::ClauseIndex>>, usize), &'static str> {
        self.traits
            .as_ref()
            .filter(|traits| !traits.is_empty())
            .map_or(Ok((None, 0)), |traits| {
                traits.index(types).map(|(index, work)| (Some(index), work))
            })
    }

    /// Exact retained clauses, in declaration order. No implicit binder opening
    /// or filtering can turn a quantified assumption into ordinary evidence.
    pub fn trait_clauses(&self) -> impl Iterator<Item = Clause> {
        self.traits.iter().flat_map(|traits| traits.clauses())
    }
    /// Coarse projected-subject buckets preserve every competing caller clause.
    /// Only complete canonical identity can establish a match without equations.
    pub(crate) fn projected_trait_candidates(
        &self,
        trait_item: crate::types::ItemId,
        projection: crate::types::ItemId,
    ) -> impl Iterator<Item = Clause> {
        self.traits
            .iter()
            .flat_map(move |traits| traits.projected((trait_item, projection)))
    }
    /// Retain all written equalities, including competing results. An unused
    /// generic declaration need not establish a unique normal form for them.
    pub fn projection_clauses(&self) -> impl Iterator<Item = Clause> {
        self.equalities
            .iter()
            .flat_map(|equalities| equalities.clauses())
    }
    /// Complete input equations distinguish candidates within this member bucket.
    pub(crate) fn projection_candidates(
        &self,
        item: crate::types::ItemId,
    ) -> impl Iterator<Item = Clause> {
        self.equalities
            .iter()
            .flat_map(move |equalities| equalities.candidates(item))
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
        self.shape_assumptions
            || self.assumptions.is_some()
            || self.builtins.is_some()
            || self.projection_promises.is_some()
    }

    /// `None` means the parameter is outside this scope, not an unbounded type.
    #[must_use]
    pub fn abilities(&self, parameter: ParameterId) -> Option<Abilities> {
        self.parameters.get(&parameter).copied()
    }
}

#[cfg(test)]
mod equality_tests {
    use super::*;
    use crate::types::{GenericArgs, ItemId, RegionId, projection, substitute, traits};

    #[test]
    fn opening_an_environment_maps_equality_results_and_preserves_the_original()
    -> Result<(), &'static str> {
        let mut store = Store::default();
        store.declare_projection(
            ItemId(2),
            projection::Schema {
                trait_item: ItemId(1),
                sized: false,
                copies: crate::types::copy::Properties::default(),
            },
        )?;
        let trait_application = store.intern_trait_application(traits::Application {
            item: ItemId(1),
            arguments: GenericArgs::EMPTY,
        })?;
        let region = RegionId::bound(0, 0)?;
        let result = store.intern(Kind::Reference {
            mutable: false,
            region,
            referent: TypeId::UNIT,
        })?;
        let ty = store.intern(Kind::Projection(projection::Application {
            item: ItemId(2),
            subject: TypeId::UNIT,
            trait_application,
            arguments: GenericArgs::EMPTY,
        }))?;
        let predicate = Predicate {
            ty,
            bound: Bound::ProjectionEqual(result),
        };
        let environment = Environment::new(&store, &[])?.with_predicates(&store, &[predicate])?;
        assert!(!environment.has_only_parameters());
        assert!(environment.requires_region_mapping());
        let mut fold = substitute::Bound::open_regions(7)?;
        let opened = environment.substitute_regions(&mut store, &mut fold)?;
        assert!(opened.requires_region_mapping());
        let expected = store.intern(Kind::Reference {
            mutable: false,
            region: RegionId(7),
            referent: TypeId::UNIT,
        })?;
        assert_eq!(
            opened
                .projection_clauses()
                .map(Clause::ordinary)
                .collect::<Result<Vec<_>, _>>()?,
            [Predicate {
                ty,
                bound: Bound::ProjectionEqual(expected)
            }]
        );
        assert_eq!(
            environment
                .projection_clauses()
                .map(Clause::ordinary)
                .collect::<Result<Vec<_>, _>>()?,
            [predicate]
        );
        assert_eq!(
            fold.opened_regions().collect::<Vec<_>>(),
            [(region, RegionId(7))]
        );
        assert!(!opened.assumes_sized(ty));
        assert!(!opened.has_shape_assumptions());
        let static_environment = environment.substitute_regions(
            &mut store,
            &mut substitute::Bound::new(vec![RegionId::STATIC]),
        )?;
        let static_predicate = static_environment
            .projection_clauses()
            .next()
            .ok_or("missing static equality assumption")?
            .ordinary()?;
        let static_result = store.intern(Kind::Reference {
            mutable: false,
            region: RegionId::STATIC,
            referent: TypeId::UNIT,
        })?;
        assert_eq!(
            static_predicate.bound,
            Bound::ProjectionEqual(static_result)
        );
        let closed = Environment::new(&store, &[])?.with_predicates(
            &store,
            &[Predicate {
                ty,
                bound: Bound::ProjectionEqual(TypeId::UNIT),
            }],
        )?;
        assert!(!closed.requires_region_mapping());
        let both = environment.with_predicates(&store, &[static_predicate])?;
        let merged = both.substitute_regions(
            &mut store,
            &mut substitute::Bound::new(vec![RegionId::STATIC]),
        )?;
        assert_eq!(merged.projection_candidates(ItemId(2)).count(), 1);
        assert_eq!(merged.projection_candidates(ItemId(3)).count(), 0);
        assert_eq!(merged.projection_clauses().count(), 1);
        Ok(())
    }
}
