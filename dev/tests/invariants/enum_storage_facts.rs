//! Retained enum layouts must agree with exact declaration identities.
use super::*;
use crate::{sema::values::Evaluator, test_source};
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
    for tag in [None, Some(Primitive::U16), Some(Primitive::I32)] {
        unsupported.repr.tag = tag;
        assert!(matches!(
            Reader::enumeration_shape(&unsupported, &layout)?,
            StoredShape::Unsupported
        ));
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
