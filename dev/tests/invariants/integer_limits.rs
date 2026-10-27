//! Sparse value uses validate their exact primitive prefix and canonical bits.
use super::*;
use std::fmt::Write;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn analyze(source: &str) -> Result<Declarations> {
    let report = crate::test_source::resolve(source, true)?;
    Ok(crate::pipeline::analyze_source(&report)?.facts)
}
fn fixture() -> Result<(Declarations, NodeKey, Selected)> {
    let facts = analyze("fn main(){let _=u32::MAX;}")?;
    let (&key, &record) = facts
        .primitive_constants
        .as_deref()
        .ok_or("uses")?
        .selected
        .iter()
        .next()
        .ok_or("selection")?;
    Ok((facts, key, record))
}
#[test]
fn constant_use_requires_its_exact_prefix_and_value() -> Result {
    let (mut facts, key, record) = fixture()?;
    assert_eq!(selected(&facts, key)?, Some(record.value));
    let wrong = facts.types.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::U32),
        bits: 7,
    })?;
    facts
        .primitive_constants
        .as_deref_mut()
        .ok_or("uses")?
        .selected
        .get_mut(&key)
        .ok_or("selection")?
        .value = wrong;
    assert!(selected(&facts, key).is_err());
    facts
        .primitive_constants
        .as_deref_mut()
        .ok_or("uses")?
        .selected
        .insert(key, record);
    facts.node_mut(record.prefix).ok_or("prefix")?.ty = Some(TypeId::primitive(Primitive::U16));
    assert!(selected(&facts, key).is_err());
    facts.node_mut(record.prefix).ok_or("prefix")?.ty = None;
    assert!(selected(&facts, key).is_err());
    Ok(())
}
#[test]
fn publication_rejects_noninteger_and_conflicting_identity() -> Result {
    let (mut facts, key, record) = fixture()?;
    let uses = facts.primitive_constants.as_deref_mut().ok_or("uses")?;
    let count = uses.selected.len();
    let before = facts.types.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::U8),
        bits: 3,
    })?;
    uses.publish(
        &mut facts.types,
        key,
        record.prefix,
        TypeId::primitive(Primitive::U32),
        Member::Max,
    )?;
    assert!(
        uses.publish(
            &mut facts.types,
            key,
            record.prefix,
            TypeId::primitive(Primitive::F32),
            Member::Max
        )
        .is_err()
    );
    assert!(
        uses.publish(
            &mut facts.types,
            key,
            record.prefix,
            TypeId::primitive(Primitive::U16),
            Member::Max
        )
        .is_err()
    );
    assert!(
        uses.publish(
            &mut facts.types,
            key,
            record.prefix,
            TypeId::UNIT,
            Member::Max
        )
        .is_err()
    );
    let after = facts.types.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::U8),
        bits: 4,
    })?;
    assert_eq!(after.0, before.0 + 1);
    assert_eq!(uses.selected.len(), count);
    assert!(
        uses.selected
            .get(&key)
            .is_some_and(|saved| *saved == record)
    );
    assert_eq!(std::mem::size_of::<Descriptor>(), 2);
    assert_eq!(std::mem::size_of::<Selected>(), 16);
    assert_eq!(std::mem::size_of::<Option<Box<Uses>>>(), 8);
    assert_eq!(std::mem::size_of::<crate::sema::model::NodeFacts>(), 80);
    println!(
        "MAX layout: Declarations={} Uses={} Selected={} map-payload={}",
        std::mem::size_of::<Declarations>(),
        std::mem::size_of::<Uses>(),
        std::mem::size_of::<Selected>(),
        std::mem::size_of::<(NodeKey, Selected)>()
    );
    Ok(())
}
#[test]
fn ordinary_sources_allocate_no_constant_uses() -> Result {
    for source in [
        "fn main(){let _=1u8+2;}",
        "trait K{const MAX:u8;}struct S;impl K for S{const MAX:u8=7;}fn main(){let _=S::MAX;}",
    ] {
        let facts = analyze(source)?;
        assert!(facts.primitive_constants.is_none());
    }
    Ok(())
}
#[test]
fn repeated_uses_are_linear_and_do_not_build_callables() -> Result {
    for width in [8, 32, 128] {
        let mut source = String::from("fn main(){");
        for _ in 0..width {
            write!(source, "let _=u32::MAX;")?;
        }
        source.push('}');
        let facts = analyze(&source)?;
        let uses = facts.primitive_constants.as_deref().ok_or("uses")?;
        assert_eq!(uses.selected.len(), width);
        assert!(facts.primitive_methods.is_none());
        assert!(uses.selected.values().all(|record| {
            record.value
                == uses
                    .selected
                    .values()
                    .next()
                    .map_or(record.value, |record| record.value)
        }));
        assert!(facts.body_work.inference_steps <= 100 * width + 100);
        println!(
            "MAX uses width={width} selections={} inference={} constants={} definitions={}",
            uses.selected.len(),
            facts.body_work.inference_steps,
            uses.selected
                .values()
                .map(|record| record.value.0)
                .max()
                .map_or(0, |id| id + 1),
            facts.definitions.len()
        );
    }
    Ok(())
}

#[test]
fn char_members_retain_exact_typed_values_and_existing_scalar_validity() -> Result {
    let mut facts = analyze("fn main(){let _=char::MIN;let _=char::MAX;}")?;
    let uses = facts.primitive_constants.as_deref().ok_or("char uses")?;
    assert_eq!(uses.selected.len(), 2);
    let records: Vec<_> = uses
        .selected
        .iter()
        .map(|(&key, &record)| (key, record))
        .collect();
    for (key, record) in records {
        let constant = facts.types.constant(record.value).ok_or("char constant")?;
        assert_eq!(constant.ty, TypeId::primitive(Primitive::Char));
        assert_eq!(
            constant.bits,
            if record.descriptor.member == Member::Min {
                0
            } else {
                0x10_FFFF
            }
        );
        assert_eq!(selected(&facts, key)?, Some(record.value));
        let opposite = if record.descriptor.member == Member::Min {
            Member::Max
        } else {
            Member::Min
        };
        let uses = facts
            .primitive_constants
            .as_deref_mut()
            .ok_or("char uses")?;
        assert!(
            uses.publish(&mut facts.types, key, record.prefix, constant.ty, opposite)
                .is_err()
        );
        assert!(uses.selected.get(&key).copied() == Some(record));
    }
    for bits in [0xD800, 0xDFFF, 0x11_0000, u128::MAX] {
        assert!(
            facts
                .types
                .intern_constant(Constant {
                    ty: TypeId::primitive(Primitive::Char),
                    bits
                })
                .is_err()
        );
    }
    assert!(
        Descriptor {
            primitive: Primitive::U32,
            member: Member::Min
        }
        .value()
        .is_err()
    );
    let char_ty = TypeId::primitive(Primitive::Char);
    let array = facts.types.intern(Kind::Array {
        element: char_ty,
        length: 3,
    })?;
    for (ty, size) in [(char_ty, 4), (array, 12)] {
        facts.layouts.demand(&facts.types, &facts.aggregates, ty)?;
        let layout = facts.layouts.get(ty).ok_or("char layout")?;
        assert_eq!((layout.size, layout.align), (size, 4));
    }
    println!(
        "char-layout Descriptor={} Selected={} Uses={} NodeFacts={} Body={} Char=4/4 Array3=12/4",
        size_of::<Descriptor>(),
        size_of::<Selected>(),
        size_of::<Uses>(),
        size_of::<crate::sema::model::NodeFacts>(),
        size_of::<crate::body::ir::Body>()
    );
    Ok(())
}

#[test]
fn actual_source_char_constants_never_use_bootstrap_values() -> Result {
    use crate::resolve::model::DefKind;
    for name in ["source-absent-max-v2", "source-absent-min-v2"] {
        let source =
            crate::test_source::fixture(&format!("pa31/tests/spec/char-constants/{name}.rs"))?;
        let report = crate::test_source::source_provider(&source, "char_constant_owner")?;
        assert!(crate::pipeline::analyze_source(&report).is_err(), "{name}");
    }
    for (name, high, low) in [
        ("source-actual-v2", '\u{10FFFF}', '\0'),
        ("source-owned-value-v2", 'z', 'a'),
    ] {
        let source =
            crate::test_source::fixture(&format!("pa31/tests/spec/char-constants/{name}.rs"))?;
        let report = crate::test_source::source_provider(&source, "char_constant_owner")?;
        let facts = crate::pipeline::analyze_source(&report)?.facts;
        assert!(facts.primitive_constants.is_none(), "{name}");
        assert!(facts.source_bindings.is_some());
        let reader = crate::sema::values::Reader::new(&facts);
        for (label, expected) in [("VALUE", high), ("LOW", low)] {
            let id = report
                .resolution
                .definitions
                .iter()
                .enumerate()
                .find_map(|(i, d)| {
                    (d.kind == DefKind::Const && report.package.names.get(d.name) == label)
                        .then_some(i)
                })
                .ok_or("source char definition")?;
            let value = reader
                .definition(crate::resolve::model::DefId(u32::try_from(id)?))
                .ok_or("source char value")?;
            assert!(
                matches!(reader.view(value)?,crate::sema::values::View::Scalar(crate::sema::model::Scalar::Character(actual)) if actual==expected),
                "{name}: {label}"
            );
        }
    }
    Ok(())
}

#[test]
fn char_constants_share_canonical_values_and_bound_sparse_use_work() -> Result {
    for width in [1, 8, 32] {
        let mut source = String::from("fn main(){");
        for _ in 0..width {
            source.push_str("let _=char::MIN;let _=char::MAX;");
        }
        source.push('}');
        let facts = analyze(&source)?;
        let uses = facts.primitive_constants.as_deref().ok_or("char uses")?;
        assert_eq!(uses.selected.len(), 2 * width);
        assert!(facts.primitive_methods.is_none());
        let mut values = uses
            .selected
            .values()
            .map(|record| record.value.0)
            .collect::<Vec<_>>();
        values.sort_unstable();
        values.dedup();
        assert_eq!(values.len(), 2);
        assert!(facts.body_work.inference_steps <= 200 * width + 100);
        println!(
            "char-uses pairs={width} selections={} canonical-values={} capacity={} capacity-payload-bytes={} inference={}",
            uses.selected.len(),
            values.len(),
            uses.selected.capacity(),
            uses.selected.capacity() * size_of::<(NodeKey, Selected)>(),
            facts.body_work.inference_steps
        );
    }
    Ok(())
}
