# PA24 implementation handoff

PA24 remains planned. This is a maintainer plan, not an implemented grading
contract. PA23 functional release validation and its frozen performance matrix have passed;
no closure compiler changes are mixed into that qualification.

The parser and resolver already retain closure parameter/result/body nodes,
`move`/async flags, closure scopes and label barriers. Implement semantics from
those nodes and resolved identities. Do not reparse or generate replacement
Rust source. The supported edition is 2024; async closure execution belongs to
PA32/PA33 and library owning pointers remain PA26.

## Required phase boundaries

1. Give each closure expression a distinct type identity with its enclosing
   generic inputs, callable signature and typed capture environment. Use the
   existing type/substitution/inference machinery; source identity is not a
   reusable function instance or permission to erase lifetimes.
2. Infer parameters and results with contextual callable expectations, retaining
   quantified inputs and the enclosing body's inference dependencies. An ordinary
   closure remains monomorphic after inference. Nested closures need distinct
   locals, returns, break targets and body contracts.
3. Collect capture paths from typed place uses. Merge overlapping prefixes with
   the required capture-mode ordering using an indexed path structure, rather
   than comparing every capture with every other capture. Keep declaration/use
   order deterministic and count actual path work and retained storage.
4. Apply precision rules: disjoint fields, shared-reference truncation, array/
   slice indexing, discriminant and wildcard reads, packed fields, and move
   contexts. Unique immutable capture is a distinct permission; replacing it
   with an ordinary shared borrow would permit invalid aliasing.
5. Construct the environment with existing aggregate, borrow, move and cleanup
   facts. Rewrite captured place accesses through the typed environment mapping.
   The environment's actual capture fields determine Copy/Clone/auto-trait
   behavior. Capture creation and closure calls must pass the ordinary region,
   initialization, loan and destruction gates.
6. Add resolved identities for `Fn`, `FnMut`, `FnOnce`, their associated output
   and the stable parenthesized bound syntax. Implement invocation through shared,
   mutable and consuming receivers, respecting the hierarchy and complete input/
   output obligations. `move` does not by itself make a closure `FnOnce`-only.
   A same-spelled user trait remains a user trait.
7. Emit callable adapters through ordinary checked body/instance discovery and
   the existing LowIR ABI. Noncapturing closures may coerce to function pointers;
   captured closures, including captured zero-sized values, may not. PA23's
   dynamic dispatch should consume proved callable-object targets without a
   second ad hoc invocation path.
8. Qualify source and native behavior, publish the owning fixtures and byte
   oracles, then run the release checks and immutable performance comparisons.
   Retain all earlier byte contracts except explicitly reviewed representations.

The pinned rules are in `doc/rust-reference/src/types/closure.md`, especially
capture precision, unique immutable borrows, call traits, other traits and drop
order. Existing special cases for unsupported source constructs must not serve
as the semantic evidence for a new rejection control.

## Initial declared controls

`obj/pa24-pa25-work/pa24-controls-v1/plan.json` records 32 outcomes before the
implementation: noncapturing/contextual calls, first-use inference, immutable/
mutable/move captures, once-only invocation, closure copying, precise field
borrows and moves, loan end points, unique immutable borrows, all three call
traits, dynamic Fn/FnMut calls, quantified input/output identity, returned
closures, function-pointer restrictions, capture destruction, shared-reference
truncation and nested captures. All 32 declarations match pinned rustc 1.98.0 object compilation, recorded in
`pa24-controls-v1/rustc-v1/`. No RustGM result or text oracle is claimed for them.

Extend the matrix with paired packed, pattern/discriminant, nested-generic,
partial-move, promotion, opaque-output, marker-bound and method-adjustment cases
as the owning typed interfaces become available. For scaling, vary independent
closures, fields per environment and repeated paths separately, with an ordinary
nonclosure control. Avoid rebuilding crate-wide indexes or scanning unrelated
locals for every closure.

## Capture graph and canonical type checkpoint

The in-progress implementation now has a flat, interned capture-place forest
in `sema::closures::captures`. Required uses retain their separate capture modes;
reverse propagation and a forward covered-prefix pass merge overlaps without
repeated subtree scans. Merely visiting a place does not capture it. Exhausted
work invalidates the query, including attempts to finalize it. The graph keeps
path discovery order; capture layout and the language's unspecified captured-value
drop order are separate concerns.

`types::Kind::Closure` retains a distinct expression ID, complete enclosing
generic arguments, a callable signature and a tuple of actual capture-field
types. The constructor remains 24 bytes. Type substitution, inference imports,
occurs checks, lifetime occurrence indexing and normalization retain every edge.
Storage layout, Copy/Clone, auto traits, destruction and runtime lifetime
components inspect the capture environment. Signature-only and unused enclosing
inputs remain identity/formation facts. Closures do not automatically implement
equality. Same-expression closure relations equate their inputs invariantly;
see the [rustc structural relation implementation](https://github.com/rust-lang/rust/blob/master/compiler/rustc_type_ir/src/relate.rs).
This upstream link is explanatory; the independently compiled controls use the
course's pinned rustc 1.98.0.

The additional declared inputs in
`obj/pa24-pa25-work/closure-properties-v1/plan.json` cover signature-only Copy,
Send and static bounds; unused enclosing inputs; shared, mutable and unique
immutable captures; captured raw pointers; captured destruction and equality.
All 12 expectations matched pinned rustc object generation before RustGM oracles
were generated. They complement the original 32 closure controls. No public
source closure execution is claimed at this checkpoint: nested inference,
capture collection, body contracts, callable adapters and lowering still need
integration.

The first model checkpoint passed ten closure-type invariants, four capture
prefix invariants, 34 existing type-query regressions, strict release Clippy,
and the syntax/architecture audit (1,134 files, zero findings). All 7,784 active
PA1–PA23 text oracles remained byte-identical. Evidence is in
`obj/pa24-pa25-work/closure-captures-v1/` and
`obj/pa24-pa25-work/closure-types-regressions-v1/`. These are development checks;
the PA24 release still requires its source/native matrix and performance study.

The immutable model-only comparison in
`obj/pa24-pa25-work/closure-model-v1/performance-v1/` compared published PA23
`ac46cc710053ff4b0638e63345baf43de2dac21516d41505894a49002395e406`
with model candidate
`4035e2734145930f99400583c856f9f1e97d1a2c3a08b5759ecbf2bfd508d649`.
Five frozen, previously rustc-qualified inputs covered binder depth, ordinary
statements, shared inputs, object dispatch and defining regions. All outputs
matched exactly. On CPU 2, four A/A observations preceded five paired ABBA
blocks (three executions per observation), eight RSS observations per binary
and Callgrind instruction counts. Instructions increased 0.091–0.227%; median
paired wall changes ranged from -8.06% to +0.32%, and median RSS changes from
-22 to +80 KiB. The object-dispatch A/A spread was 19.81%, so these timings do
not establish a speedup. Raw observations and input/binary hashes remain in the
measurement directory. This comparison predates call-trait integration.

## Call-trait declaration checkpoint

The parser now retains whether an applied path used parentheses even when its
output is omitted. Macro transfer preserves the flag; the earlier text format
has not changed. Resolution demand-loads real `Fn`, `FnMut`, `FnOnce` and
`FnOnce::Output` identities through ordinary namespaces, aliases and glob
imports. Same-spelled user traits retain their own identities. Call traits have
private parameter scopes; their members use ordinary associated-item scopes.
An existing ops glob alone does not eagerly create this metadata.

Semantic analysis registers typed language schemas separately from source owners;
there is no generated declaration AST. The ordinary trait headers encode
`Fn: FnMut` and `FnMut: FnOnce`. Parenthesized inputs form one tuple argument,
and the written or default unit output becomes an equality on the inherited
`FnOnce::Output` projection. Input and output normalization retain their original
formation roots. Written quantifiers and implicit input elision share the existing
binder machinery. Separate call bounds can use the same quantifier while keeping
their output-elision state separate. Output lifetime validation includes the
complete trait subject and arguments, not only the callable's explicit inputs.

`dev/tests/call_trait_contracts.rs` checks 26 declaration/forwarding outcomes
through checked LowIR construction and 18 namespace outcomes against pinned
rustc, plus preservation of the syntax flag across expansion. The lifetime
negative requires the ordinary template proof, not just successful type inference.
Evidence is under `obj/pa24-pa25-work/call-contracts-v1/`: `tests-v5.log`,
`clippy-v3.log` and `audit-v2.log` (1,141 files, zero findings).
Independent object receipts retain the initial 13 declarations, 11 added
contracts and five elision probes. Corrected declarations are explicit in
`plan-v2.json` and `elision-plan-v2.json`; initial predictions and observations
were retained. Rustc accepts an Output lifetime constrained by the trait subject,
but repeated input positions still make an omitted result lifetime ambiguous.
The latter existing signature behavior was preserved.

This checkpoint supplies contracts and caller assumptions. It does not yet
select callable implementations for function values or closures, invoke a generic
callable, build callable trait objects, or emit closure adapters. Those operations
still need their actual source/body proofs and executable integration. PA24 stays
planned, and PA25 has not started.

All 7,784 PA1–PA23 byte oracles passed unchanged after the callable-contract
changes, together with 59 focused parser/name/type-query tests. Their immutable
source snapshot and receipts are in
`obj/pa24-pa25-work/call-contracts-regressions-v1/`. The candidate binary is
`3c4b7007dad9df3a48991087e69769f9ee4179bdf549f9abb065b88ee444e7e0`.
The new 8/32/128-call quantified-forwarding inputs were independently object
qualified in `call-contracts-v1/rustc-scaling-v1/` before measurement.

The next immutable study, `call-contracts-v1/performance-v1/`, compares that
candidate with the preceding model-only binary using the same five shared
inputs and adds the three new forwarding inputs. Shared outputs were identical;
shared instruction changes ranged from -0.113% to +0.158%. Wall medians ranged
from -4.69% to +4.34%, but A/A spread reached 61.93% on this busy host, so this
study does not establish a small wall-time change. RSS differences were -236
to +146 KiB. The new 8/32/128-call inputs used 7.347/24.476/93.119 million
instructions, consistent with linear growth over this range; median wall times
were 7.52/14.45/38.19 ms. These new inputs have no accepted baseline output and
are candidate-only observations, not A/B speedup claims. All observations and
hashes are retained. Both development studies used the same A/A/ABBA/RSS and
instruction-count protocol described above.

The publication checkpoint in
`obj/pa24-pa25-work/closure-contracts-checkpoint-v1/validation-v1/` passed all
70 gates in 1,050.94 seconds, with the candidate and compiler/test source hashes
unchanged throughout. It includes 1,845 release maintainer tests, 122 native
maintainer groups, strict release Clippy, formatting, file and harness audits,
upstream/backend qualification, all 7,784 active byte oracles, 7,583 independent
source checks plus 11 declared fragments, and 4,285 native source checks with
2,364 executable and 314 codegen-only comparisons. Token, literal and layout
independent checks also passed. This is a foundation checkpoint, not a PA24
release: assignment status and existing references are unchanged.

## Next executable integration boundary

The next proof extension belongs in the existing scoped trait selector. Give
language-provided callable implementations their own retained evidence; do not
fabricate a source impl owner. Safe function items/pointers, closure call kinds
and reference forwarding need complete input equations and a selected Output.
Associated normalization must consume that same witness in declaration, body and
canonical proof cursors. Partial callable inputs must retain ordinary inference
and wakeups; requiring premature canonicalization would reject valid generic
function-item inference. Unresolved Self is not permission to guess a callable.

Generic invocation then needs a checked receiver adjustment and an adapter target.
Reuse method autoref/autoderef and dynamic dispatch infrastructure: an immutable
binding containing `&mut F` can call through its referent, whereas an owned mutable
closure can require a mutable binding. Reference forwarding differs by receiver:
shared references require the referent's Fn permission, while consuming a mutable
reference can use the referent's FnMut permission without consuming that referent.
These are implementation targets to qualify, not completed behavior.

Closure body inference still needs separate locals, return/control frames and
region contracts while retaining enclosing inference dependencies. Capture uses
must feed the existing indexed prefix graph after typed place construction.
Unique immutable capture needs an exclusive loan permission distinct from an
ordinary shared borrow and from requiring a mutable source binding. Environment
construction, captured-place access, call adapters and cleanup must all reach the
normal ownership/lifetime proofs before native execution is claimed.
