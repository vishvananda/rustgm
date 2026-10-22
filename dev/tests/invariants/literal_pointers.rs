//! Literal pointer patterns require the exact shared byte-slice view.
use super::*;
use crate::types::RegionId;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture() -> Result<Declarations> {
    let report = crate::test_source::resolve("fn main(){}", true)?;
    Ok(crate::pipeline::analyze_source(&report)?.facts)
}
fn reference(facts: &mut Declarations, mutable: bool, element: Primitive) -> Result<TypeId> {
    let referent = facts
        .types
        .intern(Type::Slice(TypeId::primitive(element)))?;
    Ok(facts.types.intern(Type::Reference {
        region: RegionId::STATIC,
        mutable,
        referent,
    })?)
}
fn expansion(facts: &mut Declarations) -> Expansion<'_> {
    let source = facts.functions[0].node;
    Expansion {
        facts,
        source,
        work: Vec::new(),
        products: HashMap::new(),
        references: HashMap::new(),
        texts: HashMap::new(),
        remaining: 1_000_000,
    }
}
#[test]
fn only_complete_shared_byte_slice_literals_normalize() -> Result {
    use super::super::references::Metadata;
    let mut facts = fixture()?;
    let shared = reference(&mut facts, false, Primitive::U8)?;
    let mutable = reference(&mut facts, true, Primitive::U8)?;
    let other = reference(&mut facts, false, Primitive::U16)?;
    let slice = facts
        .types
        .intern(Type::Slice(TypeId::primitive(Primitive::U8)))?;
    let raw = facts.types.intern(Type::Pointer {
        mutable: false,
        pointee: slice,
    })?;
    for bytes in [b"".as_slice(), b"abc".as_slice()] {
        let text = facts.texts.intern(bytes)?;
        let value = Value::BytePointer {
            text,
            metadata: Metadata::Length(bytes.len() as u64),
        };
        let mut expansion = expansion(&mut facts);
        assert!(expansion.literal_pointer(shared, value)? == Value::Bytes(text));
        for ty in [mutable, other, raw, slice] {
            assert!(expansion.literal_pointer(ty, value)? == value);
            expansion.select(ty, value)?;
            assert!(expansion.drain().is_err());
        }
        let mut metadata = vec![
            Metadata::Length(bytes.len() as u64 + 1),
            Metadata::Object {
                concrete: shared,
                target: shared,
            },
        ];
        if !bytes.is_empty() {
            metadata.push(Metadata::Length(bytes.len() as u64 - 1));
        }
        for metadata in metadata {
            let value = Value::BytePointer { text, metadata };
            assert!(expansion.literal_pointer(shared, value)? == value);
            expansion.select(shared, value)?;
            assert!(expansion.drain().is_err());
        }
    }
    Ok(())
}
#[test]
fn repeated_literal_pointers_share_canonical_text_expansion() -> Result {
    for width in [8, 32, 128] {
        let mut facts = fixture()?;
        let ty = reference(&mut facts, false, Primitive::U8)?;
        let text = facts.texts.intern(b"abc")?;
        let value = Value::BytePointer {
            text,
            metadata: super::super::references::Metadata::Length(3),
        };
        let mut expansion = expansion(&mut facts);
        let first = expansion.select(ty, value)?;
        for _ in 0..width {
            assert_eq!(expansion.select(ty, value)?, first);
            assert_eq!(expansion.select(ty, Value::Bytes(text))?, first);
        }
        assert_eq!(expansion.work.len(), 1);
        expansion.drain()?;
        assert_eq!(expansion.texts.len(), 2);
        assert_eq!(expansion.facts.patterns.fields.len(), 3);
        assert_eq!(expansion.remaining, 1_000_000 - 8);
        assert!(matches!(
            expansion.facts.patterns.nodes[first.0 as usize].kind,
            Kind::Dereference { .. }
        ));
    }
    Ok(())
}
