//! Reprove supported imported receiver contracts; metadata is never conformance.
use super::{Checker, DefId, Error};
use crate::{
    sema::{associated::OwnerKind, traits::implementations::Polarity},
    support::bindings::LangItem,
    types::{
        outlives::Queries,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};

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
        let definitions =
            crate::sema::traits::conformance::Definitions::new(&self.facts, predicate, &[])
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
        let methods =
            crate::sema::crates::primitive_method_plan(&mut self.facts, definition, predicate)?;
        let environment =
            self.imported_receiver_environment(definition, predicate, methods.as_ref())?;
        if let Some(methods) = &methods {
            self.check_imported_primitive_methods(definition, predicate, methods)?;
        }
        self.validate_imported_requirements(definition, predicate, environment)?;
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| Error::unlocated("imported conformance registry disappeared"))?
            .complete(
                definition,
                methods.map_or_else(Vec::new, crate::sema::crates::PrimitiveMethods::into_links),
                Vec::new(),
                Vec::new(),
            )
            .map_err(Error::unlocated)
    }
}
