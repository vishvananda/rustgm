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
The checkpoint is published on `main` as `db0facbc`. Its successful storage
guard removed 9,158 generated files (8.50 GiB of unshared blocks), retaining
diagnostics and observations in cleanup receipt `1792539731917936882.jsonl`.

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

The next independent controls are retained in `call-implementations-v1/`.
Pinned rustc matched 24 of the first 25 predictions; the corrected
`plan-v2.json` records that `&mut F` does not implement Fn even when F does.
It can forward FnMut and FnOnce using the referent's FnMut permission. The
original prediction and diagnostic remain in `plan.json` and `rustc-v1/`.
Three additional object-qualified controls in `extra-plan-v1.json` cover a
generic function return inferred solely from Output, a conflicting fixed return,
and an unconstrained unused generic parameter. These are language expectations;
their RustGM implementation is still under development.


## Callable implementation checkpoint in progress

The trait selector now has language callable evidence carrying the selected
Output, separate from source impl owners. Safe function pointers/items retain
complete tuple/input equations, including unresolved generic item arguments,
output-driven inference and nested callable binders. Closure type models supply
their declared call kind; source closures are still pending. Active configured
`target_feature` functions cannot satisfy these safe call contracts. Attribute
filtering crosses the existing resolved-unit interface, keeping expansion
internals out of semantic analysis.

Reference forwarding uses the normal scoped child proof. Shared references
require Fn; mutable references require FnMut for FnMut/FnOnce and cannot supply
Fn. A deterministic chain is walked in one inference context. The walk stops at
an indexed possible caller premise on an intermediate reference, preserving
ordinary full-predicate selection there. Partial child proofs retain their own
continuations and publish input progress without becoming completed permission.
A typed invariant at depths 8/32/128/512 checks a linear work bound, shared
unresolved input/output inference, and published callable evidence.

Evidence lives in `obj/pa24-pa25-work/call-implementations-v1/`. The 48 source
controls in `tests-v9.log` passed with the related declaration, trait, binder and
inference suites (179 groups). Three additional inferred-reference controls were
independently qualified in `rustc-inferred-v1/`; their implementation test and
work invariant are in `tests-v10.log` and `forwarding-work-v1.log`. Pinned rustc
also qualified the added output/scoped/attribute controls in `rustc-extra-v1/`,
`rustc-scoped-v1/` and `rustc-feature-v1/`. Strict Clippy `clippy-v4.log` passed;
`audit-v4.log` reports 1,146 files and zero findings before the later invariant
was added. All 7,784 earlier byte fixtures passed unchanged (`bytes-v1.json`).
No assignment status or reference sidecars changed.

The measured candidate is
`b1b54d029f98d85ff2cba634ce2b741156c04c9fbb08bc52a5a8029c7d743537`;
the pre-compression baseline is
`cb4f97446088535d6d9e613ad71545350433bfa5eae3b2f1e0fb49383b6999a5`.
Both studies retained immutable binaries, frozen input hashes, every A/A and
ABBA observation, RSS samples and Callgrind counts, and verified identical
outputs. `performance-v1/` measures the ordinary control and aliased reference
depths 8/32/96. Instructions changed by -0.050% on the ordinary control and
-23.0%/-46.4%/-57.4% on the reference inputs. Their paired wall medians changed
by -2.1%/-18.0%/-33.7%; A/A spread reached 22%, so the small wall change is not
established. Ordinary wall median changed by +1.41%. This resolved-type matrix
does not demonstrate quadratic behavior in the original solver.

`performance-v2/` measures the additional three chains whose referent function
item shares an unresolved generic input/result. Instructions changed from
5.899/40.698/450.778 million to 4.756/31.996/398.626 million; paired wall medians
changed by -8.75%/-14.71%/-11.15%, with 1.50%/5.91%/2.26% A/A spread. Candidate
medians were 5.72/8.49/59.09 ms. Full compilation remains superlinear on these
inputs: at depth 96, borrow checking accounts for about 81% of instructions and
region point propagation about 72% (inclusive profiles overlap). The solver
invariant isolates the corrected traversal; these end-to-end observations do
not claim that the separate region propagation algorithm has been fixed.

The first raw depth-128 source exceeded RustGM's existing syntax cap. Using
aliases removed that limitation but hit pinned rustc's default trait recursion
cap at depth 128. Both diagnostics and original predictions remain in
`rustc-forwarding-v1/`, `forwarding-before-v1/` and `rustc-forwarding-v2/`.
`forwarding-plan-v3.json` explicitly selects depth 96 within both unchanged
limits; all ten declared outcomes agree in `rustc-forwarding-v3/`.

Generic invocation is the next executable integration. The separate
`call-invocations-v1/plan-v2.json` contains 28 independently object-qualified
expectations (`rustc-v2/`). The original prediction for competing Fn(u8) and
FnMut(u16) was corrected: call syntax selects the stronger available Fn contract
and rejects u16, rather than falling back to FnMut. Multiple inputs under the
same Fn kind can be selected by typed arguments. These are expectations, not
implemented invocation claims. PA24 remains planned; PA25 remains pending.


## Callable invocation checkpoint

Generic call expressions now retain the selected Fn/FnMut/FnOnce permission,
implicit receiver dereferences and borrow, complete argument tuple and selected
Output. The typed CFG has an Invoke operation whose first operand owns or borrows
its receiver. Ownership, loan activation, liveness, instantiation, constant
eligibility and lowering consume that operation. Function items and pointers use
the existing native ABI, including aggregate returns and opaque representations.
Callables still require a concrete executable adapter; source closure bodies and
dynamic callable adapters remain unfinished.

Call syntax follows builtin references before selecting the referent's strongest
available call kind. A trait bound only on a reference is insufficient to call an
otherwise unconstrained referent. Isolated probes choose the kind without leaking
unconstrained argument variables into the body. Typed arguments can distinguish
multiple inputs under that kind. Normalized opaque associated results use their
caller/result promises; an unknown physical signature cannot keep a stronger
unsupported kind pending. Owned opaque function items select their canonical
instance even when the hidden runtime value occupies zero bytes.

`call-invocations-v1/tests-v8.log` retains 52 independently qualified source
controls, and `native-v7.log` retains 16 CPPGM/rustc executions. These include
reference-only negative bounds, receiver mutability and consumption, higher-ranked
input/output lifetimes, associated and opaque callers, all three call kinds,
aggregate construction/returns and function-pointer arguments. Strict release
Clippy passed in `clippy-v3.log`; `audit-v3.log` reports 1,152 files and zero
findings. Earlier mistaken predictions and their correction receipts remain in
the work directory; tests do not use planned-stage rejection as proof of a rule.

The immutable candidate is
`d91454bf7e1b975d3bff794447b4c359694abdc20262a26fc2ea58c8b956e58e`,
pinned with compiler/test source hashes in `call-invocations-v1/binaries-v2/`.
All 7,784 PA1–PA23 byte fixtures pass unchanged (`bytes-v2.json`). No assignment
status or reference sidecar has changed. The following measurement and complete
checkpoint gates qualify this candidate before publication.

`call-invocations-v1/performance-v1/` compares the published contract binary
with this candidate over all 38 frozen PA23-release inputs and three new
8/32/128-invocation programs. Every shared output is identical. Shared instruction
changes range from -0.047% to +0.577%; paired wall medians range
from -6.66% to +15.76% (median +1.95%). A/A
spread reaches 66.15%, so this study does
not establish wall-time neutrality. RSS changes range from -460 to
+492 KiB. New invocation programs take
4.02/5.04/9.25 ms and
4.278/10.931/37.566 million instructions.
Those are candidate-only observations, with baseline rejection retained; they
are not speedup claims. All A/A, ABBA, RSS and Callgrind observations survive
scoped cleanup.

The exact candidate passed all 70 full checkpoint gates in
1,028.70 seconds, with compiler/test sources
and executable hashes unchanged. `call-invocations-v1/validation-v1/` retains
1,850 release maintainer tests, 123 native maintainer groups, strict Clippy and
format checks, file/harness/layout audits, upstream and backend qualification,
all 7,784 byte oracles, 7,583 independent source checks plus
11 declared fragments, and 4,285 native source checks with
2,364 executable and 314 codegen-only comparisons. Independent token,
literal and layout checks also passed. This publishes callable implementations
and generic invocation as a development checkpoint. Source closure inference,
captures and native adapters still remain; PA24 stays planned and PA25 pending.
