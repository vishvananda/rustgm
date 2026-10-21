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

The first frontend slice is canonical scalar `read_volatile` and `write_volatile`.
Use exact library definition identities and typed intrinsic descriptors; track
unsafe permission independently of constant-operation eligibility. Volatile
operations are runtime-only, including in skipped const branches. Aggregate
volatile transfers remain explicitly unsupported initially: ordinary object
copies or zero initialization cannot substitute for volatile accesses or padding.
Atomic library methods and the remaining intrinsic families are later slices.

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
