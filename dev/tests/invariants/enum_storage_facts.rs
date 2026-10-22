//! Retained enum layouts must agree with exact declaration identities.
use super::*;
use crate::{sema::values::Evaluator, test_source, types::Primitive};
#[test]
fn enum_storage_facade_rejects_malformed_tags_bounds_counts_and_owners() -> Result<(), String> {
    let report = test_source::resolve("#[repr(u8)]enum E{A=1,B=7}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let item = facts.aggregates.iter().next().ok_or("enum")?.item;
    let ty = facts
        .types
        .intern(Kind::Adt(item, crate::types::GenericArgs::EMPTY))?;
    Evaluator::new(&report, &mut facts)
        .storage_layout(ty, 10000)
        .map_err(|e| e.message().to_owned())?;
    let layout = *facts.layouts.get(ty).ok_or("layout")?;
    let definition = *facts.aggregates.get(item).ok_or("definition")?;
    assert!(matches!(
        Reader::enumeration_shape(&definition, &layout)?,
        StoredShape::Enumeration { .. }
    ));
    let Shape::Aggregate { variants, .. } = layout.shape else {
        return Err("enum shape".into());
    };
    for (tag, size) in [
        (None, 1),
        (
            Some(Tag {
                ty: Primitive::I8,
                offset: 0,
            }),
            1,
        ),
        (
            Some(Tag {
                ty: Primitive::U8,
                offset: 1,
            }),
            1,
        ),
        (
            Some(Tag {
                ty: Primitive::U8,
                offset: u64::MAX,
            }),
            8,
        ),
    ] {
        let mut wrong = layout;
        wrong.shape = Shape::Aggregate { variants, tag };
        wrong.size = size;
        assert!(Reader::enumeration_shape(&definition, &wrong).is_err());
    }
    let mut unsupported = definition;
    unsupported.repr.tag = None;
    assert!(matches!(
        Reader::enumeration_shape(&unsupported, &layout)?,
        StoredShape::Unsupported
    ));
    for tag in [
        Primitive::Bool,
        Primitive::Char,
        Primitive::U16,
        Primitive::I32,
    ] {
        unsupported.repr.tag = Some(tag);
        assert!(Reader::enumeration_shape(&unsupported, &layout).is_err());
    }
    let first = facts.members.variants(item)[0];
    facts.members.variants[first.0 as usize].owner = ItemId(u32::MAX);
    assert!(Reader::new(&facts).enum_variant_bits(ty, first).is_err());
    facts.members.variants[first.0 as usize].owner = item;
    facts.members.variants[first.0 as usize].ordinal = 1;
    assert!(Reader::new(&facts).enum_variant_bits(ty, first).is_err());
    facts.members = crate::sema::members::Members::default();
    assert!(Reader::new(&facts).enum_variants(ty).is_err());
    Ok(())
}
#[test]
fn payload_enum_facade_keeps_exact_fields_and_rejects_malformed_declaration_ranges()
-> Result<(), String> {
    let report = test_source::resolve(
        "#[repr(u8)]enum E{A(u32),B(bool)}#[repr(C,u8)]enum F{A(u32),B(bool)}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let owners: Vec<_> = facts.aggregates.iter().map(|d| d.item).collect();
    let mut types = Vec::new();
    for owner in owners {
        let ty = facts
            .types
            .intern(Kind::Adt(owner, crate::types::GenericArgs::EMPTY))?;
        Evaluator::new(&report, &mut facts)
            .storage_layout(ty, 10000)
            .map_err(|e| e.message().to_owned())?;
        types.push((owner, ty));
    }
    let mut offsets = Vec::new();
    for &(owner, ty) in &types {
        let ids = facts.members.variants(owner);
        let first = ids[0];
        let second = ids[1];
        let a = crate::sema::members::FieldId(facts.members.variants[first.0 as usize].start);
        let b = crate::sema::members::FieldId(facts.members.variants[second.0 as usize].start);
        assert_eq!(Reader::new(&facts).enum_field_layout(ty, a)?.offset, 4);
        offsets.push(Reader::new(&facts).enum_field_layout(ty, b)?.offset);
        assert!(
            Reader::new(&facts)
                .enum_field_layout(ty, crate::sema::members::FieldId(u32::MAX))
                .is_err()
        );
    }
    offsets.sort_unstable();
    assert_eq!(offsets, [1, 4]);
    let (owner, ty) = types[0];
    let foreign = facts.members.variants(types[1].0)[0];
    let foreign_field =
        crate::sema::members::FieldId(facts.members.variants[foreign.0 as usize].start);
    assert!(
        Reader::new(&facts)
            .enum_field_layout(ty, foreign_field)
            .is_err()
    );
    let id = facts.members.variants(owner)[0];
    let original = facts.members.variants[id.0 as usize];
    facts.members.variants[id.0 as usize].end = original.end + 1;
    assert!(Reader::new(&facts).enum_variant_fields(ty, id).is_err());
    facts.members.variants[id.0 as usize] = original;
    facts.members.variants[id.0 as usize].owner = types[1].0;
    assert!(Reader::new(&facts).enum_variant_fields(ty, id).is_err());
    facts.members.variants[id.0 as usize] = original;
    facts.members.variants[id.0 as usize].ordinal = 1;
    assert!(Reader::new(&facts).enum_variant_fields(ty, id).is_err());
    Ok(())
}
