//! Each unused definition must preserve both the declared domain and its result contract.
mod mapping;
use super::{Checker, DefId, Error, NodeKey};
use crate::{
    sema::{
        generics,
        nominals::Proof,
        projections::families::{self, Requirements},
    },
    types::{
        Kind, TypeId,
        arguments::Argument,
        outlives::Queries,
        parameters::{clauses::Clause, predicates::Predicate},
        relations::ProofScope,
    },
};

struct Contract {
    mapping: mapping::Mapping,
    assumptions: Requirements,
    required: Requirements,
    result: Option<(TypeId, TypeId)>,
}

impl Checker<'_> {
    pub(in crate::sema::declarations::traits) fn check_imported_associated_family(
        &mut self,
        owner: DefId,
        predicate: Predicate,
        expected: DefId,
        actual: DefId,
    ) -> Result<(), Error> {
        if !self.facts.associated.families.contains_key(&actual) {
            return Err(Error::unlocated(
                "imported associated definition lost its actual family",
            ));
        }
        let environment = self
            .facts
            .generics
            .environments
            .get(&owner)
            .cloned()
            .ok_or_else(|| {
                Error::unlocated("imported associated family has no actual impl environment")
            })?;
        for declared in [true, false] {
            let contract =
                self.imported_family_contract(owner, predicate, (expected, actual), declared)?;
            self.prove_imported_family(owner, &environment, &contract)?;
        }
        Ok(())
    }

    fn imported_family_contract(
        &mut self,
        owner: DefId,
        predicate: Predicate,
        (expected, actual): (DefId, DefId),
        declared: bool,
    ) -> Result<Contract, Error> {
        let mut mapping = mapping::build(
            &mut self.facts,
            owner,
            predicate,
            expected,
            actual,
            declared,
        )
        .map_err(Error::unlocated)?;
        let declaration = families::inputs(&mut self.facts, expected, &mut mapping.expected)
            .map_err(Error::unlocated)?;
        let implementation = families::inputs(&mut self.facts, actual, &mut mapping.actual)
            .map_err(Error::unlocated)?;
        if [&declaration, &implementation]
            .iter()
            .any(|premises| !premises.regions.is_empty())
        {
            return Err(Error::unlocated(
                "imported associated family requires its wider premise profile",
            ));
        }
        let (assumptions, mut required) = if declared {
            (declaration, implementation)
        } else {
            (implementation, declaration)
        };
        let result = if declared {
            None
        } else {
            let value = self.facts.definitions[actual.0 as usize].ok_or_else(|| {
                Error::unlocated("imported associated family lost its actual result")
            })?;
            let value = mapping
                .actual
                .apply(&mut self.facts.types, value)
                .map_err(Error::unlocated)?;
            self.family_lifetime_inputs(predicate, value, mapping.captures)
                .map_err(Error::unlocated)?;
            let mut substitution = Some(mapping.expected);
            required.clauses.extend(
                super::conformance::mapped_result_bounds(
                    &mut self.facts,
                    predicate,
                    expected,
                    value,
                    &mut substitution,
                )
                .map_err(Error::unlocated)?,
            );
            mapping.expected = substitution
                .ok_or_else(|| Error::unlocated("missing imported family substitution"))?;
            Some((value, value))
        };
        Ok(Contract {
            mapping,
            assumptions,
            required,
            result,
        })
    }

    fn prove_imported_family(
        &mut self,
        owner: DefId,
        environment: &std::sync::Arc<crate::types::parameters::environment::Environment>,
        contract: &Contract,
    ) -> Result<(), Error> {
        if contract.result.is_none() && contract.required.clauses.is_empty() {
            return Ok(());
        }
        let assumptions = &contract.assumptions.clauses;
        let has_value_params = contract
            .mapping
            .parameters
            .iter()
            .any(|parameter| !matches!(parameter.argument, Argument::Lifetime(_)));
        let (environment, expanded) = if !has_value_params && assumptions.is_empty() {
            (std::sync::Arc::clone(environment), None)
        } else if !has_value_params {
            let expanded = self
                .facts
                .generics
                .expanded_clauses(&mut self.facts.types, assumptions.clone())
                .map_err(Error::unlocated)?;
            let environment = std::sync::Arc::new(
                environment
                    .as_ref()
                    .clone()
                    .with_clauses(&self.facts.types, &expanded)
                    .map_err(Error::unlocated)?,
            );
            (environment, Some(expanded))
        } else {
            let mut inputs: Vec<Clause> = crate::sema::declaration_inputs::all(
                &self.facts.generics,
                self.facts.crates.as_deref(),
                owner,
            )
            .map_err(Error::unlocated)?
            .map(|input| input.value)
            .collect();
            inputs.extend(assumptions.iter().copied());
            let expanded = self
                .facts
                .generics
                .expanded_clauses(&mut self.facts.types, inputs)
                .map_err(Error::unlocated)?;
            let environment = self
                .facts
                .generics
                .declared_scope(
                    &self.facts.types,
                    self.facts
                        .generics
                        .parameters_in_scope(owner)
                        .copied()
                        .chain(contract.mapping.parameters.iter().copied()),
                )
                .and_then(|env| env.with_clauses(&self.facts.types, &expanded))
                .map_err(Error::unlocated)?;
            (std::sync::Arc::new(environment), Some(expanded))
        };
        let mut proof = Proof::for_declaration(Some(environment), contract.mapping.total, 1);
        if let Some((value, _)) = contract.result
            && !proof
                .check(&mut self.facts, value)
                .map_err(Error::unlocated)?
                .well_formed
        {
            return Err(Error::unlocated(
                "imported associated family result must be well-formed",
            ));
        }
        self.prove_family_results(None, &contract.required, &mut proof)?;
        let evidence = proof.take_trait_proof();
        let region_assumptions = expanded.as_deref().unwrap_or(assumptions);
        self.prove_family_regions(owner, None, contract, region_assumptions, evidence.as_ref())
    }

    pub(super) fn check_associated_family(
        &mut self,
        owner: DefId,
        key: NodeKey,
        predicate: Predicate,
        expected: DefId,
        actual: DefId,
    ) -> Result<(), Error> {
        // First prove that every declared application meets the impl's demands.
        // Then check the definition in its own environment, including formation
        // of the declared application. Missing repeated bounds are not inherited.
        for declared in [true, false] {
            let mut mapping = mapping::build(
                &mut self.facts,
                owner,
                predicate,
                expected,
                actual,
                declared,
            )
            .map_err(|e| self.error(key, e))?;
            let a = families::inputs(&mut self.facts, expected, &mut mapping.expected)
                .map_err(|e| self.error(key, e))?;
            let b = families::inputs(&mut self.facts, actual, &mut mapping.actual)
                .map_err(|e| self.error(key, e))?;
            let (assumptions, mut required) = if declared { (a, b) } else { (b, a) };
            let result = if declared {
                None
            } else {
                let ty = self.facts.definitions[actual.0 as usize]
                    .ok_or_else(|| self.error(key, "missing associated family definition"))?;
                let raw = self.facts.unreduced.definition(actual).unwrap_or(ty);
                let ty = mapping
                    .actual
                    .apply(&mut self.facts.types, ty)
                    .map_err(|e| self.error(key, e))?;
                let raw = mapping
                    .actual
                    .apply(&mut self.facts.types, raw)
                    .map_err(|e| self.error(key, e))?;
                let mut substitution = Some(mapping.expected);
                required.clauses.extend(self.associated_result_bounds(
                    key,
                    predicate,
                    expected,
                    ty,
                    &mut substitution,
                )?);
                mapping.expected = substitution
                    .ok_or_else(|| self.error(key, "missing family result substitution"))?;
                Some((ty, raw))
            };
            self.prove_family_contract(
                owner,
                key,
                predicate,
                &Contract {
                    mapping,
                    assumptions,
                    required,
                    result,
                },
            )?;
        }
        Ok(())
    }

    fn prove_family_contract(
        &mut self,
        owner: DefId,
        key: NodeKey,
        parent: Predicate,
        contract: &Contract,
    ) -> Result<(), Error> {
        let mut assumptions: Vec<_> = self
            .facts
            .generics
            .predicates_in_scope(owner)
            .copied()
            .collect();
        assumptions.extend(
            contract
                .assumptions
                .clauses
                .iter()
                .map(|&value| generics::Predicate { source: key, value }),
        );
        let (assumptions, _) = self
            .facts
            .generics
            .expanded_trait_bounds(&mut self.facts.types, assumptions)
            .map_err(|e| self.error(key, e))?;
        let clauses: Vec<_> = assumptions.iter().map(|p| p.value).collect();
        let environment = self
            .facts
            .generics
            .declared_scope(
                &self.facts.types,
                self.facts
                    .generics
                    .parameters_in_scope(owner)
                    .copied()
                    .chain(contract.mapping.parameters.iter().copied()),
            )
            .and_then(|env| env.with_clauses(&self.facts.types, &clauses))
            .map_err(|e| self.error(key, e))?;
        let mut proof = Proof::for_declaration(Some(environment.into()), contract.mapping.total, 1);
        if let Some((ty, raw)) = contract.result {
            self.reserved_type_parent(owner, key, parent, &mut proof)?;
            for ty in [raw, ty] {
                if !proof
                    .check(&mut self.facts, ty)
                    .map_err(|e| self.error(key, e))?
                    .well_formed
                {
                    return Err(self.error(key, "associated family definition must be well-formed"));
                }
            }
            self.family_lifetime_inputs(parent, raw, contract.mapping.captures)
                .map_err(|e| self.error(key, e))?;
        }
        self.prove_family_results(Some(key), &contract.required, &mut proof)?;
        let evidence = proof.take_trait_proof();
        self.prove_family_regions(owner, Some(key), contract, &clauses, evidence.as_ref())
    }

    fn prove_family_results(
        &mut self,
        site: Option<NodeKey>,
        required: &Requirements,
        proof: &mut Proof,
    ) -> Result<(), Error> {
        for &clause in &required.clauses {
            if !clause
                .is_outlives(&self.facts.types)
                .map_err(|e| self.family_error(site, e))?
                && !proof
                    .prove_clause(&mut self.facts, clause)
                    .map_err(|e| self.family_error(site, e))?
            {
                return Err(self.family_error(
                    site,
                    "associated family definition does not satisfy the declared contract",
                ));
            }
        }
        Ok(())
    }

    fn prove_family_regions(
        &self,
        owner: DefId,
        site: Option<NodeKey>,
        contract: &Contract,
        assumptions: &[Clause],
        evidence: Option<&crate::types::lifetime_proof::Obligations>,
    ) -> Result<(), Error> {
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        let mut constraints = self
            .trait_region_environment_count(owner, &mut queries, contract.mapping.total)
            .map_err(|e| self.family_error(site, e))?;
        for parameter in &contract.mapping.parameters {
            if let Argument::Type(ty) = parameter.argument
                && let Some(Kind::Parameter(id)) = self.facts.types.kind(ty)
            {
                constraints
                    .parameter(id)
                    .map_err(|e| self.family_error(site, e.to_string()))?;
            }
        }
        for &(a, b) in &contract.assumptions.regions {
            constraints
                .assume(a, b)
                .map_err(|e| self.family_error(site, e.to_string()))?;
        }
        constraints
            .assume_outlives_clauses(&mut queries, assumptions.iter().copied())
            .map_err(|e| self.family_error(site, e))?;
        for &(a, b) in &contract.required.regions {
            constraints
                .outlives(a, b, 0)
                .map_err(|e| self.family_error(site, e.to_string()))?;
        }
        for &clause in &contract.required.clauses {
            constraints
                .require_outlives_clause(&mut queries, clause, 0)
                .map_err(|e| self.family_error(site, e))?;
        }
        if let Some((ty, raw)) = contract.result {
            for ty in std::iter::once(raw).chain((raw != ty).then_some(ty)) {
                let required = queries
                    .symbolic_requirements(ty)
                    .map_err(|e| self.family_error(site, e))?;
                constraints
                    .require_requirements(required, ProofScope::ROOT, 0)
                    .map_err(|e| self.family_error(site, e))?;
            }
        }
        if let Some(evidence) = evidence {
            evidence
                .require(&mut constraints, &mut queries)
                .map_err(|e| self.family_error(site, e))?;
        }
        constraints.solve().map(|_| ()).map_err(|e| {
            self.family_error(
                site,
                format!("associated family lifetime contract is not satisfied: {e}"),
            )
        })
    }

    fn family_lifetime_inputs(
        &self,
        parent: Predicate,
        raw: TypeId,
        captures: u32,
    ) -> Result<(), &'static str> {
        let mut inputs = crate::types::bound_uses::Uses::constraining();
        inputs.predicate(&self.facts.types, parent)?;
        let mut outputs = crate::types::bound_uses::Uses::default();
        outputs.collect(&self.facts.types, raw)?;
        if outputs
            .indices
            .iter()
            .any(|&slot| u32::from(slot) < captures && !inputs.indices.contains(&slot))
        {
            return Err(
                "associated type uses an impl lifetime not constrained by Self or the trait arguments",
            );
        }
        Ok(())
    }

    fn family_error(&self, site: Option<NodeKey>, message: impl Into<String>) -> Error {
        if let Some(site) = site {
            self.error(site, message)
        } else {
            Error::unlocated(message)
        }
    }
}
