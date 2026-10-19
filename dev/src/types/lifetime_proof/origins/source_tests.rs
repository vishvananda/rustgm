//! Diagnostic labels must survive composition without granting proof authority.
use crate::types::{
    RegionId, Store, TypeId,
    lifetime_proof::{Obligations, ScopeId},
    outlives::{Bounds, Queries},
    region_graph::Error,
    relations::Constraints,
    substitute::Bound,
    variance::Variances,
};
type ResultTest = Result<(), Box<dyn std::error::Error>>;

fn failure(proof: &Obligations, universals: u32, count: u32) -> Result<Option<u32>, String> {
    let store = Store::default();
    let variances = Variances::default();
    let bounds = Bounds::default();
    let callables = crate::types::callable::Templates::default();
    let mut queries = Queries::new(&store, &bounds, &callables);
    let mut constraints =
        Constraints::new(&store, &variances, universals, 100_000).map_err(|e| e.to_string())?;
    proof.require_sources(&mut constraints, &mut queries, 100, count)?;
    match constraints.solve() {
        Ok(_) => Ok(None),
        Err(Error::Unsatisfied { source, .. } | Error::Verification { source }) => Ok(Some(source)),
        Err(error) => Err(error.to_string()),
    }
}

fn debt(first: u32, longer: RegionId) -> Obligations {
    let mut proof = Obligations::new(first);
    proof.outlives.push((longer, RegionId::STATIC));
    proof
}

#[test]
fn shared_scope_reports_the_one_unpaid_obligations_source() -> ResultTest {
    for unpaid in [false, true] {
        let mut proof = Obligations::new(1);
        let scope = proof.scope(ScopeId::ROOT, 1)?;
        for source in 1..=3 {
            let checkpoint = proof.source_checkpoint();
            let longer = if source == 2 && unpaid {
                RegionId::bound(0, 0)?
            } else {
                RegionId::STATIC
            };
            proof.append_at(debt(1, longer), scope)?;
            proof.label_sources_since(checkpoint, source)?;
        }
        assert_eq!(proof.scope_count(), 1);
        assert_eq!(failure(&proof, 0, 4)?, unpaid.then_some(102));
        assert!(failure(&proof, 0, 3).is_err());
    }
    Ok(())
}

#[test]
fn root_labels_survive_graft_extension_rebase_and_caller_substitution() -> ResultTest {
    let mut store = Store::default();
    let mut source = Obligations::new(1);
    let checkpoint = source.source_checkpoint();
    source.extend(debt(1, RegionId::bound(0, 0)?))?;
    source.label_sources_since(checkpoint, 2)?;
    let mut grafted = Obligations::new(1);
    grafted.graft(source, ScopeId::ROOT)?;
    let mut destination = Obligations::new(1);
    destination.scope(ScopeId::ROOT, 3)?;
    destination.extend(grafted)?;
    destination.rebase(&mut store, 19)?;
    assert_eq!(failure(&destination, 1, 3)?, Some(102));
    destination.substitute(&mut store, &mut Bound::new(vec![RegionId::STATIC]))?;
    assert_eq!(failure(&destination, 0, 3)?, None);
    Ok(())
}

#[test]
fn labelled_grafts_keep_their_ambient_universe() -> ResultTest {
    let mut source = Obligations::new(1);
    let checkpoint = source.source_checkpoint();
    source.extend(debt(1, RegionId::bound(0, 0)?))?;
    source.label_sources_since(checkpoint, 1)?;
    let mut proof = Obligations::new(1);
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    proof.graft(source, scope)?;
    proof.substitute(
        &mut Store::default(),
        &mut Bound::new(vec![RegionId::STATIC]),
    )?;
    assert_eq!(failure(&proof, 0, 2)?, Some(101));
    Ok(())
}

#[test]
fn labels_do_not_claim_foreign_allocations_or_replace_earlier_origins() -> ResultTest {
    let mut proof = Obligations::new(1);
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    let before = proof.source_checkpoint();
    let mut first = Obligations::new(1);
    first.regions.end = 2;
    first.equalities.push((RegionId(1), RegionId::bound(0, 0)?));
    proof.append_at(first, scope)?;
    proof.label_sources_since(before, 1)?;
    assert!(proof.label_sources_since(before, 2).is_err());
    let next = proof.source_checkpoint();
    proof.resume_at(4)?;
    let mut second = Obligations::new(4);
    second.regions.end = 5;
    second.equalities.push((RegionId(4), RegionId(1)));
    proof.append_at(second, scope)?;
    proof.label_sources_since(next, 2)?;
    assert_eq!(proof.owned_regions().collect::<Vec<_>>(), [1..2, 4..5]);
    assert_eq!(failure(&proof, 0, 3)?, None);
    proof.equalities.push((RegionId(2), RegionId(4)));
    assert!(failure(&proof, 0, 3).is_err());
    Ok(())
}

#[test]
fn labels_grow_once_per_new_block_and_ordinary_proofs_keep_no_table() -> ResultTest {
    let mut proof = Obligations::new(1);
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    proof.append_at(debt(1, RegionId::STATIC), scope)?;
    assert!(
        proof
            .quantified
            .as_ref()
            .ok_or("missing scopes")?
            .origins
            .is_none()
    );
    for source in 1..=4096 {
        let checkpoint = proof.source_checkpoint();
        let mut next = Obligations::new(1);
        next.types.push((TypeId::UNIT, RegionId::STATIC));
        proof.append_at(next, scope)?;
        proof.label_sources_since(checkpoint, source)?;
    }
    let q = proof.quantified.as_ref().ok_or("missing scopes")?;
    let labels = q.origins.as_ref().ok_or("missing origins")?;
    assert_eq!(q.scopes.len(), 1);
    assert_eq!(q.blocks.len(), 4097);
    assert_eq!(labels.labels.len(), 4097);
    assert!(labels.labels.capacity() <= 8192);
    assert_eq!(labels.get(0), 0);
    assert_eq!(labels.get(4096), 4096);
    assert_eq!(failure(&proof, 0, 4097)?, None);
    Ok(())
}

#[test]
fn invalid_source_ranges_are_rejected_before_registering_identities() -> ResultTest {
    let store = Store::default();
    let variances = Variances::default();
    let bounds = Bounds::default();
    let callables = crate::types::callable::Templates::default();
    let mut queries = Queries::new(&store, &bounds, &callables);
    let mut constraints = Constraints::new(&store, &variances, 0, 100)?;
    let mut proof = Obligations::new(1);
    let checkpoint = proof.source_checkpoint();
    proof.regions.end = 2;
    proof.outlives.push((RegionId(1), RegionId::STATIC));
    proof.label_sources_since(checkpoint, 2)?;
    for (first, count) in [(100, 0), (u32::MAX, 2), (100, 2)] {
        assert!(
            proof
                .require_sources(&mut constraints, &mut queries, first, count)
                .is_err()
        );
    }
    assert_eq!(constraints.prove()?.identities.variables().len(), 0);
    Ok(())
}

#[test]
fn opened_universals_preserve_the_failing_debt_source_through_composition() -> ResultTest {
    let mut proof = Obligations::new(1);
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    proof.append_universals(1..2, scope)?;
    let first = proof.source_checkpoint();
    proof.append_at(debt(2, RegionId::STATIC), scope)?;
    proof.label_sources_since(first, 1)?;
    let second = proof.source_checkpoint();
    proof.append_at(debt(2, RegionId(1)), scope)?;
    proof.label_sources_since(second, 2)?;
    assert_eq!(failure(&proof, 0, 3)?, Some(102));
    let mut store = Store::default();
    proof.rebase(&mut store, 20)?;
    let mut grafted = Obligations::new(20);
    let ambient = grafted.scope(ScopeId::ROOT, 0)?;
    grafted.graft(proof, ambient)?;
    let mut combined = Obligations::new(20);
    combined.scope(ScopeId::ROOT, 2)?;
    combined.extend(grafted)?;
    combined.substitute(&mut store, &mut Bound::new(vec![RegionId::STATIC]))?;
    assert_eq!(failure(&combined, 0, 3)?, Some(102));
    Ok(())
}

#[test]
fn universal_aliases_require_exact_slots_and_cannot_escape_to_an_outer_variable() -> ResultTest {
    let mut proof = Obligations::new(1);
    proof.regions.end = 2;
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    assert!(proof.append_universals(2..4, scope).is_err());
    assert!(proof.append_universals(2..2, ScopeId::ROOT).is_err());
    assert_eq!(proof.regions, 1..2);
    proof.append_universals(2..3, scope)?;
    assert_eq!(failure(&proof, 0, 1)?, None);
    proof.equalities.push((RegionId(1), RegionId(2)));
    assert_eq!(failure(&proof, 0, 1)?, Some(100));
    Ok(())
}
