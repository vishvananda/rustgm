//! Controlled `LowIR` interference is separate from ordinary source comparisons.
use super::{loops, native, semantic};
use rustgm::lowir::{
    program::{
        Access, BlockId, Function, Instruction, Literal, LiteralId, Operand, Operation, Program,
        ResultId, Terminator, Value,
    },
    vocabulary::Type,
    writer,
};
use std::{fmt::Write, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
pub(super) fn family(name: &str) -> Option<bool> {
    if name.starts_with("rg.intrinsic.fetch_min.t") {
        Some(false)
    } else if name.starts_with("rg.intrinsic.fetch_max.t") {
        Some(true)
    } else {
        None
    }
}
fn insert_interference(body: &mut Function, injected: u32, stale: bool) -> Result<(), Error> {
    let mut headers = Vec::new();
    for block in &body.blocks {
        let code =
            &body.instructions[block.instructions.start as usize..block.instructions.end as usize];
        if let Some(seed) = code
            .iter()
            .find(|i| matches!(i.operation, Operation::AtomicLoad { .. }))
        {
            let Terminator::Jump(retry) = block.terminator else {
                return Err("seed must be in one-shot preheader".into());
            };
            let Terminator::Branch { yes, no, .. } = body.blocks[retry.0 as usize].terminator
            else {
                return Err("missing CAS retry branch".into());
            };
            assert_eq!(no, retry);
            let Some(ResultId::Scalar(seed)) = seed.result else {
                return Err("missing seed value".into());
            };
            headers.push((yes, seed));
        }
    }
    assert_eq!(headers.len(), 5);
    if stale {
        for (done, seed) in headers {
            body.blocks[done.0 as usize].terminator = Terminator::Return(Some(Value {
                ty: Type::U32,
                operand: Operand::Temporary(seed),
            }));
        }
    }
    let literal = LiteralId(u32::try_from(body.literals.len())?);
    body.literals.push(Literal::Integer(u128::from(injected)));
    let old = std::mem::take(&mut body.instructions);
    let mut offsets = Vec::with_capacity(old.len() + 1);
    let mut inserted = 0;
    for instruction in old {
        offsets.push(u32::try_from(body.instructions.len())?);
        let seed = matches!(instruction.operation, Operation::AtomicLoad { .. });
        body.instructions.push(instruction);
        if seed {
            body.instructions.push(Instruction {
                result: None,
                operation: Operation::Store {
                    access: Access::Ordinary,
                    value: Value {
                        ty: Type::U32,
                        operand: Operand::Literal(literal),
                    },
                    address: Operand::Parameter(0),
                },
            });
            inserted += 1;
        }
    }
    offsets.push(u32::try_from(body.instructions.len())?);
    assert_eq!(inserted, 5);
    for (index, block) in body.blocks.iter_mut().enumerate() {
        block.instructions.start = offsets[block.instructions.start as usize];
        block.instructions.end = offsets[block.instructions.end as usize];
        let code =
            &body.instructions[block.instructions.start as usize..block.instructions.end as usize];
        let stores = code
            .iter()
            .filter(|i| {
                matches!(
                    i.operation,
                    Operation::Store {
                        address: Operand::Parameter(0),
                        ..
                    }
                )
            })
            .count();
        if let Terminator::Branch { no, .. } = block.terminator
            && no == BlockId(u32::try_from(index)?)
        {
            assert_eq!(stores, 0);
            assert_eq!(code.len(), 8);
        }
        if stores != 0 {
            assert_eq!(stores, 1);
            assert!(matches!(code[0].operation, Operation::AtomicLoad { .. }));
            assert!(matches!(
                code[1].operation,
                Operation::Store {
                    address: Operand::Parameter(0),
                    ..
                }
            ));
        }
    }
    Ok(())
}
fn program(
    text: &str,
    injected: u32,
    stale: bool,
    original: &mut Vec<u8>,
) -> Result<Program, Error> {
    let (mut report, mut facts) = semantic::analyze(text)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let mut program = rustgm::lower::instances::build(&report, &mut facts, entry)?;
    let mut count = 0;
    for symbol in &program.symbols {
        if let Some(maximum) = family(&symbol.name) {
            loops::check(
                symbol.body.as_ref().ok_or("missing extremum body")?,
                Some(maximum),
            )?;
            count += 1;
        }
    }
    assert_eq!(count, 2);
    writer::write(&program, original)?;
    for symbol in &mut program.symbols {
        if family(&symbol.name).is_some() {
            insert_interference(
                symbol.body.as_mut().ok_or("missing extremum body")?,
                injected,
                stale,
            )?;
        }
    }
    Ok(program)
}
fn source(injected: u32, operand: u32) -> Result<String, Error> {
    let mut text = format!(
        "{}use core::sync::atomic::{{AtomicU32,Ordering}};fn main(){{",
        native::CHECK
    );
    for order in ["Relaxed", "Release", "Acquire", "AcqRel", "SeqCst"] {
        for (method, expected) in [
            ("fetch_min", injected.min(operand)),
            ("fetch_max", injected.max(operand)),
        ] {
            write!(
                text,
                "{{let a=AtomicU32::new(7);let old=a.{method}({operand},Ordering::{order});check(old=={injected});check(a.load(Ordering::Relaxed)=={expected});}}"
            )?;
        }
    }
    text.push('}');
    Ok(text)
}
#[test]
#[ignore = "requires pinned CPPGM; deliberately instrumented LowIR, not a source oracle"]
fn retried_observation_returns_old_value_and_stale_seed_fails() -> Result<(), Error> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa28/extrema-interference-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    for (injected, operand) in [
        (0x8000_0000, 37),
        (0x7fff_ffff, 0x8000_0000),
        (0x8000_0000, 0x8000_0000),
    ] {
        let text = source(injected, operand)?;
        for stale in [false, true] {
            let name = format!("{injected}-{operand}-stale{stale}");
            std::fs::write(out.join(format!("{name}.rs")), &text)?;
            let mut original = Vec::new();
            let program = program(&text, injected, stale, &mut original)?;
            std::fs::write(out.join(format!("{name}.original.lowir")), original)?;
            let mut bytes = Vec::new();
            writer::write(&program, &mut bytes)?;
            let path = out.join(format!("{name}.lowir"));
            std::fs::write(&path, bytes)?;
            let binary = out.join(format!("{name}.native"));
            let backend = Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg(&path)
                .arg("-o")
                .arg(&binary)
                .output()?;
            assert!(
                backend.status.success(),
                "{}",
                String::from_utf8_lossy(&backend.stderr)
            );
            let run = Command::new(binary).output()?;
            std::fs::write(out.join(format!("{name}.run.stderr")), &run.stderr)?;
            std::fs::write(
                out.join(format!("{name}.run.status")),
                format!("{:?}\n", run.status.code()),
            )?;
            assert_eq!(run.status.code(), Some(if stale { 134 } else { 0 }));
            assert!(run.stdout.is_empty() && run.stderr.is_empty());
            println!(
                "injected={injected} operand={operand} stale={stale} status={:?}",
                run.status.code()
            );
        }
    }
    Ok(())
}
