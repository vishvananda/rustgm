# RustGM Programming Assignment 42: Async lowering

Status: **planned**. This is a buildout brief, not a released grading contract.
Work prerequisites: [ordered dependency units](../docs/assignment-order.md). The source profile is Rust 1.98.0, initially edition 2024
and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.
See the [library bootstrap plan](../docs/library-bootstrap-roadmap.md) for
implementation priority, source pins and the migration of unreleased numbers.

## Required implementation

Async functions/blocks, suspension liveness and future state machines. Genuine
`gen` coroutine Iterator/FusedIterator relations and exhaustion state must retain
the actual authored body and yielded item contract. Ordinary FusedIterator
implementations use normal source trait/supertrait proof in PA19/20/31; recognizing
that language role grants no gen state, poll/next body or blanket implementation.
Cancellation and cleanup compose with PA43, and imported bodies with PA29.

Original core requires the genuine `CoroutineState<Y, R>` enum and `Coroutine<R>`
trait, including their actual Yield/Return associated items and pinned resume
method. PA31 owns their source declaration identities and ordinary typed/native
consumers. This assignment owns compiler-generated coroutine identity, input,
yield, return, suspension and resumption; PA43 owns captured storage, Pin, loans,
cancellation and destruction. Keep manual source implementations distinct from
compiler-generated implementations. An enum marker or trait declaration grants
no generated state or blanket implementation.

The ordinary coroutine source/manual-native checkpoint clears coroutine role
admission in original core and reaches `panic_bounds_check` in panicking.rs:266.
Whole original-module typing and generated coroutine execution remain required.
Earlier AsyncDrop, callable and iterator source checkpoints remain prerequisites,
not whole async completion. Preserve
all actual signatures and required bodies, including `async_drop_in_place`;
an accepted attribute or empty future cannot supply their implementation. The
[live blocker tracker](../docs/library-build-blockers.md) records the qualified
compiler, immutable inputs and current frontier.

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

- Poll transitions.
- Stored locals.
- Readiness.
- Suspension state.
- Real CoroutineState variants and generic Yield/Return/input identities.
- Manual resume effects versus generated yield/return state transitions.
- Reentry, opposite lifetimes, capture/cancellation and exact source ownership.

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
