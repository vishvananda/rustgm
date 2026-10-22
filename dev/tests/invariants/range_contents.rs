//! Source range snapshots enforce the same owner, extent, fragments and preallocation budget.
use super::*;
use crate::{
    sema::values::{ImageFragment, WordTarget},
    types::Kind,
};
struct RangeFixture {
    report: crate::resolve::expansion::Report,
    facts: crate::sema::model::Declarations,
    value: Value,
    base: Id,
    child: Id,
}
fn fixture() -> Result<RangeFixture, String> {
    let report = test_source::resolve("", true)?;
    let mut facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let ty = facts.types.intern(Kind::Array {
        element: TypeId::primitive(Primitive::U8),
        length: 16,
    })?;
    let (value, base, child) = {
        let mut evaluator = Evaluator::new(&report, &mut facts);
        let selected = evaluator
            .byte_array_range_type(ty, 2, 4)
            .map_err(|e| e.to_string())?;
        evaluator
            .storage_layout(selected, 10000)
            .map_err(|e| e.to_string())?;
        let pointer = evaluator
            .allocation(
                Some(Constant::scalar(Scalar::Integer(37))),
                TypeId::primitive(Primitive::U32),
            )
            .map_err(|e| e.to_string())?;
        let fragments = [ImageFragment {
            offset: 0,
            source_offset: 0,
            length: 8,
            target: WordTarget::Value(pointer.restrict_shared()),
        }];
        let (value, _) = evaluator
            .byte_array_snapshot(ty, &[7; 16], &[65535], &fragments, 10000)
            .map_err(|e| e.to_string())?;
        let base = evaluator
            .allocation(Some(value), ty)
            .map_err(|e| e.to_string())?;
        let child = evaluator
            .subsequence(base, 2, 4, selected)
            .map_err(|e| e.to_string())?;
        (
            value.0,
            super::super::super::references::References::id(base.0)?,
            super::super::super::references::References::id(child.0)?,
        )
    };
    Ok(RangeFixture {
        report,
        facts,
        value,
        base,
        child,
    })
}
#[test]
fn source_image_ranges_preserve_fragments_and_reject_mismatched_storage() -> Result<(), String> {
    let RangeFixture {
        report,
        mut facts,
        value,
        base,
        child,
    } = fixture()?;
    let key = NodeKey {
        file: FileId(0),
        node: NodeId::new(0),
    };
    let mut queries = Queries::new(&report, &mut facts);
    // The empty-source fixture supplies its explicit positive budget; exhaustion is tested below.
    queries.limit = 10000;
    let copy = queries
        .range_contents(key, child, base, 2, 4, value)
        .map_err(|e| e.to_string())?;
    let view = Reader::new(queries.typing.facts)
        .array_image(Constant(copy))?
        .ok_or("range image")?;
    assert_eq!(view.bytes.len(), 4);
    assert_eq!(
        (
            view.fragments[0].offset,
            view.fragments[0].source_offset,
            view.fragments[0].length
        ),
        (0, 2, 4)
    );
    assert!(
        queries
            .range_contents(key, child, child, 2, 4, value)
            .is_err()
    );
    assert!(
        queries
            .range_contents(key, base, base, 2, 4, value)
            .is_err()
    );
    assert!(
        queries
            .range_contents(key, child, base, 15, 4, value)
            .is_err()
    );
    let retained = queries
        .typing
        .facts
        .constant_images
        .as_ref()
        .ok_or("arena")?
        .retained_for_test();
    queries.limit = queries.steps;
    assert!(
        queries
            .range_contents(key, child, base, 2, 4, value)
            .is_err()
    );
    assert_eq!(
        queries
            .typing
            .facts
            .constant_images
            .as_ref()
            .ok_or("arena")?
            .retained_for_test(),
        retained
    );
    Ok(())
}
