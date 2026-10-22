//! Image contents require typed provenance and charge before scalar decoding.
#[path = "range_contents.rs"]
mod range_contents;
use super::*;
use crate::resolve::unit::FileId;
use crate::sema::values::{Constant, Evaluator, Reader};
use crate::syntax::ast::NodeId;
use crate::{
    body, sema, test_source,
    types::{Primitive, TypeId},
};
struct Fixture {
    report: crate::resolve::expansion::Report,
    facts: crate::sema::model::Declarations,
    ty: TypeId,
    word: crate::sema::members::Selection,
}
fn fixture() -> Result<Fixture, String> {
    let report = test_source::resolve(
        "union U{word:u32,flag:bool,wide:u64}const fn f(){let u=U{word:1};let _=u;}",
        true,
    )?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let bodies = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let (ty, word) = bodies
        .bodies
        .iter()
        .flat_map(|b| &b.values)
        .find_map(|v| {
            if let body::ir::Rvalue::Union { field, .. } = v.kind {
                Some((v.ty, field))
            } else {
                None
            }
        })
        .ok_or("union initializer")?;
    Ok(Fixture {
        report,
        facts,
        ty,
        word,
    })
}
#[test]
fn contents_validate_image_identity_masks_and_scalar_values() -> Result<(), String> {
    let Fixture {
        report,
        mut facts,
        ty,
        word,
    } = fixture()?;
    let key = NodeKey {
        file: FileId(0),
        node: NodeId::new(0),
    };
    let samples = {
        let mut evaluator = Evaluator::new(&report, &mut facts);
        let mut samples = Vec::new();
        for (bits, index, primitive, expected) in [
            (37, 0, Primitive::U32, Ok(Scalar::Integer(37))),
            (1, 1, Primitive::Bool, Ok(Scalar::Boolean(true))),
            (2, 1, Primitive::Bool, Err("invalid bool representation")),
            (1, 2, Primitive::U64, Err("uninitialized bytes")),
            (1, 1, Primitive::U32, Err("differs from its declared type")),
        ] {
            let (image, _) = evaluator
                .union_value(ty, word, Constant::scalar(Scalar::Integer(bits)), 10000)
                .map_err(|e| e.to_string())?;
            assert!(Reader::new(evaluator.facts()).field(image, index).is_err());
            let base = evaluator
                .allocation(Some(image), ty)
                .map_err(|e| e.to_string())?;
            let child = evaluator
                .projection(base, index, TypeId::primitive(primitive))
                .map_err(|e| e.to_string())?;
            samples.push((
                image.0,
                super::super::references::References::id(base.0)?,
                super::super::references::References::id(child.0)?,
                index,
                expected,
            ));
        }
        samples
    };
    let mut queries = Queries::new(&report, &mut facts);
    for &(image, base, child, index, expected) in &samples {
        let actual = queries.projected_contents(key, child, base, index, image);
        match expected {
            Ok(expected) => {
                assert!(matches!(actual, Ok(Value::Scalar(value)) if value == expected));
            }
            Err(reason) => assert!(
                actual
                    .err()
                    .ok_or("expected rejection")?
                    .to_string()
                    .contains(reason)
            ),
        }
    }
    let &(image, base, child, index, _) = samples.first().ok_or("sample")?;
    assert!(
        queries
            .projected_contents(key, child, base, u64::MAX, image)
            .is_err()
    );
    assert!(
        queries
            .projected_contents(key, child, child, index, image)
            .is_err()
    );
    let retained = queries
        .typing
        .facts
        .constant_images
        .as_ref()
        .ok_or("images")?
        .retained_for_test();
    queries.limit = queries.steps;
    assert!(
        queries
            .projected_contents(key, child, base, index, image)
            .is_err()
    );
    assert_eq!(
        queries
            .typing
            .facts
            .constant_images
            .as_ref()
            .ok_or("images")?
            .retained_for_test(),
        retained
    );
    Ok(())
}
