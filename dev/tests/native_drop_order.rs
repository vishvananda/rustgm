//! Test-only observers check actual native cleanup order after successful lowering.
#[path = "helpers/cleanup_lowering.rs"]
mod lowering;
use rustgm::lowir::{
    builder::Builder,
    data::{GlobalId, Item},
    program::{MemorySpan, Operand, Program, SymbolId, Terminator, Value},
    vocabulary::{Binary, Predicate, Type},
};
use std::{fmt::Write, path::Path, process::Command};
type Error = Box<dyn std::error::Error>;
const OBSERVER: &str = "fn trace(_:u8){}";
const CASES: &[(&str, &str, &[u8])] = &[
    (
        "or-patterns",
        "pa17/tests/spec/order/or-patterns.rs",
        &[9, 2, 1, 9, 1, 2],
    ),
    (
        "break-value",
        "pa17/tests/spec/order/break-value.rs",
        &[1, 9, 2],
    ),
    (
        "labeled-break",
        "pa17/tests/spec/order/labeled-break.rs",
        &[2, 1, 9, 3],
    ),
    (
        "conditional-local",
        "pa17/tests/spec/order/conditional-local.rs",
        &[9, 9, 1],
    ),
    (
        "moved-binding-order",
        "pa17/tests/spec/order/moved-binding-order.rs",
        &[9, 2, 1],
    ),
    (
        "short-circuit",
        "pa17/tests/spec/order/short-circuit.rs",
        &[1, 9],
    ),
    (
        "match-scrutinee",
        "pa17/tests/spec/order/match-scrutinee.rs",
        &[8, 1, 9],
    ),
    (
        "argument-return",
        "pa17/tests/spec/order/argument-return.rs",
        &[1, 9],
    ),
    (
        "nested-array",
        "pa17/tests/spec/order/nested-array.rs",
        &[9, 1, 2, 3, 4],
    ),
    (
        "guard-retry",
        "pa17/tests/spec/order/guard-retry.rs",
        &[1, 8, 1],
    ),
    (
        "guard-temporaries",
        "pa17/tests/spec/order/guard-temporaries.rs",
        &[2, 1, 9],
    ),
    (
        "conditional-fields",
        "pa17/tests/spec/order/conditional-fields.rs",
        &[9, 1, 2, 3, 1, 9, 2, 3],
    ),
    (
        "packed-fields",
        "pa17/tests/spec/order/packed-fields.rs",
        &[9, 1, 2],
    ),
    (
        "self-field-replacement",
        "pa17/tests/spec/order/self-field-replacement.rs",
        &[9, 8, 1, 3, 2],
    ),
    ("locals", "pa17/tests/spec/order/locals.rs", &[9, 2, 1]),
    ("fields", "pa17/tests/spec/order/fields.rs", &[9, 1, 2]),
    ("tuple", "pa17/tests/spec/order/tuple.rs", &[9, 1, 2, 3]),
    ("array", "pa17/tests/spec/order/array.rs", &[9, 1, 2, 3, 4]),
    (
        "enum",
        "pa17/tests/spec/order/enum.rs",
        &[9, 1, 2, 9, 3, 4, 9],
    ),
    (
        "custom-fields",
        "pa17/tests/spec/order/custom-fields.rs",
        &[9, 8, 1, 2],
    ),
    (
        "partial-field",
        "pa17/tests/spec/order/partial-field.rs",
        &[8, 2, 9, 1, 3],
    ),
    (
        "partial-array",
        "pa17/tests/spec/order/partial-array.rs",
        &[2, 3, 9, 1],
    ),
    (
        "local-temporary",
        "pa17/tests/spec/order/local-temporary.rs",
        &[9, 3, 2, 1],
    ),
    (
        "two-temporaries",
        "pa17/tests/spec/order/two-temporaries.rs",
        &[9, 4, 3, 2, 1],
    ),
    ("wildcard", "pa17/tests/spec/order/wildcard.rs", &[1, 9]),
    (
        "replacement",
        "pa17/tests/spec/order/replacement.rs",
        &[11, 12, 1, 9, 2],
    ),
    (
        "zero-repeat",
        "pa17/tests/spec/order/zero-repeat.rs",
        &[1, 9],
    ),
    (
        "parameters",
        "pa17/tests/spec/order/parameters.rs",
        &[9, 3, 2, 0, 1],
    ),
    ("return", "pa17/tests/spec/order/return.rs", &[1, 9, 2]),
    ("if-let", "pa17/tests/spec/order/if-let.rs", &[1, 9, 8, 1]),
    ("let-else", "pa17/tests/spec/order/let-else.rs", &[1, 9]),
    ("loop", "pa17/tests/spec/order/loop.rs", &[12, 2, 11, 1, 9]),
    (
        "zero-sized",
        "pa17/tests/spec/order/zero-sized.rs",
        &[9, 7, 7, 7],
    ),
];

// Instrument only the explicitly named empty observer and wrap the entry for a
// final callback count. The original compiler must succeed before this runs;
// cleanup code, field addresses, flag writes and all source operations stay intact.
fn instrument(program: &mut Program, observer: &str, expected: &[u8]) -> Result<(), Error> {
    let observer = program
        .symbols
        .iter()
        .position(|s| s.name == observer)
        .ok_or("observer not demanded")?;
    let signature = program.symbols[observer].signature;
    assert_eq!(program.parameters(signature), &[Type::U8]);
    assert_eq!(program.signatures[signature.0 as usize].result, Type::Void);
    let index = program.global(
        "test.trace.index".into(),
        MemorySpan { bytes: 8, align: 8 },
        false,
    )?;
    program.initialize(index, &[Item::Zero(8)])?;
    let abort = if let Some(i) = program.symbols.iter().position(|s| s.name == "abort") {
        SymbolId(u32::try_from(i)?)
    } else {
        let signature = program.signature(&[], Type::Void, true)?;
        program.declare("abort".into(), signature, false)?
    };
    let mut builder = Builder::default();
    let entry = builder.block()?;
    let matched = builder.block()?;
    let failed = builder.block()?;
    builder.enter(entry)?;
    let actual = builder.load(Type::I64, Operand::Global(index))?;
    for (position, &value) in expected.iter().enumerate() {
        let position = builder.integer(Type::I64, position as u128)?;
        let condition = builder.compare(Predicate::Eq, actual, position)?;
        let yes = builder.block()?;
        let no = builder.block()?;
        builder.terminate(Terminator::Branch {
            condition: condition.operand,
            yes,
            no,
        })?;
        builder.enter(yes)?;
        let expected = builder.integer(Type::U8, u128::from(value))?;
        let condition = builder.compare(
            Predicate::Eq,
            Value {
                ty: Type::U8,
                operand: Operand::Parameter(0),
            },
            expected,
        )?;
        builder.terminate(Terminator::Branch {
            condition: condition.operand,
            yes: matched,
            no: failed,
        })?;
        builder.enter(no)?;
    }
    builder.terminate(Terminator::Jump(failed))?;
    builder.enter(matched)?;
    let one = builder.integer(Type::I64, 1)?;
    let next = builder.binary(Binary::Add, actual, one)?;
    builder.store(next, Operand::Global(index))?;
    builder.terminate(Terminator::Return(None))?;
    builder.enter(failed)?;
    builder.call(
        program.symbols[abort.0 as usize].signature,
        Type::Void,
        Operand::Symbol(abort),
        &[],
    )?;
    builder.terminate(Terminator::Unreachable)?;
    program.symbols[observer].body = Some(builder.finish()?);

    check_final_count(program, index, expected.len())
}
fn check_final_count(program: &mut Program, index: GlobalId, expected: usize) -> Result<(), Error> {
    let old_entry = program
        .symbols
        .iter()
        .position(|s| s.entry)
        .ok_or("entry")?;
    program.symbols[old_entry].entry = false;
    program.symbols[old_entry].name = "test.source.entry".into();
    let signature = program.symbols[old_entry].signature;
    let new_entry = program.declare("main".into(), signature, true)?;
    let mut builder = Builder::default();
    let entry = builder.block()?;
    let yes = builder.block()?;
    let no = builder.block()?;
    builder.enter(entry)?;
    builder.call(
        signature,
        Type::I64,
        Operand::Symbol(SymbolId(u32::try_from(old_entry)?)),
        &[],
    )?;
    let actual = builder.load(Type::I64, Operand::Global(index))?;
    let expected = builder.integer(Type::I64, expected as u128)?;
    let condition = builder.compare(Predicate::Eq, actual, expected)?;
    builder.terminate(Terminator::Branch {
        condition: condition.operand,
        yes,
        no,
    })?;
    for (block, status) in [(yes, 0), (no, 1)] {
        builder.enter(block)?;
        let value = builder.integer(Type::I64, status)?;
        builder.terminate(Terminator::Return(Some(value)))?;
    }
    program.symbols[new_entry.0 as usize].body = Some(builder.finish()?);
    Ok(())
}
fn success(name: &str, output: std::process::Output) -> Result<Vec<u8>, Error> {
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "{name}: {:?}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output.stdout)
}
#[test]
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn instrumented_native_cleanup_order_matches_declared_and_rustc_traces() -> Result<(), Error> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join(format!("obj/native-drop-order-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for &(name, path, expected) in CASES {
        let source = std::fs::read_to_string(root.join(path))?;
        let input = folder.join(format!("{name}.rs"));
        std::fs::write(
            &input,
            source.replacen(OBSERVER, "fn trace(value:u8){println!(\"{}\",value);}", 1),
        )?;
        let oracle = folder.join(format!("{name}.rustc"));
        success(
            name,
            Command::new("rustc")
                .args([
                    "+1.98.0",
                    "--edition=2024",
                    "--crate-name=order",
                    "--cap-lints=allow",
                    "-C",
                    "panic=abort",
                ])
                .arg(&input)
                .arg("-o")
                .arg(&oracle)
                .output()?,
        )?;
        let trace = success(name, Command::new(&oracle).output()?)?;
        let mut declared = String::new();
        for n in expected {
            writeln!(declared, "{n}")?;
        }
        assert_eq!(trace, declared.as_bytes(), "{name}: rustc trace");
        let mut observer = None;
        let mut program = lowering::build_with(&source, |report, bodies| {
            for body in &bodies.bodies {
                if let rustgm::body::ir::Owner::Function { definition, .. } = body.owner
                    && report
                        .package
                        .names
                        .get(report.resolution.definitions[definition.0 as usize].name)
                        == "trace"
                {
                    assert!(observer.replace(format!("rg.{}", definition.0)).is_none());
                }
            }
            Ok(())
        })
        .map_err(|e| format!("{name}: {e}"))?;
        let control = lowering::build(&source)?;
        let mut ordinary = Vec::new();
        let mut captured = Vec::new();
        rustgm::lowir::writer::write(&control, &mut ordinary)?;
        rustgm::lowir::writer::write(&program, &mut captured)?;
        assert_eq!(
            ordinary, captured,
            "{name}: identity probe changed lowering"
        );
        let observer = observer.ok_or("observer identity")?;
        instrument(&mut program, &observer, expected)?;
        assert!(
            success(name, execute(root, &folder, name, &program)?)?.is_empty(),
            "{name}"
        );
        if name == "or-patterns" {
            reject_wrong_expectations(root, &folder, &source, &observer, expected)?;
        }
    }
    Ok(())
}

fn execute(
    root: &Path,
    folder: &Path,
    name: &str,
    program: &Program,
) -> Result<std::process::Output, Error> {
    let lowir = folder.join(format!("{name}.lowir"));
    rustgm::lowir::writer::write(program, &mut std::fs::File::create(&lowir)?)?;
    let native = folder.join(format!("{name}.native"));
    success(
        name,
        Command::new("python3")
            .arg(root.join("scripts/backend.py"))
            .arg(&lowir)
            .arg("-o")
            .arg(&native)
            .output()?,
    )?;
    Ok(Command::new(native).output()?)
}
fn reject_wrong_expectations(
    root: &Path,
    folder: &Path,
    source: &str,
    observer: &str,
    expected: &[u8],
) -> Result<(), Error> {
    for (name, status) in [("wrong-order", 134), ("missing-callback", 1)] {
        let mut declared = expected.to_vec();
        if status == 134 {
            declared[0] ^= 255;
        } else {
            declared.push(42);
        }
        let mut program = lowering::build(source)?;
        instrument(&mut program, observer, &declared)?;
        let output = execute(root, folder, name, &program)?;
        assert_eq!(output.status.code(), Some(status), "{name}");
        assert!(
            output.stdout.is_empty() && output.stderr.is_empty(),
            "{name}"
        );
    }
    Ok(())
}
