//! Reserved blocks cannot claim another query's intervals or silently change their scope.
use super::{Obligations, ScopeId};
use crate::types::{
    RegionId, Store,
    outlives::{Bounds, Queries},
    relations::Constraints,
    variance::Variances,
};

fn block(start: u32, end: u32) -> Obligations {
    let mut proof = Obligations::new(start);
    proof.regions.end = end;
    proof
}

#[test]
fn reserved_placement_rejects_overlap_foreign_gaps_and_missing_scopes() -> Result<(), &'static str>
{
    let mut proof = block(1, 3);
    proof.resume_at(5)?;
    proof.regions.end = 7;
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    assert!(proof.place_at(block(0, 1), scope).is_err());
    assert!(proof.place_at(block(1, 8), scope).is_err());
    assert!(proof.place_at(block(1, 2), ScopeId::ROOT).is_err());
    assert!(proof.place_at(block(1, 2), ScopeId(99)).is_err());
    assert!(proof.place_at(block(1, 7), scope).is_err());
    assert!(proof.place_at(block(3, 5), scope).is_err());
    assert!(proof.place_at(block(3, 2), scope).is_err());
    proof.place_at(block(1, 2), scope)?;
    assert!(proof.place_at(block(1, 2), scope).is_err());
    proof.place_at(block(2, 3), scope)?;
    proof.place_at(block(5, 7), scope)?;
    proof.place_at(block(7, 7), scope)?;
    assert_eq!(proof.owned_regions().collect::<Vec<_>>(), vec![1..3, 5..7]);
    Ok(())
}

#[test]
fn reserved_siblings_cannot_equal_each_others_universal_aliases()
-> Result<(), Box<dyn std::error::Error>> {
    for independent in [false, true] {
        let mut proof = block(1, 3);
        let scope = proof.scope(ScopeId::ROOT, 1)?;
        let sibling = if independent {
            proof.scope(ScopeId::ROOT, 1)?
        } else {
            scope
        };
        proof.place_at(block(1, 2), scope)?;
        proof.place_at(block(2, 3), sibling)?;
        for (id, scope) in [(1, scope), (2, sibling)] {
            let mut debts = block(3, 3);
            debts
                .equalities
                .push((RegionId(id), RegionId::bound(0, 0)?));
            proof.place_at(debts, scope)?;
        }
        proof.equalities.push((RegionId(1), RegionId(2)));
        let store = Store::default();
        let variances = Variances::default();
        let bounds = Bounds::default();
        let callables = crate::types::callable::Templates::default();
        let mut queries = Queries::new(&store, &bounds, &callables);
        let mut constraints = Constraints::new(&store, &variances, 0, 10_000)?;
        proof.require(&mut constraints, &mut queries)?;
        assert_eq!(constraints.prove().is_ok(), !independent);
    }
    Ok(())
}
