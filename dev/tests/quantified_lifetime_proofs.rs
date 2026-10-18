//! Existential witnesses beneath a forall must not become caller inference identities.
use rustgm::types::{
    RegionId, Store,
    lifetime_proof::{Obligations, ScopeId},
    outlives::{Bounds, Queries},
    relations::Constraints,
    variance::Variances,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn check(
    store: &Store,
    proof: &Obligations,
    count: u32,
    assumed: &[(RegionId, RegionId)],
) -> Result<bool, Box<dyn std::error::Error>> {
    let variances = Variances::default();
    let bounds = Bounds::default();
    let callables = rustgm::types::callable::Templates::default();
    let mut queries = Queries::new(store, &bounds, &callables);
    let mut constraints = Constraints::new(store, &variances, count, 10000)?;
    for &(a, b) in assumed {
        constraints.assume(a, b)?;
    }
    proof.require_source(&mut constraints, &mut queries, 73)?;
    Ok(constraints.solve().is_ok())
}
fn equal(first: u32, region: RegionId) -> Obligations {
    let mut proof = Obligations::new(first);
    proof.regions.end = first + 1;
    proof.equalities.push((RegionId(first), region));
    proof
}

#[test]
fn each_local_placeholder_can_have_its_own_private_witness() -> TestResult {
    let store = Store::default();
    let mut proof = Obligations::new(1);
    let a = proof.scope(ScopeId::ROOT, 1)?;
    let b = proof.scope(ScopeId::ROOT, 1)?;
    proof.append_at(equal(1, RegionId::bound(0, 0)?), a)?;
    proof.append_at(equal(2, RegionId::bound(0, 0)?), b)?;
    assert!(check(&store, &proof, 0, &[])?);
    let mut crossed = Obligations::new(1);
    let a = crossed.scope(ScopeId::ROOT, 1)?;
    let b = crossed.scope(ScopeId::ROOT, 1)?;
    crossed.append_at(equal(1, RegionId::bound(0, 0)?), a)?;
    let mut second = Obligations::new(2);
    second
        .equalities
        .push((RegionId(1), RegionId::bound(0, 0)?));
    crossed.append_at(second, b)?;
    assert!(!check(&store, &crossed, 0, &[])?);
    Ok(())
}

#[test]
fn an_outer_variable_cannot_equal_every_younger_placeholder() -> TestResult {
    let store = Store::default();
    let mut proof = Obligations::new(1);
    proof.regions.end = 2;
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    let mut inner = Obligations::new(2);
    inner.equalities.push((RegionId(1), RegionId::bound(0, 0)?));
    proof.append_at(inner, scope)?;
    assert!(!check(&store, &proof, 0, &[])?);
    Ok(())
}

#[test]
fn nested_captures_need_the_outer_assumption_and_not_an_identically_numbered_local() -> TestResult {
    let store = Store::default();
    let mut proof = Obligations::new(1);
    let outer = proof.scope(ScopeId::ROOT, 1)?;
    let inner = proof.scope(outer, 1)?;
    let mut evidence = equal(1, RegionId::bound(2, 0)?);
    evidence
        .outlives
        .push((RegionId(1), RegionId::bound(0, 0)?));
    proof.append_at(evidence, inner)?;
    assert!(!check(&store, &proof, 1, &[])?);
    assert!(check(
        &store,
        &proof,
        1,
        &[(RegionId::bound(0, 0)?, RegionId::STATIC)]
    )?);
    let mut folded = proof.clone();
    folded.substitute(
        &mut Store::default(),
        &mut rustgm::types::substitute::Bound::new(vec![RegionId::STATIC]),
    )?;
    assert!(check(&store, &folded, 0, &[])?);
    Ok(())
}

#[test]
fn extending_independent_proofs_remaps_scopes_and_retains_caller_substitution() -> TestResult {
    let mut store = Store::default();
    let mut proof = Obligations::new(1);
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    proof.append_at(equal(1, RegionId::bound(0, 0)?), scope)?;
    let mut second = Obligations::new(2);
    let scope = second.scope(ScopeId::ROOT, 1)?;
    let mut evidence = equal(2, RegionId::bound(1, 0)?);
    evidence
        .outlives
        .push((RegionId(2), RegionId::bound(0, 0)?));
    second.append_at(evidence, scope)?;
    proof.extend(second)?;
    assert!(!check(&store, &proof, 1, &[])?);
    proof.substitute(
        &mut store,
        &mut rustgm::types::substitute::Bound::new(vec![RegionId::STATIC]),
    )?;
    assert!(check(&store, &proof, 0, &[])?);
    Ok(())
}

#[test]
fn nested_evidence_and_noncontiguous_intervals_are_rejected() -> TestResult {
    let mut proof = Obligations::new(1);
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    assert!(proof.append_at(Obligations::new(2), scope).is_err());
    let mut other = Obligations::new(1);
    other.scope(ScopeId::ROOT, 1)?;
    assert!(proof.append_at(other, scope).is_err());
    Ok(())
}

#[test]
fn empty_binders_still_charge_the_scope_budget() -> TestResult {
    use rustgm::types::relations::ProofScope;
    let store = Store::default();
    let variances = Variances::default();
    let mut constraints = Constraints::new(&store, &variances, 0, 2)?;
    let scope = constraints.universal_scope(ProofScope::ROOT, 0)?;
    assert!(constraints.universal_scope(scope, 0).is_err());
    Ok(())
}

#[test]
fn deep_captures_resolve_the_exact_ancestor_without_flattening_scopes() -> TestResult {
    use rustgm::types::relations::ProofScope;
    let store = Store::default();
    let variances = Variances::default();
    let mut constraints = Constraints::new(&store, &variances, 0, 100_000)?;
    let mut scope = ProofScope::ROOT;
    let mut captured = Vec::new();
    for depth in 1_u16..=4096 {
        scope = constraints.universal_scope(scope, 1)?;
        if depth.is_power_of_two() || depth % 257 == 0 {
            let id = RegionId(u32::from(depth));
            constraints.scoped_variable(id, scope)?;
            constraints.scoped_outlives(id, RegionId::bound(0, 0)?, scope, 0)?;
            constraints.scoped_outlives(RegionId::bound(0, 0)?, id, scope, 0)?;
            captured.push((depth, id));
        }
    }
    for (depth, id) in captured {
        let region = RegionId::bound(4096 - depth, 0)?;
        constraints.scoped_outlives(id, region, scope, 0)?;
        constraints.scoped_outlives(region, id, scope, 0)?;
    }
    assert!(
        constraints
            .scoped_outlives(RegionId::bound(4097, 0)?, RegionId::STATIC, scope, 0)
            .is_err()
    );
    constraints.solve()?;
    Ok(())
}

#[test]
fn body_relations_can_use_retained_witnesses_without_granting_their_debts() -> TestResult {
    use rustgm::{
        body::regions::{Contract, Contracts, Kind, Obligation},
        resolve::unit::{FileId, NodeKey},
        syntax::ast::NodeId,
    };
    let store = Store::default();
    let variances = Variances::default();
    let bounds = Bounds::default();
    let callables = rustgm::types::callable::Templates::default();
    let source = NodeKey {
        file: FileId(0),
        node: NodeId(1),
    };
    let debt = NodeKey {
        node: NodeId(2),
        ..source
    };
    let universal = RegionId::bound(0, 0)?;
    let mut contracts = Contracts {
        callables: &callables,
        nominal_requirements: &bounds,
        bodies: vec![Contract {
            source,
            signature: None,
            universals: 1,
            assumptions: Vec::new(),
            parameters: Vec::new(),
            type_assumptions: Vec::new(),
            proofs: vec![rustgm::body::regions::Proof::new(
                source,
                equal(10, RegionId::STATIC),
            )],
            obligations: vec![
                Obligation {
                    source,
                    kind: Kind::Variable {
                        id: RegionId(1),
                        needs_provenance: false,
                    },
                },
                Obligation {
                    source,
                    kind: Kind::Equality((RegionId(1), RegionId(10))),
                },
                Obligation {
                    source: debt,
                    kind: Kind::Outlives((universal, RegionId(1))),
                },
            ],
        }],
    };
    let error = rustgm::borrow::regions::check(&store, &variances, &contracts)
        .err()
        .ok_or("unpaid retained witness was accepted")?;
    assert_eq!(error.source, Some(debt));
    assert_eq!(
        error.message,
        "required lifetime does not outlive its target"
    );
    contracts.bodies[0]
        .assumptions
        .push((universal, RegionId::STATIC));
    rustgm::borrow::regions::check(&store, &variances, &contracts)?;
    contracts.bodies[0].obligations[1].kind = Kind::Equality((RegionId(1), RegionId(11)));
    let error = rustgm::borrow::regions::check(&store, &variances, &contracts)
        .err()
        .ok_or("foreign retained witness was accepted")?;
    assert_eq!(error.message, "invalid lifetime constraint identity");
    Ok(())
}
