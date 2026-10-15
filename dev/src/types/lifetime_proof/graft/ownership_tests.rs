//! Private intervals and lexical binders remain distinct across delayed proof publication.
use super::{Obligations, ScopeId};
use crate::types::{Kind, RegionId, Store, TypeId, substitute::Bound};

fn reference(store: &mut Store, region: RegionId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        referent: TypeId::UNIT,
        mutable: false,
        region,
    })
}

#[test]
fn graft_preserves_root_gaps_empty_binders_and_sibling_identity() -> Result<(), &'static str> {
    let mut source = Obligations::new(10);
    let empty = source.scope(ScopeId::ROOT, 0)?;
    let first = source.scope(empty, 1)?;
    let sibling = source.scope(empty, 1)?;
    source.regions.end = 11;
    let mut child = Obligations::new(11);
    child.regions.end = 12;
    child
        .equalities
        .push((RegionId(11), RegionId::bound(0, 0)?));
    source.append_at(child, first)?;
    let mut child = Obligations::new(12);
    child.regions.end = 13;
    child
        .equalities
        .push((RegionId(12), RegionId::bound(0, 0)?));
    source.append_at(child, sibling)?;
    source.regions.end = 14;
    source.equalities.push((RegionId(10), RegionId(13)));
    let mut destination = Obligations::new(10);
    let ambient = destination.scope(ScopeId::ROOT, 2)?;
    let mapping = destination.graft(source, ambient)?;
    assert_eq!(destination.regions, 10..14);
    assert_eq!(mapping[0], ambient);
    let plan = destination.scope_plan();
    assert_eq!(plan.binder(mapping[empty.0 as usize])?, (ambient, 0));
    assert_eq!(
        plan.binder(mapping[first.0 as usize])?,
        (mapping[empty.0 as usize], 1)
    );
    assert_eq!(plan.depth(mapping[sibling.0 as usize])?, 3);
    assert_ne!(mapping[first.0 as usize], mapping[sibling.0 as usize]);
    let q = destination
        .quantified
        .as_ref()
        .ok_or("missing grafted proof")?;
    let intervals = q
        .blocks
        .iter()
        .map(|b| (b.scope, b.proof.regions.clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        intervals,
        vec![
            (ambient, 10..11),
            (mapping[first.0 as usize], 11..12),
            (mapping[sibling.0 as usize], 12..13),
            (ambient, 13..14)
        ]
    );
    assert_eq!(
        q.blocks[3].proof.equalities,
        vec![(RegionId(10), RegionId(13))]
    );
    Ok(())
}

#[test]
fn rebase_precedes_actual_arguments_even_when_their_ids_collide() -> Result<(), &'static str> {
    let mut store = Store::default();
    let mut proof = Obligations::new(1);
    proof.regions.end = 2;
    proof.equalities.push((RegionId(1), RegionId::bound(0, 0)?));
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    let mut child = Obligations::new(2);
    child.regions.end = 3;
    child.equalities.push((RegionId(2), RegionId::bound(1, 0)?));
    child
        .types
        .push((reference(&mut store, RegionId(2))?, RegionId::bound(0, 0)?));
    proof.append_at(child, scope)?;
    proof.rebase(&mut store, 30)?;
    proof.substitute(&mut store, &mut Bound::new(vec![RegionId(1)]))?;
    assert_eq!(proof.regions, 30..32);
    assert_eq!(proof.equalities, vec![(RegionId(30), RegionId(1))]);
    let child = &proof
        .quantified
        .as_ref()
        .ok_or("missing rebased child")?
        .blocks[0]
        .proof;
    assert_eq!(child.regions, 31..32);
    assert_eq!(child.equalities, vec![(RegionId(31), RegionId(1))]);
    assert_eq!(
        child.types,
        vec![(reference(&mut store, RegionId(31))?, RegionId::bound(0, 0)?)]
    );
    Ok(())
}

#[test]
fn ambient_substitution_preserves_locals_and_visits_quantified_type_roots()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let mut proof = Obligations::new(1);
    let ambient = reference(&mut store, RegionId::bound(0, 0)?)?;
    let caller = reference(&mut store, RegionId::bound(1, 0)?)?;
    proof.types.push((ambient, RegionId::STATIC));
    proof.types.push((caller, RegionId::STATIC));
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    let mut child = Obligations::new(1);
    let local = reference(&mut store, RegionId::bound(0, 0)?)?;
    let captured = reference(&mut store, RegionId::bound(2, 0)?)?;
    child
        .types
        .extend([(local, RegionId::STATIC), (captured, RegionId::STATIC)]);
    proof.append_at(child, scope)?;
    proof.substitute_inside(&mut store, &mut Bound::new(vec![RegionId(9)]), 1)?;
    let actual = reference(&mut store, RegionId(9))?;
    assert_eq!(
        proof.type_roots().collect::<Vec<_>>(),
        vec![
            (ScopeId::ROOT, ambient),
            (ScopeId::ROOT, actual),
            (scope, local),
            (scope, actual)
        ]
    );
    proof.map_type_roots(|_, ty| if ty == actual { TypeId::UNIT } else { ty });
    assert_eq!(
        proof
            .type_roots()
            .filter(|&(_, ty)| ty == TypeId::UNIT)
            .count(),
        2
    );
    Ok(())
}
