//! Budget failures precede publication; copies preserve immutable overlapping bytes.
use super::*;
use crate::{body, sema, sema::model::Scalar, test_source};
#[test]
fn image_updates_preserve_snapshots_and_cache_family_layout() -> Result<(), String> {
    let report = test_source::resolve(
        "union U{word:u32,byte:u8}const fn f()->u32{let mut u=U{word:1};u.byte=2;unsafe{u.word}}",
        true,
    )?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let mut facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
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
