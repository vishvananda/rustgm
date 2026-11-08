//! Resolved destructor contracts, independent of cleanup scheduling and code emission.
mod instances;
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    types::{ItemId, RegionId, TypeId, outlives::Pair, region_graph},
};
pub(crate) use instances::cleanup_method;
pub use instances::select;

pub struct Coverage {
    /// Impl arguments expressed using the nominal declaration's parameters.
    pub arguments: crate::types::GenericArgs,
    /// Captured impl lifetime slots mapped to nominal slots or extra existential slots.
    pub regions: Vec<RegionId>,
}

pub struct Implementation {
    pub source: NodeKey,
    pub self_definition: DefId,
    pub method: DefId,
    pub target: TypeId,
    pub item: ItemId,
    /// Named and anonymous impl slots captured by the method's body environment.
    pub parameters: u32,
    pub bounds: Vec<Pair>,
    pub coverage: Coverage,
}

/// Only nominal coverage is new; ordinary associated owners retain all source,
/// target, bounds and selected-member facts for async implementations.
pub struct AsyncImplementation {
    pub self_definition: DefId,
    pub item: ItemId,
    pub coverage: Coverage,
}

#[derive(Clone, Copy)]
pub(super) struct LifetimeContract<'a> {
    pub parameters: u32,
    pub bounds: &'a [Pair],
}

#[derive(Default)]
pub struct Destructors {
    declared: HashSet<ItemId>,
    implementations: Vec<Implementation>,
    methods: HashMap<DefId, usize>,
    self_types: HashMap<DefId, usize>,
    items: HashMap<ItemId, usize>,
    instances: HashMap<TypeId, crate::types::instance::Instance>,
    asynchronous: Option<Box<HashMap<ItemId, AsyncImplementation>>>,
}
impl Destructors {
    pub(super) fn coverage(&self, owner: DefId, item: ItemId) -> Option<&Coverage> {
        if let Some(implementation) = self.item(item).filter(|i| i.self_definition == owner) {
            return Some(&implementation.coverage);
        }
        self.async_item(item)
            .filter(|i| i.self_definition == owner)
            .map(|i| &i.coverage)
    }
    pub fn async_implementations(&self) -> impl Iterator<Item = &AsyncImplementation> {
        self.asynchronous
            .iter()
            .flat_map(|entries| entries.values())
    }
    #[must_use]
    pub fn async_item(&self, item: ItemId) -> Option<&AsyncImplementation> {
        self.asynchronous
            .as_deref()
            .and_then(|entries| entries.get(&item))
    }
    pub(super) fn insert_async(
        &mut self,
        implementation: AsyncImplementation,
    ) -> Result<(), &'static str> {
        let entries = self.asynchronous.get_or_insert_with(Box::default);
        if entries.contains_key(&implementation.item) {
            return Err("conflicting AsyncDrop implementations for the same nominal type");
        }
        entries.insert(implementation.item, implementation);
        Ok(())
    }
    pub(in crate::sema) fn declare(&mut self, item: ItemId) {
        self.declared.insert(item);
    }
    pub(in crate::sema) fn declared(&self) -> impl ExactSizeIterator<Item = ItemId> + '_ {
        self.declared.iter().copied()
    }
    pub(in crate::sema) fn has_declared(&self, item: ItemId) -> bool {
        self.declared.contains(&item)
    }
    pub(super) fn extend_bounds(&mut self, method: DefId, bounds: &[Pair]) {
        if let Some(&index) = self.methods.get(&method) {
            self.implementations[index].bounds.extend_from_slice(bounds);
        }
    }
    #[must_use]
    pub fn implementations(&self) -> &[Implementation] {
        &self.implementations
    }
    #[must_use]
    pub fn method(&self, id: DefId) -> Option<&Implementation> {
        self.methods.get(&id).map(|i| &self.implementations[*i])
    }
    #[must_use]
    pub fn self_type(&self, id: DefId) -> Option<&Implementation> {
        self.self_types.get(&id).map(|i| &self.implementations[*i])
    }
    #[must_use]
    pub fn item(&self, id: ItemId) -> Option<&Implementation> {
        self.items.get(&id).map(|i| &self.implementations[*i])
    }
    pub(super) fn insert(&mut self, implementation: Implementation) -> Result<(), &'static str> {
        if self.items.contains_key(&implementation.item) {
            return Err("conflicting Drop implementations for the same nominal type");
        }
        let index = self.implementations.len();
        self.declare(implementation.item);
        self.methods.insert(implementation.method, index);
        self.self_types
            .insert(implementation.self_definition, index);
        self.items.insert(implementation.item, index);
        self.implementations.push(implementation);
        Ok(())
    }
}

/// Unused impl lifetimes are existential for coverage, while each distinct
/// target argument is universal. Paths through unused slots may not strengthen
/// the nominal type's declared or field-implied requirements.
pub(super) fn check_bounds(
    implementation: LifetimeContract<'_>,
    arguments: &[RegionId],
    assumptions: &crate::types::outlives::Requirements<'_>,
    types: &crate::types::Store,
    predicates: impl Iterator<Item = crate::types::parameters::clauses::Clause>,
    evidence: Option<&super::traits::obligations::Obligations>,
) -> Result<(), String> {
    if !assumptions.projections.is_empty() {
        return Err("projected destructor bounds require scoped nominal contracts".into());
    }
    let mut graph = region_graph::Graph::new(1_000_000);
    let universals: HashSet<_> = arguments.iter().copied().collect();
    let mut slots = Vec::with_capacity(implementation.parameters as usize);
    for index in 0..implementation.parameters {
        let index = u16::try_from(index).map_err(|_| "too many impl lifetime parameters")?;
        let region = RegionId::bound(0, index)?;
        let kind = if universals.contains(&region) {
            region_graph::Kind::Universal { universe: 0 }
        } else {
            region_graph::Kind::Variable { universe: 0 }
        };
        slots.push(graph.region(kind).map_err(|e| e.to_string())?);
    }
    let mut variables = HashMap::default();
    if let Some(evidence) = evidence {
        for id in evidence.regions.clone() {
            let variable = graph
                .region(region_graph::Kind::Variable { universe: 0 })
                .map_err(|e| e.to_string())?;
            variables.insert(RegionId(id), variable);
        }
    }
    let resolve = |region: RegionId| -> Result<region_graph::Region, String> {
        if region == RegionId::STATIC {
            return Ok(region_graph::Region::STATIC);
        }
        let Some((0, index)) = region.bound_parts() else {
            return variables
                .get(&region)
                .copied()
                .ok_or_else(|| "invalid impl lifetime identity".into());
        };
        slots
            .get(usize::from(index))
            .copied()
            .ok_or_else(|| "missing impl lifetime slot".into())
    };
    for &(a, b) in assumptions.scalars.regions {
        graph
            .assume(resolve(a)?, resolve(b)?)
            .map_err(|e| e.to_string())?;
    }
    for &(a, b) in implementation.bounds {
        graph
            .require(resolve(a)?, resolve(b)?, 0)
            .map_err(|e| e.to_string())?;
    }
    let mut parameters = HashMap::default();
    for &(id, shorter) in assumptions.scalars.parameters {
        let longer = parameter_region(&mut graph, &mut parameters, id)?;
        graph
            .assume(longer, resolve(shorter)?)
            .map_err(|e| e.to_string())?;
    }
    if let Some(evidence) = evidence {
        trait_obligations(&mut graph, types, evidence, &resolve, &mut parameters)?;
    }
    let mut components = crate::types::outlives::TypeComponents::default();
    for predicate in predicates {
        // A binder-scoped trait bound adds no lifetime requirement here; only a
        // quantified outlives body would need its own scoped region proof.
        if let crate::types::parameters::clauses::Clause::ForAll(id) = predicate {
            let body = types
                .quantified(id)
                .ok_or("invalid quantified Drop requirement")?;
            if matches!(
                body.predicate.bound,
                crate::types::parameters::predicates::Bound::Outlives(_)
            ) {
                return Err("quantified clause requires a scoped consumer".into());
            }
            continue;
        }
        let predicate = predicate.ordinary()?;
        let crate::types::parameters::predicates::Bound::Outlives(shorter) = predicate.bound else {
            continue;
        };
        let shorter = resolve(shorter)?;
        let parts = components.get(types, predicate.ty)?;
        for &region in parts.regions {
            graph
                .require(resolve(region)?, shorter, 0)
                .map_err(|e| e.to_string())?;
        }
        for &id in parts.parameters {
            let longer = parameter_region(&mut graph, &mut parameters, id)?;
            graph
                .require(longer, shorter, 0)
                .map_err(|e| e.to_string())?;
        }
    }
    graph.solve().map_err(|error| match error {
        region_graph::Error::Unsatisfied { .. } => {
            "Drop impl adds lifetime requirements to its nominal type".into()
        }
        _ => error.to_string(),
    })?;
    Ok(())
}

/// Selected trait requirements have already been restored to impl coordinates.
fn trait_obligations(
    graph: &mut region_graph::Graph,
    types: &crate::types::Store,
    evidence: &super::traits::obligations::Obligations,
    resolve: &impl Fn(RegionId) -> Result<region_graph::Region, String>,
    parameters: &mut HashMap<crate::types::parameters::ParameterId, region_graph::Region>,
) -> Result<(), String> {
    for &(a, b) in &evidence.equalities {
        graph
            .require(resolve(a)?, resolve(b)?, 0)
            .map_err(|e| e.to_string())?;
        graph
            .require(resolve(b)?, resolve(a)?, 0)
            .map_err(|e| e.to_string())?;
    }
    for &(a, b) in &evidence.outlives {
        graph
            .require(resolve(a)?, resolve(b)?, 0)
            .map_err(|e| e.to_string())?;
    }
    let mut components = crate::types::outlives::TypeComponents::default();
    for &(ty, shorter) in &evidence.types {
        let shorter = resolve(shorter)?;
        let parts = components.get(types, ty)?;
        for &longer in parts.regions {
            graph
                .require(resolve(longer)?, shorter, 0)
                .map_err(|e| e.to_string())?;
        }
        for &id in parts.parameters {
            let longer = parameter_region(graph, parameters, id)?;
            graph
                .require(longer, shorter, 0)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn parameter_region(
    graph: &mut region_graph::Graph,
    parameters: &mut HashMap<crate::types::parameters::ParameterId, region_graph::Region>,
    id: crate::types::parameters::ParameterId,
) -> Result<region_graph::Region, String> {
    if let Some(&region) = parameters.get(&id) {
        return Ok(region);
    }
    let region = graph
        .region(region_graph::Kind::Universal { universe: 0 })
        .map_err(|e| e.to_string())?;
    parameters.insert(id, region);
    Ok(region)
}
