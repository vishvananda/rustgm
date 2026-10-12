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
edition-2024 binding modes, irrefutable borrowed slice rests, and owned match
decisions. Boolean, integer and character intervals compose with tuple, record,
enum, fixed-array and reference patterns for exhaustiveness and whole-arm
usefulness. Dynamic slices currently support only irrefutable rests. Unguarded owned matches and irrefutable constructor bindings can execute.
Guards, named constants, let conditions/chains and let-else remain unfinished.

The current vocabulary is `selected-patterns 1`, the canonical type table,
`root FILE:NODE pattern=ID`, and arena-ordered `pattern ID source=FILE:NODE
input=TYPE KIND...` records. Roots sort by file/node; pattern and edge identities
follow deterministic source traversal. A pattern's input type remains distinct
from a binding's local type. Kinds are:

- `wildcard` (including a selected rest with no binding operation).
- `range=ID` for a scalar literal or interval in the separate range arena.
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

`range ID ranks=LOW:HIGH` uses inclusive, ordered target ranks. Signed integers
use a sign-bit bias; character ranks exclude the surrogate gap. The input type
determines how to interpret ranks. `match ID source=FILE:NODE input=FILE:NODE
arms=START:END exhaustive=true uninhabited=BOOL work=N` retains the scrutinee and a half-open
source-ordered arm range. `arm ID source=FILE:NODE pattern=ID guard=FILE:NODE|-
useful=BOOL value=FILE:NODE` retains the guard and result identities. Guarded
arms contribute no exhaustiveness coverage. Usefulness refers to the whole arm,
not individual redundant alternatives. Coverage counts visited pattern nodes
and interval-index operations for the scalar fast path. Composite queries count
type/pattern visits and shared decision-graph operations. Both reject work beyond
1,000,000 per query. Cached type facts are keyed by the matching module; enum
domains additionally distinguish valid values from possibly invalid referents.

`uninhabited` is a contextual coverage fact, not a physical layout property or
proof that a place was initialized. Private fields cannot expose an empty-type
proof to an unrelated module, and references to empty types remain possible
coverage inputs. Constructor decisions retain non-moving `is-variant VARIANT PLACE` body values
and selected field identities.
Variant tests require an initialized owned input, including its tracked fields;
a surviving physical tag does not permit testing a partially moved enum.
Single-variant wildcard patterns need no tag read. An empty match retains a
`validate PLACE` operation in body IR; an uninitialized empty-type local may pass
pattern/body inspection but fails ownership checking before executable IR.

Composite coverage uses reduced binary decisions over normalized scalar and
constructor ranks at sparse typed places.
Union, intersection and difference queries are memoized; products and nested
alternatives are not eagerly expanded into combinations. A missing witness is a
deterministic partial assignment such as `{ $.0 = false, $.1 = true }` or
`{ $ = E::B }`. `$` is the input, `.name`/`.N` select fields, `.*` dereferences,
and `[N]` selects an array element. Unconstrained inputs render `_`; these are
diagnostic constraints, not Rust source to reparse. Whole-arm usefulness still
ignores guard outcomes and does not diagnose individual redundant alternatives.

Decisions retain ordered alternatives, read the scrutinee only when the
decision or capture requires it, and capture values after a successful decision.
Composite or-patterns save selected alternative identities in body locals;
subsequent capture branches preserve the selected places without duplicating
products of alternatives. Capture projections are formed in their own branches,
so they do not depend on values defined only along an earlier decision path.
The ordinary let/parameter path uses these decisions when an irrefutable pattern
combines refutable alternatives. A declaration without an initializer only
declares its bindings. Field identity retains its owning variant for layout;
constructor tests compare the stored discriminant with the selected layout value.
A full-domain interval requires no value test. Guarded execution is rejected
before publishing body IR: inspection has not proved protected guard accesses,
delayed moves or failed-guard retries. Irrefutable binding coverage and execution can combine
constructors and scalar intervals; conditional let/let-else execution is
not yet available.

`region-var region:N body=FILE:NODE source=FILE:NODE origin=pattern-borrow|pattern-input
pending` gives existential region provenance. IDs are globally allocated from
one and do not alias `static` or signature `bound:DEPTH:INDEX` regions. Pending
`region-equal` records use PA9's vocabulary; neither these equalities nor a
capture's fresh region constitutes a loan or outlives proof. Borrowed captures
and reference-pattern execution are rejected at the body-building boundary.

The final line is `eof roots=N patterns=N fields=N alternatives=N ranges=N
matches=N arms=N coverage-work=N region-vars=N region-equalities=N`.
`coverage-work` sums match queries; it does not count ordinary binding checks.
Count mode emits that same line without `eof `, after the
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
