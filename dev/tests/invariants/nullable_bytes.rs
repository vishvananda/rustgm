//! Fresh nullable reads preserve initialization, provenance and selected referent validity.
use super::*;
use crate::{
    const_eval::{Limits, ReadOperation, memory::Read, static_reads::ReadPurpose},
    sema::{
        model::Scalar,
        values::{Constant, Evaluator, View},
    },
    types::{Kind, Primitive},
};
fn with_nullable(
    check: impl FnOnce(&mut Machine<'_, '_>, TypeId, VariantId, VariantId) -> Result<(), Failure>,
) -> Result<(), String> {
    let report = crate::test_source::resolve(
        "fn pass(v:Option<&'static u32>)->Option<&'static u32>{v}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let owner = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == "pass")
        .ok_or("function")?;
    let item = facts.definitions[owner].ok_or("type")?;
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(item) else {
        return Err("item".into());
    };
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
        return Err("signature".into());
    };
    let ty = *facts
        .types
        .list(parameters)
        .and_then(|p| p.first())
        .ok_or("parameter")?;
    let mut machine = Machine::new(
        Evaluator::new(&report, &mut facts),
        std::iter::empty(),
        Limits::default(),
    )?;
    machine.footprint(ty).map_err(|e| e.message().to_owned())?;
    let niche = Reader::new(machine.evaluator.facts()).enum_niche(ty)?;
    let (empty, payload) = (niche.empty, niche.payload);
    check(&mut machine, ty, empty, payload).map_err(|e| e.message().to_owned())
}
fn some_root(
    machine: &mut Machine<'_, '_>,
    ty: TypeId,
    payload: VariantId,
) -> Result<(Constant, Constant, super::super::Id), Failure> {
    let referent = machine.allocate(
        &mut Vec::new(),
        Some(Constant::scalar(Scalar::Integer(37))),
        TypeId::primitive(Primitive::U32),
    )?;
    let value = machine
        .evaluator
        .fields(&[referent.restrict_shared()], Some(payload))?;
    let root = machine.allocate(&mut Vec::new(), Some(value), ty)?;
    let id = machine.convert_byte_root(root.location().ok_or("root")?)?;
    Ok((referent, root, id))
}
#[test]
fn nullable_symbolic_zero_bytes_select_some_and_none_creates_no_reference() -> Result<(), String> {
    with_nullable(|machine, ty, empty, payload| {
        let (_, root, id) = some_root(machine, ty, payload)?;
        assert!(machine.byte_backing(id)?.bytes.iter().all(|&b| b == 0));
        assert_eq!(
            machine.decode_nullable_variant(Source::Backing(id), 0, ty)?,
            payload
        );
        machine.read_operation = Some(ReadOperation {
            frame: 1,
            position: 0,
        });
        let Read::Value(value) = machine.decode_bytes(id, 0, ty, ReadPurpose::Value)? else {
            return Err("pending".into());
        };
        assert!(
            matches!(Reader::new(machine.evaluator.facts()).view(value)?, View::Product { variant: Some(v), length: 1 } if v == payload)
        );
        machine.fill_byte_root(root.location().ok_or("root")?, 0, 8, 0)?;
        assert_eq!(
            machine.decode_nullable_variant(Source::Backing(id), 0, ty)?,
            empty
        );
        let Read::Value(value) = machine.decode_bytes(id, 0, ty, ReadPurpose::Value)? else {
            return Err("pending".into());
        };
        assert!(
            matches!(Reader::new(machine.evaluator.facts()).view(value)?, View::Product { variant: Some(v), length: 0 } if v == empty)
        );
        assert!(machine.memory.bytes.enumerations.is_none());
        Ok(())
    })
}
#[test]
fn nullable_warm_reads_recheck_mask_partial_words_bounds_and_expiry() -> Result<(), String> {
    with_nullable(|machine, ty, _, payload| {
        let (referent, _, id) = some_root(machine, ty, payload)?;
        machine.read_operation = Some(ReadOperation {
            frame: 1,
            position: 0,
        });
        assert!(matches!(
            machine.decode_bytes(id, 0, ty, ReadPurpose::Value)?,
            Read::Value(_)
        ));
        assert!(
            machine
                .decode_nullable_variant(Source::Backing(id), 1, ty)
                .is_err()
        );
        machine.memory.bytes.roots[id.0 as usize].initialized[0] &= !1;
        assert!(machine.decode_bytes(id, 0, ty, ReadPurpose::Value).is_err());
        machine.memory.bytes.roots[id.0 as usize].initialized[0] |= 1;
        machine
            .memory
            .expire(&[referent.location().ok_or("referent")?]);
        assert!(machine.decode_bytes(id, 0, ty, ReadPurpose::Value).is_err());
        Ok(())
    })?;
    with_nullable(|machine, ty, _, payload| {
        let (_, root, id) = some_root(machine, ty, payload)?;
        machine.read_operation = Some(ReadOperation {
            frame: 1,
            position: 0,
        });
        machine.fill_byte_root(root.location().ok_or("root")?, 0, 1, 0)?;
        assert_eq!(
            machine.decode_nullable_variant(Source::Backing(id), 0, ty)?,
            payload
        );
        assert!(machine.decode_bytes(id, 0, ty, ReadPurpose::Value).is_err());
        Ok(())
    })
}
#[test]
fn nullable_encoding_rejects_wrong_variant_counts_and_retains_physical_field_address()
-> Result<(), String> {
    use crate::{
        const_eval::memory::Resolved,
        sema::{
            members::{FieldId, Selection},
            values::{Origin, WritePermission},
        },
    };
    with_nullable(|machine, ty, empty, payload| {
        let (_, root, id) = some_root(machine, ty, payload)?;
        let stored = Reader::new(machine.evaluator.facts()).enum_niche(ty)?;
        let target = stored.encoding.ty;
        let field = Selection::Nominal(FieldId(
            machine.evaluator.facts().members.variants[payload.0 as usize].start,
        ));
        let selected = machine.field_projection(
            Resolved {
                address: root,
                ty,
                raw: None,
                union: None,
            },
            field,
            target,
        )?;
        let restricted = machine.field_projection(
            Resolved {
                address: root.restrict_shared(),
                ty,
                raw: None,
                union: None,
            },
            field,
            target,
        )?;
        assert!(matches!(
            Reader::new(machine.evaluator.facts())
                .location(selected.location().ok_or("field")?)?
                .origin,
            Origin::ByteOffset { .. }
        ));
        assert_eq!(
            restricted.write_permission(),
            Some(WritePermission::SharedDerived)
        );
        let mut backing = machine.empty_backing(ty)?;
        let wrong = machine.evaluator.fields(&[], Some(payload))?;
        assert!(machine.encode_bytes(&mut backing, 0, ty, wrong).is_err());
        let wrong = machine.evaluator.fields(&[root], Some(empty))?;
        assert!(machine.encode_bytes(&mut backing, 0, ty, wrong).is_err());
        machine.fill_byte_root(root.location().ok_or("root")?, 0, 8, 0)?;
        assert_eq!(
            machine.decode_nullable_variant(Source::Backing(id), 0, ty)?,
            empty
        );
        let writable = machine.evaluator.word_address(selected, 1000)?.0;
        let shared = machine.evaluator.word_address(restricted, 1000)?.0;
        assert_eq!(
            (writable.root(), writable.offset()),
            (shared.root(), shared.offset())
        );
        Ok(())
    })
}
