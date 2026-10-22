//! Physical opaque storage keeps source identity and validates current bytes.
use super::*;
use crate::{
    const_eval::{memory::Read, static_reads::ReadPurpose},
    types::{GenericArgs, copy, opaque},
};

fn opaque(types: &mut crate::types::Store, head: TypeId) -> Result<TypeId, String> {
    let id = types.declare_opaque(opaque::Declaration {
        trait_result: false,
        sized: true,
        copies: copy::Properties::default(),
    })?;
    let ty = types.intern(Kind::Opaque(id, GenericArgs::EMPTY))?;
    types.publish_storage_head(ty, head)?;
    Ok(ty)
}
fn decoded(
    machine: &mut Machine<'_, '_>,
    id: super::super::Id,
    ty: TypeId,
) -> Result<Constant, Failure> {
    match machine.decode_bytes(id, 0, ty, ReadPurpose::Value)? {
        Read::Value(value) => Ok(value),
        Read::Pending => Err("private opaque read unexpectedly pending".into()),
    }
}
#[test]
fn opaque_byte_codec_rechecks_bool_initialization_after_warm_family() -> Result<(), String> {
    with_storage(
        1 << 20,
        |t| opaque(t, TypeId::primitive(Primitive::Bool)),
        |machine, ty| {
            let mut allocations = Vec::new();
            let value = Constant::scalar(Scalar::Boolean(true));
            let root = machine
                .allocate(&mut allocations, Some(value), ty)?
                .location()
                .ok_or("root")?;
            let id = machine.convert_byte_root(root)?;
            assert!(decoded(machine, id, ty)? == value);
            machine.commit_byte_snapshot(
                root,
                0,
                &Snapshot {
                    bytes: vec![2],
                    initialized: vec![1],
                    fragments: None,
                },
            )?;
            assert!(decoded(machine, id, ty).is_err());
            machine.commit_byte_snapshot(
                root,
                0,
                &Snapshot {
                    bytes: vec![1],
                    initialized: vec![0],
                    fragments: None,
                },
            )?;
            assert!(decoded(machine, id, ty).is_err());
            assert_eq!(
                Reader::new(machine.evaluator.facts()).location(root)?.ty,
                ty
            );
            Ok(())
        },
    )
}
#[test]
fn opaque_pointer_transport_preserves_restriction_and_expiry() -> Result<(), String> {
    with_storage(
        1 << 20,
        |t| {
            let head = t.intern(Kind::Pointer {
                mutable: true,
                pointee: TypeId::primitive(Primitive::U8),
            })?;
            opaque(t, head)
        },
        |machine, ty| {
            let mut allocations = Vec::new();
            let byte = TypeId::primitive(Primitive::U8);
            let target = machine.allocate(&mut allocations, Some(scalar(7)), byte)?;
            let restricted = target.restrict_shared();
            let root = machine
                .allocate(&mut allocations, Some(restricted), ty)?
                .location()
                .ok_or("root")?;
            let id = machine.convert_byte_root(root)?;
            let result = decoded(machine, id, ty)?;
            assert!(result == restricted);
            let place = crate::const_eval::memory::Resolved {
                address: result,
                ty: byte,
                union: None,
                raw: Some(crate::const_eval::memory::RawAccess {
                    alignment: crate::const_eval::memory::RawAlignment::Required,
                    location: result.location(),
                    ty: byte,
                }),
            };
            assert!(machine.read(place)?.is_some_and(|v| v == scalar(7)));
            assert!(machine.check_raw_write(place).is_err());
            machine.memory.expire(&[target.location().ok_or("target")?]);
            assert!(machine.read(place).is_err());
            Ok(())
        },
    )
}
#[test]
fn opaque_shared_head_cache_keeps_separate_value_and_application_identity() -> Result<(), String> {
    with_storage(
        1 << 20,
        |t| {
            let first = opaque(t, TypeId::primitive(Primitive::U8))?;
            let other = opaque(t, TypeId::primitive(Primitive::U16))?;
            let fields = t.intern_list(&[first, other])?;
            Ok(t.intern(Kind::Tuple(fields))?)
        },
        |machine, pair| {
            let Some(Kind::Tuple(fields)) = machine.evaluator.facts().types.kind(pair) else {
                return Err("opaque application pair".into());
            };
            let &[ty, other] = machine
                .evaluator
                .facts()
                .types
                .list(fields)
                .ok_or("pair fields")?
            else {
                return Err("opaque application count".into());
            };
            let mut allocations = Vec::new();
            for (ty, number, size) in [(ty, 11, 1), (other, 513, 2)] {
                let root = machine
                    .allocate(&mut allocations, Some(scalar(number)), ty)?
                    .location()
                    .ok_or("root")?;
                let id = machine.convert_byte_root(root)?;
                assert_eq!(machine.footprint(ty)?.size, size);
                assert!(decoded(machine, id, ty)? == scalar(number));
            }
            assert!(machine.memory.bytes.families.contains_key(&ty));
            assert!(machine.memory.bytes.families.contains_key(&other));
            Ok(())
        },
    )
}
#[test]
fn opaque_warm_byte_reads_scale_with_repetition_not_width_or_store_noise() -> Result<(), String> {
    for width in [8u64, 32, 128] {
        for noise in [0u64, 1024] {
            with_storage(
                1 << 20,
                |t| {
                    for length in 0..noise {
                        t.intern(Kind::Array {
                            element: TypeId::UNIT,
                            length,
                        })?;
                    }
                    let head = t.intern(Kind::Array {
                        element: TypeId::primitive(Primitive::U8),
                        length: width,
                    })?;
                    opaque(t, head)
                },
                |machine, ty| {
                    let mut allocations = Vec::new();
                    let value = machine.evaluator.repeat(scalar(37), width)?;
                    let root = machine
                        .allocate(&mut allocations, Some(value), ty)?
                        .location()
                        .ok_or("root")?;
                    machine.convert_byte_root(root)?;
                    let mut per_read = None;
                    for repeats in [8usize, 32, 128] {
                        let retained = machine.memory.retained_cells();
                        let before = machine.work.steps;
                        for _ in 0..repeats {
                            assert!(
                                machine
                                    .read_byte_access(
                                        root,
                                        width - 1,
                                        TypeId::primitive(Primitive::U8),
                                        ReadPurpose::Value
                                    )?
                                    .is_some_and(|v| v == scalar(37))
                            );
                        }
                        let work = machine.work.steps - before;
                        let current = work / repeats;
                        assert_eq!(work, current * repeats);
                        assert_eq!(*per_read.get_or_insert(current), current);
                        assert_eq!(retained, machine.memory.retained_cells());
                        println!(
                            "opaque width={width} noise={noise} repeats={repeats} work={work} retained={retained}"
                        );
                    }
                    Ok(())
                },
            )?;
        }
    }
    Ok(())
}

#[test]
fn opaque_shared_wide_head_validation_scales_additively_with_applications() -> Result<(), String> {
    let mut grid = Vec::new();
    for width in [8usize, 32, 128] {
        let mut row = Vec::new();
        for count in [8usize, 32, 128] {
            with_storage(
                1 << 20,
                |t| {
                    let fields = t.intern_list(&vec![TypeId::primitive(Primitive::U8); width])?;
                    let head = t.intern(Kind::Tuple(fields))?;
                    let mut applications = Vec::new();
                    for _ in 0..count {
                        applications.push(opaque(t, head)?);
                    }
                    let applications = t.intern_list(&applications)?;
                    Ok(t.intern(Kind::Tuple(applications))?)
                },
                |machine, ty| {
                    let Some(Kind::Tuple(fields)) = machine.evaluator.facts().types.kind(ty) else {
                        return Err("application list".into());
                    };
                    let applications = machine
                        .evaluator
                        .facts()
                        .types
                        .list(fields)
                        .ok_or("applications")?
                        .to_vec();
                    let before = machine.work.steps;
                    for &application in &applications {
                        assert!(machine.byte_family(application)?);
                    }
                    let work = machine.work.steps - before;
                    let retained = machine.memory.retained_cells();
                    let before = machine.work.steps;
                    for &application in &applications {
                        assert!(machine.byte_family(application)?);
                    }
                    assert_eq!(machine.work.steps - before, count);
                    assert_eq!(machine.memory.retained_cells(), retained);
                    row.push(work);
                    println!(
                        "opaque-shared width={width} applications={count} cold={work} warm={count} retained={retained}"
                    );
                    Ok(())
                },
            )?;
        }
        assert_eq!(row[2] - row[1], 4 * (row[1] - row[0]));
        grid.push(row);
    }
    assert_eq!(grid[2][2] - grid[2][1], grid[0][2] - grid[0][1]);
    Ok(())
}

#[test]
fn opaque_reference_head_checks_actual_referent_on_every_decode() -> Result<(), String> {
    let report = test_source::resolve("const fn anchor(r:&u8)->&u8{r}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let head = facts.types.intern(Kind::Reference {
        mutable: false,
        region: crate::types::RegionId::STATIC,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let ty = opaque(&mut facts.types, head)?;
    let program = crate::body::build_concrete(&report, &facts).map_err(|e| e.to_string())?;
    let body = program.bodies.first().ok_or("checked anchor body")?;
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, [body], Limits::default())?;
    let checked = (|| -> Result<(), Failure> {
        let mut allocations = Vec::new();
        let target = machine.allocate(
            &mut allocations,
            Some(scalar(7)),
            TypeId::primitive(Primitive::U8),
        )?;
        let target_root = target.location().ok_or("target")?;
        machine
            .begin(0, &[target])
            .map_err(|_| Failure::Invalid("anchor frame"))?;
        let frame = machine.frames.last().ok_or("checked anchor frame")?;
        machine.read_operation = Some(crate::const_eval::ReadOperation {
            frame: frame.instance,
            position: frame.next,
        });
        let root = machine
            .allocate(&mut allocations, Some(target), ty)?
            .location()
            .ok_or("root")?;
        let id = machine.convert_byte_root(root)?;
        assert!(decoded(&mut machine, id, ty)? == target.restrict_shared());
        machine.commit_byte_snapshot(
            target_root,
            0,
            &Snapshot {
                bytes: vec![7],
                initialized: vec![0],
                fragments: None,
            },
        )?;
        assert!(decoded(&mut machine, id, ty).is_err());
        machine.commit_byte_snapshot(
            target_root,
            0,
            &Snapshot {
                bytes: vec![7],
                initialized: vec![1],
                fragments: None,
            },
        )?;
        assert!(decoded(&mut machine, id, ty)? == target.restrict_shared());
        machine.memory.expire(&[target_root]);
        assert!(decoded(&mut machine, id, ty).is_err());
        assert_eq!(machine.memory.bytes.checked_duties, 4);
        assert_eq!(machine.memory.bytes.paid_duties, 2);
        Ok(())
    })();
    checked.map_err(|e| e.message().to_owned())
}
