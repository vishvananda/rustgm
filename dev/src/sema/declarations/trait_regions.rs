//! Trait argument lifetimes use the shared directed proof after nominal contracts complete.
use super::{Checker, DefId, Error, ItemId, NodeKey, Type};
use crate::sema::{nominals::Proof, traits::obligations::Obligations};
use crate::types::{
    arguments::Argument,
    outlives::{Pair, ParameterPair, ProjectionPair, Queries},
    parameters::predicates::Bound,
    relations::Constraints,
};
pub(super) type Proofs = std::collections::HashMap<DefId, Vec<(NodeKey, Obligations)>>;

impl Checker<'_> {
    pub(super) fn declaration_proof(
        &self,
        owner: DefId,
        environment: Option<std::sync::Arc<crate::types::parameters::environment::Environment>>,
    ) -> Proof {
        let first = self
            .trait_proofs
            .get(&owner)
            .and_then(|proofs| proofs.last())
            .map_or(1, |(_, proof)| proof.regions.end);
        Proof::for_declaration(environment, self.trait_region_count(owner), first)
    }

    pub(super) fn retain_trait_proof(&mut self, owner: DefId, source: NodeKey, proof: &mut Proof) {
        if let Some(obligations) = proof.take_trait_proof() {
            self.trait_proofs
                .entry(owner)
                .or_default()
                .push((source, obligations));
        }
    }

    pub(super) fn validate_trait_regions(&mut self) -> Result<(), Error> {
        if self.facts.generics.traits.is_none() {
            return Ok(());
        }
        let mut owners: Vec<_> = self.facts.generics.predicates.keys().copied().collect();
        owners.extend(self.trait_proofs.keys().copied());
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
                let mut projections = Vec::new();
                for &ty in &requirements.types {
                    let well_formed = queries
                        .symbolic_requirements(ty)
                        .map_err(|e| self.error(source, e))?;
                    charge(
                        &mut steps,
                        well_formed.scalars.regions.len()
                            + well_formed.scalars.parameters.len()
                            + well_formed.projections.len(),
                    )
                    .map_err(|e| self.error(source, e))?;
                    regions.extend_from_slice(well_formed.scalars.regions);
                    parameters.extend_from_slice(well_formed.scalars.parameters);
                    projections.extend_from_slice(well_formed.projections);
                }
                for predicate in &requirements.required {
                    let predicate = predicate.ordinary().map_err(|e| self.error(source, e))?;
                    if let Bound::Outlives(shorter) = predicate.bound {
                        let parts = queries
                            .components(predicate.ty)
                            .map_err(|e| self.error(source, e))?;
                        charge(&mut steps, parts.len()).map_err(|e| self.error(source, e))?;
                        regions.extend(
                            parts
                                .scalars
                                .regions
                                .iter()
                                .map(|&longer| (longer, shorter)),
                        );
                        parameters.extend(parts.scalars.parameters.iter().map(|&id| (id, shorter)));
                        projections.extend(parts.projections.iter().map(|&p| (p, shorter)));
                    }
                }
                if !regions.is_empty() || !parameters.is_empty() || !projections.is_empty() {
                    pending.push((source, regions, parameters, projections));
                }
            }
            let proofs = self.trait_proofs.get(&owner).map_or(&[][..], Vec::as_slice);
            if let Some(source) = pending
                .first()
                .map(|p| p.0)
                .or_else(|| proofs.first().map(|p| p.0))
            {
                let mut constraints = self
                    .trait_region_environment(owner, &mut queries)
                    .map_err(|e| self.error(source, e))?;
                for (key, proof) in proofs {
                    charge(
                        &mut steps,
                        proof.equalities.len() * 2 + proof.outlives.len() + proof.types.len(),
                    )
                    .map_err(|e| self.error(*key, e))?;
                    proof
                        .require(&mut constraints, &mut queries)
                        .map_err(|e| self.error(*key, e))?;
                }
                for (key, regions, parameters, projections) in pending {
                    require_regions(&mut constraints, &regions, &parameters, &projections)
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
            .map(|p| p.value.ordinary().map(|value| (p.source, value)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
        let mut requirements = Vec::with_capacity(predicates.len());
        for (source, predicate) in predicates {
            if !matches!(predicate.bound, Bound::Trait(_)) {
                continue;
            }
            let contract = self
                .facts
                .generics
                .traits
                .as_mut()
                .ok_or_else(|| Error::at(source.file, 0, "missing trait contracts"))?
                .requirements(&mut self.facts.types, predicate)
                .map_err(|e| self.error(source, e))?;
            requirements.push((source, contract));
        }
        Ok(requirements)
    }

    pub(super) fn trait_region_environment(
        &self,
        owner: DefId,
        queries: &mut Queries<'_>,
    ) -> Result<Constraints<'_>, String> {
        self.trait_region_environment_count(owner, queries, self.trait_region_count(owner))
    }

    pub(super) fn trait_region_environment_count(
        &self,
        owner: DefId,
        queries: &mut Queries<'_>,
        count: u32,
    ) -> Result<Constraints<'_>, String> {
        let function = self.facts.lifetimes.functions.get(&owner);
        let schema = self.facts.associated.schema(owner);
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
                constraints.assume_well_formed(queries, ty)?;
            }
        }
        for predicate in self.facts.generics.predicates_in_scope(owner) {
            let predicate = predicate.value.ordinary()?;
            if let Bound::Outlives(shorter) = predicate.bound {
                constraints.assume_type_outlives(queries, predicate.ty, shorter)?;
            }
        }
        Ok(constraints)
    }

    fn trait_region_count(&self, owner: DefId) -> u32 {
        self.facts
            .lifetimes
            .functions
            .get(&owner)
            .map(|f| f.total)
            .or_else(|| self.facts.associated.schema(owner).map(|s| s.captures))
            .unwrap_or_else(|| {
                self.nominal_regions
                    .get(&ItemId(owner.0))
                    .and_then(|context| context.binder)
                    .map_or(0, |binder| self.regions.count(binder))
            })
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
            constraints.assume_well_formed(queries, ty)?;
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
    projections: &[ProjectionPair],
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
    for &(projection, shorter) in projections {
        constraints
            .require_projection(
                projection,
                shorter,
                crate::types::relations::ProofScope::ROOT,
                0,
            )
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
