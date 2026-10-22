//! Code-index authority expires with its exact active scope, including nested scopes.
use super::*;
use crate::{pipeline, test_source};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn key() -> Result<Key> {
    let report = test_source::resolve(
        "const fn f<const N:u8>()->u8{const{N}}const A:u8=f::<2>();",
        true,
    )?;
    let facts = pipeline::analyze_source(&report)?.facts;
    let (instance, result, _) = facts
        .inline_constants
        .as_deref()
        .ok_or("registry")?
        .instance_values()
        .next()
        .ok_or("inline")?;
    Ok((instance, result))
}
#[test]
fn nested_code_indexes_restore_and_retired_or_foreign_rows_fail() -> Result {
    let key = key()?;
    let mut index = Scoped::default();
    let mut bytes = 0;
    let outer = index.enter(&mut bytes)?;
    index.insert(outer, key, 3, &mut bytes)?;
    assert_eq!(index.lookup(key), Some(&3));
    index.validate(3)?;
    let inner = index.enter(&mut bytes)?;
    assert!(index.validate(3).is_err());
    index.insert(inner, key, 7, &mut bytes)?;
    assert!(index.retire(outer).is_err());
    index.retire(inner)?;
    assert!(index.validate(7).is_err());
    assert_eq!(index.lookup(key), Some(&3));
    index.retire(outer)?;
    assert!(index.lookup(key).is_none());
    assert!(index.validate(3).is_err());
    assert!(index.insert(outer, key, 9, &mut bytes).is_err());
    Ok(())
}
#[test]
fn scoped_index_budgets_fail_before_cache_publication() -> Result {
    let key = key()?;
    let mut index = Scoped::default();
    let mut bytes = 0;
    let scope = index.enter(&mut bytes)?;
    bytes = 64 * 1024 * 1024;
    assert!(index.insert(scope, key, 5, &mut bytes).is_err());
    assert!(index.lookup(key).is_none());
    assert!(index.codes.is_empty());
    assert!(index.enter(&mut bytes).is_err());
    index.retire(scope)?;
    Ok(())
}
