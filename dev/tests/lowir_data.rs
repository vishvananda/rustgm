use rustgm::lowir::{
    builder::Builder,
    data::{GlobalId, Item, Target},
    program::{MemorySpan, Operand, Program, Terminator},
    vocabulary::{Convert, Predicate, Type},
    writer,
};
use std::{path::Path, process::Command};

fn relocation(addend: i64) -> Result<Program, &'static str> {
    let mut p = Program::default();
    let ptr = p.global("pointer".into(), MemorySpan { bytes: 8, align: 8 }, true)?;
    let data = p.global("bytes".into(), MemorySpan { bytes: 2, align: 1 }, true)?;
    p.initialize(
        ptr,
        &[Item::Address {
            target: Target::Global(data),
            addend,
        }],
    )?;
    p.initialize(
        data,
        &[
            Item::Bits {
                ty: Type::U8,
                bits: 9,
            },
            Item::Bits {
                ty: Type::U8,
                bits: 42,
            },
        ],
    )?;
    let sig = p.signature(&[], Type::I64, false)?;
    let main = p.declare("main".into(), sig, true)?;
    let mut b = Builder::default();
    let entry = b.block()?;
    b.enter(entry)?;
    let mut address = b.load(Type::Ptr, Operand::Global(ptr))?;
    if addend < 0 {
        let two = b.integer(Type::I64, 2)?;
        address = b.index(Type::U8, address, two)?;
    }
    let value = b.load(Type::U8, address.operand)?;
    let result = b.convert(Convert::Zext, Type::I64, value)?;
    b.terminate(Terminator::Return(Some(result)))?;
    p.symbols[main.0 as usize].body = Some(b.finish()?);
    Ok(p)
}
fn function_address() -> Result<Program, &'static str> {
    let mut p = Program::default();
    let sig = p.signature(&[], Type::I64, false)?;
    let helper = p.declare("helper".into(), sig, false)?;
    let main = p.declare("main".into(), sig, true)?;
    let g = p.global(
        "function_pointer".into(),
        MemorySpan { bytes: 8, align: 8 },
        true,
    )?;
    p.initialize(
        g,
        &[Item::Address {
            target: Target::Function(helper),
            addend: 0,
        }],
    )?;
    let mut b = Builder::default();
    let entry = b.block()?;
    b.enter(entry)?;
    let value = b.integer(Type::I64, 77)?;
    b.terminate(Terminator::Return(Some(value)))?;
    p.symbols[helper.0 as usize].body = Some(b.finish()?);
    let mut b = Builder::default();
    let entry = b.block()?;
    b.enter(entry)?;
    let pointer = b.load(Type::Ptr, Operand::Global(g))?;
    let value = b
        .call(sig, Type::I64, pointer.operand, &[])?
        .ok_or("missing indirect result")?;
    b.terminate(Terminator::Return(Some(value)))?;
    p.symbols[main.0 as usize].body = Some(b.finish()?);
    Ok(p)
}
fn cyclic_addresses() -> Result<Program, &'static str> {
    let mut p = Program::default();
    let span = MemorySpan { bytes: 8, align: 8 };
    let a = p.global("a".into(), span, true)?;
    let c = p.global("c".into(), span, true)?;
    p.initialize(
        a,
        &[Item::Address {
            target: Target::Global(c),
            addend: 0,
        }],
    )?;
    p.initialize(
        c,
        &[Item::Address {
            target: Target::Global(a),
            addend: 0,
        }],
    )?;
    let sig = p.signature(&[], Type::I64, false)?;
    let main = p.declare("main".into(), sig, true)?;
    let mut b = Builder::default();
    let entry = b.block()?;
    b.enter(entry)?;
    let pointer = b.load(Type::Ptr, Operand::Global(a))?;
    let value = b.load(Type::Ptr, pointer.operand)?;
    let expected = b.address(Operand::Global(a))?;
    let equal = b.compare(Predicate::Eq, value, expected)?;
    let result = b.convert(Convert::Zext, Type::I64, equal)?;
    b.terminate(Terminator::Return(Some(result)))?;
    p.symbols[main.0 as usize].body = Some(b.finish()?);
    Ok(p)
}
fn exact_bits(ty: Type, bits: u128) -> Result<Program, &'static str> {
    let mut p = Program::default();
    let g = p.global(
        "exact_bits".into(),
        MemorySpan {
            bytes: 64,
            align: 64,
        },
        true,
    )?;
    p.initialize(g, &[Item::Bits { ty, bits }])?;
    let sig = p.signature(&[], Type::I64, false)?;
    let main = p.declare("main".into(), sig, true)?;
    let mut b = Builder::default();
    let entry = b.block()?;
    b.enter(entry)?;
    let value = b.load(ty, Operand::Global(g))?;
    let expected = b.integer(ty, bits)?;
    let equal = b.compare(Predicate::Eq, value, expected)?;
    let result = b.convert(Convert::Zext, Type::I64, equal)?;
    b.terminate(Terminator::Return(Some(result)))?;
    p.symbols[main.0 as usize].body = Some(b.finish()?);
    Ok(p)
}
#[test]
fn persistent_objects_and_relocations_execute_through_the_pinned_backend()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!("obj/lowir-data-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for (name, program, expected) in [
        ("forward-offset", relocation(1)?, 42),
        ("negative-offset", relocation(-1)?, 42),
        ("function-address", function_address()?, 77),
        ("cyclic-addresses", cyclic_addresses()?, 1),
        ("nan-bits", exact_bits(Type::U32, 0x7fc0_0042)?, 1),
        ("negative-zero-bits", exact_bits(Type::I64, 1 << 63)?, 1),
        ("wide-bits", exact_bits(Type::I128, u128::MAX)?, 1),
    ] {
        let mut text = Vec::new();
        writer::write(&program, &mut text)?;
        let source = folder.join(format!("{name}.lowir"));
        std::fs::write(&source, &text)?;
        let output = folder.join(name);
        let build = Command::new("python3")
            .arg(root.join("scripts/backend.py"))
            .arg(&source)
            .arg("-o")
            .arg(&output)
            .output()?;
        assert!(
            build.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        let result = Command::new(output).output()?;
        assert_eq!(result.status.code(), Some(expected), "{name}");
        assert!(result.stdout.is_empty() && result.stderr.is_empty());
        let mut count = Vec::new();
        writer::count(&program, &mut count)?;
        assert!(std::str::from_utf8(&count)?.contains(&format!(
            "globals {} data_items {}",
            program.globals.len(),
            program.data.len()
        )));
    }
    Ok(())
}
#[test]
fn data_validation_rejects_invalid_or_partial_storage() -> Result<(), &'static str> {
    let mut p = Program::default();
    assert!(
        p.global("bad".into(), MemorySpan { bytes: 0, align: 1 }, true)
            .is_err()
    );
    let g = p.global("g".into(), MemorySpan { bytes: 8, align: 8 }, true)?;
    for items in [
        vec![],
        vec![Item::Zero(0)],
        vec![Item::Zero(9)],
        vec![Item::Bits {
            ty: Type::U8,
            bits: 256,
        }],
        vec![Item::Bits {
            ty: Type::I1,
            bits: 2,
        }],
        vec![Item::Address {
            target: Target::Global(GlobalId(999)),
            addend: 0,
        }],
    ] {
        assert!(p.initialize(g, &items).is_err());
        assert!(p.data.is_empty());
        assert!(p.globals[0].items.is_none());
    }
    assert!(writer::write(&p, &mut Vec::new()).is_err());
    p.initialize(g, &[Item::Zero(8)])?;
    assert!(p.initialize(g, &[Item::Zero(8)]).is_err());
    let packed = p.global("packed".into(), MemorySpan { bytes: 9, align: 1 }, true)?;
    assert!(
        p.initialize(
            packed,
            &[
                Item::Bits {
                    ty: Type::U8,
                    bits: 1
                },
                Item::Address {
                    target: Target::Global(g),
                    addend: 0
                }
            ]
        )
        .is_err()
    );
    Ok(())
}
