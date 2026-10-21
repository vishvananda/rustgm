//! Region opening/restoration must preserve nested binders and reject leaked solver identities.
use rustgm::types::{Kind, RegionId, Store, TypeId, substitute::Bound};
use std::collections::HashMap;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn opening_fast_path_checks_trait_arguments_and_respects_local_binders() -> TestResult {
    use rustgm::types::{
        GenericArgs, ItemId,
        arguments::Argument,
        parameters::predicates::{Bound as Requirement, Predicate},
        traits::Application,
    };
    let mut store = Store::default();
    let local = reference(&mut store, RegionId::bound(0, 0)?)?;
    let closed = pointer(&mut store, &[local], local)?;
    let application = store.intern_trait_application(Application {
        item: ItemId(1),
        arguments: GenericArgs::EMPTY,
    })?;
    assert!(
        !Predicate {
            ty: closed,
            bound: Requirement::Trait(application)
        }
        .has_free_regions(&store)?
    );
    for region in [RegionId::bound(0, 0)?, RegionId(3)] {
        let arguments = store.intern_arguments(&[Argument::Lifetime(region)])?;
        let application = store.intern_trait_application(Application {
            item: ItemId(1),
            arguments,
        })?;
        assert!(
            Predicate {
                ty: closed,
                bound: Requirement::Trait(application)
            }
            .has_free_regions(&store)?
        );
    }
    Ok(())
}

fn reference(store: &mut Store, region: RegionId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region,
        referent: TypeId::UNIT,
    })
}
fn pointer(store: &mut Store, fields: &[TypeId], result: TypeId) -> Result<TypeId, &'static str> {
    let parameters = store.intern_list(fields)?;
    store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })
}

#[test]
fn opening_and_restoring_preserve_multiple_surrounding_and_local_binders() -> TestResult {
    let mut store = Store::default();
    let local = reference(&mut store, RegionId::bound(0, 0)?)?;
    let captured = reference(&mut store, RegionId::bound(1, 7)?)?;
    let outer = reference(&mut store, RegionId::bound(3, 4)?)?;
    let free = reference(&mut store, RegionId(2))?;
    let static_ = reference(&mut store, RegionId::STATIC)?;
    let original = pointer(&mut store, &[local, captured, outer, free], static_)?;
    let mut open = Bound::open_regions(10)?;
    let opened = open.apply(&mut store, original)?;
    assert_eq!(open.next_region(), Some(12));
    let mapping: HashMap<_, _> = open.opened_regions().collect();
    assert!(mapping.contains_key(&RegionId::bound(0, 7)?));
    assert!(mapping.contains_key(&RegionId::bound(2, 4)?));
    assert_eq!(mapping.get(&RegionId(2)), Some(&RegionId(2)));
    let Some(Kind::Function { parameters, .. }) = store.kind(opened) else {
        return Err("missing opened signature".into());
    };
    assert_eq!(
        store.list(parameters).ok_or("missing parameters")?[0],
        local
    );
    let before = open.steps;
    assert_eq!(open.apply(&mut store, original)?, opened);
    assert_eq!(open.steps, before);
    let mut restore = Bound::restore_regions(mapping.into_iter().map(|(a, b)| (b, a)).collect());
    assert_eq!(restore.apply(&mut store, opened)?, original);
    Ok(())
}

#[test]
fn restoring_raises_captured_results_beneath_new_local_binders() -> TestResult {
    let mut store = Store::default();
    let local = reference(&mut store, RegionId::bound(0, 0)?)?;
    let selected = reference(&mut store, RegionId(21))?;
    let signature = pointer(&mut store, &[local], selected)?;
    let captured = reference(&mut store, RegionId::bound(3, 9)?)?;
    let expected = pointer(&mut store, &[local], captured)?;
    let mut restore =
        Bound::restore_regions(HashMap::from([(RegionId(21), RegionId::bound(2, 9)?)]));
    assert_eq!(restore.apply(&mut store, signature)?, expected);
    Ok(())
}

#[test]
fn opening_requires_disjoint_free_identities_and_checked_allocation() -> TestResult {
    let mut store = Store::default();
    assert!(Bound::open_regions(0).is_err());
    assert!(Bound::open_regions(0x8000_0000).is_err());
    let existing = reference(&mut store, RegionId(10))?;
    assert!(
        Bound::open_regions(10)?
            .apply(&mut store, existing)
            .is_err()
    );
    let captured = reference(&mut store, RegionId::bound(0, 0)?)?;
    assert!(
        Bound::open_regions(0x7fff_ffff)?
            .apply(&mut store, captured)
            .is_err()
    );
    Ok(())
}

#[test]
fn restoration_is_simultaneous_and_rejects_unknown_or_overflowing_captures() -> TestResult {
    let mut store = Store::default();
    let a = reference(&mut store, RegionId(1))?;
    let b = reference(&mut store, RegionId(2))?;
    let c = reference(&mut store, RegionId(3))?;
    let mut restore = Bound::restore_regions(HashMap::from([
        (RegionId(1), RegionId(2)),
        (RegionId(2), RegionId(3)),
    ]));
    assert_eq!(restore.apply(&mut store, a)?, b);
    assert_eq!(restore.apply(&mut store, b)?, c);
    assert!(restore.apply(&mut store, c).is_err());
    let signature = pointer(&mut store, &[], a)?;
    let mut overflowing =
        Bound::restore_regions(HashMap::from([(RegionId(1), RegionId::bound(0x7fff, 0)?)]));
    assert!(overflowing.apply(&mut store, signature).is_err());
    Ok(())
}

#[test]
fn rebasing_preserves_all_bound_lifetimes_and_only_freshens_its_interval() -> TestResult {
    let mut store = Store::default();
    let local = reference(&mut store, RegionId::bound(0, 0)?)?;
    let outer = reference(&mut store, RegionId::bound(3, 4)?)?;
    let private = reference(&mut store, RegionId(2))?;
    let fresh = reference(&mut store, RegionId(22))?;
    let original = pointer(&mut store, &[local, outer, private], private)?;
    let expected = pointer(&mut store, &[local, outer, fresh], fresh)?;
    let mut fold = Bound::rebase_free_regions(1..4, 21)?;
    assert_eq!(fold.apply(&mut store, original)?, expected);
    let static_ = reference(&mut store, RegionId::STATIC)?;
    assert_eq!(fold.apply(&mut store, static_)?, static_);
    let foreign = reference(&mut store, RegionId(4))?;
    assert!(fold.apply(&mut store, foreign).is_err());
    for (range, first) in [
        (0..1, 10),
        (std::ops::Range { start: 3, end: 2 }, 10),
        (1..2, 0),
        (1..3, 0x7fff_fffe),
        (1..0x8000_0000, 1),
    ] {
        assert!(Bound::rebase_free_regions(range, first).is_err());
    }
    Ok(())
}
