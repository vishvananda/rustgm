# RustGM Programming Assignment 22: Unsized types

Status: **implemented-draft**. The maintainer reference and 328-case text suite
implement the contract below; this is the current grading boundary.
Prerequisite: PA21. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

## Required implementation

Extend the existing slice and `str` representation to general storage tails.
Earlier assignments already provide array-to-slice reference coercions,
slice indexing, slice patterns, and slice/`str` references passed through
calls, returns and aggregates. Keep their byte oracles and runtime behavior.

The remaining source contract is:

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
and [type coercions](../doc/rust-reference/src/type-coercions.md). The detailed
multiple-parameter struct rule is documented in
[Rust 1.98's `Unsize` definition](https://github.com/rust-lang/rust/blob/1.98.0/library/core/src/marker.rs#L202-L208).

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier stage views remain
available. The existing LowIR vocabulary and course ABI remain unchanged. Emit deterministic
text on success (exit 0, empty stderr); diagnosed rejection is exit 1 with empty
stdout. Signals, timeouts, panics and exit 86 are implementation failures.
The committed `.ref`, `.ref.stderr` and `.ref.exit_status` files freeze these
outputs. `reference.json` records their source and fixture provenance.

## Published tests

The manifest contains 328 cases: 236 accept and 92 reject. Pinned rustc independently
qualifies every source. There are 102 execution comparisons (97 pass, five expected
panics) and 134 compile-only controls, explicitly distinguished in the manifest.
See [tests/README.md](tests/README.md) for the eight families.

Build the matrix from independently declared acceptance and runtime outcomes:

- Direct slice/`str` controls; zero-length, zero-sized and over-aligned
  elements; thin and wide pointer storage; legal unsized tuple types.
- Concrete, generic, nested and projected struct tails; sized instantiations
  of the same declarations; forward declarations and recursive pointer
  referents that must not become by-value metadata dependencies.
- Coercions in let bindings, assignments, arguments, returns, aggregate
  fields and contextual branch/match results; reborrows and metadata
  forwarding through separate functions.
- Rejections for non-final unsized fields, by-value unsized storage, invalid
  `Sized` requirements, changed prefix parameters, mismatched elements,
  strengthened mutability, invalid lifetime relationships and unsupported
  stable tuple coercions. Pair each with an accepted control that reaches
  the same semantic rule.
- Prefix and tail reads, mutable tail writes, nested projections, empty
  tails, out-of-bounds behavior and raw-pointer metadata preservation.
- Deep tails and many wrappers sharing one tail, with charged-work bounds.
  Repeated queries must reuse completed facts, including failed queries.

Add spec fixtures, positive/negative controls and cross-feature regressions in
the earliest owning assignment. Use the course reference for text oracles and
pinned rustc for independent language checks. From PA11, execute emitted LowIR
with the configured CPPGM backend. Do not require new future-stage behavior
implicitly. An empty suite or a not-implemented result cannot pass.

## Implementation sequence

1. Introduce a bounded typed metadata query over completed storage tails.
   Its result describes pointer representation; it supplies neither a
   well-formedness proof nor permission to assume `Sized`.
2. Complete nominal sizedness and declaration validation without stale
   canonical properties. Prepare the exact field instances required by
   metadata and layout queries, retaining normalization evidence.
3. Represent and prove unsizing selections, then consume those selections in
   body construction and lowering. Reuse prefix layouts and tail metadata;
   do not reparse source or reconstruct semantic choices from LowIR text.
4. Qualify source and native controls, publish manifest-listed byte oracles,
   and complete the release checks below.

## Completion and review

Publish the semantic and lowering boundary, tests/manifest.json and reference
sidecars together. Run all active earlier suites, lint, file/architecture audits,
and affected scaling measurements. Retain independent checks and the complete qualification in
[the release evidence](../docs/pa22-release.md).
Student export is a later operation: this repository owns the full solution.

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
Mutable statics and raw-pointer constant patterns are outside this release's
contract; this does not enlarge PA13's structural constant-pattern subset.

Packed declaration containment is distinct from physical alignment. The pinned
compiler permits aligned elements behind array/slice fields while rejecting a
direct aligned struct field. Completed declaration checks authorize the same
rule for sized and unsized layouts; a reference to an unaligned field still
rejects, while its raw address can be formed.

Run `make test-pa22`, `make test-native-pa22`, `make test-rustc`, and the release,
lint/audit/harness checks in `TESTING_AND_REFERENCES.md` before changing this
contract. PA23 remains planned and is not included in acceptance totals.
