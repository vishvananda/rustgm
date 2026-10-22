# RustGM Programming Assignment 31: Compiler/library bindings

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA30. The source profile is Rust 1.98.0, initially edition 2024
and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.
See the [library bootstrap plan](../docs/library-bootstrap-roadmap.md) for
implementation priority, source pins and the migration of unreleased numbers.

## Required implementation

Register source-declared language items, diagnostic items and intrinsic
identities through validated attributes and ordinary crate ownership. Validate
signatures, generic parameters, duplicate/missing items and feature gates. One
typed registry must drive semantic selection, const eligibility, capability
queries and lowering; path spelling alone grants no compiler privilege.

Inventory the active compiler contract of Rust 1.98.0 source revision
`88d9e12ae178fab0fb5cc050a94da85685d449ea`: `no_core`, internal attributes,
builtin macros/derives, layout properties and unstable language features.
Classify attributes by semantic effect; never silently ignore an active semantic
requirement. In this revision, `SizedTypeProperties` supplies layout constants
and atomic aliases use `Atomic<T: AtomicPrimitive>`; test the real protocols.

Define explicit source-sysroot selection and crate identity. Replace bootstrap
library factories with real declarations without duplicate identities or a
synthetic fallback. Ordinary Option/Result/Iterator methods belong to upstream
core. PA26 retains ownership of desugaring, derives and formatting semantics;
this assignment connects them to declarations loaded from source.

Bind Tuple, FnOnce and its associated Output declaration before admitting
intrinsics whose signatures depend on them. Preserve rust-call tuple spreading
and ordinary versus const-callability obligations. Qualify const_eval_select
with source, ownership and native controls; accepted runtime function pointers
and upstream compiler ICEs must not be mislabeled as language rejections. See
the [pinned callable evidence](../docs/library-bootstrap-inventory.md#callable-selection-followup).

Establish the minimal panic-abort entry/runtime boundary and no_core declaration
packages. Keep general unwinding at PA39. Host/build configuration lives outside
the frontend library; no environment reads enter semantic hot paths.

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

- Lang-item identity and signatures.
- Intrinsic declarations and const eligibility.
- Builtin expansion and capability answers.
- Duplicate and forged bindings.
- Panic-abort runtime boundary.

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
