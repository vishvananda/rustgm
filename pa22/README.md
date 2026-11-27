# RustGM Programming Assignment 22 — Unsized types

Build on [PA21](../pa21/README.md) to support dynamically sized storage
tails and their pointer metadata. Use Rust 1.98.0, edition 2024, on
Linux x86-64. PA34 adds edition 2021 and PA41 covers the remaining editions.

## Required implementation

Extend the existing slice and `str` representation to general storage tails.
Earlier assignments already provide array-to-slice reference coercions,
slice indexing, slice patterns, and slice/`str` references passed through
calls, returns, and aggregates. Preserve those modes and behaviors.

Support the following storage tails and conversions.

- Permit a struct's last field to be unsized, including a `?Sized` parameter,
  an associated type and nested structs with unsized tails. Earlier fields,
  enum fields and array elements must remain sized. A tuple may have an
  unsized last element, but stable Rust does not permit tuple unsizing
  coercions.
- Infer sizedness from the instantiated storage tail. A nominal type cannot
  remain unconditionally `Sized` merely because its identity was reserved
  before its fields were completed. Keep declaration requirements, caller
  assumptions, normalized field types and physical storage facts distinct.
- Carry slice element counts and `str` byte counts through wide references
  and raw pointers, including nested tail projections and reborrows. An empty
  or zero-sized-element slice still has its original element count.
- Support built-in array-to-slice and struct-tail unsizing at coercion sites.
  For a generic struct, changed type or const parameters may occur only in its
  final field, whose instantiated type must support the unsizing. Multiple
  parameters may change, including marker types used by associated-type
  projections; those marker types need not themselves unsize. Preserve the
  other arguments and prove element types, mutability and lifetimes independently.
- Permit explicit array-reference to slice-reference and same-owner nominal
  struct-tail reference `as` casts through the same coercion and reborrow rules.
  Consume the selected terminal conversion, evaluate the operand once and preserve
  its element count, including empty and zero-sized tails. Independently prove
  prefix/element compatibility, mutability and lifetimes. Nested reference casts
  and the reverse slice-to-array conversion remain invalid.
- Cover shared/mutable references and const/mutable raw pointers. Creating or
  coercing a raw pointer is safe; dereferencing it still needs the unsafe
  semantics owned by PA27. Do not infer nested coercions through arbitrary
  reference or pointer layers.
- Validate representation constraints and prefix/tail alignment, including
  `repr(C)`, transparent wrappers and packed storage. Packed tails require
  separate drop-related validation; `T: ?Sized` alone does not establish that
  a packed tail is permitted.

Trait-object metadata and dispatch belong to PA23. Standard-library items
such as `Box`, `size_of_val` and slice/`str` inherent methods belong to PA26;
their absence must not conceal a language or metadata failure in PA22.
Custom implementations of unstable unsizing traits are outside the stable
source profile. Tests can observe slice lengths through pattern selection
and bounds behavior without requiring library methods.

The pinned language rules are in
[dynamically sized types](../doc/rust-reference/src/dynamically-sized-types.md)
and [type coercions](../doc/rust-reference/src/type-coercions.md).

## Implementation sequence

1. Build a bounded metadata query over completed storage tails. Its result
   describes pointer representation. Check well-formedness and sizedness
   separately.
2. Complete nominal sizedness and declaration validation. Prepare the exact
   field instances required by metadata and layout queries.
3. Prove each selected unsizing conversion. Consume that selection in body
   construction and lowering, reusing prefix layout and tail metadata.

## Raw pointer boundary

Support safe reference-to-pointer coercions, pointer mutability weakening,
array/struct-tail unsizing and the explicit casts covered by the manifest.
Raw casts may reinterpret pointees when their metadata kinds agree; wide-to-thin
casts discard metadata. Integer addresses require a thin pointer. Comparisons
include slice/string lengths; ordering compares the address before metadata.
These tests never dereference a fabricated or dangling pointer.

`&raw const` and `&raw mut` require an initialized permitted place. They check
conflicts at address creation and create no lasting reference loan or alignment
promise. Direct `&raw const *p` / `&raw mut *p` forms a pointer without reading
through `p`; other raw dereferences/projections need PA27's unsafe semantics.
Implicit conversions preserve embedded lifetime obligations even though a raw
pointer does not keep its outer reference loan alive.

Constant pointer values preserve allocation identity or absolute address bits
and the selected length. Constant pointer comparisons and pointer-to-integer
casts reject, including in dead branches. Immutable static addresses are covered.
Mutable statics and raw-pointer constant patterns are outside the PA22 profile. PA13 defines the structural constant-pattern subset.

Packed declaration containment is distinct from physical alignment. The pinned
compiler permits aligned elements behind array/slice fields while rejecting a
direct aligned struct field. Completed declaration checks authorize the same
rule for sized and unsized layouts; a reference to an unaligned field still
rejects, while its raw address can be formed.

## Command-line interface

Use PA11's `rustgm --emit-lowir` interface and input/output options.
Emit deterministic LowIR after successful checking. A source error exits
1 with a diagnostic and empty standard output. Keep the earlier inspection
modes available.

## Run the tests

Run the PA22 test target from the repository root.

```sh
make -C pa22 test
```

[The manifest](tests/manifest.json) covers slice and string metadata,
concrete and generic tails, prefix and tail projections, reborrows, and
illegal conversions. Exercise empty tails, zero-sized elements, and
nested tails sharing the same completed layout. Use the
[PA11 backend procedure](../pa11/README.md#run-the-tests-and-an-executable)
for executable examples.
