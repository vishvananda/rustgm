//! An override must implement the trait contract without strengthening its caller requirements.
mod mapping;
mod regions;
use crate::{
    resolve::model::DefId,
    sema::{
        model::{Declarations, Function},
        nominals::Proof,
    },
    types::{
        RegionId, TypeId,
        parameters::{
            ParameterId,
            environment::Environment,
            predicates::{Bound, Predicate},
        },
    },
};
use std::sync::Arc;

struct Contract {
    expected: TypeId,
    actual: TypeId,
    environment: Arc<Environment>,
    parameters: Vec<ParameterId>,
    assumptions: Vec<Predicate>,
    obligations: Vec<Predicate>,
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
) -> Result<(), String> {
    let contract = mapping::contract(facts, owner, predicate, expected, actual)?;
    let mut shape = crate::types::infer::Context::default();
    let actual = shape
        .known(&facts.types, contract.actual)
        .map_err(|e| e.to_string())?;
    let expected = shape
        .known(&facts.types, contract.expected)
        .map_err(|e| e.to_string())?;
    shape
        .equate_shape(&facts.types, actual, expected)
        .map_err(|e| e.to_string())?;
    let mut proof = Proof::new(Some(Arc::clone(&contract.environment)));
    for &obligation in &contract.obligations {
        if !matches!(obligation.bound, Bound::Outlives(_)) && !proof.prove(facts, obligation)? {
            return Err("implementation method imposes a stronger generic requirement".into());
        }
    }
    regions::prove(facts, &contract, &proof.take_trait_equalities())
}
