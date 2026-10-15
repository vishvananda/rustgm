//! Contracts preserve private proof variables and share only identical instantiated dependencies.
use super::{DefId, Requirements, Scope, Use};
use crate::{
    sema::traits::obligations::Obligations,
    types::{GenericArgs, RegionId, Store, lifetime_proof::ScopeId},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn contract(requirements: &mut Requirements) -> Result<(), &'static str> {
    let mut proof = Obligations::new(1);
    proof.regions.end = 3;
    proof.equalities = vec![
        (RegionId(1), RegionId::bound(0, 0)?),
        (RegionId(2), RegionId::bound(0, 1)?),
    ];
    proof.outlives.push((RegionId(2), RegionId(1)));
    requirements.direct(DefId(0), DefId(0), proof, ScopeId::ROOT)?;
    requirements.seal(DefId(0))?;
    Ok(())
}

#[test]
fn diamond_dependencies_are_linear_and_argument_sensitive() -> TestResult {
    let mut store = Store::default();
    let mut requirements = Requirements::default();
    contract(&mut requirements)?;
    let arguments = store.intern_regions(&[RegionId::bound(0, 0)?, RegionId::bound(0, 1)?])?;
    for id in 1..=512 {
        for _ in 0..2 {
            requirements.dependency(
                DefId(id),
                DefId(id),
                Use {
                    owner: DefId(id - 1),
                    arguments,
                },
                ScopeId::ROOT,
            )?;
        }
        requirements.seal(DefId(id))?;
    }
    let use_ = Use {
        owner: DefId(512),
        arguments,
    };
    let mut scope = Scope::default();
    let proof = requirements.instantiate(&mut store, use_, 10, &mut scope)?;
    assert_eq!(proof.regions, 10..12);
    assert_eq!(proof.equalities.len(), 2);
    assert_eq!(proof.outlives, [(RegionId(11), RegionId(10))]);
    assert_eq!(scope.retained.len(), 513);
    assert!(scope.steps < 5000);
    let repeated = requirements.instantiate(&mut store, use_, 12, &mut scope)?;
    assert_eq!(repeated.regions, 12..12);
    assert!(repeated.equalities.is_empty());
    let arguments = store.intern_regions(&[RegionId::bound(0, 1)?, RegionId::bound(0, 0)?])?;
    let other = requirements.instantiate(&mut store, Use { arguments, ..use_ }, 12, &mut scope)?;
    assert_eq!(other.regions, 12..14);
    assert_eq!(other.equalities[0], (RegionId(12), RegionId::bound(0, 1)?));
    let independent = requirements.instantiate(&mut store, use_, 21, &mut Scope::default())?;
    assert_eq!(independent.regions, 21..23);
    Ok(())
}

#[test]
fn caller_identities_are_not_rebased_as_template_variables() -> TestResult {
    let mut store = Store::default();
    let mut requirements = Requirements::default();
    contract(&mut requirements)?;
    let arguments = store.intern_regions(&[RegionId(1), RegionId(2)])?;
    let proof = requirements.instantiate(
        &mut store,
        Use {
            owner: DefId(0),
            arguments,
        },
        101,
        &mut Scope::default(),
    )?;
    assert_eq!(
        proof.equalities,
        [(RegionId(101), RegionId(1)), (RegionId(102), RegionId(2))]
    );
    assert_eq!(proof.outlives, [(RegionId(102), RegionId(101))]);
    Ok(())
}

#[test]
fn invalid_graphs_and_failed_scopes_cannot_publish_partial_evidence() -> TestResult {
    let mut store = Store::default();
    let mut requirements = Requirements::default();
    let use_ = Use {
        owner: DefId(0),
        arguments: GenericArgs::EMPTY,
    };
    assert!(
        requirements
            .dependency(DefId(0), DefId(0), use_, ScopeId::ROOT)
            .is_err()
    );
    let mut proof = Obligations::new(1);
    proof.equalities.push((RegionId(7), RegionId::STATIC));
    requirements.direct(DefId(0), DefId(0), proof, ScopeId::ROOT)?;
    assert!(
        requirements
            .dependency(DefId(1), DefId(1), use_, ScopeId::ROOT)
            .is_err()
    );
    requirements.seal(DefId(0))?;
    assert!(
        requirements
            .direct(DefId(0), DefId(0), Obligations::new(1), ScopeId::ROOT)
            .is_err()
    );
    let mut scope = Scope::default();
    assert!(
        requirements
            .instantiate(&mut store, use_, 10, &mut scope)
            .is_err()
    );
    assert!(scope.failed);
    assert!(
        requirements
            .instantiate(&mut store, use_, 10, &mut scope)
            .is_err()
    );
    Ok(())
}

#[test]
fn quantifiers_must_finish_before_their_contract_is_sealed() -> TestResult {
    let mut requirements = Requirements::default();
    let scope = requirements.scope(DefId(3), DefId(3), ScopeId::ROOT)?;
    assert!(requirements.seal(DefId(3)).is_err());
    requirements.complete_scope(DefId(3), scope, 2);
    requirements.seal(DefId(3))?;
    assert!(requirements.scope(DefId(3), DefId(3), scope).is_err());
    Ok(())
}

#[test]
fn identical_dependencies_in_sibling_quantifiers_keep_independent_witnesses() -> TestResult {
    let mut store = Store::default();
    let mut requirements = Requirements::default();
    contract(&mut requirements)?;
    let owner = DefId(1);
    let arguments = store.intern_regions(&[RegionId::bound(0, 0)?, RegionId::STATIC])?;
    for _ in 0..2 {
        let scope = requirements.scope(owner, owner, ScopeId::ROOT)?;
        requirements.dependency(
            owner,
            owner,
            Use {
                owner: DefId(0),
                arguments,
            },
            scope,
        )?;
        requirements.complete_scope(owner, scope, 1);
    }
    requirements.seal(owner)?;
    let use_ = Use {
        owner,
        arguments: GenericArgs::EMPTY,
    };
    let mut cache = Scope::default();
    let proof = requirements.instantiate(&mut store, use_, 1, &mut cache)?;
    assert_eq!(proof.regions, 1..5);
    let variances = crate::types::variance::Variances::default();
    let bounds = crate::types::outlives::Bounds::default();
    let callables = crate::types::callable::Templates::default();
    let mut queries = crate::types::outlives::Queries::new(&store, &bounds, &callables);
    let mut constraints = crate::types::relations::Constraints::new(&store, &variances, 0, 10_000)?;
    proof.require(&mut constraints, &mut queries)?;
    let proof = constraints.prove()?;
    assert_eq!(
        proof
            .solution
            .nodes()
            .filter(|(_, kind)| matches!(kind, crate::types::region_graph::Kind::Universal { .. }))
            .count(),
        2
    );
    let repeated = requirements.instantiate(&mut store, use_, 5, &mut cache)?;
    assert!(repeated.is_empty());
    Ok(())
}
