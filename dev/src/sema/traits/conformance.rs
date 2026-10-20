//! An override must implement the trait contract without strengthening its caller requirements.
mod implied;
mod mapping;
mod outputs;
mod projections;
mod regions;
#[cfg(test)]
#[path = "../../../tests/invariants/override_clauses.rs"]
mod tests;
use crate::{
    resolve::model::DefId,
    sema::{
        model::{Declarations, Function},
        nominals::Proof,
    },
    types::{
        RegionId, TypeId,
        parameters::{
            ParameterId, clauses::Clause, environment::Environment, predicates::Predicate,
        },
    },
};
pub(in crate::sema) use projections::Definitions;
use std::sync::Arc;

struct Contract {
    outputs: Vec<(TypeId, Vec<Clause>)>,
    expected: TypeId,
    declared: TypeId,
    output_regions: Vec<RegionId>,
    actual: TypeId,
    environment: Arc<Environment>,
    parameters: Vec<ParameterId>,
    assumptions: Vec<Clause>,
    obligations: Vec<Clause>,
    assumed_regions: Vec<crate::types::outlives::Pair>,
    required_regions: Vec<crate::types::outlives::Pair>,
    universals: u32,
    variables: Vec<RegionId>,
    inputs: Vec<TypeId>,
}

pub(in crate::sema) fn check(
    facts: &mut Declarations,
    owner: DefId,
    predicate: Predicate,
    expected: Function,
    actual: Function,
    definitions: &Definitions,
) -> Result<(), String> {
    let mut contract = mapping::contract(facts, owner, predicate, expected, actual)?;
    let first = contract
        .variables
        .iter()
        .map(|region| region.0)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or("method conformance region identity space exhausted")?;
    let mut evidence = crate::types::lifetime_proof::Obligations::new(first);
    let mut implied = implied::Formation::default();
    contract.expected = definitions.signature(
        facts,
        &contract,
        &mut evidence,
        contract.expected,
        Some(&mut implied),
    )?;
    implied.extend(&mut contract);
    contract.actual =
        definitions.signature(facts, &contract, &mut evidence, contract.actual, None)?;
    contract.declared = contract.expected;
    let outputs = outputs::instantiate(facts, &mut contract, &mut evidence)?;
    let mut shape = crate::types::infer::Context::default();
    let actual = shape
        .known(&facts.types, contract.actual)
        .map_err(|e| e.to_string())?;
    let expected_term = shape
        .known(&facts.types, contract.expected)
        .map_err(|e| e.to_string())?;
    shape
        .equate_shape(&facts.types, actual, expected_term)
        .map_err(|e| e.to_string())?;
    let mut proof = Proof::for_declaration(
        Some(Arc::clone(&contract.environment)),
        contract.universals,
        evidence.regions.end,
    );
    for &obligation in &contract.obligations {
        if !obligation.is_outlives(&facts.types)? && !proof.prove_clause(facts, obligation)? {
            return Err("implementation method imposes a stronger generic requirement".into());
        }
    }
    if let Some(required) = proof.take_trait_proof() {
        evidence.extend(required)?;
    }
    regions::prove(facts, &contract, Some(&evidence))?;
    let regions: std::sync::Arc<[_]> = contract.output_regions.into();
    let parent_regions = facts
        .lifetimes
        .functions
        .get(&expected.definition)
        .map_or(0, |scheme| scheme.captures);
    for (id, ty) in outputs {
        facts.opaques.implementations.insert(
            (owner, id),
            crate::sema::opaque::Implementation {
                ty,
                parent_regions,
                regions: std::sync::Arc::clone(&regions),
            },
        );
    }
    Ok(())
}
