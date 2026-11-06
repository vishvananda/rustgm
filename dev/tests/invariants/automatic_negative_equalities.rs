//! Private equality payment never equates distinct caller identities.
use super::*;
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

fn equations(
    private: Range<u32>,
    pairs: &[(RegionId, RegionId)],
) -> std::result::Result<Equations, String> {
    let mut result = Equations {
        private,
        rigid: HashMap::default(),
        pairs: Vec::new(),
    };
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
