//! An impl's trait requirements are obligations, never extra assumptions about Self.
use super::{Checker, DefId, Error, NodeKey};
use crate::{
    sema::{nominals::Proof, traits::implementations::overlap},
    types::{
        outlives::Queries,
        parameters::predicates::{Bound, Predicate},
    },
};
use std::collections::HashSet;

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
        let members: HashSet<_> = self
            .facts
            .associated
            .members()
            .iter()
            .map(|member| member.owner)
            .collect();
        for (owner, source, predicate) in headers {
            self.validate_trait_impl_contract(owner, source, predicate)?;
            let Bound::Trait(id) = predicate.bound else {
                return Err(self.error(source, "invalid trait implementation predicate"));
            };
            let trait_ = self
                .facts
                .types
                .trait_application(id)
                .ok_or_else(|| self.error(source, "missing trait implementation application"))?
                .item;
            let own_members = self
                .facts
                .associated
                .owner(owner)
                .is_some_and(|id| members.contains(&id));
            let trait_members = self
                .facts
                .associated
                .owner(DefId(trait_.0))
                .is_some_and(|id| members.contains(&id));
            if own_members || trait_members {
                // Member signature conformance and checked default dispatch must
                // precede publishing any method-bearing implementation.
                return Err(self.error(
                    source,
                    "trait implementation requires conformance and selection semantics",
                ));
            }
            self.facts
                .generics
                .implementations
                .as_mut()
                .ok_or_else(|| Error::at(source.file, 0, "missing trait impl registry"))?
                .complete(owner, Vec::new())
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
        let mut proof = Proof::new(environment);
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
        self.retain_trait_equalities(owner, source, &mut proof);
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
