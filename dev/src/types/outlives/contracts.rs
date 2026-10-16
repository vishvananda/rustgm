//! Finite nominal requirements propagate through argument substitutions, never field expansion.
use super::{
    Argument, ComponentKind, HashMap, HashSet, ItemId, Pair, ParameterPair, RegionId, Scan, Store,
    Use, free, useful,
};
use crate::types::{
    TypeId,
    aggregate::Definitions,
    parameters::predicates::{Bound, Predicate},
};

fn region(types: &Store, use_: Use, region: RegionId) -> Result<Option<RegionId>, &'static str> {
    if region == RegionId::STATIC {
        return Ok(Some(region));
    }
    let Some((0, index)) = region.bound_parts() else {
        return Err("invalid outlives template region");
    };
    let argument = types
        .arguments(use_.arguments)
        .ok_or("missing outlives instance arguments")?
        .nth(usize::from(index));
    let Some(Argument::Lifetime(region)) = argument else {
        return Err("outlives template argument arity mismatch");
    };
    free(region, use_.depth)
}
pub(super) fn instantiate(
    types: &Store,
    use_: Use,
    pair: Pair,
) -> Result<Option<Pair>, &'static str> {
    Ok(region(types, use_, pair.0)?.zip(region(types, use_, pair.1)?))
}
impl Scan {
    fn type_bound(
        &mut self,
        types: &Store,
        ty: TypeId,
        shorter: RegionId,
        depth: u16,
    ) -> Result<(), &'static str> {
        let parts =
            self.components
                .get(types, ty, depth, ComponentKind::TypeBound, &mut self.budget)?;
        self.budget.pending(parts.len())?;
        for &longer in parts.scalars.regions {
            self.budget.charge()?;
            let pair = (longer, shorter);
            if useful(pair) && self.unique.insert(pair) {
                self.pairs.push(pair);
            }
        }
        for &id in parts.scalars.parameters {
            self.budget.charge()?;
            let pair = (id, shorter);
            if self.unique_parameters.insert(pair) {
                self.parameter_pairs.push(pair);
            }
        }
        for &projection in parts.projections {
            self.budget.charge()?;
            let pair = (projection, shorter);
            if self.unique_projections.insert(pair) {
                self.projection_pairs.push(pair);
            }
        }
        Ok(())
    }
    pub(super) fn apply_parameter(
        &mut self,
        types: &Store,
        use_: Use,
        (id, shorter): ParameterPair,
    ) -> Result<(), &'static str> {
        self.budget.charge()?;
        let parameter = types.parameter(id).ok_or("missing outlives parameter")?;
        if parameter.owner != use_.item {
            return Err("outlives parameter belongs to another declaration");
        }
        let Some(shorter) = region(types, use_, shorter)? else {
            return Ok(());
        };
        let argument = types
            .arguments(use_.arguments)
            .ok_or("missing outlives instance arguments")?
            .nth(parameter.index as usize);
        let Some(Argument::Type(ty)) = argument else {
            return Err("outlives type argument kind or arity mismatch");
        };
        self.type_bound(types, ty, shorter, use_.depth)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Requirement {
    Region(Pair),
    Parameter(ParameterPair),
}
#[derive(Default)]
struct Inferred {
    regions: Vec<Pair>,
    parameters: Vec<ParameterPair>,
}
#[derive(Default)]
pub struct Bounds {
    items: HashMap<ItemId, Inferred>,
    pub steps: usize,
}
impl Bounds {
    /// Infer required field lifetimes from all variants, with finite propagation.
    /// # Errors
    /// Rejects malformed templates and exhausted propagation work.
    pub fn infer(
        types: &Store,
        definitions: &Definitions,
        explicit: &[(ItemId, Pair)],
    ) -> Result<Self, &'static str> {
        Self::infer_with_predicates(types, definitions, explicit, [])
    }
    /// Include declared type-outlives predicates. Inferred static requirements
    /// remain obligations: declaration checking must not assume them implicitly.
    /// # Errors
    /// Rejects malformed templates and exhausted propagation work.
    pub fn infer_with_predicates(
        types: &Store,
        definitions: &Definitions,
        explicit: &[(ItemId, Pair)],
        predicates: impl IntoIterator<Item = (ItemId, Predicate)>,
    ) -> Result<Self, &'static str> {
        let mut inference = Inference::default();
        for definition in definitions {
            inference
                .result
                .items
                .insert(definition.item, Inferred::default());
            inference.scan.clear();
            for variant in definitions.variants(definition) {
                for &ty in types
                    .list(variant.fields)
                    .ok_or("missing outlives field templates")?
                {
                    inference.scan.visit(types, ty)?;
                }
            }
            inference.enqueue(definition.item)?;
            for &use_ in &inference.scan.uses {
                inference.scan.budget.charge()?;
                if use_.callable {
                    return Err("nominal field template contains an unnameable function item");
                }
                inference
                    .reverse
                    .entry(use_.item)
                    .or_default()
                    .push((definition.item, use_));
            }
        }
        for &(item, pair) in explicit {
            inference.scan.clear();
            inference.scan.record(pair);
            inference.enqueue(item)?;
        }
        for (item, predicate) in predicates {
            inference.scan.budget.charge()?;
            if let Bound::Outlives(shorter) = predicate.bound {
                inference.scan.clear();
                inference.scan.type_bound(types, predicate.ty, shorter, 0)?;
                inference.enqueue(item)?;
            }
        }
        inference.solve(types)
    }
    #[must_use]
    pub fn get(&self, item: ItemId) -> Option<&[Pair]> {
        self.items.get(&item).map(|item| item.regions.as_slice())
    }
    #[must_use]
    pub fn parameters(&self, item: ItemId) -> Option<&[ParameterPair]> {
        self.items.get(&item).map(|item| item.parameters.as_slice())
    }
}
#[derive(Default)]
struct Inference {
    result: Bounds,
    unique: HashSet<(ItemId, Requirement)>,
    work: Vec<(ItemId, Requirement)>,
    reverse: HashMap<ItemId, Vec<(ItemId, Use)>>,
    scan: Scan,
}
impl Inference {
    fn enqueue(&mut self, owner: ItemId) -> Result<(), &'static str> {
        if !self.scan.projection_pairs.is_empty() {
            return Err("nominal projected outlives templates require scoped substitution");
        }
        for requirement in self
            .scan
            .pairs
            .iter()
            .copied()
            .map(Requirement::Region)
            .chain(
                self.scan
                    .parameter_pairs
                    .iter()
                    .copied()
                    .map(Requirement::Parameter),
            )
        {
            self.scan.budget.charge()?;
            if self.unique.insert((owner, requirement)) {
                self.scan
                    .budget
                    .pending(self.work.len().saturating_add(1))?;
                self.work.push((owner, requirement));
            }
        }
        Ok(())
    }
    fn solve(mut self, types: &Store) -> Result<Bounds, &'static str> {
        while let Some((item, requirement)) = self.work.pop() {
            self.scan.budget.charge()?;
            let output = self
                .result
                .items
                .get_mut(&item)
                .ok_or("missing outlives nominal definition")?;
            match requirement {
                Requirement::Region(pair) => output.regions.push(pair),
                Requirement::Parameter(pair) => output.parameters.push(pair),
            }
            for index in 0..self.reverse.get(&item).map_or(0, Vec::len) {
                let (owner, use_) = self.reverse[&item][index];
                self.scan.budget.charge()?;
                self.scan.clear();
                match requirement {
                    Requirement::Region(pair) => {
                        if let Some(pair) = instantiate(types, use_, pair)? {
                            self.scan.record(pair);
                        }
                    }
                    Requirement::Parameter(pair) => self.scan.apply_parameter(types, use_, pair)?,
                }
                self.enqueue(owner)?;
            }
        }
        self.result.steps = self.scan.budget.0;
        Ok(self.result)
    }
}
