# RustGM Programming Assignment 25: Constant evaluation

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA24. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

## Required implementation

Const functions, statics, promotions and symbolic allocation provenance.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

`dev/rustgm --emit-constants -o OUTPUT SOURCE` and `--count-constants` inspect
completed values through the entry-free checked pipeline. Earlier stage views
remain available. The implemented draft vocabulary, graph identities, ordering,
resource/error contract and independently checked example are documented in
[the constants view draft](../docs/pa25-constants-view.md). Review its complete
fixtures and generated oracles before declaring this assignment active.
The [sequence execution checkpoint](../docs/pa25-sequences.md) records implemented
array-backed slice operations, scaling invariants and remaining sequence gaps.
The [DST field-metadata checkpoint](../docs/pa25-dst-tail-metadata.md) forwards
already-checked metadata to final unsized fields while retaining canonical
physical backing types, reference permissions and separate logical bounds.

## Tests to develop with the reference

const restrictions; pointers; recursive calls; limits; const generics.

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
