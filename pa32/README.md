# RustGM Programming Assignment 32 — Pinned core bootstrap

The strict self-hosting graph has built the original `core` from source.
This assignment qualifies its separate artifact and fresh consumers.
The graded suite is pending.

Build the complete upstream `core` crate for Rust 1.98.0 on Linux x86-64
with aborting panics. Use the original sources at
`88d9e12ae178fab0fb5cc050a94da85685d449ea`.

## Before starting core

Complete the following parts of the bootstrap path.

- [PA31](../pa31/README.md)'s source bindings and runtime integration.
- The source and generic-body metadata needed to export library APIs
  and instantiate them in fresh clients. [PA29](../pa29/README.md)
  introduces the declaration format.
- [PA34](../pa34/README.md)'s crate-build preparation, including the selected
  edition, target configuration, and generated inputs.
- The core prerequisites for layout, target features, and assembly below.

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

The core-only consumer milestone in [PA35](../pa35/README.md) can follow
this assignment. Then build `alloc` in [PA33](../pa33/README.md).
