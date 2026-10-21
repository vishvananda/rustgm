//! Snapshot and full-template budgets cover their distinct owned storage.
use super::{Catalog, Code};
use crate::test_source as source;
use crate::{body, sema, types::instance::Instance};

#[test]
fn full_bodies_charge_proof_storage_and_snapshots_charge_callable_maps() -> Result<(), String> {
    let report = source::resolve("const fn f()->u32{7}", true)?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let mut program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let body = program.bodies.first_mut().ok_or("missing checked body")?;
    let code = Code::storage_bytes(body);
    let full = body.retained_bytes();
    body.guard_places.reserve(16);
    body.temporaries.reserve(16);
    body.storage
        .as_mut()
        .ok_or("missing storage facts")?
        .exits
        .reserve(16);
    let mut owned = Box::<body::drops::values::Facts>::default();
    owned.created.reserve(16);
    owned.returned.reserve(16);
    body.owned = Some(owned);
    assert!(body.retained_bytes() > full);
    assert_eq!(Code::storage_bytes(body), code);
    let definition = facts
        .functions
        .first()
        .ok_or("missing function")?
        .definition;
    let ty = facts.definitions[definition.0 as usize].ok_or("missing callable")?;
    let instance = Instance::new(&facts.types, ty)?;
    let targets = body.resolved_callables.get_or_insert_with(Box::default);
    targets.insert(instance, instance);
    assert!(Code::storage_bytes(body) > code);
    let mut catalog = Catalog {
        bytes: 64 * 1024 * 1024,
        ..Catalog::default()
    };
    assert_eq!(
        catalog.insert(body).err(),
        Some("constant execution code exceeds 64 MiB budget")
    );
    assert!(catalog.bodies.is_empty());
    assert!(catalog.definitions.is_empty());
    assert!(catalog.functions.is_empty());
    assert_eq!(catalog.bytes, 64 * 1024 * 1024);
    Ok(())
}
