# RustGM Programming Assignment 43: Async ownership

Status: **planned**. This is a buildout brief, not a released grading contract.
Work prerequisites: [ordered dependency units](../docs/assignment-order.md). The source profile is Rust 1.98.0, initially edition 2024
and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.
See the [library bootstrap plan](../docs/library-bootstrap-roadmap.md) for
implementation priority, source pins and the migration of unreleased numbers.

## Required implementation

Borrowing across suspension, pinning, cancellation and advanced async forms.

LIB-154 binds this work to original core's AsyncDrop and compiler-replaced
`async_drop_in_place`, alongside PA31 role authority and PA42 async signatures.
Preserve actual Pin receiver, Copy/destructor exclusion, completion, cancellation,
loans and destruction order. The upstream helper's written placeholder body is
not its real implementation. Declaration-only acceptance does not qualify any
of these consumers; see the [original probe](../docs/pa31-function-pointer-checkpoint.md).

The [Pin source packet](../pa31/tests/spec/pin-source/README.md) implements the
nominal/ordinary constructor producer with actual body, borrow/const and field
drop controls. It grants no Copy or destructor suppression. This assignment
still requires real receiver/Deref/LegacyReceiver proof, Unpin, unsafe projection,
future state ownership and cancellation. The real AsyncDrop source frontier is
unchanged by establishing that prerequisite.

The maintainer implements compiler responsibilities in `dev/src/` and introduces
typed interfaces before text views. Follow [spec.md](../spec.md), the pinned Rust
Reference and CPPGM's efficient representation principles. Parse once, retain
completed facts and avoid speculative or repeated whole-graph work. The compiler
implementation remains idiomatic safe Rust; strict Clippy all/pedantic and the
syntax-aware file/architecture audit are required.

## Tool surface

Extend the existing `dev/rustgm --emit-lowir` view and explicit crate-build interfaces.
Earlier views remain available. Freeze deterministic ordering, diagnostics,
source/configuration identities and any new build-summary text contract before
activation. Whole-library acceptance includes actual linked execution through
the configured CPPGM backend; text output alone is insufficient.

## Tests to develop with the reference

- Cancel at each state.
- Drop order.
- Self references.
- Async closures/traits.

Place each fixture in the earliest assignment owning all features it exercises.
Keep original upstream bytes, attribution and checksums. Declare expected outcomes
before generating oracles; use pinned rustc independently and retain negative
controls. Register the complete selected dependency graph and native consumers,
not just reducers or successful submodules. Never hand-edit generated references.

## Completion and review

Publish the semantic/lowering boundary, nonempty `tests/manifest.json`, text
formats and generated references together after independent outcome review.
Until then, this assignment must fail explicitly and stays outside pass totals.
Run affected focused source/native controls during development. At publication,
run the required coherent checkpoint checks from
[TESTING_AND_REFERENCES.md](../TESTING_AND_REFERENCES.md), including prior active
suites, strict Clippy, file/architecture audits, harness checks and affected
performance measurements. Use the resource/storage guards and scoped cleanup.
Do not activate a stage merely because a reduced example or an empty suite passes.
