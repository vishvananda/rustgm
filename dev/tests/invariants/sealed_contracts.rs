//! A sealed contract ignores later proof reservations and rejects later obligations.
use super::*;
use crate::sema::model::BodyRegion;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn a_sealed_contract_admits_a_later_proof_reservation_but_no_later_variable() -> Result {
    let report = crate::test_source::resolve("fn first()->u8{1}fn main(){let _=first();}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let function = *(facts.functions.iter().rev())
        .find(|function| function.body.is_some())
        .ok_or("missing function")?;
    let body = function.body.ok_or("missing body")?;
    let mut collector = Collector::default();
    let sealed = collector.function(&facts, &function)?.obligations.len();
    // What a source caller of one of the body's instances does: it reserves
    // proof identities in the original body's numbering unit.
    let reserve = |facts: &mut Declarations, origin| {
        let first = facts.body_regions.frontier(body)?;
        let region = BodyRegion {
            body,
            source: body,
            origin,
        };
        facts.body_regions.allocate_reserved(region, first, 2)
    };
    reserve(&mut facts, BodyRegionOrigin::QuantifiedProof)?;
    assert_eq!(
        collector.function(&facts, &function)?.obligations.len(),
        sealed
    );
    // An inference variable would be an obligation the sealed contract lacks.
    reserve(&mut facts, BodyRegionOrigin::Inference)?;
    assert_eq!(
        collector.function(&facts, &function).err(),
        Some("completed lifetime contract received additional facts")
    );
    Ok(())
}
