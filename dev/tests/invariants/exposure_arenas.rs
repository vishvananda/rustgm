//! Sparse exposure recipes bind exact live inventories before every warm lookup.
use super::*;
use crate::types::{GenericArgs, Kind, RegionId, dynamic::Bounds};

fn dynamic(types: &mut Store, owner: u32) -> Result<TypeId, &'static str> {
    let shape = types.intern_dynamic_shape(Bounds {
        principal: Some(ItemId(owner)),
        principal_arguments: 0,
        bindings: &[],
        auto_traits: &[],
    })?;
    types.intern(Kind::Dynamic {
        shape,
        arguments: GenericArgs::EMPTY,
        bound_regions: 0,
        region: RegionId::STATIC,
    })
}

#[test]
fn exact_exposure_cache_rejects_foreign_inventory_before_warm_false_or_recipe()
-> Result<(), &'static str> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let members = Members::default();
    let root = dynamic(&mut types, 99)?;
    let cache = Cache::exposed_empty(&mut types, &mut definitions)?;
    let mut catalog = cache.attach(&types, &definitions, &members);
    let id = catalog.expose(root)?.ok_or("dynamic recipe")?;
    assert!(matches!(
        catalog.get(id).ok_or("recipe")?.kind,
        super::super::Shape::Dynamic
    ));
    assert_eq!(catalog.expose(TypeId::UNIT)?, None);
    let cache = catalog.into_cache();
    let before = cache.steps;
    let mut foreign_types = Store::default();
    let foreign_root = dynamic(&mut foreign_types, 99)?;
    assert_eq!(root, foreign_root);
    let mut catalog = cache.attach(&foreign_types, &definitions, &members);
    for ty in [root, TypeId::UNIT] {
        assert!(catalog.expose(ty).is_err());
    }
    assert_eq!(catalog.steps(), before);
    let cache = catalog.into_cache();
    let foreign_definitions = Definitions::default();
    let mut catalog = cache.attach(&types, &foreign_definitions, &members);
    assert!(catalog.demand(root).is_err());
    assert_eq!(catalog.steps(), before);
    let cache = catalog.into_cache();
    let (mut moved_types, moved_definitions) = (types, definitions);
    moved_types.intern(Kind::Array {
        element: TypeId::UNIT,
        length: 128,
    })?;
    let mut catalog = cache.attach(&moved_types, &moved_definitions, &members);
    assert_eq!(catalog.expose(root)?, Some(id));
    assert_eq!(catalog.expose(TypeId::UNIT)?, None);
    assert_eq!(catalog.steps(), before);
    Ok(())
}

#[test]
fn sparse_dynamic_exposure_work_tracks_distinct_roots_with_no_store_noise_cost()
-> Result<(), &'static str> {
    let mut baseline = None;
    for noise in [0, 8192] {
        let mut types = Store::default();
        let mut definitions = Definitions::default();
        let members = Members::default();
        for length in 0..noise {
            types.intern(Kind::Array {
                element: TypeId::UNIT,
                length,
            })?;
        }
        let roots = (0..128)
            .map(|owner| dynamic(&mut types, owner))
            .collect::<Result<Vec<_>, _>>()?;
        let cache = Cache::exposed_empty(&mut types, &mut definitions)?;
        let mut catalog = cache.attach(&types, &definitions, &members);
        let mut measured = Vec::new();
        for width in [8, 32, 128] {
            for &root in &roots[..width] {
                assert!(catalog.expose(root)?.is_some());
            }
            let before = catalog.steps();
            let bytes = catalog.cache.needs.retained_bytes();
            let retained = catalog.cache.retained;
            for _ in 0..128 {
                assert!(catalog.expose(roots[0])?.is_some());
            }
            assert_eq!(catalog.steps(), before);
            assert_eq!(catalog.cache.needs.retained_bytes(), bytes);
            assert_eq!(catalog.cache.retained, retained);
            assert_eq!(catalog.cache.storage.entries.len(), width);
            assert!(before <= 4 * width);
            measured.push((before, bytes, retained));
        }
        println!(
            "noise={noise} dynamic width/steps/query/charge={measured:?}; Cache={}",
            size_of::<Cache>()
        );
        if let Some(expected) = &baseline {
            assert_eq!(&measured, expected);
        } else {
            baseline = Some(measured);
        }
    }
    Ok(())
}

#[test]
fn failed_exact_exposure_work_cannot_reuse_previously_warm_false() -> Result<(), &'static str> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let members = Members::default();
    let root = dynamic(&mut types, 99)?;
    let cache = Cache::exposed_empty(&mut types, &mut definitions)?;
    let mut catalog = cache.attach(&types, &definitions, &members);
    assert_eq!(catalog.expose(TypeId::UNIT)?, None);
    catalog.cache.steps = 1_000_000;
    assert!(catalog.expose(root).is_err());
    assert!(catalog.expose(TypeId::UNIT).is_err());
    assert!(catalog.cache.work.is_empty());
    assert!(
        catalog
            .cache
            .states
            .values()
            .all(|state| !matches!(state, State::Active))
    );
    Ok(())
}

#[test]
fn exact_exposure_preflights_capacity_before_retaining_any_failed_growth()
-> Result<(), &'static str> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let members = Members::default();
    let cache = Cache::exposed_empty(&mut types, &mut definitions)?;
    let mut catalog = cache.attach(&types, &definitions, &members);
    assert_eq!(catalog.expose(TypeId::UNIT)?, None);
    let cache = &mut catalog.cache;
    let before = (
        cache.states.capacity(),
        cache.storage.entries.capacity(),
        cache.storage.fields.capacity(),
        cache.storage.variants.capacity(),
        cache.storage.exposed.capacity(),
        cache.work.capacity(),
        cache.needs.retained_bytes(),
    );
    assert!(cache.reserve_exposure(usize::MAX).is_err());
    assert_eq!(
        (
            cache.states.capacity(),
            cache.storage.entries.capacity(),
            cache.storage.fields.capacity(),
            cache.storage.variants.capacity(),
            cache.storage.exposed.capacity(),
            cache.work.capacity(),
            cache.needs.retained_bytes()
        ),
        before
    );
    assert!(catalog.expose(TypeId::UNIT).is_err());
    Ok(())
}

#[test]
fn dynamic_recipes_keep_full_object_binder_and_region_identity() -> Result<(), &'static str> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let members = Members::default();
    let first = dynamic(&mut types, 99)?;
    let Some(Kind::Dynamic {
        shape, arguments, ..
    }) = types.kind(first)
    else {
        return Err("dynamic root");
    };
    let region = types.intern(Kind::Dynamic {
        shape,
        arguments,
        bound_regions: 0,
        region: RegionId::bound(0, 0)?,
    })?;
    let binder = types.intern(Kind::Dynamic {
        shape,
        arguments,
        bound_regions: 1,
        region: RegionId::STATIC,
    })?;
    let cache = Cache::exposed_empty(&mut types, &mut definitions)?;
    let mut catalog = cache.attach(&types, &definitions, &members);
    let mut selected = Vec::new();
    for root in [first, region, binder] {
        let id = catalog.expose(root)?.ok_or("object recipe")?;
        let recipe = catalog.get(id).ok_or("recipe")?;
        assert_eq!(recipe.ty, root);
        assert!(recipe.method.is_none());
        assert!(matches!(recipe.kind, super::super::Shape::Dynamic));
        assert!(!selected.contains(&id));
        selected.push(id);
    }
    Ok(())
}
