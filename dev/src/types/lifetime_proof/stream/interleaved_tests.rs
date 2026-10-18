//! Resumed evidence owns its original scopes and only its explicitly reserved regions.
use super::Obligations;
use crate::types::{
    RegionId, Store,
    lifetime_proof::ScopeId,
    outlives::{Bounds, Queries},
    relations::{Constraints, Identities},
    substitute::Bound,
    variance::Variances,
};
type ResultTest = Result<(), Box<dyn std::error::Error>>;

fn equal(first: u32, region: RegionId) -> Obligations {
    let mut proof = Obligations::new(first);
    proof.regions.end += 1;
    proof.equalities.push((RegionId(first), region));
    proof
}

fn check(proofs: &[Obligations]) -> Result<Identities, String> {
    let store = Store::default();
    let variances = Variances::default();
    let bounds = Bounds::default();
    let callables = crate::types::callable::Templates::default();
    let mut queries = Queries::new(&store, &bounds, &callables);
    let mut constraints =
        Constraints::new(&store, &variances, 0, 10_000).map_err(|e| e.to_string())?;
    for proof in proofs {
        proof.require_source(&mut constraints, &mut queries, 73)?;
    }
    constraints
        .prove()
        .map(|p| p.identities)
        .map_err(|e| e.to_string())
}

#[test]
fn interleaved_queries_neither_allocate_nor_claim_their_gaps() -> ResultTest {
    let mut parent = equal(1, RegionId::STATIC);
    parent.resume_at(4)?;
    parent.extend(equal(4, RegionId::STATIC))?;
    assert_eq!(parent.regions, 1..5);
    assert_eq!(parent.owned_regions().collect::<Vec<_>>(), vec![1..2, 4..5]);
    let identities = check(&[parent.clone()])?;
    assert_eq!(identities.variables().len(), 2);
    assert!(identities.get(RegionId(2)).is_none());
    assert!(identities.get(RegionId(3)).is_none());

    let mut child = Obligations::new(2);
    let scope = child.scope(ScopeId::ROOT, 1)?;
    child.append_at(equal(2, RegionId::bound(0, 0)?), scope)?;
    child.append_at(equal(3, RegionId::bound(0, 0)?), scope)?;
    assert_eq!(
        check(&[child.clone(), parent.clone()])?.variables().len(),
        4
    );
    assert_eq!(check(&[parent.clone(), child])?.variables().len(), 4);
    parent.equalities.push((RegionId(2), RegionId::STATIC));
    assert!(check(&[parent]).is_err());
    Ok(())
}

#[test]
fn a_resumed_binder_keeps_its_identity_and_cannot_borrow_a_siblings() -> ResultTest {
    let mut parent = Obligations::new(1);
    let empty = parent.scope(ScopeId::ROOT, 0)?;
    let scope = parent.scope(empty, 1)?;
    let sibling = parent.scope(empty, 1)?;
    parent.append_at(equal(1, RegionId::bound(0, 0)?), scope)?;
    parent.resume_at(4)?;
    let mut good = parent.clone();
    good.append_at(equal(4, RegionId::bound(0, 0)?), scope)?;
    good.equalities.push((RegionId(1), RegionId(4)));
    assert_eq!(check(&[good])?.variables().len(), 2);
    parent.append_at(equal(4, RegionId::bound(0, 0)?), sibling)?;
    parent.equalities.push((RegionId(1), RegionId(4)));
    assert!(check(&[parent]).is_err());
    Ok(())
}

#[test]
fn caller_substitution_preserves_disjoint_allocations_and_local_binders() -> ResultTest {
    let mut proof = Obligations::new(1);
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    proof.append_at(equal(1, RegionId::bound(1, 0)?), scope)?;
    proof.resume_at(8)?;
    proof.append_at(equal(8, RegionId::bound(0, 0)?), scope)?;
    proof.substitute(
        &mut Store::default(),
        &mut Bound::new(vec![RegionId::STATIC]),
    )?;
    let ranges = proof.owned_regions().collect::<Vec<_>>();
    assert_eq!(ranges, vec![1..2, 8..9]);
    assert_eq!(check(&[proof.clone()])?.variables().len(), 2);
    proof.equalities.push((RegionId(1), RegionId(8)));
    assert!(check(&[proof]).is_err());
    Ok(())
}

#[test]
fn extension_keeps_holes_and_rebasing_cannot_turn_them_into_private_variables() -> ResultTest {
    let mut first = equal(1, RegionId::STATIC);
    first.resume_at(4)?;
    let mut second = Obligations::new(4);
    second.resume_at(8)?;
    second.extend(equal(8, RegionId::STATIC))?;
    first.extend(second)?;
    assert_eq!(first.owned_regions().collect::<Vec<_>>(), vec![1..2, 8..9]);
    assert_eq!(first.quantified.as_ref().map(|q| q.gaps.len()), Some(1));
    let mut store = Store::default();
    assert!(first.rebase(&mut store, 20).is_err());
    assert!(
        first
            .instantiate_at(&mut store, &mut Bound::new(Vec::new()), 20, 0)
            .is_err()
    );
    let mut restore = first.restoration()?;
    assert_eq!(restore.region(RegionId(1), 0)?, RegionId::STATIC);
    assert_eq!(restore.region(RegionId(8), 0)?, RegionId::STATIC);
    assert!(restore.region(RegionId(4), 0).is_err());
    let mut destination = Obligations::new(1);
    assert!(destination.graft(first.clone(), ScopeId::ROOT).is_err());
    assert!(first.resume_at(2).is_err());
    assert_eq!(check(&[first])?.variables().len(), 2);
    Ok(())
}

#[test]
fn empty_resumptions_do_not_allocate_regions_or_grow_the_gap_list() -> ResultTest {
    let mut proof = Obligations::new(1);
    for first in 2..8192 {
        proof.resume_at(first)?;
        assert!(proof.is_empty());
    }
    assert_eq!(proof.quantified.as_ref().map(|q| q.gaps.len()), Some(1));
    assert_eq!(check(&[proof])?.variables().len(), 0);
    Ok(())
}

#[test]
fn a_large_foreign_interval_costs_no_private_variables() -> ResultTest {
    let mut proof = Obligations::new(1);
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    proof.append_at(equal(1, RegionId::bound(0, 0)?), scope)?;
    let next = 1 << 29;
    proof.resume_at(next)?;
    proof.append_at(equal(next, RegionId::bound(0, 0)?), scope)?;
    assert_eq!(
        proof.owned_regions().collect::<Vec<_>>(),
        vec![1..2, next..next + 1]
    );
    assert_eq!(check(&[proof])?.variables().len(), 2);
    Ok(())
}

#[test]
fn repeated_resumptions_share_one_binder_tree() -> ResultTest {
    for count in [8, 64, 512] {
        let mut proof = Obligations::new(1);
        let empty = proof.scope(ScopeId::ROOT, 0)?;
        let scope = proof.scope(empty, 1)?;
        for index in 0..count {
            let first = 1 + 3 * index;
            proof.resume_at(first)?;
            proof.append_at(equal(first, RegionId::bound(0, 0)?), scope)?;
        }
        assert_eq!(proof.scope_plan().len(), 2);
        assert_eq!(proof.owned_regions().count(), count as usize);
        assert_eq!(check(&[proof])?.variables().len(), count as usize);
    }
    Ok(())
}

#[test]
fn streamed_projection_children_attach_once_without_claiming_foreign_intervals() -> ResultTest {
    let next = 1 << 29;
    let local = RegionId::bound(0, 0)?;
    let mut child = equal(1, local);
    child.resume_at(next)?;
    child.extend(equal(next, local))?;
    let mut parent = Obligations::new(1);
    let scope = parent.scope(ScopeId::ROOT, 1)?;
    parent.append_streamed_at(child.clone(), scope)?;
    assert_eq!(parent.scope_count(), 1);
    assert_eq!(
        parent.owned_regions().collect::<Vec<_>>(),
        vec![1..2, next..next + 1]
    );
    assert_eq!(check(&[parent])?.variables().len(), 2);

    child.equalities.push((RegionId(2), RegionId::STATIC));
    let mut invalid = Obligations::new(1);
    let scope = invalid.scope(ScopeId::ROOT, 1)?;
    invalid.append_streamed_at(child, scope)?;
    assert!(check(&[invalid]).is_err());
    Ok(())
}
