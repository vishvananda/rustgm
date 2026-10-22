//! Shape budgeting does not change ordinary folds or rescan shared type graphs.
use super::*;
use crate::types::{Primitive, RegionId};
#[test]
fn bounded_shape_fold_checks_tiny_budgets_and_preserves_warm_cache() -> Result<(), &'static str> {
    let mut store = Store::default();
    let reference = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId(7),
        referent: TypeId::primitive(Primitive::U32),
    })?;
    let mut fold = Bound::shape();
    let before = fold.shape_retained_bytes()?;
    assert!(fold.apply_bounded(&mut store, reference, 100, 0).is_err());
    assert_eq!(fold.shape_retained_bytes()?, before);
    let error = fold
        .apply_bounded(&mut store, reference, 0, 1 << 20)
        .err()
        .ok_or("zero work accepted")?;
    assert_eq!(error, "bounded shape fold work budget exceeded");
    assert_eq!(fold.steps, 1);
    assert!(fold.work.is_empty());
    let value = fold.apply_bounded(&mut store, reference, 100, 1 << 20)?;
    let mut ordinary = Bound::shape();
    assert_eq!(value, ordinary.apply(&mut store, reference)?);
    let steps = fold.steps;
    let bytes = fold.shape_retained_bytes()?;
    for _ in 0..128 {
        assert_eq!(fold.apply_bounded(&mut store, reference, 0, bytes)?, value);
    }
    assert_eq!(fold.steps, steps);
    assert_eq!(fold.shape_retained_bytes()?, bytes);
    Ok(())
}
#[test]
fn bounded_shape_fold_work_follows_shared_roots_not_store_noise() -> Result<(), &'static str> {
    for width in [8usize, 32, 128] {
        let mut measured = Vec::new();
        for noise in [0, 4096] {
            let mut store = Store::default();
            let leaf = store.intern(Kind::Reference {
                mutable: false,
                region: RegionId(7),
                referent: TypeId::primitive(Primitive::U8),
            })?;
            let fields = store.intern_list(&vec![leaf; width])?;
            let shared = store.intern(Kind::Tuple(fields))?;
            for length in 0..noise {
                store.intern(Kind::Array {
                    element: leaf,
                    length,
                })?;
            }
            let mut fold = Bound::shape();
            for index in 0..width {
                let root = store.intern(Kind::Reference {
                    mutable: false,
                    region: RegionId(8 + u32::try_from(index).map_err(|_| "test region")?),
                    referent: shared,
                })?;
                fold.apply_bounded(&mut store, root, 100_000, 1 << 24)?;
            }
            assert!(
                fold.steps <= 8 * width + 16,
                "width={width} work={}",
                fold.steps
            );
            measured.push((fold.steps, fold.shape_retained_bytes()?));
        }
        assert_eq!(measured[0], measured[1]);
        println!(
            "shape width={width} work={} retained={}",
            measured[0].0, measured[0].1
        );
    }
    Ok(())
}
