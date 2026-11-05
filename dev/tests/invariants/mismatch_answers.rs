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
