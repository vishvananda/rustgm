//! Interval identity is paid, store-local and independent of physical carriers.
use super::*;
use std::cell::Cell;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn interval(base: Primitive, start: u128, end: u128) -> ScalarRange {
    ScalarRange {
        base: TypeId::primitive(base),
        start,
        end,
    }
}
#[test]
fn signed_values_failed_payment_and_warm_work_keep_canonical_identity() -> TestResult {
    let mut store = Store::default();
    assert!(store.arguments.is_none());
    let steps = Cell::new(0_usize);
    let bytes = Cell::new(0_usize);
    let pay = |work, growth| {
        steps.set(steps.get() + work);
        bytes.set(bytes.get() + growth);
        Ok(())
    };
    let range = interval(Primitive::I8, 254, 2);
    let id = store.intern_scalar_range(range, &pay)?;
    for bits in [254, 255, 0, 1, 2] {
        assert!(store.range_allows(id, bits));
    }
    for bits in [253, 3, 256] {
        assert!(!store.range_allows(id, bits));
    }
    assert!(steps.get() > 0 && bytes.get() > 0);
    let growth = bytes.get();
    for _ in 0..100 {
        assert_eq!(store.intern_scalar_range(range, &pay)?, id);
    }
    assert_eq!(bytes.get(), growth);
    let extent = store.interned_extent();
    assert!(
        store
            .intern_scalar_range(interval(Primitive::I8, 3, 2), &pay)
            .is_err()
    );
    assert_eq!(store.interned_extent(), extent);
    let mut failed = Store::default();
    let attempted = Cell::new(0_usize);
    assert!(
        failed
            .intern_scalar_range(range, &|work, growth| {
                attempted.set(attempted.get() + work);
                if growth > 0 {
                    Err("fixture storage exhausted")
                } else {
                    Ok(())
                }
            })
            .is_err()
    );
    assert!(attempted.get() > 0);
    assert!(failed.arguments.is_none());
    let mut cold = None;
    let mut warm = None;
    for width in [8_u128, 32, 128] {
        let mut store = Store::default();
        let work = Cell::new(0_usize);
        let pay = |n, _| {
            work.set(work.get() + n);
            Ok(())
        };
        for n in 0..width {
            store.intern_scalar_range(interval(Primitive::U32, n, n + 5), &pay)?;
        }
        if let Some(previous) = cold {
            assert!(work.get() <= previous * 8);
        }
        cold = Some(work.get());
        let before = work.get();
        for _ in 0..1000 {
            store.intern_scalar_range(interval(Primitive::U32, 0, 5), &pay)?;
        }
        let actual = work.get() - before;
        if let Some(previous) = warm {
            assert_eq!(actual, previous);
        }
        warm = Some(actual);
    }
    Ok(())
}
#[test]
fn contents_normalize_across_stores_and_preserve_hot_record_widths() -> TestResult {
    let mut a = Store::default();
    let mut b = Store::default();
    let range = interval(Primitive::U32, 7, 9);
    let ai = a.intern_scalar_range(range, &|_, _| Ok(()))?;
    b.intern_scalar_range(interval(Primitive::U32, 11, 13), &|_, _| Ok(()))?;
    let bi = b.intern_scalar_range(a.scalar_range(ai).ok_or("actual source range")?, &|_, _| {
        Ok(())
    })?;
    assert_ne!(ai, bi);
    assert_ne!(a.scalar_range(ai), b.scalar_range(ai));
    assert_eq!(a.scalar_range(ai), b.scalar_range(bi));
    let at = a.intern(Kind::Pattern {
        base: range.base,
        constraint: Constraint::ScalarRange(ai),
    })?;
    let bt = b.intern(Kind::Pattern {
        base: range.base,
        constraint: Constraint::ScalarRange(bi),
    })?;
    let ah = super::super::structural::Shapes::default().get(&a, at)?;
    let bh = super::super::structural::Shapes::default().get(&b, bt)?;
    assert_eq!(ah, bh);
    let folded = super::super::substitute::Bound::shape().apply(&mut a, at)?;
    assert_eq!(folded, at);
    let before = a.interned_extent();
    assert_eq!(
        super::super::substitute::lookup::Lookup::new(0).ty(&a, at)?,
        Some(at)
    );
    assert_eq!(a.interned_extent(), before);
    let raw = a.intern(Kind::Pointer {
        mutable: false,
        pointee: range.base,
    })?;
    assert!(
        a.intern(Kind::Pattern {
            base: raw,
            constraint: Constraint::ScalarRange(ai)
        })
        .is_err()
    );
    assert_eq!(size_of::<Kind>(), 24);
    assert_eq!(size_of::<Constraint>(), 8);
    assert_eq!(size_of::<Store>(), 472);
    assert_eq!(size_of::<crate::types::infer::Constructor>(), 16);
    assert_eq!(size_of::<crate::syntax::ast::Kind>(), 28);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    eprintln!(
        "scalar physical Store={} argumentsStorage={} Kind={} Constraint={} Node={} Body={}",
        size_of::<Store>(),
        size_of::<super::super::arguments::Storage>(),
        size_of::<Kind>(),
        size_of::<Constraint>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::body::ir::Body>()
    );
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/closed-scalar-patterns-v1");
    std::fs::create_dir_all(&output)?;
    std::fs::write(
        output.join("layout-after.txt"),
        format!(
            "Store={} argumentsStorage={} Kind={} Constraint={} Node={} Body={}\n",
            size_of::<Store>(),
            size_of::<super::super::arguments::Storage>(),
            size_of::<Kind>(),
            size_of::<Constraint>(),
            size_of::<crate::syntax::ast::Node>(),
            size_of::<crate::body::ir::Body>(),
        ),
    )?;
    Ok(())
}

#[test]
fn signed_alternatives_retain_one_interval_without_weakening_closed_order() -> TestResult {
    let mut store = Store::default();
    let first = interval(Primitive::I8, 128, 255);
    let second = interval(Primitive::I8, 1, 127);
    let id = store.intern_scalar_or(first, second, &|_, _| Ok(()))?;
    assert_eq!(
        store.scalar_range(id),
        Some(interval(Primitive::I8, 1, 255))
    );
    for n in [1, 127, 128, 255] {
        assert!(store.range_allows(id, n));
    }
    assert!(!store.range_allows(id, 0));
    assert!(
        store
            .intern_scalar_range(interval(Primitive::I8, 1, 255), &|_, _| Ok(()))
            .is_err()
    );
    for (a, b) in [
        (
            interval(Primitive::U8, 0, 2),
            interval(Primitive::U8, 5, 255),
        ),
        (
            interval(Primitive::I8, 128, 1),
            interval(Primitive::I8, 1, 127),
        ),
        (
            interval(Primitive::I8, 128, 255),
            interval(Primitive::I8, 1, 126),
        ),
        (
            interval(Primitive::I8, 246, 251),
            interval(Primitive::I8, 255, 127),
        ),
    ] {
        assert!(store.intern_scalar_or(a, b, &|_, _| Ok(())).is_err());
    }
    let ty = store.intern(Kind::Pattern {
        base: first.base,
        constraint: Constraint::ScalarRange(id),
    })?;
    assert_eq!(
        super::super::substitute::Bound::shape().apply(&mut store, ty)?,
        ty
    );
    assert_eq!(
        super::super::substitute::lookup::Lookup::new(0).ty(&store, ty)?,
        Some(ty)
    );
    Ok(())
}
