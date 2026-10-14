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
        let bounds = Bounds::infer_with_predicates(
            &self.facts.types,
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
        {
            return Ok(());
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
            if let Bound::Outlives(shorter) = predicate.value.bound {
                let parts = components.get(&self.facts.types, predicate.value.ty)?;
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
