# RustGM Programming Assignment 28: Intrinsics and atomics

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA27. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

## Required implementation

Memory and numeric intrinsics, atomics and volatile operations.

The initial capability checkpoint uses the existing pinned LowIR vocabulary and
backend without extending either. The independent inputs in `tests/backend/`
qualify scalar volatile loads and stores for integer widths 8 through 128,
`f32`, `f64` and pointers. They also qualify `u32` atomic load, store, exchange,
compare-exchange success and failure (including the expected-value update), and
add-and-fetch. Run the ignored `intrinsic_backend` maintainer test with the
configured backend and GNU objdump to execute these controls and inspect the
atomic machine instructions. This establishes backend capabilities only.

The initial frontend checkpoint implements canonical `core::ptr` and `std::ptr`
`read_volatile` and `write_volatile` for primitive scalars and thin raw-pointer,
reference and function-pointer values. Imports and reified unsafe function
pointers retain their canonical callable identity. Reification shares a shim per
selected instance; unused direct reads retain their volatile access. Ordinary
loads and stores keep their existing LowIR spelling.

Typed intrinsic descriptors track unsafe permission independently of constant
operation eligibility. Volatile calls require the ordinary unsafe context and
are runtime-only, including skipped branches in ordinary and generic const
functions. Forming an intrinsic function-pointer constant is permitted; calling
that pointer during constant evaluation remains forbidden. As for raw reads and
writes, the source program must meet the selected type's validity and alignment
requirements; an unsafe block does not validate a pointer or its contents.

Aggregate, zero-sized and wide-pointer volatile transfers remain explicitly
unsupported. Ordinary object copies or zero initialization cannot substitute for
volatile accesses or padding. Atomic library methods and the remaining intrinsic
families are later slices. The independent `u32` atomic capability controls do
not claim source-language atomic support.

The draft `tests/manifest.draft.json` declares 53 independently qualified source
outcomes, including nine native comparisons and five valid Rust inputs that
exercise the explicit unsupported boundary. The maintainer tests
`volatile_intrinsics` and `intrinsic_backend` retain their new outputs under
`obj/parallel/pa28/`. These checkpoints do not activate PA28 or provide generated
text references.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

## Tests to develop with the reference

orderings; compare-exchange; casts; u128; memory overlap.

Add spec fixtures, positive/negative controls and cross-feature regressions in
the earliest owning assignment. Use the course reference for text oracles and
pinned rustc for independent language checks. From PA11, execute emitted LowIR
with the configured CPPGM backend. Do not require new future-stage behavior
implicitly. An empty suite or a not-implemented result cannot pass.

## Completion and review

Publish the semantic and lowering boundary, tests/manifest.json and reference
sidecars together. Run all active earlier suites, lint, file/architecture audits,
and affected scaling measurements. Record independent checks before setting
status active in `assignments.json` and adding the stage to the root report.
Student export is a later operation: this repository owns the full solution.
