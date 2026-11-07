//! Fresh trial holes retain exclusions; an already allocated prefix stays foreign.
use super::*;
use crate::{sema, types::RegionId};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn gapped() -> Result<Obligations> {
    let mut proof = Obligations::new(1);
    proof.regions.end = 2;
    proof.resume_at(4)?;
    let mut tail = Obligations::new(4);
    tail.regions.end = 5;
    proof.extend(tail)?;
    proof.outlives.push((RegionId(1), RegionId(4)));
    Ok(proof)
}

#[test]
fn fresh_gapped_evidence_uses_its_actual_body_envelope_without_owning_holes() -> Result {
    let report = crate::test_source::resolve("pub fn first(){}pub fn other(){}", true)?;
    let mut facts = sema::declarations::analyze(&report)?;
    let body = |name: &str| -> Result<NodeKey> {
        facts
            .functions
            .iter()
            .find(|function| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[function.definition.0 as usize].name)
                    == name
            })
            .and_then(|function| function.body)
            .ok_or_else(|| format!("actual {name} body").into())
    };
    let first = body("first")?;
    let other = body("other")?;
    facts.body_regions.allocate_reserved(
        BodyRegion {
            body: other,
            source: other,
            origin: BodyRegionOrigin::Inference,
        },
        1,
        12,
    )?;
    assert_eq!(facts.body_regions.frontier(first)?, 1);
    let before = facts.body_regions.log().len();
    assert!(gapped()?.retain_body(&mut facts, other, other).is_err());
    assert_eq!(facts.body_regions.log().len(), before);
    gapped()?.retain_body(&mut facts, first, first)?;
    assert_eq!(facts.body_regions.frontier(first)?, 5);
    assert_eq!(facts.body_regions.frontier(other)?, 13);
    let row = facts
        .quantified_proofs
        .last()
        .ok_or("retained gapped evidence")?;
    assert_eq!(row.body, first);
    assert_eq!(row.evidence.proof.regions, 1..5);
    assert!(row.evidence.proof.owns_region(RegionId(1)));
    assert!(!row.evidence.proof.owns_region(RegionId(2)));
    assert!(!row.evidence.proof.owns_region(RegionId(3)));
    assert!(row.evidence.proof.owns_region(RegionId(4)));
    assert_eq!(row.evidence.proof.outlives, [(RegionId(1), RegionId(4))]);
    assert!(facts.outlives.is_empty());
    let before = facts.body_regions.log().len();
    assert!(gapped()?.retain_body(&mut facts, first, first).is_err());
    assert_eq!(facts.body_regions.log().len(), before);
    Ok(())
}
