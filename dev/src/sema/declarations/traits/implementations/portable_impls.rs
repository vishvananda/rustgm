//! Reprove supported imported receiver contracts; metadata is never conformance.
use super::{Checker, DefId, Error};
use crate::{
    sema::{
        associated::{Owner, OwnerKind},
        traits::implementations::Polarity,
    },
    support::bindings::LangItem,
    types::{
        outlives::Queries,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};

#[cfg(test)]
#[path = "../../../../../tests/invariants/polarity_fences.rs"]
mod fence_tests;
#[cfg(test)]
#[path = "../../../../../tests/invariants/polarity_scaling.rs"]
mod scaling_tests;

impl Checker<'_> {
    fn imported_receiver_environment(
        &self,
        definition: DefId,
        predicate: Predicate,
        methods: Option<&crate::sema::crates::PrimitiveMethods>,
    ) -> Result<std::sync::Arc<crate::types::parameters::environment::Environment>, Error> {
        let owner = self
            .facts
            .associated
            .owner(definition)
            .and_then(|id| self.facts.associated.get(id))
            .copied()
            .ok_or_else(|| Error::unlocated("imported conformance has no actual owner"))?;
        if !matches!(owner.kind, OwnerKind::ImportedImpl(_)) || owner.source.is_some() {
            return Err(Error::unlocated(
                "source-free conformance requires an actual imported impl",
            ));
        }
        self.facts
            .associated
            .imported_implementation(&self.facts.types, owner, predicate)
            .map_err(Error::unlocated)?;
        let Bound::Trait(application) = predicate.bound else {
            return Err(Error::unlocated(
                "imported impl effect requires its wider explicit profile",
            ));
        };
        let trait_definition = self
            .facts
            .types
            .trait_application(application)
            .map(|application| DefId(application.item.0))
            .ok_or_else(|| Error::unlocated("imported impl lacks its actual trait application"))?;
        if let Some(methods) = methods {
            methods
                .require(&self.facts, definition, predicate)
                .map_err(Error::unlocated)?;
        } else if !self
            .facts
            .associated
            .imported_empty_module_trait(trait_definition)
            .map_err(Error::unlocated)?
        {
            self.require_imported_receiver_trait(trait_definition)?;
        }
        let registry = self
            .facts
            .generics
            .implementations
            .as_deref()
            .ok_or_else(|| Error::unlocated("imported conformance has no unpaid registry entry"))?;
        if registry
            .polarity(&self.facts.types, &self.facts.associated, definition)
            .map_err(Error::unlocated)?
            != Polarity::Positive
        {
            return Err(Error::unlocated(
                "negative imported conformance requires its wider profile",
            ));
        }
        self.facts
            .generics
            .environments
            .get(&definition)
            .cloned()
            .ok_or_else(|| {
                Error::unlocated("imported impl lacks its complete actual input environment")
            })
    }

    fn require_imported_receiver_trait(&self, definition: DefId) -> Result<(), Error> {
        let role = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| {
                Error::unlocated("imported receiver conformance lacks its actual role catalog")
            })?
            .validated_role(LangItem::LegacyReceiver)
            .map_err(Error::unlocated)?
            .ok_or_else(|| {
                Error::unlocated("imported receiver conformance lacks its actual receiver role")
            })?;
        self.facts
            .associated
            .require_imported_role(role)
            .map_err(Error::unlocated)?;
        if definition != role.definition
            || self
                .facts
                .associated
                .owner(role.definition)
                .and_then(|id| self.facts.associated.owner_members(id))
                .is_none_or(|members| !members.is_empty())
        {
            return Err(Error::unlocated(
                "imported impl family requires its wider complete conformance consumer",
            ));
        }
        Ok(())
    }

    fn check_imported_primitive_methods(
        &mut self,
        owner: DefId,
        predicate: Predicate,
        plan: &crate::sema::crates::PrimitiveMethods,
    ) -> Result<(), Error> {
        plan.require(&self.facts, owner, predicate)
            .map_err(Error::unlocated)?;
        self.check_imported_results(owner, plan.types())?;
        let definitions = crate::sema::traits::conformance::Definitions::new(
            &self.facts,
            predicate,
            plan.types(),
        )
        .map_err(Error::unlocated)?;
        for &(expected, actual) in plan.links() {
            let actual = actual.ok_or_else(|| {
                Error::unlocated("v4 defaults require their wider conformance profile")
            })?;
            if crate::sema::declarations::is_final_trait_method(self.report, &self.facts, expected)?
                || self.facts.methods.receivers.contains_key(&expected)
                    != self.facts.methods.receivers.contains_key(&actual)
            {
                return Err(Error::unlocated(
                    "primitive imported method changed its final/receiver contract",
                ));
            }
            let expected =
                crate::sema::traits::conformance::Method::imported(&mut self.facts, expected)
                    .map_err(Error::unlocated)?;
            let actual =
                crate::sema::traits::conformance::Method::imported(&mut self.facts, actual)
                    .map_err(Error::unlocated)?;
            crate::sema::traits::conformance::check_methods(
                &mut self.facts,
                owner,
                predicate,
                expected,
                actual,
                &definitions,
            )
            .map_err(Error::unlocated)?;
        }
        Ok(())
    }
    /// Re-prove every imported declared-result definition in the impl's own
    /// rebuilt environment: well formed, region-free and meeting each direct
    /// promise of its trait declaration. The provider's conformance is unused.
    fn check_imported_results(
        &mut self,
        owner: DefId,
        types: &[(DefId, DefId)],
    ) -> Result<(), Error> {
        if types.is_empty() {
            return Ok(());
        }
        let environment = self.facts.generics.environments.get(&owner).cloned();
        let mut proof = self.declaration_proof(owner, environment);
        for &(expected, actual) in types {
            let contract = self
                .facts
                .associated
                .type_contract(expected)
                .ok_or_else(|| Error::unlocated("imported associated result lacks its contract"))?;
            let ty = self.facts.definitions[actual.0 as usize]
                .filter(|_| self.facts.associated.type_definition(owner, expected) == Some(actual))
                .ok_or_else(|| {
                    Error::unlocated("imported associated definition changed its trait binding")
                })?;
            if self.facts.associated.families.contains_key(&expected)
                || self.facts.associated.families.contains_key(&actual)
                || self.facts.types.contains_regions(ty) != Some(false)
            {
                return Err(Error::unlocated(
                    "imported regional or family associated definitions require their wider profile",
                ));
            }
            let mut required = Vec::with_capacity(contract.bounds.len());
            for &clause in &contract.bounds {
                let Clause::Predicate(Predicate { ty: subject, bound }) = clause else {
                    return Err(Error::unlocated(
                        "imported quantified associated bounds require their wider profile",
                    ));
                };
                // A direct bound naming no trait argument needs no impl-frame
                // substitution; every other form keeps its source-only mapping.
                let closed = match bound {
                    Bound::Sized | Bound::Copy | Bound::Clone => true,
                    Bound::Trait(application) => self
                        .facts
                        .types
                        .trait_application(application)
                        .is_some_and(|application| {
                            application.arguments == crate::types::GenericArgs::EMPTY
                        }),
                    _ => false,
                };
                if subject != contract.projection || !closed {
                    return Err(Error::unlocated(
                        "imported indirect associated bounds require their wider profile",
                    ));
                }
                required.push(Clause::Predicate(Predicate { ty, bound }));
            }
            if !proof
                .check(&mut self.facts, ty)
                .map_err(Error::unlocated)?
                .well_formed
            {
                return Err(Error::unlocated(
                    "imported associated definition must be well-formed",
                ));
            }
            for requirement in required {
                if !proof
                    .prove_clause(&mut self.facts, requirement)
                    .map_err(Error::unlocated)?
                {
                    return Err(Error::unlocated(
                        "imported associated definition does not satisfy its declared bounds",
                    ));
                }
            }
        }
        if proof
            .take_trait_proof()
            .is_some_and(|evidence| !evidence.is_empty())
        {
            return Err(Error::unlocated(
                "imported associated definition region evidence requires its wider profile",
            ));
        }
        Ok(())
    }
    fn validate_imported_requirements(
        &mut self,
        definition: DefId,
        predicate: Predicate,
        environment: std::sync::Arc<crate::types::parameters::environment::Environment>,
    ) -> Result<(), Error> {
        let mut proof = self.declaration_proof(definition, Some(environment));
        let requirements = super::impl_contract_obligations(&mut self.facts, predicate, &mut proof)
            .map_err(Error::unlocated)?;
        let evidence = proof.take_trait_proof();
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        let mut constraints = self
            .trait_region_environment(definition, &mut queries)
            .map_err(Error::unlocated)?;
        if let Some(evidence) = evidence {
            evidence
                .require(&mut constraints, &mut queries)
                .map_err(Error::unlocated)?;
        }
        for &(longer, shorter) in &requirements.regions {
            constraints
                .outlives(longer, shorter, 0)
                .map_err(|e| Error::unlocated(e.to_string()))?;
        }
        for requirement in requirements.required.iter().chain(&requirements.implied) {
            if let Clause::Predicate(Predicate {
                ty,
                bound: Bound::Outlives(shorter),
            }) = *requirement
            {
                constraints
                    .require_type(
                        &mut queries,
                        ty,
                        shorter,
                        crate::types::relations::ProofScope::ROOT,
                        0,
                    )
                    .map_err(Error::unlocated)?;
            }
        }
        constraints
            .solve()
            .map_err(|e| Error::unlocated(e.to_string()))?;
        Ok(())
    }
    pub(super) fn validate_imported_trait_implementation(
        &mut self,
        definition: DefId,
        predicate: Predicate,
    ) -> Result<(), Error> {
        if let Some(owner) = self.imported_automatic_owner(definition, predicate)? {
            return self.validate_imported_auto_implementation(definition, predicate, owner);
        }
        let methods =
            crate::sema::crates::primitive_method_plan(&mut self.facts, definition, predicate)?;
        let environment =
            self.imported_receiver_environment(definition, predicate, methods.as_ref())?;
        if let Some(methods) = &methods {
            self.check_imported_primitive_methods(definition, predicate, methods)?;
        }
        self.validate_imported_requirements(definition, predicate, environment)?;
        let types_and_links = methods.map_or_else(
            || (Vec::new(), Vec::new()),
            |plan| (plan.types().to_vec(), plan.into_links()),
        );
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| Error::unlocated("imported conformance registry disappeared"))?
            .complete(definition, types_and_links.1, types_and_links.0, Vec::new())
            .map_err(Error::unlocated)
    }

    /// The actual imported owner of an impl whose trait is an imported auto
    /// trait, or `None` for every other imported impl. That trait is
    /// re-checked, by name, as its own packet's version-thirteen declaration.
    fn imported_automatic_owner(
        &self,
        definition: DefId,
        predicate: Predicate,
    ) -> Result<Option<Owner>, Error> {
        let Bound::Trait(application) = predicate.bound else {
            return Ok(None);
        };
        let trait_definition = self
            .facts
            .types
            .trait_application(application)
            .map(|application| DefId(application.item.0))
            .ok_or_else(|| Error::unlocated("imported impl lacks its actual trait application"))?;
        if !self
            .facts
            .associated
            .owner(trait_definition)
            .and_then(|id| self.facts.associated.get(id))
            .is_some_and(|owner| {
                owner.source.is_none() && matches!(owner.kind, OwnerKind::Trait { auto_: true, .. })
            })
        {
            return Ok(None);
        }
        self.facts
            .associated
            .imported_automatic_trait(trait_definition)
            .map_err(Error::unlocated)?;
        let owner = self
            .facts
            .associated
            .owner(definition)
            .and_then(|id| self.facts.associated.get(id))
            .copied()
            .filter(|owner| {
                matches!(owner.kind, OwnerKind::ImportedImpl(_)) && owner.source.is_none()
            })
            .ok_or_else(|| {
                Error::unlocated(
                    "imported automatic implementation requires an actual imported impl",
                )
            })?;
        Ok(Some(owner))
    }

    /// Version thirteen: admission is not conformance. The row's safety,
    /// empty members and exact auto trait, the local auto Self rule and a
    /// negative nominal header's coverage are each re-proved before the
    /// registry completes the impl as validated in its own polarity. That
    /// polarity is this row's: registration read it (`imported_status`), and
    /// the registry re-reads it whenever it answers one.
    fn validate_imported_auto_implementation(
        &mut self,
        definition: DefId,
        predicate: Predicate,
        owner: Owner,
    ) -> Result<(), Error> {
        let negative = self
            .facts
            .associated
            .imported_automatic_implementation(&self.facts.types, owner, predicate)
            .map_err(Error::unlocated)?;
        self.validate_imported_automatic_domain(definition, predicate.ty)?;
        if negative {
            self.validate_imported_negative_header(definition, predicate)?;
        } else {
            let environment = self
                .facts
                .generics
                .environments
                .get(&definition)
                .cloned()
                .ok_or_else(|| {
                    Error::unlocated("imported impl lacks its complete actual input environment")
                })?;
            self.validate_imported_requirements(definition, predicate, environment)?;
        }
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| Error::unlocated("imported conformance registry disappeared"))?
            .complete(definition, Vec::new(), Vec::new(), Vec::new())
            .map_err(Error::unlocated)
    }
}
