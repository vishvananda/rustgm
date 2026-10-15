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
    let proof = requirements
        .instantiate(&mut store, use_, 10, &mut scope)?
        .proof;
    assert_eq!(proof.regions, 10..12);
    assert_eq!(proof.equalities.len(), 2);
    assert_eq!(proof.outlives, [(RegionId(11), RegionId(10))]);
    assert_eq!(scope.retained.len(), 513);
    assert!(scope.steps < 5000);
    let repeated = requirements
        .instantiate(&mut store, use_, 12, &mut scope)?
        .proof;
    assert_eq!(repeated.regions, 12..12);
    assert!(repeated.equalities.is_empty());
    let arguments = store.intern_regions(&[RegionId::bound(0, 1)?, RegionId::bound(0, 0)?])?;
    let other = requirements
        .instantiate(&mut store, Use { arguments, ..use_ }, 12, &mut scope)?
        .proof;
    assert_eq!(other.regions, 12..14);
    assert_eq!(other.equalities[0], (RegionId(12), RegionId::bound(0, 1)?));
    let independent = requirements
        .instantiate(&mut store, use_, 21, &mut Scope::default())?
        .proof;
    assert_eq!(independent.regions, 21..23);
    Ok(())
}

#[test]
fn caller_identities_are_not_rebased_as_template_variables() -> TestResult {
    let mut store = Store::default();
    let mut requirements = Requirements::default();
    contract(&mut requirements)?;
    let arguments = store.intern_regions(&[RegionId(1), RegionId(2)])?;
    let proof = requirements
        .instantiate(
            &mut store,
            Use {
                owner: DefId(0),
                arguments,
            },
            101,
            &mut Scope::default(),
        )?
        .proof;
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
    let proof = requirements
        .instantiate(&mut store, use_, 1, &mut cache)?
        .proof;
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
    let repeated = requirements
        .instantiate(&mut store, use_, 5, &mut cache)?
        .proof;
    assert!(repeated.is_empty());
    Ok(())
}

#[test]
fn deferred_type_debts_share_diamonds_without_disappearing_with_empty_lifetime_proofs() -> TestResult
{
    use crate::types::{
        ItemId, Kind, Primitive,
        arguments::Argument,
        parameters::{Parameter, ParameterKind},
    };
    let mut store = Store::default();
    let owner = DefId(0);
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(owner.0),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = store.intern(Kind::Parameter(parameter))?;
    let arguments = store.intern_arguments(&[Argument::Type(ty)])?;
    let mut requirements = Requirements::default();
    requirements.normalization(owner, owner, ty, ScopeId::ROOT)?;
    requirements.sized(owner, owner, ty, ScopeId::ROOT)?;
    requirements.seal(owner)?;
    for id in 1..=128 {
        for _ in 0..2 {
            requirements.dependency(
                DefId(id),
                owner,
                Use {
                    owner: DefId(id - 1),
                    arguments,
                },
                ScopeId::ROOT,
            )?;
        }
        requirements.seal(DefId(id))?;
    }
    let mut scope = Scope::default();
    let arguments = store.intern_arguments(&[Argument::Type(crate::types::TypeId::primitive(
        Primitive::U8,
    ))])?;
    let use_ = Use {
        owner: DefId(128),
        arguments,
    };
    let expansion = requirements.instantiate(&mut store, use_, 1, &mut scope)?;
    assert!(expansion.proof.is_empty());
    assert_eq!(
        expansion.sized,
        [(
            ScopeId::ROOT,
            crate::types::TypeId::primitive(Primitive::U8)
        )]
    );
    assert_eq!(
        expansion.normalizations,
        [(
            ScopeId::ROOT,
            crate::types::TypeId::primitive(Primitive::U8)
        )]
    );
    assert_eq!(scope.retained.len(), 129);
    assert!(scope.steps < 2000);
    assert!(
        requirements
            .instantiate(&mut store, use_, 1, &mut scope)?
            .normalizations
            .is_empty()
    );
    assert_eq!(
        requirements
            .instantiate(&mut store, use_, 1, &mut Scope::default())?
            .normalizations
            .len(),
        1
    );
    let arguments = store.intern_arguments(&[Argument::Type(crate::types::TypeId::primitive(
        Primitive::U16,
    ))])?;
    let different =
        requirements.instantiate(&mut store, Use { arguments, ..use_ }, 1, &mut scope)?;
    assert_eq!(
        different.normalizations,
        [(
            ScopeId::ROOT,
            crate::types::TypeId::primitive(Primitive::U16)
        )]
    );
    assert!(
        requirements
            .normalization(owner, owner, ty, ScopeId::ROOT)
            .is_err()
    );
    assert!(requirements.sized(owner, owner, ty, ScopeId::ROOT).is_err());
    Ok(())
}

#[test]
fn deferred_types_in_sibling_binders_retain_distinct_quantifier_scopes() -> TestResult {
    let mut store = Store::default();
    let owner = DefId(0);
    let mut requirements = Requirements::default();
    for _ in 0..2 {
        let scope = requirements.scope(owner, owner, ScopeId::ROOT)?;
        requirements.normalization(owner, owner, crate::types::TypeId::UNIT, scope)?;
        requirements.complete_scope(owner, scope, 1);
    }
    requirements.seal(owner)?;
    let expansion = requirements.instantiate(
        &mut store,
        Use {
            owner,
            arguments: GenericArgs::EMPTY,
        },
        1,
        &mut Scope::default(),
    )?;
    assert_eq!(expansion.normalizations.len(), 2);
    assert_ne!(expansion.normalizations[0].0, ScopeId::ROOT);
    assert_ne!(expansion.normalizations[0].0, expansion.normalizations[1].0);
    assert!(expansion.proof.has_quantified());
    Ok(())
}

#[test]
fn sized_dependencies_keep_total_depth_across_nested_alias_contracts() -> TestResult {
    use crate::types::{
        ItemId, Kind, TypeId,
        arguments::Argument,
        parameters::{Parameter, ParameterKind},
    };
    let mut store = Store::default();
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(0),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = store.intern(Kind::Parameter(parameter))?;
    let mut requirements = Requirements::default();
    let inner = DefId(0);
    let scope = requirements.scope(inner, inner, ScopeId::ROOT)?;
    requirements.sized(inner, inner, ty, scope)?;
    requirements.complete_scope(inner, scope, 1);
    requirements.seal(inner)?;
    let outer = DefId(1);
    let first = requirements.scope(outer, inner, ScopeId::ROOT)?;
    let second = requirements.scope(outer, inner, first)?;
    let arguments = store.intern_arguments(&[Argument::Type(ty)])?;
    requirements.dependency(
        outer,
        inner,
        Use {
            owner: inner,
            arguments,
        },
        second,
    )?;
    requirements.complete_scope(outer, first, 1);
    requirements.complete_scope(outer, second, 1);
    requirements.seal(outer)?;
    let arguments = store.intern_arguments(&[Argument::Type(TypeId::UNIT)])?;
    let use_ = Use {
        owner: outer,
        arguments,
    };
    let mut cache = Scope::default();
    let expansion = requirements.instantiate(&mut store, use_, 1, &mut cache)?;
    assert!(expansion.proof.is_empty());
    assert_eq!(expansion.sized.len(), 1);
    assert_eq!(expansion.proof.scope_depth(expansion.sized[0].0)?, 3);
    assert_eq!(expansion.sized[0].1, TypeId::UNIT);
    assert!(
        requirements
            .instantiate(&mut store, use_, 1, &mut cache)?
            .sized
            .is_empty()
    );
    assert_eq!(
        requirements
            .instantiate(&mut store, use_, 1, &mut Scope::default())?
            .sized,
        expansion.sized
    );
    Ok(())
}
