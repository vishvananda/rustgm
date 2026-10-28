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

pub(super) enum Conformance<'a> {
    Complete {
        methods: &'a Members,
        types: &'a super::super::associated_types::Members,
        constants: &'a mut super::constants::Members,
    },
    FunctionsOnly {
        methods: &'a Members,
        plan: &'a super::eligibility_profile::Plan,
    },
}

impl Checker<'_> {
    pub(in crate::sema::declarations) fn validate_trait_implementations(
        &mut self,
    ) -> Result<(), Error> {
        let Some(registry) = &mut self.facts.generics.implementations else {
            return self.finish_constant_contracts();
        };
        let pairs = registry.take_overlap_pairs();
        let mut headers = registry
            .headers()
            .map(|header| (header.owner, header.source, header.predicate))
            .collect::<Vec<_>>();
        let mut steps = 0_usize;
        if let Some(mut pending) = self.early_implementations.take() {
            pending
                .charge(&mut self.facts, pairs.len())
                .map_err(Error::unlocated)?;
            pending.pairs.append(pairs);
            let result = self.check_implementation_pairs(None, &mut pending);
            steps = pending.overlap_steps;
            self.early_implementations = Some(pending);
            result?;
        } else {
            for (a, owner_a, b, owner_b) in pairs {
                let key = self
                    .facts
                    .generics
                    .implementations
                    .as_ref()
                    .and_then(|r| r.header(owner_b))
                    .map(|h| h.source)
                    .ok_or_else(|| Error::unlocated("missing trait impl owner"))?;
                self.validate_impl_pair(key, (a, owner_a), (b, owner_b), &mut steps)?;
            }
        }
        crate::sema::specialization::prepare_graph(&mut self.facts, self.report)
            .map_err(|error| self.implementation_error(None, error))?;
        crate::sema::specialization::conformance_order(&self.facts, &mut headers)
            .map_err(|error| self.implementation_error(None, error))?;
        let members = Members::new(self.report, &self.facts);
        let types = super::super::associated_types::Members::new(self.report, &self.facts);
        let mut constants = self.constant_members.take_members(self.report, &self.facts);
        for (owner, source, predicate) in headers {
            let Some(source) = source else {
                if let Some(selected) = self.selected.as_deref_mut() {
                    // Private imported conformance is driven by the actual
                    // selected dependency, never by this unrelated eager walk.
                    selected.require_imported_implementation(self.report, &self.facts, owner)?;
                    continue;
                }
                self.validate_imported_trait_implementation(owner, predicate)?;
                continue;
            };
            self.validate_impl_owner(
                owner,
                source,
                predicate,
                Conformance::Complete {
                    methods: &members,
                    types: &types,
                    constants: &mut constants,
                },
                &mut steps,
            )?;
        }
        self.finish_constant_contracts()?;
        self.finish_iteration_contracts()?;
        if let Some(selected) = self.selected.as_deref_mut() {
            selected.require_imported_pending(self.report, &self.facts)?;
            return Ok(());
        }
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
            .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
        crate::sema::specialization::seal(&mut self.facts, self.report)
            .map_err(|error| self.implementation_error(None, error))
    }

    pub(super) fn validate_impl_owner(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
        inputs: Conformance<'_>,
        steps: &mut usize,
    ) -> Result<(), Error> {
        if self.validate_iteration_blanket(owner)?
            || self.validate_collection_impl(owner)?
            || self.validate_range_impl(owner)?
            || self.validate_iterator_clone(owner)?
        {
            return Ok(());
        }
        if self
            .facts
            .generics
            .implementations
            .as_ref()
            .is_some_and(|r| r.is_validated(owner))
        {
            return Ok(());
        }
        self.validate_reference_coherence(owner, source, predicate, steps)?;
        let negative = self.negative_impl_contract(owner, source, predicate)?;
        if negative || matches!(predicate.bound, Bound::Auto(_)) {
            self.facts
                .generics
                .implementations
                .as_mut()
                .ok_or_else(|| Error::at(source.file, 0, "missing auto impl registry"))?
                .complete(owner, Vec::new(), Vec::new(), Vec::new())
                .map_err(|e| self.error(source, e))?;
            return Ok(());
        }
        let (members, types, constants) = match inputs {
            Conformance::Complete {
                methods,
                types,
                constants,
            } => {
                let types = self.trait_impl_types(owner, source, predicate, types)?;
                let definitions = crate::sema::traits::conformance::Definitions::new(
                    &self.facts,
                    predicate,
                    &types,
                )
                .map_err(|e| self.error(source, e))?;
                let methods =
                    self.trait_impl_methods(owner, source, predicate, methods, &definitions)?;
                let constants =
                    self.trait_impl_constants(owner, source, predicate, constants, &definitions)?;
                (methods, types, constants)
            }
            Conformance::FunctionsOnly { methods, plan } => {
                plan.require_no_associated(&self.facts, owner)
                    .map_err(|e| self.error(source, e))?;
                let definitions =
                    crate::sema::traits::conformance::Definitions::new(&self.facts, predicate, &[])
                        .map_err(|e| self.error(source, e))?;
                let methods =
                    self.trait_impl_methods(owner, source, predicate, methods, &definitions)?;
                (methods, Vec::new(), Vec::new())
            }
        };
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| Error::at(source.file, 0, "missing trait impl registry"))?
            .complete(owner, members, types, constants)
            .map_err(|e| self.error(source, e))?;
        Ok(())
    }

    pub(super) fn validate_impl_pair(
        &mut self,
        key: Option<NodeKey>,
        first: (Predicate, DefId),
        second: (Predicate, DefId),
        steps: &mut usize,
    ) -> Result<(), Error> {
        if crate::sema::specialization::remembered(
            &mut self.facts,
            self.report,
            [first.1, second.1],
        )
        .map_err(|error| self.implementation_error(key, error))?
        {
            return Ok(());
        }
        let (possible, work) = overlap::possible(&mut self.facts, first, second)
            .map_err(|e| self.implementation_error(key, e))?;
        *steps = steps.saturating_add(work);
        let possible = if possible {
            let (possible, work) = overlap::iteration(&mut self.facts, first, second)
                .map_err(|e| self.implementation_error(key, e))?;
            *steps = steps.saturating_add(work);
            possible
        } else {
            false
        };
        if *steps > 1_000_000 {
            return Err(
                self.implementation_error(key, "trait implementation overlap work budget exceeded")
            );
        }
        if possible
            && crate::sema::specialization::requested(self.report, [first.1, second.1], steps)
                .map_err(|error| self.implementation_error(key, error))?
        {
            return crate::sema::specialization::compare(
                &mut self.facts,
                self.report,
                [first.1, second.1],
            )
            .map_err(|error| self.implementation_error(key, error));
        }
        if possible {
            return Err(self.implementation_error(
                key,
                "overlapping trait implementation headers require coherence proof",
            ));
        }
        Ok(())
    }

    pub(super) fn implementation_error(
        &self,
        source: Option<NodeKey>,
        message: impl Into<String>,
    ) -> Error {
        if let Some(source) = source {
            self.error(source, message)
        } else {
            Error::unlocated(message)
        }
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
            .polarity(&self.facts.types, &self.facts.associated, owner)
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
        self.validate_source_dispatch_impl(source, predicate, &mut proof)?;
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
    validate_impl_requirements(facts, predicate, proof, &requirements)?;
    Ok(requirements)
}

pub(in crate::sema) fn validate_impl_requirements(
    facts: &mut crate::sema::model::Declarations,
    predicate: Predicate,
    proof: &mut crate::sema::nominals::Proof,
    requirements: &crate::sema::traits::Requirements,
) -> Result<(), String> {
    validate_impl_requirements_prepared(facts, predicate, proof, requirements, None)
        .map_err(crate::sema::dispatch::Failure::message)
}
pub(in crate::sema) fn validate_impl_requirements_prepared(
    facts: &mut crate::sema::model::Declarations,
    predicate: Predicate,
    proof: &mut crate::sema::nominals::Proof,
    requirements: &crate::sema::traits::Requirements,
    mut preparation: Option<&mut dyn crate::sema::dispatch::Preparation>,
) -> Result<(), crate::sema::dispatch::Failure> {
    for ty in std::iter::once(predicate.ty).chain(requirements.types.iter().copied()) {
        if !proof.check(facts, ty)?.well_formed {
            return Err("trait implementation header must be well-formed".into());
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
            Clause::ForAll(_) | Clause::UnstableFeature(_) => requirement,
        };
        let proven = match &mut preparation {
            Some(service) => proof.prove_clause_prepared(facts, requirement, &mut **service)?,
            None => proof.prove_clause(facts, requirement)?,
        };
        if !proven {
            return Err("trait implementation does not satisfy the trait's requirements".into());
        }
    }
    Ok(())
}
