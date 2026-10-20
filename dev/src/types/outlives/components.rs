//! Bounded component walks distinguish symbolic result bounds from runtime inputs.
#[cfg(test)]
#[path = "../../../tests/invariants/caller_component_work.rs"]
mod tests;
use super::{
    Argument, BoundParts, Budget, ComponentLists, HashMap, HashSet, Kind, ParameterId, Parts,
    Projection, RegionId, Store, TypeId, children, free,
};
#[derive(Default)]
pub(super) struct Components {
    cache: HashMap<(TypeId, u16, ComponentKind), Vec<RegionId>>,
    parameters: HashMap<(TypeId, u16, ComponentKind), Vec<ParameterId>>,
    projections: HashMap<(TypeId, u16, ComponentKind), Vec<Projection>>,
    captures: HashMap<u16, crate::types::bound_uses::captures::Captures>,
    walk: Walk,
}
/// Incremental component traversal for roots with one target and component policy.
#[derive(Default)]
pub(super) struct Walk {
    work: Vec<(TypeId, u16)>,
    seen: HashSet<(TypeId, u16)>,
    regions: HashSet<RegionId>,
    parameter_seen: HashSet<ParameterId>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum ComponentKind {
    /// Minimal components, under separately required reference well-formedness.
    Outlives,
    /// Explicit type bounds include every referent component, even in assumptions.
    TypeBound,
    LiveValue,
    /// Custom destruction retains all type arguments, including zero-length arrays.
    DropInputs,
    ProjectionInputs,
}
/// All components a type bound can constrain, including nested referents and
/// zero-length element types. Unlike owned-drop queries, Copy removes nothing.
#[derive(Default)]
pub(crate) struct TypeComponents {
    components: Components,
    budget: Budget,
}
impl TypeComponents {
    /// Collect independent parts of several already-normalized clause subjects.
    /// Shared tails are visited once for this common target, without constructing
    /// a synthetic type or caching a flattened component vector for every prefix.
    pub(crate) fn clause_roots(
        &mut self,
        types: &Store,
        roots: &[TypeId],
    ) -> Result<(Vec<RegionId>, Vec<ParameterId>), &'static str> {
        self.budget.work(roots.len())?;
        for &root in roots {
            if types.contains_projections(root) != Some(false) {
                return Err("concrete caller premise retains an associated projection");
            }
        }
        let (regions, parameters, projections) = self.components.collect_roots(
            types,
            roots,
            1,
            ComponentKind::TypeBound,
            &mut self.budget,
        )?;
        if !projections.is_empty() {
            return Err("concrete caller premise retains an associated projection");
        }
        Ok((regions, parameters))
    }

    /// Every input remaining live is sufficient for an opaque result's cleanup.
    /// This gives no reverse bound on inputs from a bound on the result.
    pub(crate) fn live_value(
        &mut self,
        types: &Store,
        ty: TypeId,
    ) -> Result<Parts<'_>, &'static str> {
        self.components
            .get(types, ty, 0, ComponentKind::LiveValue, &mut self.budget)?
            .scalar_only()
    }
    pub(crate) const fn steps(&self) -> usize {
        self.budget.0
    }
    pub(crate) fn captures(
        &mut self,
        types: &Store,
        projection: Projection,
    ) -> Result<bool, &'static str> {
        self.components
            .captures(types, projection.ty, projection.depth, &mut self.budget)
    }

    pub(crate) fn projection_inputs(
        &mut self,
        types: &Store,
        projection: Projection,
    ) -> Result<BoundParts<'_>, &'static str> {
        self.components.get(
            types,
            projection.ty,
            projection.depth,
            ComponentKind::ProjectionInputs,
            &mut self.budget,
        )
    }

    pub(crate) fn get(&mut self, types: &Store, ty: TypeId) -> Result<Parts<'_>, &'static str> {
        self.components
            .get(types, ty, 0, ComponentKind::TypeBound, &mut self.budget)?
            .scalar_only()
    }
    pub(crate) fn drop_inputs(
        &mut self,
        types: &Store,
        ty: TypeId,
    ) -> Result<Parts<'_>, &'static str> {
        self.components
            .get(types, ty, 0, ComponentKind::DropInputs, &mut self.budget)?
            .scalar_only()
    }
}
/// Cached runtime components, including nested referents and opaque type parameters.
/// Zero-length arrays contain no runtime reference values. Type well-formedness
/// still checks their element types independently through `Queries`.
#[derive(Default)]
pub struct LiveRegions {
    components: Components,
    budget: Budget,
}
impl LiveRegions {
    /// Queries belong to one immutable canonical type store.
    ///
    /// # Errors
    /// Rejects malformed types, binder overflow and bounded traversal exhaustion.
    pub fn get(&mut self, types: &Store, ty: TypeId) -> Result<&[RegionId], &'static str> {
        if types.contains_parameters(ty) == Some(true)
            || types.contains_projections(ty) == Some(true)
        {
            return Err("generic live-region checking requires instantiated value facts");
        }
        self.symbolic(types, ty).map(|parts| parts.regions)
    }
    /// Retain opaque type-parameter components alongside named lifetimes. The
    /// caller must resolve both kinds in the current body's rigid environment.
    /// This structural cache depends only on the immutable canonical type store.
    /// # Errors
    /// Rejects malformed types, binder overflow and bounded traversal exhaustion.
    pub fn symbolic(&mut self, types: &Store, ty: TypeId) -> Result<Parts<'_>, &'static str> {
        self.components
            .get(types, ty, 0, ComponentKind::LiveValue, &mut self.budget)?
            .scalar_only()
    }
    #[must_use]
    pub const fn steps(&self) -> usize {
        self.budget.0
    }
}
impl Components {
    pub(super) fn get(
        &mut self,
        types: &Store,
        root: TypeId,
        depth: u16,
        kind: ComponentKind,
        budget: &mut Budget,
    ) -> Result<BoundParts<'_>, &'static str> {
        if types.contains_regions(root) == Some(false)
            && types.contains_parameters(root) == Some(false)
            && types.contains_projections(root) == Some(false)
        {
            return Ok(BoundParts {
                scalars: Parts {
                    regions: &[],
                    parameters: &[],
                },
                projections: &[],
            });
        }
        let key = (root, depth, kind);
        if !self.cache.contains_key(&key) {
            let (values, parameters, projections) =
                self.collect(types, root, depth, kind, budget)?;
            if !projections.is_empty() {
                self.projections.insert(key, projections);
            }
            self.cache.insert(key, values);
            if !parameters.is_empty() {
                self.parameters.insert(key, parameters);
            }
        }
        let regions = self
            .cache
            .get(&key)
            .map(Vec::as_slice)
            .ok_or("missing outlives component cache")?;
        Ok(BoundParts {
            scalars: Parts {
                regions,
                parameters: self.parameters.get(&key).map_or(&[], Vec::as_slice),
            },
            projections: self.projections.get(&key).map_or(&[], Vec::as_slice),
        })
    }
    fn collect(
        &mut self,
        types: &Store,
        root: TypeId,
        depth: u16,
        kind: ComponentKind,
        budget: &mut Budget,
    ) -> Result<ComponentLists, &'static str> {
        self.collect_roots(types, &[root], depth, kind, budget)
    }
    pub(super) fn collect_roots(
        &mut self,
        types: &Store,
        roots: &[TypeId],
        depth: u16,
        kind: ComponentKind,
        budget: &mut Budget,
    ) -> Result<ComponentLists, &'static str> {
        self.walk.work.clear();
        self.walk.seen.clear();
        self.walk.regions.clear();
        self.walk.parameter_seen.clear();
        self.walk.extend(types, roots, depth, kind, budget)
    }
    pub(super) fn captures(
        &mut self,
        types: &Store,
        ty: TypeId,
        depth: u16,
        budget: &mut Budget,
    ) -> Result<bool, &'static str> {
        if depth == 0 {
            return Ok(false);
        }
        let query = self
            .captures
            .entry(depth)
            .or_insert_with(|| crate::types::bound_uses::captures::Captures::new(depth));
        let before = query.steps;
        let result = query.collect(types, ty)?;
        let work = query.steps.saturating_sub(before);
        budget.pending(work)?;
        budget.0 += work;
        Ok(result)
    }
}
impl Walk {
    pub(super) fn extend(
        &mut self,
        types: &Store,
        roots: &[TypeId],
        depth: u16,
        kind: ComponentKind,
        budget: &mut Budget,
    ) -> Result<ComponentLists, &'static str> {
        budget.pending(roots.len())?;
        let expanded = if kind == ComponentKind::ProjectionInputs {
            let [root] = roots else {
                return Err("projection input components require one root");
            };
            Some(*root)
        } else {
            None
        };
        self.work.clear();
        self.work.extend(roots.iter().map(|&root| (root, depth)));
        let mut values = Vec::new();
        let mut parameters = Vec::new();
        let mut projections = Vec::new();
        while let Some((ty, depth)) = self.work.pop() {
            budget.charge()?;
            if !self.seen.insert((ty, depth)) {
                continue;
            }
            match types.kind(ty).ok_or("invalid outlives component type")? {
                Kind::Parameter(id) => {
                    if self.parameter_seen.insert(id) {
                        parameters.push(id);
                    }
                }
                Kind::Reference {
                    region, referent, ..
                } => {
                    self.record_region(region, depth, &mut values)?;
                    // Only minimal outlives queries rely on separately proved
                    // reference well-formedness. Projection inputs retain the
                    // referent even when this region is bound by a callable.
                    if kind != ComponentKind::Outlives {
                        self.work.push((referent, depth));
                    }
                }
                Kind::Array { length: 0, .. } if kind == ComponentKind::LiveValue => {}
                Kind::Opaque(id, args) => {
                    if matches!(kind, ComponentKind::LiveValue | ComponentKind::DropInputs) {
                        let mut cost = 0;
                        let bound = types.opaque_live_bound(id, args, depth, &mut cost)?;
                        budget.work(cost)?;
                        if let Some(bound) = bound {
                            self.record_region(bound, 0, &mut values)?;
                            continue;
                        }
                    } else if expanded != Some(ty) {
                        projections.push(Projection { ty, depth });
                        continue;
                    }
                    self.arguments(
                        types.arguments(args).ok_or("missing live-value captures")?,
                        depth,
                        &mut values,
                        budget,
                    )?;
                }
                Kind::Adt(_, args)
                | Kind::FunctionItem {
                    arguments: args, ..
                } => {
                    self.arguments(
                        types.arguments(args).ok_or("missing outlives arguments")?,
                        depth,
                        &mut values,
                        budget,
                    )?;
                }
                Kind::Projection(application) => {
                    if kind != ComponentKind::LiveValue && expanded != Some(ty) {
                        projections.push(Projection { ty, depth });
                        continue;
                    }
                    // Every input outliving a use is sufficient for its associated
                    // result to outlive that use. This conservative liveness route
                    // grants no declared result bound or reverse input assumption.
                    self.arguments(
                        types.projection_inputs(application)?,
                        depth,
                        &mut values,
                        budget,
                    )?;
                }
                _ => children(types, ty, depth, &mut self.work, budget)?,
            }
        }
        Ok((values, parameters, projections))
    }
    fn record_region(
        &mut self,
        region: RegionId,
        depth: u16,
        values: &mut Vec<RegionId>,
    ) -> Result<(), &'static str> {
        if let Some(region) = free(region, depth)?
            && region != RegionId::STATIC
            && self.regions.insert(region)
        {
            values.push(region);
        }
        Ok(())
    }
    fn arguments(
        &mut self,
        arguments: impl Iterator<Item = Argument>,
        depth: u16,
        values: &mut Vec<RegionId>,
        budget: &mut Budget,
    ) -> Result<(), &'static str> {
        for argument in arguments {
            budget.charge()?;
            match argument {
                Argument::Lifetime(region) => self.record_region(region, depth, values)?,
                Argument::Type(ty) => {
                    budget.pending(self.work.len().saturating_add(1))?;
                    self.work.push((ty, depth));
                }
                Argument::Const(_) => {}
            }
        }
        Ok(())
    }
}
