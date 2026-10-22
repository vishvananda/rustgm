# RustGM Programming Assignment 24: Closures

Status: **implemented-draft**. The maintainer reference, manifest and reviewed
byte oracles implement the contract below.
Prerequisite: PA23. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

## Required implementation

Extend PA23's callable and object machinery with anonymous closure types,
precise environments and checked call adapters. Implement the source contract
through typed capture facts, ordinary ownership/lifetime proof and demanded
LowIR instances.

- Give each closure expression a distinct anonymous identity. Infer its inputs
  and result from annotations, calls and contextual callable requirements.
  Preserve enclosing generic parameters and nested body identities. An input
  used in a closure's signature is not automatically an environment field.
- Collect typed capture paths and the required shared, unique immutable,
  mutable or by-value permission. Merge overlapping prefixes using the
  strongest required permission. Respect explicit `move` and the difference
  between copying a value, moving it and merely mentioning a place.
- Preserve disjoint field captures and the source rules for shared-reference
  truncation, wildcard/rest patterns and discriminant reads. Indexing and
  subslicing capture the containing array or slice. Unused places do not gain
  reads, moves or loans merely because they occur in a pattern.
- Build the actual environment layout, moves and destruction from its stored
  captures. Derive applicable Copy, Clone and auto-trait obligations from those
  fields and their permissions. Preserve partial moves and independent field
  lifetimes; an unused owner parameter must not create a fictitious capture.
- Determine Fn, FnMut and FnOnce from the closure body's use of its captures.
  A `move` closure may still implement Fn or FnMut. Implement the call-trait
  hierarchy, complete argument tuples and Output, generic bounds, method
  selection and shared/mutable/consuming receiver adapters. A shared receiver
  does not authorize mutation or consumption of its referent.
- Coerce noncapturing closures to compatible function pointers through the
  existing calling convention. Check inputs, result and callable safety. A
  captured closure must not silently discard its environment during coercion.
- Support borrowed callable objects, nested reference receivers, associated
  and projected callable inputs, quantified lifetimes and returned closures.
  Keep public late-bound slots, enclosing capture lifetimes and local proof
  variables distinct. Required callable goals may wait for inference; unresolved
  caller terms must not enter a completed semantic cache.
- Prove original bodies, including unused closures and unused nested children,
  before accepting the program. Prove demanded substitutions and adapters as
  well. Returned references require actual source lifetime evidence; selection
  of a callable implementation or an object table cannot establish that proof.
  Entry-free library validation must report an ordinary body failure before
  rejecting an otherwise unsupported metadata transport shape.

Consult the pinned [closure type rules](../doc/rust-reference/src/types/closure.md),
[closure expressions](../doc/rust-reference/src/expressions/closure-expr.md),
[call expressions](../doc/rust-reference/src/expressions/call-expr.md) and
[coercions](../doc/rust-reference/src/type-coercions.md). Unsafe capture operations
with raw pointers or packed storage belong to PA27; owning library pointers
belong to PA26. Async closures belong to PA32/PA33. PA29 owns cross-crate transport
of exposed closure and opaque types. These boundaries do not excuse missing
ordinary capture, call or lifetime checks in this assignment.

## Runtime and text contract

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`; preserve earlier inspection
modes and their proof boundaries. Successful complete lowering exits 0 with
deterministic LowIR and empty stderr. Diagnosed rejection exits 1 with empty
stdout. A panic, signal, timeout or exit 86 is a failed implementation, not a
language rejection. The published `.ref`, `.ref.stderr` and `.ref.exit_status`
sidecars define the byte contract; `reference.json` records their provenance.

Use the course's aggregate/out-pointer ABI and PA23 object layout. A closure
environment carries its selected capture fields. Fn and FnMut adapters borrow
that environment with their respective permissions; FnOnce transfers ownership
and performs ordinary cleanup. Noncapturing pointer adapters use the ordinary
function-pointer ABI. Borrowed callable objects use checked method tables and
the existing data/table pair. This is the course ABI; matching rustc's private
environment layout or symbols is not required. CPPGM remains the unchanged
LowIR-to-native backend.

## Implementation and performance

Consume typed uses and places produced by the ordinary body checker. Capture
merging must use an indexed path representation rather than repeatedly comparing
every pair of captures. Retain the checked source conditions needed by truncated
captures and explicit reference results; do not reconstruct them from rendered
types or lower away a still-unproved lifetime requirement.

Reuse source CFGs, trait selection, canonical argument identities, region proof,
instance discovery and destructor recipes. Suspension must retain the exact
owner's inference and proof state without taking authority over another body's
fresh regions. Share completed selection and projection work; repeated callers
must not repeatedly scan unrelated declarations or regenerate the same adapter.

Measure repeated and distinct captures, nested receivers, generic inputs,
projected and quantified objects, and growth in unrelated source. Keep actual
work/storage counters alongside immutable-binary A/A calibration and paired
ABBA timings. Verify byte-identical shared outputs. A cache alone is not evidence
of linear work; inspect adapter count and per-body proof growth separately.
Follow `spec.md` for the architecture and measurement requirements.

Use safe idiomatic Rust. Strict Clippy `all` and `pedantic`, the file/function
and nesting audit, and compiler phase boundaries remain mandatory. Keep lint
exceptions narrow and explained. Do not use blanket allowances, split files
mechanically to evade limits, or read the environment from `dev/src/`.

## Completion and review

The manifest contains 467 independently declared source outcomes:
323 accept and 144 reject. The reviewed native classifications contain 311
runtime comparisons, two unused-body code-generation controls and ten entry-free
object controls. Draft outcome files retain the history of the source hypotheses;
they are not substitutes for the released manifest. See
[the release review](../docs/pa24-release-review.md) for current evidence and
release qualification. No planned assignment is counted as passing.

When updating this assignment, qualify every source with pinned rustc, compare declared
executions through RustGM/CPPGM, generate and review the complete byte oracles,
and publish the manifest and provenance together. A negative must exercise its
intended semantic rule rather than an unrelated unsupported-feature guard.
Compile-only and object controls must retain those explicit classifications;
do not add a synthetic main or claim their unused bodies executed.

Run all active earlier text and independent source/native suites, full release
Cargo and native maintainer tests, formatting, strict Clippy, architecture/file
and layout audits, harness and upstream checks, and the performance matrix.
Run the owning `test-pa24` and `test-native-pa24` targets and preserve root
text, independent-source and native-suite coverage. Follow
`TESTING_AND_REFERENCES.md` and use the storage/resource guards. Student export
is a later operation; this repository owns the full reference implementation.
