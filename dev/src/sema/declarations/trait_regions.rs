//! Trait argument lifetimes use the shared directed proof after nominal contracts complete.
use super::{Checker, DefId, Error, ItemId, NodeKey, Type};
use crate::types::{
    arguments::Argument,
    outlives::{Pair, ParameterPair, Queries},
    parameters::predicates::Bound,
    relations::Constraints,
};
pub(super) type Equalities = std::collections::HashMap<DefId, Vec<(NodeKey, Pair)>>;

impl Checker<'_> {
    pub(super) fn retain_trait_equalities(
        &mut self,
        owner: DefId,
        source: NodeKey,
        proof: &mut crate::sema::nominals::Proof,
    ) {
        let equalities = proof.take_trait_equalities();
        if !equalities.is_empty() {
            self.trait_equalities
                .entry(owner)
                .or_default()
                .extend(equalities.into_iter().map(|pair| (source, pair)));
        }
    }

    pub(super) fn validate_trait_regions(&mut self) -> Result<(), Error> {
        if self.facts.generics.traits.is_none() {
            return Ok(());
        }
        let mut owners: Vec<_> = self.facts.generics.predicates.keys().copied().collect();
        owners.extend(self.trait_equalities.keys().copied());
        owners.sort_unstable_by_key(|owner| owner.0);
        owners.dedup();
        let mut steps = 0_usize;
        for owner in owners {
            let requirements = self.owner_trait_requirements(owner)?;
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
            let equalities = self
                .trait_equalities
                .get(&owner)
                .map_or(&[][..], Vec::as_slice);
            if let Some(source) = pending
                .first()
                .map(|p| p.0)
                .or_else(|| equalities.first().map(|p| p.0))
            {
                let mut constraints = self
                    .trait_region_environment(owner, &mut queries)
                    .map_err(|e| self.error(source, e))?;
                for &(key, (left, right)) in equalities {
                    charge(&mut steps, 2).map_err(|e| self.error(key, e))?;
                    require_regions(&mut constraints, &[(left, right), (right, left)], &[])
                        .map_err(|e| self.error(key, e))?;
                }
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

    fn owner_trait_requirements(
        &mut self,
        owner: DefId,
    ) -> Result<Vec<(NodeKey, std::sync::Arc<crate::sema::traits::Requirements>)>, Error> {
        let predicates: Vec<_> = self
            .facts
            .generics
            .predicates
            .get(&owner)
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .filter(|p| matches!(p.value.bound, Bound::Trait(_)))
            .copied()
            .collect();
        let mut requirements = Vec::with_capacity(predicates.len());
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
        Ok(requirements)
    }

    pub(super) fn trait_region_environment(
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
        self.assume_trait_impl_inputs(owner, queries, &mut constraints)?;
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

    /// Impl input well-formedness implies lifetimes, never Copy or user-trait evidence.
    fn assume_trait_impl_inputs(
        &self,
        owner: DefId,
        queries: &mut Queries<'_>,
        constraints: &mut Constraints<'_>,
    ) -> Result<(), String> {
        let Some(header) = self
            .facts
            .associated
            .owner(owner)
            .and_then(|id| self.facts.associated.get(id))
            .and_then(|owner| {
                self.facts
                    .generics
                    .implementations
                    .as_ref()
                    .and_then(|registry| registry.header(owner.self_definition))
            })
        else {
            return Ok(());
        };
        let Bound::Trait(id) = header.predicate.bound else {
            return Err("invalid implementation trait predicate".into());
        };
        let application = self
            .facts
            .types
            .trait_application(id)
            .ok_or("missing implementation trait application")?;
        let arguments = self
            .facts
            .types
            .arguments(application.arguments)
            .ok_or("missing implementation trait arguments")?;
        for ty in std::iter::once(header.predicate.ty).chain(arguments.filter_map(|argument| {
            if let Argument::Type(ty) = argument {
                Some(ty)
            } else {
                None
            }
        })) {
            let implied = queries.symbolic_requirements(ty)?;
            for &(a, b) in implied.regions {
                constraints.assume(a, b).map_err(|e| e.to_string())?;
            }
            for &(id, shorter) in implied.parameters {
                constraints
                    .assume_parameter(id, shorter)
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
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
