# RustGM Programming Assignment 22: Unsized types

Status: **planned**. This is a buildout brief, not a released grading contract.
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
  For a generic struct, the changed type parameter may occur only in its
  final field, which must itself support the unsizing. Preserve the other
  arguments and prove element types, mutability and lifetimes independently.
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

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

## Tests to develop with the reference

array-to-slice; metadata; size/alignment; sizedness rejection.

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
   and complete the release checks below. Until then this stage stays planned.

## Completion and review

Publish the semantic and lowering boundary, tests/manifest.json and reference
sidecars together. Run all active earlier suites, lint, file/architecture audits,
and affected scaling measurements. Record independent checks before setting
status active in `assignments.json` and adding the stage to the root report.
Student export is a later operation: this repository owns the full solution.
