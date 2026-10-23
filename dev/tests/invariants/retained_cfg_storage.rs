//! Snapshot and full-template budgets cover their distinct owned storage.
use super::{Catalog, Code, Inline, PreparedOperand};
use crate::test_source as source;
use crate::{body, sema, types::instance::Instance};

#[test]
fn full_bodies_charge_proof_storage_and_snapshots_charge_callable_maps() -> Result<(), String> {
    let report = source::resolve("const fn f()->u32{7}", true)?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let mut program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let body = program.bodies.first_mut().ok_or("missing checked body")?;
    assert_eq!(size_of::<body::ir::Rvalue>(), 20);
    assert_eq!(size_of::<body::ir::Value>(), 24);
    assert_eq!(size_of::<body::ir::Body>(), 448);
    let code = Code::storage_bytes(body)?;
    let scalar = Code::snapshot(body)?;
    assert!(scalar.operands.is_empty());
    assert_eq!(scalar.operands.capacity(), 0);
    assert_eq!(
        size_of::<Vec<PreparedOperand>>(),
        size_of::<Vec<(crate::body::ir::ValueId, Inline, crate::types::TypeId)>>()
    );
    println!(
        "prepared operand bytes={}, former inline entry bytes={}, Code bytes={}, Catalog bytes={}",
        size_of::<PreparedOperand>(),
        size_of::<(crate::body::ir::ValueId, Inline, crate::types::TypeId)>(),
        size_of::<Code>(),
        size_of::<Catalog>()
    );
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
    assert_eq!(Code::storage_bytes(body)?, code);
    let definition = facts
        .functions
        .first()
        .ok_or("missing function")?
        .definition;
    let ty = facts.definitions[definition.0 as usize].ok_or("missing callable")?;
    let instance = Instance::new(&facts.types, ty)?;
    let targets = body.resolved_callables.get_or_insert_with(Box::default);
    targets.insert(instance, instance);
    assert!(Code::storage_bytes(body)? > code);
    let mut catalog = Catalog {
        bytes: 64 * 1024 * 1024,
        ..Catalog::default()
    };
    assert!(catalog.source_results.is_none());
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

#[test]
fn malformed_direct_variant_facts_do_not_publish_snapshots() -> Result<(), String> {
    use crate::{body::ir::Rvalue, sema::model::Cast};
    let report = source::resolve("enum E{A}const fn f()->isize{E::A as isize}", true)?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let original = program.bodies.first().ok_or("missing enum cast body")?;
    let cast = original
        .values
        .iter()
        .position(|v| {
            matches!(
                v.kind,
                Rvalue::Cast {
                    direct_variant: true,
                    ..
                }
            )
        })
        .ok_or("missing direct variant fact")?;
    let Rvalue::Cast { operand, .. } = original.values[cast].kind else {
        return Err("missing cast".into());
    };
    for malformed in 0..3 {
        let mut body = original.clone();
        match malformed {
            0 => {
                body.values[cast].kind = Rvalue::Cast {
                    kind: Cast::IntegerToInteger,
                    operand,
                    direct_variant: true,
                }
            }
            1 => body.values[operand.0 as usize].kind = Rvalue::Unit,
            _ => {
                let Rvalue::Aggregate { variant, fields } =
                    original.values[operand.0 as usize].kind
                else {
                    return Err("missing constructor".into());
                };
                body.values[operand.0 as usize].kind = Rvalue::Aggregate {
                    variant,
                    fields: crate::body::ir::ValueList {
                        end: fields.start + 1,
                        ..fields
                    },
                };
            }
        }
        let mut catalog = Catalog::default();
        assert!(catalog.insert(&body).is_err());
        assert!(catalog.bodies.is_empty());
        assert!(catalog.definitions.is_empty());
        assert!(catalog.functions.is_empty());
        assert_eq!(catalog.bytes, 0);
    }
    Ok(())
}

#[test]
fn owned_publication_consumes_the_snapshot_without_reading_a_later_body() -> Result<(), String> {
    let report = source::resolve("const fn f()->u8{7}", true)?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let mut program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let body = program.bodies.first_mut().ok_or("body")?;
    let values = body.values.len();
    let source = body.source;
    let mut catalog = Catalog::default();
    assert!(super::Prepared::plain(body, 0).is_err());
    assert!(catalog.bodies.is_empty());
    let prepared = super::Prepared::plain(body, 64 * 1024 * 1024)?;
    // No execution permission is asserted by this structural snapshot control.
    // The modified source body is never supplied to the publication consumer.
    body.values.clear();
    body.const_context = false;
    let index = catalog.insert_prepared(prepared)?;
    let code = catalog.code(index).ok_or("owned snapshot")?;
    assert_eq!(code.values.len(), values);
    assert_eq!(code.source, source);
    assert!(code.const_context);
    assert!(code.callables.is_empty());
    assert_eq!(code.callables.capacity(), 0);
    Ok(())
}
