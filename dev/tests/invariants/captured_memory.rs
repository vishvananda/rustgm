//! Captures retain typed identity, current bytes and the original pointer restriction.
use super::*;
use crate::{
    const_eval::{memory::Read, static_reads::ReadPurpose},
    sema::values::View,
};
use std::fmt::Write;

fn with_capture(
    source: &str,
    noise: usize,
    check: impl FnOnce(&mut Machine<'_, '_>, TypeId) -> Result<(), Failure>,
) -> Result<(), String> {
    let report = test_source::resolve(source, true)?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let ty=(0..facts.types.len()).find_map(|n|{
        let ty=TypeId(u32::try_from(n).ok()?);
        matches!(facts.types.kind(ty),Some(Kind::Closure{environment,..}) if environment != TypeId::UNIT).then_some(ty)
    }).ok_or("captured closure")?;
    for length in 0..noise {
        facts.types.intern(Kind::Array {
            element: TypeId::primitive(Primitive::U8),
            length: length as u64,
        })?;
    }
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, std::iter::empty(), Limits::default())?;
    machine.footprint(ty).map_err(|e| e.message().to_owned())?;
    check(&mut machine, ty).map_err(|e| e.message().to_owned())
}

#[test]
fn captured_zero_byte_nested_environment_still_validates_its_product() -> Result<(), String> {
    with_capture(
        "fn f(){let inner=||37u32;let outer=move||inner();let _=outer;}",
        0,
        |machine, ty| {
            assert_eq!(machine.footprint(ty)?.size, 0);
            let empty = machine.evaluator.fields(&[], None)?;
            let valid = machine.evaluator.fields(&[empty], None)?;
            let mut backing = machine.empty_backing(ty)?;
            machine.encode_bytes(&mut backing, 0, ty, valid)?;
            let bad = machine.evaluator.fields(&[scalar(37)], None)?;
            assert!(machine.encode_bytes(&mut backing, 0, ty, bad).is_err());
            Ok(())
        },
    )
}

#[test]
fn captured_zero_stride_repeat_validates_one_recipe_without_count_scaled_work() -> Result<(), String>
{
    let mut observations = Vec::new();
    for count in [8u64, 32, 128, 1 << 30] {
        let source = format!(
            "fn f(){{let inner=||37u32;let a=[inner;{count}];let outer=move||a[0]();let _=outer;}}"
        );
        with_capture(&source, 0, |machine, ty| {
            assert_eq!(machine.footprint(ty)?.size, 0);
            let empty = machine.evaluator.fields(&[], None)?;
            let repeated = machine.evaluator.repeat(empty, count)?;
            let value = machine.evaluator.fields(&[repeated], None)?;
            let mut backing = machine.empty_backing(ty)?;
            let before = machine.work.steps;
            machine.encode_bytes(&mut backing, 0, ty, value)?;
            observations.push(machine.work.steps - before);
            let bad = machine.evaluator.repeat(scalar(37), count)?;
            let bad = machine.evaluator.fields(&[bad], None)?;
            assert!(machine.encode_bytes(&mut backing, 0, ty, bad).is_err());
            Ok(())
        })?;
    }
    assert!(observations.iter().all(|&n| n == observations[0]));
    Ok(())
}

fn width_source(width: usize) -> Result<String, String> {
    let mut source = String::from(
        "#![recursion_limit=\"512\"]#[repr(u8)]enum E<T>{A(T),Empty}fn main(){let f=const{",
    );
    for n in 0..width {
        write!(&mut source, "let a{n}={n}u32;").map_err(|e| e.to_string())?;
    }
    source.push_str("let e=E::A(move||");
    source.push_str(
        &(0..width)
            .map(|n| format!("a{n}"))
            .collect::<Vec<_>>()
            .join("+"),
    );
    source.push_str(" );match e{E::A(f)=>f,E::Empty=>loop{}}};check(f()==");
    source.push_str(&(width * (width - 1) / 2).to_string());
    source.push_str(")}fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}");
    Ok(source)
}

fn selected_word(
    machine: &mut Machine<'_, '_>,
    owner: Resolved,
    index: u32,
) -> Result<Resolved, Failure> {
    let ty = TypeId::primitive(Primitive::U32);
    let address = machine.field_projection_for::<true>(
        owner,
        crate::sema::members::Selection::Tuple(index),
        ty,
    )?;
    Ok(Resolved {
        address,
        ty,
        union: None,
        raw: Some(crate::const_eval::memory::RawAccess {
            alignment: crate::const_eval::memory::RawAlignment::Required,
            location: address.location(),
            ty,
        }),
    })
}

#[test]
fn captured_selected_field_work_is_independent_of_capture_width_and_unrelated_types()
-> Result<(), String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa27/closure-byte-backing/scaling-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut expected = None;
    for width in [8, 32, 128] {
        let source = width_source(width)?;
        std::fs::write(out.join(format!("captures{width}.rs")), &source)
            .map_err(|e| e.to_string())?;
        let mut observations = Vec::new();
        for noise in [0, 1024] {
            with_capture(&source, noise, |machine, ty| {
                let values = vec![scalar(37); width];
                let value = machine.evaluator.fields(&values, None)?;
                let address = machine.allocate(&mut Vec::new(), Some(value), ty)?;
                let root = address.location().ok_or("root")?;
                let before = machine.work.steps;
                machine.convert_byte_root(root)?;
                let owner = Resolved {
                    address,
                    ty,
                    raw: None,
                    union: None,
                };
                let index = u32::try_from(width - 1).map_err(|_| "capture index")?;
                let first = selected_word(machine, owner, index)?;
                assert!(machine.read(first)?.ok_or("initial field")? == scalar(37));
                let cold = machine.work.steps - before;
                let mut rows = Vec::new();
                let retained = machine.memory.retained_cells();
                for repeats in [8, 32, 128] {
                    let before = machine.work.steps;
                    for _ in 0..repeats {
                        let selected = selected_word(machine, owner, index)?;
                        machine.write_place(selected, scalar(41))?;
                        assert!(machine.read(selected)?.ok_or("read")? == scalar(41));
                    }
                    let steps = machine.work.steps - before;
                    assert_eq!(machine.memory.retained_cells(), retained);
                    println!(
                        "capture-work width={width} noise={noise} repeats={repeats} cold={cold} warm={steps} retained={retained}"
                    );
                    rows.push(steps);
                }
                assert_eq!(rows[1], 4 * rows[0]);
                assert_eq!(rows[2], 4 * rows[1]);
                if let Some(expected) = &expected {
                    assert_eq!(&rows, expected);
                } else {
                    expected = Some(rows.clone());
                }
                observations.push((cold, retained, rows));
                Ok(())
            })?;
        }
        assert_eq!(observations[0], observations[1]);
    }
    Ok(())
}

#[test]
fn captured_product_count_variant_and_current_initialization_are_exact() -> Result<(), String> {
    with_capture(
        "fn f(){let x=37u32;let f=move||x;let _=f;}",
        0,
        |machine, ty| {
            let value = machine.evaluator.fields(&[scalar(37)], None)?;
            for (values, variant) in [
                (&[][..], None),
                (&[scalar(37), scalar(41)][..], None),
                (
                    &[scalar(37)][..],
                    Some(crate::sema::members::VariantId(u32::MAX)),
                ),
            ] {
                let value = machine.evaluator.fields(values, variant)?;
                let mut backing = machine.empty_backing(ty)?;
                assert!(machine.encode_bytes(&mut backing, 0, ty, value).is_err());
            }
            let address = machine.allocate(&mut Vec::new(), Some(value), ty)?;
            let root = address.location().ok_or("root")?;
            let id = machine.convert_byte_root(root)?;
            let Read::Value(value) = machine.decode_bytes(id, 0, ty, ReadPurpose::Value)? else {
                return Err("pending".into());
            };
            assert!(matches!(
                Reader::new(machine.evaluator.facts()).view(value)?,
                View::Product {
                    length: 1,
                    variant: None
                }
            ));
            assert!(machine.evaluator.field(value, 0)? == scalar(37));
            machine.memory.bytes.roots[id.0 as usize].initialized[0] = 0;
            assert!(machine.decode_bytes(id, 0, ty, ReadPurpose::Value).is_err());
            Ok(())
        },
    )
}

#[test]
fn captured_raw_word_never_upgrades_shared_write_permission() -> Result<(), String> {
    with_capture(
        "fn f(p:*mut u32){let f=move||p;let _=f;}",
        0,
        |machine, ty| {
            let pointer = machine.allocate(
                &mut Vec::new(),
                Some(scalar(37)),
                TypeId::primitive(Primitive::U32),
            )?;
            for pointer in [pointer, pointer.restrict_shared()] {
                let value = machine.evaluator.fields(&[pointer], None)?;
                let address = machine.allocate(&mut Vec::new(), Some(value), ty)?;
                let id = machine.convert_byte_root(address.location().ok_or("root")?)?;
                let Read::Value(value) = machine.decode_bytes(id, 0, ty, ReadPurpose::Value)?
                else {
                    return Err("pending".into());
                };
                assert_eq!(
                    machine.evaluator.field(value, 0)?.write_permission(),
                    pointer.write_permission()
                );
            }
            Ok(())
        },
    )
}

#[test]
fn captured_reference_expiry_is_rechecked_after_warm_decode() -> Result<(), String> {
    with_capture(
        "fn f(p:&'static u32){let f=move||*p;let _=f;}",
        0,
        |machine, ty| {
            let pointer = machine
                .allocate(
                    &mut Vec::new(),
                    Some(scalar(37)),
                    TypeId::primitive(Primitive::U32),
                )?
                .restrict_shared();
            let value = machine.evaluator.fields(&[pointer], None)?;
            let address = machine.allocate(&mut Vec::new(), Some(value), ty)?;
            let id = machine.convert_byte_root(address.location().ok_or("root")?)?;
            machine.read_operation = Some(crate::const_eval::ReadOperation {
                frame: 1,
                position: 0,
            });
            assert!(matches!(
                machine.decode_bytes(id, 0, ty, ReadPurpose::Value)?,
                Read::Value(_)
            ));
            let before = machine.memory.bytes.checked_duties;
            machine
                .memory
                .expire(&[pointer.location().ok_or("referent")?]);
            assert!(machine.decode_bytes(id, 0, ty, ReadPurpose::Value).is_err());
            assert_eq!(machine.memory.bytes.checked_duties, before + 1);
            Ok(())
        },
    )
}
