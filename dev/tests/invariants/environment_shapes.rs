//! Exact capture ownership, layout and empty product validity are independent facts.
use super::*;
use crate::{sema::values::Evaluator, test_source};

#[test]
fn closure_storage_facade_requires_completed_exact_tuple_layout() -> Result<(), String> {
    let report = test_source::resolve("fn f(){let x=37u32;let f=move||x;let _=f;}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let (ty, environment) = (0..facts.types.len())
        .find_map(|n| {
            let ty = TypeId(u32::try_from(n).ok()?);
            match facts.types.kind(ty) {
                Some(Kind::Closure { environment, .. }) if environment != TypeId::UNIT => {
                    Some((ty, environment))
                }
                _ => None,
            }
        })
        .ok_or("closure")?;
    assert!(Reader::new(&facts).storage_shape(ty).is_err());
    Evaluator::new(&report, &mut facts)
        .storage_layout(ty, 10000)
        .map_err(|e| e.message().to_owned())?;
    let layout = *facts.layouts.get(ty).ok_or("layout")?;
    let reader = Reader::new(&facts);
    assert!(
        matches!(reader.storage_shape(ty)?,StoredShape::Product{fields,variant:None,union:false} if fields == [TypeId::primitive(crate::types::Primitive::U32)])
    );
    assert!(reader.stored_closure(TypeId::UNIT, &layout).is_err());
    assert!(
        reader
            .stored_closure(TypeId::primitive(crate::types::Primitive::U32), &layout)
            .is_err()
    );
    let mut wrong = layout;
    wrong.size = wrong.size.saturating_add(1);
    assert!(reader.stored_closure(environment, &wrong).is_err());
    let mut wrong = layout;
    wrong.align = wrong.align.saturating_add(1);
    assert!(reader.stored_closure(environment, &wrong).is_err());
    let Shape::Aggregate {
        variants,
        tag: None,
    } = layout.shape
    else {
        return Err("shape".into());
    };
    let mut wrong = layout;
    wrong.shape = Shape::Aggregate {
        variants,
        tag: Some(crate::abi::layout::Tag {
            ty: crate::types::Primitive::U8,
            offset: 0,
        }),
    };
    assert!(reader.stored_closure(environment, &wrong).is_err());
    let mut wrong = layout;
    wrong.shape = Shape::Aggregate {
        variants: crate::abi::layout::Range {
            start: variants.start,
            end: variants.end.saturating_add(1),
        },
        tag: None,
    };
    assert!(reader.stored_closure(environment, &wrong).is_err());
    Ok(())
}
