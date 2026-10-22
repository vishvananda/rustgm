//! Default Rust byte storage follows its retained direct tag, not rustc niches.
use super::{payload::payload_root, *};
use crate::{
    const_eval::{memory::Read, static_reads::ReadPurpose},
    types::Primitive,
};
#[path = "replay.rs"]
mod replay;

#[test]
fn default_enum_tag_lookup_and_field_work_are_additive_and_ignore_unrelated_types()
-> Result<(), String> {
    c::warm_tag_work("", 0, 8)?;
    payload::repeated_field_work("")
}

#[test]
fn default_enum_tag_cache_never_treats_provenance_as_numeric_discriminants() -> Result<(), String> {
    wide::tag_provenance("")
}

#[test]
fn default_enum_signed_tags_and_warm_initialization_remain_exact() -> Result<(), String> {
    with_enum_source("enum E{A=-3,B=7}", 0, |machine, enums| {
        let (ty, ids) = &enums[0];
        let reader = Reader::new(machine.evaluator.facts());
        assert_eq!(reader.enum_variants(*ty)?.tag.ty, Primitive::I64);
        assert_eq!(
            reader.enum_variant_bits(*ty, ids[0])?,
            u128::from(u64::MAX) - 2
        );
        let (address, id) = root(machine, *ty, ids[0])?;
        assert!(
            machine
                .decode_byte_enum(Source::Backing(id), 0, *ty)
                .is_ok()
        );
        machine.memory.bytes.roots[id.0 as usize].initialized[0] = 1;
        assert!(
            machine
                .decode_byte_enum(Source::Backing(id), 0, *ty)
                .is_err()
        );
        machine.fill_byte_root(address, 0, 8, 255)?;
        assert!(
            machine
                .decode_byte_enum(Source::Backing(id), 0, *ty)
                .is_err()
        );
        machine.fill_byte_root(address, 0, 8, 0)?;
        machine.fill_byte_root(address, 0, 1, 7)?;
        assert!(
            machine
                .decode_byte_enum(Source::Backing(id), 0, *ty)
                .is_ok()
        );
        assert!(
            machine
                .decode_byte_enum(Source::Backing(id), 1, *ty)
                .is_err()
        );
        Ok(())
    })
}

#[test]
fn default_enum_selected_reference_duties_remain_fresh_on_cached_families() -> Result<(), String> {
    with_enum_source("enum E{A(&'static u32),Empty}", 0, |machine, enums| {
        let (ty, ids) = &enums[0];
        let referent = machine.allocate(
            &mut Vec::new(),
            Some(payload::integer(37)),
            TypeId::primitive(Primitive::U32),
        )?;
        let (_, id) = payload_root(machine, *ty, ids[0], &[referent.restrict_shared()])?;
        machine.read_operation = Some(crate::const_eval::ReadOperation {
            frame: 1,
            position: 0,
        });
        assert!(matches!(
            machine.decode_bytes(id, 0, *ty, ReadPurpose::Value)?,
            Read::Value(_)
        ));
        let before = machine.memory.bytes.checked_duties;
        machine
            .memory
            .expire(&[referent.location().ok_or("referent")?]);
        assert!(
            machine
                .decode_bytes(id, 0, *ty, ReadPurpose::Value)
                .is_err()
        );
        assert_eq!(machine.memory.bytes.checked_duties, before + 1);
        Ok(())
    })
}

#[test]
fn default_enum_index_exhaustion_is_retryable_without_publishing_partial_facts()
-> Result<(), String> {
    with_enum_source("enum E{A=3,B=7}", 0, |machine, enums| {
        let ty = enums[0].0;
        machine.footprint(ty)?;
        machine.limits.slots = 1;
        assert!(machine.prepare_byte_enum(ty).is_err());
        assert!(machine.memory.bytes.enumerations.is_none());
        machine.limits.slots = 1 << 20;
        machine.limits.steps = machine.work.steps + 1;
        assert!(machine.prepare_byte_enum(ty).is_err());
        assert!(machine.memory.bytes.enumerations.is_none());
        machine.limits.steps = 1 << 20;
        assert!(machine.prepare_byte_enum(ty)?);
        Ok(())
    })
}
