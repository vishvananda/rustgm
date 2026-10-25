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
            .imported_implementation(owner, predicate)
            .map_err(Error::unlocated)?;
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
        let Bound::Trait(application) = predicate.bound else {
            return Err(Error::unlocated(
                "imported impl effect requires its wider explicit profile",
            ));
        };
        if self
            .facts
            .types
            .trait_application(application)
            .is_none_or(|application| application.item.0 != role.definition.0)
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
        let registry = self
            .facts
            .generics
            .implementations
            .as_deref()
            .ok_or_else(|| Error::unlocated("imported conformance has no unpaid registry entry"))?;
        if registry
            .polarity(&self.facts.associated, definition)
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

    pub(super) fn validate_imported_trait_implementation(
        &mut self,
        definition: DefId,
        predicate: Predicate,
    ) -> Result<(), Error> {
        let environment = self.imported_receiver_environment(definition, predicate)?;
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
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| Error::unlocated("imported conformance registry disappeared"))?
            .complete(definition, Vec::new(), Vec::new(), Vec::new())
            .map_err(Error::unlocated)
    }
}
