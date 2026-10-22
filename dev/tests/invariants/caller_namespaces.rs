//! Scope ownership survives alpha transport before colliding actual arguments.
use super::*;
use crate::types::{Kind, TypeId, lifetime_proof::ScopeId};

fn reference(store: &mut Store, region: RegionId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        referent: TypeId::UNIT,
        mutable: false,
        region,
    })
}

#[test]
fn private_roots_rebase_before_colliding_actuals_and_preserve_nested_binders()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let mut proof = Obligations::new(1);
    proof.regions.end = 2;
    proof.equalities.push((RegionId(1), RegionId::bound(0, 0)?));
    let empty = proof.scope(ScopeId::ROOT, 0)?;
    let nested = proof.scope(empty, 1)?;
    let mut child = Obligations::new(2);
    child.regions.end = 3;
    child.equalities.push((RegionId(2), RegionId::bound(2, 0)?));
    child
        .types
        .push((reference(&mut store, RegionId(2))?, RegionId::bound(0, 0)?));
    proof.append_at(child, nested)?;
    let mut transport = OwnedTransport::new(std::slice::from_ref(&(1..3)), &[], 1, 30)?;
    transport.check_actuals(&[RegionId(1)])?;
    assert!(transport.check_actuals(&[RegionId(30)]).is_err());
    transport.proof(&mut store, &mut proof)?;
    proof.substitute(&mut store, &mut Bound::new(vec![RegionId(1)]))?;
    assert_eq!(proof.regions, 30..32);
    assert_eq!(proof.equalities, [(RegionId(30), RegionId(1))]);
    let plan = proof.scope_plan();
    assert_eq!(plan.binder(empty)?, (ScopeId::ROOT, 0));
    assert_eq!(plan.binder(nested)?, (empty, 1));
    let child = &proof
        .quantified
        .as_ref()
        .ok_or("lost nested scopes")?
        .blocks[0]
        .proof;
    assert_eq!(child.regions, 31..32);
    assert_eq!(child.equalities, [(RegionId(31), RegionId(1))]);
    assert_eq!(
        child.types,
        [(reference(&mut store, RegionId(31))?, RegionId::bound(0, 0)?)]
    );
    Ok(())
}

#[test]
fn independent_proof_segments_keep_foreign_gaps_and_empty_scope_identity()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let mut proof = Obligations::new(10);
    proof.regions.end = 12;
    let empty = proof.scope(ScopeId::ROOT, 0)?;
    proof.resume_at(20)?;
    let mut child = Obligations::new(20);
    child.regions.end = 22;
    child
        .equalities
        .push((RegionId(20), RegionId::bound(1, 0)?));
    proof.append_streamed_at(child, empty)?;
    proof.equalities.push((RegionId(10), RegionId(21)));
    let mut transport = OwnedTransport::new(&[10..12, 15..16, 20..22], &[RegionId(7)], 1, 30)?;
    assert!(transport.fold.region(RegionId(13), 0).is_err());
    assert_eq!(transport.fold.region(RegionId(7), 0)?, RegionId(7));
    transport.proof(&mut store, &mut proof)?;
    assert_eq!(proof.regions, 30..35);
    assert!(proof.owns_region(RegionId(30)));
    assert!(!proof.owns_region(RegionId(32)));
    assert!(proof.owns_region(RegionId(34)));
    assert!(proof.template_interval().is_err());
    assert_eq!(proof.equalities, [(RegionId(30), RegionId(34))]);
    assert_eq!(proof.scope_plan().binder(empty)?, (ScopeId::ROOT, 0));
    Ok(())
}

#[test]
fn namespace_preflight_rejects_overlap_foreign_roots_and_oversized_batches()
-> Result<(), &'static str> {
    assert!(OwnedTransport::new(&[10..12, 11..13], &[], 0, 30).is_err());
    assert!(OwnedTransport::new(&[20..21, 10..11], &[], 0, 30).is_err());
    assert!(OwnedTransport::new(std::slice::from_ref(&(10..12)), &[RegionId(11)], 0, 30).is_err());
    assert!(OwnedTransport::new(std::slice::from_ref(&(1..1_000_001)), &[], 1, 30).is_err());
    let mut transport =
        OwnedTransport::new(std::slice::from_ref(&(10..12)), &[RegionId(7)], 1, 30)?;
    let mut store = Store::default();
    let mut foreign = Obligations::new(10);
    foreign.regions.end = 13;
    assert!(transport.proof(&mut store, &mut foreign).is_err());
    assert!(transport.fold.region(RegionId(8), 0).is_err());
    assert_eq!(
        transport.fold.region(RegionId::bound(0, 0)?, 0)?,
        RegionId::bound(0, 0)?
    );
    assert!(transport.fold.region(RegionId::bound(0, 1)?, 0).is_err());
    Ok(())
}
