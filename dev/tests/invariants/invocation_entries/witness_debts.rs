//! Normalized execution heads keep witness and nested-binder debts at the caller.
use crate::{
    body::regions::{Contract, Contracts, Kind, Obligation},
    sema::model::Declarations,
    types::{
        RegionId,
        lifetime_proof::{Obligations, ScopeId},
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture() -> Result<(Declarations, Contract)> {
    let report = crate::test_source::resolve("fn f<'a>(x:&'a u8){}fn main(){}", true)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let contract = crate::body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|c| c.universals == 1)
        .ok_or("missing universal contract")?;
    Ok((facts, contract))
}
fn accepted(facts: &Declarations, contract: &Contract) -> bool {
    crate::borrow::regions::check(
        &facts.types,
        &facts.lifetimes.variances,
        &Contracts {
            callables: &facts.lifetimes.callables,
            nominal_requirements: &facts.lifetimes.nominal_requirements,
            bodies: vec![contract.clone()],
        },
    )
    .is_ok()
}
#[test]
fn closure_execution_normalization_retains_static_debt() -> Result {
    let (mut facts, mut original) = fixture()?;
    original.obligations.push(Obligation {
        source: original.source.into(),
        kind: Kind::Variable {
            id: RegionId(7),
            needs_provenance: false,
        },
    });
    assert!(accepted(&facts, &original));
    for (actual, valid) in [(RegionId::bound(0, 0)?, false), (RegionId::STATIC, true)] {
        let mut contract = original.clone();
        let mut proof = Obligations::new(1);
        proof.regions.end = 2;
        proof.equalities.push((RegionId(1), RegionId::bound(0, 0)?));
        proof.outlives.push((RegionId(1), RegionId::STATIC));
        crate::body::instantiate::retain_closure_proof(
            &mut facts,
            &mut contract,
            proof,
            [(0, actual)].into_iter(),
        )?;
        let retained = &contract
            .proofs
            .last()
            .ok_or("missing normalization proof")?
            .proof;
        assert_eq!(retained.equalities.len(), 1);
        assert_eq!(retained.outlives.len(), 1);
        assert_eq!(retained.equalities[0].1, actual);
        assert!(retained.regions.start > 7);
        assert_eq!(accepted(&facts, &contract), valid);
    }
    Ok(())
}
#[test]
fn closure_normalization_cannot_capture_a_nested_local() -> Result {
    let (mut facts, mut contract) = fixture()?;
    let mut proof = Obligations::new(1);
    let nested = proof.scope(ScopeId::ROOT, 1)?;
    let mut block = Obligations::new(1);
    block
        .equalities
        .push((RegionId::bound(0, 0)?, RegionId::bound(1, 0)?));
    proof.append_at(block, nested)?;
    crate::body::instantiate::retain_closure_proof(
        &mut facts,
        &mut contract,
        proof,
        [(0, RegionId::STATIC)].into_iter(),
    )?;
    assert_eq!(
        contract
            .proofs
            .last()
            .ok_or("missing proof")?
            .proof
            .scope_count(),
        1
    );
    assert!(!accepted(&facts, &contract));
    Ok(())
}
