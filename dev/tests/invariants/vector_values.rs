//! `v128` values and the `vector` operations lowering emits keep the
//! backend's spellings, and an operand that differs from its shape is
//! rejected before it is emitted.
use crate::lowir::{
    builder::Builder,
    program::{Operand, Program, Terminator, Value},
    vocabulary::{Type, Vector},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const LEFT: Value = Value {
    ty: Type::V128,
    operand: Operand::Parameter(0),
};
const RIGHT: Value = Value {
    ty: Type::V128,
    operand: Operand::Parameter(1),
};
const WORD: Value = Value {
    ty: Type::I32,
    operand: Operand::Parameter(2),
};

#[test]
fn vector_operations_write_the_backend_forms_in_operand_order() -> TestResult {
    let mut program = Program::default();
    let signature = program.signature(&[Type::V128, Type::V128, Type::I32], Type::I32, false)?;
    let symbol = program.declare("mask".into(), signature, false)?;
    let mut builder = Builder::default();
    let block = builder.block()?;
    builder.enter(block)?;
    let equal = builder.vector(Vector::Pcmpeqb, LEFT, Some(RIGHT), 0)?;
    let greater = builder.vector(Vector::Pcmpgtb, RIGHT, Some(LEFT), 0)?;
    let either = builder.vector(Vector::Por, equal, Some(greater), 0)?;
    let low = builder.vector(Vector::FromI32, WORD, None, 0)?;
    let all = builder.vector(Vector::Pshufd, low, None, 0)?;
    let both = builder.vector(Vector::Pand, either, Some(all), 0)?;
    let slot = builder.slot(Type::V128)?;
    builder.store(both, Operand::Slot(slot))?;
    let back = builder.load(Type::V128, Operand::Slot(slot))?;
    let mask = builder.vector(Vector::Pmovmskb, back, None, 0)?;
    builder.terminate(Terminator::Return(Some(mask)))?;
    program.define(symbol, builder.finish()?)?;
    let mut text = Vec::new();
    crate::lowir::writer::write(&program, &mut text)?;
    let text = String::from_utf8(text)?;
    let lines: Vec<&str> = text
        .lines()
        .filter_map(|line| line.split_once(" = vector ").map(|(_, rest)| rest))
        .collect();
    let [eq, gt, or, from, shuffle, and, movemask] = lines.as_slice() else {
        return Err(format!("vector lines {lines:?}\n{text}").into());
    };
    let operands = |line: &str, op: &str| -> Vec<String> {
        line.strip_prefix(op)
            .unwrap_or("")
            .split(", ")
            .map(|operand| operand.trim().to_owned())
            .collect()
    };
    let (left, right) = (operands(eq, "pcmpeqb "), operands(gt, "pcmpgtb "));
    assert_eq!((left.len(), right.len()), (2, 2), "{text}");
    assert_eq!((&left[0], &left[1]), (&right[1], &right[0]), "{text}");
    assert_eq!(operands(or, "por ").len(), 2, "{text}");
    assert_eq!(operands(from, "fromi32 ").len(), 1, "{text}");
    assert_eq!(
        operands(shuffle, "pshufd ").last().map(String::as_str),
        Some("0"),
        "{text}"
    );
    assert_eq!(operands(and, "pand ").len(), 2, "{text}");
    assert_eq!(operands(movemask, "pmovmskb ").len(), 1, "{text}");
    assert!(text.contains("store v128 "), "{text}");
    assert!(text.contains("load v128 "), "{text}");
    Ok(())
}

#[test]
fn vector_operands_that_differ_from_their_shape_are_rejected() -> TestResult {
    let mut builder = Builder::default();
    let block = builder.block()?;
    builder.enter(block)?;
    for (op, left, right, immediate) in [
        (Vector::Pand, LEFT, None, 0),
        (Vector::Pand, LEFT, Some(RIGHT), 1),
        (Vector::Pand, LEFT, Some(WORD), 0),
        (Vector::Pmovmskb, WORD, None, 0),
        (Vector::Pshufd, LEFT, Some(RIGHT), 0),
        (Vector::FromI32, LEFT, None, 0),
    ] {
        assert_eq!(
            builder.vector(op, left, right, immediate).err(),
            Some("LowIR vector operands differ from their operation's shape"),
            "{op:?}"
        );
    }
    Ok(())
}
