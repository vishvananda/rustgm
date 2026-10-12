# RustGM Programming Assignment 13: Pattern semantics

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA12. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

## Required implementation

Binding modes, match lowering and exhaustiveness/usefulness.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

## Maintainer inspection checkpoint

The in-progress implementation also accepts `--emit-patterns` and
`--count-patterns`. This is a sparse selected-pattern view, not a completed PA13
grading contract or a lifetime/loan validity claim. Simple standalone value
bindings and wildcards use the earlier direct path and have no selected root.
The view covers irrefutable owned products/arrays, explicit reference patterns,
edition-2024 binding modes, and irrefutable borrowed slice rests. Conditional
patterns, match decisions, guards, usefulness and witnesses remain unfinished.

The current vocabulary is `selected-patterns 1`, the canonical type table,
`root FILE:NODE pattern=ID`, and arena-ordered `pattern ID source=FILE:NODE
input=TYPE KIND...` records. Roots sort by file/node; pattern and edge identities
follow deterministic source traversal. A pattern's input type remains distinct
from a binding's local type. Kinds are:

- `wildcard` (including a selected rest with no binding operation).
- `bind def=ID local=TYPE mutable=BOOL capture=value|ref|ref-mut`, followed by
  `region=REGION loan=pending` for borrowed captures, then `subpattern=ID|-`.
- `deref mutable=BOOL implicit=BOOL pattern=ID` for each reference adjustment.
- `product variant=ID|- fields=START:END` with a half-open edge range.
- `or alternatives=START:END` with shared resolved binding identities.

`field ID tuple=INDEX|nominal=FIELD|index=INDEX|subarray=START:END|subslice=PREFIX:SUFFIX
pattern=ID` records retain selected places. Subarray endpoints are half-open;
subslice values count excluded elements at either end. Currently only `0:0`
is irrefutable for a dynamic slice. `alternative ID pattern=ID` records retain
each occurrence, including distinct capture operations for a shared binding.

`region-var region:N body=FILE:NODE source=FILE:NODE origin=pattern-borrow|pattern-input
pending` gives existential region provenance. IDs are globally allocated from
one and do not alias `static` or signature `bound:DEPTH:INDEX` regions. Pending
`region-equal` records use PA9's vocabulary; neither these equalities nor a
capture's fresh region constitutes a loan or outlives proof. Borrowed captures
and reference-pattern execution are rejected at the body-building boundary.

The final line is `eof roots=N patterns=N fields=N alternatives=N region-vars=N
region-equalities=N`. Count mode emits that same line without `eof `, after the
same checks. Diagnostics return 1 and emit no partial pattern view. No PA13
manifest or generated sidecars are published until the whole assignment passes.

## Tests to develop with the reference

guards; or-patterns; slices; unreachable arms; missing constructors.

Add spec fixtures, positive/negative controls and cross-feature regressions in
the earliest owning assignment. Use the course reference for text oracles and
pinned rustc for independent language checks. From PA11, execute emitted LowIR
with the configured CPPGM backend. Do not require new future-stage behavior
implicitly. An empty suite or a not-implemented result cannot pass.

## Completion and review

Publish the semantic and lowering boundary, tests/manifest.json and reference
sidecars together. Run all active earlier suites, lint, file/architecture audits,
and affected scaling measurements. Record independent checks before setting
status active in `assignments.json` and adding the stage to the root report.
Student export is a later operation: this repository owns the full solution.
