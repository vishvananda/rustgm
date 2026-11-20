# RustGM Programming Assignment 44: Frontend acceptance

Status: **planned**. This is a buildout brief, not a released grading contract.
Work prerequisites: [ordered dependency units](../docs/assignment-order.md). The source profile is Rust 1.98.0, initially edition 2024
and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.
See the [library bootstrap plan](../docs/library-bootstrap-roadmap.md) for
implementation priority, source pins and the migration of unreleased numbers.

## Carried requirements

PA25–PA30 are closed at their finite contracts. The unreleased units and the
[requirement rows](../docs/assignment-requirements.json) those contracts defer
are owed here. `assignments.json` records each closed assignment's deferred list
and the matching `carries` entry of this assignment; the plan check rejects a
closure whose deferred unit or row is not carried here, and rejects a complete
PA44 while a carried unit is unreleased. A requirement can move here. It cannot
disappear.

| From | Units | Requirement rows |
| --- | --- | --- |
| [PA25](../pa25/README.md#graded-contract) | `const-memory`, `const-callables`, `source-runtime` | 25.3, 25.4, 25.5, 25.6, 25.7, 25.8, 25.9 |
| [PA26](../pa26/README.md#graded-contract) | `hook-closure`, `format-derive`, `generic-transport`, `source-runtime` | 26.1, 26.3, 26.6, 26.7, 26.8, 26.9, 26.10 |
| [PA27](../pa27/README.md#graded-contract) | `raw-validity`, `const-memory` | 27.4, 27.5, 27.6, 27.7 |
| [PA28](../pa28/README.md#graded-contract) | `intrinsic-closure`, `const-memory`, `generic-transport`, `const-callables`, `source-runtime` | 28.1, 28.4, 28.5, 28.6 |
| [PA29](../pa29/README.md#graded-contract) | `value-metadata`, `provided-profile`, `generic-transport` | 29.2, 29.3, 29.4, 29.5, 29.6 |
| [PA30](../pa30/README.md#graded-contract) | `ffi-closure`, `value-metadata`, `generic-transport` | 30.5, 30.6 |

## Required implementation

Complete feature census, cross-feature corpus, debug locations and scaling.

Include the complete required upstream core/alloc/std and real-crate acceptance
graph, not only isolated language fixtures. Reproducible self-hosting is the
required [PA45](../pa45/README.md) capstone. It has its own contract and does
not redefine this frontend acceptance contract.

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

- All previous stages.
- Round trips.
- No stubs.
- Deterministic output.
- Performance.
- Upstream library and crate acceptance.

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
