# RustGM Programming Assignment 24 — Closures

Build on [PA23](../pa23/README.md) to infer closure types, form precise
capture environments, and lower callable adapters. Use Rust 1.98.0,
edition 2024, on Linux x86-64. PA34 adds edition 2021 and PA40 covers
the remaining editions.

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
  A wildcard discarding a computed value still evaluates that expression's
  operands. Computed match, `if let`, and `while let` scrutinees retain their
  ordinary move and initialization effects. A try block consumes its tail
  through the actual source protocol;
  a discarded result must not hide an owned capture's move.
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
  variables distinct. Use inferred callable inputs through aliases and
  associated projections while retaining their original proof obligations.
  A function item can satisfy an abstract callable input when its full
  signature matches that input. Required callable goals may wait for inference.
  Cache completed results only after caller terms are settled.
- Settle a generic call's actual callable-output constraints before consuming
  its result as a tuple, nominal, array or callable. Retain complete argument,
  receiver, ownership, lifetime and quantified proof; a field use supplies no
  result shape. Normalize projections introduced by the selected output in the
  existing body environment.
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
belong to PA26. Async closures belong to PA42/PA43. PA29 owns cross-crate transport
of exposed closure and opaque types. These boundaries do not excuse missing
ordinary capture, call or lifetime checks in this assignment.

## Runtime and text contract

Use PA11's `rustgm --emit-lowir` interface and input/output options.
Preserve the earlier inspection modes and their proof boundaries.
Success exits 0 with deterministic LowIR and empty standard error.
A source error exits 1 with a diagnostic and empty standard output.

Use the course's aggregate/out-pointer ABI and PA23 object layout. A closure
environment carries its selected capture fields. Fn and FnMut adapters borrow
that environment with their respective permissions; FnOnce transfers ownership
and performs ordinary cleanup. Noncapturing pointer adapters use the ordinary
function-pointer ABI. Borrowed callable objects use checked method tables and
the existing data/table pair. Use the course's environment layout and
callable symbol rules for LowIR.

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

Test distinct and repeated captures, nested receivers, generic callable
inputs, and growth in unrelated declarations. Verify that repeated demands
reuse adapters and that adding unrelated bodies does not increase capture
analysis work.

## Run the tests

Run the PA24 test target from the repository root.

```sh
make -C pa24 test
```

[The manifest](tests/manifest.json) covers closure creation, invocation,
captured storage, and callable types, including library cases without a
`main`. For an executable case, use the
[PA11 backend procedure](../pa11/README.md#run-the-tests-and-an-executable).
