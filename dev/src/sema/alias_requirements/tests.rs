//! Contracts preserve private proof variables and share only identical instantiated dependencies.
use super::{DefId, Requirements, Scope, Use};
use crate::{
    sema::traits::obligations::Obligations,
    types::{GenericArgs, RegionId, Store},
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
    requirements.direct(DefId(0), DefId(0), proof)?;
    requirements.seal(DefId(0));
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
            )?;
        }
        requirements.seal(DefId(id));
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
    assert!(requirements.dependency(DefId(0), DefId(0), use_).is_err());
    let mut proof = Obligations::new(1);
    proof.equalities.push((RegionId(7), RegionId::STATIC));
    requirements.direct(DefId(0), DefId(0), proof)?;
    assert!(requirements.dependency(DefId(1), DefId(1), use_).is_err());
    requirements.seal(DefId(0));
    assert!(
        requirements
            .direct(DefId(0), DefId(0), Obligations::new(1))
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
