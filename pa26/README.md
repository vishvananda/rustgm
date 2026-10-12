# RustGM Programming Assignment 26: Language and library hooks

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA25. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

## Required implementation

Operators, indexing, iteration, question-mark, built-in derives and formatting.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

## Tests to develop with the reference

desugaring; trait selection; early return; derive bounds; formatting.

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

The builtin boundary includes `option_env!`: its expansion needs the library's
nominal `Option<&'static str>` identity and Some/None constructors. PA8 supplies
the eager argument worklist and explicit environment provider, but must not
fabricate core/std paths while those library identities are unavailable.
Formatting, assertion and panic-related macro hooks similarly need explicit
library/runtime contracts and tests at their owning stage.
