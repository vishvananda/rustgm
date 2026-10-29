//! Finite nominal requirements propagate through argument substitutions, never field expansion.
use super::{
    Argument, ComponentKind, HashMap, HashSet, ItemId, Pair, ParameterPair, ProjectionPair,
    RegionId, Scan, Store, Use, free, useful,
};
use crate::types::{
    TypeId,
    aggregate::Definitions,
    parameters::{
        clauses::Clause,
        predicates::{Bound, Predicate},
    },
};
#[path = "outliving_inference.rs"]
mod demanded_bounds;

pub(super) fn region(
    types: &Store,
    use_: Use,
    region: RegionId,
) -> Result<Option<RegionId>, &'static str> {
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
impl Scan<'_> {
    fn type_bound(
        &mut self,
        types: &Store,
        ty: TypeId,
        shorter: RegionId,
        depth: u16,
    ) -> Result<(), &'static str> {
        self.component_bound(types, ty, shorter, depth, ComponentKind::TypeBound)
    }
    pub(super) fn component_bound(
        &mut self,
        types: &Store,
        ty: TypeId,
        shorter: RegionId,
        depth: u16,
        kind: ComponentKind,
    ) -> Result<(), &'static str> {
        let parts = self
            .components
            .get(types, ty, depth, kind, &mut self.budget)?;
        self.budget.pending(parts.len())?;
        for &longer in parts.scalars.regions {
            self.budget.charge()?;
            let pair = (longer, shorter);
            if useful(pair) && self.budget.insert(&mut self.unique, pair)? {
                self.budget.push(&mut self.pairs, pair)?;
            }
        }
        for &id in parts.scalars.parameters {
            self.budget.charge()?;
            let pair = (id, shorter);
            if self.budget.insert(&mut self.unique_parameters, pair)? {
                self.budget.push(&mut self.parameter_pairs, pair)?;
            }
        }
        for &projection in parts.projections {
            self.budget.charge()?;
            let pair = (projection, shorter);
            if self.budget.insert(&mut self.unique_projections, pair)? {
                self.budget.push(&mut self.projection_pairs, pair)?;
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
        self.parameter_with(
            types,
            use_,
            (id, shorter),
            &mut super::requirements::Ordinary,
        )
    }
    pub(super) fn parameter_with(
        &mut self,
        types: &Store,
        use_: Use,
        (id, shorter): ParameterPair,
        demand: &mut impl super::requirements::Demand,
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
        demand.require(
            self,
            types,
            ty,
            shorter,
            use_.depth,
            ComponentKind::TypeBound,
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Requirement {
    Region(Pair),
    Parameter(ParameterPair),
    Projection(ProjectionPair),
}
#[derive(Default)]
struct Inferred {
    regions: Vec<Pair>,
    parameters: Vec<ParameterPair>,
}
#[derive(Default)]
pub struct Bounds {
    items: HashMap<ItemId, Inferred>,
    pub(super) projected: Option<Box<super::instances::Projected>>,
    pub steps: usize,
}
impl Bounds {
    /// Append one newly completed nominal whose fields have no nominal or
    /// projected dependency. Existing inferred entries remain immutable.
    /// # Errors
    /// Rejects existing entries, dependent fields and ordinary inference limits.
    pub(crate) fn append_leaf(
        &mut self,
        types: &mut Store,
        definitions: &Definitions,
        item: ItemId,
    ) -> Result<(), &'static str> {
        if self.items.contains_key(&item) {
            return Err("nominal outlives contract is already complete");
        }
        let definition = definitions
            .get(item)
            .ok_or("missing leaf nominal definition")?;
        let mut inference = Inference::default();
        inference.fields(types, definitions, definition)?;
        if !inference.scan.uses.is_empty() || !inference.scan.projection_pairs.is_empty() {
            return Err("leaf outlives append has a nominal or projected dependency");
        }
        let mut inferred = inference.solve(types)?;
        if inferred.projected.is_some() || inferred.items.len() != 1 {
            return Err("leaf outlives append has nonlocal requirements");
        }
        let value = inferred
            .items
            .remove(&item)
            .ok_or("missing inferred leaf contract")?;
        let steps = self
            .steps
            .checked_add(inferred.steps)
            .filter(|n| *n <= 1_000_000)
            .ok_or("type outlives work budget exceeded")?;
        self.items.insert(item, value);
        self.steps = steps;
        Ok(())
    }
    /// Infer required field lifetimes from all variants, with finite propagation.
    /// # Errors
    /// Rejects malformed templates and exhausted propagation work.
    pub fn infer(
        types: &mut Store,
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
        types: &mut Store,
        definitions: &Definitions,
        explicit: &[(ItemId, Pair)],
        predicates: impl IntoIterator<Item = (ItemId, Predicate)>,
    ) -> Result<Self, &'static str> {
        Self::infer_with_clauses(
            types,
            definitions,
            explicit,
            predicates
                .into_iter()
                .map(|(item, predicate)| (item, Clause::Predicate(predicate))),
        )
    }
    /// Infer only binder-independent outlives consequences of declaration clauses.
    /// A dependent quantified requirement remains an obligation of nominal formation;
    /// it supplies no implied scalar or projected bound here.
    /// # Errors
    /// Rejects malformed templates, clauses and exhausted propagation work.
    pub fn infer_with_clauses(
        types: &mut Store,
        definitions: &Definitions,
        explicit: &[(ItemId, Pair)],
        predicates: impl IntoIterator<Item = (ItemId, Clause)>,
    ) -> Result<Self, &'static str> {
        Self::infer_with_inputs(
            types,
            definitions,
            explicit,
            predicates,
            super::families::Families::default(),
        )
    }

    pub(crate) fn infer_with_inputs(
        types: &mut Store,
        definitions: &Definitions,
        explicit: &[(ItemId, Pair)],
        predicates: impl IntoIterator<Item = (ItemId, Clause)>,
        families: super::families::Families,
    ) -> Result<Self, &'static str> {
        let mut inference = Inference::default();
        if !families.is_empty() {
            inference.result.projected = Some(Box::new(
                super::instances::Projected::with_families(families)?,
            ));
        }
        for definition in definitions {
            inference.fields(types, definitions, definition)?;
        }
        for &(item, pair) in explicit {
            inference.scan.clear()?;
            inference.scan.record(pair)?;
            inference.enqueue(types, item)?;
        }
        let mut clauses = super::clauses::Declarations::default();
        for (item, clause) in predicates {
            inference.scan.budget.charge()?;
            let Some(predicate) = clauses.outlives(types, clause, &mut inference.scan.budget)?
            else {
                continue;
            };
            if let Bound::Outlives(shorter) = predicate.bound {
                inference.scan.clear()?;
                inference.scan.type_bound(types, predicate.ty, shorter, 0)?;
                inference.enqueue(types, item)?;
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
struct Inference<'a> {
    result: Bounds,
    unique: HashSet<(ItemId, Requirement)>,
    work: Vec<(ItemId, Requirement)>,
    reverse: HashMap<ItemId, Vec<(ItemId, Use)>>,
    scan: Scan<'a>,
    projected: Option<Box<super::instances::Mapping>>,
}
impl Inference<'_> {
    fn fields(
        &mut self,
        types: &mut Store,
        definitions: &Definitions,
        definition: &crate::types::aggregate::Definition,
    ) -> Result<(), &'static str> {
        self.scan.budget.map(&mut self.result.items, 1)?;
        self.result
            .items
            .insert(definition.item, Inferred::default());
        self.scan.clear()?;
        for variant in definitions.variants(definition) {
            for &ty in types
                .list(variant.fields)
                .ok_or("missing outlives field templates")?
            {
                if self.scan.budget.1.is_some() && types.contains_projections(ty) != Some(false) {
                    return Err("demanded projected nominal outlives contracts require completion");
                }
                self.scan.visit(types, ty)?;
            }
        }
        // Naming a family in a field does not grant its own input conditions.
        // The declaration must prove them from explicit or independently inferred bounds.
        self.enqueue(types, definition.item)?;
        for &use_ in &self.scan.uses {
            self.scan.budget.charge()?;
            if use_.callable {
                return Err("nominal field template contains an unnameable function item");
            }
            if !self.reverse.contains_key(&use_.item) {
                self.scan.budget.map(&mut self.reverse, 1)?;
            }
            self.scan.budget.push(
                self.reverse.entry(use_.item).or_default(),
                (definition.item, use_),
            )?;
        }
        Ok(())
    }
    fn enqueue(&mut self, types: &mut Store, owner: ItemId) -> Result<(), &'static str> {
        if self.scan.budget.1.is_some() && !self.scan.projection_pairs.is_empty() {
            return Err("demanded projected nominal outlives contracts require completion");
        }
        for index in 0..self.scan.projection_pairs.len() {
            let (projection, shorter) = self.scan.projection_pairs[index];
            // Predicates mentioning a surrounding callable local remain deferred.
            // Captured predicates lift to the nominal owner's original scope.
            if let Some(projection) = self.projected.get_or_insert_with(Default::default).lift(
                types,
                projection,
                &mut self.scan.budget,
            )? {
                self.push(owner, Requirement::Projection((projection, shorter)))?;
            }
        }
        for index in 0..self.scan.pairs.len() {
            self.push(owner, Requirement::Region(self.scan.pairs[index]))?;
        }
        for index in 0..self.scan.parameter_pairs.len() {
            self.push(
                owner,
                Requirement::Parameter(self.scan.parameter_pairs[index]),
            )?;
        }
        Ok(())
    }
    fn push(&mut self, owner: ItemId, requirement: Requirement) -> Result<(), &'static str> {
        self.scan.budget.charge()?;
        if self
            .scan
            .budget
            .insert(&mut self.unique, (owner, requirement))?
        {
            self.scan
                .budget
                .pending(self.work.len().saturating_add(1))?;
            self.scan
                .budget
                .push(&mut self.work, (owner, requirement))?;
        }
        Ok(())
    }
    fn solve(mut self, types: &mut Store) -> Result<Bounds, &'static str> {
        while let Some((item, requirement)) = self.work.pop() {
            self.scan.budget.charge()?;
            let output = self
                .result
                .items
                .get_mut(&item)
                .ok_or("missing outlives nominal definition")?;
            match requirement {
                Requirement::Region(pair) => self.scan.budget.push(&mut output.regions, pair)?,
                Requirement::Parameter(pair) => {
                    self.scan.budget.push(&mut output.parameters, pair)?;
                }
                Requirement::Projection(pair) => self
                    .result
                    .projected
                    .get_or_insert_with(Default::default)
                    .templates
                    .entry(item)
                    .or_default()
                    .push(pair),
            }
            for index in 0..self.reverse.get(&item).map_or(0, Vec::len) {
                let (owner, use_) = self.reverse[&item][index];
                self.scan.budget.charge()?;
                self.scan.clear()?;
                match requirement {
                    Requirement::Region(pair) => {
                        if let Some(pair) = instantiate(types, use_, pair)? {
                            self.scan.record(pair)?;
                        }
                    }
                    Requirement::Parameter(pair) => self.scan.apply_parameter(types, use_, pair)?,
                    Requirement::Projection((projection, shorter)) => {
                        if let Some(shorter) = region(types, use_, shorter)? {
                            let projection = self
                                .projected
                                .get_or_insert_with(Default::default)
                                .apply(types, use_, projection, &mut self.scan.budget)?;
                            self.scan.projection_pairs.push((projection, shorter));
                        }
                    }
                }
                self.enqueue(types, owner)?;
            }
        }
        self.result.steps = self.scan.budget.0;
        Ok(self.result)
    }
}
