//! Artifact ordinals resolve through the one authenticated, current catalog.
use super::*;
use crate::support::crates::owner_catalog::{BodyRef, DeclarationRef};
#[test]
fn coordinate_lookups_preserve_artifact_kind_bounds_and_constant_work() -> Result<(), &'static str>
{
    let definitions = crate::resolve::body_origin::tests::definitions(4);
    let mut catalog = Catalog::new(Limits::default())?;
    let left = catalog.install(
        &definitions,
        crate::resolve::body_origin::tests::input("left", 0, 2)?,
    )?;
    let right = catalog.install(
        &definitions,
        crate::resolve::body_origin::tests::input("right", 2, 2)?,
    )?;
    let a = CrateIdentity::new("provider", "left")?;
    let b = CrateIdentity::new("provider", "right")?;
    assert_eq!(catalog.definition_at(&a, DeclarationRef(1))?, DefId(1));
    assert_eq!(catalog.definition_at(&b, DeclarationRef(1))?, DefId(3));
    assert_eq!(catalog.body_at(&a, BodyRef(1))?, left.body(BodyRef(1))?);
    assert_eq!(catalog.body_at(&b, BodyRef(1))?, right.body(BodyRef(1))?);
    assert_ne!(
        catalog.body_at(&a, BodyRef(1))?,
        catalog.body_at(&b, BodyRef(1))?
    );
    for owner in [&a, &b] {
        assert!(catalog.definition_at(owner, DeclarationRef(2)).is_err());
        assert!(catalog.body_at(owner, BodyRef(2)).is_err());
    }
    let unknown = CrateIdentity::new("provider", "unknown")?;
    assert!(catalog.definition_at(&unknown, DeclarationRef(0)).is_err());
    assert!(catalog.body_at(&unknown, BodyRef(0)).is_err());
    let before = catalog.work();
    for _ in 0..64 {
        catalog.definition_at(&a, DeclarationRef(0))?;
        catalog.body_at(&b, BodyRef(0))?;
    }
    assert_eq!(catalog.work() - before, 64 * 5);
    assert_eq!(size_of::<Artifact>(), 144);
    assert_eq!(size_of::<OwnerRow>(), 40);
    Ok(())
}
