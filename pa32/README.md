# RustGM Programming Assignment 32: Pinned core bootstrap

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA31. The source profile is Rust 1.98.0, initially edition 2024
and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.
See the [library bootstrap plan](../docs/library-bootstrap-roadmap.md) for
implementation priority, source pins and the migration of unreleased numbers.

## Required implementation

Compile the entire upstream core crate for the declared Rust 1.98.0 Linux
x86-64 panic-abort profile, using original source bytes at revision
`88d9e12ae178fab0fb5cc050a94da85685d449ea`. Freeze source/submodule checksums,
target cfg, required generated configuration and host-tool provenance first.
A partial module build or reduced imitation of core is not completion.

Implement every active internal feature used by this source profile, including
its const-trait, specialization, layout and compiler-builtin protocols where
selected. Inventory actual failures rather than assuming stable source-language
support is sufficient. PA31 bindings and PA25–PA30 typed contracts must compose
with ordinary upstream definitions and exported generic bodies.

Core directly imports stdarch/portable-simd modules. Implement their active
parsing, typing, repr(simd), layout and target-feature contracts as required by
the pinned source; unstable public API status does not remove those modules.
Freeze internal cfg facts, including target_has_atomic_load_store, rather than
assuming stable rustc --print cfg reports the complete compiler configuration.
Track source binding, internal-feature closure and full-crate/consumer execution
as separate checked milestones within this assignment.

Build independent consumer crates against RustGM's core metadata and objects.
Exercise real Option/Result methods, slices, iterators, formatting, conversions,
const operations, pointer/layout APIs and trait selection. Include negative
binding/type/ownership controls. All target Rust code uses RustGM, with CPPGM
only behind the established LowIR boundary; rustc is an independent oracle.

Require deterministic sysroot identities and bounded shared dependency/proof
work. Record whole-crate compile time and peak memory alongside scaling inputs.
A needed target operation must be qualified now, even if broad target closure
belongs to PA41.

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

- Complete core source build.
- Real generic bodies across crates.
- Layout and const trait protocols.
- Slices iterators and formatting.
- Deterministic sysroot metadata.

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
