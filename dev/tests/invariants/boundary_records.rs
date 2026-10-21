//! Object boundary identities cannot be substituted for scalar storage or results.
use super::Value as BoundaryValue;
use super::*;
use crate::lowir::{builder::Builder, program::*, vocabulary::Type as Scalar};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn pair() -> Object {
    Object {
        span: MemorySpan {
            bytes: 16,
            align: 8,
        },
        classes: Classes::Eightbytes {
            first: Class::Sse,
            second: Some(Class::Integer),
        },
    }
}
fn identity() -> TestResult<(Program, SignatureId)> {
    let mut program = Program::default();
    let object = pair();
    let signature =
        program.object_signature(&[Type::Object(object)], Type::Object(object), false)?;
    let symbol = program.declare("pair_identity".into(), signature, false)?;
    let mut builder = Builder::default();
    let block = builder.block()?;
    builder.enter(block)?;
    builder.return_object(
        &program,
        signature,
        ObjectValue {
            operand: Operand::Parameter(0),
            span: object.span,
        },
    )?;
    program.define(symbol, builder.finish()?)?;
    Ok((program, signature))
}
#[test]
fn canonical_object_boundaries_preserve_classes_and_reject_incompatible_aliases() -> TestResult {
    let (mut program, signature) = identity()?;
    assert_eq!(
        program.object_signature(&[Type::Object(pair())], Type::Object(pair()), false)?,
        signature
    );
    assert_eq!(program.signatures.len(), 1);
    let wrong = Object {
        classes: Classes::Eightbytes {
            first: Class::Integer,
            second: Some(Class::Sse),
        },
        ..pair()
    };
    let other = program.object_signature(&[Type::Object(wrong)], Type::Object(wrong), false)?;
    assert!(
        program
            .declare("pair_identity".into(), other, false)
            .is_err()
    );
    assert_eq!(program.symbols.len(), 1);
    let invalid = Object {
        classes: Classes::Eightbytes {
            first: Class::Integer,
            second: None,
        },
        ..pair()
    };
    assert!(
        program
            .object_signature(&[Type::Object(invalid)], Type::Object(invalid), false)
            .is_err()
    );
    assert_eq!(program.signatures.len(), 2);
    let mut text = Vec::new();
    crate::lowir::writer::write(&program, &mut text)?;
    let text = String::from_utf8(text)?;
    assert!(text.contains("%p0 : obj<16x8> [classes=sse_integer]"));
    assert!(text.contains("-> obj<16x8> [result_classes=sse_integer]"));
    Ok(())
}
#[test]
fn sparse_object_calls_reject_invalid_ids_spans_and_cross_kind_results() -> TestResult {
    let (mut program, signature) = identity()?;
    let target = SymbolId(0);
    let mut builder = Builder::default();
    let block = builder.block()?;
    builder.enter(block)?;
    assert!(
        builder
            .object_call(
                &program,
                signature,
                Operand::Symbol(target),
                &[BoundaryValue::Object(ObjectValue {
                    operand: Operand::ObjectTemporary(ObjectTempId(0)),
                    span: pair().span
                })]
            )
            .is_err()
    );
    assert!(
        builder
            .object_call(
                &program,
                signature,
                Operand::Symbol(target),
                &[BoundaryValue::Scalar(crate::lowir::program::Value {
                    ty: Scalar::I64,
                    operand: Operand::Parameter(0)
                })]
            )
            .is_err()
    );
    let ResultValue::Object(result) = builder.object_call(
        &program,
        signature,
        Operand::Symbol(target),
        &[BoundaryValue::Object(ObjectValue {
            operand: Operand::Parameter(0),
            span: pair().span,
        })],
    )?
    else {
        return Err("object result".into());
    };
    builder.return_object(&program, signature, result)?;
    let symbol = program.declare("pair_call".into(), signature, false)?;
    program.define(symbol, builder.finish()?)?;
    crate::lowir::writer::write(&program, &mut Vec::new())?;
    let body = program.symbols[symbol.0 as usize]
        .body
        .as_mut()
        .ok_or("body")?;
    let instruction = body.instructions.first_mut().ok_or("call")?;
    let original = instruction.result;
    instruction.result = Some(ResultId::Scalar(TempId(0)));
    assert!(crate::lowir::writer::write(&program, &mut Vec::new()).is_err());
    let body = program.symbols[symbol.0 as usize]
        .body
        .as_mut()
        .ok_or("body")?;
    body.instructions[0].result = original;
    let objects = body.objects.as_mut().ok_or("object arena")?;
    objects.temporaries[0] = SpanId(u32::MAX);
    assert!(crate::lowir::writer::write(&program, &mut Vec::new()).is_err());
    let mut scalar = Builder::default();
    let block = scalar.block()?;
    scalar.enter(block)?;
    assert!(
        scalar
            .copy(
                Scalar::I64,
                crate::lowir::program::Value {
                    ty: Scalar::I64,
                    operand: Operand::ObjectTemporary(ObjectTempId(0))
                }
            )
            .is_err()
    );
    Ok(())
}
#[test]
#[cfg(target_pointer_width = "64")]
fn scalar_records_and_arrays_keep_their_compact_layout() {
    assert_eq!(size_of::<Scalar>(), 1);
    assert_eq!(size_of::<crate::lowir::program::Value>(), 12);
    assert_eq!(size_of::<Operand>(), 8);
    assert_eq!(size_of::<Option<ResultId>>(), 8);
    assert_eq!(size_of::<Operation>(), 32);
    assert_eq!(size_of::<Instruction>(), 40);
    assert_eq!(size_of::<crate::lowir::program::Signature>(), 12);
    assert_eq!(size_of::<Function>(), 200);
}

#[test]
fn object_identities_cannot_enter_scalar_addresses_calls_or_returns() -> TestResult {
    let (mut program, signature) = identity()?;
    let mut scalar = Builder::default();
    let block = scalar.block()?;
    scalar.enter(block)?;
    let object = Operand::ObjectTemporary(ObjectTempId(0));
    assert!(scalar.load(Scalar::I64, object).is_err());
    assert!(scalar.call(signature, Scalar::Void, object, &[]).is_err());
    let body = program.symbols[0].body.as_mut().ok_or("body")?;
    body.blocks[0].terminator = Terminator::Return(Some(crate::lowir::program::Value {
        ty: Scalar::I64,
        operand: Operand::Parameter(0),
    }));
    assert!(crate::lowir::writer::write(&program, &mut Vec::new()).is_err());
    let body = program.symbols[0].body.as_mut().ok_or("body")?;
    body.blocks[0].terminator = Terminator::ReturnObject {
        operand: Operand::Parameter(0),
        span: SpanId(u32::MAX),
    };
    assert!(crate::lowir::writer::write(&program, &mut Vec::new()).is_err());
    Ok(())
}
