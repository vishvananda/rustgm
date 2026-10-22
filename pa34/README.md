# RustGM Programming Assignment 34: Edition and crate build integration

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA33. The source profile is Rust 1.98.0, initially edition 2024
and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.
See the [library bootstrap plan](../docs/library-bootstrap-roadmap.md) for
implementation priority, source pins and the migration of unreleased numbers.

## Required implementation

Provide reproducible crate-build orchestration for Rust 2021 and 2024, retaining
per-crate editions through expansion and metadata. Resolve the selected manifest,
lockfile, dependency renames, features and target configuration used by the
acceptance graph. General 2015/2018 and remaining edition closure stay at PA40;
no edition behavior required by this graph may be deferred.

Execute build scripts and preserve their generated source, cfg, environment and
link directives with complete provenance. Model host and target artifacts
separately. An explicitly pinned host toolchain may bootstrap build-time tools;
every target Rust library/dependency must compile through RustGM. This is not a
claim of rustc-private metadata compatibility or a drop-in Cargo/rustc interface.
Freeze the supported driver protocol before publishing its grading contract.

Develop this path alongside PA31/PA32 using small separate crates first. Its
implementation need not wait for alloc: core-only itoa/serde_core acceptance
should become available as soon as core and their build prerequisites work.
The cumulative assignment prerequisite and release checks remain unchanged.

Test original Serde build-script outputs and serde_json configuration. Record
source/package checksums, all transitive versions, compiler/runtime identities,
features, target facts and output hashes. Offline replay after source acquisition
must reproduce the selected graph without hidden target fallbacks.

Keep orchestration outside dev/src frontend phases. Cache completed per-crate
work by explicit inputs; do not rescan or reparse dependency sources for each
consumer. Preserve the existing three-job controller and scoped cleanup.

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

- Per-crate editions.
- Feature and dependency resolution.
- Host versus target artifacts.
- Build script outputs.
- Reproducible build provenance.

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
