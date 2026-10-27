//! Selected impls pay canonical requirements and every actual member before completion.
use super::{Declarations, DefId, Error, Report, SelectedConsumer, s};
use crate::{
    sema::{nominals::Proof, traits::implementations::Polarity},
    types::{Kind, parameters::predicates::Predicate},
};
use std::sync::Arc;

impl SelectedConsumer<'_> {
    fn empty_implementation(
        &mut self,
        report: &Report,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<Arc<crate::types::parameters::environment::Environment>, Error> {
        self.require_imported_implementation(report, facts, definition)?;
        self.work.charge(6)?;
        let catalog = self.reservation.catalog();
        let id = catalog.declaration(definition).map_err(Error::unlocated)?;
        let (_, row, _) = catalog
            .declaration_coordinate(id)
            .map_err(Error::unlocated)?;
        let view = self.reservation.recipe_view(report, self.work)?;
        let rows = view.rows()?;
        let Some(s::HeaderRef::Implementation(index)) =
            rows.header(row).map_err(Error::unlocated)?
        else {
            return Err(Error::unlocated(
                "selected conformance lost its actual impl row",
            ));
        };
        let raw = rows.implementations[index as usize];
        let schema = facts.associated.schema(definition).ok_or_else(|| {
            Error::unlocated("selected conformance requires its actual pending schema")
        })?;
        let (environment, inputs) = self.selected.implementation_environment(
            facts,
            &super::SchemaAuthority {
                report,
                reservation: &self.reservation,
                completion: &self.completion,
            },
            definition,
            self.work,
        )?;
        // This is a positive completed-scope lookup, never map-miss emptiness.
        let own = facts.generics.parameters.get(&definition).ok_or_else(|| {
            Error::unlocated("selected conformance omitted its actual complete formals")
        })?;
        if raw.inputs.start != raw.inputs.end
            || raw.declared_inputs != 0
            || raw.region_captures != 0
            || raw.region_bounds.start != raw.region_bounds.end
            || inputs != 0
            || !own.is_empty()
            || facts.generics.parents.contains_key(&definition)
            || schema.captures != 0
            || !schema.bounds.is_empty()
        {
            return Err(Error::unlocated(
                "selected generic/regional impl conformance requires its complete input adapter",
            ));
        }
        Ok(environment)
    }

    pub(super) fn validate_appended_pairs(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        owner: DefId,
        mut pairs: std::ops::Range<usize>,
    ) -> Result<(), Error> {
        if let Some(index) = pairs.next() {
            self.work.charge(1)?;
            let (left, right) = facts
                .generics
                .implementations
                .as_deref()
                .ok_or_else(|| Error::unlocated("selected coherence lost its actual registry"))?
                .imported_overlap_pair(owner, index)
                .map_err(Error::unlocated)?;
            self.empty_implementation(report, facts, left.1)?;
            self.empty_implementation(report, facts, right.1)?;
            if left.0 == right.0 {
                return Err(Error::unlocated(
                    "conflicting selected trait implementations",
                ));
            }
            return Err(Error::unlocated(
                "selected coherence requires its actual nonidentical input/accounting adapter",
            ));
        }
        Ok(())
    }

    pub(super) fn validate_implementation(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        owner: DefId,
    ) -> Result<(), Error> {
        let environment = self.empty_implementation(report, facts, owner)?;
        let registry = facts
            .generics
            .implementations
            .as_deref()
            .ok_or_else(|| Error::unlocated("selected conformance lost its actual registry"))?;
        let header = registry.header(owner).ok_or_else(|| {
            Error::unlocated("selected conformance requires its actual pending header")
        })?;
        let predicate = header.predicate;
        if registry
            .polarity(&facts.types, &facts.associated, owner)
            .map_err(Error::unlocated)?
            != Polarity::Positive
        {
            return Err(Error::unlocated(
                "selected negative conformance requires its actual producer",
            ));
        }
        if registry.is_validated(owner) {
            return Ok(());
        }
        let methods = self.default_members(report, facts, owner)?;
        self.requirements(facts, predicate, environment)?;
        facts
            .generics
            .implementations
            .as_deref_mut()
            .ok_or_else(|| {
                Error::unlocated("selected conformance registry disappeared before completion")
            })?
            .complete(owner, methods, Vec::new(), Vec::new())
            .map_err(Error::unlocated)
    }

    fn default_members(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        owner: DefId,
    ) -> Result<Vec<(DefId, Option<DefId>)>, Error> {
        let links = {
            let catalog = self.reservation.catalog();
            let id = catalog.declaration(owner).map_err(Error::unlocated)?;
            let (_, row, _) = catalog
                .declaration_coordinate(id)
                .map_err(Error::unlocated)?;
            let view = self.reservation.recipe_view(report, self.work)?;
            let rows = view.rows()?;
            let Some(s::HeaderRef::Implementation(index)) =
                rows.header(row).map_err(Error::unlocated)?
            else {
                return Err(Error::unlocated(
                    "selected members lost their actual impl row",
                ));
            };
            let raw = rows.implementations[index as usize];
            let links = raw
                .members
                .slice(&rows.member_links)
                .map_err(Error::unlocated)?;
            self.work.charge(links.len().saturating_add(1))?;
            self.work.reserve(
                links
                    .len()
                    .saturating_mul(size_of::<(DefId, Option<DefId>)>()),
            )?;
            let mut result = Vec::with_capacity(links.len());
            for link in links {
                let member = link.trait_member.ok_or_else(|| {
                    Error::unlocated("selected trait member lost its actual declaration link")
                })?;
                if member.artifact != rows.policy.provider {
                    return Err(Error::unlocated(
                        "selected member requires its actual dependency producer",
                    ));
                }
                result.push((
                    view.definition(member.declaration, self.work)?,
                    link.implementation
                        .map(|actual| view.definition(actual, self.work))
                        .transpose()?,
                ));
            }
            result
        };
        for &(expected, actual) in &links {
            self.definition(report, facts, expected)?;
            let method = crate::sema::traits::conformance::Method::imported(facts, expected)
                .map_err(Error::unlocated)?;
            if method.definition != expected {
                return Err(Error::unlocated(
                    "selected member changed its canonical declaration",
                ));
            }
            if let Some(actual) = actual {
                self.definition(report, facts, actual)?;
                return Err(Error::unlocated(
                    "selected override requires its actual member conformance producer",
                ));
            }
            if facts
                .declaration_header(expected)
                .map_err(Error::unlocated)?
                .is_none_or(|header| header.body.is_none())
            {
                return Err(Error::unlocated(
                    "implementation is missing a required trait method",
                ));
            }
        }
        Ok(links)
    }

    fn requirements(
        &mut self,
        facts: &mut Declarations,
        predicate: Predicate,
        environment: Arc<crate::types::parameters::environment::Environment>,
    ) -> Result<(), Error> {
        let requirements = {
            let work = std::cell::RefCell::new(&mut *self.work);
            facts
                .generics
                .traits
                .as_deref_mut()
                .ok_or_else(|| {
                    Error::unlocated("selected conformance requires actual trait contracts")
                })?
                .requirements_counted(&mut facts.types, predicate, &|steps, bytes| {
                    let mut work = work
                        .try_borrow_mut()
                        .map_err(|_| "reentrant selected conformance accounting")?;
                    work.charge(steps)
                        .map_err(|_| "selected conformance work budget exceeded")?;
                    work.reserve_raw(bytes)
                })
                .map_err(Error::unlocated)?
        };
        // The immediate closed implementation has no declared lifetime premise.
        // Do not execute a broader regional/nominal proof behind that boundary.
        let roots = std::iter::once(predicate.ty).chain(requirements.types.iter().copied());
        for ty in roots {
            self.work.charge(1)?;
            if !matches!(facts.types.kind(ty), Some(Kind::Primitive(_))) {
                return Err(Error::unlocated(
                    "selected nominal conformance requires its actual formation accounting",
                ));
            }
        }
        self.work.charge(
            requirements
                .required
                .len()
                .saturating_add(requirements.implied.len()),
        )?;
        if !requirements.regions.is_empty() || !requirements.results.is_empty() {
            return Err(Error::unlocated(
                "selected conformance requires its actual regional/result producer",
            ));
        }
        for clause in requirements.required.iter().chain(&requirements.implied) {
            let crate::types::parameters::clauses::Clause::Predicate(requirement) = *clause else {
                return Err(Error::unlocated(
                    "selected conformance requires its actual quantified producer",
                ));
            };
            if !matches!(facts.types.kind(requirement.ty), Some(Kind::Primitive(_)))
                || (requirement.bound != crate::types::parameters::predicates::Bound::Sized
                    && !crate::sema::compiler_items::sizing_predicate(facts, requirement)
                        .map_err(Error::unlocated)?)
            {
                return Err(Error::unlocated(
                    "selected conformance requires its actual non-sizing requirement producer",
                ));
            }
        }
        self.work.reserve(
            size_of::<Proof>().saturating_add(size_of::<crate::types::parameters::shape::Shapes>()),
        )?;
        let mut proof = Proof::for_declaration(Some(environment), 0, 1);
        let result = crate::sema::declarations::validate_impl_requirements(
            facts,
            predicate,
            &mut proof,
            &requirements,
        );
        self.work.charge(proof.formation_steps())?;
        result.map_err(Error::unlocated)?;
        let evidence = proof.take_trait_proof();
        if !requirements.regions.is_empty()
            || evidence.as_ref().is_some_and(|proof| !proof.is_empty())
        {
            return Err(Error::unlocated(
                "selected conformance requires its actual retained regional proof",
            ));
        }
        Ok(())
    }
}
