//! A transported interval is re-proved through the local interning rules: a
//! wrapped signed row is the connected alternative a local pattern interns.
use super::*;
use crate::types::{Primitive, ScalarRange};
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[derive(Default)]
struct Counted {
    work: usize,
    bytes: usize,
    limit: Option<usize>,
}
impl Budget for Counted {
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.work += count;
        Ok(())
    }
    fn reserve(&mut self, bytes: usize) -> Result<(), Error> {
        self.bytes += bytes;
        if self.limit.is_some_and(|limit| self.bytes > limit) {
            return Err(error("fixture consumer storage exhausted"));
        }
        Ok(())
    }
}
const fn row(start: u128, end: u128) -> s::PatternRange {
    s::PatternRange { start, end }
}
fn interval(base: Primitive, start: u128, end: u128) -> ScalarRange {
    ScalarRange {
        base: TypeId::primitive(base),
        start,
        end,
    }
}
#[test]
fn wrapped_rows_intern_the_local_connected_alternative() -> TestResult {
    for (base, minimum, maximum, wrapped) in [
        // `i8 is ..0 | 1..` and `i32 is ..-1 | 0..`.
        (Primitive::I8, 0x80, 0x7f, row(1, 0xff)),
        (
            Primitive::I32,
            0x8000_0000,
            0x7fff_ffff,
            row(0, 0xffff_fffe),
        ),
    ] {
        let mut store = Store::default();
        let mut budget = Counted::default();
        let ty = TypeId::primitive(base);
        let imported = scalar_range(&mut store, ty, wrapped, &mut budget)?;
        assert!(budget.work > 0 && budget.bytes > 0);
        let local = store.intern_scalar_or(
            interval(base, minimum, wrapped.end),
            interval(base, wrapped.start, maximum),
            &|_, _| Ok(()),
        )?;
        assert_eq!(imported, local);
        assert_eq!(
            store.scalar_range(imported),
            Some(interval(base, wrapped.start, wrapped.end))
        );
        for bits in [minimum, wrapped.end, wrapped.start, maximum] {
            assert!(store.range_allows(imported, bits));
        }
        assert!(!store.range_allows(imported, wrapped.end + 1));
    }
    // A closed row in source order is the local closed pattern.
    let mut store = Store::default();
    let percent = scalar_range(
        &mut store,
        TypeId::primitive(Primitive::I8),
        row(0, 100),
        &mut Counted::default(),
    )?;
    let local = store.intern_scalar_range(interval(Primitive::I8, 0, 100), &|_, _| Ok(()))?;
    assert_eq!(percent, local);
    Ok(())
}
#[test]
fn forged_rows_reject_by_the_local_rule_and_budget_refusals_keep_their_error() -> TestResult {
    for (base, range, message) in [
        (
            Primitive::U8,
            row(2, 1),
            "scalar alternatives require a signed integer base",
        ),
        (
            Primitive::Char,
            row(0x0010_ffff, 1),
            "scalar alternatives require a signed integer base",
        ),
        (
            Primitive::Char,
            row(1, 0xd800),
            "scalar pattern endpoint is invalid for its actual base",
        ),
        (
            Primitive::U8,
            row(1, 0x1ff),
            "scalar pattern endpoint is invalid for its actual base",
        ),
        (
            Primitive::U8,
            row(0x1ff, 1),
            "scalar pattern endpoint is invalid for its actual base",
        ),
        (
            Primitive::Bool,
            row(0, 1),
            "scalar pattern requires an integer or char base",
        ),
    ] {
        let mut store = Store::default();
        let extent = store.interned_extent();
        let failed = scalar_range(
            &mut store,
            TypeId::primitive(base),
            range,
            &mut Counted::default(),
        );
        let error = failed.err().ok_or("forged row was accepted")?;
        assert!(error.to_string().contains(message), "{error}");
        assert_eq!(store.interned_extent(), extent);
    }
    let mut store = Store::default();
    let extent = store.interned_extent();
    let mut budget = Counted {
        limit: Some(0),
        ..Counted::default()
    };
    let refused = scalar_range(
        &mut store,
        TypeId::primitive(Primitive::U32),
        row(0, 9),
        &mut budget,
    );
    let error = refused.err().ok_or("exhausted budget was accepted")?;
    assert!(
        error
            .to_string()
            .contains("fixture consumer storage exhausted"),
        "{error}"
    );
    assert_eq!(store.interned_extent(), extent);
    Ok(())
}
/// The 6/24/96 scaling widths. Re-proving a warm interval costs exactly the
/// same work each time. A cold interval also pays the store's table growth,
/// which copies at geometric thresholds, so cold work is bounded linear, not
/// exact: 127, 576 and 2,348 steps in the first candidate run.
#[test]
fn imported_interval_work_grows_linearly() -> TestResult {
    let mut observed = Vec::new();
    for width in [6_u128, 24, 96] {
        let mut store = Store::default();
        let mut budget = Counted::default();
        for index in 0..width {
            scalar_range(
                &mut store,
                TypeId::primitive(Primitive::U32),
                row(0, 1000 + index),
                &mut budget,
            )?;
        }
        let cold = budget.work;
        for index in 0..width {
            scalar_range(
                &mut store,
                TypeId::primitive(Primitive::U32),
                row(0, 1000 + index),
                &mut budget,
            )?;
        }
        observed.push((cold, budget.work - cold));
    }
    let [
        (small, warm_small),
        (middle, warm_middle),
        (large, warm_large),
    ] = <[(usize, usize); 3]>::try_from(observed.as_slice())?;
    assert!(large - middle <= (middle - small) * 4, "{observed:?}");
    assert!(large <= 96 * 25, "{observed:?}");
    assert_eq!(
        [warm_small, warm_middle, warm_large],
        [6 * 14, 24 * 14, 96 * 14],
        "{observed:?}"
    );
    Ok(())
}
