//! Private equality payment never equates distinct caller identities.
use super::*;
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

fn equations(
    private: Range<u32>,
    pairs: &[(RegionId, RegionId)],
) -> std::result::Result<Equations, String> {
    let mut result = Equations::new(private);
    for &(a, b) in pairs {
        let a = result.region(a, &mut |_| Ok(()))?;
        let b = result.region(b, &mut |_| Ok(()))?;
        result.pairs.push((a, b));
    }
    Ok(result)
}
#[test]
fn owned_negative_equations_preserve_rigid_callers() -> Result {
    let (_, facts) = super::super::tests::declarations("fn main(){}")?;
    assert!(equations(2..3, &[(RegionId(1), RegionId(2))])?.pay(&facts, 1000)?);
    assert!(
        !equations(
            3..4,
            &[(RegionId(1), RegionId(3)), (RegionId(2), RegionId(3))]
        )?
        .pay(&facts, 1000)?
    );
    assert!(
        !equations(
            1..2,
            &[
                (RegionId::bound(0, 0)?, RegionId(1)),
                (RegionId::bound(0, 1)?, RegionId(1))
            ]
        )?
        .pay(&facts, 1000)?
    );
    assert!(equations(2..3, &[(RegionId(1), RegionId(3))]).is_err());
    assert!(equations(2..3, &[(RegionId::bound(1, 0)?, RegionId(2))]).is_err());
    assert!(
        equations(2..3, &[(RegionId(1), RegionId(2))])?
            .pay(&facts, 0)
            .is_err()
    );
    Ok(())
}
#[test]
fn interrupted_allocation_envelopes_are_not_private_payment() -> Result {
    let mut allocations = super::super::allocations::Allocations::new(2);
    allocations.record(3);
    assert_eq!(allocations.plain(), Some(2..3));
    allocations.reserve_scoped();
    assert_eq!(allocations.plain(), None);
    allocations.resume_at(8)?;
    assert_eq!(allocations.plain(), None);
    Ok(())
}

#[test]
fn private_equations_register_only_touched_slots() -> Result {
    let (_, facts) = super::super::tests::declarations("fn main(){}")?;
    for (private, relevant) in [(2..3, 2), (2..402, 401), (1_000_002..1_000_402, 1_000_401)] {
        let proof = equations(private.clone(), &[(RegionId(1), RegionId(relevant))])?;
        assert_eq!(proof.private, private);
        assert_eq!(
            (proof.private_count, proof.rigid_count, proof.regions.len()),
            (1, 1, 2)
        );
        assert_eq!(proof.pairs, [(RegionId::bound(0, 0)?, RegionId(1))]);
        assert_eq!(proof.limit(), 200);
        assert!(proof.pay(&facts, 200)?);
    }
    let proof = equations(
        2..402,
        &[
            (RegionId::STATIC, RegionId(401)),
            (RegionId::STATIC, RegionId(401)),
        ],
    )?;
    assert_eq!(
        (proof.private_count, proof.rigid_count, proof.regions.len()),
        (1, 0, 1)
    );
    assert!(proof.pay(&facts, 1000)?);
    let pairs: Vec<_> = (2..402).map(|id| (RegionId(1), RegionId(id))).collect();
    let proof = equations(2..402, &pairs)?;
    assert_eq!((proof.private_count, proof.rigid_count), (400, 1));
    assert!(proof.limit() > 1_000_000);
    let assumptions = crate::sema::traits::assumptions::Assumptions::default();
    let mut search = Search::new(
        super::super::tests::caller(&assumptions, None),
        crate::sema::traits::implementations::Availability::Declarations,
    );
    assert!(search.account_query_work(proof.limit()).is_err());
    assert_eq!(search.query.steps, proof.limit());
    assert_eq!(size_of::<Equations>(), 72);
    Ok(())
}
#[test]
fn failed_map_payment_does_not_publish_an_identity() {
    let mut proof = Equations::new(2..402);
    let error = proof.region(RegionId(401), &mut |_| Err("paid failure".into()));
    assert_eq!(error, Err("paid failure".into()));
    assert_eq!(
        (proof.private_count, proof.rigid_count, proof.regions.len()),
        (0, 0, 0)
    );
}
