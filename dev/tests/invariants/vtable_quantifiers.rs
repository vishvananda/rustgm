//! A converted callable's public placeholders cannot become caller permissions.
use super::*;
use crate::{
    body::regions::Obligation,
    types::{
        outlives::{Bounds, Queries},
        region_graph::Error,
        relations::Constraints,
        variance::Variances,
    },
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn fixture() -> TestResult<(Declarations, Contract, ScopeId)> {
    let report =
        crate::test_source::resolve("fn hold<'a>(x: &'a u8) -> &'a u8 { x } fn main(){}", true)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let caller = crate::body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|contract| contract.universals == 1)
        .ok_or("missing test contract")?;
    let mut opening = Obligations::new(100);
    let scope = opening.scope(ScopeId::ROOT, 1)?;
    opening.append_universals(100..101, scope)?;
    Ok((facts, selection_contract(&caller, opening), scope))
}

fn accepts(facts: &Declarations, proof: &Obligations, paid: bool) -> TestResult<bool> {
    let variances = Variances::default();
    let bounds = Bounds::default();
    let callables = crate::types::callable::Templates::default();
    let mut queries = Queries::new(&facts.types, &bounds, &callables);
    let mut constraints = Constraints::new(&facts.types, &variances, 1, 100_000)?;
    if paid {
        constraints.assume(RegionId::bound(0, 0)?, RegionId::STATIC)?;
    }
    proof.require(&mut constraints, &mut queries)?;
    match constraints.solve() {
        Ok(_) => Ok(true),
        Err(Error::Unsatisfied { .. } | Error::Verification { .. }) => Ok(false),
        Err(error) => Err(error.into()),
    }
}

#[test]
fn call_quantifiers_keep_caller_debts_and_reject_local_static_leaks() -> TestResult {
    for public in [false, true] {
        let (mut facts, mut selected, scope) = fixture()?;
        selected.obligations.push(Obligation {
            source: selected.source.into(),
            kind: Kind::Outlives((
                if public {
                    RegionId(100)
                } else {
                    RegionId::bound(0, 0)?
                },
                RegionId::STATIC,
            )),
        });
        let proof = close(&mut facts, selected, scope)?;
        assert_eq!(proof.regions, 100..101);
        assert_eq!(proof.scope_count(), 1);
        assert!(!accepts(&facts, &proof, false)?);
        assert_eq!(accepts(&facts, &proof, true)?, !public);
    }
    Ok(())
}

#[test]
fn grafted_normalization_keeps_private_witnesses_and_nested_universals() -> TestResult {
    for leak in [false, true] {
        let (mut facts, mut selected, scope) = fixture()?;
        let mut proof = Obligations::new(101);
        proof.regions.end = 102;
        proof
            .equalities
            .push((RegionId(101), RegionId::bound(0, 0)?));
        let child = proof.scope(ScopeId::ROOT, 1)?;
        proof.append_universals(102..103, child)?;
        let mut debts = Obligations::new(103);
        debts
            .equalities
            .push((RegionId(102), RegionId::bound(0, 0)?));
        if leak {
            debts.outlives.push((RegionId(102), RegionId::bound(1, 0)?));
        }
        proof.append_at(debts, child)?;
        selected.proofs.push(Proof::new(selected.source, proof));
        let proof = close(&mut facts, selected, scope)?;
        assert_eq!(proof.regions, 100..103);
        assert_eq!(proof.scope_count(), 2);
        assert_eq!(proof.owned_regions().flatten().count(), 3);
        assert_eq!(accepts(&facts, &proof, false)?, !leak);
    }
    Ok(())
}
