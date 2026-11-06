//! Persistent match bindings agree with ordinary maps, including failed unions.
use super::{Error, Id, Map};
use std::collections::BTreeMap;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn charged_radix_growth_is_prepaid_and_cached_unions_keep_the_same_identity() -> Result {
    let mut map = Map::<u32>::default();
    assert_eq!(
        map.singleton_charged(1, 2, &|_, _| Err(Error::Budget)),
        Err(Error::Budget)
    );
    assert!(map.nodes.is_empty() && map.index.is_empty() && map.unions.is_empty());
    let bytes = std::cell::Cell::new(0usize);
    let pay = |_, amount| {
        bytes.set(bytes.get().saturating_add(amount));
        Ok(())
    };
    let left = map.singleton_charged(1, 2, &pay)?;
    let right = map.singleton_charged(u32::MAX, 7, &pay)?;
    let state = (map.nodes.len(), map.index.len(), map.unions.len());
    assert_eq!(
        map.union_charged(left, right, &|_, _| Err(Error::Budget)),
        Err(Error::Budget)
    );
    assert_eq!((map.nodes.len(), map.index.len(), map.unions.len()), state);
    let merged = map
        .union_charged(left, right, &pay)?
        .ok_or("paid disjoint union")?;
    assert_eq!(map.union(left, right)?, Some(merged));
    let retained = bytes.get();
    for _ in 0..32 {
        assert_eq!(map.union_charged(right, left, &pay)?, Some(merged));
        assert_eq!(map.singleton_charged(1, 2, &pay)?, left);
    }
    assert_eq!(bytes.get(), retained);
    let conflict = map.singleton_charged(1, 9, &pay)?;
    assert_eq!(map.union_charged(left, conflict, &pay)?, None);
    assert_eq!(map.get(merged, 1)?, Some(2));
    Ok(())
}

#[test]
fn unions_are_canonical_commutative_and_conflict_preserving() -> Result {
    let keys = [0, 1, 2, 3, 127, 128, 65535, 65536, 1 << 31, u32::MAX];
    let mut map = Map::default();
    let mut roots = Vec::new();
    for mask in 0..64 {
        let mut root = Id::EMPTY;
        let mut expected = BTreeMap::new();
        for (index, key) in keys.iter().copied().enumerate() {
            if mask & (1 << (index % 6)) != 0 {
                let value = key ^ (mask & 2);
                let leaf = map.singleton(key, value)?;
                root = map.union(root, leaf)?.ok_or("disjoint insertion failed")?;
                expected.insert(key, value);
            }
        }
        roots.push((root, expected));
    }
    for (a, left) in &roots {
        for (b, right) in &roots {
            let conflict = left
                .iter()
                .any(|(key, value)| right.get(key).is_some_and(|other| other != value));
            let merged = map.union(*a, *b)?;
            assert_eq!(merged, map.union(*b, *a)?);
            assert_eq!(merged.is_none(), conflict);
            if let Some(root) = merged {
                let mut expected = left.clone();
                expected.extend(right);
                let mut reverse = Id::EMPTY;
                for (&key, &value) in expected.iter().rev() {
                    let leaf = map.singleton(key, value)?;
                    reverse = map
                        .union(reverse, leaf)?
                        .ok_or("reverse insertion failed")?;
                }
                assert_eq!(root, reverse);
                for key in keys {
                    assert_eq!(map.get(root, key)?, expected.get(&key).copied());
                }
                assert_eq!(map.get(root, 5)?, None);
            }
            for (&key, &value) in left {
                assert_eq!(map.get(*a, key)?, Some(value));
            }
        }
    }
    Ok(())
}

#[test]
fn growing_prefixes_share_storage_and_cached_unions() -> Result {
    for width in [16_u32, 64, 256, 1024] {
        let mut map = Map::default();
        let mut root = Id::EMPTY;
        let mut roots = Vec::new();
        for key in 0..width {
            let leaf = map.singleton(key, key + 1)?;
            root = map.union(root, leaf)?.ok_or("prefix insertion failed")?;
            roots.push(root);
        }
        // At most 32 radix levels, with shared untouched siblings. This bound
        // rules out copying every prior binding into each successive result.
        assert!(map.nodes.len() < 12 * width as usize);
        assert!(map.steps < 40 * width as usize);
        let before = map.steps;
        for (key, prefix) in roots.into_iter().enumerate() {
            assert_eq!(
                map.get(prefix, u32::try_from(key)?)?,
                Some(u32::try_from(key)? + 1)
            );
            assert_eq!(map.union(prefix, prefix)?, Some(prefix));
        }
        assert!(map.steps - before < 12 * width as usize);
        let invalid = Id(u32::MAX);
        assert!(matches!(map.get(invalid, 0), Err(Error::Identity)));
    }
    Ok(())
}
