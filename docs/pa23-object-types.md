# PA23 object-type implementation notes

These notes preserve the prerelease investigation and intermediate checkpoints.
PA23 now has a qualified implemented-draft contract; see [the release evidence](pa23-release.md).
PA24 closures and PA25 general constant execution remain outstanding. The
published prerequisite baseline was `a8219d0f`. Earlier planned/pending statements
below describe their recorded checkpoint, not the current release status.

## Representation and queries

`types::Kind::Dynamic` retains a canonical bound shape, complete mixed arguments,
one trait binder and a separate outer object lifetime. Associated bindings
identify both the member and its complete parent application. The shape's auto
trait identities are canonical sets. Source marker bounds and structural proofs,
including auto-only objects, are wired into the ordinary obligation pipeline.
Substitution, inference, variance, shape checking and outlives queries preserve
the distinction between the trait binder and object lifetime. Trait arguments
still contribute type-outlives requirements even with a static object bound.

`abi::Metadata::VTable` is distinct from slice length metadata. A reference or
raw pointer to an object has two pointer-sized words; the object itself is
unsized. Table construction, dispatch, supported upcasting and dynamic tail
projection are now implemented; their validation and outstanding checks are in
[pa23-object-runtime.md](pa23-object-runtime.md).

The first inferred object constructor grew every inference term from 32 to
40 bytes. A compiled layout probe compared the prior and candidate enums:
constructors/values/terms were `[16, 16, 32]` versus `[24, 24, 40]`.
The revised constructor stores a compact identity for lazily allocated object
lifetime metadata, restoring 16-byte constructors and 32-byte terms. A unit
invariant guards these sizes. The probe is retained under
`obj/pa24-pa25-work/layout-probe-v1/`.

Dyn compatibility consumes completed declaration contracts. The inventory
indexes direct members by owner. The query retains only direct supertrait
edges, dispatchable methods and associated members, and memoizes shared
declaration results. A missing declaration is not a cached negative result.
Object uses are validated after declaration completion so a trait method can
mention its own object type without introducing a false declaration cycle.

Instantiated supertrait traversal uses complete quantified clauses, not only
trait declaration IDs. Binder composition reuses the existing typed machinery.
It does not retain a flattened ancestor prefix per trait. Missing-associated-type
checks consider inherited equalities as well as explicit object bindings.
Repeated formation checks share a key containing the shape, complete arguments
and binder arity, excluding the separate outer object lifetime.

## Independent language findings

All outcomes below were probed with the pinned Rust 1.98 compiler. Diagnostic
files and sources are retained under `obj/pa24-pa25-work/`.

- `Self: Sized` members can opt out of dynamic dispatch, including generic
  associated types. By-value `self` methods are also non-dispatchable.
- Associated projections are permitted in dispatchable signatures. The same
  use of `Self::Assoc` as a supertrait argument is incompatible.
- A method requirement `where Self: Trait` is incompatible even when `Trait`
  is the containing trait or an existing supertrait.
- Merely naming `dyn T<Arg>` can be accepted when `Arg` does not satisfy the
  trait's parameter bounds. These annotations must not be rejected as though
  a concrete implementation had already been selected.
- Explicit associated bindings can disagree with a supertrait's equality in
  an otherwise accepted object annotation. This is not evidence that an
  inhabited concrete-to-object conversion exists.
- Object lifetime defaults distinguish early and late function lifetimes.
  `trait T<'a>: 'a {}` can default to static in `fn f<'a>(_: *const dyn T<'a>)`,
  while a bound making `'a` early changes the result. Multiple early trait
  lifetime bounds can be ambiguous; multiple late ones do not produce the
  same diagnostic. Alias and field contexts differ from callable quantifiers.

The default-lifetime implementation uses resolved lifetime identities and
typed alias occurrence summaries. Each alias summary and function binding-time
classification is reused. Reverse lifetime-slot lookup is indexed lazily,
then updated when later named slots are assigned.

The [Reference's lifetime-default section](https://doc.rust-lang.org/reference/lifetime-elision.html#default-trait-object-lifetimes)
does not describe all these distinctions. The pinned compiler's
[binding-time classification](https://github.com/rust-lang/rust/blob/1.98.0/compiler/rustc_hir_analysis/src/collect/resolve_bound_vars.rs)
provides additional implementation guidance; executable controls remain the
authority for the selected language profile.

## Evidence and remaining work

Source auto-trait admission now uses resolver-owned builtin identities,
ordinary bound proof and structural propagation, including auto-only objects.
The pinned Reference's `special-types-and-traits.md` is the rule inventory.
Standard library overrides matter: shared-reference `Send` depends on referent
`Sync`, raw pointers do not implement either, and mutable references are not
`UnwindSafe`. A same-spelled user trait must remain an ordinary principal.

Recursive nominal fields use a greatest-fixed-point query, as the existing
`types/equality.rs` uses for recursive equality. A failing field invalidates
every dependent member of a cycle. The implementation reuses completed field
applications and a reverse dependency worklist, with explicit node/edge
accounting. Caller assumptions and lifetime evidence retain their proof scope.
Missing fields and projected results use the existing readiness/normalization
continuations. Explicit marker implementations and automatic propagation from
opaque defining types remain outstanding.

`dynamic-source-v6/` passed strict Clippy, the file audit, 28 compatibility
controls, 43 annotation controls, five binding-time controls and seven
canonical-object tests. Compatibility also has a shared-supertrait graph work
invariant. PA1–PA18 and PA20–PA22 byte comparisons passed. PA19 passed 1,104
of 1,105 cases: its explicitly marked future-stage trait-object boundary now
reaches conversion checking instead of rejecting the annotation. Its diagnostic
changed from `unsupported type annotation` at byte 79 to `incompatible types`
at byte 85. The original fixture and oracle remain unchanged pending migration
to PA23 as a working positive. No new PA23 public oracles have been blessed.

Concrete erasure now constructs obligations in the existing inference and
scoped proof pipeline. These include `Sized`, the complete principal
application, associated equalities and the separate outer object lifetime.
Conversion facts distinguish slices from objects and carry both concrete and
target types through CFG construction and specialization. At this snapshot,
constant object metadata and executable table lowering still diagnosed their
implementation gap; the later runtime work is recorded below.
`object-conversion-v7/` passed 29 conversion controls, two lifetime outcomes,
three object proof invariants and ten existing projection-normalization tests.
The canonical normalizer now gives object inputs their own explicit binder
scope, including an empty binder; it preserves the separate outer object
lifetime. `object-conversion-v8/` rebuilt the candidate and passed the audit and
all active byte suites except the same PA19 boundary. That case now passes
conversion checking and reports `no applicable method for this receiver` at
byte 96. The committed oracle remains unchanged.

The first conversion checks exposed an erroneous extra dereference of a
reference-valued concrete source. They also established that negative
higher-ranked controls must run the retained lifetime proof after inference.
Inference alone intentionally retains those obligations; it cannot classify
the complete program's outcome. Shared object input folds have a separate
algorithmic work invariant to detect repeated traversal of common type graphs.

Explicit associated bindings can now select quantified supertraits. The source
lookup indexes direct members through the complete instantiated graph once per
root input, sharing the result among bindings and repeated uses. Appended
universal slots coexist with the principal's slots in one object binder.
Comparison may rename appended locals while preserving the principal's used
prefix; unused quantifier slots cannot make equivalent inherited parents appear
distinct. Original proof clauses and binder arities remain intact.

The independent probes in `object-inheritance-probes-v1/` establish that an
associated binding's quantified result lifetimes must occur in its parent
trait's input types. The check consumes unreduced types so projection
normalization cannot invent an input occurrence. `object-inheritance-v3/` passed
strict Clippy, the audit and the focused source/proof suites: 54 annotation
controls, five default-lifetime controls, 33 conversion controls and two
conversion lifetime outcomes, alongside compatibility and typed invariants.
Its separately filtered unit command selected zero tests and is not validation
evidence. The corrected `object-inheritance-v4/` run executed all 460 library
unit tests successfully, including the principal-slot comparison invariant.

The later runtime work implements generic-container lifetime defaults and
independently checks redundant inherited bindings and local reference elision.
Source auto traits remain outstanding, along with broader qualification and
publication of the PA23 contract. These implementation notes do not replace
the assignment release gates.

Object membership now supplies declared-result evidence through the ordinary
scoped trait selector. Associated equations use that same evidence before
normalization. A lazy index per complete object application shares the
instantiated supertrait traversal across trait queries, equations and method
lookup. It is invalidated with the declaration-contract epoch; it stores mapped
clauses, not caller permissions or concrete implementation proofs.

Method lookup finds object and inherited methods even when their trait names
are outside lexical scope. These methods have bound lookup priority while their
actual authorization comes from object evidence, not the caller's parameter
assumptions. Qualified methods and associated arguments/results use the existing
callable pipeline.

`object-members-v5/` passed strict Clippy, the audit (1,085 files), all 462 library
unit tests and the focused object suites. The 34 new member/membership controls
match pinned rustc and check the actual lifetime and loan proofs, including
generic templates. Inference alone accepts a mutable-method call through a shared
reference; the ordinary loan checker rejects it. A first test helper incorrectly
passed generic contracts to the concrete-only verifier; the corrected helper
checks templates separately and supplies exactly the concrete body contracts.
These checks do not invoke native virtual dispatch.

The scaling controls exposed two algorithmic problems. Clause comparison
revisited an object's captured type graph once per inherited application:
128 trait nodes and 128 shared capture levels took 84,991 query steps. Reusing
the existing local-slot occurrence summaries avoids folding graphs with no
renameable locals. The revised counts are 167, 671 and 2,687 steps at sizes
8, 32 and 128. These are this query's charged work, not whole-compiler timing
or memory measurements.

Quantified diamond inheritance also repeated equivalent derived bounds with
different unused binder arities until declaration validation exhausted inference
work. Derived consequence discovery now compares complete clauses modulo local
renaming. Written clauses retain their original identities and declaration
duties. The 128-node quantified case now passes; its object indexing takes
1,395 steps. The ordinary diamond takes 1,017. Repeated requests share the
same index without creating additional types. Raw observations and the earlier
failure trace remain in `object-members-v3/` through `object-members-v5/`.

`object-members-v6/` rebuilt the compiler and reran every active byte suite.
PA1–PA18 and PA20–PA22 passed. PA19 remains 1,104/1,105: its sole future-object
boundary now reaches concrete dispatch and reports `concrete dispatch cannot
use a symbolic assumption` at byte 71. Its committed rejection oracle is
unchanged. No assignment fixture/reference or release status changed.

`object-members-v7/` passed the full release Cargo suite (1,792 tests), all 114
native maintainer groups, harness tests, audit tests and discovery checks.
The native groups exercise the previously supported language against the pinned
backend; they are not native trait-object evidence. The active byte total is
7,385/7,386, with the single boundary difference described above. Compiler
comment clarifications and these notes followed the tested implementation.
No hot-path checkpoint has been published; immutable-binary performance
qualification remains required before publication.
The successful batch's storage cleanup removed 3,848 generated files, reclaiming
7.98 GiB of unshared blocks. Receipt: `obj/cleanup/1792514468782903906.jsonl`.
The active Cargo build cache and existing immutable comparison pins remain.

The subsequent runtime implementation is recorded in
[pa23-object-runtime.md](pa23-object-runtime.md). Dynamic tail layout and lifetime
formation now have focused validation there. PA23 still needs its public
contract, independently qualified fixtures, full checkpoint/release gates and
immutable-binary performance measurements. Existing focused checks do not
constitute a PA23 release.

The obsolete `pa22-work/body-proof-pin` executable was retired after the
prerequisite checkpoint was published. Its hash and retirement receipt remain;
the PA22 release baseline and prerequisite candidate remain pinned.

## Auto-trait implementation checkpoint

The next development slice introduces typed `Auto` obligations for the five
stable markers (`Send`, `Sync`, `Unpin`, `UnwindSafe`, `RefUnwindSafe`). Their
shared spelling identities live in support vocabulary; resolution does not
acquire a dependency on type checking. Prelude exposure excludes the two panic
markers. Resolved aliases preserve identity, and user-defined names still shadow
the builtins.

Structural proofs use an iterative graph of `(type, marker)` obligations.
References can change the required marker (`&T: Send` requires `T: Sync`),
while callable signatures do not create stored-field dependencies. Nominals
inspect substituted fields, not every generic argument. All reachable fields
and external leaves are processed before reverse edges propagate failures to
the entire recursive component. A positive active cycle is never independently
cached. The 16,384-level shared-field control checks a linear work bound.

The trait selector retains a suspended structural query in its existing proof
frame. Hidden projected fields descend through the same normalization stack;
the parent retains child witnesses and their lifetime scope. Builtin assumption
indexing stores auto permissions separately from Sized/Copy/Clone. Immutable
associated and opaque result promises, and explicit/inherited object promises,
can grant an auto bound without exposing private representation.

`obj/pa24-pa25-work/auto-structural-v1/` passes six structural controls and strict
Clippy. `auto-source-v1/` passes those controls, seven dynamic-type invariants,
and 39 independently declared source outcomes checked against pinned rustc.
The latter batch also passes strict Clippy and the syntax-aware audit. These are
focused development checks, not a publication checkpoint or PA23 release.
The subsequent `auto-source-v2/` passes 46 independent source outcomes and all
17 object-dispatch test groups, including 64 native runtime controls. Seven new
runtime controls cover explicit/inherited marker bounds, dropping auto bounds,
principal-to-auto-only upcasts, data-pointer preservation and a constant
auto-only object. Strict Clippy and the syntax-aware audit also pass. Cleanup
removed 128 generated files (0.27 GiB); receipt
`obj/cleanup/1792523775672488338.jsonl`.

Explicit marker implementations, opaque automatic trait propagation, further
edge qualification and full regression/performance qualification remain to be
completed. The public assignment remains planned.

`auto-regression-v3/` passed the existing object annotation/conversion/member
suites and all active text comparisons except the already obsolete PA19
trait-object rejection (7,385/7,386 match). No earlier oracle was regenerated.

A subsequent proof review removed the auto query's conditional projection-summary
shortcut. The structural selector must descend through the associated parent's
proof even when a summary promises the result is Send. The dedicated invariant
supplies that summary explicitly and verifies retained lifetime debts under two
separate caller allocations; both an unrestricted and a static-constrained impl
are checked. Auto assumption matching work is charged to the enclosing query.
`auto-evidence-v2/` passes all 466 library tests, the six structural controls,
46 pinned source outcomes and all 64 object runtime controls. Strict Clippy and
the 1,113-file audit pass. Cleanup receipt:
`obj/cleanup/1792524279340126025.jsonl` (128 files, 0.27 GiB).

`auto-regression-v4/` rebuilds that exact compiler and records its SHA-256 before
rerunning all 7,386 active byte comparisons. The result is unchanged: 7,385
matches and the sole PA19 future-object rejection requiring migration to PA23.

Explicit builtin marker impls now share the ordinary pending-header, candidate,
overlap and lifetime-proof machinery through a typed user/auto identity.
An explicit impl suppresses the structural default for its entire nominal
family. Specialized applications and conditional bounds therefore cannot fall
back to field inspection. Safe/unsafe declarations, local nominal ownership,
empty member sets and generic arguments are checked before publication.
`auto-explicit-v1/` qualified 47 source outcomes against pinned rustc and passed
strict Clippy. Its final file audit found one repeated test-helper filename stem;
the helper was renamed before the next batch. The source harness includes body
region checking, since semantic inference intentionally retains lifetime debts.

Independent probes in `marker-cycle-probes/` establish the pinned toolchain's
cycle behavior. Complete equal predicates can close a pure auto cycle; an
intervening ordinary trait cannot. Static lifetime cycles are distinguished from
fresh-lifetime expansion. The selector retains complete predicate/scope/path
identities for positive coinduction and uses a separate static-preserving shape
only for negative cutoffs.

The first shared-cycle scaling test exposed repeated proof traversal: 16 nodes
required 71,856 steps (`auto-explicit-v1/cyclic-baseline.log`). Provisional answers
are now shared inside their contiguous auto proof segment and invalidated before
a failed/deferred child can affect another field or alternative. A transitive
proof-node flag keeps them out of completed and cross-query caches. In
`auto-explicit-v2/cycles.log`, widths 8/16/32/64 require 256/528/1,072/2,160 steps.
A separate negative/positive/negative control checks invalidation, caller
isolation, and that completed caches contain no cycle-dependent proof.


`auto-explicit-v4/` passed strict Clippy, all 469 library tests, 53 independently
qualified explicit-marker outcomes, all 67 object runtime controls and the
1,119-file audit. All 7,386 active byte cases were compared; only the existing
PA19 future-object rejection differs. Cleanup receipt:
`obj/cleanup/1792527116014946601.jsonl` (134 files, 0.28 GiB).

The next review excludes bare coinductive assumption leaves from provisional
memoization. Full auto proofs can share their existing private witness DAG
inside one exact scope and uninterrupted segment: markers expose no associated
outputs. A failure invalidates the whole segment across its nested scopes.
Region-independent goals use the caller's input scope, allowing irrelevant
quantifiers to disappear without moving predicates containing local lifetimes.

`marker-quantified-probes/` independently establishes four accepted quantified
cycles. `auto-explicit-v5/` passes 60 pinned-rustc source outcomes, all 469 library
tests, the structural/object conversion controls, all 67 native object controls,
strict Clippy and the audit. Its 7,386 byte comparisons retain exactly the same
single PA19 boundary mismatch. No oracle was regenerated. Cleanup receipt:
`obj/cleanup/1792528113092315493.jsonl` (134 files, 0.28 GiB).

The growing cyclic diamond invariant now validates the complete lifetime proof,
not just the selection result. At widths 8/16/32/64 it records:

| Graph | 8 | 16 | 32 | 64 |
|---|---:|---:|---:|---:|
| Plain | 256 | 528 | 1,072 | 2,160 |
| Captured witnesses | 304 | 624 | 1,264 | 2,544 |
| Quantified references | 474 | 994 | 2,034 | 4,114 |

These are actual selector work counts, not whole-compiler timings. The v5 CLI
is pinned with its hash under `auto-explicit-v5/binaries/` for the next changed-path
comparison. Automatic opaque-result markers still need a demand boundary that
retains defining-body and lifetime proof; public PA23 publication and the full
release/performance gates remain outstanding.

## Opaque auto-trait readiness

Opaque marker selection now asks for the defining result's hidden type when its
public promises are insufficient. It substitutes the complete capture arguments,
then retains the hidden type's ordinary auto proof as a child. That path grants
only the requested marker; the hidden representation never authorizes ordinary
traits such as Copy. Revealing an opaque identity starts a new coinductive
segment rather than treating a hidden-type cycle as a structural field cycle.

The selector can suspend on the exact opaque declaration. Repeated polling of an
unready result does no additional selection work. A body dependency worklist types
each demanded defining function once and resumes the caller's existing query and
inference graph, including foreign gaps in its lifetime allocation. Completed
bodies remain complete regardless of declaration preparation order. Active body
dependency cycles diagnose instead of recursively retrying. There is a separate
callback boundary for closing captured hidden lifetimes from verified typed CFGs;
that later region-proof integration remains unfinished.

`opaque-marker-probes/` records fourteen independent Rust probes, including
captured lifetime restrictions, generic caller bounds and trait-method results.
`opaque-cycle-probes/` confirms that self/mutual hidden-auto inference cycles
reject, while a sufficient explicitly declared Send promise breaks the dependency.
The latter four controls are not yet installed in the maintainer case table.

`opaque-readiness-v5/` passes all 470 library tests, 69 independent explicit-marker
source outcomes, strict Clippy and the 1,120-file audit. V6 adds a dependency
scaling invariant and three object/opaque lowering controls. At 8/32/128 defining
functions, ordinary chain inference costs 296/1,016/3,896 steps and retains a peak
of 14 suspended slots. Nested body dependencies cost 632/2,504/9,992 inference steps
and retain 112/448/1,792 slots. Every function completes once, and no suspended
slots remain. These counters cover actual body work and slot capacity, not total
frontend memory.

`opaque-readiness-v7/` passes strict Clippy, all 471 library tests, 70 explicit-marker
source outcomes, the structural/conversion controls, all 70 native object controls,
and the 1,121-file audit. Its active byte result is still 7,385/7,386: only the PA19
future-object rejection awaits migration. Cleanup removed 140 native artifacts
(0.29 GiB), receipt `obj/cleanup/1792529300510982810.jsonl`. The immutable compiler
is pinned under `opaque-readiness-v7/binaries/`; changed-path performance validation
is running against the preceding `auto-explicit-v5` compiler.

Remaining opaque work includes captured-lifetime closure from the defining body's
region proof, resumable final canonical requirements, full trait-method result
qualification, and reuse of demanded CFGs/contracts in later lowering. Incremental
contract collection must consume appended semantic facts once, avoiding a whole
crate scan for each demanded body. PA23 publication and PA24/PA25 implementation
remain outstanding; none is marked complete by this checkpoint.


The completed `opaque-readiness-v7/performance-v1` comparison exposed a 14.8%
instruction regression on the existing opaque-repeated workload: switching every
body in an opaque-containing crate to sparse slots was too expensive. The
`opaque-storage-v1` compiler keeps ordinary dense scratch storage and converts
only a genuinely suspended body's touched nodes and bindings. It does not scan
unrelated file slots. The additional borrowed-local dependency family retains
329/1,409/5,729 peak slots and uses 783/3,111/12,423 inference steps at widths
8/32/128; all suspended slots are released.

`opaque-storage-v1/performance-v1` compares immutable readiness-v7 and storage-v1
binaries on five identical inputs. Each output matches byte-for-byte. The opaque
repeated case drops from 23,526,934 to 20,488,869 instructions, recovering the
pre-feature 20,499,889 baseline. Instruction changes on the ordinary, distinct
capture, chain-128 and nested-128 controls are respectively -0.10%, +0.07%, +0.06%
and +0.03%. Every A/A calibration and five paired ABBA blocks is retained; wall
times are too noisy to support a broad speedup claim. Cleanup receipt:
`obj/cleanup/1792529845071300034.jsonl`.

The storage-v1 focused checkpoint passes strict Clippy, all 74 independent marker
sources, object lowering and the 1,121-file audit. Storage-v2 additionally passes
all 471 library tests and four hidden-object native programs. Active byte suites
remain 7,385/7,386, with only the explicitly documented PA19 future-object boundary.
The next independent probe batch found five source-order failures in final hidden
bounds and nominal formation, including a quantified return bound. Its seven
inputs and independent outcomes are retained in `opaque-finalization-probes/`;
these motivate a resumable final-requirement frontier before hidden publication.


`opaque-finalization-v1` adds retained canonical requirements and withholds hidden
publication until those requirements complete. All 471 unit tests and 87 independent
marker outcomes pass. Its first byte comparison exposed 28 PA21 representation
changes, so v2 retains ordinary and quantified proof paths for inputs without opaque
identities. The new query path publishes immediate results until a real dependency
waits, retains completed-proof reuse and private allocation intervals, and resumes
solved finalization without repeating validators. Interned predicate properties
include the complete trait arguments and equality result when detecting opaques.

V2 passes strict Clippy, the four readiness/scaling invariants, 87 independently
qualified marker programs, all 74 native object programs and the 1,122-file audit.
Canonical finalization at 8/32/128 dependent functions performs 481/2,041/8,281
canonical steps, 478/1,822/7,198 ordinary inference steps and retains
161/665/2,681 peak slots. Every defining body completes exactly once and all
suspended slots are released. Its active byte result is 7,384/7,386: the established
PA19 boundary plus one PA21 `universal-forward` proof representation. That remaining
diff switches the opaque-bearing quantified forwarding proof to the resumable
selection vocabulary; its extra scoped bridge and source labels require review
with the PA23 release. No old reference was regenerated.

The subsequent `opaque-pattern-probes` independently demonstrates a scheduler
ownership defect: a completed child's deferred constant-pattern batch disappeared
when restoring its caller. Scalar non-exhaustive matches could consequently reach
LowIR, and valid tuple patterns failed later as incomplete constructors. V3 retains
one owned batch per completed function and joins those batches once in declaration
order before the existing final pattern pass. Active callers retain their unfinished
suffix in their suspended state. No recursive batch concatenation or per-body scan
of the complete source is introduced. Four independent source controls and a native
constant-tuple-pattern control exercise that fix; V3 validation is in progress.


V3 is qualified through strict Clippy, all 472 library tests, 91 independent marker
sources, all 75 native object programs and the 1,122-file audit. The active byte
result remains 7,384/7,386 with the same two recorded differences. Native cleanup
removed 150 generated artifacts (0.31 GiB), receipt
`obj/cleanup/1792531405021371299.jsonl`.

The immutable V3 compiler is `c6527d414950f6d7467493e21d2a82f0ec1cf68f74af09cdc15ce96ae6f42839`.
`opaque-finalization-v3/performance-v1` preserves A/A calibration, five ABBA blocks,
RSS and instruction profiles against storage-v1 on five identical shared inputs.
LowIR matches exactly in every shared run. Instruction changes are +0.31% ordinary,
+0.08% opaque-repeated, +0.03% distinct captures, +0.06% chain-128 and +0.10% nested-128.
The independently object-qualified finalization family at widths 8/32/128 costs
5,869,167/20,199,647/77,863,267 instructions and has candidate medians of
4.29/7.65/21.43 ms. Wall-time noise remains material; these are observations, not
an overall speedup claim. Cleanup removed the three temporary object files,
receipt `obj/cleanup/1792531510085306689.jsonl`.

The `defining-regions-v3` checkpoint coordinates early defining proofs in
`pipeline`, above sema/body/borrow, and retains each original CFG for lowering.
The incremental contract collector consumes appended lifetime facts once and
keeps variables before obligations. The original direct collector remains the
ordinary path. Early proofs perform promotion and the complete template borrow
checks before publishing portable hidden lifetimes; final lowering still runs
its full template and instance gates.

This checkpoint passes strict Clippy, 472 library tests, ten independently
qualified defining-region outcomes, five native comparisons, the 1,125-file
audit, audit unit checks and the harness. Active byte suites remain 7,384/7,386
with the same documented PA19 boundary and PA21 forwarding representation.
Its immutable binary is
`1716d3506e910968e818737762f3f30899f65708cc86635abf627c3c0affd353`.
Cleanup receipt `obj/cleanup/1792532424053090903.jsonl` records ten native
artifacts removed (0.02 GiB).

The subsequent index review found a new product-cost path: an evaluator created
for every demanded body rebuilt its crate-wide constant-definition index.
The V4 candidate transfers report-owned indexes and cleared scratch storage
between those promotion batches and into the final template pass. Active query
state and caller permissions remain batch-local. New work counters distinguish
constant preparation, consumed lifetime facts and actual CFG builds. Additional
controls cover two hidden results with separate precise captures, static versus
unpaid marker lifetimes in both demand orders, reborrows and promoted results.
V4 qualification and immutable-binary measurements remain in progress.

V4 is now functionally qualified: 472 library tests, all 17 declared defining-region
outcomes, ten native programs, the four growth/source/native groups, strict Clippy,
1,127 audited files and four audit unit tests pass. Its immutable candidate is
`ac46cc710053ff4b0638e63345baf43de2dac21516d41505894a49002395e406`.
At 8/32/128 defining bodies, CFG builds are exact and lifetime-fact consumption is
191/767/3,071. Constant preparation is 81/153/441 steps. Growing unrelated constants
adds exactly two setup steps per declaration across the whole dependency chain.
The six-input comparison preserves exact LowIR; instruction changes on the ordinary
control and zero-unrelated-constant chains remain within 0.084%. With 2,048 unrelated
constants the index reuse saves 0.811% total instructions. This is a bounded setup
improvement, not a claim that all compiler passes have become linear. Later concrete
instance evaluators remain a PA25 integration concern, documented in its handoff.

The public PA23 matrix and reviewed PA19/PA21 changes are now installed. Complete
functional release checks pass; see `docs/pa23-release.md` for the 70-gate results,
public counts, source identities and cleanup. Publication awaits the broad frozen
performance matrix. PA24/PA25 remain unimplemented.

The 38-input broad study is complete; its 29 shared outputs match exactly.
The qualified release record includes the observed instruction and wall-time
overhead. PA23 is ready for publication; PA24/PA25 remain the active goal.
