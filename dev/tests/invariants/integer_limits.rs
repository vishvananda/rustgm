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
    )?;
    assert!(
        uses.publish(
            &mut facts.types,
            key,
            record.prefix,
            TypeId::primitive(Primitive::F32)
        )
        .is_err()
    );
    assert!(
        uses.publish(
            &mut facts.types,
            key,
            record.prefix,
            TypeId::primitive(Primitive::U16)
        )
        .is_err()
    );
    assert!(
        uses.publish(&mut facts.types, key, record.prefix, TypeId::UNIT)
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
    assert_eq!(std::mem::size_of::<Descriptor>(), 1);
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
