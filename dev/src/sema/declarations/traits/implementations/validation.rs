//! An impl's trait requirements are obligations, never extra assumptions about Self.
use super::methods::Members;
use super::{Checker, DefId, Error, NodeKey};
use crate::{
    sema::traits::implementations::overlap,
    types::{
        outlives::Queries,
        parameters::predicates::{Bound, Predicate},
    },
};

impl Checker<'_> {
    pub(in crate::sema::declarations) fn validate_trait_implementations(
        &mut self,
    ) -> Result<(), Error> {
        let Some(registry) = &mut self.facts.generics.implementations else {
            return Ok(());
        };
        let pairs = registry.take_overlap_pairs();
        let headers = registry
            .headers()
            .map(|header| (header.owner, header.source, header.predicate))
            .collect::<Vec<_>>();
        let mut steps = 0_usize;
        for (a, owner_a, b, owner_b) in pairs {
            let key = self
                .facts
                .associated
                .owner(owner_b)
                .and_then(|id| self.facts.associated.get(id))
                .ok_or_else(|| {
                    Error::at(
                        crate::resolve::unit::FileId(0),
                        0,
                        "missing trait impl owner",
                    )
                })?
                .source;
            let (possible, work) = overlap::possible(&mut self.facts, (a, owner_a), (b, owner_b))
                .map_err(|e| self.error(key, e.to_string()))?;
            steps = steps.saturating_add(work);
            if steps > 1_000_000 {
                return Err(self.error(key, "trait implementation overlap work budget exceeded"));
            }
            if possible {
                return Err(self.error(
                    key,
                    "overlapping trait implementation headers require coherence proof",
                ));
            }
        }
        let members = Members::new(self.report, &self.facts);
        let types = super::super::associated_types::Members::new(self.report, &self.facts);
        for (owner, source, predicate) in headers {
            self.validate_trait_impl_contract(owner, source, predicate)?;
            let types = self.trait_impl_types(owner, source, predicate, &types)?;
            let methods = self.trait_impl_methods(owner, source, predicate, &members)?;
            self.facts
                .generics
                .implementations
                .as_mut()
                .ok_or_else(|| Error::at(source.file, 0, "missing trait impl registry"))?
                .complete(owner, methods, types)
                .map_err(|e| self.error(source, e))?;
        }
        Ok(())
    }

    fn validate_trait_impl_contract(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
    ) -> Result<(), Error> {
        let requirements = self
            .facts
            .generics
            .traits
            .as_mut()
            .ok_or_else(|| Error::at(source.file, 0, "missing trait contracts"))?
            .requirements(&mut self.facts.types, predicate)
            .map_err(|e| self.error(source, e))?;
        let environment = self.facts.generics.environments.get(&owner).cloned();
        let mut proof = self.declaration_proof(owner, environment);
        for ty in std::iter::once(predicate.ty).chain(requirements.types.iter().copied()) {
            if !proof
                .check(&mut self.facts, ty)
                .map_err(|e| self.error(source, e))?
                .well_formed
            {
                return Err(self.error(source, "trait implementation header must be well-formed"));
            }
        }
        for &requirement in requirements.required.iter().chain(&requirements.implied) {
            if !matches!(requirement.bound, Bound::Outlives(_))
                && !proof
                    .prove(&mut self.facts, requirement)
                    .map_err(|e| self.error(source, e))?
            {
                return Err(self.error(
                    source,
                    "trait implementation does not satisfy the trait's requirements",
                ));
            }
        }
        self.retain_trait_proof(owner, source, &mut proof);
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        let mut constraints = self
            .trait_region_environment(owner, &mut queries)
            .map_err(|e| self.error(source, e))?;
        for &(a, b) in &requirements.regions {
            constraints
                .outlives(a, b, 0)
                .map_err(|e| self.error(source, e.to_string()))?;
        }
        for requirement in requirements.required.iter().chain(&requirements.implied) {
            if let Bound::Outlives(shorter) = requirement.bound {
                let parts = queries
                    .components(requirement.ty)
                    .map_err(|e| self.error(source, e.to_string()))?;
                for &longer in parts.regions {
                    constraints
                        .outlives(longer, shorter, 0)
                        .map_err(|e| self.error(source, e.to_string()))?;
                }
                for &id in parts.parameters {
                    constraints
                        .parameter_outlives(id, shorter, 0)
                        .map_err(|e| self.error(source, e.to_string()))?;
                }
            }
        }
        constraints.solve().map(|_| ()).map_err(|e| {
            self.error(
                source,
                format!("trait implementation lifetime requirement is not satisfied: {e}"),
            )
        })
    }
}
