//! Shared metadata cannot reopen installation or produce any proof permission.
use super::super::tests::{definitions, input};
use super::*;
use crate::resolve::model::DefKind;

#[test]
fn empty_source_compilation_never_allocates_a_shared_catalog() -> Result<(), &'static str> {
    let mut slot = CatalogSlot::default();
    slot.freeze()?;
    assert!(slot.get().is_none());
    assert!(slot.shared()?.is_none());
    assert_eq!(size_of::<CatalogSlot>(), size_of::<usize>());
    Ok(())
}

#[test]
fn clone_drop_and_unique_access_never_reopen_frozen_installation() -> Result<(), &'static str> {
    let defs = definitions(2);
    let mut slot = CatalogSlot::default();
    assert_eq!(
        slot.ensure()?.retained_bytes(),
        size_of::<Catalog>() + 2 * size_of::<usize>()
    );
    assert!(slot.shared().is_err());
    let body = slot
        .ensure()?
        .install(&defs, input("frozen", 0, 1)?)?
        .body(BodyRef(0))?;
    slot.freeze()?;
    let first = slot.shared()?.ok_or("missing frozen catalog")?;
    let second = slot.shared()?.ok_or("missing shared catalog")?;
    assert!(Rc::ptr_eq(&first, &second));
    assert_eq!(first.coordinate(body)?.1, BodyRef(0));
    let bytes = first.retained_bytes();
    drop(first);
    drop(second);
    assert!(slot.ensure().is_err());
    slot.freeze()?;
    let unique = Rc::get_mut(slot.0.as_mut().ok_or("missing frozen owner")?)
        .ok_or("unexpected catalog borrower")?;
    let work = unique.work();
    assert!(unique.install(&defs, input("later", 1, 1)?).is_err());
    assert_eq!(unique.work() - work, 1);
    let work = unique.work();
    let mut callback_calls = 0;
    assert!(
        super::super::publication::install(unique, &defs, input("later", 1, 1)?, |_| {
            callback_calls += 1;
            Ok(())
        })
        .is_err()
    );
    assert_eq!(callback_calls, 0);
    assert_eq!(unique.work() - work, 1);
    assert_eq!(unique.body_count(), 1);
    assert_eq!(unique.retained_bytes(), bytes);
    Ok(())
}

#[test]
fn separately_frozen_catalogs_reject_equal_numeric_foreign_owners() -> Result<(), &'static str> {
    let defs = definitions(1);
    let mut first = CatalogSlot::default();
    let body = first
        .ensure()?
        .install(&defs, input("same", 0, 1)?)?
        .body(BodyRef(0))?;
    first.freeze()?;
    let left = first.shared()?.ok_or("missing first catalog")?;
    let declaration = left.declaration(DefId(0))?;
    let mut second = CatalogSlot::default();
    second.ensure()?.install(&defs, input("same", 0, 1)?)?;
    second.freeze()?;
    let right = second.shared()?.ok_or("missing second catalog")?;
    assert!(!Rc::ptr_eq(&left, &right));
    assert!(right.coordinate(body).is_err());
    assert!(
        right
            .validate_declaration(declaration, DefId(0), DefKind::Function)
            .is_err()
    );
    assert!(second.ensure().is_err());
    Ok(())
}
