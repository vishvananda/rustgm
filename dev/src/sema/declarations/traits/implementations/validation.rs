//! An impl's trait requirements are obligations, never extra assumptions about Self.
use super::methods::Members;
use super::{Checker, DefId, Error, NodeKey};
use crate::{
    sema::traits::implementations::overlap,
    types::{
        outlives::Queries,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};

impl Checker<'_> {
    pub(in crate::sema::declarations) fn validate_trait_implementations(
        &mut self,
    ) -> Result<(), Error> {
        let Some(registry) = &mut self.facts.generics.implementations else {
            return self.finish_constant_contracts();
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
                .generics
                .implementations
                .as_ref()
                .and_then(|r| r.header(owner_b))
                .ok_or_else(|| {
                    Error::at(
                        crate::resolve::unit::FileId(0),
                        0,
                        "missing trait impl owner",
                    )
                })?
                .source;
            let (possible, work) = overlap::possible(&mut self.facts, (a, owner_a), (b, owner_b))
                .map_err(|e| self.error(key, e))?;
            steps = steps.saturating_add(work);
            let possible = if possible {
                let (possible, work) =
                    overlap::iteration(&mut self.facts, (a, owner_a), (b, owner_b))
                        .map_err(|e| self.error(key, e))?;
                steps = steps.saturating_add(work);
                possible
            } else {
                false
            };
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
        let mut constants = self.constant_members.take_members(self.report, &self.facts);
        for (owner, source, predicate) in headers {
            if self.validate_iteration_blanket(owner)?
                || self.validate_collection_impl(owner)?
                || self.validate_range_impl(owner)?
                || self.validate_iterator_clone(owner)?
            {
                continue;
            }
            self.validate_reference_coherence(owner, source, predicate, &mut steps)?;
            let negative = self.negative_impl_contract(owner, source, predicate)?;
            if negative || matches!(predicate.bound, Bound::Auto(_)) {
                self.facts
                    .generics
                    .implementations
                    .as_mut()
                    .ok_or_else(|| Error::at(source.file, 0, "missing auto impl registry"))?
                    .complete(owner, Vec::new(), Vec::new(), Vec::new())
                    .map_err(|e| self.error(source, e))?;
                continue;
            }
            let types = self.trait_impl_types(owner, source, predicate, &types)?;
            let definitions =
                crate::sema::traits::conformance::Definitions::new(&self.facts, predicate, &types)
                    .map_err(|e| self.error(source, e))?;
            let methods =
                self.trait_impl_methods(owner, source, predicate, &members, &definitions)?;
            let constants =
                self.trait_impl_constants(owner, source, predicate, &mut constants, &definitions)?;
            self.facts
                .generics
                .implementations
                .as_mut()
                .ok_or_else(|| Error::at(source.file, 0, "missing trait impl registry"))?
                .complete(owner, methods, types, constants)
                .map_err(|e| self.error(source, e))?;
        }
        self.finish_constant_contracts()?;
        self.finish_iteration_contracts()?;
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| {
                Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "missing completed implementation inventory",
                )
            })?
            .finish_validation()
            .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))
    }

    fn negative_impl_contract(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
    ) -> Result<bool, Error> {
        let negative = self
            .facts
            .generics
            .implementations
            .as_ref()
            .ok_or_else(|| self.error(source, "missing implementation registry"))?
            .polarity(&self.facts.associated, owner)
            .map_err(|e| self.error(source, e))?
            == crate::sema::traits::implementations::Polarity::Negative;
        if negative {
            self.validate_negative_header(owner, source, predicate)?;
        }
        self.validate_trait_impl_contract(owner, source, predicate)?;
        Ok(negative)
    }

    fn finish_iteration_contracts(&mut self) -> Result<(), Error> {
        super::super::super::iteration::collections::finish_prepared(self.report, &mut self.facts)?;
        super::super::super::iteration::range_contracts::complete(
            self.report,
            &mut self.facts,
            true,
        )?;
        Ok(())
    }

    fn validate_reference_coherence(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
        steps: &mut usize,
    ) -> Result<(), Error> {
        let (possible, work) = overlap::reference_comparison(&mut self.facts, (predicate, owner))
            .map_err(|e| self.error(source, e))?;
        *steps = steps.saturating_add(work);
        if *steps > 1_000_000 {
            return Err(self.error(source, "trait implementation overlap work budget exceeded"));
        }
        if possible {
            return Err(self.error(
                source,
                "source implementation overlaps the canonical reference comparison implementation",
            ));
        }
        Ok(())
    }

    pub(in crate::sema::declarations) fn validate_trait_impl_contract(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
    ) -> Result<(), Error> {
        if self
            .facts
            .associated
            .owner(owner)
            .and_then(|id| self.facts.associated.get(id))
            .is_some_and(|owner| {
                matches!(
                    owner.kind,
                    crate::sema::associated::OwnerKind::Impl { constant: true, .. }
                )
            })
        {
            let (application, _) = predicate.bound.trait_parts().ok_or_else(|| {
                self.error(source, "const implementation requires a trait application")
            })?;
            let item = self
                .facts
                .types
                .trait_application(application)
                .ok_or_else(|| self.error(source, "const implementation trait is not prepared"))?
                .item;
            self.require_const_trait(source, DefId(item.0))?;
        }
        let environment = self.facts.generics.environments.get(&owner).cloned();
        let mut proof = self.declaration_proof(owner, environment);
        let requirements = impl_contract_obligations(&mut self.facts, predicate, &mut proof)
            .map_err(|e| self.error(source, e))?;
        self.validate_source_copy_impl(source, predicate, &mut proof)?;
        let coerced = self.validate_source_coercion_impl(source, predicate, &mut proof)?;
        self.retain_trait_proof(owner, source, &mut proof);
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        let mut constraints = self
            .trait_region_environment(owner, &mut queries)
            .map_err(|e| self.error(source, e))?;
        if let Some((longer, shorter)) = coerced {
            constraints
                .outlives(longer, shorter, 0)
                .map_err(|e| self.error(source, e.to_string()))?;
        }
        for &(a, b) in &requirements.regions {
            constraints
                .outlives(a, b, 0)
                .map_err(|e| self.error(source, e.to_string()))?;
        }
        for &requirement in requirements.required.iter().chain(&requirements.implied) {
            if let Clause::Predicate(Predicate {
                ty,
                bound: Bound::Outlives(shorter),
            }) = requirement
            {
                constraints
                    .require_type(
                        &mut queries,
                        ty,
                        shorter,
                        crate::types::relations::ProofScope::ROOT,
                        0,
                    )
                    .map_err(|e| self.error(source, e))?;
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

pub(in crate::sema::declarations) fn impl_contract_obligations(
    facts: &mut crate::sema::model::Declarations,
    predicate: Predicate,
    proof: &mut crate::sema::nominals::Proof,
) -> Result<std::sync::Arc<crate::sema::traits::Requirements>, String> {
    let requirements = if matches!(predicate.bound, Bound::Auto(_)) {
        std::sync::Arc::new(crate::sema::traits::Requirements::default())
    } else {
        facts
            .generics
            .traits
            .as_mut()
            .ok_or_else(|| "missing trait contracts".to_owned())?
            .requirements(&mut facts.types, predicate)?
    };
    for ty in std::iter::once(predicate.ty).chain(requirements.types.iter().copied()) {
        if !proof.check(facts, ty)?.well_formed {
            return Err("trait implementation header must be well-formed".to_owned());
        }
    }
    for &requirement in requirements.required.iter().chain(&requirements.implied) {
        let requirement = match requirement {
            Clause::Predicate(predicate) => {
                let predicate = proof.normalize_builtin(facts, predicate)?;
                if matches!(predicate.bound, Bound::Outlives(_)) {
                    continue;
                }
                Clause::Predicate(predicate)
            }
            Clause::ForAll(_) => requirement,
        };
        if !proof.prove_clause(facts, requirement)? {
            return Err(
                "trait implementation does not satisfy the trait's requirements".to_owned(),
            );
        }
    }
    Ok(requirements)
}
