//! Infer nominal lifetime contracts once; static field requirements need declared proof.
use super::{Checker, DefId, Error, ItemId};
use crate::types::{
    RegionId,
    outlives::{Bounds, Pair, ParameterPair, TypeComponents},
    parameters::{ParameterId, predicates::Bound},
    region_graph::{self, Graph, Region},
};
use std::collections::HashMap;

impl Checker<'_> {
    pub(super) fn nominal_outlives(&mut self) -> Result<(), Error> {
        let predicates = self
            .facts
            .generics
            .predicates
            .iter()
            .filter(|(owner, _)| self.facts.aggregates.get(ItemId(owner.0)).is_some())
            .flat_map(|(owner, predicates)| predicates.iter().map(|p| (ItemId(owner.0), p.value)));
        let mut bounds = Bounds::infer_with_clauses(
            &mut self.facts.types,
            &self.facts.aggregates,
            &self.facts.lifetimes.nominal_bounds,
            predicates,
        )
        .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
        let mut explicit: HashMap<ItemId, Vec<Pair>> = HashMap::new();
        for &(item, pair) in &self.facts.lifetimes.nominal_bounds {
            explicit.entry(item).or_default().push(pair);
        }
        let mut components = TypeComponents::default();
        for entry in &self.nominals.entries {
            let ty = self.facts.definitions[entry.item.0 as usize]
                .ok_or_else(|| self.error(entry.key, "missing nominal requirement root"))?;
            bounds
                .prepare(&mut self.facts.types, ty)
                .map_err(|e| self.error(entry.key, e))?;
            self.nominal_static_bounds(entry.item, &bounds, &explicit, &mut components)
                .map_err(|e| self.error(entry.key, e))?;
        }
        self.facts.lifetimes.nominal_requirements = bounds;
        Ok(())
    }
    fn nominal_static_bounds(
        &self,
        item: ItemId,
        bounds: &Bounds,
        explicit: &HashMap<ItemId, Vec<Pair>>,
        components: &mut TypeComponents,
    ) -> Result<(), String> {
        let pairs = bounds
            .get(item)
            .ok_or("missing nominal lifetime requirements")?;
        let parameters = bounds
            .parameters(item)
            .ok_or("missing nominal type lifetime requirements")?;
        if !pairs.iter().any(|pair| pair.1 == RegionId::STATIC)
            && !parameters.iter().any(|pair| pair.1 == RegionId::STATIC)
            && !bounds
                .projections(item)
                .iter()
                .any(|pair| pair.1 == RegionId::STATIC)
        {
            return Ok(());
        }
        if !bounds.projections(item).is_empty() {
            return self.projected_static_bounds(item, bounds, explicit);
        }
        let mut proof = Proof::new();
        for &pair in pairs {
            proof.regions(pair, pair.1 != RegionId::STATIC)?;
        }
        for &pair in parameters {
            proof.parameter(pair, pair.1 != RegionId::STATIC)?;
        }
        for &pair in explicit.get(&item).into_iter().flatten() {
            proof.regions(pair, true)?;
        }
        for predicate in self
            .facts
            .generics
            .predicates
            .get(&DefId(item.0))
            .into_iter()
            .flatten()
        {
            let predicate = predicate.value.ordinary()?;
            if let Bound::Outlives(shorter) = predicate.bound {
                let parts = components.get(&self.facts.types, predicate.ty)?;
                for &longer in parts.regions {
                    proof.regions((longer, shorter), true)?;
                }
                for &id in parts.parameters {
                    proof.parameter((id, shorter), true)?;
                }
            }
        }
        proof.graph.solve().map_err(|e| e.to_string())?;
        Ok(())
    }
    fn projected_static_bounds(
        &self,
        item: ItemId,
        bounds: &Bounds,
        explicit: &HashMap<ItemId, Vec<Pair>>,
    ) -> Result<(), String> {
        use crate::types::{
            Kind,
            arguments::Argument,
            outlives::{Queries, Requirements, ScalarRequirements},
            relations::{Constraints, ProofScope},
        };
        let owner = DefId(item.0);
        let count = u32::try_from(
            self.facts
                .lifetimes
                .parameters
                .get(&owner)
                .map_or(0, Vec::len),
        )
        .map_err(|_| "too many nominal lifetime parameters")?;
        let mut proof = Constraints::new(
            &self.facts.types,
            &self.facts.lifetimes.variances,
            count,
            1_000_000,
        )
        .map_err(|e| e.to_string())?;
        for parameter in self.facts.generics.parameters_in_scope(owner) {
            if let Argument::Type(ty) = parameter.argument
                && let Some(Kind::Parameter(id)) = self.facts.types.kind(ty)
            {
                proof.parameter(id).map_err(|e| e.to_string())?;
            }
        }
        for &(longer, shorter) in bounds
            .get(item)
            .ok_or("missing nominal region requirements")?
        {
            if shorter == RegionId::STATIC {
                proof.outlives(longer, shorter, 0)
            } else {
                proof.assume(longer, shorter)
            }
            .map_err(|e| e.to_string())?;
        }
        for &(id, shorter) in bounds
            .parameters(item)
            .ok_or("missing nominal parameter requirements")?
        {
            if shorter == RegionId::STATIC {
                proof.parameter_outlives(id, shorter, 0)
            } else {
                proof.assume_parameter(id, shorter)
            }
            .map_err(|e| e.to_string())?;
        }
        for &(projection, shorter) in bounds.projections(item) {
            if shorter == RegionId::STATIC {
                proof
                    .require_projection(projection, shorter, ProofScope::ROOT, 0)
                    .map_err(|e| e.to_string())?;
            } else {
                proof.assume_requirements(Requirements {
                    scalars: ScalarRequirements {
                        regions: &[],
                        parameters: &[],
                    },
                    projections: &[(projection, shorter)],
                })?;
            }
        }
        for &(longer, shorter) in explicit.get(&item).into_iter().flatten() {
            proof.assume(longer, shorter).map_err(|e| e.to_string())?;
        }
        let mut queries = Queries::new(&self.facts.types, bounds, &self.facts.lifetimes.callables);
        for predicate in self.facts.generics.predicates_in_scope(owner) {
            let predicate = predicate.value.ordinary()?;
            if let Bound::Outlives(shorter) = predicate.bound {
                proof.assume_type_outlives(&mut queries, predicate.ty, shorter)?;
            }
        }
        proof.solve().map_err(|e| e.to_string())?;
        Ok(())
    }
}
struct Proof {
    graph: Graph,
    regions: HashMap<RegionId, Region>,
    parameters: HashMap<ParameterId, Region>,
}
impl Proof {
    fn new() -> Self {
        Self {
            graph: Graph::new(1_000_000),
            regions: HashMap::new(),
            parameters: HashMap::new(),
        }
    }
    fn region(&mut self, id: RegionId) -> Result<Region, String> {
        if id == RegionId::STATIC {
            return Ok(Region::STATIC);
        }
        if !matches!(id.bound_parts(), Some((0, _))) {
            return Err("nominal lifetime requirement escapes its declaration".into());
        }
        if let Some(&region) = self.regions.get(&id) {
            return Ok(region);
        }
        let region = self
            .graph
            .region(region_graph::Kind::Universal { universe: 0 })
            .map_err(|e| e.to_string())?;
        self.regions.insert(id, region);
        Ok(region)
    }
    fn record(&mut self, longer: Region, shorter: Region, assume: bool) -> Result<(), String> {
        if assume {
            self.graph.assume(longer, shorter)
        } else {
            self.graph.require(longer, shorter, 0)
        }
        .map_err(|e| e.to_string())
    }
    fn regions(&mut self, pair: Pair, assume: bool) -> Result<(), String> {
        let longer = self.region(pair.0)?;
        let shorter = self.region(pair.1)?;
        self.record(longer, shorter, assume)
    }
    fn parameter(&mut self, (id, shorter): ParameterPair, assume: bool) -> Result<(), String> {
        let longer = if let Some(&region) = self.parameters.get(&id) {
            region
        } else {
            let region = self
                .graph
                .region(region_graph::Kind::Universal { universe: 0 })
                .map_err(|e| e.to_string())?;
            self.parameters.insert(id, region);
            region
        };
        let shorter = self.region(shorter)?;
        self.record(longer, shorter, assume)
    }
}
