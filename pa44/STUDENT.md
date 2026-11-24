# RustGM Programming Assignment 44 — Frontend acceptance

This is a draft handout. The assignment's grading suite will accompany
its release.

Qualify the complete configured Rust 1.98.0 frontend for Linux x86-64.
Exercise combinations of language features through checked LowIR, native
execution, and cross-crate artifacts. The work carries the broader
requirements deferred by the focused PA25–PA30 assignments.

## Carried requirements

| Earlier assignment | Features to exercise here |
| --- | --- |
| PA25 | Named, generic, and associated initializers, const calls and closures, constant patterns and lengths, symbolic storage and provenance, and slice and unsized constant values |
| PA26 | Source-bound operators and iteration, question-mark and residual control flow, built-in derives, formatting, and panic hooks |
| PA27 | Constant raw accesses, typed validity over byte-backed values, interior mutation, and raw slices and reconstructed wide values |
| PA28 | The remaining intrinsic, atomic, volatile, and numeric operations, symbolic fragment transport, and actual source layout and transmute calls |
| PA29 | Complete interface and value artifacts, generic and provided bodies, source attribute policies, and proof-aware executable transport |
| PA30 | Foreign signatures, storage, arrays, and variadic metadata, opaque foreign types, and the supported representation and ABI rules |

Check combinations of these features with editions, macros, ownership,
unwinding, and async behavior from the other assignments. Include
deterministic source and debug locations and compare both information
retained in artifacts and behavior after importing them.

## Tool surface

Run the full selected crate-build and LowIR pipeline. Build original
`core`, `alloc`, `std`, and the required upstream crate graphs.
Compile each selected target Rust unit with RustGM and link it through
the pinned CPPGM backend.

## Check your implementation

Run the preceding assignment suites and compile and execute complete
upstream consumers. Compare repeat builds and artifact round trips.
Exercise cross-feature failures and check their diagnostics. Measure
time, memory, and scaling on complete graphs as well as focused inputs.
The acceptance run requires working implementations throughout.
[PA45](../pa45/README.md) then checks the separate self-hosting contract.
