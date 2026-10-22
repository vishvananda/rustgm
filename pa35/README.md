# RustGM Programming Assignment 35: Upstream crate acceptance

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA34. The source profile is Rust 1.98.0, initially edition 2024
and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.
See the [library bootstrap plan](../docs/library-bootstrap-roadmap.md) for
implementation priority, source pins and the migration of unreleased numbers.

## Required implementation

Build original itoa 1.0.18 and serde_core 1.0.229 with defaults disabled, then
serde_json 1.0.151 with defaults disabled and alloc enabled. Freeze the exact
transitive graph, including serde_core, itoa, memchr and zmij, with package/source
checksums and features. These are initial declared targets, not current successes.

Compile every selected target Rust dependency using RustGM's core/alloc sysroot.
Execute separate consumer crates covering integer formatting, Serde visitors,
JSON parsing/serialization, floating and integer boundaries, string escaping,
malformed input, nesting and round trips. Declare outputs and independent pinned
rustc outcomes before generating deterministic text/native oracles.

Test actual target-specific source paths selected by honest cfg facts. Do not
set miri, falsify CPU features, patch out active source or quietly choose older
dependencies to bypass missing functionality. If the frozen backend cannot
represent a required operation, record the exact failing capability and leave
the milestone incomplete pending separately reviewed backend work.

A reduced feature reproducer guides implementation but cannot replace full crate
acceptance. Record complete build summaries and compile-time/RSS measurements.
Procedural Serialize/Deserialize derives are a separate PA38 acceptance target;
the base Serde libraries do not require that feature.

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

- Complete locked dependency graph.
- Visitor and generic API consumers.
- JSON parsing and serialization.
- Escaping malformed input and round trips.
- Compile time and memory scaling.

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
