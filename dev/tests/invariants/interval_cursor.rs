//! Appended evidence preserves private ownership without rescanning prior body debts.
use super::ProofCursor;
use crate::{
    body::regions::{Kind, Obligation, Proof},
    types::{
        RegionId,
        lifetime_proof::{Obligations, ScopeId},
    },
};

#[test]
fn repeated_proofs_keep_disjoint_intervals_and_linear_allocation_work()
-> Result<(), Box<dyn std::error::Error>> {
    let mut observations = Vec::new();
    for count in [8u32, 32, 128] {
        let report = crate::test_source::resolve("fn main(){}", true)?;
        let mut facts = crate::pipeline::analyze_source(&report)?.facts;
        let mut contract = crate::body::regions::collect(&facts)?
            .bodies
            .into_iter()
            .next()
            .ok_or("missing contract")?;
        for index in 0..count {
            contract.obligations.push(Obligation {
                source: contract.source,
                kind: Kind::Variable {
                    id: RegionId(42 + index),
                    needs_provenance: false,
                },
            });
        }
        let mut earlier = Obligations::new(1000);
        earlier.regions.end = 1002;
        contract.proofs.push(Proof::new(contract.source, earlier));
        let before = facts.body_work.canonical_steps;
        let mut cursor = ProofCursor::default();
        for index in 0..count {
            let mut proof = Obligations::new(1);
            proof.regions.end = 2;
            proof.equalities.push((RegionId(1), RegionId::bound(0, 0)?));
            let scope = proof.scope(ScopeId::ROOT, 1)?;
            proof.append_universals(2..3, scope)?;
            cursor.retain(
                &mut facts,
                &mut contract,
                proof,
                std::iter::once((0, RegionId(42))),
            )?;
            let retained = &contract
                .proofs
                .last()
                .ok_or("missing retained proof")?
                .proof;
            let first = 1002 + 2 * index;
            assert_eq!(retained.regions, first..first + 2);
            assert_eq!(retained.equalities, [(RegionId(first), RegionId(42))]);
            assert_eq!(retained.scope_count(), 1);
            assert_eq!(retained.owned_regions().flatten().count(), 2);
        }
        let work = facts.body_work.canonical_steps - before;
        eprintln!(
            "proof-cursor/{count}: work={work}, retained={}",
            contract.proofs.len()
        );
        observations.push(work);
    }
    assert!(observations[2] - observations[1] <= 5 * (observations[1] - observations[0]));
    Ok(())
}
