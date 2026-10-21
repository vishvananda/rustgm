//! Checked-empty entries and shared payloads retain distinct source authority.
use super::*;
#[test]
fn condition_payload_is_shared_and_duplicate_publication_is_immutable() -> Result<(), &'static str>
{
    let pairs: Arc<[Pair]> = [(RegionId::STATIC, RegionId::STATIC)].into();
    let mut catalog = Catalog::default();
    catalog.retain(ClosureId(1), Some(&pairs))?;
    assert_eq!(Arc::strong_count(&pairs), 2);
    let payload = catalog.payload;
    catalog.retain(ClosureId(1), Some(&pairs))?;
    assert_eq!(catalog.payload, payload);
    assert_eq!(Arc::strong_count(&pairs), 2);
    assert!(catalog.retain(ClosureId(1), None).is_err());
    catalog.retain(ClosureId(2), None)?;
    assert!(matches!(catalog.values.get(&ClosureId(2)), Some(None)));
    assert!(!catalog.values.contains_key(&ClosureId(3)));
    catalog.payload = Catalog::LIMIT;
    assert!(catalog.retain(ClosureId(3), None).is_err());
    assert!(!catalog.values.contains_key(&ClosureId(3)));
    Ok(())
}
