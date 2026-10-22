//! Only a checked complete application map authorizes physical head selection.
use super::*;
use crate::{
    sema::values::Evaluator,
    test_source,
    types::{GenericArgs, copy, opaque},
};

#[test]
fn opaque_storage_heads_require_exact_applications_and_completed_layouts() -> Result<(), String> {
    let report = test_source::resolve("", true)?;
    let mut facts = crate::sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let id = facts.types.declare_opaque(opaque::Declaration {
        trait_result: false,
        sized: true,
        copies: copy::Properties::default(),
    })?;
    let ty = facts.types.intern(Kind::Opaque(id, GenericArgs::EMPTY))?;
    assert!(Reader::new(&facts).storage_type(ty).is_err());
    let head = TypeId::primitive(crate::types::Primitive::U32);
    facts.types.publish_storage_head(ty, head)?;
    assert!(Reader::new(&facts).storage_type(ty).is_err());
    Evaluator::new(&report, &mut facts)
        .storage_layout(ty, 10000)
        .map_err(|e| e.message().to_owned())?;
    assert_eq!(Reader::new(&facts).storage_type(ty)?, head);
    let other = facts
        .types
        .intern_regions(&[crate::types::RegionId::STATIC])?;
    let other = facts.types.intern(Kind::Opaque(id, other))?;
    assert!(Reader::new(&facts).storage_type(other).is_err());
    assert!(facts.types.publish_storage_head(other, ty).is_err());
    assert!(
        facts
            .types
            .publish_storage_head(other, TypeId(u32::MAX))
            .is_err()
    );
    assert!(facts.types.publish_storage_head(ty, TypeId::UNIT).is_err());
    assert!(matches!(facts.types.kind(ty), Some(Kind::Opaque(..))));
    assert!(
        !facts
            .types
            .opaque_declaration(id)
            .ok_or("declaration")?
            .copies
            .copy
    );
    Ok(())
}

#[test]
fn opaque_storage_layout_comparison_checks_flags_and_full_shape() {
    let layout = Layout {
        size: 8,
        align: 4,
        inhabited: true,
        contains_explicit_align: false,
        contains_reference: false,
        shape: Shape::Array {
            element: TypeId::UNIT,
            stride: 4,
            length: 2,
        },
    };
    assert!(same_layout(&layout, &layout));
    let mut other = layout;
    other.size += 1;
    assert!(!same_layout(&layout, &other));
    let mut other = layout;
    other.contains_reference = true;
    assert!(!same_layout(&layout, &other));
    let mut other = layout;
    other.inhabited = false;
    assert!(!same_layout(&layout, &other));
    let mut other = layout;
    other.shape = Shape::Array {
        element: TypeId::UNIT,
        stride: 8,
        length: 1,
    };
    assert!(!same_layout(&layout, &other));
    let mut other = layout;
    other.shape = Shape::Scalar;
    assert!(!same_layout(&layout, &other));
}

#[test]
fn opaque_product_inspection_uses_only_its_exact_checked_head() -> Result<(), String> {
    let report = test_source::resolve("", true)?;
    let mut facts = crate::sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let fields = facts
        .types
        .intern_list(&[TypeId::primitive(crate::types::Primitive::U8)])?;
    let head = facts.types.intern(Kind::Tuple(fields))?;
    let id = facts.types.declare_opaque(opaque::Declaration {
        trait_result: false,
        sized: true,
        copies: copy::Properties::default(),
    })?;
    let ty = facts.types.intern(Kind::Opaque(id, GenericArgs::EMPTY))?;
    facts.types.publish_storage_head(ty, head)?;
    let mut evaluator = Evaluator::new(&report, &mut facts);
    evaluator
        .storage_layout(ty, 10000)
        .map_err(|e| e.message().to_owned())?;
    let value = evaluator
        .fields(
            &[super::super::Constant::scalar(
                crate::sema::model::Scalar::Integer(7),
            )],
            None,
        )
        .map_err(|e| e.message().to_owned())?;
    assert_eq!(
        Reader::new(&facts).field_types(ty, value)?.get(0),
        Some(TypeId::primitive(crate::types::Primitive::U8))
    );
    assert!(matches!(facts.types.kind(ty), Some(Kind::Opaque(..))));
    Ok(())
}
