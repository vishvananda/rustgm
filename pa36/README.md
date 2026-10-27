# RustGM Programming Assignment 36: Std and platform bootstrap

Status: **planned**. This is a buildout brief, not a released grading contract.
Work prerequisites: [ordered dependency units](../docs/assignment-order.md). The source profile is Rust 1.98.0, initially edition 2024
and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.
See the [library bootstrap plan](../docs/library-bootstrap-roadmap.md) for
implementation priority, source pins and the migration of unreleased numbers.

## Required implementation

Compile upstream std from the same pinned Rust sources for Linux x86-64 and the
explicit panic-abort profile, with RustGM-built core, alloc and all selected
Rust dependencies. Freeze platform/runtime libraries, build-time configuration
and linker inputs. A hand-implemented std API surface is not acceptance.

Implement the selected platform contracts, TLS/thread and synchronization hooks,
I/O, errors, process/runtime entry and symbol visibility as required by the
original sources. Keep Rust library algorithms in upstream std and native host
services at the declared FFI boundary. Validate target operations with independent
probes before relying on CPPGM support; this brief does not authorize backend
changes.

Build default-feature serde_json and run std-facing consumers, including I/O
and error propagation, in addition to the PA35 alloc-only graph. Test runtime
initialization, thread/TLS behavior, resource destruction and repeatable builds.
Record source/configuration/output provenance and whole-library time/memory.

Full unwind behavior remains required at PA39; this first std profile uses abort.
Any feature actually active in the selected std source must work here, including
compiler-private features otherwise scheduled for broader closure later.

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

- Complete std dependency graph.
- Platform runtime and linkage.
- Thread and TLS behavior.
- I/O and error consumers.
- Default-feature serde_json.

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
