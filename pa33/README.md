# RustGM Programming Assignment 33: Allocator and alloc bootstrap

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA32. The source profile is Rust 1.98.0, initially edition 2024
and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.
See the [library bootstrap plan](../docs/library-bootstrap-roadmap.md) for
implementation priority, source pins and the migration of unreleased numbers.

## Required implementation

Compile original upstream alloc from the same source pin and against PA32's
RustGM-built core. Freeze and build every selected target dependency; no prebuilt
rustc target library or compiler-written container replaces the upstream code.

Implement the allocator language/runtime contract, layout/alignment, allocation,
zeroed allocation, reallocation, deallocation and allocation-failure behavior.
Distinguish compiler hooks, Rust library code and the explicitly declared native
allocator provider. Qualify symbol identity and C/runtime ABI with PA30 controls.
The pinned allocator shims use Rust ABI, typed Alignment/NonNull arguments and
generated __rust_* symbols plus an OOM handler; C malloc aliases alone do not
satisfy that contract.

Execute real Box, Vec, String and collection consumers, including growth,
zero-sized elements, iterator use, moved values, exact destruction and failure
paths. Record the whole alloc build and linked native outputs, not merely a
handwritten allocator smoke test. Private features required by active source
are prerequisites here rather than promises deferred to later assignments.

Keep bootstrap helpers limited to genuine compiler/runtime interfaces. Container
algorithms, methods and trait implementations remain ordinary upstream Rust.

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

- Allocation and reallocation.
- Box Vec String and collections.
- Allocation failure.
- Drop and ownership across crates.
- Allocator ABI and symbols.

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
