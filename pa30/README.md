# RustGM Programming Assignment 30: Foreign interfaces

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA29. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

## Required implementation

C ABI, repr attributes, linkage and imported/exported declarations.

The maintainer callable checkpoint retains `Abi::{Rust,C}` and a variadic bit
independently of function safety and constant eligibility. Omitted `extern`
remains distinct syntax and selects C. Fixed-arity scalar C definitions and
function pointers use ordinary selected calls. Top-level safe-to-unsafe
coercions retain a checked lifetime proof view; nested callable equality and
calling-convention identity remain strict. Draft source outcomes are in
`tests/convention-outcomes.json`; this does not activate the assignment.

The maintainer scalar-import checkpoint retains actual foreign-block ownership
through macro expansion, inherits ABI and default unsafe permission, and emits
real declarations for selected imports. Five C-linked native controls cover
scalar imports, callbacks and ordinary/generic/constant pointer demand paths.
Draft outcomes are in `tests/import-outcomes.json`.

Foreign statics, linkage attributes and export roots, C aggregate classification
and variadic calls remain unfinished. The reviewed optional object backend
profile is selected explicitly; its native tests use the test-only
`object-interop-tests` Cargo feature and fail if the profile is missing. PA30's
release target and CI must configure that profile and run those tests before
activation. See `docs/pa30-import-checkpoint.md` and
`docs/pa30-callable-checkpoint.md` for qualification and remaining boundaries.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

## Tests to develop with the reference

repr(C); aggregate ABI; callbacks; exported globals; C interoperability.

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
