//! A failed comparison cannot leave a warm parent marked as a proved match.
use super::*;

#[test]
fn negative_and_error_answers_do_not_cache_unproved_parent_pairs() -> Result<(), Error> {
    let mut store = Store::default();
    let a = TypeId::primitive(crate::types::Primitive::U8);
    let b = TypeId::primitive(crate::types::Primitive::U16);
    let left = store.intern_list(&[a, a]).map_err(|_| Error::Identity)?;
    let right = store.intern_list(&[a, b]).map_err(|_| Error::Identity)?;
    let left = store
        .intern(Kind::Tuple(left))
        .map_err(|_| Error::Identity)?;
    let right = store
        .intern(Kind::Tuple(right))
        .map_err(|_| Error::Identity)?;
    let mut shape = Normalized::<true>::new(&store);
    for _ in 0..3 {
        assert!(!shape.matches(left, right)?);
        assert!(shape.matches(left, left)?);
        assert!(shape.matches(TypeId(u32::MAX), left).is_err());
        assert!(shape.matches(TypeId(u32::MAX), left).is_err());
    }
    Ok(())
}

#[test]
fn repeated_small_failures_preserve_large_completed_subgraphs() -> Result<(), Error> {
    let mut store = Store::default();
    let byte = TypeId::primitive(crate::types::Primitive::U8);
    let word = TypeId::primitive(crate::types::Primitive::U16);
    let mut left = Vec::new();
    let mut right = Vec::new();
    for index in 1..=1024 {
        for (rows, offset) in [(&mut left, 0), (&mut right, 1024)] {
            rows.push(
                store
                    .intern(Kind::Reference {
                        mutable: false,
                        referent: byte,
                        region: RegionId(index + offset),
                    })
                    .map_err(|_| Error::Identity)?,
            );
        }
    }
    let left = store.intern_list(&left).map_err(|_| Error::Identity)?;
    let right = store.intern_list(&right).map_err(|_| Error::Identity)?;
    let left = store
        .intern(Kind::Tuple(left))
        .map_err(|_| Error::Identity)?;
    let right = store
        .intern(Kind::Tuple(right))
        .map_err(|_| Error::Identity)?;
    let mut shape = Normalized::<true>::new(&store);
    assert!(shape.matches(left, right)?);
    let completed = shape.seen.len();
    assert!(completed >= 1024);
    let before = shape.steps;
    for _ in 0..128 {
        assert!(!shape.matches(byte, word)?);
    }
    assert_eq!(shape.seen.len(), completed);
    assert!(shape.steps - before <= 128 * 8);
    assert!(shape.matches(left, right)?);
    Ok(())
}

#[test]
fn paid_correspondence_preserves_cold_growth_warm_pairs_and_real_sticky_failures()
-> Result<(), &'static str> {
    use std::cell::Cell;
    let mut store = Store::default();
    let byte = TypeId::primitive(crate::types::Primitive::U8);
    let left = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId(1),
        referent: byte,
    })?;
    let right = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId(2),
        referent: byte,
    })?;
    let bytes = Cell::new(0usize);
    let pay = |_: usize, amount: usize| {
        bytes.set(bytes.get().saturating_add(amount));
        Ok(())
    };
    let mut shape = Normalized::<false>::new_charged(&store, &pay, Memo::default());
    assert!(shape.matches_charged(left, right)?);
    let cold = bytes.get();
    assert!(cold > 0);
    for _ in 0..8 {
        assert!(shape.matches_charged(left, right)?);
    }
    assert_eq!(bytes.get(), cold);
    assert!(shape.matches_charged(TypeId(u32::MAX), left).is_err());
    let mut resumed = Normalized::<false>::new_charged(&store, &pay, shape.into_memo());
    assert!(resumed.matches_charged(left, right)?);
    assert_eq!(bytes.get(), cold);
    let refuse = |_: usize, amount: usize| {
        if amount == 0 {
            Ok(())
        } else {
            Err("actual test payer exhausted")
        }
    };
    let mut denied = Normalized::<false>::new_charged(&store, &refuse, Memo::default());
    assert_eq!(
        denied.matches_charged(left, right),
        Err("actual test payer exhausted")
    );
    assert!(denied.work.is_empty());
    assert!(denied.seen.is_empty());
    assert_eq!(
        denied.matches_charged(byte, byte),
        Err("actual test payer exhausted")
    );
    assert_eq!(size_of::<Normalized<'_, false>>(), 72);
    assert_eq!(size_of::<Normalized<'_, false, PaidCost<'_>>>(), 104);
    Ok(())
}

#[test]
fn shared_prefix_correspondence_work_grows_with_unique_pairs() -> Result<(), &'static str> {
    for depth in [8, 32, 128] {
        let mut store = Store::default();
        let byte = TypeId::primitive(crate::types::Primitive::U8);
        let mut left = store.intern(Kind::Reference {
            mutable: false,
            region: RegionId(1),
            referent: byte,
        })?;
        let mut right = store.intern(Kind::Reference {
            mutable: false,
            region: RegionId(2),
            referent: byte,
        })?;
        for _ in 0..depth {
            let a = store.intern_list(&[left, left])?;
            let b = store.intern_list(&[right, right])?;
            left = store.intern(Kind::Tuple(a))?;
            right = store.intern(Kind::Tuple(b))?;
        }
        let work = std::cell::Cell::new(0usize);
        let pay = |amount: usize, _: usize| {
            work.set(work.get().saturating_add(amount));
            Ok(())
        };
        let mut fingerprints = Shapes::default();
        assert_eq!(
            fingerprints.get_charged(&store, left, &pay)?,
            fingerprints.get_charged(&store, right, &pay)?
        );
        let mut shape = Normalized::<false>::new_charged(&store, &pay, Memo::default());
        assert!(shape.matches_charged(left, right)?);
        assert!(
            shape.steps <= 12 * depth + 32,
            "depth{depth}: {}",
            shape.steps
        );
        assert!(fingerprints.steps <= 128 * depth + 128);
        let before = work.get();
        for _ in 0..8 {
            assert!(shape.matches_charged(left, right)?);
        }
        assert!(work.get() - before <= 8 * 8);
    }
    Ok(())
}
