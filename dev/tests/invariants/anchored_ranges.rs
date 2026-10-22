//! Original scope ranges admit only their own anchored lifetime aliases.
use super::*;
use crate::{
    sema::{declarations, model::RegionEquality},
    types::RegionId,
};
#[test]
fn missing_foreign_and_stale_original_equalities_grant_no_alias()
-> Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve("const A:u8=7;const B:u8=9;fn main(){}", true)?;
    let mut facts = declarations::analyze(&report)?;
    let body = facts.initializers[0].semantic_source();
    let other = facts.initializers[1].semantic_source();
    let mut prepared = PreparedScope {
        scope: super::super::Scope {
            values: crate::sema::constants::PatternScope {
                body,
                owner: Some(facts.initializers[0].definition),
                binders: 1,
                end: 0,
            },
            module: crate::resolve::package::ModuleId(0),
            ranges: 0,
            roots: 0,
            matches: 0,
            proof: Some(Ranges::new(0..2, 2..2)?),
        },
        requests: Vec::new(),
        ranges: Vec::new(),
        roots: Vec::new(),
        matches: Vec::new(),
    };
    facts.region_equalities = vec![
        RegionEquality {
            body,
            left: RegionId(1),
            right: RegionId(2),
        },
        RegionEquality {
            body,
            left: RegionId(2),
            right: RegionId::bound(0, 0)?,
        },
    ];
    let (mut fold, _) = prepared.caller_regions(&facts, None)?;
    assert_eq!(fold.region(RegionId(1), 0)?, RegionId::bound(0, 0)?);
    assert!(fold.region(RegionId(3), 0).is_err());
    facts.region_equalities[1].body = other;
    assert!(prepared.caller_regions(&facts, None).is_err());
    facts.region_equalities[1].body = body;
    prepared.scope.proof = Some(Ranges::new(0..1, 1..1)?);
    let (mut fold, _) = prepared.caller_regions(&facts, None)?;
    assert!(fold.region(RegionId(1), 0).is_err());
    prepared.scope.proof = Some(Ranges::new(0..3, 3..3)?);
    assert!(prepared.caller_regions(&facts, None).is_err());
    prepared.scope.proof = None;
    assert!(prepared.caller_regions(&facts, None).is_err());
    assert_eq!(size_of::<Ranges>(), 16);
    assert_eq!(size_of::<Option<Ranges>>(), 16);
    Ok(())
}
