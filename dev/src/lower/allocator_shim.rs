//! rustc 1.98.0's allocator shim, which the final crate of a program gets
//! when any of its crates declares `#![needs_allocator]`, as alloc does
//! (`rustc_codegen_ssa::base::allocator_shim_contents`):
//!
//! - A `#[global_allocator]` defines the `__rust_alloc` family itself; rustc
//!   finds the global allocator by that internal-symbol item. Otherwise a
//!   `#![default_lib_allocator]` crate (std) gets forwarders to its
//!   `__rdl_*` functions, and otherwise the program has rustc's error.
//! - Without an `#[alloc_error_handler]`, `__rust_alloc_error_handler`
//!   forwards to alloc's `__rdl_alloc_error_handler`.
//! - `__rust_no_alloc_shim_is_unstable_v2` is always defined, empty.
//!
//! A forwarder keeps the signature of the program's declaration of it, from
//! alloc's `extern "Rust"` block, or else rustc's lowering of its
//! `AllocatorTy`s: a `Layout` is a `usize` size and alignment. Its target
//! keeps its own. alloc passes a `Layout`'s alignment as `Alignment`, which
//! this compiler passes by address unless it is a scalar carrier, while
//! std's `__rdl_*` take a `usize`: the forwarder then loads the `usize`.
use crate::{
    lowir::{
        program::{Operand, Program, SignatureId, Terminator, Value},
        vocabulary::Type,
    },
    support::linkage::internal_symbol,
};

/// rustc's `ALLOCATOR_METHODS`, their arguments lowered as `AllocatorTy`,
/// with the position of each `Layout`'s alignment.
const METHODS: [(&str, &[Type], Type, usize); 4] = [
    ("alloc", &[Type::I64, Type::I64], Type::Ptr, 1),
    ("dealloc", &[Type::Ptr, Type::I64, Type::I64], Type::Void, 2),
    (
        "realloc",
        &[Type::Ptr, Type::I64, Type::I64, Type::I64],
        Type::Ptr,
        2,
    ),
    ("alloc_zeroed", &[Type::I64, Type::I64], Type::Ptr, 1),
];

/// rustc's error for a program that needs an allocator and has none.
const MISSING: &str = "no global memory allocator found but one is required; link to std or add `#[global_allocator]` to a static item that implements the GlobalAlloc trait";

/// The crate-root requests that select the shim, read once loading
/// validated them.
#[derive(Clone, Copy, Default)]
pub(super) struct Requests {
    /// A crate declares `#![needs_allocator]`.
    needs: bool,
    /// A crate declares `#![default_lib_allocator]`.
    default_library: bool,
}
impl Requests {
    pub(super) fn read(package: &crate::resolve::package::Package) -> Result<Self, &'static str> {
        Ok(Self {
            needs: package.any_root_word("needs_allocator")?,
            default_library: package.any_root_word("default_lib_allocator")?,
        })
    }
}

/// Define the shim's functions in the finished program.
/// # Errors
/// rustc's error without an allocator, and invalid or incompatible
/// declarations of the shim's symbols.
pub(super) fn install(program: &mut Program, requests: Requests) -> Result<(), String> {
    if !requests.needs {
        return Ok(());
    }
    if !defines(program, "__rust_alloc")? {
        if !requests.default_library {
            return Err(MISSING.to_owned());
        }
        for (method, parameters, result, align) in METHODS {
            let name = format!("__rust_{method}");
            let shape = program.signature(parameters, result, false)?;
            let target = format!("__rdl_{method}");
            forward(program, (&name, &target), shape, Some(align))?;
        }
    }
    if !defines(program, "__rust_alloc_error_handler")? {
        let shape = program.signature(&[Type::I64, Type::I64], Type::Void, true)?;
        let names = ("__rust_alloc_error_handler", "__rdl_alloc_error_handler");
        forward(program, names, shape, None)?;
    }
    let shape = program.signature(&[], Type::Void, false)?;
    let (symbol, _) = declared(program, "__rust_no_alloc_shim_is_unstable_v2", shape)?;
    let mut builder = program.body_builder();
    let entry = builder.block()?;
    builder.enter(entry)?;
    builder.terminate(Terminator::Return(None))?;
    program.define(symbol, builder.finish()?)?;
    Ok(())
}

/// Whether some crate of the program defines the internal symbol `name`.
fn defines(program: &Program, name: &str) -> Result<bool, &'static str> {
    Ok(program
        .function(&internal_symbol(name)?)
        .and_then(|symbol| program.symbols.get(symbol.0 as usize))
        .is_some_and(|symbol| symbol.body.is_some()))
}

/// The internal symbol `name`, declared with the program's own signature for
/// it or else `shape`, and that signature.
fn declared(
    program: &mut Program,
    name: &str,
    shape: SignatureId,
) -> Result<(crate::lowir::program::SymbolId, SignatureId), &'static str> {
    let name = internal_symbol(name)?;
    let signature = program
        .function(&name)
        .and_then(|symbol| program.symbols.get(symbol.0 as usize))
        .map_or(shape, |symbol| symbol.signature);
    Ok((program.declare(name, signature, false)?, signature))
}

/// Define `name` as a call of `target` with its own arguments, returning the
/// result, or unreachable after a call that does not return. Only a
/// `Layout`'s alignment at `align` may differ, passed by address to a target
/// that takes the `usize`.
fn forward(
    program: &mut Program,
    (name, target): (&str, &str),
    shape: SignatureId,
    align: Option<usize>,
) -> Result<(), String> {
    let (symbol, signature) = declared(program, name, shape)?;
    let (callee, callee_signature) = declared(program, target, signature)?;
    let differ =
        || format!("allocator shim cannot forward `{name}` to `{target}`: their signatures differ");
    let (from, to) = (
        program.scalar_signature(signature),
        program.scalar_signature(callee_signature),
    );
    let (Ok(from), Ok(to)) = (from, to) else {
        return Err(differ());
    };
    if (from.result, from.noreturn, from.variadic) != (to.result, to.noreturn, to.variadic) {
        return Err(differ());
    }
    let (result, noreturn) = (from.result, from.noreturn);
    let (from, to) = (
        program.parameters(signature)?.to_vec(),
        program.parameters(callee_signature)?.to_vec(),
    );
    if from.len() != to.len() {
        return Err(differ());
    }
    let mut builder = program.body_builder();
    let entry = builder.block()?;
    builder.enter(entry)?;
    let mut arguments = Vec::new();
    arguments
        .try_reserve_exact(from.len())
        .map_err(|_| "allocator shim argument allocation failed")?;
    for (index, (&have, &want)) in from.iter().zip(&to).enumerate() {
        let parameter = Operand::Parameter(
            u32::try_from(index).map_err(|_| "allocator shim parameter space exhausted")?,
        );
        arguments.push(if have == want {
            Value {
                ty: have,
                operand: parameter,
            }
        } else if align == Some(index) && (have, want) == (Type::Ptr, Type::I64) {
            builder.load(Type::I64, parameter)?
        } else {
            return Err(differ());
        });
    }
    let value = builder.call(
        callee_signature,
        result,
        Operand::Symbol(callee),
        &arguments,
    )?;
    builder.terminate(if noreturn {
        Terminator::Unreachable
    } else {
        Terminator::Return(value)
    })?;
    program.define(symbol, builder.finish()?)?;
    Ok(())
}
