# RustGM Programming Assignment 40: Edition and attribute closure

Status: **planned**. This is a buildout brief, not a released grading contract.
Work prerequisites: [ordered dependency units](../docs/assignment-order.md). The source profile is Rust 1.98.0, initially edition 2024
and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.
See the [library bootstrap plan](../docs/library-bootstrap-roadmap.md) for
implementation priority, source pins and the migration of unreleased numbers.

## Required implementation

Pinned edition differences and remaining mandatory attribute semantics.

PA34 already owns the 2021/2024 behavior needed by the first crate graph. Complete
2015/2018 and remaining edition/attribute semantics here. Required internal source
features cannot be deferred past an earlier library that actually exercises them.

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

- 2015/2018/2021/2024 differences.
- Capture.
- Drop scope.
- Fragment editions.

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

The unpublished [finite lint expectation checkpoint](../docs/pa40-lint-expectations-checkpoint.md)
connects genuine local deprecation events to owner/list identities, effective
expectations and completed request scopes. Its source contracts include
independent positive and opposing controls; full lint/edition and release
requirements remain open.

The unpublished [hidden glob reexport checkpoint](../docs/pa40-hidden-glob-reexports-checkpoint.md)
adds completed resolver events, exact public import chains and namespace-sensitive
expectations, with the next unchanged original-core boundary recorded explicitly.

The unpublished [ineffective unstable trait impl checkpoint](../docs/pa40-ineffective-unstable-impl-checkpoint.md)
adds genuine resolved header events, exact unstable-attribute witnesses and
header-only expectation completion, with imported transport remaining explicit.
