//! Match each associated type once, then prove its definition in the impl environment.
use super::{Checker, DefId, Error, NodeKey};
use crate::support::hash::HashMap;
use crate::{
    resolve::{expansion::Report, model::DefKind},
    sema::model::Declarations,
    types::{
        outlives::Queries,
        parameters::{
            clauses::{Clause, Quantified},
            predicates::{Bound, Predicate},
        },
    },
};

#[derive(Default)]
pub(in super::super) struct Members {
    owners: HashMap<DefId, Vec<DefId>>,
}
impl Members {
    pub(in crate::sema::declarations::traits) fn prepare_source_owner(
        &mut self,
        report: &Report,
        facts: &Declarations,
        owner: DefId,
    ) -> Result<(), &'static str> {
        use crate::{sema::associated::OwnerKind, support::retained};
        crate::sema::header_attributes::charge_source_contract_query(facts, 1)
            .map_err(|_| "source associated type query work exhausted")?;
        facts
            .source_bindings
            .as_deref()
            .ok_or("source type index has no provider")?
            .require_provider_report(report)?;
        let id = facts
            .associated
            .owner(owner)
            .ok_or("source type owner is missing")?;
        let actual = facts
            .associated
            .get(id)
            .ok_or("source type owner disappeared")?;
        if actual.self_definition != owner
            || !matches!(
                actual.kind,
                OwnerKind::Trait { .. } | OwnerKind::Impl { .. }
            )
        {
            return Err("type member index requires an actual source owner");
        }
        actual.source_owner()?;
        let members = facts
            .associated
            .owner_members(id)
            .ok_or("source type members are missing")?;
        if let Some(cached) = self.owners.get(&owner) {
            crate::sema::header_attributes::charge_source_contract_query(
                facts,
                members.len().saturating_add(1),
            )
            .map_err(|_| "source associated type query work exhausted")?;
            return Self::require_source_members(report, facts, id, members, cached);
        }
        let growth = retained::Growth::map::<DefId, Vec<DefId>>(
            self.owners.len(),
            self.owners.capacity(),
            1,
        );
        crate::sema::header_attributes::charge_source_retention(
            facts,
            members.len().saturating_add(growth.work).saturating_add(1),
            growth
                .bytes
                .saturating_add(members.len().saturating_mul(size_of::<DefId>())),
        )
        .map_err(|_| "source associated type storage or work exhausted")?;
        let mut types = Vec::new();
        types
            .try_reserve_exact(members.len())
            .map_err(|_| "source type member allocation failed")?;
        for &member in members {
            if facts.associated.owner(member) != Some(id) {
                return Err("source type member belongs to another owner");
            }
            if report
                .resolution
                .definitions
                .get(member.0 as usize)
                .ok_or("source type member definition is missing")?
                .kind
                == DefKind::TypeAlias
            {
                types.push(member);
            }
        }
        self.owners.insert(owner, types);
        Ok(())
    }
    fn require_source_members(
        report: &Report,
        facts: &Declarations,
        owner: crate::sema::associated::OwnerId,
        members: &[DefId],
        cached: &[DefId],
    ) -> Result<(), &'static str> {
        let mut expected = cached.iter();
        for member in members {
            if facts.associated.owner(*member) != Some(owner) {
                return Err("source type member belongs to another owner");
            }
            if report
                .resolution
                .definitions
                .get(member.0 as usize)
                .ok_or("source type member definition is missing")?
                .kind
                == DefKind::TypeAlias
                && expected.next() != Some(member)
            {
                return Err("source type member index changed its complete bucket");
            }
        }
        if expected.next().is_some() {
            return Err("source type member index contains foreign declarations");
        }
        Ok(())
    }
    #[cfg(test)]
    pub(in crate::sema::declarations::traits) fn indexed_storage(&self) -> (usize, usize) {
        (
            self.owners.len(),
            crate::support::retained::map(&self.owners)
                + self
                    .owners
                    .values()
                    .map(crate::support::retained::vector)
                    .sum::<usize>(),
        )
    }
    pub(in super::super) fn new(report: &Report, facts: &Declarations) -> Self {
        let mut result = Self::default();
        for (member, owner) in facts.associated.declaration_members() {
            let definition = &report.resolution.definitions[member.0 as usize];
            if definition.kind == DefKind::TypeAlias {
                result.owners.entry(owner).or_default().push(member);
            }
        }
        result
    }
}

impl Checker<'_> {
    pub(in crate::sema::declarations::traits) fn trait_impl_types(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
        members: &Members,
    ) -> Result<Vec<(DefId, DefId)>, Error> {
        let Some((id, _)) = predicate.bound.trait_parts() else {
            return Err(self.error(source, "missing impl trait predicate"));
        };
        let app = self
            .facts
            .types
            .trait_application(id)
            .ok_or_else(|| self.error(source, "missing impl trait application"))?;
        let trait_item = DefId(app.item.0);
        let trait_owner = self
            .facts
            .associated
            .declaration_owner(trait_item)
            .ok_or_else(|| self.error(source, "missing trait owner"))?;
        let impl_owner = self
            .facts
            .associated
            .declaration_owner(owner)
            .ok_or_else(|| self.error(source, "missing impl owner"))?;
        if !members.owners.contains_key(&trait_owner) && !members.owners.contains_key(&impl_owner) {
            return Ok(Vec::new());
        }
        // Header preparation already checked every actual type identifier and
        // duplicate mapping. Reuse those canonical links for the result proofs.
        let environment = self.facts.generics.environments.get(&owner).cloned();
        let mut proof = self.declaration_proof(owner, environment);
        let mut result = Vec::new();
        let mut lifetimes = Vec::new();
        let mut substitution: Option<crate::types::substitute::Bound> = None;
        for &expected in members.owners.get(&trait_owner).into_iter().flatten() {
            let actual = self
                .facts
                .associated
                .type_definition(impl_owner, expected)
                .ok_or_else(|| {
                    self.error(
                        source,
                        "implementation is missing a required associated type",
                    )
                })?;
            let key = self.report.resolution.definitions[actual.0 as usize]
                .node
                .ok_or_else(|| self.error(source, "missing associated type definition"))?;
            if self.facts.associated.families.contains_key(&expected)
                || self.facts.associated.families.contains_key(&actual)
            {
                self.check_associated_family(owner, key, predicate, expected, actual)?;
                result.push((expected, actual));
                continue;
            }
            let ty = self.facts.definitions[actual.0 as usize]
                .ok_or_else(|| self.error(key, "missing associated type definition type"))?;
            if !proof
                .check(&mut self.facts, ty)
                .map_err(|e| self.error(key, e))?
                .well_formed
            {
                return Err(self.error(key, "associated type definition must be well-formed"));
            }
            for requirement in
                self.associated_result_bounds(key, predicate, expected, ty, &mut substitution)?
            {
                if requirement
                    .is_outlives(&self.facts.types)
                    .map_err(|e| self.error(key, e))?
                {
                    lifetimes.push((key, requirement));
                    continue;
                }
                if !proof
                    .prove_clause(&mut self.facts, requirement)
                    .map_err(|e| self.error(key, e))?
                {
                    return Err(self.error(
                        key,
                        "associated type definition does not satisfy its declared bounds",
                    ));
                }
            }
            // The mapping is published only with the rest of the checked implementation.
            result.push((expected, actual));
        }
        self.associated_type_regions(owner, source, predicate, &result, &lifetimes)?;
        self.retain_trait_proof(owner, source, &mut proof);
        Ok(result)
    }

    /// Instantiate result contracts with one shared mapping, leaving actual types in impl scope.
    pub(in crate::sema::declarations) fn associated_result_bounds(
        &mut self,
        key: NodeKey,
        predicate: Predicate,
        expected: DefId,
        ty: crate::types::TypeId,
        substitution: &mut Option<crate::types::substitute::Bound>,
    ) -> Result<Vec<Clause>, Error> {
        crate::sema::declarations::comparisons::LibraryContracts {
            report: self.report,
            facts: &mut self.facts,
        }
        .associated_result_bounds(key, predicate, expected, ty, substitution)
    }

    fn associated_type_regions(
        &self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
        definitions: &[(DefId, DefId)],
        lifetimes: &[(NodeKey, Clause)],
    ) -> Result<(), Error> {
        let mut inputs = crate::types::bound_uses::Uses::constraining();
        inputs
            .predicate(&self.facts.types, predicate)
            .map_err(|e| self.error(source, e))?;
        let mut outputs = crate::types::bound_uses::Uses::default();
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        let mut constraints = self
            .trait_region_environment(owner, &mut queries)
            .map_err(|e| self.error(source, e))?;
        for &(expected, definition) in definitions {
            if self.facts.associated.families.contains_key(&expected)
                || self.facts.associated.families.contains_key(&definition)
            {
                continue;
            }
            let key = self.report.resolution.definitions[definition.0 as usize]
                .node
                .ok_or_else(|| self.error(source, "missing associated type source"))?;
            let ty = self.facts.definitions[definition.0 as usize]
                .ok_or_else(|| self.error(key, "missing associated type definition type"))?;
            outputs
                .collect(
                    &self.facts.types,
                    self.facts.unreduced.definition(definition).unwrap_or(ty),
                )
                .map_err(|e| self.error(key, e))?;
            if !outputs.indices.is_subset(&inputs.indices) {
                return Err(self.error(key, "associated type uses an impl lifetime not constrained by Self or the trait arguments"));
            }
            let required = queries
                .symbolic_requirements(ty)
                .map_err(|e| self.error(key, e))?;
            constraints
                .require_requirements(required, crate::types::relations::ProofScope::ROOT, 0)
                .map_err(|e| self.error(key, e))?;
        }
        for &(key, clause) in lifetimes {
            constraints
                .require_outlives_clause(&mut queries, clause, 0)
                .map_err(|e| self.error(key, e))?;
        }
        constraints.solve().map(|_| ()).map_err(|e| {
            self.error(
                source,
                format!("associated type lifetime requirement is not satisfied: {e}"),
            )
        })
    }
}

impl crate::sema::declarations::comparisons::LibraryContracts<'_> {
    pub(in crate::sema::declarations) fn associated_result_bounds(
        &mut self,
        key: NodeKey,
        predicate: Predicate,
        expected: DefId,
        ty: crate::types::TypeId,
        substitution: &mut Option<crate::types::substitute::Bound>,
    ) -> Result<Vec<Clause>, Error> {
        let contract = self
            .facts
            .associated
            .type_contract(expected)
            .ok_or_else(|| self.error(key, "missing associated type contract"))?;
        let projection = contract.projection;
        let mut required = contract.bounds.clone();
        for requirement in &mut required {
            *requirement = self.map_result_bound(
                key,
                predicate,
                (projection, ty),
                *requirement,
                substitution,
            )?;
        }
        Ok(required)
    }

    /// The concrete result is already in impl scope. Insert it beneath a retained
    /// clause binder after mapping the declaration's inputs, never before.
    fn map_result_bound(
        &mut self,
        key: NodeKey,
        parent: Predicate,
        result_types: (crate::types::TypeId, crate::types::TypeId),
        clause: Clause,
        substitution: &mut Option<crate::types::substitute::Bound>,
    ) -> Result<Clause, Error> {
        let (projection, actual) = result_types;
        let (mut requirement, count) = match clause {
            Clause::UnstableFeature(_) => return Ok(clause),
            Clause::Predicate(value) => (value, None),
            Clause::ForAll(id) => {
                let value = self
                    .facts
                    .types
                    .quantified(id)
                    .ok_or_else(|| self.error(key, "invalid associated result binder"))?;
                (value.predicate, Some(value.bound_regions))
            }
        };
        let depth = u16::from(count.is_some());
        let formal = self
            .facts
            .types
            .shift_type(projection, depth)
            .map_err(|e| self.error(key, e))?
            .0;
        let direct = requirement.ty == formal;
        if !direct || !matches!(requirement.bound, Bound::Copy | Bound::Clone | Bound::Sized) {
            if substitution.is_none() {
                *substitution = Some(
                    crate::sema::traits::results::substitution(self.facts, parent)
                        .map_err(|e| self.error(key, e))?,
                );
            }
            if direct {
                requirement.ty = crate::types::TypeId::UNIT;
            }
            requirement = substitution
                .as_mut()
                .ok_or_else(|| self.error(key, "missing associated result substitution"))?
                .predicate_inside(&mut self.facts.types, requirement, depth)
                .map_err(|e| self.error(key, e))?;
        }
        if direct {
            requirement.ty = self
                .facts
                .types
                .shift_type(actual, depth)
                .map_err(|e| self.error(key, e))?
                .0;
        }
        Ok(if let Some(bound_regions) = count {
            Clause::ForAll(
                self.facts
                    .types
                    .intern_quantified(Quantified {
                        bound_regions,
                        predicate: requirement,
                    })
                    .map_err(|e| self.error(key, e))?,
            )
        } else {
            Clause::Predicate(requirement)
        })
    }
}
