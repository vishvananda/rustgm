# RustGM Programming Assignment 26 — Language and library hooks

Extend [PA24](../pa24/README.md) and [PA25](../pa25/README.md) with operators,
indexing, library values, and iteration. Use Rust 1.98.0, edition 2024, on
Linux x86-64 with checked arithmetic and aborting panics. The
[PA26 manifest](tests/manifest.json) covers the operator and iteration kernels
below. PA44 carries the broader question-mark, derive, formatting, and
library hook requirements.

## Operators and indexing

Identify canonical library traits and members by resolved identity. Aliases,
re-exports, and imports retain that identity. A user trait with the same name
has its own rules. Publish the supported library declarations on demand as
typed records. Prove the complete selected trait application, including its
arguments, associated output, and ordinary method requirements.

- Use the selected `Neg`, `Not`, arithmetic, bitwise, and shift operations.
  Overloaded operands evaluate in source order and pass through the normal
  call, move, and cleanup rules. Selected primitive methods retain PA11's
  checked arithmetic and use the correct scalar widths.
- For shared indexing, borrow the receiver, evaluate the index, and dereference
  the selected `Index` call's returned reference as a place. Use `IndexMut`
  for writes and mutable borrows, with the same `Index::Output` and the
  selected mutable receiver and result. Preserve one-time evaluation and
  place identity through nested indexing, receiver adjustment, and capture.
- Preserve PA10's right-hand-side-first order for primitive compound
  assignments. An overloaded assignment evaluates its destination before
  the right-hand side and uses the selected mutable receiver call. Preserve
  two-phase receiver reservation where applicable. A borrow for mutable
  square indexing itself is immediate.
- Select canonical methods through the same qualified, lexical, and
  parameter-bound lookup used for user traits. Retain the usual orphan and
  overlap checks, and verify normal call and reification requirements.

## Comparisons and library values

Support the graded `PartialEq`, `Eq`, `PartialOrd`, and `Ord` cases.
Both comparison operands are borrowed in source order. Honor selected
overrides and prove default bodies under their own assumptions. Keep floating
NaN behavior and the canonical comparison `Ordering` distinct from atomic
ordering. Reference and sequence implementations use the selected element
or referent comparison with the ordinary borrow and lifetime rules.
`Ord::min` returns the first equal input and `Ord::max` returns the second.
`clamp` with invalid bounds aborts.

Provide the canonical `Option`, `Result`, and `ManuallyDrop` identities for the
graded declarations and expressions. Their constructors and fields obey
ordinary visibility and type rules. `ManuallyDrop<T>` preserves the
transparent layout of `T` while suppressing automatic destruction of that
field. Distinguish actual constructors from calls such as
`ManuallyDrop::new` when deciding whether storage can be promoted.

Support the declared integer `MAX` constants and slice length operations
through the selected type or receiver. `option_env!` gets its result
from the compilation's source provider and constructs the canonical
`Option<&'static str>`. A missing value selects `None`. Disabled or unused
macro input does not request a lookup.

## Iteration

Lower a `for` loop through separately selected `IntoIterator::into_iter` and
`Iterator::next` calls. Evaluate the source once. Use the canonical
`Option` variants to select the item or end the loop. Apply the same borrow,
move, and destruction rules as for explicit calls, including break and
normal exhaustion.

Support user-defined iterators, the declared shared and mutable slice and
array cases, and half-open integer ranges. Shared slice iterator cloning
copies cursor state independently of the elements. Mutable iteration
returns disjoint items and preserves the remaining slice. The item's source
lifetime and the short borrow of the iterator receiver remain distinct.
Empty slices and zero-sized elements retain their logical length. See the
[half-open range example](tests/spec/iteration/integer-ranges/range-exclusive.rs).

## Run the tests

Use PA11's `rustgm --emit-lowir` interface. Run the PA26 test target from
the repository root.

```sh
make -C pa26 test
```

The manifest declares file-input cases and their expected diagnostics and
LowIR. Use the [PA11 backend procedure](../pa11/README.md#run-the-tests-and-an-executable)
for cases with runtime assertions. Include source order, receiver
adjustment, and early loop exits in additional examples.
