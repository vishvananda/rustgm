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
    assert!(Reader::enumeration_shape(&unsupported, &layout).is_err());
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
#[test]
fn c_enum_storage_facade_rejects_non_abi_tags_and_preserves_written_repr_checks()
-> Result<(), String> {
    let report = test_source::resolve("#[repr(C)]enum E{A=3,B=7}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let owner = facts.aggregates.iter().next().ok_or("enum")?.item;
    let ty = facts
        .types
        .intern(Kind::Adt(owner, crate::types::GenericArgs::EMPTY))?;
    Evaluator::new(&report, &mut facts)
        .storage_layout(ty, 10000)
        .map_err(|e| e.message().to_owned())?;
    let original = *facts.layouts.get(ty).ok_or("layout")?;
    let definition = *facts.aggregates.get(owner).ok_or("definition")?;
    assert!(matches!(
        Reader::enumeration_shape(&definition, &original)?,
        StoredShape::Enumeration {
            tag: Tag {
                ty: Primitive::U32,
                ..
            }
        }
    ));
    let Shape::Aggregate { variants, .. } = original.shape else {
        return Err("shape".into());
    };
    for primitive in [
        Primitive::Bool,
        Primitive::Char,
        Primitive::I8,
        Primitive::U8,
        Primitive::I16,
        Primitive::U16,
        Primitive::I128,
        Primitive::U128,
    ] {
        let mut layout = original;
        layout.size = 32;
        layout.shape = Shape::Aggregate {
            variants,
            tag: Some(Tag {
                ty: primitive,
                offset: 0,
            }),
        };
        assert!(Reader::enumeration_shape(&definition, &layout).is_err());
    }
    let mut written = definition;
    written.repr.tag = Some(Primitive::U8);
    assert!(Reader::enumeration_shape(&written, &original).is_err());
    written.repr.tag = Some(Primitive::U32);
    assert!(matches!(
        Reader::enumeration_shape(&written, &original)?,
        StoredShape::Enumeration { .. }
    ));
    let mut rust = definition;
    rust.repr.representation = Representation::Rust;
    assert!(Reader::enumeration_shape(&rust, &original).is_err());
    let mut retained = original;
    retained.size = 8;
    retained.shape = Shape::Aggregate {
        variants,
        tag: Some(Tag {
            ty: Primitive::I64,
            offset: 0,
        }),
    };
    assert!(matches!(
        Reader::enumeration_shape(&rust, &retained)?,
        StoredShape::Enumeration { .. }
    ));
    retained.size = 7;
    assert!(Reader::enumeration_shape(&rust, &retained).is_err());
    Ok(())
}
#[test]
fn tagless_enum_storage_facade_rejects_wrong_owner_count_ordinal_and_bounds() -> Result<(), String>
{
    let report = test_source::resolve("enum E{Only(u32)}enum F{Other(bool)}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let owners: Vec<_> = facts.aggregates.iter().map(|d| d.item).collect();
    let ty = facts
        .types
        .intern(Kind::Adt(owners[0], crate::types::GenericArgs::EMPTY))?;
    Evaluator::new(&report, &mut facts)
        .storage_layout(ty, 10000)
        .map_err(|e| e.message().to_owned())?;
    let id = facts.members.variants(owners[0])[0];
    let foreign = facts.members.variants(owners[1])[0];
    let original = facts.members.variants[id.0 as usize];
    let reader = Reader::new(&facts);
    assert!(
        matches!(reader.storage_shape(ty)?,StoredShape::Product{variant:Some(actual),fields,..} if actual==id&&fields==[TypeId::primitive(Primitive::U32)])
    );
    assert!(reader.enum_variants(ty).is_err());
    assert!(reader.enum_variant_fields(ty, foreign).is_err());
    assert!(reader.projection_layout(ty, 1).is_err());
    assert!(reader.projection_layout(ty, u64::MAX).is_err());
    facts.members.variants[id.0 as usize].owner = owners[1];
    assert!(Reader::new(&facts).storage_shape(ty).is_err());
    facts.members.variants[id.0 as usize] = original;
    facts.members.variants[id.0 as usize].ordinal = 1;
    assert!(Reader::new(&facts).storage_shape(ty).is_err());
    facts.members.variants[id.0 as usize] = original;
    facts.members.variants[id.0 as usize].end += 1;
    assert!(Reader::new(&facts).storage_shape(ty).is_err());
    facts.members.variants[id.0 as usize] = original;
    facts.members = crate::sema::members::Members::default();
    assert!(Reader::new(&facts).storage_shape(ty).is_err());
    Ok(())
}
#[test]
fn tagless_enum_field_types_follow_complete_closed_generic_identity() -> Result<(), String> {
    let report =
        test_source::resolve("enum E<T>{Only(T)}fn use_types(_:E<u32>,_:E<bool>){}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let types: Vec<_> = (0..facts.types.len())
        .filter_map(|n| u32::try_from(n).ok().map(TypeId))
        .filter(|&ty| {
            matches!(facts.types.kind(ty), Some(Kind::Adt(..)))
                && facts.types.contains_parameters(ty) == Some(false)
        })
        .collect();
    assert_eq!(types.len(), 2);
    let mut fields = Vec::new();
    for ty in types {
        Evaluator::new(&report, &mut facts)
            .storage_layout(ty, 10000)
            .map_err(|e| e.message().to_owned())?;
        fields.push(Reader::new(&facts).projection_layout(ty, 0)?.ty);
    }
    fields.sort_by_key(|ty| ty.0);
    let mut expected = [
        TypeId::primitive(Primitive::U32),
        TypeId::primitive(Primitive::Bool),
    ];
    expected.sort_by_key(|ty| ty.0);
    assert_eq!(fields, expected);
    Ok(())
}
