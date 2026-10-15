//! Trait argument lifetimes use the shared directed proof after nominal contracts complete.
use super::{Checker, DefId, Error, ItemId, Type};
use crate::types::{
    arguments::Argument,
    outlives::{Pair, ParameterPair, Queries},
    parameters::predicates::Bound,
    relations::Constraints,
};

impl Checker<'_> {
    pub(super) fn validate_trait_regions(&mut self) -> Result<(), Error> {
        if self.facts.generics.traits.is_none() {
            return Ok(());
        }
        let mut owners: Vec<_> = self.facts.generics.predicates.keys().copied().collect();
        owners.sort_unstable_by_key(|owner| owner.0);
        let mut steps = 0_usize;
        for owner in owners {
            let predicates: Vec<_> = self.facts.generics.predicates[&owner]
                .iter()
                .filter(|p| matches!(p.value.bound, Bound::Trait(_)))
                .copied()
                .collect();
            let mut requirements = Vec::new();
            for predicate in predicates {
                let contract = self
                    .facts
                    .generics
                    .traits
                    .as_mut()
                    .ok_or_else(|| Error::at(predicate.source.file, 0, "missing trait contracts"))?
                    .requirements(&mut self.facts.types, predicate.value)
                    .map_err(|e| self.error(predicate.source, e))?;
                requirements.push((predicate.source, contract));
            }
            let mut queries = Queries::new(
                &self.facts.types,
                &self.facts.lifetimes.nominal_requirements,
                &self.facts.lifetimes.callables,
            );
            let mut pending = Vec::new();
            for (source, requirements) in requirements {
                charge(
                    &mut steps,
                    1 + requirements.regions.len()
                        + requirements.types.len()
                        + requirements.required.len(),
                )
                .map_err(|e| self.error(source, e))?;
                let mut regions = requirements.regions.clone();
                let mut parameters = Vec::new();
                for &ty in &requirements.types {
                    let well_formed = queries
                        .symbolic_requirements(ty)
                        .map_err(|e| self.error(source, e))?;
                    charge(
                        &mut steps,
                        well_formed.regions.len() + well_formed.parameters.len(),
                    )
                    .map_err(|e| self.error(source, e))?;
                    regions.extend_from_slice(well_formed.regions);
                    parameters.extend_from_slice(well_formed.parameters);
                }
                for predicate in &requirements.required {
                    if let Bound::Outlives(shorter) = predicate.bound {
                        let parts = queries
                            .components(predicate.ty)
                            .map_err(|e| self.error(source, e))?;
                        charge(&mut steps, parts.len()).map_err(|e| self.error(source, e))?;
                        regions.extend(parts.regions.iter().map(|&longer| (longer, shorter)));
                        parameters.extend(parts.parameters.iter().map(|&id| (id, shorter)));
                    }
                }
                if !regions.is_empty() || !parameters.is_empty() {
                    pending.push((source, regions, parameters));
                }
            }
            if let Some(&(source, _, _)) = pending.first() {
                let mut constraints = self
                    .trait_region_environment(owner, &mut queries)
                    .map_err(|e| self.error(source, e))?;
                for (key, regions, parameters) in pending {
                    require_regions(&mut constraints, &regions, &parameters)
                        .map_err(|e| self.error(key, e))?;
                }
                constraints.solve().map_err(|e| {
                    self.error(
                        source,
                        format!("trait argument lifetime requirement is not satisfied: {e}"),
                    )
                })?;
            }
        }
        Ok(())
    }

    fn trait_region_environment(
        &self,
        owner: DefId,
        queries: &mut Queries<'_>,
    ) -> Result<Constraints<'_>, String> {
        let function = self.facts.lifetimes.functions.get(&owner);
        let schema = self.facts.associated.schema(owner);
        let nominal = self.nominal_regions.get(&ItemId(owner.0)).copied();
        let count = function
            .map(|f| f.total)
            .or_else(|| schema.map(|s| s.captures))
            .unwrap_or_else(|| {
                nominal
                    .and_then(|c| c.binder)
                    .map_or(0, |b| self.regions.count(b))
            });
        let mut constraints = Constraints::new(
            &self.facts.types,
            &self.facts.lifetimes.variances,
            count,
            1_000_000,
        )
        .map_err(|e| e.to_string())?;
        for parameter in self.facts.generics.parameters_in_scope(owner) {
            if let Argument::Type(ty) = parameter.argument
                && let Some(Type::Parameter(id)) = self.facts.types.kind(ty)
            {
                constraints.parameter(id).map_err(|e| e.to_string())?;
            }
        }
        let bounds = function
            .map(|f| f.bounds.as_slice())
            .or_else(|| schema.map(|s| s.bounds.as_slice()));
        for &(longer, shorter) in bounds.into_iter().flatten() {
            constraints
                .assume(longer, shorter)
                .map_err(|e| e.to_string())?;
        }
        if let Some(ty) = self.facts.definitions[owner.0 as usize] {
            if function.is_some() {
                let Some(Type::FunctionItem { signature, .. }) = self.facts.types.kind(ty) else {
                    return Err("missing trait-bound function signature".into());
                };
                constraints.assume_signature(queries, signature)?;
            } else if self.facts.aggregates.get(ItemId(owner.0)).is_some() {
                let requirements = queries.symbolic_requirements(ty)?;
                for &(a, b) in requirements.regions {
                    constraints.assume(a, b).map_err(|e| e.to_string())?;
                }
                for &(id, shorter) in requirements.parameters {
                    constraints
                        .assume_parameter(id, shorter)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        for predicate in self.facts.generics.predicates_in_scope(owner) {
            if let Bound::Outlives(shorter) = predicate.value.bound {
                constraints.assume_type_outlives(queries, predicate.value.ty, shorter)?;
            }
        }
        Ok(constraints)
    }
}

fn charge(steps: &mut usize, count: usize) -> Result<(), &'static str> {
    *steps = steps.saturating_add(count);
    if *steps > 1_000_000 {
        return Err("trait region validation work budget exceeded");
    }
    Ok(())
}

fn require_regions(
    constraints: &mut Constraints<'_>,
    regions: &[Pair],
    parameters: &[ParameterPair],
) -> Result<(), String> {
    for &(longer, shorter) in regions {
        constraints
            .outlives(longer, shorter, 0)
            .map_err(|e| e.to_string())?;
    }
    for &(id, shorter) in parameters {
        constraints
            .parameter_outlives(id, shorter, 0)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
