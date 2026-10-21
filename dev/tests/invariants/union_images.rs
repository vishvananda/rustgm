//! Budget failures precede publication; copies preserve immutable overlapping bytes.
use super::*;
use crate::{body, sema, sema::model::Scalar, test_source};
#[test]
fn image_updates_preserve_snapshots_and_cache_family_layout() -> Result<(), String> {
    let Fixture {
        report,
        mut facts,
        ty,
        word,
        byte,
    } = fixture()?;
    let mut evaluator = Evaluator::new(&report, &mut facts);
    let value = Constant::scalar(Scalar::Integer(0x1122_3344));
    assert!(evaluator.union_value(ty, word, value, 0).is_err());
    assert!(evaluator.facts().constant_images.is_none());
    let (original, _) = evaluator
        .union_value(ty, word, value, 10000)
        .map_err(Failure::message)?;
    let layout_steps = evaluator.facts().layouts.steps;
    let retained = evaluator
        .facts()
        .constant_images
        .as_ref()
        .ok_or("arena")?
        .retained_for_test();
    assert!(
        evaluator
            .union_write(original, ty, byte, Constant::scalar(Scalar::Integer(9)), 1)
            .is_err()
    );
    assert_eq!(
        evaluator
            .facts()
            .constant_images
            .as_ref()
            .ok_or("arena")?
            .retained_for_test(),
        retained
    );
    let (changed, _) = evaluator
        .union_write(
            original,
            ty,
            byte,
            Constant::scalar(Scalar::Integer(0xaa)),
            10000,
        )
        .map_err(Failure::message)?;
    assert!(original != changed);
    for _ in 0..32 {
        let (old, _) = evaluator
            .union_read(original, ty, word, 10000)
            .map_err(Failure::message)?;
        let (new, _) = evaluator
            .union_read(changed, ty, word, 10000)
            .map_err(Failure::message)?;
        assert_eq!(
            old.as_scalar().map_err(Failure::message)?,
            Scalar::Integer(0x1122_3344)
        );
        assert_eq!(
            new.as_scalar().map_err(Failure::message)?,
            Scalar::Integer(0x1122_33aa)
        );
    }
    assert_eq!(evaluator.facts().layouts.steps, layout_steps);
    assert_eq!(
        Reader::new(evaluator.facts())
            .image(original)
            .map_err(str::to_owned)?
            .initialized,
        &[15]
    );
    assert!(
        evaluator
            .union_read(original, ty, Selection::Tuple(0), 10000)
            .is_err()
    );
    Ok(())
}

struct Fixture {
    report: crate::resolve::expansion::Report,
    facts: crate::sema::model::Declarations,
    ty: TypeId,
    word: Selection,
    byte: Selection,
}
fn fixture() -> Result<Fixture, String> {
    let report = test_source::resolve(
        "union U{word:u32,byte:u8}const fn f()->u32{let mut u=U{word:1};u.byte=2;unsafe{u.word}}",
        true,
    )?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let bodies = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let body = bodies.bodies.first().ok_or("body")?;
    let (ty, word) = body
        .values
        .iter()
        .find_map(|v| {
            if let body::ir::Rvalue::Union { field, .. } = v.kind {
                Some((v.ty, field))
            } else {
                None
            }
        })
        .ok_or("union initializer")?;
    let byte = body
        .projections
        .iter()
        .find_map(|p| {
            if let body::ir::ProjectionKind::UnionField(field) = p.kind {
                Some(field)
            } else {
                None
            }
        })
        .ok_or("byte field")?;
    Ok(Fixture {
        report,
        facts,
        ty,
        word,
        byte,
    })
}
#[test]
fn image_addresses_retain_capabilities_and_validate_owners() -> Result<(), String> {
    use crate::sema::values::{Origin, ProjectionStorage, WritePermission};
    let Fixture {
        report,
        mut facts,
        ty,
        word,
        byte,
    } = fixture()?;
    let mut evaluator = Evaluator::new(&report, &mut facts);
    let (image, _) = evaluator
        .union_value(ty, word, Constant::scalar(Scalar::Integer(7)), 10000)
        .map_err(Failure::message)?;
    let base = evaluator
        .allocation(Some(image), ty)
        .map_err(Failure::message)?;
    assert!(evaluator.union_address(base, ty, word, 0).is_err());
    assert!(
        evaluator
            .union_address(Constant::scalar(Scalar::Integer(0)), ty, word, 10000)
            .is_err()
    );
    assert!(
        evaluator
            .union_address(base, TypeId::primitive(Primitive::U32), word, 10000)
            .is_err()
    );
    assert!(
        evaluator
            .union_address(base, ty, Selection::Tuple(0), 10000)
            .is_err()
    );
    assert!(
        evaluator
            .union_address(
                base,
                ty,
                Selection::Nominal(crate::sema::members::FieldId(u32::MAX)),
                10000
            )
            .is_err()
    );
    for selected in [word, byte] {
        for address in [base, base.restrict_shared()] {
            let (pointer, _) = evaluator
                .union_address(address, ty, selected, 10000)
                .map_err(Failure::message)?;
            assert_eq!(pointer.write_permission(), address.write_permission());
            let reader = Reader::new(evaluator.facts());
            let location = reader
                .location(pointer.location().ok_or("pointer location")?)
                .map_err(str::to_owned)?;
            let Origin::Projection {
                base: origin,
                index,
            } = location.origin
            else {
                return Err("projection origin".into());
            };
            assert_eq!(Some(origin), base.location());
            let projection = reader.projection_layout(ty, index).map_err(str::to_owned)?;
            assert_eq!(projection.ty, location.ty);
            assert_eq!(projection.offset, 0);
            assert!(
                matches!(projection.storage,ProjectionStorage::UnionScalar { field } if field==selected)
            );
        }
    }
    assert_eq!(base.write_permission(), Some(WritePermission::Writable));
    Ok(())
}
