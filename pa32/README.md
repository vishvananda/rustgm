# RustGM Programming Assignment 32 — Pinned core bootstrap

The graded suite for this assignment is pending.

Build the complete upstream `core` crate for Rust 1.98.0 on Linux x86-64
with aborting panics. Use the original sources at
`88d9e12ae178fab0fb5cc050a94da85685d449ea`.

## Before starting core

Complete the following parts of the bootstrap path.

- [PA31](../pa31/README.md)'s source bindings and runtime integration.
- The source and generic-body metadata needed to export library APIs
  and instantiate them in fresh clients. [PA29](../pa29/README.md)
  introduces the declaration format.
- The core prerequisites for layout, target features, and assembly below.

## Prepare the core build

Use the crate's Rust 2024 edition and Linux x86-64 panic-abort configuration.
Capture its selected configuration flags, environment values, and generated
files and pass them to the target compiler. Preserve the crate identity,
feature selection, and explicit lint settings in the build request.
[PA34](../pa34/README.md) extends this preparation to mixed-edition graphs,
build scripts, and renamed dependencies.

## Required target support for core

Compile the active stdarch and portable SIMD source as part of the crate.
Implement its `repr(simd)` layout, selected ABI and real target-feature
requirements. Qualify the original `_mm_stream_ps` path through CPPGM
with its generated template, full-width vector input and typed memory
effects. Check the resulting non-temporal store's instruction and its
effect on all 128 input bits and the surrounding bytes.

Retain typed assembly operands, effect and safety facts across expansion,
lowering, and any library artifact consumed by a separate crate. Evaluate
operands in source order. Check target features, alignment and register
classes before lowering the selected operations. Keep the broader
target features and assembly forms for the follow-on track.

## Diagnostics used by core

Complete [PA31's source attributes and diagnostics](../pa31/README.md#source-declarations)
before building the standalone library. Apply the crate's selected edition
and honor the active scoped lint levels and expect attributes. Check
`ffi_unwind_calls` for the panic-abort profile. A function defined in the
crate stops a foreign unwind at its boundary. The follow-on unwinding
assignment extends this check to callers that can propagate an unwind.

Use the checked operand and result types to detect explicit casts between raw
pointers and integers. Honor the crate's `deny(implicit_provenance_casts)` and
the local `allow` and `expect` attributes on the pointer APIs. Complete each
expectation using an actual cast in its scope. Use [PA28's const transmute
check](../pa28/README.md#memory-operations) for the corresponding expectations
on pointer-to-integer transmutes.

## Build the library

Use the pinned sources and submodules without edits. Retain the target
configuration, generated inputs, and host tool identities so the build can
be repeated. Enable the actual internal features used by the pinned
configuration, including const traits, specialization, compiler builtins,
and layout rules.

Compile the selected stdarch and portable SIMD modules as part of the crate.
They need their actual type and target rules, including `repr(simd)` and
target-specific configuration such as `target_has_atomic_load_store`.
Resolve selected calls against the real source definitions.

## Tool surface

Build every target Rust unit with RustGM and pass LowIR to CPPGM for native
code generation. Produce complete `core` metadata and linkable objects.
The sysroot identities and artifacts must be reproducible. Share completed
dependency and proof work across the graph.

## Check your implementation

Compile separate consumers against the resulting metadata and objects.
Exercise real `Option` and `Result` methods, slices, iteration, formatting,
conversions, constant operations, pointer and layout APIs, and trait
selection. Check rejected bindings and invalid type or ownership uses as
well. Link and run the executable consumers. A successful build includes
the full crate and independently compiled consumers.

Build `alloc` in [PA33](../pa33/README.md) once `core` and its consumers
pass. With the [PA34](../pa34/README.md) driver, you can also run the
core-only upstream checks in [PA35](../pa35/README.md).
