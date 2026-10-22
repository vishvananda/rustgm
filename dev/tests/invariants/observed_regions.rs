//! Observation budgets preserve binders and never become restoration authority.
use super::*;
use crate::types::{Kind, Store, TypeId};

fn reference(store: &mut Store, region: RegionId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region,
        referent: TypeId::UNIT,
    })
}

#[test]
fn observed_free_ids_preserve_nested_binders_and_fixed_restore_authority()
-> Result<(), &'static str> {
    assert_eq!(size_of::<Bound>(), 192);
    assert_eq!(size_of::<Mapping>(), 64);
    let mut store = Store::default();
    let local = reference(&mut store, RegionId::bound(0, 0)?)?;
    let outer = reference(&mut store, RegionId(19))?;
    let parameters = store.intern_list(&[local, outer])?;
    let signature = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: local,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let mut fold = Bound::observe_budgeted_regions(1000, 1 << 20)?;
    assert_eq!(fold.apply(&mut store, signature)?, signature);
    assert_eq!(fold.collect_observed_free_regions()?, [RegionId(19)]);
    fold.validate_observation_owner()?;
    assert!(fold.validate_restore_owner().is_err());
    let mut restore = Bound::restore_budgeted_regions(HashMap::default(), 1000, 1 << 20)?;
    assert!(restore.region(RegionId(19), 0).is_err());
    assert!(restore.validate_observation_owner().is_err());
    let mut ordinary = Bound::observe_regions();
    assert_eq!(ordinary.apply(&mut store, signature)?, signature);
    assert_eq!(
        ordinary.observed_regions().collect::<Vec<_>>(),
        [RegionId(19)]
    );
    Ok(())
}

#[test]
fn observation_preflights_map_growth_and_retains_failed_work() -> Result<(), &'static str> {
    assert!(Bound::observe_budgeted_regions(100, 0).is_err());
    let bytes = size_of::<Bound>() + size_of::<Mapping>() + size_of::<Owner>();
    let mut fold = Bound::observe_budgeted_regions(100, bytes)?;
    for expected in 1..=2 {
        assert!(fold.region(RegionId(7), 0).is_err());
        assert_eq!(fold.steps, expected);
        assert_eq!(fold.observed_regions().count(), 0);
        assert_eq!(fold.region_retained_bytes()?, bytes);
    }
    let mut store = Store::default();
    let root = reference(&mut store, RegionId(7))?;
    let mut work = Bound::observe_budgeted_regions(0, 1 << 20)?;
    assert!(work.apply(&mut store, root).is_err());
    assert_eq!(work.steps, 1);
    assert!(work.work.is_empty());
    let mut changed = Bound::observe_budgeted_regions(100, 1 << 20)?;
    changed.arguments.push(RegionId(1));
    assert!(changed.apply(&mut store, root).is_err());
    assert!(changed.validate_observation_owner().is_err());
    Ok(())
}

#[test]
fn observation_cache_hits_and_output_keep_the_persistent_budget() -> Result<(), &'static str> {
    let mut store = Store::default();
    let root = reference(&mut store, RegionId(7))?;
    let mut fold = Bound::observe_budgeted_regions(1000, 1 << 20)?;
    fold.apply(&mut store, root)?;
    let before = fold.steps;
    let bytes = fold.region_retained_bytes()?;
    for _ in 0..32 {
        assert_eq!(fold.apply(&mut store, root)?, root);
    }
    assert_eq!(fold.steps, before + 32);
    assert_eq!(fold.region_retained_bytes()?, bytes);
    assert_eq!(fold.collect_observed_free_regions()?, [RegionId(7)]);
    assert_eq!(fold.region_retained_bytes()?, bytes);
    fold.validate_observation_owner()?;
    Ok(())
}

#[test]
fn observation_output_and_signature_scratch_are_preflighted_before_growth()
-> Result<(), &'static str> {
    let mut store = Store::default();
    let root = reference(&mut store, RegionId(7))?;
    let mut fold = Bound::observe_budgeted_regions(1000, 1 << 20)?;
    fold.apply(&mut store, root)?;
    let bytes = fold.region_retained_bytes()?;
    let Action::Regions(mapping) = &mut fold.action else {
        return Err("mapping");
    };
    let Mode::ObserveBudget(owner) = &mut mapping.mode else {
        return Err("owner");
    };
    owner.budget = Budget::new(bytes, 1000);
    let steps = fold.steps;
    assert!(fold.collect_observed_free_regions().is_err());
    assert!(fold.steps > steps);
    assert_eq!(fold.region_retained_bytes()?, bytes);
    assert_eq!(fold.observed_regions().collect::<Vec<_>>(), [RegionId(7)]);
    let parameters = store.intern_list(&[root; 32])?;
    let signature = store.intern(Kind::Function {
        parameters,
        result: root,
        bound_regions: 0,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    assert!(fold.signature(&mut store, signature, 0).is_err());
    assert_eq!(fold.region_retained_bytes()?, bytes);
    fold.validate_observation_owner()?;
    Ok(())
}
