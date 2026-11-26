//! The Rust side of `make test-backend`: the module that
//! `tests/backend/library.c` states in C, built here through `RustGM`'s own
//! `LowIR` model and compiled in process by the bridge.
//!
//! Usage: `rustgm-bridge-handshake FOLDER`
//!
//! Writes `text.lowir`, the program as `RustGM`'s writer serializes it, and
//! for each of `fixed` and `pic` the object `FORM.o` with the library's text
//! view of the module, `FORM.lowir`. `tests/backend/library-observer.c` links
//! each object and observes its executable cases:
//!
//! - `rg_library_seed`: a read-only global holding 40;
//! - `rg_library_answer(x)`: the seed plus `x`, through pure readonly AT&T assembly;
//! - `rg_library_swap(p)`: `{i64, f64}` to `{f64, i64}`, by `classes=` and
//!   `result_classes=`;
//! - `rg_library_apply(f, x, p)`: `f(p)` through a pointer, a call boundary
//!   `as (...)` with an object temporary for its result;
//! - `rg_library_tally(n, d)`: the variadic `rg_host_tally(1, {n, d})`, by
//!   `arity=variadic` and a call-site `classes=1:`;
//! - `rg_library_stream(d, s)`: a pure vector snapshot, then one `movntps`;
//! - `rg_library_masked_load(s, d, k, p)`: an object check of a masked AVX-512F load;
//! - `rg_library_turn(r)`: `{{f32, f32}, i64}` to `{i64, f32[2]}`, by the
//!   `classes=` and `result_classes=` that `RustGM` states and that the
//!   library derives for `library.c`'s record types;
//! - `rg_library_stop(code)`: `return=noreturn`, calling `exit(code)`.
//! - `rg_library_match(a, b, x)`: the sign bits of the bytes equal in two
//!   `v128` values and in `x`'s dword broadcast, by `vector` `pcmpeqb`,
//!   `fromi32`, `pshufd 0`, `pand` and `pmovmskb`;
//! - `rg_library_tick()`: its thread's instance of the thread-local
//!   `rg_library_ticks`, which starts at 10, incremented.
use rustgm::lowir::{
    asm::{self, Gpr, Kind, Memory, Wide},
    boundary::{self, Class, Classes, ObjectValue, ResultValue},
    data::{GlobalStorage, Item, Target},
    program::{MemorySpan, Operand, Program, SignatureId, Terminator, Value},
    vocabulary::{Binary, Type, Vector},
    writer,
};
use rustgm_backend_bridge::Request;
use std::{fs, io::BufWriter, path::Path, process::ExitCode};

const PAIR: MemorySpan = MemorySpan {
    bytes: 16,
    align: 8,
};

fn pair(first: Class, second: Class) -> boundary::Type {
    boundary::Type::Object(boundary::Object {
        span: PAIR,
        classes: Classes::Eightbytes {
            first,
            second: Some(second),
        },
    })
}

fn parameter(ty: Type, index: u32) -> Value {
    Value {
        ty,
        operand: Operand::Parameter(index),
    }
}

/// The seed and `rg_library_answer`.
fn answer(program: &mut Program) -> Result<(), &'static str> {
    let seed = program.global(
        "rg_library_seed".into(),
        MemorySpan { bytes: 8, align: 8 },
        GlobalStorage::Readonly,
    )?;
    program.initialize(
        seed,
        &[Item::Bits {
            ty: Type::I64,
            bits: 40,
        }],
    )?;
    let signature = program.signature(&[Type::I64], Type::I64, false)?;
    let symbol = program.declare("rg_library_answer".into(), signature, false)?;
    let mut draft = program.assembly()?;
    draft.binding(Kind::Integer(Type::I64), Some(Gpr::Ax), false, true, false)?;
    draft.operand(Kind::Pointer)?;
    draft.binding(Kind::Integer(Type::I64), None, true, false, false)?;
    draft.text("mov (")?;
    draft.operand_use(1)?;
    draft.text("), ")?;
    draft.operand_use_view(0, 64)?;
    draft.text("\nadd ")?;
    draft.operand_use(2)?;
    draft.text(", ")?;
    draft.operand_use_view(0, 64)?;
    let template = draft.finish_memory(asm::DUPLICABLE, 0, asm::FLAGS, true, Memory::ReadOnly)?;
    let mut body = program.body_builder();
    let entry = body.block()?;
    body.enter(entry)?;
    let seed_address = body.address(Operand::Global(seed))?;
    let slot = body.slot(Type::I64)?;
    let pointer = body.address(Operand::Slot(slot))?;
    body.asm(template, &[pointer, seed_address, parameter(Type::I64, 0)])?;
    let sum = body.load(Type::I64, pointer.operand)?;
    body.terminate(Terminator::Return(Some(sum)))?;
    program.define(symbol, body.finish()?)
}

/// `rg_library_tick`: a counter with one instance per thread.
fn tick(program: &mut Program) -> Result<(), &'static str> {
    let ticks = program.global(
        "rg_library_ticks".into(),
        MemorySpan { bytes: 8, align: 8 },
        GlobalStorage::ThreadLocal,
    )?;
    program.initialize(
        ticks,
        &[Item::Bits {
            ty: Type::I64,
            bits: 10,
        }],
    )?;
    let signature = program.signature(&[], Type::I64, false)?;
    let symbol = program.declare("rg_library_tick".into(), signature, false)?;
    let mut body = program.body_builder();
    let entry = body.block()?;
    body.enter(entry)?;
    let address = body.address(Operand::Global(ticks))?;
    let loaded = body.load(Type::I64, address.operand)?;
    let one = body.integer(Type::I64, 1)?;
    let sum = body.binary(Binary::Add, loaded, one)?;
    body.store(sum, address.operand)?;
    body.terminate(Terminator::Return(Some(sum)))?;
    program.define(symbol, body.finish()?)
}

/// `rg_library_swap`: the fields of the pair in the other order.
fn swap(program: &mut Program, signature: SignatureId) -> Result<(), &'static str> {
    let symbol = program.declare("rg_library_swap".into(), signature, false)?;
    let mut body = program.body_builder();
    let entry = body.block()?;
    body.enter(entry)?;
    let incoming = body.object_slot(PAIR)?;
    let outgoing = body.object_slot(PAIR)?;
    let from = body.address(Operand::Slot(incoming))?;
    let argument = ObjectValue {
        operand: Operand::Parameter(0),
        span: PAIR,
    };
    body.copy_object_value(program, signature, argument, from)?;
    let integer = body.load(Type::I64, from.operand)?;
    let eight = body.integer(Type::I64, 8)?;
    let second = body.index(Type::U8, from, eight)?;
    let double = body.load(Type::F64, second.operand)?;
    let to = body.address(Operand::Slot(outgoing))?;
    body.store(double, to.operand)?;
    let eight = body.integer(Type::I64, 8)?;
    let second = body.index(Type::U8, to, eight)?;
    body.store(integer, second.operand)?;
    let result = ObjectValue {
        operand: Operand::Slot(outgoing),
        span: PAIR,
    };
    body.return_object(program, signature, result)?;
    let function = body.finish()?;
    program.define(symbol, function)
}

/// `rg_library_apply`: the pair passed to a function through a pointer.
fn apply(program: &mut Program, callee: SignatureId) -> Result<(), &'static str> {
    let parameters = [
        boundary::Type::Scalar(Type::Ptr),
        boundary::Type::Scalar(Type::F64),
        pair(Class::Integer, Class::Sse),
    ];
    let signature =
        program.object_signature(&parameters, pair(Class::Sse, Class::Integer), false)?;
    let symbol = program.declare("rg_library_apply".into(), signature, false)?;
    let mut body = program.body_builder();
    let entry = body.block()?;
    body.enter(entry)?;
    let argument = body.object_slot(PAIR)?;
    let to = body.address(Operand::Slot(argument))?;
    let incoming = ObjectValue {
        operand: Operand::Parameter(2),
        span: PAIR,
    };
    body.copy_object_value(program, signature, incoming, to)?;
    let passed = boundary::Value::Object(ObjectValue {
        operand: Operand::Slot(argument),
        span: PAIR,
    });
    let ResultValue::Object(returned) =
        body.object_call(program, callee, Operand::Parameter(0), &[passed])?
    else {
        return Err("the applied function returns no object");
    };
    let result = body.object_slot(PAIR)?;
    let to = body.address(Operand::Slot(result))?;
    body.copy_object_value(program, signature, returned, to)?;
    let result = ObjectValue {
        operand: Operand::Slot(result),
        span: PAIR,
    };
    body.return_object(program, signature, result)?;
    let function = body.finish()?;
    program.define(symbol, function)
}

/// `rg_library_tally`: one pair passed through `...` of a host function.
fn tally(program: &mut Program) -> Result<(), &'static str> {
    let host = program.signature_with_arity(&[Type::I32], Type::F64, false, true)?;
    let host_symbol = program.declare("rg_host_tally".into(), host, false)?;
    let signature = program.signature(&[Type::I64, Type::F64], Type::F64, false)?;
    let symbol = program.declare("rg_library_tally".into(), signature, false)?;
    let mut body = program.body_builder();
    let entry = body.block()?;
    body.enter(entry)?;
    let value = body.object_slot(PAIR)?;
    let to = body.address(Operand::Slot(value))?;
    body.store(parameter(Type::I64, 0), to.operand)?;
    let eight = body.integer(Type::I64, 8)?;
    let second = body.index(Type::U8, to, eight)?;
    body.store(parameter(Type::F64, 1), second.operand)?;
    let one = body.integer(Type::I32, 1)?;
    let arguments = [
        boundary::Value::Scalar(one),
        boundary::Value::Object(ObjectValue {
            operand: Operand::Slot(value),
            span: PAIR,
        }),
    ];
    let tail = [pair(Class::Integer, Class::Sse)];
    let ResultValue::Scalar(result) = body.boundary_call(
        program,
        host,
        Operand::Symbol(host_symbol),
        &arguments,
        &tail,
    )?
    else {
        return Err("the host tally returns no scalar");
    };
    body.terminate(Terminator::Return(Some(result)))?;
    let function = body.finish()?;
    program.define(symbol, function)
}

/// The `movntps` template and `rg_library_stream`.
fn stream(program: &mut Program) -> Result<(), &'static str> {
    let mut draft = program.assembly()?;
    draft.binding(Kind::Vector, None, true, true, false)?;
    draft.text("movdqa ")?;
    draft.operand_use(0)?;
    draft.text(", ")?;
    draft.operand_use(0)?;
    let snapshot = draft.finish_memory(asm::DUPLICABLE, 0, 0, false, Memory::None)?;
    let mut draft = program.assembly()?;
    draft.operand(Kind::Pointer)?;
    draft.operand(Kind::Vector)?;
    draft.text("movntps [")?;
    draft.operand_use(0)?;
    draft.text("],")?;
    draft.operand_use(1)?;
    let template = draft.finish(asm::SIDE_EFFECT, 0, 0)?;
    let signature = program.signature(&[Type::Ptr, Type::Ptr], Type::Void, false)?;
    let symbol = program.declare("rg_library_stream".into(), signature, false)?;
    let mut body = program.body_builder();
    let entry = body.block()?;
    body.enter(entry)?;
    let slot = body.object_slot(MemorySpan {
        bytes: 16,
        align: 16,
    })?;
    let pointer = body.address(Operand::Slot(slot))?;
    body.asm(snapshot, &[parameter(Type::Ptr, 1), pointer])?;
    body.asm(template, &[parameter(Type::Ptr, 0), pointer])?;
    body.terminate(Terminator::Return(None))?;
    program.define(symbol, body.finish()?)
}

/// Vector payload storage and register views are independent: the narrow
/// case captures only 128 bits while its invocation names a YMM register.
fn vector_copy(
    program: &mut Program,
    name: &str,
    kind: Kind,
    view: u16,
) -> Result<(), &'static str> {
    let mut draft = program.assembly()?;
    draft.binding(kind, None, true, true, false)?;
    draft.text(if view == 512 {
        "vmovdqa32 "
    } else {
        "vmovdqa "
    })?;
    draft.operand_use_view(0, view)?;
    draft.text(", ")?;
    draft.operand_use_view(0, view)?;
    let template = draft.finish_memory(asm::DUPLICABLE, 0, 0, false, Memory::None)?;
    let signature = program.signature(&[Type::Ptr, Type::Ptr], Type::Void, false)?;
    let symbol = program.declare(name.into(), signature, false)?;
    let mut body = program.body_builder();
    let entry = body.block()?;
    body.enter(entry)?;
    body.asm(
        template,
        &[parameter(Type::Ptr, 1), parameter(Type::Ptr, 0)],
    )?;
    body.terminate(Terminator::Return(None))?;
    program.define(symbol, body.finish()?)
}

/// `rg_library_turn`: the two floats after the integer. The floats share the
/// first eightbyte of the parameter, which C passes in an SSE register.
fn turn(program: &mut Program) -> Result<(), &'static str> {
    let signature = program.object_signature(
        &[pair(Class::Sse, Class::Integer)],
        pair(Class::Integer, Class::Sse),
        false,
    )?;
    let symbol = program.declare("rg_library_turn".into(), signature, false)?;
    let mut body = program.body_builder();
    let entry = body.block()?;
    body.enter(entry)?;
    let incoming = body.object_slot(PAIR)?;
    let outgoing = body.object_slot(PAIR)?;
    let from = body.address(Operand::Slot(incoming))?;
    let argument = ObjectValue {
        operand: Operand::Parameter(0),
        span: PAIR,
    };
    body.copy_object_value(program, signature, argument, from)?;
    let first = body.load(Type::F32, from.operand)?;
    let four = body.integer(Type::I64, 4)?;
    let at = body.index(Type::U8, from, four)?;
    let second = body.load(Type::F32, at.operand)?;
    let eight = body.integer(Type::I64, 8)?;
    let at = body.index(Type::U8, from, eight)?;
    let integer = body.load(Type::I64, at.operand)?;
    let to = body.address(Operand::Slot(outgoing))?;
    body.store(integer, to.operand)?;
    let eight = body.integer(Type::I64, 8)?;
    let at = body.index(Type::U8, to, eight)?;
    body.store(first, at.operand)?;
    let twelve = body.integer(Type::I64, 12)?;
    let at = body.index(Type::U8, to, twelve)?;
    body.store(second, at.operand)?;
    let result = ObjectValue {
        operand: Operand::Slot(outgoing),
        span: PAIR,
    };
    body.return_object(program, signature, result)?;
    let function = body.finish()?;
    program.define(symbol, function)
}

/// `exit` and `rg_library_stop`, both `return=noreturn`.
fn stop(program: &mut Program) -> Result<(), &'static str> {
    let signature = program.signature(&[Type::I32], Type::Void, true)?;
    let exit = program.declare("exit".into(), signature, false)?;
    let symbol = program.declare("rg_library_stop".into(), signature, false)?;
    let mut body = program.body_builder();
    let entry = body.block()?;
    body.enter(entry)?;
    body.call(
        signature,
        Type::Void,
        Operand::Symbol(exit),
        &[parameter(Type::I32, 0)],
    )?;
    body.terminate(Terminator::Unreachable)?;
    program.define(symbol, body.finish()?)
}

/// `rg_library_match`: every vector shape the SIMD lowering states.
fn matched(program: &mut Program) -> Result<(), &'static str> {
    let signature = program.signature(&[Type::V128, Type::V128, Type::I32], Type::I32, false)?;
    let symbol = program.declare("rg_library_match".into(), signature, false)?;
    let mut body = program.body_builder();
    let entry = body.block()?;
    body.enter(entry)?;
    let left = parameter(Type::V128, 0);
    let equal = body.vector(Vector::Pcmpeqb, left, Some(parameter(Type::V128, 1)), 0)?;
    let low = body.vector(Vector::FromI32, parameter(Type::I32, 2), None, 0)?;
    let all = body.vector(Vector::Pshufd, low, None, 0)?;
    let both = body.vector(Vector::Pand, equal, Some(all), 0)?;
    let mask = body.vector(Vector::Pmovmskb, both, None, 0)?;
    body.terminate(Terminator::Return(Some(mask)))?;
    program.define(symbol, body.finish()?)
}

/// Keep both undefined addresses reachable through one private pointer pair.
fn weak_bindings(program: &mut Program) -> Result<(), &'static str> {
    let data = program.declare_weak_global(
        "rg_library_weak_data".into(),
        MemorySpan { bytes: 1, align: 1 },
        GlobalStorage::Writable,
    )?;
    let signature = program.signature(&[Type::I32], Type::I32, false)?;
    let function = program.declare_weak_function("rg_library_weak_function".into(), signature)?;
    let cell = program.internal_global(
        "rg_library_weak_cells".into(),
        PAIR,
        GlobalStorage::Writable,
    )?;
    program.initialize(
        cell,
        &[
            Item::Address {
                target: Target::Global(data),
                addend: 0,
            },
            Item::Address {
                target: Target::Function(function),
                addend: 0,
            },
        ],
    )?;
    let signature = program.signature(&[], Type::Ptr, false)?;
    let accessor = program.declare("rg_library_weak_addresses".into(), signature, false)?;
    let mut body = program.body_builder();
    let entry = body.block()?;
    body.enter(entry)?;
    let address = body.address(Operand::Global(cell))?;
    body.terminate(Terminator::Return(Some(address)))?;
    program.define(accessor, body.finish()?)
}

fn masked_load(program: &mut Program) -> Result<(), &'static str> {
    let mut draft = program.assembly()?;
    draft.binding(Kind::Vector, None, true, true, false)?;
    draft.operand(Kind::Mask(Type::U16))?;
    draft.operand(Kind::Pointer)?;
    draft.text("vmovss ")?;
    draft.operand_use(0)?;
    draft.text("{")?;
    draft.operand_use(1)?;
    draft.text("}, [")?;
    draft.operand_use(2)?;
    draft.text("]")?;
    let template = draft.finish_memory(asm::DUPLICABLE, 0, 0, false, Memory::ReadOnly)?;
    let arguments = [Type::Ptr, Type::Ptr, Type::U16, Type::Ptr];
    let signature = program.signature(&arguments, Type::Void, false)?;
    let symbol = program.declare("rg_library_masked_load".into(), signature, false)?;
    let mut body = program.body_builder();
    let entry = body.block()?;
    body.enter(entry)?;
    let values = [
        parameter(Type::Ptr, 0),
        parameter(Type::Ptr, 1),
        parameter(Type::U16, 2),
        parameter(Type::Ptr, 3),
    ];
    body.asm(template, &values)?;
    body.terminate(Terminator::Return(None))?;
    program.define(symbol, body.finish()?)
}

fn module() -> Result<Program, &'static str> {
    let mut program = Program::default();
    answer(&mut program)?;
    let swapped = program.object_signature(
        &[pair(Class::Integer, Class::Sse)],
        pair(Class::Sse, Class::Integer),
        false,
    )?;
    swap(&mut program, swapped)?;
    apply(&mut program, swapped)?;
    tally(&mut program)?;
    stream(&mut program)?;
    masked_load(&mut program)?;
    vector_copy(
        &mut program,
        "rg_library_copy256",
        Kind::WideVector(Wide::Ymm),
        256,
    )?;
    vector_copy(&mut program, "rg_library_narrow_ymm", Kind::Vector, 256)?;
    vector_copy(
        &mut program,
        "rg_library_copy512",
        Kind::WideVector(Wide::Zmm),
        512,
    )?;
    turn(&mut program)?;
    stop(&mut program)?;
    matched(&mut program)?;
    tick(&mut program)?;
    weak_bindings(&mut program)?;
    Ok(program)
}

fn run(folder: &Path) -> Result<(), String> {
    let program = module().map_err(str::to_owned)?;
    let mut text = BufWriter::new(
        fs::File::create(folder.join("text.lowir")).map_err(|error| error.to_string())?,
    );
    writer::write(&program, &mut text).map_err(|error| error.to_string())?;
    drop(text);
    for (form, pic) in [("fixed", false), ("pic", true)] {
        let view = folder.join(format!("{form}.lowir"));
        let request = Request {
            level: 0,
            pic,
            view: Some(&view),
        };
        let object = rustgm_backend_bridge::compile(&program, &request)
            .map_err(|failure| format!("{form}: {failure}"))?;
        let bytes = object.bytes().map_err(|failure| failure.to_string())?;
        fs::write(folder.join(format!("{form}.o")), bytes).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [folder] = arguments.as_slice() else {
        eprintln!("USAGE: rustgm-bridge-handshake FOLDER");
        return ExitCode::from(2);
    };
    match run(Path::new(folder)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("rustgm-bridge-handshake: {message}");
            ExitCode::FAILURE
        }
    }
}
