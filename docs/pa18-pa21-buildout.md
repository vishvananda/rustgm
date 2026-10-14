# PA18–PA21 implementation and validation

The active goal is to implement all four assignments. PA18–PA21 remain planned;
none has a grading manifest or a released reference yet. PA17 at `8e33000` is
the regression baseline. This document records the sequence and remaining work,
not a reduced completion contract.

## Shared representation

The existing parser retains generic parameters and path arguments. Extend the
resolved, typed representations; do not parse source again for each instance.
The pinned Reference in `doc/rust-reference/src/items/generics.md` owns parameter
ordering, const parameter types, scope and substitution rules. The existing
capture-safe lifetime substitution and region graph remain shared machinery.

Canonical nominal and function-item identities now use `GenericArgs`, preserving
argument kinds and declaration order. Lifetime-only lists keep their original
flat storage and numeric identities. Mixed lists and evaluated scalar constants
have separate compact arenas allocated on first use. Integer constants retain
their type and all target-width bits; bool and char validate their scalar domains.
Canonical argument equality never uses source spelling or rendered text.

Lifetime substitution and bound-use collection traverse type arguments with the
same binder depth as their owner. Shared type DAGs use the existing memoized,
bounded worklists. Nested callable binders remain capture-safe. Const arguments
distinguish evaluated scalar values from rigid const parameter terms.

Source type/const aliases now exercise joint substitution through the complete
frontend and native bridge. Generic functions and aggregates retain their
semantic boundaries. Layout, Copy/equality and destruction queries reject
mixed-argument nominal types until instantiated fields and obligations are
supplied. The alias checkpoint below records its exact scope and open work.

## PA18: generic templates and demanded instances

1. Retain type/const parameter identities by owning declaration and ordinal.
   Validate kinds, declaration order, defaults, arity and const parameter types.
   Resolve each syntax argument once, including inferred arguments and the
   type/const ambiguity of single-segment paths. Distinguish early lifetime
   arguments from late callable binders.
2. Elaborate type and const parameter terms and substitute them through types,
   signatures and aggregate field templates. Cache substitution in one argument
   environment. Keep lifetime shifting correct through nested callable binders.
   Substitute early lifetimes and type/const terms jointly when removing a
   template binder. Inserted type arguments belong to the destination context:
   shift them beneath nested callable binders, and never process their lifetimes
   again as if they belonged to the source template. Preserve parameters from
   other owners unless the immutable binding environment explicitly binds them.
   Generic array lengths must remain typed terms until known, then become the
   existing fixed array shape. Retain all types, including zero-sized fields.
3. Check generic bodies once against their parameter environment. Type-check
   unused bodies too; demanding only executable instances is not permission to
   accept an invalid unused generic body. Keep selected operations, calls,
   adjustments and obligations as typed facts for specialization.
4. Intern demanded function/constructor/destructor instances by declaration,
   complete type/const arguments and every semantic context affecting selection.
   Use an iterative demand queue with explicit states and budgets. Ordinary
   recursive calls can refer to a reserved instance; infinitely growing instance
   chains require a diagnosed limit. Do not confuse either with alias cycles.
5. Instantiate typed body facts and field/layout facts, then feed the existing
   ownership, borrow, destruction and LowIR pipeline. Repeated use of one instance
   must share work and emission. Keep deterministic first-demand ordering separate
   from hash lookup; retain earlier output when no generic work is demanded.
6. Publish the LowIR contract and fixtures together. Include type/const identity,
   inference/defaults, aliases, aggregates, recursive demand, function-item
   identity, array lengths, repeated instantiations, moves/borrows/destruction,
   negative unused bodies and invalid arguments. Move PA17's generic-destructor
   boundary to this owning stage when the actual implementation supports it.

PA18 implements parameterized code without requiring user-defined trait selection
prematurely. Its language-boundary tests must distinguish that PA19 responsibility
from required built-in bounds and existing destruction behavior.

## PA19: declarations, method selection and bounds

Retain trait/impl identities, generic parameter environments, associated method
signatures, receivers and conformance results. Ordinary inherent methods and
trait methods must share method selection; integrate the already supported
built-in Drop identity without identifying traits by spelling. Index candidates
by typed receiver/item facts rather than scanning every impl at each call.

Implement inherent and trait lookup, qualified calls, visibility, candidate
ordering, receiver autoderef/autoref and bounds. Record the chosen method,
substitution and every adjustment for body construction. Diagnose ambiguity
and invalid receivers before lowering. Check impl method conformance even when
unused. Publish `--emit-semantics` from these facts; the semantic view is an
inspection surface, not a substitute for executable validation.

Fixtures cover shadowing and hygiene, inherent/trait priority, qualified calls,
generic impls, by-value/shared/mutable receivers, adjustment ordering, ambiguous
methods, missing methods and bound failures. Move PA17's inherent-method boundary
to PA19 when supported. Run accepted programs through the existing native bridge.

## PA20: obligations, associated items and coherence

Introduce typed predicates, goals, parameter environments, associated item
projections and normalized results. Solver cache keys include environment,
binder universe and query mode. Cache failures only in their complete context;
do not reuse an answer learned under extra assumptions. Explicit states and
budgets must distinguish productive work, inductive cycles and exhausted work.

Implement associated types/constants, normalization, nested obligations and
impl selection. Check overlap and orphan rules independently of whether an impl
is demanded by an executable root. Keep coherence and ordinary proving modes
separate. The semantic view must expose the retained selection/normalization
facts that later phases actually consume.

Fixtures cover projections, where-clause assumptions, associated defaults and
overrides, invalid impl conformance, conflicting impls, orphan rules, recursive
goals, negative cache isolation and repeated normalized demands. Add native
controls proving that different selections produce the intended behavior.

## PA21: binders, variance, GATs, opaques and drop checking

Extend existing lifetime variance and directed outlives analysis to generic type
parameters, using a bounded fixed point for mutually recursive nominal types.
Higher-ranked obligations must introduce fresh universes and prevent placeholder
leakage; alpha-equivalent binders may share only after safe canonicalization.

Implement generic associated type families, their bounds and well-formedness at
each use. Retain opaque definition identity, hidden type and precise lifetime/type/
const capture rules, including edition-2024 capture. Generic destruction must
check coverage, field requirements, ownership and lifetime constraints for the
actual instantiation; it must not specialize a destructor illegally or erase a
parameter that affects layout or drop checking.

Fixtures cover covariant/contravariant/invariant positions, nested binders,
universe leakage, GAT families and bounds, opaque identity/captures, invalid hidden
types, generic Drop conformance and runtime cleanup order. Pair every positive
family with negative controls and cross-feature regressions.

## Release evidence and performance

For each assignment, declare acceptance and native outcomes before generating
oracles. Check language outcomes against pinned rustc and runtime outcomes through
the qualified CPPGM LowIR backend. Explain any course-profile differences in the
manifest. A text match alone is not independent language or runtime evidence.
Keep planned assignments failing until their contracts and complete suites ship.

Every compiler checkpoint runs all active earlier suites, Clippy all/pedantic
with warnings denied, the syntax-aware Rust file/architecture audit and harness
checks. Run fixture provenance checks after adding fixtures, and bridge checks
after changing the backend contract. New source tests live in their earliest
owning PA manifest when that contract is ready. Typed invariants can precede it.

Measure hot-path changes against immutable binaries and frozen inputs. Retain
A/A calibration, paired ABBA blocks, every observation and full/count output
equality. Do not run our builds/tests concurrently with timing. Generic workloads
must separately vary body size, instance count, repeated uses, structural sharing,
recursive demand and solver environment count. Work counters must demonstrate
sharing; wall time alone cannot establish that an algorithm is bounded.

Commit and push validated checkpoints on `work/pa18-pa21`, and check CI for each
pushed implementation. Activate each assignment only after its reference,
handout, independent qualification and regression evidence are complete.

## Canonical argument checkpoint validation

Eight new maintainer invariants cover argument kind/order/identity, constant
width and scalar validity, rejected IDs, both lifetime argument entrypoints,
nested binder capture, function-item argument binding depth, shared deep graphs,
traversal budgets and the uninstantiated-field proof gates. They pass in debug
and release. The 516 earlier frontend invariants and all 3,293 PA1–PA17 text
cases pass; eight explicit native maintainer tests remain excluded from the
ordinary suite. Existing course oracles are unchanged. Rustfmt and Clippy
all/pedantic pass without new lint exceptions. The file/architecture audit checks
352 files with zero findings; all 25 harness tests and assignment layout pass.

Performance uses the actual PA17 release CLI and the candidate CLI, immutable
binary copies, the same frozen thin-reference, wide-reference and destructor
inputs at 256/1024/4096 procedures, CPU 2, two warmups, four A/A observations and
three ABBA blocks per job. All 162 wall/RSS observations and full/count output
equality checks are retained under `obj/pa18-work/arguments-v1-perf-results.json`
and its referenced receipts. A second run retains 54 hardware-counter observations
with the same calibration and pairing at 4096 procedures. No local build/test
jobs ran during either measurement.

| 4096-procedure input | PA17 median ms | Candidate median ms | Retired instruction change | CPU cycle change |
| --- | ---: | ---: | ---: | ---: |
| Thin references | 458.706 | 475.640 | +0.073% | +2.15% |
| Wide references | 914.460 | 935.885 | +0.092% | +1.39% |
| Destruction | 2627.708 | 2752.228 | -0.252% | +0.69% |

The wall-time ranges overlap and calibration varies, but most paired blocks are
slower. This is a small measured cost, not evidence of a speedup or zero regression.
The hardware counters do not show material additional instruction work. Retain
these controls as source-level generics are integrated. The largest median RSS
changes are -18, -1110 and +316 KiB respectively. Generic DAG invariants separately
bound work at 4096 shared levels; they do not claim source-level generic execution.

Local evidence: `arguments-full-v1.log`, `arguments-focused-v2.log`,
`arguments-release-v1.log`, `arguments-v1-source-sha256.json`,
`arguments-v1-perf.log` and `arguments-v1-counters-results.json` in
`obj/pa18-work/`. The full run used the first six new invariants; the focused
debug/release runs include the two added argument-depth/budget controls. Together
they cover 524 distinct frontend tests. Generated logs and binaries are not
committed. No PA18–PA21 assignment is activated by this checkpoint.

## Rigid parameter representation

The next checkpoint adds immutable type/const parameter declarations keyed by
owner and argument slot. Repeating a declaration shares its identity; changing
its kind, const type or Sized requirement is rejected. These records use the
same lazily allocated generic storage as argument lists. Type parameters are
canonical type constructors and rigid inference atoms, distinct from inference
variables. An inference variable may resolve to a parameter, but two different
parameters cannot unify and a numeric literal cannot choose a parameter's type.

Const parameter terms retain their declared scalar type and cannot be read as
evaluated values. Type/const kind confusion and invalid scalar parameter types
are diagnosed. Canonical properties retain generic presence through every type
constructor, including zero-length arrays and function-item arguments. Lifetime
substitution preserves parameter identity and declared requirements.

The type-property cache stores six independent flags in one byte beside the
16-bit escaping-binder count. The record occupies four bytes on the pinned host,
compared with six at the canonical-argument checkpoint. Combining a child unions
its dependencies and intersects well-formedness; each constructor retains control
of sizedness. This avoids growing the property table for every ordinary type just
to add the generic-presence flag, without adding lint exemptions.

Closed-profile layout, ownership, equality, outlives and destruction queries
reject a parameter when its answer requires a generic environment. Universal
facts such as copying a shared reference and not dropping its referent remain
available. The outlives scan checks generic presence before its region-free fast
path, preventing an unresolved type parameter from becoming an empty proof.

This is still typed representation and inference groundwork. Declaration
elaboration, type/const substitution, generic array lengths, generic body proofs
and demanded source instances remain to be connected before PA18 can be released.

### Parameter checkpoint validation

Six additional invariants check immutable declaration ownership, rigid inference,
const terms versus evaluated values, generic presence through all constructors,
lifetime substitution and the closed-proof boundaries. The complete debug suite
passes 530 frontend tests, with eight explicit native tests ignored. All fourteen
generic argument/parameter invariants also pass in release. All 3,293 earlier
course cases remain byte-identical to their committed oracles. Rustfmt, Clippy,
the 355-file architecture audit and all 25 harness tests pass without exemptions.

The performance baseline is `6fa24df`'s actual CLI. The final candidate uses the
four-byte property record. Nine jobs use the same frozen inputs, sizes, CPU,
warmups, A/A calibration and ABBA pairing as above; all 162 observations and
full/count output equality checks are retained. No local builds/tests ran during
timing. At 4096 procedures:

| Input | Baseline median ms | Candidate median ms | Baseline RSS KiB | Candidate RSS KiB |
| --- | ---: | ---: | ---: | ---: |
| Thin references | 454.499 | 453.010 | 76296 | 77318 |
| Wide references | 862.694 | 868.509 | 123316 | 124156 |
| Destruction | 2598.148 | 2493.326 | 290394 | 290150 |

Ranges overlap and calibration shows host variation. These results do not
establish a general speedup, and the smaller property record does not imply
lower whole-process peak RSS in every workload. The earlier uncompressed
parameter candidate and its complete measurements remain retained separately;
they are not the final measured implementation.

Final local evidence in `obj/pa18-work/`: `parameters-full-v3.log`,
`parameters-release-v3.log`, `property-size-probe.log`,
`parameters-v2-source-sha256.json`, `parameters-v2-perf-results.json` and its
referenced raw receipts. The preceding candidate's observations are in
`parameters-v1-perf-results.json`. No generic source execution or completed PA18
grading contract is claimed by these typed representation tests.

## Joint substitution and source alias checkpoint

`Bound::joint` substitutes one owner's type/const parameters and early lifetimes
in one bounded traversal. Inserted type arguments belong to the destination
context and are shifted beneath nested callable binders. Replacements are
simultaneous: substituting `T -> U, U -> T` swaps the parameters; it does not
chase substitutions recursively. Cache keys include binder depth, and semantic
substitution caches include the declaration owner and full argument identity.
Six invariants cover capture, nested callable arguments, foreign owners, const
types, overflow, shared DAG work and repeat-query reuse.

An array length can now be a typed const parameter. Instantiation normalizes a
known length to the existing concrete array identity, preserving layout and
earlier LowIR behavior. Unresolved terms cannot acquire a fabricated storage
layout or destruction proof. Type-parameter use collection visits shared nodes
once across completed aliases, so erased type parameters are diagnosed without
repeated source scans.

The declaration worklist now elaborates alias type/const parameters, declaration
order, trailing defaults, dependent defaults, arity and argument kinds. It checks
unused declarations, alias cycles and unused type parameters. Source aliases
support scalar const parameters, standalone parameter forwarding, closed scalar
expressions supported by the existing evaluator, named constants, lifetime
arguments/elision and type arguments under callable binders. Alias expansion
retains Rust's deferred underlying well-formedness: an unused ill-formed alias
can be declared, while a function signature or body requiring it is rejected.
Alias bounds and generic body obligations remain separate responsibilities.

Name resolution prefers the type namespace for ambiguous bare arguments, falls
back to named constants when no type exists, and resolves explicit const blocks
as expressions. Four new PA6 fixtures own this resolver behavior. Outcomes were
declared before generating their references. The ambiguity fixture records that
PA6 resolves the type successfully while PA18 must reject its use as a const
argument. Existing PA6 text oracles are unchanged; its generated receipt now
records the current source hashes and the four new fixtures.

The maintainer suite independently qualifies 60 accepted/rejected alias programs
with the pinned rustc. Qualification retains default lints, including overflowing
literals; suppressing every lint previously hid that class of error. An additional
2048-link generic alias chain exercises the iterative dependency graph. Nine
programs pass the complete frontend and match rustc/native execution through
CPPGM, including different array layouts, defaults, function pointers, references,
custom destruction, zero-sized elements, nested callable reference binders and
a bounds panic.

Const argument checking reuses cleared dense body slots and an append-only
initializer index across the declaration phase. It keeps the existing scalar
typing and constant evaluator. Only touched nodes/bindings are cleared after a
successful query; failed checking discards its scratch. This avoids initializing
whole-file vectors separately for each small argument expression. The scratch
belongs to declaration checking and is released before later phases.

This is an incremental PA18 implementation, not a release. Remaining alias work
includes type/const inference holes, type parameter bounds, deferred evaluation
of unused const arguments/defaults, and broader shape-constant scheduling. In
particular, the current evaluator can reject an unused faulting const expression
that rustc never demands. Blocks with constant statements retain the existing
general-constant-evaluation boundary. Aggregate expressions used in shape
constants still need completed member facts when demanded in item signatures.
Const blocks use their resolved lexical module for privacy; the alias's defining
module cannot grant access to private fields or remove the caller's access.
Generic functions/aggregates, symbolic-body proofs, demanded instances and all
PA19–PA21 contracts remain required. No PA18 manifest is published prematurely.

### Alias performance measurements

The first alias prototype and the final scratch-reuse implementation are frozen
as separate binaries. Nine generic workloads vary type-only aliases, literal
const arguments and closed const expressions at 256/1024/4096 functions. Initial
A/A scaling retains 162 observations. The final comparison retains 324 more
observations: nine ordinary LowIR workloads against `61fa2af`, and the nine generic
workloads against the initial alias prototype. Every job has two warmups, four
A/A observations and three ABBA blocks on CPU 2. Full views and count outputs
match exactly within every comparison; all source/input/binary hashes are checked.
No builds or tests from this run overlap timing.

At 4096 functions, paired median wall times were:

| Workload | Baseline ms | Final ms | Baseline RSS KiB | Final RSS KiB |
| --- | ---: | ---: | ---: | ---: |
| Ordinary thin references | 486.160 | 477.482 | 77306 | 76208 |
| Ordinary wide references | 891.899 | 867.775 | 124080 | 124434 |
| Ordinary destruction | 2706.684 | 2684.530 | 290096 | 290616 |
| Type-only aliases | 59.744 | 56.697 | 17004 | 16984 |
| Literal const arguments | 290.904 | 77.343 | 19110 | 18392 |
| Closed const expressions | 344.031 | 96.585 | 21204 | 21228 |

The ordinary comparisons have overlapping ranges and show host variation; they
do not establish a general speedup. Both const workloads have separated paired
ranges after scratch reuse. Retained local evidence is
`obj/pa18-work/aliases-v1-perf-results.json`, `aliases-v3-perf-results.json`, their
referenced raw receipts, and `aliases-v3-source-sha256.json`. The intermediate
scratch-only candidate's complete `aliases-v2-*` measurements are also retained.

An additional 54-sample names comparison covers 4000/16000/64000 generated
functions against `61fa2af`, with full output equality. At 64000 functions the
paired medians are 905.773 ms baseline and 899.315 ms final; candidate peak RSS
is 140108 KiB. The evidence is `aliases-v3-names-perf.log` and
`obj/perf/1791956432623523018/samples.json`.

### Alias checkpoint validation

The final debug run passes 542 frontend tests; nine backend-dependent maintainer
tests are explicitly ignored by the default suite. All 3,297 active course cases
pass, including the four new PA6 cases; the previous 3,293 oracles are unchanged.
All 27 focused generic tests pass in release with native execution enabled.
Rustfmt, Clippy all/pedantic, the 365-file architecture audit, 25 harness tests,
assignment discovery and pinned upstream/reference verification pass. The full
independent rustc/layout checks and all nine explicit native maintainer tests
also passed before the privacy correction; final privacy controls are separately
qualified against rustc, and the final nine alias native programs pass in both
debug and release. The final compiler still matches every earlier text oracle.

Local final logs are `aliases-full-v4.log`, `aliases-release-v3.log`,
`aliases-native-final.log`, `aliases-audits-v7.log`,
`aliases-pa6-reference-v3.log` and the preceding broad independent qualification
`aliases-independent-v2.log`, all under `obj/pa18-work/`. The measured compiler
sources and generated PA6 receipt are checked against the final worktree before
publication. PA18–PA21 remain planned and the complete four-assignment goal
remains active.

### Symbolic function type checking and parameter environments

The declaration worklist now registers free-function type and const parameters
before elaborating their signatures. Their identities retain the owner and full
declaration slot, including gaps for lifetime declarations. Lifetime-only work
continues to assign signature slots on first use; registering a type parameter
does not change early/late lifetime classification. Function generic defaults
are rejected. Const parameters retain their declared scalar type in body
expressions and their symbolic identity in array lengths.

Body type checking visits every function once, including unused generic
functions. Rigid parameters support identity, tuple construction, branches,
local annotations, references, symbolic array types and function-pointer
arguments. They cannot unify with unrelated parameters or concrete numeric
types. Generic `main` entry points are rejected even when their signatures do
not mention their parameters. Destructor methods continue to reject type/const
parameters; allowing generic free functions must not relax the Drop contract.

`types::parameters::environment::Environment` freezes an explicit parameter
scope and its Copy/Clone assumptions. An unbounded parameter is present with no
abilities; a parameter from outside the scope is an error. Copy entails Clone,
while Clone alone does not permit copying. `Copies::in_environment` owns a
fixed environment and separate positive/negative caches. Scoped caches contain
only queried types, avoiding allocation and initialization proportional to the
whole crate for every small body. Closed queries retain their existing dense
cache and continue to reject unresolved parameters. Generic array-repeat
checking uses the body's scoped query; reference abilities remain independent
of referent abilities, and zero-length arrays still require Copy elements to
be Copy types.

This checkpoint supports **unbounded source type/const parameters**. Source
trait bounds, inferred/explicit generic function values, demanded instances,
and symbolic ownership/lifetime/destruction proofs remain unfinished. The
typed environment API has direct bounded-query tests, but this does not claim
that source `T: Copy` bounds are already elaborated. CFG construction explicitly
rejects generic function bodies until move/drop planning can preserve their
symbolic work. A successful type inspection is not an executable proof. No
assignment is activated and no earlier text oracle is regenerated here.

The maintainer tests independently qualify 21 accepted and 18 rejected source
templates with pinned rustc, check lifetime slot ordering and owner identity,
and exercise 512 independent generic bodies. Query tests cover unbounded,
Clone-only and Copy environments, foreign parameters, cached failures, closed
query isolation, reference/callable rules, zero-length arrays and a shared
4,096-level type graph. Existing Drop-header controls caught a missing method
restriction during development; that restriction is restored, with an added
const-parameter method control.

The final checkpoint passes 554 frontend tests (nine native tests ignored by
default), all 3,297 active course cases, 37 focused generic tests in release,
the full independent rustc checks and all nine native maintainer tests.
Rustfmt, Clippy all/pedantic, the 368-file audit, 25 harness tests, assignment
discovery and upstream/reference checksum verification pass. Final logs in
`obj/pa18-work/` are `function-environments-full-v2.log`,
`function-environments-release-v2.log` and
`function-environments-independent-v2.log`.

Performance uses frozen binaries and inputs, CPU 2, two warmups, four A/A
calibration observations and three paired ABBA blocks per input. Fifteen inputs
at 256/1,024/4,096 functions cover existing thin-reference, wide-reference and
destructor LowIR plus new generic identity and repeated-shared-reference type
checking. The 270 observations retain wall/CPU time, RSS, context switches and
full/count output equality; no own builds/tests overlapped measurement. Ordinary
programs compare against the preceding alias checkpoint. New generic programs
compare against the first implementation with dense per-body Copy caches.

At 4,096 functions, repeated-shared-reference checking improves from 213.94 to
94.89 ms (paired ranges 209.37–221.10 and 91.02–101.97 ms). Generic identity
checking is 76.18 versus 75.43 ms with overlapping ranges. Ordinary thin/wide
reference LowIR is 490.08/866.06 versus 489.42/849.34 ms, likewise with
overlapping ranges; this does not establish a general speedup. RSS remains
similar. Receipts and immutable input/binary/source hashes are recorded in
`functions-v2-perf-results.json`, `functions-v2-perf-jobs.json`,
`functions-v2-source-sha256.json` and the linked per-input `obj/perf/` records.
The production source hashes and release binary are rechecked before publication.

### Function-call type and const inference

Function value paths now consume arguments elaborated by the declaration
worklist. Omitted slots and `_` introduce body-local inference variables;
explicit arguments reuse typed annotations and scalar constant facts. Type
holes inside an argument remain in the same inference graph. Function signatures
are instantiated from their checked templates, using rigid parameter identities
rather than assuming compact argument positions equal declaration indices.
The existing argument traversal publishes a generic callable template at its
first function-value use and reuses it afterward. Unused generic declarations
still have their bodies checked; they do not pay for an unneeded callable
catalog entry. Existing early-lifetime template publication is preserved.
Early lifetime arguments precede the compact type/const arguments in function
item identity; late lifetimes belong only to the callable binder.

The inference context allocates generic argument and const storage on demand.
Const variables retain a fixed scalar type, use path-compressed union/find,
and have no numeric fallback. Array constraints can infer their lengths while
preserving a caller's rigid const parameter. A known length exposes the existing
fixed-array shape for indexing and unsizing. Mixed nominal/item argument lists
retain type arguments as graph edges, so occurs checks and final resolution
also visit parameters erased from a signature. All arguments contribute to
canonical function-item identity.

Region substitution over unresolved graphs preserves type and const variables
and respects nested callable binders. Instantiation receives arguments already
expressed in the destination body context; caller universals are normalized to
free region identities first. Freshening an inferred function item freshens each
argument position once and rebuilds its signature from the template. It cannot
freshen the signature independently and lose its connection to those arguments.
Distinct formal parameters remain distinct positions even if their written
types share one canonical identity. Const inference variables remain shared
across freshened uses of the same local item.

A non-binding graph comparison detects forced differences before joining
function items. Different instances can reify to one compatible function pointer,
including instances whose differing type/const arguments are absent from their
signatures. Unknown arguments remain unconstrained by this inspection; a
non-mismatch is not a proof of equality. Ordinary lifetime-only item joins retain
their existing path. Final argument checks enforce implicit Sized requirements,
including erased arguments. Type views render mixed argument kinds and scalar
bits deterministically; the old lifetime-only vocabulary remains unchanged.

Source controls cover argument/result inference, later calls through locals,
explicit and partial arguments, expected function pointers, independent uses,
recursive generic calls, const forwarding, early/late lifetimes, borrowed type
arguments, returned function items, array lengths, indexing, unsizing and erased
instance coercions. They independently qualify 30 accepted and 14 rejected
programs against pinned rustc. Typed tests additionally exercise occurs checks,
const type/value conflicts, unresolved consts, catalog owner/order validation,
unknown-preserving binder substitution, signature/argument coupling, and a
shared 4,096-level template graph. The CLI test checks mixed-argument output,
determinism and agreement with count mode.

This remains a **type-checking checkpoint**. Source trait bounds, generic nominal
fields, symbolic ownership/lifetime/destruction proofs, demanded instances and
executable generic lowering remain required. The CFG boundary still rejects
parameterized bodies, even when unused, until those proofs and transformations
exist. Const-expression evaluation retains the alias checkpoint's limitations;
symbolic array indexing still needs later body support. PA18–PA21 remain planned,
and no course oracle is regenerated for these maintainer-only controls.

The final validation passes 572 frontend tests (nine native tests explicitly
ignored by the default suite), all 3,297 active course cases, 57 focused release
invariants, the full independent rustc checks and all nine explicit native
maintainer tests. Rustfmt, Clippy all/pedantic, the 378-file architecture audit,
25 harness tests, assignment discovery and pinned upstream/reference verification
pass. Logs in `obj/pa18-work/` are `inference-full-v3.log`,
`inference-independent-v2.log` and `inference-release-v2.log`.

The first inference candidate retained 432 performance observations. A focused
60-observation repeat confirmed roughly 5–6% extra checking time for two large
unused-generic workloads. That exposed eager callable-catalog construction;
the final implementation defers the new entries until an actual function-value
use. The final comparison retains another 540 observations, including direct
prototype/final comparisons. All earlier observations remain available.

Final measurements use immutable binaries and inputs at 256/1,024/4,096
functions, CPU 2, two warmups, four A/A observations and three paired ABBA
blocks per input. Existing LowIR and generic-body programs compare with
`caa7cc4`. New generic-call workloads and the catalog-demand comparisons use
the first inference candidate. Every pair has matching full views and count
outputs. No own builds or tests overlap these measurements.

At 4,096 functions, the final comparisons with `caa7cc4` are:

| Workload | Baseline ms | Final ms | Baseline RSS KiB | Final RSS KiB |
| --- | ---: | ---: | ---: | ---: |
| Thin-reference LowIR | 500.42 | 505.06 | 76456 | 76838 |
| Wide-reference LowIR | 880.03 | 906.41 | 124596 | 124578 |
| Destruction LowIR | 2608.16 | 2695.12 | 290606 | 290968 |
| Generic identity bodies | 67.04 | 70.19 | 19846 | 19820 |
| Generic shared-reference bodies | 91.95 | 93.89 | 24028 | 24060 |

These wall-time ranges overlap and include small median increases; the shared
host varies substantially between blocks. They do not establish a general
speedup. Deferred catalog construction removes the unnecessary allocations:
unused-function RSS returns near the preceding checkpoint's level. The final
new type-call, const-call and borrowed-call workloads take 95.46, 101.05 and
227.77 ms respectively at 4,096 functions, with approximately linear scaling
across the measured sizes. Their prototype/final timing ranges also overlap.

Raw receipts, frozen hashes and all observations are linked from
`inference-v1-perf-results.json`, `inference-followup-v1-perf-results.json` and
`inference-v2-perf-results.json` in `obj/pa18-work/`. The final source manifest is
`inference-v2-source-sha256.json`; the release binary is
`inference-candidate-v2`. Source and binary hashes are checked again before
publication. These local measurements supplement the committed semantic tests;
they do not activate PA18 or replace its still-required execution contract.

### Built-in bounds and fixed body assumptions

The frontend now elaborates inline and where-clause `Copy`, `Clone`, `Sized`
and `?Sized` bounds for generic function checking. Trait identity comes from
resolved definitions. Small external `core`/`std` trait modules are loaded on
demand; forward aliases wake the existing import dependency worklist. Prelude,
external-crate, namespace and shadowing rules remain ordinary resolution rules.
PA6 owns 23 new name fixtures and generated text oracles. Their outcomes were
independently qualified before generation; no previous oracle changed. Expansion
and definition-identity tests additionally cover generated imports and shadows.

Each function owns an immutable parameter environment before body queries begin.
Copy entails Clone and Sized, and Clone entails Sized. Composite assumptions
remain attached to their complete typed subject: `(T,U): Copy` permits copying
that tuple, without deriving `T: Copy` or `U: Copy`. Likewise, assumed sizedness
cannot hide an ill-formed array element or non-final tuple element. Canonical
Store properties remain unchanged. Scoped shape and Copy queries use separate
caches, shared-graph worklists and bounded work; no query result moves between
functions with different assumptions. Extra assumption maps are allocated only
when predicates exist. Relaxed-bound membership uses an indexed parameter set.

Function-item uses retain typed obligations alongside type/const inference.
After inference, each substituted subject must satisfy its bound in the caller's
environment, even when the item is never called or the constrained argument is
absent from its signature. Closed false function bounds fail at declaration;
parameter-dependent impossible bounds are checked at use. Ordinary type-alias
bounds retain pinned rustc's unenforced behavior, including closed false bounds.
The source syntax restriction on relaxed bounds is deliberate: `T: ?Sized` is
allowed, but `(T): ?Sized` is rejected by pinned rustc. Built-in traits take no
generic arguments, and optional bounds other than Sized are rejected.

Lifetime uses in predicates participate in early/late classification. Body
queries retain assumptions in the declaration context and, when needed, their
capture-safe substitution into the caller's free universal aliases. Both forms
are frozen before constructing the caches. This permits forwarding an explicit
lifetime-dependent assumption without erasing regions or mutating cached proof
contexts. Bound references require named lifetimes except inside their own
nested callable binder.

There are 82 declared source controls (44 accepted, 38 rejected), independently
checked against pinned rustc, plus typed invariants for environment isolation,
foreign parameters, structural well-formedness, supertraits, exact composite
assumptions and a shared 4,096-level graph. Resolver tests verify module demand,
unique builtin identities, macros, aliases, namespace errors and prelude policy.
The source and typed controls pass in debug and release modes.

This is another **type-checking checkpoint**, not a complete bound solver or
released PA18. Explicit symbolic type-outlives predicates and their implied
proofs remain required. A bound subject whose well-formedness depends on those
proofs reports that boundary instead of silently accepting it. General matching
of region-related predicates, user traits, higher-ranked trait/where bounds,
generic nominal fields and symbolic ownership/destruction remain open. The
existing CFG boundary still rejects parameterized bodies. Demanded instances,
native generic lowering and the remaining PA19–PA21 contracts are still part of
the active goal; the assignments remain planned.

Validation passes 583 frontend tests, all 3,320 active course cases, 40 focused
release tests, the full independent rustc checks and nine explicit native
maintainer tests. The default frontend run reports those nine native tests as
ignored. Rustfmt, Clippy all/pedantic, the 388-file architecture audit, 25 harness
tests, manifest discovery and all pinned upstream/Reference checks pass. Evidence
is retained in `obj/pa18-work/bounds-{full,independent,release,harness}-v1.log`,
`bounds-audits-v3.log` and `bounds-pa6-{independent,bless}-v1.log`.

The frozen release binary `bounds-candidate-v1` has SHA-256
`32a520b422e5cd84d4e4dfdafefbc664e65a28aac31e139a693b8c7dbc1ac64e`.
Measurements retain 756 observations across 42 workloads at 256/1,024/4,096
functions or parameters. Each uses CPU 2, two warmups, four A/A calibration
observations and three paired ABBA blocks. Existing workloads compare against
`2a77baf` (`inference-candidate-v2`); new bounds workloads use identical candidate
binaries for calibration and scaling. Full rendered and count outputs agree for
every pair. No own builds or tests overlap the measurements; all frozen source,
input and binary hashes were rechecked afterward.

The existing workloads at 4,096 functions are:

| Workload | Previous ms | Bounds ms | Previous RSS KiB | Bounds RSS KiB |
| --- | ---: | ---: | ---: | ---: |
| Thin-reference LowIR | 453.35 | 441.12 | 76244 | 76908 |
| Wide-reference LowIR | 760.54 | 800.84 | 124988 | 124990 |
| Destruction LowIR | 2523.39 | 2553.69 | 291184 | 291280 |
| Generic identity bodies | 57.63 | 59.49 | 19814 | 19814 |
| Generic shared-reference bodies | 91.33 | 92.22 | 24002 | 24020 |
| Inferred type calls | 89.82 | 86.79 | 20534 | 20538 |
| Inferred const calls | 83.38 | 85.34 | 22616 | 22620 |
| Borrowed generic calls | 231.24 | 229.31 | 40914 | 40940 |

All reported A/B timing ranges overlap. Medians include both decreases and
increases (up to 5.3% for wide-reference LowIR), so this noisy shared-host run
establishes no general speedup. Peak memory stays close to the preceding
checkpoint. New workloads exercise Copy bodies/calls, tuple and const-array
assumptions, forward trait-module aliases and one wide relaxed parameter list.
At 4,096, their candidate B medians are respectively 106.95, 85.45, 153.87,
176.39, 174.05 and 31.24 ms. The wide-bound measurements show no quadratic
increase over the measured sizes. Every raw observation, including calibration,
is retained through `bounds-v1-perf-results.json` and `bounds-v1-perf-jobs.json`;
`bounds-v1-source-sha256.json` records the source freeze.

### Symbolic type-outlives proofs

Inline and where-clause type-outlives predicates now retain their typed subject
and lifetime. Lifetime classification includes the bound's right-hand side;
function-item uses substitute both the subject and that lifetime before retaining
the obligation. Inference completes before the existing region proof discharges
it. Alias bounds keep the pinned language's unenforced behavior.

The shared outlives query returns separate lifetime and rigid type-parameter
components. Parameter identities are registered in each function's existing
directed region graph. Signature well-formedness and explicit predicates supply
assumptions; local annotations, where-clause subject well-formedness and call-site
obligations supply requirements. Thus `&'a T` in a signature implies `T: 'a`,
but mentioning `&'a T: Copy` in a where-clause does not. Another function's
parameter or a previously failed query cannot acquire the current assumptions.
Const parameter terms are not type-parameter lifetime components.

Explicit type bounds traverse every referent and structural component, including
zero-length array elements. For example, `&'a T: 'a` establishes the component
bound needed to make its own subject well-formed. Minimal reference
well-formedness components and runtime live-value components retain their
different rules. Nested callable binders export only their free lifetimes.
An inferred `fn(&'a u8)` argument can be widened contravariantly to satisfy a
`'static` type bound; explicitly fixing that same type argument cannot. Source
controls independently qualify both outcomes.

Traversal remains iterative, bounded and cached by typed identity and component
mode. Parameter vectors are retained only for queries that find parameters;
membership scratch is reused. Leading lifetime argument substitution uses direct
slice indexing through the argument iterator instead of rescanning mixed lists.
Closed-type and runtime live-region consumers still reject symbolic values,
preventing them from silently discarding a pending generic proof.

Five typed invariants cover component distinctions, nested binders, graph scope
and direction, negative-result isolation, diagnostics and shared 10,000-level
types. There are 67 declared source controls (40 accepted, 27 rejected), checked
against pinned rustc and the complete typed contract-proof path. The preceding
82 built-in-bound controls now run that proof as well. A separate rustc-accepted
higher-ranked where-predicate control retains the explicit PA21 boundary.

PA15 gains 11 lifetime-only type-bound fixtures; PA16 gains three unchanged-source
native programs covering shared, mutable and static references. Outcomes were
declared and independently checked before generating their sidecars. No previous
oracle bytes changed. These owning stages now contain 202 and 296 cases,
respectively, with 116 executable PA16 comparisons.

This checkpoint proves symbolic lifetime contracts through the typed maintainer
interface. The CLI region view still requires a CFG and rejects type/const
generic bodies. Symbolic ownership, loans and destruction, instantiated nominal
fields, demanded instances and native generic lowering remain required for PA18.
General region-related built-in predicate matching and the PA19–PA21 contracts
remain open. All four assignments remain planned under the original active goal.

Validation passes 591 frontend tests, all 3,334 active course cases, 46 focused
release tests, the full independent rustc checks, nine explicit native maintainer
tests and all 116 PA16 native comparisons. Rustfmt, Clippy all/pedantic, the
390-file architecture audit, 25 harness tests, assignment discovery and pinned
upstream/Reference verification pass. Logs are retained in `obj/pa18-work/` as
`outlives-{full,independent,release,harness}-v1.log`, with independent fixture
qualification and reference-generation logs for PA15 and PA16.

Performance retains 1,134 observations across 63 jobs at 256/1,024/4,096
functions or parameters. The 42 existing jobs compare the preceding `e8dddc2`
binary with the frozen candidate; the 21 new jobs use identical candidate
binaries for calibration and scaling. Each job uses CPU 2, two warmups, four A/A
observations and three ABBA blocks. Full and count outputs agree for every pair,
and source, binary and input hashes remain unchanged. No own builds or tests
overlap the measurements.

At 4,096 functions, the existing workloads are:

| Workload | Previous ms | Candidate ms | Previous RSS KiB | Candidate RSS KiB |
| --- | ---: | ---: | ---: | ---: |
| Thin-reference LowIR | 491.85 | 493.41 | 76908 | 77358 |
| Wide-reference LowIR | 924.96 | 934.38 | 125026 | 125256 |
| Destruction LowIR | 2727.37 | 2776.90 | 291296 | 291428 |
| Generic identity bodies | 73.47 | 75.31 | 19792 | 19854 |
| Generic shared-reference bodies | 109.08 | 111.77 | 24012 | 24050 |
| Inferred type calls | 102.63 | 103.74 | 20520 | 20524 |
| Inferred const calls | 116.40 | 118.42 | 22610 | 22668 |
| Borrowed generic calls | 456.53 | 454.11 | 40948 | 40980 |

All A/B ranges for the 14 existing workload families at 4,096 overlap. Median
changes range from -1.3% to +2.5%; this shared-host run establishes no general
speedup. Memory remains close to the preceding checkpoint.

The new generic workloads use a frozen maintainer driver that invokes source
analysis, typed contract collection and the completed region proof, then emits
that proof's full or count view. Its dependency versions/checksums match the
frontend lockfile. This avoids measuring only elaboration while leaving the
new proof pending. At 4,096, implied-reference, static-parameter, forwarding,
tuple-bound and reference-bound proofs take 113.65, 97.85, 179.30, 187.78 and
198.67 ms respectively. One 4,096-parameter bound list takes 22.61 ms. The
lifetime-only CLI region workload takes 166.59 ms. Scaling stays approximately
linear over the measured sizes; the wide list shows no quadratic increase.

The frozen CLI `outlives-candidate-v1` has SHA-256
`27d3a1a76f759f9cede7802a286e954a9e008a67f642e11d114bc2288e9c670e`;
`outlives-proof-candidate-v1` has SHA-256
`266a6c4a3d7e056f533c9c242c7453dc78a25dd0f32d44c36026562820f23067`.
The source freeze includes the driver. Raw observations, receipt paths and job
definitions are retained in `obj/pa18-work/`: `outlives-v1-perf-results.json`,
`outlives-v1-perf-jobs.json`, `outlives-v1-source-sha256.json` and
`outlives-perf-v1.log`.

### Symbolic destruction requirements

The shared destruction query now accepts an immutable parameter environment.
An unconstrained type parameter may need cleanup; a Copy assumption proves that
its value needs none. Clone, Sized and outlives bounds do not supply that proof.
A composite Copy assumption applies to the whole subject without deriving the
same property for its elements. References, raw pointers and function values
do not own their referenced or signature types.

Fixed zero-length arrays still need no element cleanup. An unknown const length
preserves the element's possible requirement without enumerating elements or
guessing a length. A billion-element fixed array takes the same structural work
as a one-element array. Closed queries retain their explicit generic boundary,
and mixed nominal field queries remain pending instantiated-field support.

Scoped caches are sparse and own their fixed assumption context. Ordinary closed
queries retain dense storage. Successful and failed results cannot leak between
different environments; failed dependencies do not poison independent types.
Five maintainer tests cover these distinctions, source environments, closed-query
boundaries, a shared 20,000-level graph and repeated cached demands. Fourteen
source controls and fourteen concrete `needs_drop` witnesses are independently
qualified with pinned rustc. A concrete witness shows that possible cleanup can
occur; it does not claim every substitution needs destruction.

This is the query needed to preserve generic owned temporaries when constructing
template CFGs. It does not yet enable those CFGs, select generic glue or authorize
execution. The existing constructor and executable boundaries remain in place;
symbolic move/loan checks and demanded body specialization are still required.

Validation passes 596 frontend tests, all 3,334 active course cases, 22 focused
release tests, the independent rustc suites, nine explicit native maintainer
tests and all 132 PA17 native comparisons. Rustfmt, Clippy all/pedantic, the
391-file architecture audit and 25 harness tests pass. No course fixture or
oracle changes. Evidence is retained in `obj/pa18-work/` as
`symbolic-drops-{full,release,focused}-v1.log` and `symbolic-drops-audits-v2.log`.

Measurements retain 486 observations across 27 jobs, using CPU 2, immutable
binaries/inputs, two warmups, four A/A observations and three ABBA blocks. The
nine existing LowIR jobs compare with `58223f2`; 18 scoped-query jobs use the
same candidate on both sides for calibration and scaling. All pairs have equal
full/count outputs. All frozen hashes were rechecked, and no own builds or tests
overlapped timing. The 18 generated generic inputs also passed rustc and query
preflight checks before measurement.

| 4,096-function workload | Previous ms | Candidate ms | Previous RSS KiB | Candidate RSS KiB |
| --- | ---: | ---: | ---: | ---: |
| Thin-reference LowIR | 482.79 | 480.22 | 77566 | 77344 |
| Wide-reference LowIR | 889.83 | 847.99 | 125068 | 125128 |
| Destruction LowIR | 2352.32 | 2289.48 | 291376 | 291320 |

All three timing ranges overlap. The lower medians do not establish a speedup;
memory remains close to the preceding checkpoint. The frozen maintainer driver
measures source analysis and scoped destruction queries separately from native
lowering. At 4,096 functions, unknown, Copy, Clone, const-array and composite-bound
workloads take 57.08, 98.60, 70.44, 102.24 and 134.79 ms respectively. One wide
4,096-parameter tuple takes 15.97 ms. Growth over 256/1,024/4,096 is approximately
linear; per-function caches do not allocate the whole crate's type-store size.

The frozen CLI `symbolic-drops-candidate-v1` has SHA-256
`09c4dc47d5deb8d02e48c65650b340ca596bdee4befb2b88942b4e96b0daf3ee`;
`symbolic-drops-driver-candidate-v1` has SHA-256
`1ef0c972ab109c8f611314d2b6bba958b5753d58f614d973c21f5c9715dcd4fe`.
The driver's dependency versions/checksums match the frontend lockfile. Its
source is included in `symbolic-drops-v1-source-sha256.json`; measurements and
receipt paths are retained in `symbolic-drops-v1-perf-results.json`,
`symbolic-drops-v1-perf-jobs.json` and `symbolic-drops-perf-v1.log` under
`obj/pa18-work/`.

### Template CFGs and scoped move checking

Generic function templates now use the ordinary function CFG builder and its
scope scratch. A distinct `body::templates::Templates` result retains each body,
its fixed environment and possible destruction requirements. It is not a concrete
`Program` or an executable proof. Construction includes unused generic functions
and preserves owned temporary creation, block forwarding, return transfers and
discarded call results even when no custom destructor exists in the crate.

The ordinary move/initialization engine checks these templates under scoped Copy
assumptions. It diagnoses repeated moves, uninitialized reads, immutable writes,
invalid loop reuse and non-Copy field moves out of objects with custom Drop.
Composite Copy assumptions apply only to their whole subject. The borrow module
consumes typed types, definitions, derive selections and a destructor-identity
lookup; it acquires no dependency on semantic analysis.

Body environments retain the existing substitution of lifetime-dependent
assumptions into free caller universals. Only changed environments get a sparse
override; other bodies share their declaration environment. Scoped destruction
queries share one immutable destructor registry instead of rebuilding that set
for every function. Their caches remain independent.

Const-parameter repeat lengths now use the same resolved-parameter reader as
generic arguments and array annotations. `[x; N]` and `[x; {N}]` retain an
`ArrayTerm` through inference, repeat validation and CFG construction. An unknown
length requires Copy for an ordinary repeated operand. A selected constant path
keeps independent constant instantiation semantics; the zero-length ordinary
operand still retains its own possible cleanup. Wrong const types, local shadows
and unsupported generic arithmetic remain diagnosed rather than inventing a
concrete length.

Five maintainer tests include 36 declared move/repeat controls (19 accepted,
17 rejected), independently checked with rustc, plus scoped-environment identity
and owned-temporary invariants. Three additional rustc-rejected mutable-reference
programs deliberately pass the move-only proof: their automatically inserted
reborrows still need loan checking. The tests assert the concrete CFG gate stays
closed, so a template move proof cannot authorize those programs' execution.

This checkpoint supplies symbolic CFGs, pending cleanup and move facts through
typed maintainer APIs. The CLI still requires the complete generic pipeline.
Symbolic loan/liveness and destruction proof, const-length projection handling,
instantiated nominal fields, demanded body specialization and native generic
lowering remain required for PA18. PA19–PA21 remain part of the active goal.

Validation passes 601 frontend tests, all 3,334 active course cases, 47 focused
release tests, all independent rustc checks, nine explicit native maintainer
tests and 132 PA17 native comparisons. Final repeat controls also pass in debug
and release after qualification. Rustfmt, Clippy all/pedantic, the 394-file
architecture audit and 25 harness tests pass. No existing course fixture or
oracle changed. Logs in `obj/pa18-work/` include `template-cfg-full-v1.log`,
`template-cfg-release-v2.log`, `template-cfg-final-report-v1.log`,
`template-cfg-focused-v7.log`, `template-cfg-release-final-controls-v1.log` and
`template-cfg-audits-final-v1.log`.

Performance retains 486 observations across 27 jobs at 256/1,024/4,096 functions.
The nine existing LowIR jobs compare with `4ca8e09`; 18 new jobs measure source
analysis, actual template CFG construction and the scoped move proof with
identical candidate binaries. Every pair has matching full/count output. Each
job uses CPU 2, two warmups, four A/A observations and three ABBA blocks. Source,
binary and input hashes were rechecked, and no own builds/tests overlapped timing.
All 18 new inputs passed independent rustc and template-proof preflight checks.

| 4,096-function workload | Previous ms | Candidate ms | Previous RSS KiB | Candidate RSS KiB |
| --- | ---: | ---: | ---: | ---: |
| Thin-reference LowIR | 465.08 | 464.30 | 77326 | 77364 |
| Wide-reference LowIR | 844.35 | 836.71 | 125052 | 125070 |
| Destruction LowIR | 2637.34 | 2704.49 | 291528 | 291552 |

Ranges overlap for these comparisons. The destruction median is 2.5% higher;
this is not a claim of zero cost. Memory remains close to the preceding checkpoint.
New move, Copy, repeat, discarded-call, loop-restoration and shared-registry
workloads take 79.24, 127.29, 179.93, 116.56, 183.29 and 149.75 ms respectively
at 4,096 functions. The registry workload also declares 4,096 custom destructors.
Growth over the measured sizes shows no quadratic per-function registry rebuild.
The identical repeat-workload binaries differ by about 20% in paired medians,
underscoring shared-host noise; these measurements establish no general speedup.

The frozen CLI `template-cfg-candidate-v1` has SHA-256
`dfa10b03448060ee7c7292aeb4413d49a289d74592acffca4998c7cfd09b5a48`;
`template-cfg-driver-candidate-v1` has SHA-256
`4116230b1e0236827d08d3f7ee83261ba8d1297acd759658bd1f95a42a5f622c`.
The driver uses the frontend's locked dependency versions/checksums. Its source
is included in `template-cfg-v1-source-sha256.json`; job definitions, observations
and raw receipt paths are retained in `template-cfg-v1-perf-jobs.json`,
`template-cfg-v1-perf-results.json` and `template-cfg-perf-v1.log` under
`obj/pa18-work/`.

### Template promotion and lifetime provenance

Generic templates now share the ordinary demanded promotion engine and its
constant evaluator. Each body supplies its scoped possible-destruction facts,
so an unknown owned type cannot accidentally be promoted simply because the
crate declares no custom destructor. Completed facts are reused; bodies without
borrow demand do not allocate a constant evaluator.

The template region entrypoint maps actual CFG bodies to unique lifetime
contracts and invokes the existing directed prover. It proves only the supplied
templates, leaving concrete functions, initializers and constructors to their
own pipeline. Missing or duplicate mappings fail. The concrete program entrypoint
continues to require CFG coverage for every contract. Neither result proves
runtime loans or destructor-induced liveness.

Const-parameter array relations now compare the retained length identities and
relate element lifetimes with the usual variance. They do not guess lengths or
equate distinct const parameters. Shared shortening is valid; mutable referents
still enforce invariance.

The source controls include local and temporary escapes, forwarded references,
shared/mutable reborrows, type-outlives bounds and unused generic bodies. Empty
generic arrays can promote without knowing their element layout, including
mutable and nested shared borrows. An ordinary zero-repeat operand still retains
its possible cleanup even when the resulting empty array promotes. Custom Drop
values and runtime parameter values remain ineligible for permanent storage.

Unknown-length repeat promotion has an explicit deferred-constant boundary.
The tests independently distinguish rustc's acceptance of `&[1u8; N]` from its
rejection of a runtime parameter repeat; this checkpoint does not claim either
template has completed promotion. Three further rustc-rejected loan controls
intentionally pass provenance alone. All remain behind the concrete generic-body
gate, pending symbolic liveness, ordered abstract cleanup and loan checking.

Six new maintainer tests include 25 declared source controls (14 accepted,
11 rejected), independently qualified with pinned rustc, plus mapping, promotion
cache and phase-boundary invariants. Validation passes 607 frontend tests, all
3,334 active course cases, 26 focused release tests, all independent rustc checks,
nine explicit native maintainer tests and 132 PA17 executable comparisons.
Rustfmt, Clippy all/pedantic, the 395-file audit and 25 harness tests pass. No
course fixture or text oracle changed. Logs are
`obj/pa18-work/template-provenance-full-v1.log`,
`template-provenance-release-v1.log` and
`template-provenance-focused-v2.log` in the same directory.

Performance retains 468 observations: 24 initial jobs at 256/1,024/4,096
functions, followed by two targeted repeats of noisy closed 4,096-function jobs.
The nine initial closed jobs compare with `10d6f04`; the 15 new jobs measure
semantic analysis, template CFG construction, scoped moves, promotion and actual
provenance proofs using identical candidate binaries. All 15 new inputs passed
independent rustc and proof-driver preflight checks. Each job uses CPU 2, two
warmups, four A/A observations and three ABBA blocks. Every comparison has equal
full/count output. No own builds or tests overlapped measurements; all frozen
source, binary and input hashes were rechecked. Both timing runs are retained.

| Closed 4,096-function workload | Previous ms | Candidate ms | Previous RSS KiB | Candidate RSS KiB |
| --- | ---: | ---: | ---: | ---: |
| Thin references | 467.02 | 470.11 | 77336 | 77556 |
| Wide references, initial | 1363.92 | 1452.39 | 125006 | 125226 |
| Wide references, repeat | 918.65 | 931.20 | 125382 | 125188 |
| Destruction, initial | 2773.47 | 2976.97 | 291492 | 291430 |
| Destruction, repeat | 2699.17 | 2701.24 | 291490 | 291520 |

The initial wide-reference A/A calibration ranged from 923 to 1,663 ms, and
both binaries varied substantially during the initial destruction comparison.
The targeted repeats put wide references 1.4% higher and destruction 0.1% higher,
with overlapping sample ranges; this is not a claim of zero cost or a speedup.
The new identity, reborrow, zero-array promotion, nested promotion and
const-array shortening workloads take 126.82, 217.61, 177.35, 176.07 and 219.80 ms
respectively at 4,096 functions. Measurements across the three sizes retain
approximately linear growth, without a per-function whole-crate query rebuild.

The frozen CLI `template-provenance-candidate-v1` has SHA-256
`c1abd651476b082cca7c5250abe6ff194bc2e486b43c8f7e0233a7d1873cc03b`;
`template-provenance-driver-candidate-v1` has SHA-256
`bc1e0e091b619e7fc3ce6922c43bf33b96d05254b01849cb8750bd65c4704334`.
The driver's dependency versions/checksums match the frontend lockfile. Its
source is included in `template-provenance-v1-source-sha256.json`; the job,
result and raw receipt indexes are `template-provenance-v1-perf-jobs.json`,
`template-provenance-v1-perf-results.json` and their `perf-recheck` counterparts
under `obj/pa18-work/`.

### Symbolic live components and template loans without cleanup

Live-value queries now expose named regions and opaque type-parameter components
separately. The closed `LiveRegions::get` API still rejects symbolic types;
`symbolic` shares the same bounded structural cache while retaining both kinds.
Zero-length arrays have no runtime components, const-parameter arrays retain
possible element components, and nested callable binders export only free named
lifetimes. Repeated shared shapes are visited once per query and cached.

Backward liveness retains parameter use seeds alongside named lifetime seeds.
Loan inference resolves those parameter identities through the actual body
contract and gives rigid parameter universals the body-wide extent of caller
inputs. Missing parameter identities fail. A bound such as `T: 'a` remains
one-way; making `T` live cannot make every shorter reference loan live. Directed
bound, last-use and replacement controls exercise that distinction.

`borrow::loans::templates::check` combines the selected provenance, scoped move
facts, actual storage and ordinary loan dataflow for templates without possible
cleanup. It checks complete unique body coverage and destruction-fact dimensions.
Unknown owned values, destructible temporaries and potential replacement cleanup
retain an explicit abstract-cleanup boundary, even when no custom Drop is
actually declared. A projection through `&mut T` alone does not destroy `T`.
No fake cleanup operation, concrete specialization or executable token is created.

The new source controls reject overlapping mutable reborrows, reading through a
live mutable borrow, and writes through a live shared borrow. They allow reuse
after last use, mutually exclusive branches, repeated loop reborrows, scoped
Copy operations and independently instantiated generic calls. The three mutable
loan controls that intentionally pass the separate move/provenance APIs now fail
this combined template loan check. Empty shared/mutable array promotions also
pass storage and loan checking.

Abstract destructor liveness and ordered cleanup still need integration, as do
symbolic repeat promotion, generic nominal fields and demanded concrete instances.
PA18–PA21 remain planned and the generic executable gate remains closed.

Six maintainer tests include 28 declared source controls (19 accepted, nine
rejected), three explicit pending-cleanup controls, malformed-identity/coverage
checks and a cached 20,000-level shared type graph. Pinned rustc independently
confirms every declared language outcome. Validation passes 613 frontend tests,
all 3,334 active course cases, 32 focused release tests, all independent rustc
checks, nine native maintainer tests and 132 PA17 executable comparisons. The
final loan-conflict assertions also pass in debug and release. Rustfmt, Clippy
all/pedantic, the 397-file audit and 25 harness tests pass. No course fixture or
oracle changed. Logs under `obj/pa18-work/` include `template-loans-full-v1.log`,
`template-loans-release-v1.log`, `template-loans-focused-final-v1.log`,
`template-loans-release-final-v1.log` and `template-loans-audits-final-v1.log`.

Performance retains 432 observations across 24 jobs at 256/1,024/4,096 units.
Nine closed jobs compare with `e186dd0`; 15 new jobs use identical candidate
binaries and include the actual template loan proof after semantic analysis,
CFG construction, scoped moves, promotion and provenance. Each job uses CPU 2,
two warmups, four A/A observations and three ABBA blocks. All full/count output
matches. All new inputs passed pinned rustc and proof-driver preflight checks.
No own builds/tests overlapped measurements, and source, binary and input hashes
were verified before and after timing.

| Closed 4,096-function workload | Previous ms | Candidate ms | Previous RSS KiB | Candidate RSS KiB |
| --- | ---: | ---: | ---: | ---: |
| Thin references | 454.67 | 447.00 | 76998 | 77272 |
| Wide references | 843.80 | 839.27 | 124968 | 125260 |
| Destruction | 2379.96 | 2381.75 | 291480 | 291508 |

The sample ranges overlap; these measurements establish no general speedup or
zero-cost claim. New reborrow, loop last-use, Copy replacement, generic call and
const-repeat workloads take 135.00, 480.99, 233.30, 428.59 and 275.36 ms respectively
at 4,096 units. The call workload contains two functions per unit, or 8,192
generic functions at that size. Growth is approximately linear across the three
sizes; loop and call timings retain shared-host variability.

The frozen CLI `template-loans-candidate-v1` has SHA-256
`f73094aa6cb1e73b46aa0c51d8a39e56ce711f611dd20e0a0907088f377a52ce`;
`template-loans-driver-candidate-v1` has SHA-256
`48be83ede2755b2e4931715ef758525743acc3247ef62eb2450e98a44a4771a7`.
The driver uses the frontend's locked dependency versions/checksums. Its source
is included in `template-loans-v1-source-sha256.json`; job definitions, results
and raw receipt paths are retained in `template-loans-v1-perf-jobs.json`,
`template-loans-v1-perf-results.json` and `template-loans-perf-v1.log` under
`obj/pa18-work/`.

## PA18 abstract cleanup and destructor-aware template loans

Generic bodies now have a combined initialization, cleanup and loan verifier in
`borrow::loans::templates::verify`. It checks every selected body under its fixed
parameter environment after promotion and provenance. It validates unique total
contract coverage, repeats the ordinary scoped ownership proof with retained
may-initialization, enforces custom-destructor field-move rules, selects abstract
cleanup and runs the existing ordered liveness/loan analysis. The older `check`
entry point still explicitly requires bodies without possible cleanup.

Concrete and abstract plans share an event schedule and the initialization
transfer recorder. The concrete plan keeps its native recipes and flags; the
abstract plan contains only possible accesses, original-type lifetime needs and
ordered events. It cannot construct an executable destruction proof. Partial
moves use the existing sparse field/array paths. They can remove every possible
runtime destructor access while preserving the original aggregate's drop-check
obligation. Whole moves and storage ends remove that obligation at the existing
ordered transfer points. Nominal field coordinates come from retained member
identities through a typed callback; borrow analysis has no semantic-phase
lookup dependency.

Drop-region queries now have fixed environment scopes and preserve opaque owned
parameters separately from named lifetimes. References do not own their
referents, zero arrays have no element cleanup, unknown lengths preserve possible
element cleanup and exact Copy assumptions suppress only their own subject.
Coerced/aggregate operand liveness also preserves both kinds of component.
Canonical component lists keep ordinary lifetime-only identities compact and
allocate parameter-list storage only when demanded. Iterative caches, work and
storage budgets remain explicit.

Consumption distinguishes symbolic repeat modes without guessing their lengths:
a constant repeat instantiates elements; an ordinary unknown-length repeat must
have no destructible prototype because typing requires Copy. A fixed zero-length
ordinary repeat still disposes of its evaluated operand immediately.

Six added maintainer tests cover 44 independently qualified programs (26 accepted,
18 rejected), malformed contract coverage, assumption isolation, cached failures,
non-owning references, zero/unknown/huge arrays and a shared 20,000-level type graph.
The source controls include replacement, branches, loop cleanup, lifetime
substitution, wildcard temporaries, whole versus partial moves, references that
outlive a moved sibling field, and sparse moves from billion-element arrays.
Loan-negative controls check an actual live-loan conflict, separately from the
non-Copy repeat and forbidden custom-destructor field move.

This completes the abstract cleanup integration for the currently supported
symbolic body profile. Generic nominal field instantiation, deferred const
promotion/projections and demanded concrete specialization remain required.
PA18–PA21 remain planned; the generic executable gate remains closed.

Validation passes 619 frontend tests, all 3,334 active course cases, 46 focused
release tests, every independent rustc check, nine native maintainer tests and
132 PA17 executable comparisons. The final source-control assertions also pass
in debug/release. Rustfmt, Clippy all/pedantic, the 402-file audit and 25 harness
tests pass. No course fixture or oracle changed. The initial full run stopped at
the audit after frontend/course success because tree-sitter misparsed a test's
`&raw` identifier; renaming it to `collected` fixed the audit, and the remaining
validation completed. Retained logs include `template-cleanup-full-v1.log`,
`template-cleanup-validation-v2.log`, `template-cleanup-release-v1.log`,
`template-cleanup-focused-final-v2.log`, `template-cleanup-release-final-v2.log`
and `template-cleanup-audits-final-v1.log` under `obj/pa18-work/`.

Performance retains 378 observations across 21 jobs at 256/1,024/4,096 generic
functions or closed comparison units. Each job uses CPU 2, two warmups, four A/A
observations and three ABBA blocks. Nine closed jobs compare against `291d72f`;
12 new jobs use identical candidate binaries and exercise the full abstract
cleanup verifier after typing, body construction, promotion and provenance.
All new inputs pass pinned rustc and driver preflight checks. Full/count output
matches for every job. No own builds/tests overlap timing; source, binary and
input hashes are checked before and after measurement.

| Closed 4,096-unit workload | Previous ms | Candidate ms | Previous RSS KiB | Candidate RSS KiB |
| --- | ---: | ---: | ---: | ---: |
| Thin references | 496.56 | 497.98 | 77164 | 77882 |
| Wide references | 894.92 | 919.39 | 125034 | 125024 |
| Destruction | 2621.55 | 2589.34 | 291530 | 291494 |

These ranges overlap; they establish no general speedup or zero-cost claim.
The new workload medians grow roughly with function count, with noticeable
shared-host variability retained in the middle partial-reference run and the
largest replacement run.

| New template proof | 256 ms | 1,024 ms | 4,096 ms | 4,096 RSS KiB |
| --- | ---: | ---: | ---: | ---: |
| Owned replacement | 35.43 | 131.97 | 535.55 | 53988 |
| Reference after partial nominal move | 71.51 | 399.68 | 1104.70 | 83226 |
| Constant Drop repeat | 17.81 | 57.53 | 227.88 | 38070 |
| Sparse billion-element array moves | 26.34 | 91.89 | 370.74 | 38156 |

The frozen CLI `template-cleanup-candidate-v1` has SHA-256
`7200f31f9c140c4e182cbfee5de4ecb6ab89e1c3aba06facb222b6f4a4d56b86`;
`template-cleanup-driver-candidate-v1` has SHA-256
`8f8843ffcdab3143a0c911caa2d2e94555b4531e68c6291b590f6350c87eb611`.
The driver uses the frontend's locked dependency versions/checksums. Its source
and manifests are included in the 282-file `template-cleanup-v1-source-sha256.json`.
Job definitions, results, preflight checks and raw receipt paths are retained in
`template-cleanup-v1-perf-jobs.json`, `template-cleanup-v1-perf-results.json`,
`template-cleanup-v1-perf-preflight.json` and `template-cleanup-perf-v1.log`.

## Generic nominal field queries

Nominal field instantiation now has a shared typed boundary. `Definitions`
retains a lazy cache keyed by the full canonical nominal type, including type,
const and lifetime arguments. A single capture-safe joint substitution serves
all fields of one instance. The borrowed shape combines substituted field lists
with the declaration's variant order, representation and final discriminants.
It does not clone syntax or duplicate representation facts.

Preparation follows owned fields and type arguments with a bounded worklist;
equality preparation additionally follows reference referents. Pointer targets
and function signatures remain outside these preparation graphs. Consequently,
`Node<T>` containing a pointer to `Node<(T,)>` does not cause infinite expansion.
Repeated roots reuse completed work. Failures cannot publish unfinished
children as prepared, and preparation itself does not prove sizedness, finite
layout, declaration validity or bounds.

Layouts, Copy/Clone, equality, destruction requirements, drop-lifetime queries
and abstract cleanup now consume these field instances. Derived abilities also
constrain type arguments behind pointers. Packed Clone strengthens those bounds
to Copy. Packed equality receives the ordinary Copy query through
`Equality::demand_with_copy`; the declaration and constant-pattern callers
supply the same immutable selections. The simpler equality API reports the
missing Copy proof explicitly for a packed generic type. Rustc's
[PartialEq derive implementation](https://doc.rust-lang.org/stable/nightly-rustc/src/rustc_builtin_macros/deriving/cmp/partial_eq.rs.html)
records the extra packed Copy requirement; pinned 1.98 controls qualify the
actual behavior used here.

Custom Drop retains every argument's full type-outlives components, including
Copy parameters and nested reference lifetimes. Ordinary owned destruction
instead follows actual fields and skips zero-length arrays. Lifetime-only
nominal queries retain their shared template path. Concrete cleanup still rejects
mixed nominal arguments before looking up its lifetime-only cache key: an item
identity cannot select a type/const destructor instance.

Thirteen maintainer tests exercise different layouts, const/lifetime cache keys,
invalid const kinds, final discriminants, nested instances, expanding pointer
recursion, a shared 10,000-level graph, failed preparation, by-value cycles,
conditional derives, exact Copy assumptions, custom Drop components and the
concrete cleanup boundary. Eighteen source controls declare eleven acceptances
and seven rejections before independent rustc checks. The first packed Clone
hypothesis was wrong: Clone alone does not suffice for the argument of a packed
pointer wrapper. Its failed receipt is retained; packed and ordinary controls
now distinguish the correct conditions.

This integrates nominal field queries, not nominal source elaboration or
monomorphization. Header/default/bound scheduling, constructor inference,
generic variance and field-implied outlives requirements, conditional derive
validation, and generic Drop body/instance selection remain required. Deferred
const-length promotion and concrete body specialization remain required too.
PA18–PA21 remain planned and the generic executable gate stays closed. No course
fixture, manifest or oracle changed.

Final local validation passes 632 frontend tests, all 3,334 active course cases,
22 focused release tests, nine native maintainer tests, rustfmt, Clippy
all/pedantic, the 405-file audit and 25 harness tests. Authoritative logs are
`nominal-instances-full-v2.log`, `nominal-instances-validation-v2.log`,
`nominal-instances-release-v2.log` and `nominal-instances-native-v2.log` under
`obj/pa18-work/`. The first full run's formatting check encountered an incomplete
additional test edit; the completed test and full final run pass. No production
source changed after the final frozen builds.

Performance retains 306 observations across 17 final jobs, plus 162 observations
from nine targeted paired rechecks. Each job uses CPU 2, two warmups, four A/A
observations and three ABBA blocks. Eight jobs compare the published parent
`2747ef8` on closed reference/destruction workloads and abstract cleanup. Nine
new jobs measure field preparation plus layout, Copy, equality and drop queries
at 1,024/4,096/16,384 instances. They use identical candidate binaries because
the parent lacks this field-query boundary. The scratch driver's external
locked dependencies match the frontend's. All nine typed-query preflights pass;
full/count output matches in every measurement. Sources, binaries and inputs
remain frozen and hashes verify before/after. No own builds or tests overlap
timing.

| Existing 4,096-unit workload | Parent ms | Candidate ms | Candidate RSS KiB |
| --- | ---: | ---: | ---: |
| thinrefs | 498.89 | 506.66 | 77338 |
| fatrefs | 934.24 | 931.60 | 125254 |
| native-destructors | 2766.77 | 2770.32 | 291500 |
| partial-reference | 1097.35 | 1085.16 | 83432 |

These overlapping ranges establish no general speedup or zero-cost claim.
During the final new-workload run, timings increased partway through the scale
ladder. The recheck compares the pre-packed-equality binary with the final
binary on identical inputs. Both return to lower, overlapping ranges; this
supports machine variability rather than a regression caused by that change,
without identifying its physical cause. All slower observations are retained.

| New 16,384-instance query (paired recheck) | Earlier candidate ms | Final ms | Final RSS KiB |
| --- | ---: | ---: | ---: |
| deep | 85.43 | 85.12 | 17000 |
| mixed | 141.60 | 142.06 | 25436 |
| pointer | 131.88 | 131.24 | 24628 |

Preparation work is exactly 12/16/24 steps per instance in the deep shared,
mixed const/lifetime and expanding-pointer cases respectively. Array lengths
and repeated queries do not expand that work. The original measurements before
packed equality integration remain available as v1 evidence, superseded by v2
for the final compiler source.

The final frozen CLI SHA-256 is
`8add4e0a0a8f05f5f8be1d5860b427b98d7b994702c5e88adbc695b3622da534`;
the typed-query driver is
`d5811dd2d890a63246e227104bb41e9926eb5782901fa706eed011a55c5aad65`;
the rebuilt template-cleanup driver is
`bb38ec33ac9ca7c15b5577ead0557f2c896c4a30f771d51b25e0d49fbf1863d5`.
The 287-file source manifest, job definitions, preflight, results and raw receipt
paths are `nominal-instances-v2-*` under `obj/pa18-work/`, with the timing logs
`nominal-instances-perf-v2.log` and `nominal-instances-perf-recheck-v2.log`.

### Nominal type-parameter variance and directed region relations

The variance table now uses every nominal declaration slot: lifetime, type and
const parameters retain their declaration order and rigid owner/index identities.
Const slots are invariant even when unused. Type/lifetime occurrences are
inferred from completed field templates; bounds do not invent occurrences.
Source declarations obtain the schema from their canonical nominal identity.
The existing unused-lifetime diagnostic and course output remain unchanged.

Nested nominal arguments require products of unknown variances. A shared
expression graph interns those products and propagates changes through reverse
dependencies over the four-element lattice. It simplifies literal constants,
never a variable whose present value happens to be bivariant. Recursive-only
parameters remain bivariant; a direct occurrence can propagate through mutual
recursion, and opposed recursive uses reach invariance. The traversal visits
finite declaration templates, without instantiating or expanding fields.
Function binder depth keeps locally quantified lifetimes separate from captured
nominal lifetimes. All registration, traversal, graph and propagation work is
bounded, including wide shared inputs and const-only argument comparisons.

Directed region relations now traverse type arguments with their inferred
variance as well as lifetime arguments. Const arguments require canonical
identity equality. Existing inference can retain distinct actual/expected
nominal type arguments while solving their structural shape; a new integration
test passes those completed types to the region proof and checks both covariant
acceptance and invariant rejection. No equality of storage lifetimes is silently
introduced by structural inference.

Eleven maintainer tests cover nested products under different declaration orders,
mutual/expanding/recursive-only templates, mixed and interleaved parameter slots,
higher-ranked binders, exact const compatibility, inference-to-region handoff,
a 10,000-level shared product graph, budget exhaustion and malformed identities.
Thirteen independently declared source controls are checked with pinned rustc:
eight acceptances and five rejections. Their expected outcomes were recorded
before this implementation in `obj/pa18-work/nominal-variance-next-v1/declared.json`.
The pinned Reference's `subtyping.variance.builtin-types` and
`subtyping.variance.user-composite-types` rules are the specification baseline.

This completes the nominal variance query and its region consumer. Generic
nominal source declarations remain gated on header/default readiness, scoped
field well-formedness and implied bounds, declaration storage-cycle checks,
conditional derives and constructor/pattern inference. Generic Drop selection,
deferred const promotion and concrete body specialization also remain required.
PA18–PA21 retain their full scope and planned status. Course fixtures, manifests
and text oracles are unchanged.

Final local validation passes 643 frontend tests, all 3,334 PA1–PA17 cases,
28 focused release tests and nine explicit native maintainer tests. Rustfmt,
Clippy all/pedantic, the 407-file audit (zero findings) and 25 harness tests pass.
The final logs are `nominal-variance-full-v3.log`,
`nominal-variance-validation-final.log`, `nominal-variance-release-v1.log` and
`nominal-variance-native-v1.log` under `obj/pa18-work/`. Earlier logs remain as
history; the final full run follows the const-argument budget correction.

Performance retains 270 observations across 15 jobs on CPU 2. Each job has two
warmups, four A/A observations and three ABBA blocks. Six jobs compare the
published parent `0246a20` on identical closed CLI inputs at 1,024/4,096 units.
Nine new typed-query jobs use two identical candidate binaries at
1,024/4,096/16,384 units because the parent cannot infer type-parameter variance.
They include construction of the typed input, variance inference and positive
and negative directed region checks. All full/count outputs agree; all 284
source hashes, frozen binary hashes and input hashes verify before and after.
No own builds or tests overlap timing. No sample is discarded.

| Existing 4,096-unit workload | Parent ms | Candidate ms | Candidate RSS KiB |
| --- | ---: | ---: | ---: |
| thinrefs | 480.82 | 480.07 | 77354 |
| native-destructors | 2730.08 | 2767.43 | 291492 |
| lifetime-variance | 206.29 | 209.27 | 52254 |

The latter two median increases are about 1.4%, with overlapping observed
ranges; these measurements do not establish a speedup or zero overhead.

| New 16,384-unit query | Candidate ms | Candidate RSS KiB | Variance work |
| --- | ---: | ---: | --- |
| deep shared products | 30.88 | 12760 | `14n + 14` |
| mutual recursive declarations | 83.60 | 14866 | `15n + 3` |
| mixed lifetime/type/const slots | 101.39 | 18950 | `18n` |

The work formulas hold at all three scales. No query instantiates aggregate
fields. The frozen CLI SHA-256 is
`331ff7f0e726a102375ee728a7985d051119d3b4d417a6124e0d0b5ecf2324f4`;
the typed driver is
`3dbc595311fb15d5cc3ad41ea2697b1fac4f9d4f54e0e4d1ae8b82d6ee0e4f0b`.
Receipts, source hashes, preflights, job definitions and raw observation paths
are `nominal-variance-v2-*`; the timing log is `nominal-variance-perf-v2.log`.

### Alias-body and generic-default readiness

Generic parameter identities now precede default evaluation. Defaults have
separate unseen/active/complete query states keyed by their parameter binding;
alias expansion retains its independent cycle states. A written argument never
demands an unrelated default. An omitted argument schedules only the required
template through the ordinary bounded declaration worklist. Every unused default
is still checked before declaration facts are published. Direct self/forward
parameter references remain rejected by name resolution.

This accepts finite self and mutual applications such as `type A<T=A<u8>>=T`
and `type A<T=A<u8>,U=T>=(T,U)`. In the latter, the inner application supplies
`T=u8`; the later default template `U=T` therefore yields `u8`, without demanding
T's default again. Actual omitted-default cycles report `recursive generic
default`; recursive alias bodies retain their existing diagnostic. Alias bound
subjects are elaborated after their bodies become available, permitting finite
self/mutual subjects without treating alias bounds as enforced use-site traits.

Default substitution now uses one append-only environment per use. Existing
arguments never change and replacements remain simultaneous, so successful
substitution and binder-shift caches remain valid when later slots are appended.
The completed argument list is interned once; its environment is retained for
ordinary expanded-body substitution. The prior implementation copied and
interned every prefix and retained a separate environment for each omitted
argument, requiring quadratic storage on wide parameter lists.

Seven maintainer readiness tests cover 18 accepted and 14 rejected source
controls, independently qualified with pinned rustc; exact canonical argument
results; separate cycle diagnostics; a 2,048-link dependency chain; and 2,048
dependent parameters in one declaration. Two further substitution tests check
capture-safe prefix extension, error recovery, simultaneous replacements, const
kind validation and linear work across 10,000 slots. Five new programs exercise
self-dependent type/const/lifetime defaults, mutual defaults and owned Drop in
the existing native-alias suite. They also pass the ordinary LowIR generation
check; native execution is qualified separately.

This fixes shared alias/default readiness needed by PA18; it does not enable
generic nominal declarations or complete PA18–PA21. Nominal fields, implied
bounds, storage cycles, derives, constructors and generic Drop selection still
need integration, followed by concrete specialization and the remaining trait
assignments. A separate independently qualified next-stage probe also exposes
an existing field-readiness gap: nominal construction in a const default or a
signature's const argument can reach `missing struct member facts`. Tuple and
array constants succeed. Resolving that gap needs demanded nominal field facts,
not treating all defaults as completed or silently accepting a missing proof.
The five probes are retained under `obj/pa18-work/default-fields-next-v1`.
No course manifest, fixture or oracle changes in this checkpoint.

Final validation passes 652 frontend tests, all 3,334 active course cases,
28 focused release tests, nine native maintainer tests, rustfmt, Clippy
all/pedantic, the 409-file audit (zero findings) and 25 harness tests. The native
alias suite now has 14 programs: 13 clean executions and one declared panic.
After the full run, the new owned-Drop program was strengthened to increment an
observable borrowed counter; both its LowIR and native suite were rerun and pass.
Compiler source remained unchanged. Logs are `generic-defaults-full-v1.log`,
`generic-defaults-release-v1.log`, `generic-defaults-native-v1.log`,
`generic-defaults-native-counter-v1.log` and
`generic-defaults-validation-final.log` under `obj/pa18-work/`.

Performance retains 180 observations across ten jobs on CPU 2, each with two
warmups, four A/A observations and three paired ABBA blocks. Seven jobs compare
parent `4ac7274` with the final candidate on identical existing inputs. Three
new self/default-dependent workloads use identical candidate copies because the
parent rejects their source. Every full/count output matches, and all 282
source hashes, binary hashes and input hashes verify before and after timing.
No own builds or tests overlap timing; all observations, including slow samples
on both sides of the thin-reference comparisons, remain in the receipts.

| Existing workload | Units | Parent ms | Candidate ms | Parent RSS KiB | Candidate RSS KiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| wide dependent defaults | 512 | 15.77 | 9.34 | 7156 | 5390 |
| wide dependent defaults | 1024 | 38.54 | 14.09 | 14886 | 5744 |
| wide dependent defaults | 2048 | 117.19 | 23.90 | 41658 | 7142 |
| forward default chain | 4096 | 83.31 | 78.26 | 20502 | 17680 |
| thinrefs | 4096 | 500.82 | 499.43 | 77362 | 77358 |

The wide-list reduction matches the removal of cumulative prefix copying.
The ordinary thin-reference ranges overlap, with large slow observations on
both binaries; this is not evidence of a general frontend speedup. New finite
self-default workloads measure 35.79/144.25/653.04 ms and
12068/36012/129986 KiB at 1,024/4,096/16,384 declarations. The typed substitution
test independently bounds 10,000 dependent slots and final tuple expansion to
fewer than 50,000 charged steps.

The frozen CLI SHA-256 is
`93046f9a9de7579cdbca9d4d7221b2fd51b28d0d2176b1ea6785c3542dc80603`.
Source hashes, preflights, jobs, validation and raw observation paths are the
`generic-defaults-v1-*` receipts; the timing log is
`generic-defaults-perf-v1.log`.
