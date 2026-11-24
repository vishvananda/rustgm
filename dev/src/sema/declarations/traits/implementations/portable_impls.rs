//! Reprove supported imported receiver contracts; metadata is never conformance.
use super::{Checker, DefId, Error};
use crate::{
    sema::associated::{ImplPolarity, Owner, OwnerKind},
    support::{bindings::LangItem, constness::Constness},
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
        member_plan: Option<&crate::sema::crates::ImportedMemberPlan>,
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
        let (application, effect) = predicate.bound.trait_parts().ok_or_else(|| {
            Error::unlocated("imported impl effect requires its wider explicit profile")
        })?;
        let trait_definition = self
            .facts
            .types
            .trait_application(application)
            .map(|application| DefId(application.item.0))
            .ok_or_else(|| Error::unlocated("imported impl lacks its actual trait application"))?;
        if effect != Constness::Ordinary {
            self.require_imported_const_operator(trait_definition, effect)?;
        }
        if let Some(plan) = member_plan {
            plan.require(&self.facts, definition, predicate)
                .map_err(Error::unlocated)?;
        } else if !self
            .facts
            .associated
            .imported_empty_module_trait(trait_definition)
            .map_err(Error::unlocated)?
        {
            self.require_imported_receiver_trait(trait_definition)?;
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

    fn require_imported_const_operator(
        &self,
        definition: DefId,
        effect: Constness,
    ) -> Result<(), Error> {
        if self.facts.source_bindings.is_none() {
            return Err(Error::unlocated(
                "imported const impl lacks its actual role catalog",
            ));
        }
        let role = self
            .facts
            .imported_const_operator_role(definition)
            .map_err(Error::unlocated)?;
        let Some(role) = role.filter(|_| {
            effect == Constness::Const
                && self.facts.associated.constant_trait(definition) == Some(true)
        }) else {
            return Err(Error::unlocated(
                "imported impl effect requires its wider explicit profile",
            ));
        };
        self.facts
            .associated
            .require_imported_role(role)
            .map_err(Error::unlocated)
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

    fn check_imported_members(
        &mut self,
        owner: DefId,
        predicate: Predicate,
        plan: &crate::sema::crates::ImportedMemberPlan,
    ) -> Result<(), Error> {
        plan.require(&self.facts, owner, predicate)
            .map_err(Error::unlocated)?;
        let members = &plan.members;
        self.check_imported_results(owner, predicate, &members.types, plan.declaration_policies)?;
        let definitions = crate::sema::traits::conformance::Definitions::new(
            &self.facts,
            predicate,
            &members.types,
        )
        .map_err(Error::unlocated)?;
        for &(expected, actual) in &members.methods {
            let Some(actual) = actual else {
                if self
                    .facts
                    .imported_conditional_member_trait(expected)
                    .map_err(Error::unlocated)?
                    .is_some()
                    && self
                        .facts
                        .declaration_header(expected)
                        .map_err(Error::unlocated)?
                        .is_some_and(|header| {
                            matches!(
                                header.body,
                                Some(crate::resolve::body_origin::BodyOrigin::Imported(_))
                            )
                        })
                {
                    // The trait owns the checked original. Dispatch reuses
                    // its full defining contract with the impl's actual Self.
                    continue;
                }
                return Err(Error::unlocated(
                    "v4 defaults require their wider conformance profile",
                ));
            };
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
        self.check_imported_constant_links(owner, predicate, &members.constants, &definitions)
    }
    /// Re-prove every imported declared-result definition in the impl's own
    /// rebuilt environment. A v14 result promise is instantiated with the
    /// actual impl's Self and trait inputs before it is proved.
    fn check_imported_results(
        &mut self,
        owner: DefId,
        predicate: Predicate,
        types: &[(DefId, DefId)],
        declaration_policies: bool,
    ) -> Result<(), Error> {
        if types.is_empty() {
            return Ok(());
        }
        let environment = self.facts.generics.environments.get(&owner).cloned();
        let mut proof = self.declaration_proof(owner, environment);
        let mut substitution = None;
        for &(expected, actual) in types {
            let ty = self.facts.definitions[actual.0 as usize]
                .filter(|_| self.facts.associated.type_definition(owner, expected) == Some(actual))
                .ok_or_else(|| {
                    Error::unlocated("imported associated definition changed its trait binding")
                })?;
            if self.facts.associated.families.contains_key(&expected)
                || self.facts.associated.families.contains_key(&actual)
            {
                self.check_imported_associated_family(owner, predicate, expected, actual)?;
                continue;
            }
            if self.facts.types.contains_regions(ty) != Some(false) {
                return Err(Error::unlocated(
                    "imported regional or family associated definitions require their wider profile",
                ));
            }
            let required = if declaration_policies {
                super::super::associated_types::mapped_result_bounds(
                    &mut self.facts,
                    predicate,
                    expected,
                    ty,
                    &mut substitution,
                )
                .map_err(Error::unlocated)?
            } else {
                self.direct_imported_result_bounds(expected, ty)?
            };
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
                if requirement
                    .is_outlives(&self.facts.types)
                    .map_err(Error::unlocated)?
                {
                    return Err(Error::unlocated(
                        "imported associated result lifetime bounds require their wider profile",
                    ));
                }
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
        if let Some(evidence) = proof.take_trait_proof().filter(|proof| !proof.is_empty()) {
            if self
                .facts
                .associated
                .schema(owner)
                .is_none_or(|schema| schema.captures == 0)
            {
                return Err(Error::unlocated(
                    "imported associated definition region evidence requires its wider profile",
                ));
            }
            let mut queries = Queries::new(
                &self.facts.types,
                &self.facts.lifetimes.nominal_requirements,
                &self.facts.lifetimes.callables,
            );
            let mut constraints = self
                .trait_region_environment(owner, &mut queries)
                .map_err(Error::unlocated)?;
            evidence
                .require(&mut constraints, &mut queries)
                .map_err(Error::unlocated)?;
            constraints
                .solve()
                .map_err(|e| Error::unlocated(e.to_string()))?;
        }
        Ok(())
    }
    fn direct_imported_result_bounds(
        &self,
        expected: DefId,
        ty: crate::types::TypeId,
    ) -> Result<Vec<Clause>, Error> {
        let contract = self
            .facts
            .associated
            .type_contract(expected)
            .ok_or_else(|| Error::unlocated("imported associated result lacks its contract"))?;
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
        Ok(required)
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
        let polarity = self.imported_polarity(definition, predicate)?;
        if polarity == ImplPolarity::Reservation {
            return self.validate_imported_reservation(definition);
        }
        if polarity == ImplPolarity::Negative {
            // As rustc does, a negative impl of a trait that is not auto only
            // denies its application: it has no requirement, member or
            // coverage proof, and it is never positive evidence.
            return self
                .facts
                .generics
                .implementations
                .as_mut()
                .ok_or_else(|| Error::unlocated("imported conformance registry disappeared"))?
                .complete(definition, Vec::new(), Vec::new(), Vec::new())
                .map_err(Error::unlocated);
        }
        let plan =
            crate::sema::crates::imported_member_plan(&mut self.facts, definition, predicate)?;
        let environment =
            self.imported_receiver_environment(definition, predicate, plan.as_ref())?;
        if let Some(plan) = &plan {
            self.check_imported_members(definition, predicate, plan)?;
        }
        self.validate_imported_requirements(definition, predicate, environment)?;
        let members = plan.map(|plan| plan.members).unwrap_or_default();
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| Error::unlocated("imported conformance registry disappeared"))?
            .complete(
                definition,
                members.methods,
                members.types,
                members.constants,
            )
            .map_err(Error::unlocated)
    }

    fn validate_imported_reservation(&mut self, definition: DefId) -> Result<(), Error> {
        if !self.facts.generics.environments.contains_key(&definition) {
            return Err(Error::unlocated(
                "imported reservation lacks its complete actual input environment",
            ));
        }
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| Error::unlocated("imported reservation registry disappeared"))?
            .complete(definition, Vec::new(), Vec::new(), Vec::new())
            .map_err(Error::unlocated)
    }

    /// Read the actual header after authenticating its trait and safety.
    fn imported_polarity(
        &self,
        definition: DefId,
        predicate: Predicate,
    ) -> Result<ImplPolarity, Error> {
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
                Error::unlocated("source-free conformance requires an actual imported impl")
            })?;
        self.facts
            .associated
            .imported_implementation(&self.facts.types, owner, predicate)
            .map(|(_, polarity)| polarity)
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
