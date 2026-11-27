//! Reconstruct normal-form evidence in the imported impl's actual declaration scope.
use super::{Checker, DefId, Error};
use crate::{
    sema::{
        callee_premises::Inputs,
        crates::{charge_source_header_query, pay_source_headers, source_header_payments},
        declaration_inputs,
        generics::elaborated_certificates,
        model::Declarations,
        nominals::Proof,
    },
    types::{
        GenericArgs,
        arguments::Argument,
        lifetime_proof::Obligations,
        outlives::Queries,
        parameters::{clauses::Clause, environment::Environment, predicates::Predicate},
    },
};
use std::sync::Arc;

struct Candidate {
    clause: Clause,
    predicate: Predicate,
    arguments: GenericArgs,
    locals: Option<u32>,
}

impl Checker<'_> {
    pub(super) fn reprove_imported_premises(
        &mut self,
        owner: DefId,
        environment: Arc<Environment>,
    ) -> Result<Arc<Environment>, Error> {
        let candidates = projected_inputs(&self.facts, owner).map_err(Error::unlocated)?;
        if candidates.is_empty() {
            return Ok(environment);
        }
        let mut proof = self.declaration_proof(owner, Some(Arc::clone(&environment)));
        let mut normal = Vec::new();
        for candidate in candidates {
            if candidate
                .reprove(&mut proof, &mut self.facts)
                .map_err(Error::unlocated)?
            {
                normal.push(candidate.clause);
            }
        }
        pay_source_headers(
            &mut self.facts,
            proof.formation_steps().saturating_add(proof.nested_work()),
            0,
        )
        .map_err(Error::unlocated)?;
        if normal.is_empty() {
            return Ok(environment);
        }
        if let Some(evidence) = proof.take_trait_proof() {
            self.prove_imported_premise_regions(owner, &evidence)?;
        }
        normal.extend(inherited_premises(&self.facts, owner, &normal)?);
        let budget =
            source_header_payments(self.facts.crates.as_deref_mut()).map_err(Error::unlocated)?;
        let result = environment
            .with_reproved_normal_premises(&self.facts.types, &normal, &mut |steps, bytes| {
                budget.pay(steps, bytes)
            })
            .map_err(Error::unlocated)?;
        Ok(Arc::new(result))
    }

    fn prove_imported_premise_regions(
        &mut self,
        owner: DefId,
        evidence: &Obligations,
    ) -> Result<(), Error> {
        for (_, ty) in evidence.type_roots() {
            Inputs::prepare_type(&mut self.facts, ty).map_err(Error::unlocated)?;
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
            .map_err(|error| Error::unlocated(error.to_string()))?;
        Ok(())
    }
}

fn inherited_premises(
    facts: &Declarations,
    owner: DefId,
    normal: &[Clause],
) -> Result<Vec<Clause>, Error> {
    let linkage = facts.crates.as_deref();
    let declared = declaration_inputs::declared_own(&facts.generics, linkage, owner)
        .map_err(Error::unlocated)?
        .len();
    let all =
        declaration_inputs::own_view(&facts.generics, linkage, owner).map_err(Error::unlocated)?;
    if all.len() == declared {
        return Ok(Vec::new());
    }
    charge_source_header_query(facts, normal.len().saturating_add(all.len()))?;
    elaborated_certificates::inherited(
        &facts.types,
        normal,
        all.iter().skip(declared).map(|input| input.value),
    )
    .map_err(Error::unlocated)
}

fn projected_inputs(facts: &Declarations, owner: DefId) -> Result<Vec<Candidate>, &'static str> {
    let types = &facts.types;
    let inputs = declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), owner)?;
    let mut projected = Vec::new();
    for input in inputs.iter() {
        let (predicate, locals) = match input.value {
            Clause::Predicate(predicate) => (predicate, None),
            Clause::ForAll(id) => {
                let quantified = types
                    .quantified(id)
                    .ok_or("missing quantified imported premise")?;
                (quantified.predicate, Some(quantified.bound_regions))
            }
            Clause::UnstableFeature(_) => continue,
        };
        let Some((application, _)) = predicate.bound.trait_parts() else {
            continue;
        };
        if predicate.contains_projections(types)? {
            let arguments = types
                .trait_application(application)
                .ok_or("missing imported premise application")?
                .arguments;
            projected.push(Candidate {
                clause: input.value,
                predicate,
                arguments,
                locals,
            });
        }
    }
    Ok(projected)
}

impl Candidate {
    fn reprove(&self, proof: &mut Proof, facts: &mut Declarations) -> Result<bool, &'static str> {
        if !proof.reprove_normal_term(facts, self.predicate.ty, self.locals)? {
            return Ok(false);
        }
        let count = facts
            .types
            .arguments(self.arguments)
            .ok_or("missing imported premise arguments")?
            .len();
        for index in 0..count {
            let argument = facts
                .types
                .arguments(self.arguments)
                .and_then(|mut arguments| arguments.nth(index))
                .ok_or("missing imported premise argument")?;
            let ty = match argument {
                Argument::Type(ty) => ty,
                Argument::Const(id) => facts
                    .types
                    .const_type(id)
                    .ok_or("missing imported premise const type")?,
                Argument::Lifetime(_) => continue,
            };
            if !proof.reprove_normal_term(facts, ty, self.locals)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
