# PA15–PA17 buildout

PA14 is published at `f940b309477a300f0332b159c71d0405921437bd`.
The active work is PA15 lifetime contracts, PA16 borrow checking and PA17
destruction. These assignments remain planned until their contracts, independent
checks and course fixtures are complete.

## Lifetime representation

Nominal types retain their declaration identity and an interned lifetime-argument
slice. Empty arguments preserve the existing nongeneric path. Physical layout,
Copy and structural-equality queries depend on declaration fields; lifetime
substitution and variance must separately retain the actual argument identities.
Declaration field types are templates, not the types of selected instance fields.

Canonical type and region lists share a typed flat-slice interner. Canonical type
properties summarize the number of enclosing binders needed by free bound
lifetimes. Substitution can return an unchanged type in constant time when it
contains no lifetime escaping into the substituted environment.

`types::substitute::Bound` removes one enclosing binder with an iterative worklist.
It preserves nested function binders, shifts replacement lifetimes under them,
and adjusts references to more distant binders. Its cache belongs to one store
and one fixed argument environment and is keyed by type and binder depth. Reuse
that cache across an instantiation's parameters, result and demanded fields;
never reuse it with another caller's arguments. Work and queued dependencies
have an explicit budget. Nominal argument rewriting reuses scratch storage.

The maintainer tests exercise distinct nominal arguments, retained equality
obligations, nested-binder capture avoidance, separate substitution environments,
10,000-level shared type graphs, missing arguments and resource/depth errors.
Ordinary node facts retain their existing 80-byte layout.

## Declaration and proof responsibilities

PA15 owns lifetime-only nominal and alias parameters, including the boundary
currently declared by PA12. General type and const parameters remain PA18.
Implement explicit and implied outlives environments, field substitution,
variance and fresh call instantiation before publishing a region-proof view.
Early-bound item arguments and late-bound function lifetimes have distinct
instantiation times; function-item coercions must preserve that distinction.

The existing elision implementation counts candidates per input parameter.
Pinned-rustc probes confirm that two parameters with the same named lifetime
still require an explicit output lifetime. One tuple parameter containing that
same lifetime twice permits elision. Preserve those independently checked rules.

The current type view retains pending region equalities, directed coercions and
explicit outlives obligations. The ownership view
proves moves and initialization only. Neither is evidence of valid reference
returns. Return checking must distinguish borrowing a parameter's local slot
from reborrowing its referent, and must account for control flow and legitimate
constant promotion. Inferred local reference types need inference regions;
equating them rigidly to the first initializer's universal lifetime is incorrect.

PA16 adds loan and storage proofs to typed reference operations before those
operations may reach LowIR. PA17 consumes initialization and move facts when
elaborating conditional destruction and cleanup. Passing lifetime contracts alone
must not erase either of those later proof obligations.

## Evidence

The independently declared starting probes are retained under
`obj/pa15-work/contract-probes-1791871629076194885/`: 34 programs with explicit
expected acceptance, pinned rustc results and existing type/ownership outputs.
The original, disproved two-parameter elision hypothesis is preserved separately;
it was corrected before creating any PA15 text oracle.

Implementation and validation are still in progress. No PA15 acceptance totals
or executable-reference claims are published by this buildout record.

The representation checkpoint passes all 2,622 existing course cases with no
oracle byte changes, 352 release tests, the full previous 351-test debug run plus
the final five-test substitution suite, Clippy, rustfmt, 219 audited files and
25 harness tests. Logs are `obj/pa15-work/foundation-*.log`. The frozen checkpoint
binary is `obj/pa15-work/foundation-candidate`, SHA-256
`46dd80470b8fdb1745405c13cd8c0daba0a7ed8c3418f584213b89725903d51e`.

Two sequential CPU-10 performance runs retain 108 observations, A/A calibration,
three paired ABBA blocks per scale, and frozen inputs/binaries with full/count
output equality. No local build or test overlapped timing. Verification is in
`obj/pa15-work/foundation-perf-verified.json`; raw runs are
`obj/perf/1791872583387466624/` and `obj/perf/1791872656600046189/`.
The baseline is PA13's published compiler (also used unchanged by PA14).

| Largest workload | Baseline median ms | Checkpoint median ms | Baseline/checkpoint median KiB |
| --- | ---: | ---: | ---: |
| 64k ordinary type-checked functions | 2,397.77 | 2,396.39 | 464,016 / 464,184 |
| 16k shared nominal constant matches, LowIR | 1,494.80 | 1,456.76 | 239,994 / 240,756 |

These measurements show no material regression in the sampled existing paths.
Host variation is visible in the retained observations; the small timing
differences are not a precise speedup claim. Source-level lifetime instantiation
and proof workloads still need their own measurements when implemented.

## Declaration and inference checkpoint (unreleased)

The working implementation now elaborates lifetime-only nominal/alias parameters,
substitutes selected fields, and computes nominal variance with a bounded monotone
dependency graph. Function schemes distinguish early item arguments from late
call binders. Function-item identity retains early arguments; each call freshens
its late parameters once. Sparse aliases keep caller universals from being
captured by a callee's binder. Local concrete annotations reuse the declaration
elaborator with their owning function's binder, including nested function pointers.
Empty argument lists infer lifetimes, and resolved enum variants accept lifetime
arguments on the enum or the variant according to Rust's placement rules.

Inferred locals and joins receive independent free-region positions. Numeric
referents remain unresolved until ordinary inference completes; arrays retain
one element type, while tuple fields retain separate positions. Function items
keep identity and nested callable binders keep their own lifetimes. The iterative
freshening walk memoizes unchanged subgraphs within one query and bounds work
before expanding distinct positions. Canonical region-presence summaries let
closed type graphs skip traversal. Ordinary node facts remain 80 bytes.

Coercions constrain type shapes and publish their actual/expected types and
selected adjustment separately from equality. All sites survive, even when one
syntax node constrains several targets. Type, pattern and body views expose the
pending obligations. Body IR distinguishes inference variables from actual
borrows; the latter remain gated until reference operations have typed loan
provenance. LowIR rejects pending lifetime obligations explicitly, including
bounds in unused functions with no reference-valued parameters. Move checking
alone cannot authorize their erasure.

This is not a completed region proof. Nominal/alias bounds, implied environments,
higher-ranked coercion subsumption, local annotations with reference inference
holes, borrow-place provenance and promotion still need implementation and
validation. PA15–PA17 remain planned; no new course oracles have been blessed.

The maintainer checks include 76 pinned-rustc acceptance controls, eight focused
inference/proof-boundary tests, and the existing lifetime substitution and
variance invariants. The full 363-test debug run exposed two old assumptions
that every region variable denoted a pattern borrow. Those assertions now count
the actual borrow origins and check provenance; all fifteen pattern tests pass.
The added executable-boundary test and affected lowering suites pass separately.
Final rustfmt/Clippy, all 230 audited files and 25 harness tests pass.

The current course report is **2,564/2,622 byte-identical**, not a clean release
report. Of 58 differences, 57 preserve acceptance and reflect pending lifetime
facts, inferred reference types or the strengthened lowering diagnostic. PA12's
nominal-lifetime boundary now succeeds and must move to PA15 when it is released.
The complete case/source/status review is retained in
`obj/pa15-work/inference-oracle-review.json`. No committed reference was changed.

The frozen binary `obj/pa15-work/inference-candidate` has SHA-256
`a983adfe7f5762aa9b5b25dbdc1834e6d0ee5a9b5810c7d739d5e8b4a0262953`.
Two sequential A/A-calibrated ABBA runs compare it with the published foundation,
retaining 108 observations and exact full/count output equality. No local build
or test overlaps timing. Verification is in `inference-perf-verified.json` under
the same work directory; raw runs are `obj/perf/1791878746162539896/` and
`obj/perf/1791878814074446467/`.

| Largest workload | Foundation median ms | Candidate median ms | Foundation/candidate median KiB |
| --- | ---: | ---: | ---: |
| 64k ordinary type-checked functions | 2,515.10 | 2,520.79 | 464,020 / 465,432 |
| 16k shared nominal constant matches, LowIR | 1,438.57 | 1,486.88 | 240,812 / 240,030 |

The largest ordinary workload is stable; the nominal workload is 3.4% slower
in these samples, with overlapping ranges and substantial A/A variation.
These observations do not establish a precise speed difference.

New lifetime paths have a separate frozen A/A scaling run,
`obj/perf/1791879019976404192/`, with 54 retained observations. All three source
sizes pass pinned rustc before timing starts. At 512/2,048/8,192 functions,
combined paired medians are 40.47/162.50/692.01 ms and
12,808/40,200/151,492 KiB. Each function exercises a higher-ranked pointer call,
an explicit nominal argument, local annotations, reassignment and separate tuple
positions. This measures declaration/inference scaling, not region-proof or
reference-execution performance.

## Directed contract proof checkpoint (unreleased)

The region checker now consumes a typed per-body contract assembled by
`body::regions`; it has no dependency on resolution or semantic analysis.
The adapter groups obligations once, preserving globally allocated region IDs,
source locations, early/late function schemes and all selected coercions.
The proof itself uses bounded directed constraints and immutable assumptions.
Universal requirements are checked against declared and implied bounds;
obligations cannot manufacture assumptions. Younger placeholders cannot escape
into older inference variables. Shared references are covariant, mutable
referents invariant, function inputs contravariant, and nominal parameters use
the completed variance table. Mutable array-to-slice coercions preserve element
invariance. Structural type inference permits callable binder specialization;
the directed proof decides whether the quantifier relationship is valid.

Well-formedness queries derive implied requirements from both parameter and
return types. Recursive nominal definitions propagate only newly found pairs
through typed dependency edges, including inactive enum variants and argument
permutations. Local annotations supply obligations, never assumptions. Queries
cache completed canonical-type results and use iterative, bounded traversal.
Nested callable-local regions do not escape into the body's assumptions.
These rules follow the pinned Reference's
[implied-bound context](https://doc.rust-lang.org/reference/trait-bounds.html#bound.implied.context).
The distinction between function pointers and function items also matters:
joining the same function item retains its coupled early arguments, instead of
reifying its signature merely because two lifetime argument lists differ.

The full maintainer suite passes **381 tests**. The new proof controls include
54 small programs independently checked with Rust 1.98.0, 768 small directed
constraint graphs checked against exhaustive set-inclusion models, deep iterative
chains, shared type DAGs, nested binders and recursive nominal requirements.
The final three source controls were added after the full run; the affected
8-test contract suite passes separately. Rustfmt, Clippy, the syntax-aware audit
of 239 files and all 25 harness tests pass. No broad lint suppression was added;
one cohesive type-relation dispatch has a reasoned Clippy line-count expectation.
The architecture audit initially caught a backwards dependency; it was corrected
by introducing the typed body adapter, without weakening the audit.

The course remains **2,564/2,622 byte-identical**. All 58 differences match the
previous checkpoint's complete source/status/diff review exactly, as recorded
in `obj/pa15-work/regions-oracle-review.json`. No text oracle or release receipt
has been regenerated. PA15–PA17 are still planned.

Frozen `obj/pa15-work/regions-candidate` has SHA-256
`63d5e66bec95fdc7762f18c7102624b84bee018ab7c77e97eebe8fa0e5f036f8`.
Two sequential A/A-calibrated ABBA runs compare it with the preceding inference
checkpoint, preserving 108 raw observations and complete/count output equality.
Runs are `obj/perf/1791882978697896389/` and
`obj/perf/1791883048538542901/`; their verification is
`obj/pa15-work/regions-perf-verified.json`.

| Largest workload | Inference median ms | Contract checkpoint median ms | Median KiB, inference/checkpoint |
| --- | ---: | ---: | ---: |
| 64k ordinary type-checked functions | 2,477.49 | 2,557.26 | 465,498 / 465,518 |
| 16k shared nominal constant matches, LowIR | 1,492.46 | 1,455.65 | 240,154 / 240,168 |

The observed differences are +3.2% and -2.5%, with overlapping ranges and
calibration variation; they do not establish a precise speed change.
The new proof path has a separate frozen A/A run with 54 observations,
`obj/perf/1791883087438955268/`, checked against pinned rustc before timing.
A temporary driver calls the production contract adapter and checker directly;
this does not introduce or claim a released course CLI. At 512/2,048/8,192
functions it takes 31.07/106.31/533.32 ms and 9,986/28,286/101,510 KiB.
Each body has five graph regions, four obligations and twelve solver steps at
all three scales. Full/count output equality, binary/input hashes and every raw
observation are verified by `obj/pa15-work/verify_region_proofs.py`.
No local build or test overlapped any timing interval.

### Remaining proof work and corrected hypotheses

This contract checker is not connected to executable lowering. It still rejects
actual borrow origins until typed place provenance is available, and the existing
LowIR proof gate remains intact. Reference/mutable-reference body operations,
reachability, valid constant promotion, loans and destruction still need their
respective implementations. Nominal/alias bounds and local reference annotations
with inference holes also remain open.

Inferred generic function-item lifetimes need a further correction: atomic
function-item freshening falsely rejects an inferred contravariant item assigned
first at an outer lifetime and then at static. Freshening must preserve one
coupled argument per formal early parameter, including distinct formal positions
that were instantiated with equal arguments. An explicit uninstantiated callable
template is needed; freshening signature occurrences independently is incorrect.
Function-item casts also still use strict binder equality and reject a valid
higher-ranked-to-static specialization. The frozen probes in
`obj/pa15-work/fnitem-variance-1791882664543798824/` and
`obj/pa15-work/rank-implied-1791883219059350487/` retain every result.

A suspected need to grant extra implied assumptions during higher-ranked
function-pointer conversion was not supported by the pinned compiler: both
proposed conversions are rejected by Rust 1.98.0, while an explicit outer-static
bound makes the relevant direction valid. These are now positive/negative
controls. Do not add scoped assumptions merely to make the originally hypothesized
acceptance succeed. The online compiler-development variance guide also contains
an outdated claim about function variance; the
[pinned compiler implementation](https://github.com/rust-lang/rust/blob/1.98.0/compiler/rustc_hir_analysis/src/variance/constraints.rs)
and the independent probes establish that early function arguments do participate
in variance inference.

### Explicit callable casts

The subsequent cast correction now retains a directed lifetime obligation for
an explicit function-item cast while preserving the selected runtime cast.
It permits valid higher-ranked specialization and rejects early-bound-to-universal
casts. The operand receives no extra reification adjustment. Five pinned-rustc
controls and the selection invariant pass; the affected 33 body/type/lowering
and contract tests pass. There are now 59 independent source contract controls
and 382 maintainer tests in total; the full 381-test run above predates this one
additional test. Final lint, audit and harness checks pass. The course's 58
differences remain exactly unchanged (`obj/pa15-work/casts-oracle-review.json`).

The current frozen compiler is `obj/pa15-work/casts-candidate`, SHA-256
`74b0ee0a9486ae7a439acb044c8e47d08cba3aa0a8258f069e90c85e60e4bdaa`.
A further calibrated ABBA run, `obj/perf/1791883861192150219/`, retains 54
observations and exact output equality against the preceding region checkpoint.
At 64k ordinary functions the medians are 2,489.14/2,574.07 ms and
465,476/465,764 KiB. The 3.4% timing difference is within the broad sampled
ranges and A/A variation; no precise performance change is established.
The new cast-and-call proof path has a separate 54-observation frozen A/A run,
`obj/perf/1791883927683502203/`. All inputs pass pinned rustc before timing.
At 512/2,048/8,192 functions it takes 23.30/89.82/360.79 ms and
7,848/19,798/69,310 KiB. Each additional function adds six regions, six
obligations and twelve solver steps. Verification scripts and all source hashes
are retained under `obj/pa15-work/`; no build or test overlapped timing.
Early-argument function-item freshening remains the next correction.

### Callable templates, local annotations and initial borrow-place proofs

This checkpoint supersedes the open callable/annotation items above. Generic
function items now retain typed, uninstantiated early-argument templates in
`types::callable::Templates`. Inferred uses freshen each formal argument once,
then substitute the coupled signature. Distinct formal parameters remain
independent even when their original arguments happen to be equal. Constructors
publish the same template facts. Well-formedness checking retains declaration
bounds for freshened items, and naming reuses the existing substitution cache.

Nominal inline and where-clause lifetime bounds now contribute both obligations
and input/output implied bounds. Written alias bounds are resolved but are not
enforced at uses, matching pinned rustc; bounds of an expanded underlying nominal
type still apply. Ten independent controls distinguish these cases.

Partial local reference/function annotations reuse declaration lifetime
elaboration. `_` stays in the body inference graph; completed annotation roots
supply well-formedness obligations. Annotation children beneath a private callable
binder must not become independent body-level obligations. Completed inferred
signatures also check output-bound lifetime use.

Callable type holes need specialization through nested function results and
references, not just at the outermost function. `types::specialize` performs
bounded iterative specialization, preserves item identity and creates independent
arguments per structural position. A cached canonical-type flag skips closed
subtrees. Only bodies with partial callable annotations enter this inference
path, and its work is charged to the body's inference budget. The original
polymorphic source remains the subject of directed subsumption checking.

These controls exposed an additional generality check: a free inferred region
must be available in the current body, while a younger forall placeholder does
not promise that local extent. The region solver now distinguishes those cases.
This rejects converting a monomorphic `fn(_)` to a universally quantified input,
including under an outer function's contravariant parameter. It preserves valid
nested specialization. A dedicated graph test complements the existing 768
exhaustive inclusion-model comparisons.

The body IR now represents a borrow of a selected place and explicit/automatic
dereference projections. Forming a reference does not read or move its referent.
Borrow evaluation preserves source order, evaluates effectful bases/indices
once, and balances the place stack across diverging operands. Owned local,
static and referenced storage remain distinct. Reference-pattern construction
and executable loan gates remain explicit.

`borrow::regions::check_bodies` consumes this typed CFG for initial provenance
proofs. Persistent reborrow ancestry retains outer constraints through mutable
references and stops at a shared inner reference. The
[pinned rustc reborrow implementation](https://github.com/rust-lang/rust/blob/1.98.0/compiler/rustc_borrowck/src/type_check/mod.rs)
was consulted to validate this distinction; no implementation source was copied.
Eighteen pinned-rustc cases cover local escapes, parameter storage, reborrows,
field/index projections, static storage and CFG reachability. Temporary promotion,
precise storage liveness and loans are still pending. This analysis produces no
executable proof token and cannot authorize lowering.

All **392 maintainer tests** pass. Rustfmt, Clippy all/pedantic with warnings
denied, the syntax-aware audit of 245 files, and 25 harness tests pass. Dense
`NodeFacts` remains 80 bytes. No lint or architecture rule was weakened.
The course report is **2,561/2,622 byte-identical**: the earlier 58 reviewed
changes remain exactly the same, and three PA13 execution-boundary diagnostics
now originate in the ownership checker after body construction. Their status
remains rejection. The complete review and delta are retained in
`obj/pa15-work/provenance-oracle-{review,delta}.json`; no oracle was regenerated.

Frozen compiler `obj/pa15-work/provenance-candidate` has SHA-256
`98e47e2502b8794460bd5ac416ffed7e8a8d1dcc72fae8cfa0e3d1de77ff9f4a`.
Calibrated ABBA run `obj/perf/1791887462882757984/` compares it with the cast
checkpoint and retains all 54 observations and complete/count output equality.
For 64k ordinary functions, baseline/current medians are 2,567.50/2,572.92 ms
and 465,790/465,800 KiB. The 0.2% difference is within observed ranges and
calibration variation; this is not a precise performance-change claim.

A separate frozen A/A run, `obj/perf/1791887533222055806/`, exercises nominal
bounds, inferred generic items, nested callable holes and shared reborrow CFG
proofs together. All inputs pass pinned rustc before timing. At
512/2,048/8,192 functions the medians are 67.22/275.11/1,210.28 ms and
15,574/52,230/197,600 KiB. Each additional function adds thirteen graph regions,
twenty-two obligations and fifty-six solver steps; the two fixed helpers add six
regions/steps. Binary, source and input hashes, every raw observation, and full/
count output equality are verified by the retained `verify_provenance_*.py`
scripts. No build or test overlapped timing.

PA15–PA17 remain planned and unpublished. Next are temporary promotion and storage
facts, mutable borrow/reborrow operations, selected reference-pattern places, the
loan/liveness proof and reference lowering, then conditional destruction. The
assignment contracts, manifests and reviewed text oracles must ship together with
those completed phase boundaries.

## Temporary promotion and discarded-repeat lifetime contracts

The current promotion checkpoint adds `body::promote::analyze`. It demands
constant values from the typed CFG and retains promotion facts separately from
storage and loan proofs. A shared `sema::values::Evaluator` reuses the existing
constant-definition queries, scalar arithmetic, product arena and reference
arena. One evaluator serves the whole program. Demanded temporary DAGs cache
both promotable and runtime-only results and use an explicit work budget;
repeated arrays retain one value and their length. No frontend subprocess or
source-text reconstruction is involved. Ordinary node facts remain 80 bytes.

Promotion roots stop at dereferences. Shared reborrows reuse their reference
value, and field/index reads consume the selected place graph. Constant operand
provenance is distinct from a temporary assigned a constant: division requires
the former, while the index eligibility check can accept identity casts of a
constant. The query consults retained typed syntax only for demanded division
operands, with a lazily built value-to-source index. Groups and attributes are
transparent; blocks still introduce operand temporaries. Constructor arguments,
functional updates and surrounding effects retain the existing CFG evaluation
order. The [pinned rustc promotion implementation](https://github.com/rust-lang/rust/blob/1.98.0/compiler/rustc_mir_transform/src/promote_consts.rs)
and the [vendored Reference](../doc/rust-reference/src/destructors.md) supplied
behavioral guidance; no implementation source was copied.

The original executable statements remain intact. Promotion computes wrapping
integer values while preserving runtime overflow assertions; required constant
evaluation still diagnoses overflow. A typed `Failure` distinguishes evaluation
faults from inconsistent facts or exhausted storage. Bad fixed projections and
query cycles cannot silently become runtime-only values. Immutable static reads
and ordinary function calls are not evaluated speculatively. Empty repeats may
promote while their discarded operand still evaluates exactly once.

Zero-length repetition revealed a separate lifetime distinction. `[p; 0]` does
not store `p`, so its element coercion does not impose free storage-lifetime
bounds. It still checks the element's type and higher-ranked compatibility. The
checker records an alternate proof relation with independent free regions for
that discarded coercion; private binders remain intact. Coercions inside the
operand, including calls or explicit local annotations, remain obligations.
Ordinary zero-length array parameters retain their normal variance and outlives
rules. This does not implement storage liveness or permit reference lowering.

`dev/tests/promotions.rs` has six maintainer tests, including **92 pinned-rustc
acceptance comparisons**, deep shared DAGs, a billion-element repeat represented
without expansion, unchanged executable statements, typed faults and invalid
fixed projections. The complete pre-final-negative-test maintainer run passes
**397 tests**; the final focused run passes all six promotion tests, including
the newly added negative control (398 tests in the tree). Six additional native
rustc probes retain stdout/stderr and prove runtime overflow/shift/index panics
and single evaluation of effects. These are recorded in
`obj/pa15-work/promotion-rustc/runtime/receipt.json`. No RustGM reference execution
is claimed: the ownership/lowering gates remain necessary.

Rustfmt, Clippy all/pedantic with warnings denied, the syntax-aware audit of
**252 files**, and all **25 harness tests** pass. The final course comparison is
still **2,561/2,622 byte-identical**, with the same 61 reviewed differences as the
provenance checkpoint. All three full output streams for those differences were
also compared directly with the frozen provenance binary. No fixture, manifest
or generated oracle was changed. Review and quality logs use the
`obj/pa15-work/promotion-v2-*` prefix.

The frozen current compiler is `obj/pa15-work/promotion-candidate-v2`, SHA-256
`b8bba5e41cccf017adb2f4b62c5111b11dbcecd5adbd8d968642b7e057f93e22`.
The separate promotion/lifetime probe has SHA-256
`50d0e99467a323f029e72cf6d65a5d5fd50ebf40ea4cd2fb8f76a3e9a500c2c5`.
Its CLI is an experiment harness, not an assignment interface.

Calibrated ABBA runs `1791891017672617953` and `1791891086876426960`
compare the current compiler with the frozen provenance compiler. At 64k
ordinary functions, baseline/current medians are **2,554.58/2,555.72 ms** and
465,744/465,618 KiB. At 16k constant-arithmetic/LowIR functions they are
**825.22/814.53 ms** and 149,570/149,616 KiB. The timing ranges overlap;
these observations do not establish a precise speed change. All 108 raw
observations, A/A calibration and full/count output equality are retained.

A separate A/A run, `1791891108741434190`, measures promotion and lifetime
proofs together after checking every input with pinned rustc. At
512/2,048/8,192 functions, medians are **31.23/127.48/516.26 ms** and
9,268/26,106/93,054 KiB. Each additional function contributes one promoted root,
52 promotion steps, five graph regions, six obligations and eighteen solver
steps. All 54 raw observations, input/binary/source hashes and full/count output
equality pass the retained `verify_promotion_v2_*.py` scripts. No build or test
overlapped timing. The earlier v1 binary and ordinary measurement remain as
historical evidence; v2 supersedes them after the additional language controls.

PA15–PA17 remain planned and unpublished. Next are precise temporary scopes and
lifetime extension, mutable/reference-pattern operations, loan liveness and
reference lowering, then destruction. The loan integration must preserve the
discarded-repeat distinction rather than reconstructing the removed storage
constraint from an unused adjustment. Promotion also needs value-sensitive
Drop/interior-mutability qualification before those later capabilities are
admitted; they are not yet supported by the current declaration profile.

## Temporary scope facts and materialization provenance

`body::scopes` now computes Rust 2024 lexical storage endpoints separately from
promotion and inferred loan regions. It consumes configured, resolved syntax
using bounded iterative worklists. Function/block, statement, tail-expression,
boolean-condition, if-then, loop-iteration, match-arm and guard scopes retain
parent identities. A match scrutinee inherits its surrounding temporary scope.
Bindings in a block share its endpoint; existing persistent BodyIR storage tails
retain declaration order and birth, rather than copying each declaration suffix.
These facts do not yet emit CFG cleanup or authorize reference execution.

Lifetime extension follows extending borrow operands, tuples/arrays, casts,
written struct fields, direct tuple constructors, block tails, if branches and
match arms. Index extension follows the base, not the index operand. Repeat
operands, ordinary calls, constructor aliases and struct-update bases do not
propagate extension. Explicit reference patterns qualify through product/or
patterns; by-value `@` bindings and reference-pattern wrappers stop that search.
Const/static initializer extension records a static endpoint. Configuration facts
exclude disabled fields, arms and their bindings from both ordinary traversal
and extension qualification. Pattern scratch is reused within a body, and
nested borrow chains share extension entries rather than repeatedly walking
all preceding operands. No dense whole-crate array is allocated per body.

BodyIR now retains each temporary place's source expression and materialization
point (block plus statement boundary), in addition to its value identity. This
matters for `let x = &{temp()}`: the borrowed block result has the extended scope,
while the call that produced its value is a block-tail expression. The value
moves out of that tail; assigning storage from the value's first evaluation
source would incorrectly end its lifetime. Materialization records are typed
metadata and do not alter existing executable statements or text views.

Expression-only macro expansions preserve place identity. Borrowing `id!(x)`
therefore borrows `x` instead of a copied temporary. Scope qualification sees a
constructor path through a transparent expansion, while an actual block in the
expansion remains a block. Promotion's direct-operand query also preserves this
distinction: `&(7 / id!(2))` qualifies, but `&(7 / id!({2}))` does not. These
queries consume retained expansion syntax; none reparses source or output.

Validation at this checkpoint:

- All 413 maintainer tests pass. The 15 new scope tests include 16 independently
  declared acceptance controls against rustc 1.98.0 and 13 native rustc drop-order
  controls. The latter instrument the same expression shapes with `Drop`; they
  are evidence for scope rules, not RustGM destruction/lowering support. Promotion
  now has 94 pinned acceptance controls, including the two macro cases.
- Scope collection also succeeds for all 977 semantically accepted PA11–PA14
  inputs in a 1,238-file scan. The other 261 are explicitly recorded semantic
  rejections before scope collection, not passing scope tests. The first scan
  found five disabled-syntax failures; those are fixed and covered by regressions.
- Formatting, Clippy all/pedantic with warnings denied, all 257 file audits,
  audit contract tests and all 25 harness tests pass. No unsafe code, dependency,
  broad lint allowance, audit exception, fixture or oracle change was introduced.
- The course report remains 2,561/2,622 exact, with all 61 existing differences
  byte-for-byte unchanged from the promotion checkpoint, including diagnostics
  and exit status. The PA12 unused lifetime-ADT boundary still needs relocation
  when PA15 is released; no reference was regenerated to hide it.
- Native validation of the executable-path changes passed PA11 (194 independent
  checks, 180 executable comparisons), PA13 (538/364) and PA14 (12/12). PA13's 16
  previously documented runtime differences remain explicit. PA12 ran all 97
  independent checks and rejected only that known boundary expectation. These
  runs use the v1 snapshot; the subsequent v2 source changes are confined to the
  scope pass's inactive-node handling, which is not yet in the released CLI.

Frozen compiler/probe binaries, source hashes, corpus outcomes, full course
comparison reviews and raw measurements are under `obj/pa15-work/scopes-*`.
Remaining work includes CFG scope exits and storage liveness, mutable/reference
pattern construction, actual loan checking, reference lowering and destruction.
The coarse region graph's `LOCAL` cap and this lexical plan are not substitutes
for those proofs. PA15–PA17 remain planned and unpublished.

Performance verification retained all 162 raw observations and checked immutable
binary/input hashes, full rendered equality and counter equality. No builds or
tests overlapped the timed measurements. ABBA against the promotion-v2 compiler
measured 64,000 ordinary BodyIR functions at 3,079.34/3,038.41 ms (A/B medians),
with 566,028/567,880 KiB median RSS. The 16,000-function array-pattern LowIR case
measured 1,744.70/1,764.38 ms and 391,432/389,766 KiB. Paired ranges and A/A
calibration overlap; these measurements do not support a precise speed claim.
The new scope driver, including semantics and BodyIR construction, measured
512/2,048/8,192 functions at 69.98/243.64/1,168.59 ms and
16,276/53,930/206,322 KiB. Each added function contributes exactly 25 scopes,
98 scheduled visits and seven materialized temporaries; the two helper bodies
contribute six scopes and five visits. All scaling inputs were accepted by the
pinned rustc before timing. Receipts: `1791894266100335814` (ordinary BodyIR),
`1791894349832423501` (array-pattern LowIR), and `1791894397830680317` (scope A/A).

The scope rules were checked against the pinned
[Rust Reference](../doc/rust-reference/src/destructors.md) and the
[Rust 1.98 scope resolver](https://github.com/rust-lang/rust/blob/1.98.0/compiler/rustc_hir_analysis/src/check/region.rs).
The source lookup was used for behavioral guidance; no implementation was copied.

## CFG storage endpoints and availability

Body construction now consumes the lexical scope plan. Each materialized place
retains its lifetime, CFG birth point and event order. Compact ancestor ranges
record statement, block, condition, arm, guard, iteration and abrupt exits;
failure edges close only their selected path. Failed `let-else` initializers
also end lifetime-extended operands before entering `else`, while preserving
previous enclosing-block temporaries. A pinned rustc `Drop` probe independently
confirms that ordering.

`borrow::storage::check` checks direct temporary-place availability using must-live
and may-live bitsets. It diagnoses uses outside storage lifetime, repeated live
instances on loop reentry, invalid metadata and storage left live on return.
Shared scope masks avoid scanning every temporary individually at every exit.
Index and dataflow storage have explicit 64 MB bounds and work is charged under
an explicit 32-million-step limit. Dereference referents still require a separate
loan proof; this analysis neither supplies that proof nor enables reference
lowering. Local scope identities are retained for later destruction, but owned
SSA operands and destructor elaboration remain unfinished.

Completed bodies retain only the scope tree and exit records. Syntax, binding
and boundary lookup tables and traversal scratch are recycled between bodies.
Oversized hash tables are discarded when a subsequent body uses less than an
eighth of their capacity, avoiding repeated capacity-wide clears after a large
function. Independent scope-query APIs remain available for invariant tests.

Validation at the development checkpoint:

- All 424 maintainer tests pass on the final sources, including 26 focused scope
  and storage tests. Formatting, Clippy all/pedantic with warnings denied,
  the 263-file audit and all 25 harness tests pass.
- A 1,238-input PA11–PA14 scan checks storage successfully for all 839 programs
  reaching BodyIR. The other 261 semantic and 138 body rejections are explicitly
  classified; they are not passing storage checks. Final v2 results, including
  work counts and all output streams, exactly match the v1 snapshot.
- The final storage-v2 course report remains 2,561/2,622 exact, with all 61
  differences byte-identical to the prior checkpoint. No fixture, manifest or
  oracle changed.
  Native results remain PA11 194 checks/180 executions, PA13 538/364 with its 16
  documented runtime differences, PA14 12/12, and PA12's one previously reviewed
  lifetime-boundary expectation mismatch.
- The first measured integration adds work: at 64k ordinary functions the
  baseline/current medians are 2,965/3,513 ms and 567,884/614,812 KiB. At 16k
  array-pattern functions they are 1,685/1,903 ms and 389,760/396,976 KiB.
  This prompted the scratch-reuse change. The initial storage proof scales to
  8,192 functions in 1,258 ms and to 2,048 early-return temporaries in 45.6 ms.
  All 216 initial observations, calibration, input/binary hashes and rendered/count
  equality were checked.

This is an **unreleased development checkpoint**, not a completed PA15–PA17
release. The public assignment status remains planned, and the full course
report still fails on the 61 recorded differences. Do not regenerate references
merely to make this branch green. Remaining work includes mutable borrows and
coercion reborrows, reference-pattern places, NLL loan checks, reference lowering,
Drop elaboration, assignment contracts and independent release fixtures.


The scratch-reuse comparison retains another 216 verified raw observations.
At 64k ordinary functions, v1/v2 medians are **3,169.8/2,971.8 ms** and
615,058/611,536 KiB. At 16k array-pattern functions they are **1,974.7/1,927.9 ms**
and 397,210/396,154 KiB. Storage proofs at 8,192 functions take
1,287.5/1,258.4 ms. At 2,048 early-return temporaries, the medians are
46.5/52.2 ms, with overlapping 41.8–53.7/42.1–54.3 ms ranges. These measurements
support retaining scratch reuse but do not establish a precise overall speedup.
All full renderings, count outputs, work counts, frozen input hashes and binaries
were checked; no local build or test overlapped either timing series. The v2
proof measurements reuse the independently rustc-checked v1 input bytes.

Final source hashes and evidence use `obj/pa15-work/storage-v2-*`; verifier scripts
are `verify_storage_v2_perf.py` and `verify_storage_v2_proofs.py`. The immutable
compiler snapshot is `storage-candidate-v2`, SHA-256
`7f372c3ccfe8c720d300e575b227cf6f274c6e465625259a8b27da0c4195ef88`.
The independent storage probe SHA-256 is
`1db38fbae84dfc6967d65f842826e5ffc9950384a0410c86c951ccca83d0fc57`.

Implementation checkpoint `eb2b634` is published on `work/pa15-pa17`; `main`
remains at the last released foundation. Further validated development checkpoints
will be pushed to this branch while assignment publication remains pending.

[GitHub run 38407785877](https://github.com/vishvananda/rustgm/actions/runs/38407785877)
for `eb2b634` fails the course comparison on exactly the same 61 reviewed paths.
The failed-run log was checked against the local review list. Later workflow
steps did not run after that failure; the formatting, Clippy, architecture and
harness results above are the completed local checks.

## Mutable references and selected reborrows

Body typing now records mutable borrows and implicit reference conversions as
typed facts. Conversions retain the chosen dereference depth, mutability,
unsizing and permitted two-phase reservation. Body construction borrows the
selected place directly; it does not first read and move a mutable reference.
An implicit borrow has a fresh region and an explicit constraint to its
destination type. Contract checking retains mutable referent invariance and
shared referent covariance, including ancestry through nested references.

Coercion propagation retains contextual reference hints separately from inferred
join conversions. Inferred `[p]` moves a mutable pointer, while an annotated
`[&mut u8; 1]` can reborrow it. A block entered with an expected reference
coerces its tail; a later inferred join cannot undo a move through that block.
Direct match arms and array elements can therefore differ from `if` branch
blocks. Earlier array elements and break values can establish a reference hint
for later values: `[p, q]` moves `p` but can reborrow `q`. Match-arm joins permit
later direct reborrows without retroactively changing an untyped block's tail.
Loop breaks preserve their existing inference order. Propagation uses
an explicit worklist with a per-body work budget, and pending constraints retain
the actual type selected before any wrapper type changes.

Two-phase reservation is limited to eligible implicit call-argument reborrows.
Parentheses preserve that site; block tails, branches and explicit mutable
borrows remain immediate. Zero-length array repeats still execute their operand
and preserve its move/borrow operation, but discard its destination lifetime
obligation. Mutable constant-reference evaluation remains explicitly unsupported.

These are inputs to the pending loan checker. Initialization, write permissions,
loan conflicts, NLL extents and activation points still require execution proofs;
the ownership/lowering gates remain in place. This checkpoint does not release
PA15–PA17 or provide reference execution through LowIR.

Validation for the reborrow checkpoint:

- All 432 maintainer tests pass, including eight reborrow tests with pinned
  rustc acceptance controls and independent assertions on retained moves,
  borrows and two-phase reservations. Formatting, Clippy all/pedantic with
  warnings denied, the 264-file audit and all 25 harness tests pass.
- The course report is 2,558/2,622 exact. The 61 previously reviewed differences
  remain; three additional mutable-constant fixtures still reject with status 1
  but now diagnose the unsupported constant-memory operation. The existing
  mutable-array binding view now retains a reborrow instead of region equality.
  No fixture, manifest or oracle was changed. The full course report remains
  failing until the assignment contracts and reference updates are released.
- Native checks on the first immutable reborrow build pass PA11 (194 checks,
  180 executions), PA13 (538/364, with its 16 documented runtime differences)
  and PA14 (12/12). PA12 retains its one previously reviewed boundary mismatch
  among 97 checks. Final reference-inference and control-view corrections are
  checked by the full test run and identical emitted LowIR and rejection streams
  for all 1,230 manifest-listed PA11–PA14 inputs against that native-tested build.
- Three measurement series retain 162 verified raw observations, A/A
  calibration, paired ABBA blocks, frozen inputs and binary/source hashes.
  Ordinary-body and array-pattern full/count outputs equal the preceding storage
  checkpoint. At 64k ordinary functions, baseline/current medians are
  2,982.7/3,089.9 ms and 611,508/611,528 KiB; at 16k array-pattern functions they
  are 1,783.9/1,806.7 ms and 396,442/397,080 KiB. The timing ranges overlap, so
  this does not establish a precise speed difference. New reborrow inputs,
  independently accepted by rustc, scale from 256 to 1,024 to 4,096 functions
  in candidate medians of 23.5, 96.1 and 441.5 ms, with 8,562, 21,954 and
  77,216 KiB median peak RSS. Their retained IR counts grow linearly. No local
  RustGM build or test overlapped the timing series.

Final source and measurement receipts use `obj/pa15-work/reborrows-v2-*`;
`verify_reborrows_v2_perf.py` validates all raw observations and hashes. The
immutable compiler is `reborrows-candidate-v2`, SHA-256
`b933aa6a8053953b12b1673ef504d144b0f36a35ee058bb2286218b67c774c7a`.

## Reference-pattern places

Body construction now retains selected dereference places and by-reference
captures for fixed-size patterns: parameters, local bindings, matches,
conditional patterns, arrays, products and enum alternatives. Each borrowed
occurrence has its exact solved reference type, separately from the shared
binding definition's canonical type. This preserves region constraints when
different alternatives bind the same name from different places, including an
unselected alternative behind an irrefutable first choice.

Guarded borrowed captures use anonymous reference-storage slots. Guard views
alias those slots with the correct reference type, and their contiguous local
range excludes the storage slots. An alternative that captures by value still
views its original place until success, even when another alternative borrows
into the same binding definition. Projection recipes retained for guard
protection and retry planning do not form runtime addresses for inactive enum
variants. Actual tests and captures emit their required projections.

Dynamic slice and text pattern execution remain unsupported. The existing
ownership and LowIR boundaries still require region and loan proofs for
reference operations. These body facts do not establish alias permissions,
non-lexical lifetimes or safe reference execution. Reference lowering and Drop
elaboration remain pending, and PA15–PA17 remain planned.

Validation for the pattern-place checkpoint:

- The 435 maintainer tests pass across the full run and affected-suite reruns.
  The full run exposed two obsolete boundary assertions; the corrected tests
  now reach the later reference-proof diagnostic and both affected suites pass.
  Three new tests cover occurrence types, guarded storage and lifetime contracts
  independently checked against pinned rustc 1.98.0. Formatting, Clippy
  all/pedantic with warnings denied, the 265-file audit and all 25 harness tests
  pass.
- The course report is 2,555/2,622 exact. All 64 previously reviewed differences
  are unchanged; three additional PA13 boundary fixtures still reject with
  status 1 and now reach the later reference-proof diagnostic. No fixture,
  manifest or oracle changed. The course report remains failing on this
  unreleased development branch.
- All 1,230 manifest-listed PA11–PA14 inputs were compared with the preceding
  reborrow checkpoint through LowIR emission. Every status and every successful
  output stream is identical. Among rejected inputs, 91 diagnostics change as
  previously gated pattern operations reach later checks. This is an output
  comparison against the previously native-qualified checkpoint, not a new
  claim of native reference execution.
- Three measurement series retain 162 verified raw observations, A/A
  calibration, paired ABBA blocks, source/binary/input hashes and full/count
  output equality. At 64k ordinary functions, baseline/current medians are
  3,032.6/3,268.0 ms and 611,516/611,404 KiB; at 16k guard-retry functions they
  are 1,346.4/1,372.0 ms and 290,346/292,646 KiB. Timing ranges overlap and
  paired blocks disagree on the direction, so these measurements do not
  establish a precise speed difference. New guarded reference-pattern inputs,
  independently accepted by rustc, scale from 256 to 1,024 to 4,096 functions
  in candidate medians of 22.7, 78.8 and 314.3 ms, with 8,578, 22,002 and
  77,664 KiB median peak RSS. Their retained IR counts grow linearly. No local
  RustGM build or test overlapped the timing series.

The immutable compiler is `obj/pa15-work/pattern-places-candidate-v1`, SHA-256
`5d90b508ac276600893d97777e775006e901a9064dd0b9aeae1c8ac18b5e389a`.
Source hashes and validation receipts use `obj/pa15-work/pattern-places-v1-*`.
`verify_pattern_places_v1_perf.py` checks the raw measurement receipts.

## Reference access permissions and initialization

Ownership analysis now inspects borrows and dereferences before reaching the
pending loan/region gate. Sparse move paths retain the closest owned pointer
path and an owned/shared/mutable access capability. Dereferencing never creates
move paths for borrowed contents. A store through a reference therefore cannot
initialize its pointer, and a referent use cannot move that pointer implicitly.
Pointer reads require initialized storage; borrowing an owned place requires
its selected fields to be initialized. Non-Copy moves through references reject,
including zero-sized referents.

Mutable access follows the [Reference's place rules](../doc/rust-reference/src/expressions.md#mutability).
An immutable binding can contain a writable mutable reference. Shared access
in an enclosing reference chain prevents mutable access, while ordinary owned
fields and array elements containing mutable pointers preserve their referent
capability. Direct mutable borrows still require a mutable local or temporary;
anonymous parameter input storage supports `ref mut` parameter patterns.
Immutable statics and match-guard bindings do not grant mutable access.

Internal guard liveness treats borrowing a view as a use. Its physical-path
write check distinguishes changing pointer storage from changing a referent.
User-reference aliases still require the pending loan analysis. No reference
operation receives an executable `Checked` token from this checkpoint: region
contracts, storage/loan liveness, reference alignment and lowering still need
integration before PA15–PA17 can be released.

Validation for the access checkpoint:

- Three new tests compare 44 independently declared programs with pinned rustc
  1.98.0. They cover nested shared/mutable access, projected pointers, anonymous
  parameters, moved and uninitialized pointers, partial moves, reborrows,
  statics and guard bindings. Accepted programs must reach the remaining proof
  gate; rejected programs must report their specific access error.
- All 438 maintainer tests pass across the full run and the affected boundary
  rerun. One existing test now expects the specific shared-storage error in its
  mutable-through-shared case. Formatting, Clippy all/pedantic with warnings
  denied, the 266-file audit and all 25 harness tests pass.
- The course comparison remains 2,555/2,622 exact on the same 67 reviewed paths.
  Only `pa13/tests/spec/boundaries/execute-mutable-through-shared.rs` changes
  from the preceding checkpoint, still rejecting with status 1 but now reporting
  its access error. No fixture, manifest or oracle was changed.
- All 1,230 manifest-listed PA11–PA14 LowIR inputs retain their previous statuses
  and identical successful output streams. The same single rejected input changes
  its diagnostic. The native backend and successful emitted programs are unchanged.
- Measurements retain 162 verified observations with immutable binaries and
  inputs, A/A calibration and paired blocks. At 64k owned-path functions,
  baseline/current medians are 5,219.2/5,201.0 ms and 923,306/923,198 KiB.
  At 16k guard-retry functions they are 1,489.5/1,524.0 ms and
  292,666/292,146 KiB; timing ranges overlap. Their complete rendered and count
  outputs match. New reference-access inputs are independently rustc-accepted
  and scale from 256 to 1,024 to 4,096 functions in candidate medians of
  16.7, 63.9 and 268.7 ms, with 6,470, 16,666 and 52,706 KiB peak RSS.
  This last series measures access inspection followed by the mandatory loan
  rejection, not completed reference compilation. Every rejection status and
  diagnostic is checked; separate BodyIR counts and full renderings agree and
  grow linearly. No local RustGM build or test overlapped the timing series.

The immutable compiler is `obj/pa15-work/permissions-candidate-v1`, SHA-256
`9bc9485f0065659bb6ba45ab3c03ecd106d06add4165602023ffb2fa815c1ad8`.
Source hashes and validation receipts use `obj/pa15-work/permissions-v1-*`.
`verify_permissions_v1_perf.py` checks the raw measurement receipts.

## Retained lifetime graphs and CFG use liveness

Lifetime contract proofs now retain their original directed obligations,
declared assumptions and body-level region identities. Later analysis can
consume these facts directly without rebuilding a type relation or parsing a
rendered inspection. Nested callable placeholders remain local to their graph;
they cannot masquerade as a named lifetime of the enclosing body.

Implicit reference adjustments now also retain the relation from the fresh
runtime reborrow to its destination. The relation uses the proof's expected
type, including the freshened expectation for discarded zero-length repeats.
It therefore preserves destination uses without incorrectly requiring a
discarded operand to have the array annotation's lifetime.

The new backward CFG pass computes use-point seeds for locals and evaluated
values, including values held across later call arguments. It handles branch
and loop successors, imaginary ownership edges, overwrite and storage kills,
internal guard views and unreachable blocks. Whole aggregate uses retain their
free type regions. Zero-length arrays have no runtime element regions; their
element types still require well-formedness. The cached type traversal shifts
free regions through callable binders and includes nested referents.

Sparse point propagation consumes the retained directed requirements. It never
uses the abstract contract-only `LOCAL` sentinel as a CFG extent or as an edge
between unrelated local lifetimes. CFG bitsets, sparse words, worklists and
seeds have explicit storage/work budgets. This follows the distinction between
[liveness seeds, region constraints and loan checking](https://rustc-dev-guide.rust-lang.org/borrow-check/region-inference.html).

These APIs are tested together but are not yet an executable loan proof.
Universal external extents, loan generation and kills, overlapping places,
two-phase activation, storage conflicts and alignment remain to be integrated.
In particular, a region can contain disconnected use intervals; a loan killed
between them must not reappear just because that region becomes live again.
The existing execution gate remains mandatory, and PA15–PA17 remain planned.

Validation for this checkpoint:

- All 446 maintainer tests pass. New tests compare sparse propagation with an
  independent set-closure model on 128 graphs, preserve original constraint
  source identities, check root versus nested binders, and cover sparse large
  point identities and budget/identity errors. Thirteen independently declared
  source cases are checked with pinned rustc for last use, overwrites, branches,
  loops, pending call operands and empty arrays. Additional typed tests check
  implicit reborrow edges and imaginary-edge liveness.
- Formatting, Clippy all/pedantic with warnings denied, the 270-file audit and
  all 25 harness tests pass. No lint allowance was added.
- The course report remains 2,555/2,622 exact on the same 67 reviewed paths.
  Only the existing PA13 mutable-array inspection changes from the preceding
  checkpoint: it now displays the additional pending reborrow constraint.
  No acceptance status, fixture, manifest or oracle changed. This unreleased
  branch's course report still fails until its contracts are released.
- All 1,230 manifest-listed PA11–PA14 LowIR inputs are byte-for-byte identical
  to the preceding permissions checkpoint, including rejection diagnostics.
- Measurements retain 162 verified raw observations with immutable binaries
  and frozen inputs, A/A calibration and paired ABBA blocks. At 64k owned-path
  functions, baseline/current medians are 5,464.9/5,308.8 ms and
  922,726/923,232 KiB; at 16k guard-retry functions they are
  1,475.3/1,468.3 ms and 292,176/292,176 KiB. Timing ranges overlap.
  Both workloads have identical complete rendering and count outputs.
  A separate driver exercises the new region-point APIs at 256/1,024/4,096
  functions, with candidate medians of 25.9/70.1/353.0 ms and
  7,090/17,656/61,048 KiB. Points, seeds, retained masks and work counters grow
  linearly; complete point sets and counters agree across repeated runs.
  This measures the new analysis APIs, not completed borrow checking or native
  reference execution. The largest input is independently accepted by pinned
  rustc. No local RustGM build or test overlapped these measurements.

The immutable compiler is `obj/pa15-work/nll-candidate-v1`, SHA-256
`926189080f845939414568281ad3f6cea5a483af44f89f854fad30f4046e0a7b`.
Source hashes and validation receipts use `obj/pa15-work/nll-v1-*`.
`verify_nll_v1_perf.py` checks the raw observations and the separate probe's
source/binary identity; `nll-probe.rs` records that probe's exact scope.

## Integrated loan checking and the combined proof boundary

Ownership inspection now runs lifetime-contract, storage, owned-access and loan
checks together. Valid reference programs can reach the ownership view; invalid
aliasing or escaping storage receives a semantic diagnostic. Lowering requires
the new combined `borrow::Checked` token, whose constructor is private to this
pipeline. Inspection retains per-body facts, while compilation releases them
after each body. Synthesized tuple-struct and enum constructors receive lifetime
contracts from their existing typed signatures.

The forward loan pass consumes the retained region solution and backward CFG
use liveness. Root universal and static regions cover the body; nested callable
placeholders and the contract-only `LOCAL` sentinel do not become external
extents. Loan reachability is separate from region membership: killing a loan
between two use intervals prevents its later resurrection. Implicit two-phase
reborrows reserve at creation and activate at the consuming call, following
typed coercion identities. Explicit mutable borrows remain exclusive from
creation. See the Rust compiler's
[two-phase borrowing model](https://rustc-dev-guide.rust-lang.org/borrow-check/two-phase-borrows.html).

A shared canonical place index compares fields, fixed pattern positions and
subranges, conservatively overlapping dynamic indices while retaining later
field disjointness. Access depth distinguishes reading a referent from replacing
or ending its pointer storage. Loans behind shared references rely on retained
parent lifetime constraints. Temporary ends consume existing storage event and
edge indexes; storage scope walks and all loan/path/dataflow allocations have
explicit budgets.

Guard protection uses the same place comparisons when references occur. It now
rejects mutations through a protected referent and mutable borrows passed to
helpers, while accepting disjoint fields and paths that diverge before a later
guard use. Nested guard views resolve to their selected source places. Ending
an internal view's pointer storage does not end the selected referent. Pinned
rustc independently accepts a reference to a selected copy binding escaping the
guard into a later use; the hidden view slot must not give that reference an
artificially short lifetime.

Reference code generation remains explicitly gated on alignment and
representation checks, including reference operations in unused functions.
Thin/fat reference lowering, dynamic slice/text execution and PA17 destruction
are still incomplete. This checkpoint does not activate PA15–PA17 or change any
assignment fixture, manifest or text oracle.

Validation for this checkpoint:

- All 452 maintainer tests pass. The new combined checker has 65 independently
  declared source cases checked against pinned rustc 1.98.0, covering last uses,
  alias conflicts, reborrows, moves, pointer replacement, two-phase activation,
  disjoint places, loops, temporary ends, zero-length repeats and guard views.
  CLI checks verify ownership acceptance, semantic rejection and the remaining
  reference-lowering boundary.
- Formatting, Clippy all/pedantic with warnings denied, the 274-file audit and
  all 25 harness tests pass. No lint allowance was added.
- The course report remains 2,555/2,622 exact on the same 67 reviewed paths.
  Six rejection diagnostics change from the preceding checkpoint: five now
  report pending alignment/representation work, and the escaping reference
  reports its actual invalid lifetime contract. No acceptance status changed.
  This unreleased branch's course report still fails until contracts release.
- All 1,230 manifest-listed PA11–PA14 LowIR inputs retain their statuses and
  identical successful outputs. The same six rejection diagnostics are the
  only differences from the preceding liveness checkpoint.
- Measurements retain 162 verified raw observations with frozen binaries and
  inputs, A/A calibration and paired ABBA blocks. At 64k owned-path functions,
  baseline/current medians are 4,730.0/5,038.2 ms and 923,186/948,998 KiB; at
  16k guard-retry functions they are 1,314.4/1,434.0 ms and
  292,372/297,074 KiB. These CLI paths now perform the combined checks.
  Time medians rise 6.5%/9.1%, with overlapping ranges and noisy calibration;
  memory rises 2.8%/1.6%. Both complete rendering and count outputs agree.
  A separate frozen driver exercises the combined reference checks at
  256/1,024/4,096 functions, with candidate medians of 27.0/98.9/455.0 ms and
  7,846/21,928/76,096 KiB. Each generated function contributes two loans,
  47 points, six region words and 983 work steps; one shared helper contributes
  no loans. The largest input is independently accepted by pinned rustc.
  Driver rendering and count outputs agree across repeated runs. This measures
  reference checking, not reference code generation. No local RustGM build or
  test overlapped these measurements.

The immutable compiler is `obj/pa15-work/loans-candidate-v1`, SHA-256
`257d98849d92703f1c7e0907d787735172cd99dfb9e278724d42d3aeef466e2f`.
Source hashes and validation receipts use `obj/pa15-work/loans-v1-*`.
`verify_loans_v1_perf.py` checks raw observations, source/binary/input hashes
and linear counters; `loans-probe.rs` records the separate driver's scope.

## Reference alignment across reachable and discarded code

Reference formation now checks the required ABI alignment before ownership
inspection or LowIR lowering completes. Taking a reference to an unaligned
packed field fails even when the reference is unused, the field is zero sized,
or its function is never called. By-value field reads remain valid. Explicit
borrows, borrowed pattern captures and implicit guard binding views are covered.

The check separates pointer storage from the pointed-to storage. A dereference
resets the alignment requirement to that of the referent; loading a reference
from a packed field does not make its pointee unaligned. Selected field and
array projections retain the containing storage's alignment limit. Required
alignment for references, slices and text does not require inventing a fixed
size for an unsized referent.

An independent probe found that BodyIR discards operations after unconditional
returns, while Rust still diagnoses unaligned references there. Type checking
therefore retains sparse packed-place obligations before CFG construction.
The per-body traversal follows typed expression selections and selected pattern
graphs, memoizes place provenance, resets at explicit/automatic dereferences,
and charges traversal and pending children against a work budget. It records
only packed obligations and adds no field to ordinary node facts. The later
alignment check consumes those obligations, including unreachable explicit
borrows, let patterns, conditions, match captures and guarded captures.

The same check validates remaining generated BodyIR places with memoized
projection alignment. Sources already covered by retained obligations reuse
those facts. This also covers the maintainer's synthetic guard-view component
fixtures. The old function-emission-only guard check is removed: validation now
precedes function demand, and correctly stops at a reference dereference.

Validation for this checkpoint:

- All 453 maintainer tests pass. Thirty-two independently declared source cases
  agree with pinned rustc 1.98.0; ownership rendering, counts and LowIR rejection
  are checked. They include packed alignment caps, nested products/arrays,
  zero-sized and explicitly aligned values, pointer storage versus referents,
  guard bindings, and unreachable code. Existing synthetic guard-view tests pass.
- Formatting, Clippy all/pedantic with warnings denied, the 277-file audit and
  all 25 harness tests pass. No lint allowance was added.
- The course report remains 2,555/2,622 exact on the same 67 reviewed paths.
  Five boundary diagnostics now name pending representation checks, since
  alignment has been implemented. No acceptance status, fixture, manifest or
  text oracle changed; the unreleased branch's report still fails.
- All 1,230 manifest-listed PA11–PA14 LowIR inputs preserve their statuses and
  successful outputs. Only those five boundary diagnostics differ from the
  preceding loan-checking checkpoint.
- Measurements retain 216 verified raw observations, immutable binaries and
  inputs, A/A calibration and paired ABBA blocks, with complete rendering and
  count equality. At 64k owned-path functions, baseline/current medians are
  5,265.3/5,259.8 ms and 949,792/949,936 KiB; at 16k guard-retry functions they
  are 1,341.1/1,327.8 ms and 297,064/296,882 KiB. The reference workload at
  256/1,024/4,096 functions measures 28.8/114.8/454.9 ms before and
  30.0/139.1/450.2 ms after the change. A separate workload exercises retained
  packed-field obligations, byte-array projections and guard captures: its
  medians are 32.2/115.1/518.9 ms before and 32.6/128.0/545.0 ms after.
  At the largest packed scale, memory is 79,420/79,748 KiB. Timing ranges
  overlap; all observations and calibration remain available, without a precise
  speed claim. Both reference workloads have linear inspection counts and their
  largest inputs are independently accepted by pinned rustc. No local RustGM
  build or test overlapped the measurement series.

The immutable compiler is `obj/pa15-work/alignment-candidate-v1`, SHA-256
`466294bfad1e9e09c44c18df7dcb77108abaea04cac021f6ac7a1f9b907cb9b6`.
Source hashes and validation receipts use `obj/pa15-work/alignment-v1-*`.
`verify_alignment_v1_perf.py` verifies raw receipts, immutable identities,
output equality and the reference workloads' count scaling.

Reference representations and execution are still gated. Persistent backing
for promoted and static places must precede returning references from calls;
then thin/fat pointer lowering, dynamic slice/text operations and PA17
destruction remain. PA15–PA17 are still planned assignments.

## Persistent data and constant-place identity

LowIR now retains globals, initializer items and address targets as typed IDs
and flat arenas. Reserving a global before initializing it supports forward
and cyclic address dependencies. The writer emits explicit object size,
alignment, read-only storage and signed relocation offsets. Integer data items
preserve exact floating-point bits without converting through decimal text.
Existing programs without globals retain their previous text and count output.

Initialization validates the complete item sequence before appending it. It
rejects invalid identities, bit widths, spans, repeated initialization and
incompatible item alignment. The pinned backend naturally aligns typed data
items and requires object alignment not to exceed object size. Consequently,
this interface rejects unaligned pointer relocations rather than changing
packed layout. Lowering packed pointer-containing constants needs a separate
supported strategy; aligned backing for zero-sized referents must also satisfy
the backend's object-span rule.

Constant places now retain sparse parent/index projection records. An immutable
reader exposes completed values, original static identity, projection ancestry,
interned bytes and compressed repeated products. It cannot start an evaluator
or revisit syntax. Equal-valued statics remain distinct storage, and selecting
a static subobject no longer loses the path needed to compute its address.
Projection records refer to earlier locations and use flat ownership.

Validation for this checkpoint:

- All 456 maintainer tests pass. Seven generated LowIR programs execute through
  the pinned backend, covering forward and negative relocation offsets,
  function pointers, cyclic addresses, NaN payload bits, negative-zero bits
  and all 128 integer bits. Invalid initializers leave the data arena unchanged.
- Independently rustc-accepted source checks nested static/product/array
  projections, explicit and automatic dereferencing, equal-valued statics,
  promoted temporary ancestry, exact text bytes and a million-element repeat
  retained without expansion.
- Formatting, Clippy all/pedantic with warnings denied, the 280-file audit,
  all 25 harness tests and `make test-backend` pass. No lint allowance was added.
- The course report remains 2,555/2,622 exact on the same 67 reviewed paths.
  Every reviewed discrepancy is byte-identical to the alignment checkpoint;
  no acceptance status, fixture, manifest or oracle changed. All 1,230
  manifest-listed PA11–PA14 LowIR inputs preserve their complete outputs,
  including rejection diagnostics. The unreleased branch's report still fails.
- Measurements retain 216 verified observations with immutable binaries/frozen
  inputs, A/A calibration and paired ABBA blocks. Full rendering and count
  outputs agree throughout. At 64k owned-path functions, baseline/current
  medians are 5,087.9/5,055.2 ms and 949,952/949,956 KiB; at 16k guard-retry
  functions they are 1,401.5/1,427.9 ms and 296,864/296,860 KiB. Constant-place
  workloads at 1k/4k/16k functions measure 40.5/160.3/827.0 ms before and
  41.5/164.7/828.1 ms after. Constant-reference chains at depths 128/512/2,048
  measure 5.0/10.9/35.3 ms before and 5.0/10.3/35.9 ms after. Inspection counts
  grow linearly in both constant workloads. Timing ranges overlap; this is not
  a precise speed claim. No local RustGM build or test overlapped timing.

The immutable compiler is `obj/pa15-work/data-candidate-v1`, SHA-256
`db16d04fa8636fc7bb3902c1dc0c4e57299c5b17ddca4c8e64ce7eff8a67f2f6`.
Source hashes and receipts use `obj/pa15-work/data-v1-*`.
`verify_data_v1_perf.py` checks all raw observations, source/binary/input hashes,
output equality and constant-workload count scaling. These CLI measurements
exercise retained constant provenance; global-data serialization is currently
covered by the component/native checks rather than source-driven lowering.

This checkpoint supplies typed storage and read-only semantic access. Rust
reference execution is still gated until lowering connects checked places to
persistent data. Thin/fat reference lowering, dynamic slices/text and PA17
destruction remain; PA15–PA17 are still planned assignments.

## Lowering completed constants into persistent storage

The typed body now collects named constant/static demands in one shared query
session before LowIR construction. Static initializer evaluation has its own
entry point: it does not pretend to read or move the static, and promotion's
existing prohibition on reading static storage remains intact. Lowering receives
only an immutable constant reader. Its mutable layout cache is restored to the
semantic result on both successful construction and errors.

Lowering reserves each static, named constant object, promoted root and constant
location before queuing its initializer. Function addresses can demand further
functions while draining this queue. Constant projection ancestry is resolved
iteratively through canonical array strides and selected variant/field offsets.
Reborrowing an existing shared constant reference retains its location identity.

The data serializer uses a bounded flat worklist and exact integer bit patterns.
It writes canonical padding and packed scalar bytes, retains compressed zero
runs, and memoizes zero-filled products without expanding repeated arrays.
It charges pending sibling work before expansion, including large nonzero
repeats. Zero-sized persistent objects get aligned, nonempty physical backing.
Named aggregate constants produce fresh runtime snapshots, so mutating one use
cannot change another use or write the read-only backing.

The pinned backend supports naturally aligned address relocations. For packed
pointer fields, lowering instead writes zero bytes at their exact offsets and
queues typed stores in the entry adapter. Only affected backing becomes writable;
all fixups execute before the Rust entry call. This covers unaligned function
pointers today and provides the same mechanism for reference data. It does not
change CPPGM or reinterpret a packed field at a different offset.

Validation for this checkpoint:

- All 458 maintainer tests pass. Fourteen independent source vectors execute
  successfully with pinned rustc and the CPPGM backend, covering named scalars,
  static subobjects, independent constant snapshots, negative and payload enum
  data, packed integers and function pointers, repeated packed pointers,
  floating-point values, over-alignment and huge zero-sized arrays. A nested
  million-element zero repeat emits one 16 MB zero run without enumerating its
  elements. A nonzero repeat exceeding the work budget fails before expansion
  and produces no partial output.
- The former maintainer rejection of an ordinary named constant inside a match
  is now an independently executed positive case. General constant evaluation
  and reference-execution boundaries retain their rejection tests.
- Formatting, Clippy all/pedantic with warnings denied, the 285-file audit,
  all 25 harness tests and `make test-backend` pass. No lint allowance was added.
- All 1,230 PA11–PA14 LowIR inputs preserve previously successful output bytes.
  Exactly two former rejections now succeed: PA11's constant/static boundary
  fixtures. Both were independently compiled and executed with pinned rustc
  and CPPGM. The course report is therefore 2,553/2,622 exact, with the previous
  67 reviewed differences unchanged and these two additional contract changes.
  No assignment fixture, manifest or oracle was changed or blessed. The working
  branch remains unreleased, and the course report still fails.
- A larger independently executed source contains 4,096 packed static arrays
  and 8,192 pointer fixups. Both native implementations compute the declared
  result. Its LowIR contains 4,096 globals and 16,384 initializer items.
- Measurements retain 270 verified raw observations: four existing workloads
  use A/A calibration followed by paired ABBA blocks, and the new packed-storage
  workload uses A/A scaling because the old compiler rejects that capability.
  All binaries and inputs are frozen, and complete rendering/count outputs agree
  between the compared runs. At 64k owned-path functions, baseline/current
  medians are 5,901.6/5,659.6 ms and 949,230/949,870 KiB; at 16k guard-retry
  functions they are 1,555.1/1,583.5 ms and 296,820/297,086 KiB. Constant-place
  workloads at 1k/4k/16k functions measure 47.6/179.8/777.7 ms before and
  47.5/163.5/784.8 ms after; largest-scale memory is 132,660/134,626 KiB.
  Reference chains at depths 128/512/2,048 measure 4.6/10.4/33.5 ms before and
  4.7/10.3/35.1 ms after. Timing ranges overlap; no precise speed claim is made.
  New packed-storage medians at 256/1,024/4,096 arrays are 18.8/63.2/288.0 ms
  and 7,810/18,558/61,194 KiB. Inspection counts grow linearly for both constant
  workloads and the new data workload. No local RustGM build or test overlapped
  the timing series.

The immutable compiler is `obj/pa15-work/persistent-candidate-v1`, SHA-256
`962faf4aa4da0f8bda7ea276783cb8500d690bc48dda74355efce464bbe34738`.
Course/native/performance receipts and final source hashes use
`obj/pa15-work/persistent-v1-*`; `verify_persistent_v1_perf.py` verifies the
complete raw series, immutable identities, output equality and count scaling.
Maintainer checks are recorded in `storage-full-tests2.log`, `storage-native3.log`
and `storage-final-quality3.log` in the same directory.

This materializes the existing constant evaluator's supported subset to supply
the storage needed for PA15 references; it does not implement PA25's general
const-function, const-generic or memory-evaluation contract. Static address
cycles still need evaluation support distinct from value-read cycles. The
source reference gate remains until thin/fat pointer operations and temporary
address lifetimes are implemented; dynamic slices/text and PA17 destruction
also remain. PA15–PA17 are still planned assignments.


## Executing checked thin references

LowIR construction now accepts sized reference values after the existing region,
storage and loan checks. Reference parameters/results use pointer values, proved
lifetime binders erase, and representation-preserving lifetime conversions reuse
the checked value. Aggregates may contain thin references. Unsized pointer
metadata and unsafe calls retain explicit boundaries.

Lowering collects address-observable places and their ancestors in a sparse set.
Borrowed scalar temporaries get one slot initialized at their defining evaluation;
object temporaries reuse their snapshot storage. Reads reload materialized
storage when a mutable reference can change it. Promoted roots use the persistent
data from the previous checkpoint while preserving source effects, overflow
checks and bounds checks. Borrowed zero-sized locals, temporaries and function
items receive nonempty aligned backing; borrowed zero-sized projections preserve
canonical parent offsets. Their value loads and native parameters remain erased.

Pattern selections retain compact ordinary offset recipes and share flat,
persistent dereference prefixes. Address calculation and pointer loads occur in
the actual use block, including guard retries. Dereferencing restores the
referent's canonical alignment independently of packed pointer-field storage.
No rendered syntax, reparsing or external frontend implements these operations.

Validation for this checkpoint:

- All 459 maintainer tests pass. Forty-one independent source vectors compile
  and execute with pinned rustc and CPPGM: reference returns, mutable field
  returns, reborrows, last uses, reference aggregates, promoted storage and
  effects, mutable temporaries, constant/static projections, packed references,
  zero-sized values, higher-ranked calls, borrowed constant patterns, conditional
  patterns, array rest captures and guard retries. Two cases independently
  require runtime panic for overflow and bounds checks, using each runtime's
  explicit termination contract. Repeated LowIR emission is byte-identical.
- Former maintainer capability rejections become positive checks only where
  thin-reference execution is implemented. An erased reference-containing array
  is accepted, while a zero-repeat initializer borrowing a local as `'static`
  still fails; pinned rustc independently confirms that negative case.
- Formatting, Clippy all/pedantic with warnings denied, the 287-file audit,
  all 25 harness tests and the backend smoke test pass. No lint allows were
  introduced. The release build succeeds.
- All 1,230 PA11–PA14 LowIR inputs preserve previously successful output bytes.
  Six former reference-boundary rejections now succeed; all six sources also
  compile and execute independently with pinned rustc and CPPGM. Some of these
  old boundary inputs have empty entry functions, so the dedicated native
  vectors above exercise the corresponding operations directly.
- The course report is 2,552/2,622 exact with 70 differences. Relative to the
  previous checkpoint, five already-differing reference boundary cases now
  accept, and PA12's nested-reference boundary is one additional difference.
  No fixture, manifest or generated oracle was changed or blessed. This remains
  an unreleased working branch with a failing course report.
- A larger independently executed program calls 4,096 separate wrappers that
  borrow two tuples, mutate their fields, return a mutable field reference and
  read the tuple after the reference's last use. Both native implementations
  compute the declared sum 40,960. Its LowIR has 4,101 defined functions,
  176,175 instructions and 28,679 slots.

Measurements retain 216 verified observations across existing ownership, guard
retry and packed-storage workloads plus the new reference-execution workload.
Existing workloads compare immutable baseline/current binaries with A/A
calibration and three ABBA blocks per scale; reference execution uses two copies
of the candidate because the baseline rejects it. Full and count outputs agree,
and packed-storage/reference counts grow linearly. No RustGM build or test
runs concurrently with timing.

At 64k ownership functions, baseline/current medians are 5,808.9/5,797.7 ms
and 949,572/949,932 KiB. At 16k guard functions they are 1,537.8/1,558.6 ms
and 296,796/296,516 KiB. The initial 4,096-object packed-storage comparison
measures 244.8/265.8 ms with equal 61,202 KiB median memory. Because that timing
difference warranted investigation, a focused follow-up retains another 42
observations with nine ABBA blocks on the identical frozen input and binaries.
Its medians are 294.2/291.0 ms, and the median paired candidate/baseline ratio
is 0.990. The initial slowdown is not reproduced; host timing varies enough
that no precise speed claim is supported. Both series remain in the evidence.

New reference-execution medians at 256/1,024/4,096 wrappers are
32.7/106.6/439.9 ms and 9,238/22,606/77,512 KiB. The largest frozen input is
also the independently executed scaling qualification above.
`verify_thinrefs_v1_perf.py` verifies the main series, source/binary/input hashes,
rendering/count equality and linear counts. The focused storage series retains
its own verified receipt under `thinrefs-v1-storage-followup/`.

The immutable compiler is `obj/pa15-work/thinrefs-candidate-v1`, SHA-256
`31aca89b4825c51e1010ebd55939f80e0ecb07e83fc3f9cc6ef1ddba32137b3d`.
Final source hashes, corpus comparisons, independent acceptance checks and
scaling qualification use `obj/pa15-work/thinrefs-v1-*`; the complete maintainer
and quality logs are `thinrefs-full-tests5.log` and `thinrefs-quality5.log`.

Static address cycles still need symbolic address evaluation distinct from
constant value-read cycles. Fat references, unsizing, dynamic slices/text and
PA17 destruction remain incomplete. PA15–PA17 are still planned assignments.


## Static address cycles and deferred location reads

Static allocation identity is now independent of initializer value evaluation.
Each static has one interned root location; fixed field/index projections retain
that root and their typed path while their contents may remain unread. Address
formation still checks array bounds and packed-field alignment. Actual reads
check Copy/access requirements before demanding the stored value, and cache
completed contents through the existing bounded query worklist. Automatic
dereferences suspend at the current path segment rather than rescanning prefixes.

A growing queue completes every static initializer reachable through recorded
addresses before the immutable lowerer runs. The existing data queue reserves
global identities before serializing pointers, including packed pointer fixups,
so self-cycles and mutual cycles need no backend changes. The
[Rust Reference's static-item contract](https://doc.rust-lang.org/reference/items/static-items.html#items.static.init)
permits static initializers to refer to other statics; the pinned local reference
remains the course source profile.

Constant patterns actually inspect referents. Their preparation uses a bounded,
memoized graph traversal, keeps repeated arrays compressed and diagnoses cyclic
structural expansion separately from initializer value cycles. Pure allocation
graphs need no additional traversal: they are already complete and acyclic.
Executable static pointer graphs are not rejected merely because a pattern could
not finitely expand them. The existing cyclic-pattern robustness test continues
to avoid the pinned rustc input that previously caused a compiler signal; it now
expects the specific finite-pattern diagnostic.

Validation for this checkpoint:

- All 462 maintainer tests pass. After the final pure-allocation fast path and
  test-helper cleanup, the affected location, static-address, reference-pattern,
  nominal/product-constant and native-reference suites pass again.
- The native reference suite has 49 independent source vectors. New cases cover
  self/mutual static links, cyclic arrays, packed cyclic pointers, a reference to
  a static's own field, constant reads through a cycle, static reference values
  and static-backed constant patterns. Each executes through pinned rustc and
  CPPGM; the original overflow/bounds panic controls remain intact.
- Six negative sources are independently rejected by pinned rustc and both
  emitted/count LowIR modes: direct and reference-mediated value cycles,
  a bad initializer reachable only through an address, an out-of-bounds static
  projection, a non-Copy referent read and an unaligned self-field reference.
  Candidate failures have empty stdout and diagnosed status one.
- A component test proves that repeated address queries share identity and leave
  a value-cyclic initializer unread, while backing completion correctly rejects
  that initializer. Another follows all 4,096 links of a static ring and checks
  every definition identity and the return to its original root without recursion.
- A larger source has 4,096 packed statics in one ring, plus 4,096 demanded
  functions reading through two links each. Both native implementations compute
  the independently declared sum. Its LowIR contains 4,096 globals, 4,100 defined
  functions and 81,936 instructions; 8,175 initializer items reflect merged zero
  runs for zero-valued tags.
- Formatting, Clippy all/pedantic with warnings denied, the 289-file audit,
  all 25 harness tests, backend smoke and release build pass. No lint allowances
  were introduced.
- All 1,230 PA11–PA14 LowIR results, including rejected cases' diagnostics,
  remain byte-identical to the preceding checkpoint. The course report remains
  2,552/2,622 exact with precisely the same 70 reviewed differences. No assignment
  fixture, manifest or oracle changed; this remains an unreleased working branch.

Measurements retain 270 verified raw observations: four existing workloads use
immutable baseline/current binaries, A/A calibration and paired ABBA blocks;
the new static-ring workload uses A/A scaling because the baseline rejects it.
Complete rendering and count outputs agree for every comparison. Constant
pattern counters and static-ring program counters grow linearly. Ring data-item
counts additionally follow the declared zero-tag periodicity exactly:
`2*N - ceil(N/251)`; accounting for that zero-run coalescing leaves a linear
count. Original raw outputs are retained without normalization.

At 64k ownership functions, baseline/current medians are 5,709.0/5,772.1 ms and
949,958/949,544 KiB. Constant-place patterns at 1k/4k/16k functions measure
57.6/192.4/836.2 ms before and 48.7/198.9/853.2 ms after; largest-scale memory
is 134,752/135,504 KiB. The largest place workload therefore has a small recorded
median time/memory cost, approximately 2%/0.6%. Constant reference chains at
depths 128/512/2,048 measure 4.5/11.0/36.0 ms before and 4.4/10.9/36.1 ms after.
Packed storage at 4,096 objects measures 305.0/301.0 ms and 61,156/61,218 KiB.
Timing ranges overlap, and the ownership series also records substantial host
noise at its middle scale; no precise speed improvement is claimed.

New static-ring medians at 256/1,024/4,096 nodes are 16.3/58.6/247.5 ms and
7,158/16,120/49,762 KiB. The largest frozen input is also independently executed
by both compilers. No RustGM build or test overlapped the measurements.
`verify_static_addresses_v1_perf.py` verifies all observations, immutable
source/binary/input hashes, output equality and the count relationships above.

The immutable compiler is `obj/pa15-work/static-addresses-candidate-v1`, SHA-256
`ccbdd66d52674aefd21d47f0a545bfe456065b2cd2e19189ad4e23c13c9cf715`.
Source hashes, course/corpus comparisons, ring qualification and measurement
receipts use `obj/pa15-work/static-addresses-v1-*`. Maintainer evidence is in
`static-address-full-tests2.log`, `static-address-final-tests3.log` and
`static-address-quality3.log` in the same directory.

Fat reference metadata, unsizing, dynamic slices/text and PA17 destruction remain
incomplete. PA15–PA17 handouts, required fixtures and oracles still need their
coherent release; the assignments remain planned.
