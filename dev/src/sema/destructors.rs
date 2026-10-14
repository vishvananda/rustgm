//! Resolved destructor contracts, independent of cleanup scheduling and code emission.
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    types::{ItemId, RegionId, TypeId, outlives::Pair, region_graph},
};
use std::collections::{HashMap, HashSet};

pub struct Implementation {
    pub source: NodeKey,
    pub self_definition: DefId,
    pub method: DefId,
    pub target: TypeId,
    pub item: ItemId,
    /// Named and anonymous impl slots captured by the method's body environment.
    pub parameters: u32,
    pub bounds: Vec<Pair>,
}

#[derive(Clone, Copy)]
pub struct Receiver {
    pub binding: NodeKey,
    pub definition: DefId,
    pub ty: TypeId,
    /// Binding mutability, independent of the reference's referent mutability.
    pub mutable: bool,
}

#[derive(Default)]
pub struct Destructors {
    declared: HashSet<ItemId>,
    implementations: Vec<Implementation>,
    methods: HashMap<DefId, usize>,
    self_types: HashMap<DefId, usize>,
    items: HashMap<ItemId, usize>,
    pub receivers: HashMap<NodeKey, Receiver>,
}
impl Destructors {
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
    implementation: &Implementation,
    arguments: &[RegionId],
    assumptions: &crate::types::outlives::Requirements<'_>,
    types: &crate::types::Store,
    predicates: impl Iterator<Item = crate::types::parameters::predicates::Predicate>,
) -> Result<(), String> {
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
    let resolve = |region: RegionId| -> Result<region_graph::Region, String> {
        if region == RegionId::STATIC {
            return Ok(region_graph::Region::STATIC);
        }
        let Some((0, index)) = region.bound_parts() else {
            return Err("invalid impl lifetime identity".into());
        };
        slots
            .get(usize::from(index))
            .copied()
            .ok_or_else(|| "missing impl lifetime slot".into())
    };
    for &(a, b) in assumptions.regions {
        graph
            .assume(resolve(a)?, resolve(b)?)
            .map_err(|e| e.to_string())?;
    }
    for &(a, b) in &implementation.bounds {
        graph
            .require(resolve(a)?, resolve(b)?, 0)
            .map_err(|e| e.to_string())?;
    }
    let mut parameters = HashMap::new();
    for &(id, shorter) in assumptions.parameters {
        let longer = parameter_region(&mut graph, &mut parameters, id)?;
        graph
            .assume(longer, resolve(shorter)?)
            .map_err(|e| e.to_string())?;
    }
    let mut components = crate::types::outlives::TypeComponents::default();
    for predicate in predicates {
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
