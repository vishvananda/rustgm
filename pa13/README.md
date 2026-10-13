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
usefulness. Dynamic slices currently support only irrefutable rests. Owned matches, including
boolean and conditional guards, and irrefutable constructor bindings can execute.
Owned let-else, if-let, while-let and condition chains also execute. Named
constants and refutable dynamic slices remain unfinished.

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

Conditional selection retains `condition source=FILE:NODE input=FILE:NODE
pattern=ID otherwise=FILE:NODE|-`, sorted by source identity. The failure block
exists only for let-else. Sparse chain identities distinguish conditional `&&`
from ordinary boolean expressions; the builder consumes them directly. Inputs
run once; successful captures become visible to subsequent operands and the
success body. False operands end every earlier chain binding's storage.
While-let recreates bindings at each iteration and balances storage for loop
exit, continue and labeled transfers. Let-else checks its failure block in the
surrounding scope, requires divergence, and preserves success bindings for the
following statements. Owned decisions and delayed captures share the same
constructor worklists as matches.

Even statically impossible pattern failures must pass ownership checks. Body IR
retains `false-edge real=bN imaginary=bN` only when ordinary decision edges
cannot reach the failure. Ownership traverses both successors; executable
lowering follows only the real successor and allocates no storage used solely
on imaginary paths. Non-final irrefutable match arms use the same rule to check
later arms. A diverging input has no such failure continuation. None of these
rules bypass pending region or loan obligations.
Conditional match guards and their let chains can be parsed, resolved and
selected using these same conditional facts. Initializers retain pre-shadowing
binding identities; later operands and arm bodies see the new bindings. This
selection feeds the guarded execution described below.

A full-domain interval requires no value test. Owned guard execution uses
internal read-only views of selected bindings, then commits owning captures
only after success. Guard-let bindings have their ordinary local ownership and
mutability. Failed guards end attempt-local storage and resume the next matching
alternative in written order; the scrutinee is evaluated once. Nested products
traverse fields in written order and keep the last field as the innermost retry.
Each subtree has first-match and resume continuations, so generated decisions
stay proportional to the pattern rather than expanding products of alternatives.
A product field with no first match rejects that product immediately.

Body IR records `guard-view`, `guard`, `guard-place`, `guard-start`, `bind-view`
and `guard-use` facts. Shallow protection covers tested places across the match;
selected binding views additionally protect the whole bound value. Mutation of
an untested, disjoint field can remain valid. Protection follows future uses:
divergent paths need not preserve values for a guard continuation they cannot
reach. Moving a non-Copy value or assigning through a view is rejected. Packed
origins must support the alignment of an internal reference, even for zero-sized
fields. An empty rest still requires and protects its containing array's shallow
state; moving disjoint elements remains possible. These are internal view checks,
separate from user-reference regions.

Retry paths retain imaginary ownership edges for bypassed guards and later
alternatives. They do not invent a guard-failure edge after an irrefutable
conditional guard. Native lowering follows only real edges. Static retry paths
use `select pN anchor=PLACE`: cached typed offset recipes form fresh addresses
in each use block, preserving dominance after a resume without replaying every
parent projection. Proof-only tested-place metadata never forms addresses for
inactive enum variants. Irrefutable and conditional binding execution can combine
constructors and scalar intervals.

`region-var region:N body=FILE:NODE source=FILE:NODE origin=pattern-borrow|pattern-input
pending` gives existential region provenance. IDs are globally allocated from
one and do not alias `static` or signature `bound:DEPTH:INDEX` regions. Pending
`region-equal` records use PA9's vocabulary; neither these equalities nor a
capture's fresh region constitutes a loan or outlives proof. Borrowed captures
and reference-pattern execution are rejected at the body-building boundary.

The final line is `eof roots=N patterns=N fields=N alternatives=N ranges=N
matches=N arms=N coverage-work=N region-vars=N region-equalities=N conditions=N
chains=N`.
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
