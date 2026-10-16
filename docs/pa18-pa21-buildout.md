# PA18–PA21 implementation and validation

The active goal is to implement all four assignments. PA18 now has its public
CLI, handout and grading manifest; its release evidence is recorded in
[pa18-validation.md](pa18-validation.md). PA19 is also implemented-draft, with
[pa19-validation.md](pa19-validation.md) recording its course release. PA20 and
PA21 remain planned and required. PA17 at `8e33000` is
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

Commit and push validated checkpoints directly to `main`, and check CI for each
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


### Declaration-time enum values and constant destruction

Const arguments now use an explicit pending discriminant registry. The prior
local-annotation path treated an empty query specification as proof that enum
values were final, reading the field table's provisional zero. It consequently
accepted a zero-length array where an enum tag required length eight. Pending
and finalized values are now distinct. The registry retains the variant index
and completed values across argument queries, including long predecessor chains;
final declaration checks still visit unused values, duplicates and overflows.

Constant eligibility now checks the owned temporary bases of projections,
updates and ordinary zero-length repeats. This includes unexecuted branches and
the pinned stable compiler's conservative rejection of partially moved Drop
temporaries. It preserves transferred results, immutable static reads, named
constant repeat operands and references with static lifetime extension. The
existing lexical-scope analyzer moved from `body::scopes` to `sema::scopes` and
remains reexported by `body`; constant checking and body construction share it.
Scope analysis is demanded only when a temporary may need destruction. Place
roots and destruction requirements are cached; extending the destructor registry
invalidates cached absence of destruction. Enums implementing Drop are also
excluded from integer casts in both constant and ordinary expressions.

Reference-bearing anonymous constants retain a typed body when inference emits
lifetime obligations. These bodies enter ordinary region, loan and destruction
proof and never become native functions. Pure scalar arguments need no additional
body. Static lifetime extension now supplies permanent provenance independently
of promotion, allowing a borrowed Drop value in a constant without permitting an
ordinary function to return a reference to such a temporary. No obligation is
discarded or assigned a fabricated declaration identity.

The maintainer tests include 20 independent enum acceptance/rejection controls,
a 2,048-variant/2,048-use chain, 31 independently qualified destruction controls,
anonymous-body ownership invariants, and an independent negative control for
ordinary temporary provenance. Four new alias execution programs cover enum
arguments, signed tags, extended constant references and a returned Drop constant
whose destructor must panic. The native-alias suite has 18 programs: 16 clean
executions and two declared panics. Generated LowIR and native execution are
validated separately. Eight regressions belong to PA17 and extend its course
manifest to 207 cases (135 accepted, 72 rejected; 133 clean executions and two
panics). Their expected outcomes were independently checked before generating
text oracles. Earlier fixture outputs remain unchanged.

This completes the two correctness repairs found while investigating nominal
readiness; it does not complete PA18–PA21. General nominal field demand, mixed
parameter headers, declaration bounds and storage validity, constructor/pattern
inference, generic destructor selection, concrete specialization and the full
PA19–PA21 responsibilities remain. Nominal construction in an early default or
signature argument still needs the forthcoming field-readiness query.

Validation passes 662 frontend tests (nine native tests explicitly ignored in
that default run), all 3,342 active course cases, 33 focused release tests, all
nine explicit native maintainer tests and PA17's 135 executable comparisons.
Rustfmt and Clippy all/pedantic pass; the 413-file architecture audit has zero
findings; the 25 harness tests and upstream provenance checks pass. Logs are
`constant-readiness-full-v2.log`, `constant-readiness-release-v2.log`,
`constant-readiness-native-v2.log`, `constant-readiness-validation-v3.log`,
`constant-readiness-course-v1.log` and `constant-readiness-pa17-native-v1.log`
under `obj/pa18-work/`. The initial full run exposed the missing anonymous
constant proof owner; the final run includes the fix and native qualification.

Performance retains 216 observations in twelve jobs on CPU 2: two warmups, four
A/A observations and three ABBA blocks per job. Nine jobs compare frozen parent
`66113e4` with the candidate; three newly accepted enum-argument workloads use
identical candidate copies. Full/count outputs match for every pair. All 284
source hashes and frozen binary/input hashes verify before and after timing.
No local builds or tests overlap timing.

| Existing workload | Units | Parent ms | Candidate ms | Parent RSS KiB | Candidate RSS KiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| implicit-enums | 4096 | 24.22 | 24.85 | 7152 | 7166 |
| explicit-enums | 4096 | 17.75 | 18.78 | 7824 | 8610 |
| drop-scalar-arguments | 4096 | 66.48 | 66.33 | 13454 | 13830 |
| drop-static-arguments | 4096 | 65.70 | 67.58 | 14204 | 14220 |
| thinrefs | 4096 | 516.43 | 516.80 | 77364 | 77374 |

The explicit-enum inventory has a measured cost of roughly 1 ms and 0.8 MiB at
4,096 variants from retaining pending value/index state. Scalar const arguments
and the lowering control have overlapping timing ranges. The newly accepted
1,024/4,096/16,384 variant-and-use workloads have candidate medians near
26/97/700 ms and peak RSS near 8/19/65 MiB. The largest workload varies from
398 to 712 ms between observations of identical binaries; retain that variation
and do not infer an exact whole-frontend scaling law. The source/query structure
shares completed discriminants; timing alone is not a query-work proof.

The frozen candidate SHA-256 is
`cfa73fcf504d508125037d031b6c89895a6345b5b14a63c30f0423d754c06c4d`.
Source hashes, preflight comparisons, job definitions and raw sample receipt paths
are recorded in `constant-readiness-v1-*`; the measurement log is
`constant-readiness-perf-v1.log`.

### Constant layout prerequisites

Packed constant borrows now demand complete enum discriminants before constructing
their referent layouts. The previous path could cache a four-byte `repr(C)` tag
from provisional zeros, then accept a borrow that actually required eight-byte
alignment. The same stale layout could truncate a later persistent enum value.
Signed tags, forward named constants, nested products and zero-length arrays all
retain their genuine dependencies.

Layout begin/finish work shares the existing iterative constant stack. Completed
layouts provide the cache; active layout queries detect storage/value cycles.
Each completed discriminant is published to the aggregate facts immediately,
while the final pass still checks every unused value, duplicate and overflow.
Pointers stop the by-value walk. Field traversal consumes full-TypeId aggregate
shapes, preserving the existing instantiated-field boundary. No recursive query
calls, global layout resets or repeated whole-enum inventory scans are introduced.

Constant preparation consumes the packed-borrow obligations already retained by
body typing, including unexecuted branches. An append-only source index makes
those obligations available without rewalking places or evaluating a dead index
or arithmetic expression. Prerequisites run before value continuations. Pinned
rustc rejects self/cross enum layout cycles even when the enum has an explicit
integer representation; those cycles remain errors here too.

Five new maintainer tests cover 28 independently qualified outcomes, cached tag
width/discriminant invariants and 1,024 repeated borrows with fewer than 64 ABI
layout visits. Three additional alias execution programs cover large C tags,
zero-array alignment and a dead division whose value must remain unevaluated.
PA16 owns eight new regressions: five required rejections and three executable
programs. Their outcomes were declared and independently checked before reference
generation; all 296 existing PA16 oracle outputs remain unchanged. The new native
oracle for a large C enum retains an eight-byte readonly value and eight-byte
loads, including the high discriminant bits.

This is another prerequisite repair for PA18 field/header readiness. PA18–PA21
remain planned. The full nominal generics, constructor/pattern inference, scoped
bounds, specialization, trait and advanced relationship responsibilities above
are unchanged. Pending Drop-owner availability during impl-header constants is
still a required part of the forthcoming nominal query design.

Validation passes 667 frontend tests (nine explicit native tests ignored in that
default run), all 3,350 active PA1–PA17 cases, 24 focused release tests, all nine
explicit native maintainer tests and 119 PA16 executable comparisons. The alias
native suite now has 21 programs: 19 clean executions and two declared panics.
All 304 PA16 outcomes independently match pinned rustc. Rustfmt/Clippy all/pedantic,
the 415-file audit with zero findings, all 25 harness tests and upstream provenance
pass. Logs are `alignment-full-v1.log`, `alignment-release-v1.log`,
`alignment-native-v1.log` and `alignment-pa16-independent-v1.log`. The first build
caught a borrow-checker error, fixed before tests; an early harness run detected the
intentionally pending reference receipt, which was generated before the final run.

Eleven frozen performance jobs retain 198 observations on CPU 2: two warmups, four
A/A observations and three ABBA blocks per job. All full/count outputs match; all
285 source hashes and immutable binaries/inputs verify. No local build or test
jobs overlap timing. Eight jobs compare parent `2ebe00e` with the candidate;
three corrected enum-borrow workloads compare identical candidate copies.

| Workload | Units | Parent ms | Candidate ms | Parent RSS KiB | Candidate RSS KiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| implicit-enums | 4096 | 14.71 | 14.98 | 7170 | 7156 |
| explicit-enums | 4096 | 17.97 | 17.54 | 9224 | 8564 |
| drop-scalar-arguments | 4096 | 63.79 | 63.44 | 13520 | 14186 |
| drop-static-arguments | 4096 | 64.32 | 64.36 | 14154 | 13460 |
| thinrefs lowering | 4096 | 493.39 | 493.41 | 77694 | 77400 |
| packed-scalar arguments | 1024 | 32.70 | 33.26 | 8614 | 8562 |
| packed-scalar arguments | 4096 | 119.10 | 121.86 | 20858 | 20530 |
| packed-scalar arguments | 16384 | 499.96 | 500.86 | 69150 | 67792 |

The packed-borrow workload includes the new obligation lookup/check, with a roughly
2.8 ms median increase at 4,096 uses and overlapping paired ranges. Its 16,384-use
ranges also overlap. The corrected wide-enum workloads scale from about 37 to 146
to 608 ms at 1,024/4,096/16,384 variants and borrows, with roughly 10/25/87 MiB RSS.
These are observations, not an exact scaling law or a general speedup claim.

The candidate SHA-256 is
`37806b2556f36ae968a59b700c977fa97ef73e331263ac97e6c357582b465dec`.
`alignment-v1-*` records source hashes, job definitions, preflight comparisons,
validation and raw receipt paths; `alignment-perf-v1.log` retains the measurements.
The parent miscompilation is retained as `alignment-large-parent.lowir`: it emits
four zero bytes and a u32 load for the large enum. The new PA16 oracle emits the
correct eight-byte value and i64 load.

### Nominal fields and destructor owners before early constants

Nominal identities and field availability now have separate declaration states.
A field demand schedules annotations on the existing bounded worklist, then
publishes complete field groups, constructor mappings and pending discriminants.
Ordinary type annotations still need only the nominal identity, so pointer
recursion remains legal. An actual field/default dependency cycle diagnoses.
The final pass completes unused declarations and retains all existing variance,
derive, destructor, signature and layout checks. Representation and derive
selections are registered before early consumers can cache their absence.

Constant dependency discovery follows resolved constructors, aliases, named
constants and immutable statics, and walks owned field shapes once per TypeId.
It follows explicit enum discriminant dependencies once per owner. Cast and local
annotations reuse the declaration elaborator with their lexical context; a cast
through `I<u8>` must select `u8`, not the alias's symbolic template. Inactive syntax
is excluded. Tuple constructors retain real callable metadata, initialized once;
no provisional empty field lists or fabricated TypeIds are used.

All actual Drop owners are reserved before any impl target is fully elaborated.
The separate head query yields a nominal item, a symbolic type parameter, or a
nonnominal head from resolved syntax. It follows alias applications and only the
defaults needed for that head, substitutes supplied arguments simultaneously
once, and uses iterative memoized queries with explicit active states. This
preserves finite self/mutual defaults and rejects real head/default cycles.
Full target argument validity and destructor conformance still use ordinary
elaboration. No incomplete destructor implementation is manufactured. Early
constant cleanup and enum-cast eligibility consume the reserved owner set, so
an owned temporary cannot evade Drop just because its impl occurs later or its
own target contains the constant. Reading immutable static storage remains valid.

The maintainer readiness tests independently qualify 78 distinct source cases
against pinned rustc. Additional invariants check 512 defaults sharing one field
template and the exact array length, finite versus cyclic field dependencies,
and a 1,024-link generic alias chain selecting the actual destructor owner.
Six new native alias programs exercise early tuple/reference defaults, named enum
casts, wide packed enum storage, destructor target defaults with an observed drop
count, and a declared destructor panic after an immutable static read.

This resolves the field/header prerequisite described in the preceding
checkpoints. It does not activate PA18–PA21. Mixed nominal source parameters,
scoped field obligations, conditional derives, constructor/pattern inference,
generic Drop coverage/instance selection, deferred const-length promotion and
demanded native body specialization remain part of the full PA18 contract.
PA19–PA21 retain their complete trait, solver and advanced lifetime scope.
No assignment manifest or course oracle is changed by this checkpoint.

Validation passed 676 frontend tests (nine backend tests explicitly ignored in
that default run), all 3,350 active PA1–PA17 text cases, 35 focused release tests
and all nine explicitly selected native maintainer tests. The alias execution
suite now contains 27 programs: 24 clean executions and three declared panics,
compared independently through rustc and the qualified CPPGM backend. Rustfmt,
Clippy all/pedantic with warnings denied, the 419-file audit (zero findings) and
25 harness checks passed. The logs are `nominal-readiness-full-v2.log`,
`nominal-readiness-release-v1.log`, `nominal-readiness-native-v1.log` and
`nominal-readiness-validation-v5.log` under `obj/pa18-work/`.

The initial readiness probes preserved one newly investigated wrong expectation:
an impl-target constant constructing its own lifetime-bearing Drop type is
rejected by rustc because the temporary is destroyed. The committed regression
correctly declares rejection. The earlier-slot unused type parameter hypothesis
from the previous checkpoint remains excluded from positive tests. Early failed
lint/audit runs and exploratory probe outputs remain retained, not overwritten.

The performance comparison froze 289 production/configuration files and the
previous `d815346` binary against candidate SHA256
`efcbf4ae278e59e92cfb401115afe61e87401d9527d5ea46bf49152332cf4557`.
Seventeen workloads retained 306 observations on CPU 2, each with two warmups,
four A/A calibration samples and three ABBA blocks. A targeted repeat retained
54 further observations for enum controls and a noisy packed-scalar case. No
local builds or tests overlapped timing. Every source, binary and input hash,
full output and count output was checked. New early-field cases compare two
immutable copies of the candidate because the parent rejects those inputs.

Paired wall-time medians in milliseconds:

| Workload | Scale | Parent | Candidate |
| --- | ---: | ---: | ---: |
| implicit-enums | 4096 | 13.28 | 15.24 |
| explicit-enums | 4096 | 16.35 | 18.11 |
| drop-scalar-arguments | 4096 | 58.52 | 58.34 |
| drop-static-arguments | 4096 | 59.81 | 59.99 |
| thinrefs | 4096 | 478.84 | 480.95 |
| packed-scalar | 1024 | 31.95 | 31.11 |
| wide-enum-borrows | 1024 | 36.77 | 37.93 |
| packed-scalar | 4096 | 217.74 | 242.37 |
| wide-enum-borrows | 4096 | 134.06 | 136.79 |
| packed-scalar | 16384 | 513.48 | 518.24 |
| wide-enum-borrows | 16384 | 599.19 | 620.91 |
| drop-alias-chain | 1024 | 14.25 | 16.33 |
| early-fields | 1024 | — | 43.05 |
| drop-alias-chain | 4096 | 59.81 | 66.14 |
| early-fields | 4096 | — | 180.40 |
| drop-alias-chain | 16384 | 242.43 | 280.62 |
| early-fields | 16384 | — | 737.71 |

The first packed-scalar 4,096 run varied from roughly 101–258 ms across the
pair. Its retained repeat measured 112.38 → 115.21 ms with overlapping ranges
109.69–120.32 / 110.96–124.25 ms. Repeated implicit/explicit enum controls measured
14.00 → 15.28 / 17.15 → 19.34 ms. The field/constructor indexes and exact owner
query have measurable cost: the 16,384-alias control increased 242.43 → 280.62 ms,
with roughly 59 → 61 MiB RSS. These are added correctness prerequisites, not a
claimed speedup. Newly supported early fields used approximately 11/33/117 MiB
at 1,024/4,096/16,384 declarations. Both noisy and repeated observations remain.

Evidence is in `nominal-readiness-v1-{freeze,source-sha256,preflight,perf-jobs,
perf-results,repeat-jobs,repeat-results,validation,reverify}.json`, the per-job
`obj/perf/*/samples.json` and `observations.jsonl`, and
`nominal-readiness-{perf,repeat}-v1.log`. No observations were discarded.

### Nominal contracts and unresolved constructor inference

Source structs and enums now retain mixed lifetime/type/const parameter schemas,
defaults and immutable declaration environments. Fields, signatures and nominal
uses prove Sized/Copy/Clone requirements in their own environment. Copy, Clone
and equality derives use separate generated assumptions, including packed Copy
requirements. Finite-storage inference propagates by-value parameter requirements
without expanding recursive instances; zero arrays still require alignment.
Symbolic outlives contracts retain type-parameter requirements, and explicit
static requirements remain obligations. Closed type defaults are validated one
slot at a time, without composing unrelated defaults into a stronger requirement.

Generic representation checks retain symbolic layouts as unknown. A declaration
still validates its own representation and concrete sublayouts. Transparent side
fields use cached symbolic queries with retained failures; unrelated queries can
continue after failure. The packed containment check follows pinned rustc's
direct struct-field graph before substitution, independently of physical layout.

Constructors, records, variants, fields and patterns share unresolved type/const
arguments and one substitution session per receiver. Repeated field templates
reuse completed subgraphs. Explicit partial argument lists apply trailing defaults;
omitted, empty and lifetime-only argument lists permit inference. Partial local
annotations reuse these same arguments, including alias applications. Expected
constructor result types constrain field arguments before function-item coercion.
Alias-qualified enum variants use retained resolution-prefix facts and canonical
member identities. Callable constructor metadata keeps its value identity separate
from the nominal owner of its parameters.

Unused generic constructors are excluded from concrete executable CFGs and their
contracts. Closed record literals already lower and execute with actual layouts.
Demanding a callable generic constructor still diagnoses the missing demanded
instance stage. The PA12 generic boundary now reports that precise limitation;
PA17's type-generic Drop boundary reaches its remaining impl-header restriction.
Pinned rustc accepts both programs, as already declared in their manifests. Only
these two diagnostic sidecars and their generated source-hash receipts change;
no expected outcome or fixture is weakened.

PA18–PA21 remain planned. Generic Drop conformance and inherited method
environments, generic constructors in early shape constants, deferred promotion,
and demanded executable body specialization remain required PA18 work. PA19–PA21
retain their complete trait, solver and advanced lifetime responsibilities.

Validation passed all 699 frontend tests in both debug and release builds, all
3,350 active PA1–PA17 text cases and all ten explicitly selected native maintainer
tests. A final narrowing of expected-result inference to mixed generic constructors
preserves the existing PA15 region oracle; 29 targeted tests passed in each build
mode afterward, as did the new native record checks. Rustfmt, Clippy all/pedantic
with warnings denied, the 442-file audit with zero findings, all 25 harness tests
and upstream provenance checks passed. The complete run logs and earlier failed
attempts remain under `obj/pa18-work/constructor-*`.

Independent source probes qualify nominal declarations, defaults, derives,
constructor inference, partial annotations and higher-ranked alias cases against
pinned rustc. The probes retain rejected hypotheses as evidence: an inferred
higher-ranked annotation is valid in one direct form, while three nested alias or
default forms are insufficiently general. Regression expectations follow the
independent outcomes. Additional typed invariants cover 4,096 shared field
subgraphs, finite storage propagation, scoped derive assumptions, binder-safe
parameter insertion and failed representation-query caching.

Performance froze 304 production/configuration files and baseline `eb821615`
against candidate SHA256
`7ec1508aa96c03fe2b8bcdf5033dfe00c3026fd5efca448586874b769652afe2`.
Twenty-one workloads retained 378 observations on CPU 2: two warmups, four A/A
calibration samples and three ABBA blocks per job. No RustGM builds or tests
overlapped timing. Full/count outputs and every source, binary and input hash
verified. The nine newly supported inputs were independently accepted by rustc;
they compare two immutable candidate copies because the parent rejects them.

Paired wall-time medians in milliseconds:

| Workload | Scale | Parent | Candidate |
| --- | ---: | ---: | ---: |
| implicit-enums | 4096 | 27.58 | 28.13 |
| explicit-enums | 4096 | 20.02 | 20.40 |
| drop-scalar-arguments | 4096 | 64.55 | 65.46 |
| drop-static-arguments | 4096 | 68.26 | 68.96 |
| thinrefs | 4096 | 505.22 | 507.82 |
| packed-scalar | 1024 | 34.71 | 35.31 |
| wide-enum-borrows | 1024 | 39.58 | 40.84 |
| packed-scalar | 4096 | 129.41 | 132.62 |
| wide-enum-borrows | 4096 | 152.96 | 158.54 |
| packed-scalar | 16384 | 516.19 | 548.68 |
| wide-enum-borrows | 16384 | 635.09 | 644.32 |
| drop-alias-chain | 4096 | 65.36 | 66.23 |
| nominal-chain | 256 | — | 8.24 |
| record-functions | 256 | — | 10.60 |
| wide-record | 256 | — | 4.90 |
| nominal-chain | 1024 | — | 23.00 |
| record-functions | 1024 | — | 30.87 |
| wide-record | 1024 | — | 8.81 |
| nominal-chain | 4096 | — | 96.00 |
| record-functions | 4096 | — | 115.95 |
| wide-record | 4096 | — | 24.76 |

Existing controls mostly increase by a few percent. The 16,384 packed-scalar
case increases 516.19 → 548.68 ms, with paired ranges 512.59–539.85 and
521.78–584.54 ms. These observations include the cost of the new declaration
contracts and are not a speedup claim. At 4,096 units the new nominal chain,
record-function and wide-record cases use about 23, 21 and 9 MiB peak RSS.
Their measurements support bounded growth over these input sizes, not a general
complexity guarantee. Typed work-count invariants separately guard against
re-expanding shared field templates and recursive storage instances.

Evidence is in `constructor-v2-{freeze,source-sha256,preflight,perf-jobs,
perf-results,reverify}.json`, `constructor-perf-v2.log` and each retained
`obj/perf/*/{samples.json,observations.jsonl}`. The earlier preflight candidate,
pre-fix PA15 mismatch, boundary qualification attempts and source probes remain
retained. No timing observations were discarded and no unrelated oracle changed.

## Early mixed arguments in shape constants

Early shape constants now use the same type/const argument scheduler as ordinary
expression paths. Constructor and function values can refer forward to their
headers, alias-qualified variants publish their callable facts, and explicitly
partial argument lists demand their consumed trailing defaults. Readiness also
follows types introduced by those defaults and partial annotations. Omitted,
empty and lifetime-only lists retain inference behavior. These changes extend
PA18 declaration support; they do not implement general constant statements,
const function calls or executable generic function instances.

Value-readiness traversal follows each nominal declaration's fields once and
actual argument types separately. It no longer substitutes every intermediate
symbolic field graph. The 512-owner shared-chain regression originally performed
1,062,407 field-instantiation steps; the corrected test requires fewer than
32,768 steps and 16,384 canonical types. A separate 512-use repeated-default
case requires fewer than 64 field-instantiation steps.

Const arguments reject syntactic references to enclosing type, const and lifetime
parameters. A permitted standalone const parameter remains symbolic. Lifetimes
bound by a higher-ranked function type inside the constant remain valid, including
macro-generated types. Cached minimum/maximum scope-preorder positions summarize
captures without copying free-variable sets into each enclosing expression.
Aliases cannot erase an otherwise forbidden syntactic capture.
Completion flags use compact node-indexed tables; only actual captures allocate
hash-map records. This avoids storing empty capture summaries for ordinary syntax.

Nested constants elaborate annotations in their own body contexts. An inferred
annotation lifetime allocated before body inference still registers a proof body,
even if an alias subsequently erases that lifetime and inference adds no new facts.
This prevents orphaned body-region identities and preserves the ordinary region,
loan and destruction proof requirements.

The new maintainer regressions contain 35 constructor cases and 37 argument/scope
cases. Positive concrete array lengths are independently checked with both an
exact-length and an off-by-one rustc call. Argument/scope cases also collect and
prove the resulting region contracts. Seven native programs compare CPPGM-backed
execution with rustc, including an intentional value-failure control. The original
probe receipts retain a rejected hypothesis: `{(N)}` is not a permitted standalone
const parameter in this pinned stable profile; the regression expects rejection.

PA18–PA21 remain planned. The next PA18 work includes generic Drop conformance,
inherited method environments, deferred const lengths/promotions and demanded
executable instances of retained generic body templates.

Performance froze 305 production/configuration files and compared parent
`ae49bc4` with candidate SHA256
`35bbe024d8221399b7cc7fccab817a330ea79f5524235bc626d838fad5b460df`.
Thirty workloads retained 540 observations on CPU 2: two warmups, four A/A
calibration samples and three ABBA blocks per job. No RustGM builds or tests
overlapped timing. All full/count outputs and frozen source, binary and input
hashes verified. Nine newly supported inputs were independently accepted by
rustc, with exact-length acceptance and off-by-one rejection controls; these
compare two immutable candidate copies because the parent rejects them.

The first complete run also retained 540 observations. Its empty capture-summary
hash entries increased several existing workload medians by 13–19%. Compact
completion flags removed most of that overhead. The final run still measures
cost: drop-scalar-arguments increases 64.80 → 69.96 ms (+7.9%), and packed-scalar
at 16,384 increases 530.60 → 563.76 ms (+6.2%). Most other existing comparisons
are within a few percent in either direction. No speedup is claimed from these
short, noisy measurements; every observation from both candidates is retained.

Selected paired medians in milliseconds (— means candidate-only qualification):

| Workload | Scale | Parent | Candidate | Candidate RSS, KiB |
| --- | ---: | ---: | ---: | ---: |
| drop-scalar-arguments | 4096 | 64.80 | 69.96 | 14194 |
| drop-static-arguments | 4096 | 68.95 | 65.56 | 13948 |
| thinrefs | 4096 | 484.56 | 500.51 | 77764 |
| packed-scalar | 4096 | 127.51 | 131.49 | 21198 |
| wide-enum-borrows | 4096 | 154.12 | 156.91 | 26034 |
| packed-scalar | 16384 | 530.60 | 563.76 | 70652 |
| wide-enum-borrows | 16384 | 611.80 | 618.22 | 91534 |
| shared-early-fields | 128 | — | 14.94 | 7194 |
| shared-early-fields | 512 | — | 47.81 | 12794 |
| shared-early-fields | 2048 | — | 197.99 | 35204 |
| repeated-early-defaults | 256 | — | 18.54 | 7130 |
| repeated-early-defaults | 1024 | — | 67.69 | 13472 |
| repeated-early-defaults | 4096 | — | 260.15 | 39604 |
| local-constant-binders | 256 | — | 22.07 | 8544 |
| local-constant-binders | 1024 | — | 84.43 | 17692 |
| local-constant-binders | 4096 | — | 357.63 | 56548 |

The shared-field, repeated-default and local-binder families show bounded growth
across these measured sizes. The typed instantiation-count invariants provide
separate evidence against repeated symbolic field expansion; timings alone do
not establish an asymptotic complexity bound.

Evidence is retained in `early-uses-v{1,2}-{freeze,source-sha256,preflight,
perf-jobs,perf-results}.json`, the read-only source snapshots and candidate
binaries, both `early-uses-perf-v*.log` files, and each retained
`obj/perf/*/{samples.json,observations.jsonl}`. Independent source and rejected
hypothesis receipts are under `early-uses-v{3,4,5}/`; the initial quadratic
field-instantiation failure remains in `early-uses-sharing-v1.log`.

Final local validation passed all 703 frontend tests, all 3,350 active PA1–PA17
text cases and all eleven explicitly selected native maintainer tests. The final
cache change also passed twenty focused tests, Rustfmt, Clippy all/pedantic with
warnings denied and the 445-file audit with zero findings. All 25 harness tests
and upstream provenance checks passed. The initial candidate also passed all
703 release tests; hosted CI repeats release validation for the final commit.
No existing oracle or declared course outcome changed. Validation and final
source/binary/input verification receipts are in `early-uses-v2-validation.json`
and `early-uses-v2-reverify.json`.

## Generic Drop conformance and inherited method environments

Generic Drop headers now elaborate type, const and lifetime parameters. The
canonical target must cover every nominal argument with a distinct direct impl
parameter; type/const parameters cannot remain unconstrained. Canonical aliases
and reordered arguments are supported. Impl defaults, specialized targets,
repeated arguments and extra requirements remain errors.

Conformance inverts the target mapping into the original nominal parameter
environment. It proves builtin requirements there, including implicitly Sized
impl parameters, before the method body can assume them. This distinguishes a
nominal `T: ?Sized` from an impl that silently requires `T: Sized`. The shared
outlives graph proves lifetime and type-parameter requirements, treating unused
impl lifetime slots existentially. Anonymous impl lifetimes have separate binder
slots and never renumber declared type/const argument indices.

Methods inherit the impl's completed parameter environment while retaining each
parameter's original owner/index. Region contracts, symbolic CFG classification,
Copy/shape queries and cleanup proofs consume that inherited environment. Method
bounds may repeat requirements already implied by the nominal; they cannot add
requirements or make the receiver lifetime early-bound. Explicit `Self`
annotations consume the completed target without demanding written impl arguments.

The destructor conformance suite qualifies 56 declared language cases with pinned
rustc before checking their symbolic ownership, lifetime, loan and cleanup
results. Additional invariants check reordered field identities, a 256-parameter
schema, separate captured-binder substitution, cache reuse and ordinary generic
functions using implicit field dereferences. The two original probes with unsized
by-value tail fields remain a PA22 boundary; sized pointer/function-field controls
exercise the Drop `?Sized` conformance rule without claiming unsized aggregates.
Higher-ranked where-clause binders remain PA21 work; nested callable type binders
are included in the current checks.

Field inference already retained the selected nominal receiver, but an implicit
dereference did not necessarily have its own syntax node. Publishing solved body
types now prepares those receivers' fields in recorded discovery order. This
makes the completed field types available to immutable template construction and
keeps canonical type identities independent of hash-map iteration. A repeated
analysis test checks the resulting type order across fresh randomized maps.

This is semantic and symbolic-body support. Demanded executable instances,
generic destructor recipes and specialization of deferred const/promotion facts
remain required PA18 work. PA18–PA21 remain planned.

Local validation passed 709 frontend tests in both debug and release, all 3,350
active PA1–PA17 course cases, all eleven explicit maintainer native tests, the
independent rustc suites and PA17's 135 executable comparisons. Rustfmt, Clippy
all/pedantic with warnings denied, the 448-file audit, all 25 harness tests and
upstream provenance checks passed. The only course oracle change is PA17's
existing generic boundary diagnostic: header elaboration now succeeds, then
concrete body construction reports its missing instantiation/destruction stage.
Its source is independently accepted by rustc; its course outcome remains the
explicitly documented rejection. The generator refreshed PA17's source receipt.

Performance compared baseline `c729b4e` with candidate SHA256
`49a1848144316adc55bcc83f9b501e34853c1d3987f2dc43dc58ff85ddb4095a`,
with 306 frozen production/configuration files. Thirty-six jobs retained 648
observations on CPU 2, using two warmups, four A/A calibration samples and three
ABBA blocks per job. No RustGM builds or tests overlapped timing. All binary,
source and input hashes and full/count output equality verified. Twenty-nine of
thirty existing workload medians remained within 3.3% of the parent in either
direction. Repeated early defaults at scale 256 measured 24.15 → 20.88 ms
(-13.6%); these short, noisy measurements do not establish a speedup.

Six newly supported type-stage inputs were independently accepted by rustc.
They compare immutable candidate copies, since the parent rejects generic Drop.
These timings cover declaration/body type analysis, not generic native execution.
The wide schema reverses its type parameters; the many-method family exercises
inherited Copy requirements, implicit field access and symbolic const arrays.

| Workload | Scale | Candidate median, ms | Candidate RSS, KiB |
| --- | ---: | ---: | ---: |
| wide-drop-schema | 128 | 4.93 | 5056 |
| wide-drop-schema | 512 | 8.88 | 6468 |
| wide-drop-schema | 2048 | 25.56 | 9248 |
| many-drop-methods | 128 | 15.86 | 7134 |
| many-drop-methods | 512 | 56.86 | 14168 |
| many-drop-methods | 2048 | 242.41 | 42206 |

The new families show bounded growth over these measured sizes. Separate typed
storage/work invariants guard against repeated schema and field expansion.
Evidence remains under `obj/pa18-work/generic-drop-*`, including freeze,
preflight, independent source qualification, validation and reverify receipts,
and each retained `obj/perf/*/{samples.json,observations.jsonl}`. Earlier failed
exploratory runs remain retained and are not counted as passing validation.

## Symbolic index bounds, const operands and deferred promotion

Fixed-array indexing now retains the canonical const parameter when its length
is symbolic. The checker publishes the selection after inference, so a length
hole that later resolves to a concrete argument becomes an ordinary numeric
bound. Separate compact selection variants preserve the 80-byte semantic node
layout. Symbolic indexes use the existing conservative indexed-place rules for
moves and overlapping loans; executable lowering still requires a concrete bound.

Runtime const-parameter uses have their own typed operand, distinct from named
constant definitions. Joint substitution preserves the owner and scalar kind;
evaluated bool, char and integer arguments enter the shared value engine without
losing their bits. Unresolved operands cannot silently become runtime constants.

Promotion records eligibility independently of an evaluated value. Borrows of
`N`, `N + 1` and `[3; N]` may supply permanent storage provenance while retaining
the original typed DAG for later instantiation. A repeat of unknown length still
requires an eligible operand. Division/remainder eligibility requires a known
safe divisor; signed division by negative one additionally needs a safe dividend.
An unknown array bound cannot justify promoting a value-level index operation,
although the array backing a borrowed element can itself be promoted. Original
calls, arithmetic and bounds checks remain in the body.

Storage, region and abstract cleanup checks consume proved eligibility. Concrete
borrow verification rejects any unresolved promoted value before it can mint an
executable proof. Demanded instantiation must substitute operands/bounds and
materialize deferred values; this checkpoint does not implement that stage.
PA18–PA21 remain planned.

The new operand suite independently qualifies thirteen language cases and checks
nested const identities, inference-resolved bounds, kind-preserving substitution,
and symbolic ownership/loan outcomes. The promotion suite independently qualifies
28 positive/negative language cases plus cache, effect-preservation and scaling
inputs. It checks full symbolic proofs, rejects attempts to authorize concrete
execution, and bounds promotion work for 128, 512 and 2,048 retained repeat roots.
The existing concrete-promotion and generic-region suites remain active.

Local validation passed 719 frontend tests in both debug and release, all 3,350
active PA1–PA17 course cases, all eleven explicit maintainer native tests, every
active independent rustc suite and PA17's 135 executable comparisons. Rustfmt,
Clippy all/pedantic with warnings denied, the 451-file audit with zero findings,
25 harness tests and upstream provenance checks passed. No course fixture,
manifest, outcome or text oracle changed.

Performance froze 307 production/configuration files and candidate SHA256
`6b949520b0341c1970ee5072c340214c00379566615508705a1979b27b7d9b1f`.
Identical symbolic-proof drivers were built against separate frozen parent and
candidate libraries. Thirty-two jobs retained 576 observations on CPU 2, with
two warmups, four A/A calibration samples and three ABBA blocks each. No RustGM
build or test overlapped measurement. All full/count outputs agreed. Twenty-three
parent comparisons stayed within 2.7% in median wall time; these measurements do
not establish a speedup. They include concrete LowIR, declaration analysis,
ordinary generic promotions and complete template borrow/cleanup proofs.

Twelve new scaling inputs were independently accepted by pinned rustc. Nine
require the new functionality and therefore compare immutable candidate copies;
the three literal-promotion controls also run against the parent. The following
medians pool both candidate copies' paired samples and cover parsing through
complete symbolic proofs, not executable generic lowering.

| Workload | Scale | Median, ms | RSS, KiB |
| --- | ---: | ---: | ---: |
| symbolic-index | 128 | 12.71 | 6412 |
| symbolic-index | 512 | 38.51 | 9252 |
| symbolic-index | 2048 | 133.70 | 22624 |
| const-arithmetic | 128 | 7.21 | 5716 |
| const-arithmetic | 512 | 17.79 | 7140 |
| const-arithmetic | 2048 | 54.77 | 14914 |
| deferred-promotion | 128 | 10.32 | 5734 |
| deferred-promotion | 512 | 29.89 | 8536 |
| deferred-promotion | 2048 | 111.96 | 19804 |

Frozen source, drivers, binaries, declarations, preflight results, qualification,
validation and reverify receipts remain under `obj/pa18-work/deferred-promotions-*`.
Each performance receipt links its complete raw `observations.jsonl`. Exploratory
compile/lint failures remain retained and are excluded from passing results.

## Demanded execution through the maintainer API

`lower::instances::build` now checks every generic template and specializes
demanded type/const function bodies from their retained CFGs. It handles direct,
recursive and mutually recursive calls, inferred function values, function
pointers in constants/statics and promoted allocations, symbolic array lengths,
const operands and deferred promotions. The course CLI still retains its PA18
boundary. Generic constructor factories and concrete generic destructor selection
remain necessary before that boundary can be released; PA18–PA21 stay planned.

Callable values retain their full function-item type and argument identity.
Lowering caches code by the selected instance and promoted storage by concrete
body index. Different const instances therefore retain different backing values.
Ordinary functions keep their existing symbols and behavior.

Template verification returns a token borrowing the immutable original bodies.
Materialization substitutes explicit parameter identities, preserving source
keys, control flow, effects and cleanup scopes. A separate instance contract
abstracts free lifetimes inside actual type arguments into fresh universals;
nested callable binders and original template variables remain distinct. Region
proofs carry the selected instance and are paired in body order. Swapping proofs
for two instances of one source is rejected. Ordinary proof entry points cannot
authorize specialized bodies.

The new lifetime tests exposed inferred caller lifetimes being captured under
late callable binders. Generic shape inference now uses free aliases for caller
universals, with explicit equalities retaining their original meaning. Selected
signatures are still validated against the substituted declaration template.

The demand worklist registers an instance before following its dependencies and
bounds active recursion per definition independently of the number of separate
uses. Retained body storage is checked before cloning. Dependency traversal
follows typed constant graphs, including projected allocations; it neither
expands repeat values element by element nor demands discarded function-item
values merely because their type exists.

The execution suite declares 25 programs: twenty-one clean runs and four runtime
panics for overflow or invalid indexes. Each runs through this API and the
qualified CPPGM backend against pinned rustc. Three additional non-generic
constructor-pointer cases exercise the ordinary CLI. Typed tests cover sparse
parameter identities, capture avoidance, separate instance proofs, preserved
cleanup scopes, unused invalid templates, recursion limits and 160 independent
instances of one function. Full regression and performance evidence is recorded
with this checkpoint below.

Validation passed all 731 frontend tests in debug and release, 3,350 active
PA1–PA17 course cases, thirteen explicit maintainer native tests and all six
course native suites (858 executable comparisons, including the sixteen already
documented PA13 differences). Every active independent rustc suite passed.
Rustfmt, Clippy all/pedantic with denied warnings, the 463-file audit with zero
findings, 25 harness tests and upstream provenance checks passed. Course fixtures,
manifests, expected outcomes and text oracles are unchanged.

Performance froze 315 production/configuration files and candidate SHA256
`27bb5d154840fd128de581d4a073c11c26cec455af33875992dde9c7eb640209`.
Twenty-nine jobs retained 522 observations on CPU 2 with two warmups, four A/A
samples and three ABBA blocks each. Twelve new inputs were independently accepted
by pinned rustc. They measure parsing through checked generic LowIR construction
using immutable candidate copies; the parent has no such executable path.
Seventeen parent comparisons cover ordinary LowIR, declarations and complete
symbolic proofs. Full/count outputs agree. No RustGM builds or tests overlapped
measurements.

Parent comparisons ranged from -0.7% to +5.0% in median wall time. The largest
difference, symbolic indexing at 2,048 functions, was within its A/A variation;
a targeted repeat retained eighteen additional observations and measured -3.1%,
with broad overlapping ranges. Both runs are retained (540 observations total).
These shared-host timings do not establish a speedup or a consistent change for
that workload.

| New executable workload | 128 median, ms | 512 median, ms | 2048 median, ms | 2048 RSS, KiB |
| --- | ---: | ---: | ---: | ---: |
| repeated instance | 5.15 | 8.66 | 22.17 | 8578 |
| distinct const instances | 5.78 | 12.85 | 44.37 | 12074 |
| promoted instance values | 8.11 | 19.04 | 77.15 | 17664 |
| reference argument instances | 16.00 | 55.33 | 265.40 | 38082 |

Frozen source, three drivers, binaries, inputs, qualification, validation and
reverification receipts are under `obj/pa18-work/demanded-execution-*`, with raw
samples under `obj/perf/`. The separate `demanded-dead-paths-v1` probe retained a
disproved hypothesis: pinned rustc still rejects growing instantiation in both a
literal-false and a const-false branch; this implementation also rejects those
programs. Earlier failed compile/lint attempts remain excluded from pass claims.

## Generic constructor instances

The demanded-instance API now includes tuple-struct and enum-variant constructor
factories. Type checking retains referenced constructor identities, avoiding
synthetic bodies for unused declarations. Ordinary and generic constructors use
the same field-move CFG builder.
Implicit generic bodies retain the nominal declaration's parameter assumptions,
lifetime contract and owned-value scopes; enum variants share one immutable
parameter environment. Their original proofs precede ordinary demanded
materialization and the separate instance proofs.

The maintainer suite declares eighteen clean constructor executions covering
inferred and explicit items, direct calls, local/constant/static/promoted/returned
function pointers, const arrays, enum variants, nested generic calls, borrowed
fields, owned field transfers and zero-sized results. Four independent rejection
controls cover borrowed-storage escape, duplicate moves and unsatisfied Copy
bounds. A typed test checks shared nominal assumptions, field transfers and
return scopes before specialization.

These cases exposed constructor inference capturing a caller lifetime beneath a
callable binder, and declaration-context generic return types lacking prepared
field facts after expression types acquired free lifetime aliases. Constructor
inference now follows the same alias discipline as generic functions. Return
field preparation retains both contexts for subsequent immutable ownership
queries. Earlier assignment oracles remain unchanged. The course CLI still
retains the PA18 boundary; concrete generic destructor selection and cleanup
integration remain required before release.

The final constructor revision passes 735 frontend tests in debug and release,
all 3,350 PA1–PA17 course cases, fourteen explicit maintainer native tests and
858 course executable comparisons, with the same sixteen documented PA13
runtime differences. All active independent checks, Clippy all/pedantic with
denied warnings, rustfmt, the 465-file audit with zero findings, harness checks
and upstream provenance checks pass. The unused-factory invariant separately
checks that unreferenced implicit constructors create no synthetic bodies.

The first measured implementation built factories for all declarations. At
2,048 unused generic enums this took 193.20 ms versus the parent's 63.75 ms.
That evidence prompted the sparse referenced-constructor set described above.
The initial frozen sources, binaries and 252 raw observations remain retained;
the final revision is compared against both that version and the parent.

The final run retains 306 more observations across seventeen jobs, including
three direct comparisons with the first candidate. At 2,048 unused enums that
comparison measures 193.26 ms before and 62.67 ms after, with median RSS reduced
from 28,918 to 17,718 KiB. The separate parent comparison measures 63.82 versus
64.82 ms; all eight parent comparisons range from -0.9% to +1.6%. These are
shared-host wall-time observations, with A/A calibration and every sample
retained, rather than a fixed performance guarantee.

| Final constructor workload | 128 median, ms | 512 median, ms | 2048 median, ms | 2048 RSS, KiB |
| --- | ---: | ---: | ---: | ---: |
| repeated factory calls | 5.80 | 10.23 | 27.51 | 9906 |
| distinct const factories | 8.09 | 19.27 | 79.04 | 14976 |
| unused generic enums | 7.78 | 18.13 | 64.82 | 17670 |

Both runs retain 558 observations in total. Full and count outputs agree;
nine new inputs pass pinned-rustc code generation. No RustGM build or test
process overlapped either measurement run. The final snapshot contains 316
production/configuration files and CLI SHA256
`1652eb675d5e07a84d84d096b757020db0567b7e937dbfd20f1f825a39215c19`.
Receipts, immutable inputs/binaries and independent evidence remain under
`obj/pa18-work/constructor-instances-*`, with raw samples under `obj/perf/`.
Earlier failed implementation attempts and the superseded candidate are retained
separately from the final validation claims.


## Concrete destructor selection and recipes

Completed Drop implementations now retain their checked inverse mapping from
impl parameters to the nominal schema. The maintainer selection API uses it to
form a canonical method instance for a concrete target, including permuted type
arguments and bool, char and signed const arguments. Callable captures preserve
the target's lifetime order and an independent late receiver. Unused impl
lifetimes remain in the original proof with their bounds; a separate check
prevents erasing any lifetime that occurs in the callable signature.

Recipe construction consumes prepared concrete field lists and uses the complete
canonical type as the key for type/const instances. Declaration member identities
and order remain checked. Ordinary lifetime-only recipes keep their shared key.
The retained cache can release its semantic borrows and resume after append-only
type discovery, preserving recipes, successful/failed query states and budgets.
Generic method recipes remain explicitly unresolved until a method instance is
selected for emission.

Four new independently qualified typed tests cover argument permutations,
distinct captured and nested reference lifetimes, preservation of unused method
proof lifetimes, concrete field/array recipe selection and cache reuse. The
existing preparation invariant now verifies that a cached lifetime-only result
cannot bypass missing generic field facts. The distinct-lifetime test initially
assumed declaration-order slots; its final assertions use the canonical target
identities, which are assigned by first use.

This is an intermediate PA18 implementation. Actual cleanup-driven method demand
and combined instance/destruction proofs still need integration into the
scheduler. The maintainer generic execution API and the course CLI retain their
existing pending-Drop and PA18 boundaries. Assignment statuses, fixtures,
manifests and text oracles remain unchanged.

A frozen planning probe checks eleven previously rustc-qualified inputs. Seven
select a runtime cleanup, including the demanded growing-destructor case; unused
Drop and three diverging cases select none. Two diverging cases nevertheless
retain an unresolved generic recipe after planning. This directly confirms that
future method demand must traverse selected cleanup roots rather than every
cached recipe. These are initialization/planning observations, not claims of
completed generic destructor execution.

Validation passes 739 frontend tests in debug and release, all 3,350 PA1–PA17
course cases, fourteen maintainer native tests and 858 course executable
comparisons, preserving the sixteen documented PA13 differences. All active
independent checks, rustfmt, Clippy all/pedantic with denied warnings, the
468-file audit with zero findings, 25 harness tests and upstream provenance
verification pass. Earlier failed test/lint attempts remain separately recorded.

Performance freezes 318 production/configuration files and candidate SHA256
`903b03bcf29be213f3773ff7edf82b85bdf446c61deba6b80c05a233d2941e6c`.
Eighteen jobs retain 324 observations on CPU 2, with two warmups, four A/A samples
and three ABBA blocks per job. Fifteen new inputs pass pinned-rustc code
generation. No RustGM builds or tests overlap measurement. Full and count output
equality and all frozen source, binary and input hashes verify.

Nine parent comparisons exercise ordinary shared/distinct cleanup and generic
Drop declarations. Median wall-time differences range from -3.9% to +3.0%, with
overlapping ranges; these shared-host measurements do not establish a consistent
speed change. At 2,048 distinct ordinary destructors, median RSS is 47,074 KiB
versus the parent's 46,560 KiB. All observations, including wider wall-time
outliers in that workload, are retained.

The new API workload measures parsing, semantics, concrete CFG construction,
recipe construction, method selection and cached repeat queries. It does not yet
measure executable generic cleanup or original template proof:

| Selection workload | 128 median, ms | 512 median, ms | 2048 median, ms | 2048 RSS, KiB |
| --- | ---: | ---: | ---: | ---: |
| repeated concrete target | 10.30 | 15.75 | 51.43 | 12760 |
| distinct const targets | 12.63 | 20.84 | 71.91 | 16254 |
| generic fields and arrays | 15.87 | 29.09 | 105.36 | 19762 |

Frozen inputs, sources, drivers, binaries and verification receipts are under
`obj/pa18-work/drop-recipes-*`; raw observations are under `obj/perf/`.
`drop-plans-probe-v1` retains the separate planning observations described above.

## Executable generic destructor instances

The maintainer generic execution API now follows selected cleanup roots into
concrete Drop method instances. Those methods share the ordinary depth-first
instance worklist and its recursive-growth limits. Merely cached recipes, unused
functions and diverging ownership paths do not force destructor code generation.
Fields, enum variants and nonempty array intervals contribute method dependencies
without expanding arrays element by element.

Each body receives its region, initialization, ownership and destructor-loan proof
before its executable dependencies are followed. An opaque token owns the body
and optional cleanup plan, keeping them immutable until final assembly. Assembly
checks cleanup coverage and rejects erasing a completed destruction proof. Original
generic templates, including unused bodies and preserved impl lifetimes, remain
separately mandatory. Structural region queries and recipe state survive short
semantic borrows; their caches and work budgets are not rebuilt for each method.

The partial-move tests exposed a missing semantic prerequisite: coercion can
produce a nominal lifetime instance that has no expression node of its own.
Adjustment publication now prepares that destination's concrete fields. A test
that read a counter while a surviving wrapper field retained its mutable borrow
was independently rejected by rustc; it is retained as a negative control, with
the positive counter assertions after the scope closes.

Eighteen native counter programs cover reordered type, lifetime and const
arguments, unused/intermediate impl lifetimes, nested reference arguments,
multiple instances, custom Drop before fields, enum fields, partial moves,
conditional initialization, generic arrays and evaluated zero-array prototypes.
Separate controls check both ordinary-call and destructor-to-destructor recursive
growth, codegen-only diverging programs, unused owners, invalid field moves,
double moves, dead referents and active loans. Proof-boundary tests reject swapped
instances, foreign owners, missing original templates, unproved assembly bodies
and erased cleanup authorization.

PA18 is still planned: this checkpoint enables the maintainer API, while CLI
activation, its assignment contract, course fixtures/oracles and release evidence
remain required. PA19–PA21 remain planned as well.

The first scaling candidate rejected 2,048 sequential borrowed cleanup scopes
with `value liveness work budget exceeded`; 128 and 512 scopes executed correctly.
Point recording scanned every dense liveness word, including inactive words, at
every cleanup event. Liveness now keeps a compact summary tree of nonzero words,
updates it on zero/nonzero transitions, and rebuilds it after dense CFG joins.
Point seeds still enumerate identities in ascending order. The single-word case
needs no summary allocation. The work limit is unchanged, and dense join work
remains charged. A retained 2,048-scope regression and boundary tests across
multiple summary levels exercise this change. The first immutable candidate and
its failed qualification remain available for comparison.

The second candidate executed repeated and distinct-const cleanup at 2,048 scopes,
but partial-field cleanup then reached the same inactive-word scanning problem in
forward loan checking. Loan state now uses the shared summary index for pruning,
access checks, storage endings and optional inspection. Dense CFG joins and
independent two-phase activation bits remain intact. Copying incoming state
rebuilds its occupied-word index; pruning and regeneration update it directly.
A state-transition test and a second 2,048-scope regression cover that change.
Both intermediate full-check runs were cancelled after their quality checks when
these scaling failures required implementation changes; they are not final
validation evidence.

Final performance qualification executes nine declared counter programs through
pinned rustc and CPPGM before timing. Thirty jobs retain 540 observations on CPU 2,
with two warmups, four A/A samples and three ABBA blocks each. Full LowIR and count
outputs match byte for byte in every comparison. No RustGM build or test process
overlaps measurement. The final snapshot contains 324 production/configuration
files; CLI SHA256 is
`d9881b1ff44a0974ee2114217faa41aafe58123f05d03dca229d923206c4072b`.

| Executable cleanup workload | 128 median, ms | 512 median, ms | 2048 median, ms | 2048 RSS, KiB |
| --- | ---: | ---: | ---: | ---: |
| repeated generic transfers | 27.46 | 103.49 | 534.51 | 39316 |
| distinct const destructors | 26.43 | 100.34 | 502.22 | 36772 |
| partially moved generic fields | 67.14 | 287.09 | 1666.03 | 78464 |

Nine direct comparisons with the first integration candidate isolate the sparse
state changes where that candidate succeeds. At 512 scopes, median wall times
change by -6.4% for repeated cleanup, -2.6% for const cleanup and -7.2% for partial
fields. Reference-instance controls range from -0.7% to +0.9%, with overlapping
ranges. The larger failed inputs are acceptance improvements under unchanged
budgets; they are not paired speedup measurements against successful old runs.

Twelve comparisons with earlier functionality range from -3.2% to +12.2% in median
wall time. These include shared-host variation and additional required proof and
demand work; this checkpoint does not claim a general speedup or zero regression.
For ordinary cleanup at 2,048 functions, the existing CLI takes 86.96 ms and the
new maintainer API takes 94.38 ms (+8.5%). That comparison crosses entrypoints;
the existing CLI still uses its earlier path. All samples, including outliers,
remain in `obj/perf/`. Inputs, source snapshots, drivers, failure receipts,
independent executions and verification scripts remain under
`obj/pa18-work/drop-integration-*`.

Final validation passes 748 frontend tests in both debug and release and all
3,350 PA1–PA17 text-oracle cases without reference changes. Fifteen maintainer
native tests and 858 course executable comparisons pass, preserving the sixteen
previously documented PA13 runtime differences. Independent rustc checks,
upstream provenance, rustfmt, Clippy all/pedantic with denied warnings, the
476-file architecture audit with zero findings and all 25 harness tests pass.
Frozen production/test sources, binaries, independent execution receipts and all
540 performance observations are reverified after the checks. Final evidence is
recorded in `drop-integration-v3-validation.json` and
`drop-integration-v3-reverified.json` under `obj/pa18-work/`.


## PA18 public execution and course release

The CLI now enters the checked template/instance pipeline before ordinary body
inspection, preserving earlier inspection contracts and macro diagnostics. All
ordinary bodies are proved even when unused. After explicit and implicit
constructor template discovery, an empty sealed template set skips the redundant
generic demand traversal; ordinary lowering already discovers ordinary uses.

The public suite declares 425 cases, independently checked with pinned rustc:
262 accepted and 163 rejected. Of the accepted programs, 253 execute cleanly,
six produce qualified panic-aborts and three intentionally divergent programs
undergo native code generation without execution. Four documented rejections
reserve PA19–PA21 features. Growing generic-demand checks require rustc object
generation instead of metadata-only emission. The native harness validates and
reports compilation-only controls separately, and its tests prohibit running them.

PA12's generic constructor and PA17's generic destructor boundary sources move
unchanged to PA18. Earlier remaining sidecar bytes are unchanged. The source-to-
feature ledger, public contract and complete validation receipts are summarized
in the release review; earlier checkpoints in this journal remain historical
records of their narrower implementation surfaces.

## PA19 shared associated declarations

The declaration model now retains trait/impl owners, their Self identities and
direct associated members in one inventory built from completed resolution.
Members preserve resolver declaration order, including appended macro expansions.
Generic parameters and items nested inside method bodies are excluded from the
enclosing owner's members. Drop header discovery consumes this index, replacing
its separate scans of the complete definition table. Receiver facts now live in
shared declaration storage used by signature checking, bodies, scopes and CFG
construction; Drop-specific conformance remains in the destructor checker.

Six new invariants, with independent rustc checks for accepted trait/impl programs,
cover direct ownership, nested items, configured/generated declarations, generated traits in distinct modules,
real versus shadowed Drop identities, receiver binding mutability, 256 generic
impl owners and rejection of unresolved input. Two initial test failures were
test mistakes: attempting to mutate a private phase flag and assuming generated
definitions preceded original crate declarations. The final tests use the actual
expansion/resolution APIs and assert resolver order.

All 756 frontend tests pass in debug and release. All 3,773 PA1–PA18 text cases
match their existing oracles; no sidecars change. Fifteen maintainer native tests,
1,117 course executable comparisons and three codegen-only comparisons pass,
preserving the sixteen documented PA13 runtime differences. Rustfmt, Clippy
all/pedantic with warnings denied, the 478-file audit with zero findings, all
27 harness tests and upstream/reference provenance checks pass.

Eight immutable comparisons against the published PA18 CLI retain 144 observations:
two warmups, four A/A samples and three paired ABBA blocks each, on CPU 2, with no
RustGM builds/tests during timing. Full LowIR and count outputs are identical.
Distinct Drop owner workloads at 128, 512 and 2,048 owners take candidate medians
of 33.65, 63.92 and 277.30 ms, versus 33.46, 63.63 and 273.44 ms for the baseline.
At 2,048 owners median peak RSS is 44,684 versus 44,544 KiB. Across all workloads,
median wall-time changes range from -0.5% to +3.7%; sample ranges overlap and A/A
spreads range from 1.7% to 5.8%. These observations do not establish zero overhead.
The other controls cover shared Drop ownership, ordinary functions, generic
records, generic cleanup and const-generic cleanup.

The frozen candidate SHA256 is
`a88a3fbef24a89eb469de68d35e12d5c016a1db93a4c1c7542e19b243afaba84`.
All 481 source/config/test hashes, binary/input hashes, validation logs and raw
observations were reverified. Receipts and scripts live under
`obj/pa19-work/inventory-*`; raw observations remain in `obj/perf/`.

This inventory records ownership, not completed general method signatures or
impl proofs. PA19 remains planned. Sixteen additional frozen header probes,
declared before independent rustc checks, establish eight accepted and eight
rejected signature rules for the next implementation: required versus default
methods with unsized Self, Sized bounds, receiver elision, explicit receivers,
unconstrained impl parameters and safe/unsafe trait impl mismatches. The full
PA19–PA21 contracts above remain required.

## PA19 inherent owner environments and method headers

Inherent impl targets now elaborate under their original type/const parameter
identities. Unconstrained type/const parameters, primitive targets and unsafe or
negative inherent impls are rejected. Methods inherit the owner environment and
capture all owner lifetime slots separately from their own early/late slots.
Callable templates validate ordered owner scopes without renaming inherited
parameters. Const occurrences in arrays and nominal arguments participate in
the target constraint check.

Shared receiver elaboration supports value, mutable binding, shared/mutable
reference, named-lifetime and explicit nested-reference forms. The receiver binds
the whole value. A direct reference to Self supplies output lifetime elision;
nested references retain ordinary input-elision rules. Explicit receiver targets
prove equality with Self using declared and implied lifetime assumptions. The
existing directed type relation engine now lives in `types/relations.rs`, shared
by declaration and body proofs. Drop keeps its stricter independent `&mut Self`
contract. Unused method bodies pass the ordinary ownership/region pipeline.

Six new integration invariants independently qualify positive and negative Rust
programs before checking these retained facts and body proofs. All 762 frontend
tests pass in debug and release; all 3,773 PA1–PA18 text cases pass. Fifteen native
maintainer tests, 1,117 course executable comparisons and three codegen-only
comparisons pass, with the sixteen existing PA13 runtime differences. Rustfmt,
Clippy all/pedantic with warnings denied, the 481-file audit with zero findings,
27 harness tests and upstream/reference checks pass.

Three boundary diagnostics changed because checking now reaches the unsupported
call or trait-receiver stage. Their inputs and rejection outcomes are unchanged.
All three were independently checked before the official PA17/PA18 reference
regeneration; every other sidecar remained byte-identical. Full independent PA17
and PA18 checks also pass, retaining their declared course-profile differences.

Fourteen immutable timing jobs retain 252 observations: two warmups, four A/A
samples and three ABBA blocks each on CPU 2. Eight compare the published inventory
binary against this candidate; six measure new method workloads with identical
candidate binaries as noise controls. All full LowIR and count outputs match.
The six new inputs were independently rustc-qualified. RustGM builds/tests did
not overlap timing; unrelated workspace jobs were active.

Existing-workload median wall changes range from -3.6% to +3.4%. At 2,048 distinct
Drop owners, the candidate takes 291.78 ms versus 282.25 ms, with all three paired
blocks 3.0–3.9% slower and a 288 KiB median RSS increase. This is a measured cost,
not evidence of zero overhead. New method-owner medians at 64/256/1,024 owners
are approximately 21/74/301 ms. A shared impl with 32 type parameters and
128/512/2,048 methods measures approximately 20/68/465 ms in the B lane. The
512-method identical-binary control differs by 26% between lanes; shared-impl
scaling is therefore not evidence of a linear bound or a speedup. These programs
check unused headers and bodies, and do not execute method calls.

The candidate SHA256 is
`3f6f34102ba128ac38a5f5c48a9452c4f104fca4fc67246be5b838059aec7a2f`.
All 484 frozen source/config/test hashes, binary/input hashes, raw observations
and validation logs were reverified. Local receipts are
`obj/pa19-work/headers-{freeze,validation,reverified}-v1.json`; all timing rows
remain under `obj/perf/`. The native log contains terminal PASS records for every
requested suite; its original shell-session status was unavailable after context
continuation and is not represented as an independently observed exit status.

PA19 remains planned. General method calls, qualified selection, trait owner
environments, required/default method handling, impl conformance and the public
semantic contract remain required before release, followed by all PA20/PA21 work.

## PA19 trait schemas and required/default method validation

Trait Self now has its own rigid parameter owner and is implicitly possibly
unsized. Explicit trait parameters retain their original declaration indices;
method environments inherit the Self, trait and method owners in order. Trait
lifetime captures, builtin supertrait bounds, where-bounds and parameter defaults
use the existing declaration machinery. Shared closed-default validation now
lives in `generics/default_validation.rs` and serves both nominal and trait
declarations. Invalid owner bounds are checked even when no method uses them.

Required method signatures allow unsized arguments/results while checking their
well-formedness and permitted parameter patterns. They do not produce CFG
templates. Default methods receive ordinary sizedness, ownership and region
proofs under their declared assumptions, even when unused. Const trait methods,
mutable required parameter bindings, invalid receivers and invalid defaults are
rejected. Seven new integration invariants include a separately declared and
rustc-qualified corpus of 38 header controls, plus identity and pending-impl
controls. They check inherited parameter ownership, unused default-body failures
and the absence of required signatures from executable templates.

User trait implementations now fail explicitly until conformance and selection
are implemented; genuine builtin Drop retains its implementation. This tightens
the earlier inventory-only acceptance of empty user impls. The shadow-Drop test
still checks distinct raw owners and genuine destructor receiver facts, and now
checks the explicit boundary for the unchecked user impl. It does not treat a
user item named Drop as the language destructor trait.

All 769 frontend tests pass in debug and release. The final test-only Clippy
rewrite also passes the 13 focused associated/header tests. All 3,773 PA1–PA18
text cases, fifteen native maintainer tests, 1,117 course executable comparisons
and three codegen-only comparisons pass. The sixteen existing PA13 runtime
differences remain documented. Rustfmt, Clippy all/pedantic with warnings denied,
the 483-file audit with zero findings, 27 harness tests and provenance checks pass.

Only PA18's associated-type and trait-method boundary diagnostics changed. Their
sources and rejection outcomes are unchanged. All 425 PA18 cases were checked
independently before official oracle regeneration; the four declared phase
differences remain explicit. Every other sidecar is byte-identical. Regeneration
was repeated after the final test-only lint rewrite to keep compiler/test source
provenance current. PA19 remains planned: these are declaration and unused-body
proofs, not executable method selection or completed trait conformance.

Twenty immutable performance jobs retain 360 observations on CPU 2: two warmups,
four A/A samples and three ABBA blocks each. Fourteen compare the published
inherent-header binary with this candidate; six measure independently qualified
new trait workloads using identical candidate binaries. Full LowIR and count
outputs match throughout. Existing-workload median wall changes range from -5.3%
to +3.2%; the generic-drop workload has three slower paired blocks, but its A/A
spread is 10.7%. The identical-binary 512-method trait control differs by 16.6%
between lanes. These noisy observations do not establish a speedup or zero cost.

A focused repeat of generic Drop, ordinary functions and the 512-method trait
control retains another 90 observations, using six ABBA blocks each with the
same frozen binaries/inputs. Generic Drop measures 496.12 versus 497.09 ms
(-0.2%); five of its six paired blocks remain 0.5–3.5% slower, with one at -0.1%.
The pooled medians therefore do not rule out a small cost. The same-binary trait
control differs by 0.4%. Both runs, including all outliers,
remain in the evidence. No RustGM builds or tests overlapped either timing run;
other workspace jobs were active.

New trait-owner medians at 64/256/1,024 owners are 13.93/45.51/183.99 ms in the
initial B lane. A shared trait with 32 type parameters and 128/512/2,048 methods
measures 14.47/56.46/183.39 ms. These are unused declaration/body proof workloads,
not method dispatch benchmarks or evidence of a linear algorithmic bound.

The frozen candidate SHA256 is
`a44c9ebf9f849ec997c56f521bbeb8078e4554791704bba1ed946ef7a1ae10e2`.
All 486 source/config/test hashes, immutable binary/input hashes, validation logs
and 450 raw observations were reverified. Local receipts include
`obj/pa19-work/trait-headers-freeze-v3.json`, `trait-headers-validation-v4.json`
and `trait-headers-reverified-v2.json`; per-sample files remain in `obj/perf/`.
Earlier failed checks and source snapshots are retained rather than overwritten.
Method selection, actual calls, user trait bounds and impl conformance are next;
the full PA19 release contract and PA20/PA21 requirements remain unchanged.

## Trait method qualifier review

Independent review found that the header implementation accepted explicit
visibility and `safe fn` on trait methods. Four predeclared rustc controls reject
these forms. Trait method validation now requires inherited visibility and
rejects the explicit `safe` qualifier. A new invariant covers those rejections,
public traits with inherited method visibility, unsafe traits/methods, and
configured-out declarations. The check consumes the existing item flags after
configuration; ordinary inherent method visibility is unchanged.

All 770 frontend tests pass in debug and release, along with all 3,773 PA1–PA18
text cases, fifteen maintainer native tests, 1,117 executable comparisons and
three codegen-only comparisons. The sixteen documented PA13 runtime differences
remain. Clippy all/pedantic, formatting, the 483-file audit, 27 harness tests and
provenance checks pass. No course sidecar changes were needed.

Three immutable comparisons against the trait-header binary retain 54 timing
observations, with A/A calibration and three paired ABBA blocks each, CPU 2,
identical full/count outputs and no overlapping RustGM builds/tests. Median wall
changes are -0.2% for 4,096 ordinary functions, -1.7% for 1,024 trait owners and
+0.5% for a shared trait with 2,048 methods. Ranges overlap; these observations do
not establish a speedup or zero overhead. All 486 source/config/test hashes,
binaries, inputs, validation logs and raw samples were reverified. The candidate
SHA256 is `8d6efd98c964b048eb48b336d4c70b535d18a130b8ad827b5dc550b16c260ea5`;
receipts are under `obj/pa19-work/trait-qualifiers-*` and `obj/perf/`.

The next selection work has 56 predeclared, independently qualified lookup and
visibility cases: 34 execute successfully with rustc and 22 reject. A separate
qualified-call probe identifies a resolver prerequisite: public trait methods
currently receive private default access. Correct inherited trait-member access
and retained anonymous trait imports must accompany method selection. The caller
must also bind owner captures and inherited generic parameters rather than using
the free-function-only argument path. These remain required PA19 work.

## Associated visibility and anonymous import prerequisites

The resolver now assigns direct trait members their trait's access. Public and
restricted traits therefore permit qualified member paths wherever the trait
itself is accessible; nested default-body items and inherent methods retain their
own access. Generated members use the same declaration path. A macro invocation
shares its parent's scope and no longer clears that scope's associated-item
metadata while being registered.

Anonymous type imports now retain their final binding by import identity.
Ordinary scope and expansion-hygiene records supply the import's provenance;
there is still no lexical name `_`. Retrying an import first withdraws its old
target, so a stronger explicit import replacing a provisional glob cannot leave
a stale trait identity behind. The record also preserves non-trait type targets;
method selection can filter by the resolved declaration kind.

Five typed invariants cover direct/generated members, nested visibility,
restricted access, reexports, repeated anonymous imports, distinct block scopes,
replacement imports and generated-import hygiene. Five new PA6 fixtures and one
PA8 fixture were independently qualified against the pinned rustc before text
reference generation. Existing course oracle bytes are unchanged.

Additional independently qualified controls distinguish impl predicates from
method predicates. An unsatisfied impl bound excludes that candidate and allows
a visible trait method; failure of a selected method's own bound does not retry
lookup. Inherent overlap checking also needs predicate reasoning: `S<T>` under
`T: Sized` can coexist with `S<str>`, and `S<T>` under `T: Copy` can coexist with a
concrete local non-Copy target. Generic Copy-bound and Clone-bound implementations
with the same method overlap. All five predeclared controls match rustc; three
accepted programs also execute successfully. Sources, commands, stderr and runtime
results remain in `obj/pa19-work/method-owner-bounds-v1`.

PA19 remains planned. These resolver facts prepare qualified and trait method
selection; they do not implement method dispatch, user trait bounds or impl
conformance. The complete PA19–PA21 responsibilities above remain required.

All 775 frontend tests pass in debug and release, together with 3,779 PA1–PA18
text cases, fifteen maintainer native tests, 1,117 executable comparisons and
three codegen-only comparisons. The sixteen documented PA13 runtime differences
remain. Clippy all/pedantic, formatting, the 484-file audit, 27 harness tests and
provenance checks pass. Independent checks cover all 210 PA6 and 94 PA8 fixtures;
PA6 retains its two documented name-versus-type-checking differences.

Ten immutable measurement jobs retain 180 observations: ordinary-function LowIR,
anonymous-import name resolution, replacement imports, qualified trait-member
paths and generated trait members. Each uses A/A calibration and three paired
ABBA blocks on CPU 2, with full/count output equality and no overlapping RustGM
builds or tests. The newly accepted qualified/generated paths use identical
candidate binaries in both lanes. A focused repeat retains another 54 observations.

The initial 4,096 ordinary-function comparison differs by -1.3%. Anonymous-import
results are noisy in the first run; the focused 4,096-import repeat has a +2.5%
median change and all three paired blocks are +2.3% to +2.7%. The 1,024-import
replacement workload changes by +4.5% initially and +1.5% on repeat, with all six
blocks positive. These measurements retain evidence of a small cost for the
additional import facts. The identical-binary qualified-path control differs by
13.0% initially and -0.6% on repeat; absolute times shift substantially between
runs, so the measurements do not support a zero-overhead or speedup claim.
Initial candidate medians at 64/256/1,024 qualified paths are 5.07/9.40/30.09 ms;
generated members measure 8.34/19.05/38.34 ms. These exercise name resolution and
expansion, not method dispatch.

The 487 frozen source/config/test files and all binaries, inputs and observations
were reverified. The candidate SHA256 is
`45d7fb588e59e9dfda91f0dd884135c9bc2a8c8f96bd495c86bd6888be7e8ec3`.
Validation, qualification, frozen sources and measurement receipts are under
`obj/pa19-work/associated-resolution-*`; raw per-sample files are under `obj/perf/`.

## Qualified inherent calls

Qualified inherent function paths now feed ordinary function inference and
executable instance demand. Supported prefixes include nominal paths, explicit
`<Type>` paths, aliases, `Self`, explicit generic arguments and inference holes.
The resolver retains the owner of an associated prefix and resolved suffix
identities; declaration elaboration builds nominal/name buckets once from the
shared inventory and typed impl targets. Method arguments are read from the
method suffix, separately from the prefix's arguments.

Call formation uses the callable's full ordered parameter list. Inherited impl
type/const arguments are inferred by equating the instantiated impl target with
the prefix. Only the method's own parameters consume written method arguments.
Owner lifetime captures receive distinct fresh body regions, including unused
captures, before own early-bound arguments. Inherited and method predicates pass
through ordinary obligation checks. Selected function values, constants and
calls reuse existing template substitution, ownership proofs and lowering.

The maintainer corpus contains 23 accepted native programs covering value,
shared and mutable receivers; function pointers; impl/method type and const
parameters; aliases and explicit type prefixes; captured and unused lifetimes;
inherited and own bounds; Self paths; and mixed inherited/own arguments.
Eight independently rejected controls cover target mismatch, private access,
failed impl/method Copy bounds, escaping borrows, explicit mutable-borrow conflicts
and incorrectly supplying impl arguments as method arguments. A typed invariant
checks that unused owner captures remain distinct nonstatic regions before
instance lifetime erasure.

This initial checkpoint supported only a unique inherent method in the
nominal/name bucket. The overlap and prefix-selection checkpoints below remove
that restriction and validate unused duplicate declarations.
Dot-call autoderef/autoref, trait candidate visibility and fallback, user trait
bounds, impl conformance and selected default bodies remain required work.
PA19 stays planned; PA20 and PA21 remain required by the active goal.

These programs exposed two earlier array defects. Semantic indexing now accepts
an inferred const array length and publishes its canonical bound after solving.
Lowering an empty array with a nonzero-sized element retains a typed address for
the unreachable continuation after its unconditional bounds guard. It must still
compile and panic, rather than fail compilation because the empty base has no
storage. Two PA12 controls cover ordinary array reads and field writes; two PA18
controls cover generic field indexing and the zero-length panic. Pinned rustc
acceptance and runtime outcomes were checked before generating their oracles.

Follow-up prefix probes also retain three open PA19 boundaries: omitted arguments
on a generic alias, an alias to an enum variant sharing a name with an inherent
function, and the corresponding explicit `<Enum>::Variant` path. Rustc accepts
these controls, while this checkpoint rejects them. The ordinary `Enum::Variant`
control remains accepted. Alias inference and variant precedence must be resolved
when completing qualified selection; explicit generic alias arguments are the
currently qualified method-call form. Evidence is under
`obj/pa19-work/qualified-calls-prefix-corners-v1`.

Validation passes all 778 frontend tests in debug and release, all 3,783 active
text cases, sixteen maintainer native tests, 1,121 executable comparisons and
three codegen-only comparisons. The sixteen existing PA13 runtime differences
remain explicitly reported. Formatting, Clippy all/pedantic, the 488-file audit,
27 harness tests, backend qualification and upstream/reference integrity checks
pass. The four new fixtures have independently reviewed expected outcomes;
no existing text-oracle bytes changed. PA12 and PA18 generation receipts refresh
the source provenance for their newly generated suites.

Fifteen immutable performance jobs retain 270 observations on CPU 2, each with
A/A calibration and three paired ABBA blocks. Full and count outputs agree in
both lanes. No RustGM builds or tests ran during measurement. Six controls compare
the preceding resolver binary with this candidate: 4,096 ordinary functions
(+0.2%), 4,096 generic records (-1.6%), 2,048 generic destructors (+0.8%), 2,048
const-generic destructors (+0.02%), 1,024 method owners (-1.7%) and 2,048 methods
sharing one impl (+1.1%). Identical-candidate controls vary by up to 2.8%; these
observations do not establish a speedup or zero overhead.

New demanded-call workloads use identical candidate binaries at 64/256/1,024
calls. Candidate medians are 7.67/19.43/66.62 ms for distinct monomorphic owners,
13.17/40.55/162.80 ms for generic methods sharing an impl, and
11.38/33.56/128.66 ms for captured-lifetime owners including an unused capture.
All benchmark inputs were independently accepted by the pinned rustc. These
measure actual LowIR emission; they are not a claim of full method selection or
an asymptotic proof from wall time.

All 491 frozen source/config/test files, binaries and inputs were reverified.
The candidate SHA256 is
`9e57d6d33db3a005c00ee05a4e38518f80a2f723a803b808d3a2a4752b95a5fe`.
Qualification, validation and measurement receipts remain under
`obj/pa19-work/qualified-calls-*`, with every raw observation under `obj/perf/`.

## Alias inference and enum variant precedence

The qualified-prefix boundaries identified above are now implemented. An alias
value path with omitted type/const arguments uses the existing partial-annotation
inference machinery; it preserves alias substitution, supplied lifetime slots
and repeated parameter identity. Omitted slots infer even when the alias has
defaults. Explicit type prefixes such as `<Alias<T>>::method` retain ordinary
type-annotation rules: required arguments cannot disappear, and omitted defaulted
arguments use their defaults. Partially written argument lists still require
all nondefaulted slots. Prepared annotation facts are reused during traversal.

Enum variants win over same-named inherent methods through aliases and explicit
type prefixes. A shared lookup consumes the enum's resolved member bindings and
access facts; publication retains the typed prefix for ordinary constructor
inference. It also handles qualified tuple-constructor function values and unit
variants in constants. Alias method names that are not variants still use the
inherent method path. Explicit qualified record construction remains rejected:
the pinned stable rustc reports that context as experimental.

The maintainer corpus now covers 32 native programs and twelve independently
rejected controls, in addition to the capture-identity invariant. Nine course
fixtures exercise earlier owning features: PA12 gets qualified tuple/unit enum
paths and a qualified-record rejection; PA18 gets alias value inference, inferred
defaulted type/const arguments, qualified generic constructors and arity controls.
Independent rustc acceptance and runtime qualification precede text generation;
receipts retain the corrected exploratory expectations about alias defaults and
explicit type-prefix arity. No method-selection ambiguity or trait functionality
is claimed by these prefix fixes; the complete remaining PA19–PA21 work stays
required.

All 778 frontend tests pass in debug and release, all 3,792 active text cases
pass, and native validation passes sixteen maintainer groups, 1,126 executable
comparisons and three codegen-only comparisons. The sixteen existing PA13 runtime
differences remain. Formatting, Clippy all/pedantic, the 488-file audit, 27 harness
tests and upstream/reference integrity checks pass. No existing oracle bytes
changed; the PA12/PA18 generation receipts include the nine new fixtures.

Ten immutable performance jobs retain 180 observations, with A/A calibration,
three paired ABBA blocks, CPU 2 and full/count output equality. No RustGM builds
or tests overlap timing. The four baseline comparisons change by -1.1% for
4,096 ordinary functions, +1.4% for 4,096 generic records, +0.9% for 1,024 generic
method calls and +1.0% for 1,024 captured-lifetime method calls. New alias-method
and alias-variant workloads use identical candidate binaries at 64/256/1,024
calls: candidate medians are 7.12/15.68/50.59 ms and 8.53/21.62/73.52 ms.

One initial generic-method paired block is +36.1%; its two slower candidate
samples also consume more CPU time, so it cannot simply be attributed to time
waiting for the scheduler. All observations remain retained. A focused repeat
adds 54 samples: generic records change by -0.9%, generic methods by +0.8% and
the identical-binary alias-method control by -0.9%. Repeat block changes for
generic methods are -1.2%, +1.3% and approximately zero; the large first-block
cost does not recur. This evidence is noisy and does not establish a speedup,
zero overhead or a consistent regression.

All 491 frozen source/config/test files and every binary/input hash were
reverified. The candidate SHA256 is
`4593a43159e488a818632cceaa7cb6ef91ea5975ce5a33af199123d94f70c217`.
The 234 raw observations, independent qualification and validation receipts live
under `obj/pa19-work/qualified-prefix-*` and `obj/perf/`.

## Unused inherent overlap validation

Inherent method declarations now require disjoint implementation headers even
when no call demands them. One isolated inference graph instantiates both owner
schemas and their impl predicates; repeated type/const parameters retain their
identity. Structural mismatch and occurs failure establish disjointness. Remaining
existentials retain representative parameter identities under a possibility-only
environment, rather than being replaced with an arbitrary concrete witness.
Method predicates, receiver shape, access and lifetimes do not resolve duplicates.

Copy/Clone coherence uses a separate query mode. Local non-derived nominals can
establish a negative result; validated conditional derives constrain their type
arguments. Foreign tuples, arrays and other type heads cannot be ruled out merely
because the present body query lacks an implementation. Reference locality follows
its referent. Sized requirements still exclude unsized subjects. This possibility
mode never supplies move or body permissions.

Nominal/name buckets retain declaration order. Closed targets use canonical shape
keys that erase lifetimes and callable binder counts only for indexing. Distinct
closed shapes skip pair probes; generic pairs share results across method names.
The work is bounded, with retained inference-work counters. A 1,024-impl const
case performs zero probes; 128 shared method names reuse one owner-pair proof.

The maintainer suite independently qualifies 59 declaration cases against pinned
rustc, covering repeated variables, occurs checks, concrete consts, impl versus
method bounds, local/foreign types, derives, visibility, receivers and binders.
Two exploratory predictions about mutable-reference/tuple Copy bounds were wrong;
the original receipts remain in `obj/pa19-work/inherent-overlap-rules-v1` and the
compatible-world matrix records the investigation. The final tests declare the
corrected expectations before running both implementations. Five additional
higher-ranked function-pointer probes also agree. No course fixture or oracle
bytes changed, and PA19 remains planned. Qualified selection follows below;
dot lookup/adjustments, trait dispatch/conformance, and PA20–PA21 remain required.

Validation passes: 781 frontend tests in both debug and release; 16 explicit
maintainer native groups; 3,792 PA1–PA18 text cases; 1,126 executable comparisons
and three codegen-only comparisons. The sixteen existing PA13 runtime differences
remain explicitly reported. Formatting, Clippy all/pedantic, the 491-file audit,
27 harness tests, backend qualification and upstream/Reference integrity pass.

Eight frozen A/A and three-block ABBA jobs retain 144 observations with full/count
output equality. Ordinary-function, generic-record and qualified-generic-method
median changes are +0.8%, -0.3% and +0.7%. Closed-impl workloads at 64/256/1,024
change by +2.1%, -2.1% and +0.7%; shared-owner workloads at 64/1,024 by +0.3% and
+1.0%. Individual blocks and A/A samples remain noisy; these results do not show
a substantial regression or establish zero cost. Candidate medians at 1,024 closed
impls and 1,024 shared names are 28.90 ms and 35.72 ms respectively. The structural
work assertions provide separate evidence for skipped pairs and shared proofs.

No RustGM builds or tests overlapped timing. All 494 frozen source/config/test
files, binaries and inputs were reverified. Candidate SHA256:
`e10a945ba53bc72b1916911158feb428e0c1336b6660164f4568d9c25d025d91`.
Qualification, validation and measurement receipts are under
`obj/pa19-work/inherent-overlap-*`, with per-sample records under `obj/perf/`.

## Qualified selection across disjoint inherent implementations

Qualified paths now choose among multiple valid inherent implementations using
only the prefix's type and const information. Explicit types, partial types,
aliases, defaults, Self, function-pointer shapes and repeated type/const holes
share ordinary annotation inference. Caller parameters retain their identities
and declared assumptions. Result types, call arguments and method turbofish do
not resolve an otherwise ambiguous prefix. The selected method feeds the existing
callable, instance, ownership and native-lowering path.

The annotation worklist and generic-application/default logic are now shared by
body checking and isolated prefix queries. Each candidate imports only the prefix's
reachable inference graph, preserving type-variable domains and const equivalence
classes. Failed probes cannot affect later candidates or the body. After header
equations, a bounded built-in property query handles unresolved subtrees and
canonical composite assumptions without inventing concrete witness types or
synthetic declaration identities. Actual caller bounds govern rigid parameters;
current Copy/Clone/Sized implementations govern candidate applicability. The
compatible-world coherence mode remains separate. Method-own bounds are checked
after selection; lifetime/ownership legality still requires the ordinary body proof.

Visibility filters candidates before uniqueness. An initial prediction that an
inaccessible private method would make an unknown qualified prefix ambiguous was
incorrect. The pinned-rustc investigation is preserved in
`obj/pa19-work/method-selection-visibility-v1`; the original and corrected expected
case tables are both retained. A public method can be selected from outside its
module while a same-named private method is ignored. Inside the owning module,
both candidates remain visible and the unresolved prefix is ambiguous.

The overlap checker and selector now share one candidate partition. Concrete
prefixes use its closed-shape index; generic and partial prefixes use bounded
isolated probes. The shape folder survives declaration checking and memoizes later
prefix normalization. A 256-implementation/256-call invariant requires exactly
256 probes. Another invariant performs 128 repeated calls through a wide alias
without any additional shape-folding steps. Empty predicate sets avoid allocating
a property-query environment.

The maintainer suite declares and independently checks 46 language cases: 31
accepted native programs and 15 rejections. Cases cover disjoint type/const impls,
partial and nested holes, failed-first probe isolation, repeated variable identity,
conditional derives, unsized subjects, caller/composite bounds, environment
isolation, mutable receivers, captured lifetimes, own generic arguments, aliases,
function values and visibility. Both structural work invariants are also qualified
with rustc. No course fixture or oracle bytes changed.

PA19 remains planned. Dot-call autoderef/autoref, trait visibility and bound lookup,
impl conformance/default dispatch, the semantic CLI and the assignment's released
contract remain required. PA20 and PA21 retain their full responsibilities above.

The final candidate's twelve A/A and three-block ABBA jobs retain 216 observations
with full/count output equality. Unchanged ordinary-function, generic-record,
qualified-generic-method, alias-method and unused-closed-impl median changes versus
the published overlap checkpoint are +2.2%, +1.7%, +0.7%, +0.3% and +0.6%.
Ordinary-function observations range from roughly 504 to 954 ms across lanes;
the host is noisy, and these measurements do not establish zero overhead.

New concrete selection at 64/256/1,024 impls and calls takes 7.27/17.24/59.34 ms;
partial-bound selection at 64/1,024 takes 8.70/74.79 ms. These new-feature jobs
compare identical candidate binaries as controls, not the unsupported old frontend.
Repeated wide-alias selection versus the frozen uncached implementation improves
from 10.26 to 5.87 ms at 64 calls and 83.97 to 16.35 ms at 1,024 calls. The latter
is an 80.5% median reduction. Peak-RSS observations at 1,024 calls are about 7.2 MiB
in both final lanes; earlier observations showed a small increase, so this does
not establish a memory reduction.

The first candidate's 216 observations and 54 focused repeat observations are
also retained. Its repeatable roughly 2% unused-impl cost motivated consolidating
the overlap and selection partitions. No RustGM builds/tests overlapped timing.
All binaries, frozen inputs and 502 source/config/test files were reverified;
read-only source archives preserve both candidates and the uncached implementation.
Final candidate SHA256:
`5e73fba3850ad1f518d40f1e55dc8f158d101b001cf1b9b2adecbf8970d80966`.
Receipts live under `obj/pa19-work/method-selection-*`, with individual timing
samples under `obj/perf/`.

Final validation passes: 785 frontend tests in both debug and release, all 17
explicit maintainer native groups (including the 31 new accepted programs), and
3,792 PA1–PA18 text cases. Formatting, Clippy all/pedantic, the 499-file audit,
27 harness tests, backend qualification and upstream/Reference integrity pass.
The course-native run before the final candidate-table consolidation also passed
1,126 executable comparisons and three codegen-only comparisons, retaining the
sixteen declared PA13 runtime differences. The final release executable matches
the frozen candidate's SHA256. Validation receipts record exact log hashes and
source verification under `obj/pa19-work/method-selection-validation-v2.json`.

## Inherent receiver calls

Inherent dot calls now use receiver-only selection followed by the ordinary
callable, instance and ownership path. Lookup visits each built-in dereference
level in value/shared/mutable order, checks visibility and impl applicability,
and defers method-own requirements until after selection. Call arguments, result
expectations and the method's turbofish cannot choose an otherwise ambiguous
receiver. The selected method may still infer its arguments through the call.

Written method types, holes and constant dependencies are prepared before lookup;
kind/arity checks and constant evaluation use the selected declaration afterward.
Declaration and body paths share argument grouping, arity, capture checks and
scalar constant evaluation. Existing callable templates carry inherited type,
const and lifetime arguments. Hidden callable values receive the same nominal and
well-formedness checks as source callee expressions.

Body construction evaluates the receiver place once before explicit arguments.
It applies the retained dereferences and implicit borrow/reborrow, then passes that
value as the first ordinary call operand. Mutable implicit borrows use existing
two-phase loans. Explicit source borrows retain their own immediate events.
Returned references, packed places, moves, Drop and cleanup use the existing proof
pipeline. No source rewrite or second parser implements dot calls.

The maintainer corpus declares 56 language cases: 32 accepted native programs
and 24 rejections. It includes the exact former PA17/PA18 inherent-method boundary
sources, which are no longer valid rejection fixtures. Their obsolete sidecars
were removed; regenerated PA17/PA18 receipts leave every remaining oracle byte
unchanged. Those active suites now contain 205 and 433 cases respectively. The
graduated sources will join the PA19 manifest with its complete grading contract.

One initially predicted positive uses a general constant block with statements.
Pinned rustc accepts it, but the existing course constant-expression subset leaves
statement evaluation to PA25. The original declaration is retained in
`obj/pa19-work/dot-methods-rules-v1`; a dedicated test preserves the explicit
acceptance boundary, while the scalar-expression block is a positive control.
This is separate from the 56 matching language outcomes.

Two structural tests require exactly 256 probes for 256 closed impls/calls and
one small probe for a 96-reference receiver. Filtering receiver depth before graph
import avoids repeatedly copying inapplicable reference chains. The resolver's
three site invariants cover raw/NFC method names, macro expansion, scoped imports
and unresolved operands independently of semantic selection.

PA19 remains planned: visible trait and caller-bound lookup, impl conformance,
default dispatch, qualified trait calls, the semantic CLI and the complete course
contract remain required. Built-in reference adjustment does not claim user Deref
hooks, unsized receiver coercions or general const-function execution. PA20 and
PA21 retain their full responsibilities above.

Validation passes 793 frontend tests in both debug and release, all 18 maintainer
native groups, 3,790 PA1–PA18 text cases and the complete independent rustc sweep.
Course-native validation passes 1,126 executable comparisons and three codegen-only
comparisons, retaining the sixteen declared PA13 runtime differences. Formatting,
Clippy all/pedantic, the 507-file audit with zero findings, 27 harness tests,
upstream/Reference integrity and backend qualification pass.

Twelve frozen A/A and three-block ABBA jobs retain 216 observations with full/count
output equality. Relative to the published qualified-selection binary, median
changes for ordinary functions, generic records, qualified generic methods,
alias methods and unused closed impls are +0.3%, -0.3%, +0.1%, +2.4% and -0.1%.
The alias workload has a small measured cost; identical-binary controls also vary
by roughly 0–2%, so these observations do not establish zero overhead. Median RSS
increases by about 442 KiB for the unused-impl workload and 190 KiB for qualified
generic methods; other unchanged workload medians differ by about 12 KiB or less.

New closed receiver selection at 64/256/1,024 impls and calls takes
8.46/21.69/76.96 ms; repeated generic receiver calls at 64/1,024 take
6.78/45.75 ms. Reference depths 64/96 take 4.74/5.70 ms. These jobs compare
identical candidate binaries as controls. An initial 256-reference input exceeded
the existing parser nesting limit of 128, before timing. Its source and failure
remain retained; the qualified replacement stays within the parser contract.

No local builds or tests overlapped timing. All 510 frozen source/config/test
files, binaries and inputs were reverified. Candidate SHA256:
`a14fe68dabb17cf81d6799bc00649010447f830f0610c51f5c002404b602cfb4`.
Evidence lives under `obj/pa19-work/dot-methods-*`, including
`dot-methods-validation-v1.json` and `dot-methods-perf-results-v2.json`; individual
measurements retain every sample under `obj/perf/`.

## User trait contracts and bound environments

Canonical trait applications now retain the resolved declaration and every explicit
lifetime, type and const argument. Self remains the predicate subject. Complete
predicates share ordinary argument/default elaboration, capture-safe substitution,
parameter-use traversal and inference equivalence classes. Closed-subject bounds
with generic arguments remain generic; arguments occurring only in obligations
cannot silently disappear at inference completion.

Trait headers precede dependent bounds. Immutable environments contain the written
trait assumptions and transitive Self requirements, including builtin abilities
and outlives bounds. Requirements on explicit trait parameters or composite types
must be proved separately. A user trait named Copy or Clone grants no language
ability. Definition-identity cycle checks precede expansion, and a deduplicated
worklist handles diamond inheritance without exponential traversal. Application
contracts and their substitutions are shared across declaration proof consumers.

Declaration checking validates trait argument shapes, required predicates and
lifetimes even for unused items. Lifetime proof uses the completed nominal
contracts and existing directed relation engine, with explicit and signature-implied
assumptions separate from application obligations. Closed defaults include const
parameters; a const occurring only in a trait argument participates in the check.

Six typed representation/inference tests cover identities, capture-safe folding,
foreign parameters, constant-only obligations, compact predicate layout and shared
wide-argument work. The source corpus independently checks 61 outcomes against
pinned rustc and runs the complete instance proof for accepted unused generic
bodies. An additional 24-level diamond retains exactly 73 distinct trait assumptions.
The source tests include exact function-item obligations and nominal bounds,
forward supertraits, defaults involving Self, argument requirements, lifetime-only
trait parameters and negative move/identity controls.

This is a PA19 implementation checkpoint, not its grading release. General trait
impl proofs, inference driven by trait selection, lifetime-sensitive goal matching,
trait-aware inherent applicability/coherence, user-trait bounded Drop headers,
visible and caller-bound method
lookup, conformance and concrete dispatch remain outstanding. Closed user-trait
obligations without implementation evidence still reject. PA19 remains planned;
PA20 and PA21 remain required.

Final validation passes 805 frontend tests in both debug and release and all 3,790
PA1–PA18 text cases, with no oracle changes. Clippy all/pedantic, formatting, the
516-file audit, 27 harness tests and upstream/Reference integrity pass. Before the
final two-line trait-header elision correction, the complete independent rustc sweep,
18 maintainer native groups, 1,126 course executable comparisons and three codegen
checks passed, retaining the sixteen declared PA13 runtime differences. Final full
frontend/course runs and all 61 independent source cases include the correction.

Ten frozen A/A and three-block ABBA jobs retain 180 observations with full/count
output equality. Against the published inherent-dot-call compiler, median wall-time
changes for ordinary functions, generic records, qualified generic methods, unused
closed impls and generic receiver calls are -1.0%, -1.1%, +0.5%, +0.2% and -1.5%.
The largest RSS increase is about 710 KiB for qualified generic methods; ordinary
functions and unused impls increase by 356 and 294 KiB. These measurements do not
establish zero overhead or a speedup: identical-binary controls vary by up to 3.1%.

New trait-bound bodies at 64/256/1,024 functions take 9.08/22.17/76.72 ms; diamond
supertraits at 8/24 levels take 4.90/10.08 ms. New-feature jobs compare identical
candidate binaries, since the previous frontend rejects their source. No local
RustGM builds or tests overlapped timing. All 519 source/config/test files, binaries
and inputs were reverified; read-only archives and every observation remain in
`obj/pa19-work/trait-contracts-*` and `obj/perf/`. The final candidate SHA256 is
`700e6bd8ce8d8678905845410db7b8292f8584ce4ada890c021a55e2465387a1`.
`trait-contracts-validation-v2.json` records log hashes and exact validation scope.

## Caller-bound trait inference

Calls now use caller trait assumptions to infer type and const arguments that
appear only in bounds. Complete goals share the ordinary inference graph. A
pending-goal worklist revisits ambiguity after another goal constrains a shared
argument, independently of goal order. A wholly unknown Self is not guessed from
a caller assumption; a known outer type constructor may constrain its children.

Assumptions are indexed by trait identity and the rigid outer subject shape, with
exact duplicates removed. Isolated probes preserve shared variables across Self,
type arguments and const arguments, including array lengths. Only a unique match
commits equations. The selected assumption remains explicit evidence, and its
lifetime equalities must pass the ordinary region proof. Declaration checking also
rejects lifetime-compatible duplicate bounds on unused items. A declaration
containing only its own exact candidate skips inference construction.

The new source suite checks 36 outcomes against pinned rustc and proves complete
instances for accepted unused generic bodies. Three typed tests cover failed-probe
isolation, cross-root aliases, region obligations and 4,096 repeated arguments.
Two internal tests check unknown Self and 4,096 unrelated rigid subjects; the
latter stays below 150,000 inference steps. Tests live with the existing trait
application and source-bound suites. The source suite also checks
partial tuple, array, reference and nominal subjects, conflicting repeated roots
and ambiguous partial subjects.

This also reaches ordinary qualified calls and method-own bounds through their
existing callable obligations. General trait impls and conformance, impl-bound dot
applicability, canonical nominal proof across equivalent lifetimes, trait-aware
coherence, bounded Drop headers, trait method dispatch and the PA19 course release
remain outstanding. PA20 and PA21 remain required.

Validation passes 811 frontend tests in both debug and release, all 3,790
PA1–PA18 text cases, the full independent rustc sweep, 18 maintainer native groups,
1,126 course executable comparisons and three codegen-only controls. The sixteen
previously declared PA13 runtime differences remain explicit. Clippy all/pedantic,
formatting, the 522-file audit, 27 harness tests and pinned upstream/Reference
integrity pass. No course fixtures or oracle bytes changed.

Twelve frozen A/A and three-block ABBA jobs retain 216 final observations with
full/count output equality. The initial implementation added about 33% wall time
to the 24-level diamond workload. Skipping inference for a declaration's exact,
unique candidate reduces that overhead to 9.6% against the published compiler.
A direct frozen before/after comparison measures an 18.9% improvement for that
change, with all three paired blocks improving. Existing trait-bound bodies add
7.6% for the new declaration ambiguity and call-evidence checks. Median RSS changes
for these two workloads are -46 and -20 KiB, respectively.

An unstable ordinary-function run was repeated: the repeat changes median wall
time by +0.3%, with paired blocks between -0.2% and +1.8%. Qualified generic method
timing remains inconclusive: two runs have large spreads and opposite median
deltas (-21.5% and +14.7%). These are not speedup or regression claims. Generic
records, unused closed impls and receiver calls have median changes of -1.5%, +2.8%
and -2.8%, with calibration noise of comparable magnitude. All observations remain
retained, including the noisy runs; the three repeat/comparison jobs add 54 samples.

New inference bodies at 64/256/1,024 functions take 8.30/20.69/76.76 ms. Single-body
rigid-subject workloads at 64/128 parameters take 19.53/56.74 ms. A 256-parameter
source exceeded the existing runtime-lifetime storage budget before timing; its
failure is retained. The separate typed index test covers 4,096 subjects without
that downstream work. New-feature scaling jobs compare identical candidate bytes.

No RustGM builds or tests overlapped timing. All 525 source/config/test files,
binaries and inputs were reverified. Final compiler SHA256:
`85439989c23340edd2f7f8acfb57f03a0ae435c7d7b2d890666d3c9219728cbb`.
`obj/pa19-work/trait-inference-validation-v3.json` records the final validation
and log hashes; source snapshots, all timing receipts and earlier measurements
remain under `obj/pa19-work/trait-inference-*` and `obj/perf/`.

## Impl-bound method applicability

Inherent method selection now uses complete caller trait assumptions when checking
an impl's requirements. Prefix and receiver probes share one substitution across
the impl target and every type/const trait argument. Missing evidence for a rigid
goal excludes that candidate; unresolved evidence may remain possible until call
arguments or the result constrain the selected ordinary callable. The final body
proof still requires actual evidence and all lifetime equalities. Method-own bounds
do not filter candidates before selection.

The assumption matcher distinguishes absence, deferred inference and proved
matching evidence. Body and qualified-prefix lookups cache caller indexes per
scope; body universal aliases are normalized before indexing. Trait goals resolve
before builtin property queries, so an inferred non-Copy argument can exclude a
candidate without leaving a stale property result. Failed candidates retain no
caller constraints. Structurally disjoint inherent headers no longer fail merely
because they contain a trait predicate; potentially overlapping user-trait headers
still require the pending coherence implementation.

Receiver testing exposed a separate pre-existing lowering failure: a generic
receiver read or lifetime conversion can produce a runtime value without an
expression node having that exact type. Such values had no prepared fields.
The hidden callable and both receiver types now use the existing body value
validation/preparation path. The published compiler independently rejects two valid
controls that exercise this gap; the new implementation accepts them, while moving
an owned generic value through a shared reference still rejects.

Thirty-five new source cases check outcomes against pinned rustc and run full
instance proof for accepted unused generic bodies. They cover ambiguous/missing
bounds, selection before method-own requirements, failed-first-candidate isolation,
shared-goal inference, deferred call arguments/results, const arguments and lifetime
requirements. Two additional unchanged-source executable controls join the existing
receiver suite: generic lifetime conversion and copying a receiver through a borrow.

General trait implementations and conformance, trait method lookup/dispatch,
canonical nominal proof across equivalent lifetimes, bounded Drop headers and the
PA19 course release remain outstanding. PA20 and PA21 remain required.


Validation passes 812 frontend tests in both debug and release, all 3,790
PA1–PA18 text cases, the complete independent rustc sweep, 18 maintainer native
groups, 1,126 course executable comparisons and three codegen checks. The sixteen
declared PA13 runtime differences remain explicit. Clippy all/pedantic, formatting,
the 523-file audit, 27 harness tests and upstream/Reference integrity pass. Course
fixtures and oracle bytes are unchanged.

Fifteen frozen A/A and three-block ABBA jobs retain 270 observations; three targeted
repeats add 54. Full/count outputs and all source, binary and input hashes match.
The first ordinary-function and trait-bound runs were noisy; their repeats change
median wall time by +0.2% and +0.3%. Generic receiver calls add 3.4% in the repeat,
with all paired blocks between +2.7% and +4.4%, for the extra receiver value checks.
Their median RSS changes by -6 KiB. Generic records, unused closed impls and trait
inference change by -0.6%, -1.2% and less than +0.1%. Qualified generic methods
change by +1.2%, but their calibration and paired samples run at substantially
different absolute speeds, so that result does not establish a stable effect.
Every noisy observation is retained; no speedup is claimed.

New bounded-receiver workloads at 64/256/1,024 bodies take approximately
10.7/28.7/106.8 ms. Qualified bound selection at 64/256 bodies takes 9.7/25.2 ms.
One caller environment with 64/256/1,024 distinct bounds and sixteen repeated
method calls takes 6.0/9.3/21.4 ms. These new-feature jobs compare identical candidate
bytes; absolute timings are measurements from this run, not throughput guarantees.
No local RustGM builds or tests overlapped timing. All 526 source/config/test files
and immutable binaries remain archived under `obj/pa19-work/candidate-bounds-*`;
raw samples remain in `obj/perf/`. The validation receipt is
`obj/pa19-work/candidate-bounds-validation-v1.json`. Final compiler SHA256:
`e389a9d9a0704e2acbf0e56cabd52008673f5bb4cd71d74e1e9da16cfe62636f`.


## Canonical trait proofs across equivalent lifetimes

Nominal type uses and trait argument requirements now share the complete caller
assumption matcher with inferred calls. Caller type and const parameters stay
rigid. The ordinary exact-assumption lookup remains the first path; nonexact
matches build a reusable assumption index and bounded inference context lazily.
Matching a type shape retains lifetime equations and never grants equality.

Declaration proofs attach these equations to the owning signature, aggregate or
associated owner. Validation waits for completed nominal lifetime contracts and
uses the existing directed region engine with explicit and signature-implied
permissions. Both directions must hold. Body proofs normalize universal aliases
and append equations to the existing body region facts. Caches reset between
bodies, and earlier successful obligations cannot grant later obligations new
permissions.

Twenty-eight source controls independently match pinned rustc and run complete
instance proof for accepted unused generic bodies. They cover signature and field
requirements, body annotations, borrowed bound subjects, trait argument contracts,
impl/trait lifetime captures, implied lifetime bounds, type/const argument identity,
and both cross-body and within-body proof isolation. Annotation-only controls have
no callee whose obligations could substitute for the nominal check. The previously published
compiler rejects ten valid cases in the first two declared control sets; the
remaining cases reject missing or stronger requirements. These tests do not claim
concrete user-trait dispatch, which remains a separate implementation dependency.

Bounded user-trait Drop headers, general trait impl conformance and dispatch,
method lookup through traits, and the PA19 course release remain outstanding.
PA20 and PA21 remain required.


Validation passes 814 frontend tests in both debug and release and all 3,790
PA1–PA18 text cases. The final 28 source controls were also rerun together in both
profiles after adding the annotation-only cases. The full independent rustc sweep,
18 maintainer native groups, 1,126 course executable comparisons, three codegen
checks, and upstream/Reference integrity pass. The sixteen declared PA13 runtime
differences remain explicit. Formatting, Clippy all/pedantic, the 524-file audit
and 27 harness tests pass. No course fixture or oracle bytes changed.


Sixteen frozen A/A and three-block ABBA jobs retain 288 observations. Three
repeats add 54, and a twelve-block ordinary-function comparison adds another 54.
All 396 observations, full/count output equality and source/binary/input hashes
are retained. Ordinary-function timing remains inconclusive: the three median
changes are -2.3%, +5.8% and -0.2%, while the longer run's paired blocks range from
-28.2% to +38.6% and both binaries have samples near twice their median. That is
not evidence of a speedup or zero overhead.

The trait-bound repeat changes median wall time by +1.5%, with paired blocks
between -0.6% and +2.8%. Other existing workloads change by between -2.8% and +0.4%
in the initial run; these differences are generally comparable with calibration
noise. The cached-caller repeat changes by -1.4%. Existing-workload median RSS
changes across the runs stay between -20 and +74 KiB. These measurements do not
establish a universal compile-time bound.

New equivalent-lifetime signatures at 64/256/1,024 functions take approximately
10.1/24.8/88.2 ms; corresponding body-annotation workloads take 16.4/47.5/170.1 ms.
One caller with 64/256/1,024 trait bounds and sixteen nominal uses takes
6.3/9.0/21.5 ms. New-feature measurements compare identical candidate binaries,
since the previous frontend rejects these sources. No local RustGM build or test
overlapped timing. All 527 source/config/test files are archived and verified;
final compiler SHA256 is
`7bb158d5244904cb6fb30836424f46b99ff097c66b9fc182f512fd5598b02575`.
`obj/pa19-work/nominal-trait-validation-v1.json` records final validation and log
hashes; source snapshots and measurements remain under `obj/pa19-work/nominal-trait-*`
and `obj/perf/`.


## Trait bounds on Drop headers

Trait contracts now precede Drop header elaboration. Destructor inventory still
precedes nominal completion, and the unsupported general-trait-impl gate runs after
Drop owners are ready. Recognizing a trait declaration does not grant arbitrary
impl conformance or destructor identity from its spelling.

Drop coverage substitutes the complete trait predicate into the nominal parameter
context, including type, const and lifetime arguments. Type compatibility retains
lifetime equations. These equations use the existing destructor coverage graph:
the target lifetimes remain universal, and extra impl lifetimes remain existential.
An inverse slot map translates nominal coverage order back to impl order once.
Both directed requirements are added, including equality with static. Keeping
these obligations under the nominal declaration alone would incorrectly reject
valid extra impl lifetime slots, which do not exist in that declaration.

Twenty-five independent source controls cover complete argument remapping,
stronger/missing requirements, trait aliases, supertrait permissions in destructor
bodies, rejected field moves, and extra lifetime slots. Three-lifetime permutation
controls check the inverse map; paths through extra slots may not strengthen the
nominal contract. Initial extra-slot controls caught an invalid-identity rejection
before this integration. Required generic bodies undergo complete instance proof.
These cases do not claim concrete user-trait execution before general impl dispatch.
Existing general-trait gates and destructor identity/conformance suites also pass.

General trait impl conformance, selection and dispatch, trait method lookup, and
the PA19 course release remain outstanding. PA20 and PA21 remain required.


Validation passes 817 frontend tests in debug and release, all 3,790 PA1–PA18
text cases, the complete independent rustc sweep, 18 maintainer native groups,
1,126 course executable comparisons and three codegen checks. The sixteen declared
PA13 runtime differences remain explicit. Clippy all/pedantic, formatting, the
525-file audit, 27 harness tests and upstream/Reference integrity pass. Course
fixtures and oracle bytes remain unchanged.

A follow-up trait probe exposed a separate false acceptance: a function-pointer
trait argument returning its first bound lifetime can incorrectly satisfy one
returning its second, when unrelated outer caller lifetimes are equal. Typed facts
retain the distinct signatures, but an inner lifetime equation reaches the outer
body proof without its binder scope. The reproducer and independent rejection are
retained in `obj/pa19-work/trait-nested-binder-v1`. Fixing this is required before
general trait dispatch and the PA19 release; the current checkpoint does not fix it.


Fifteen frozen A/A and three-block ABBA jobs retain 270 observations; three repeats
add 54. Full/count outputs and all source, binary and input hashes agree. Existing
builtin-Drop declarations change median wall time by +0.2%, with paired blocks
between -0.6% and +0.3%. Trait-bound and inference repeats change by -0.6% and -0.1%;
these are within the observed noise and are not speedup claims. Median RSS changes
for existing workloads remain between -30 and +30 KiB.

The initial run has substantial timing instability: even identical candidate
binaries differ by +50.8% in the largest new trait-Drop workload. Its repeat differs
by +1.6%. Both runs are retained. Ordinary functions, generic records and receivers
change by +1.2%, +2.7% and -2.5% initially, with unstable blocks/calibration;
these observations do not establish stable regressions or improvements.

New trait-Drop declarations at 64/256/1,024 items take approximately
12.0/37.4/131.0 ms, using the repeat for the last value. Extra-lifetime Drop headers
take 11.1/56.9/208.4 ms. These are same-binary scaling observations on this host,
not throughput guarantees. No local RustGM build, test or diagnostic probe overlapped
timing. All 528 source/config/test files remain archived and verified. Final compiler
SHA256 is `36111e0653f3cb859682b3f4ac25cbdac94581796c101d057a4c20b7e4be1ea4`.
The validation receipt is `obj/pa19-work/drop-trait-validation-v1.json`; all source
snapshots and measurements remain under `obj/pa19-work/drop-trait-*` and `obj/perf/`.


## Scoped trait equality for function-pointer arguments

The callable-lifetime false acceptance recorded at the preceding checkpoint is
fixed. A local function binder's return/input correlation no longer becomes an
ordinary caller lifetime equality. Twenty-five source controls independently agree
with pinned rustc, and four typed invariants inspect the actual equations, inference
results, alpha mappings and bounded shared work. No course fixtures or reference
sidecars change at this checkpoint.

Trait type arguments require rigid alpha equality. Renaming or reordering local
binder declarations and adding unused declarations preserve that equality; merging
two independently used slots does not. Independent probes rejected the initial
hypothesis that bidirectional function subtyping would suffice. The final matcher
uses a per-callable bijection of used slots, keeps nested frames distinct, and counts
zero-slot callable binders when normalizing captured outer lifetimes. Only those
outer equalities reach the ordinary region proof. Identical child type identities
inside a nonidentity alpha mapping do not bypass correlation checks.

Type and const structure still use the existing inference equations. A variable
inside a callable cannot capture that callable's local lifetime; a replacement with
its own closed function binder is valid. A shared region-constructor mapper rebases
inferred types without parsing or canonical-store mutation. Repeated variable uses
cache rebasing, and all traversal/probe work remains charged. The existing inference
API's caller-universal convention remains free region identities; declaration-only
canonical queries retain rigid parameters. One propagated dependency bit selects
the callable path without increasing the compact type-property storage size.

The source controls cover the original call and nominal-annotation failures, outer
equality versus one-way outlives, local versus static/captured regions, nested and
nominal lifetime arguments, distinct assumptions, inferred and preconstrained type
arguments, and capture rejection. Generic user-trait bodies remain checked templates;
these controls do not claim concrete general-trait execution before impl dispatch.

A separate exploratory probe found a pre-existing pinned-rustc acceptance difference:
redundant, differently spelled but canonically identical function-pointer trait
bounds can produce rustc E0283, while both the published baseline and this candidate
deduplicate them under the current canonical-predicate contract. Three initial
acceptance predictions were wrong for rustc; predictions, actual diagnostics and
both compiler outcomes remain in `obj/pa19-work/binder-duplicates-v1`. They are not
counted as matching qualification cases. PA19 release review must resolve or document
that diagnostic-boundary difference; this scoped-equality fix does not change it.

General trait impl conformance, selection and dispatch, trait method lookup, and
the PA19 release remain outstanding. PA20 and PA21 remain part of the active goal.

The first implementation passes 825 frontend tests in debug and release, the full
3,790-case course report, the independent rustc sweep, all 18 maintainer native
groups, 1,126 course executable comparisons and three codegen controls. The sixteen
PA13 runtime differences remain explicit. A final allocation shortcut uses canonical
escaping-binder metadata before creating rebasing storage or traversing an identical
closed subtree. It passes the full 825-test debug suite and course report again,
all 30 affected release tests, Clippy all/pedantic, formatting, the 530-file audit,
27 harness tests and upstream/Reference integrity. Final course LowIR remains
byte-identical to the same oracles qualified by the native run; neither lowering
nor the backend bridge changed.

The original frozen implementation retains 270 timing observations. The final
candidate adds 416 observations across sixteen A/A and five-block ABBA jobs,
including a direct comparison against that first implementation. All frozen source,
binary and input hashes and full/count output equality pass. No local compiler
build, test or diagnostic probe overlapped either timing run.

The initial callable-inference workload showed roughly 5–8% median overhead. After
the allocation shortcut, the 64/256/1,024-item versions measure 8.99/23.63/89.33 ms,
about +3.5/+2.6/+2.4% against the published baseline. The direct 1,024-item comparison
against the first candidate changes median wall time by -3.7%; all five paired
blocks are faster (-3.1% to -7.8%). These measurements support removing the avoidable
allocation, not a claim of zero cost for scoped matching.

Final ordinary functions and generic records differ by +0.3%/+0.4%, and ordinary
trait bounds are effectively unchanged; their calibration/block noise is larger
than those median changes. Other existing workloads have mixed blocks, including
18% calibration spread for trait inference, so their medians are not speedup claims.
Final median RSS differences are within -6 to +50 KiB. The final callable-alpha and
outer-equality workloads at 1,024 items take 65.57 and 86.47 ms on this host.

All 533 source/config/test files are archived under `obj/pa19-work/binder-source-v2`.
Final compiler SHA256 is
`0580d4325d404b6bd481d4257e4147a65e79692de4dfea02b1b97493031b369a`.
The validation receipt is `obj/pa19-work/binder-validation-v2.json`; prior snapshots,
qualification records and all observations remain in `obj/pa19-work/binder-*` and
`obj/perf/`. This checkpoint advances PA19; it does not complete PA19–PA21.


## General trait implementation headers

Local trait impl headers now retain typed Self, the complete trait application,
owner identity and generic/lifetime environment. Inherent, builtin Drop and user
trait impls share the same owner elaboration. Type and const parameters must
constrain Self or a trait argument; a where bound alone is insufficient. Unused
impl lifetimes remain legal. Trait safety, target and argument well-formedness,
trait parameter requirements and supertrait obligations are checked explicitly.

Prepared headers are distinct from validated implementations. This checkpoint
accepts memberless local trait implementations whose contracts can already be
proved. It does not select these impls as evidence in callers or dispatch trait
methods. A trait or impl with associated members retains the conformance gate.
General method conformance, implementation selection, dispatch and PA19 release
remain required; PA20 and PA21 are also unfinished.

Overlap indexing shares rigid subject keys with caller assumptions. Closed headers
use a complete lifetime-erased application key, so thousands of distinct const
arguments on the same Self do not create quadratic pair lists. Open candidates
retain their matching rigid and wildcard buckets. A single existential equation
context covers Self and every type/const argument, preserving repeated-variable
correlation. Candidate-pair capacity is charged before extending storage. This
structural test rejects compatible headers conservatively; predicate-sensitive
coherence is still required in PA20. Lifetime erasure never grants body permission.

An independently qualified nominal-input case exposed a missing source of lifetime
assumptions. An impl's Self and type arguments now contribute their implied
lifetime well-formedness facts to both header requirements and retained equality
proofs. Equal lifetimes can satisfy a nominal trait argument equation; a one-way
bound cannot. These inputs do not grant Copy or arbitrary user-trait bounds.

Forty-seven source controls agree with pinned rustc across parameter constraints,
primitive/reference/tuple/function targets, defaults, safety, contract requirements,
lifetimes and complete-header overlap. Typed assertions retain actual owner
parameter identities across Self and trait arguments. Two registry invariants
check 4,096 distinct closed const applications without overlap probes and preserve
lifetime-erased overlap candidates without publishing unvalidated evidence.

The previous blanket-gate expectation for an empty user trait named Drop was
updated to check separate user-impl and builtin-destructor identities. A new native
control, independently executed by rustc and CPPGM, verifies that wrapping a real
destructor-bearing field in a custom Drop marker still destroys the field exactly
once. The full course run caught a changed diagnostic for the unsupported associated-
type boundary. Associated type/const impls now retain their explicit early gate,
before ordinary definition scheduling, preserving the existing text contract.
No course fixtures or generated reference sidecars change.

Validation passes 832 frontend tests in both debug and release. After restoring
the associated-item diagnostic gate and factoring the safety query for Clippy,
all 36 affected release tests and all 3,790 PA1–PA18 text oracles pass again.
Clippy all/pedantic, formatting, the 538-file audit and 27 harness tests pass.
Upstream and Reference integrity checks pass (24 fixtures and 126 files).
The independent rustc sweep, 18 existing maintainer native groups, 1,126 course
executions and three codegen controls pass. The new custom-Drop native control
also passes on the final release binary. The sixteen declared PA13 runtime
differences remain explicit. Lowering and the external backend bridge are unchanged.

Eighteen frozen A/A and five-block ABBA jobs retain 468 observations; one repeat
adds 26. Every source, binary and input hash and full/count output equality check
passes. No local RustGM build, test or diagnostic probe overlaps timing. Ordinary
functions, records, receivers, trait bounds and builtin Drop change median wall
time by +0.6%, -0.9%, -0.4%, -1.9% and -1.3%; mixed blocks and calibration spreads
of 3–6% do not establish stable improvements or regressions. Existing-workload
median RSS differences range from -44 to +26 KiB. Trait-Drop's initial -16.8%
median is unstable (blocks range from -16.8% to +8.8%); the retained repeat is
-0.6%, so the initial observation is not a speedup claim.

New same-binary 64/256/1,024-item workloads take 4.65/7.07/19.94 ms for distinct
nominal Self types, 4.60/7.31/18.38 ms for distinct const arguments on one Self,
6.79/15.08/47.13 ms for generic bounded headers and 6.50/15.91/53.35 ms for lifetime
headers. These are measured scaling points on this host, not throughput guarantees.
All 541 source/config/test files are archived under
`obj/pa19-work/impl-headers-source-v2`. Final compiler SHA256 is
`1a39b2007f5390a83a60735feb41be3d87cc46c016fe2244b4a6cd4aed756ee9`.
The receipt is `obj/pa19-work/impl-headers-validation-v2.json`; qualification, prior
snapshots, failed checks and measurements remain under `obj/pa19-work/impl-headers-*`
and `obj/perf/`. This advances PA19 without declaring PA19–PA21 complete.

## Trait method conformance before dispatch

Method-bearing implementations now validate direct associated methods instead of
failing the blanket impl gate. One owner/name index matches members in declaration
order, rejects extra methods and missing required methods, and retains each trait
method identity with either its override or its default. Receiver presence, safety,
parameter kind/arity and supported function qualifiers are checked even when the
program never calls a method. Required signatures still never become executable
bodies. Associated types/constants retain their separate PA20 boundary.

Conformance substitutes trait Self and explicit arguments from the actual impl
header. The trait method's own type/const parameters remain rigid proof inputs;
the impl method's parameters map to them. This matters for implicit Sized: using
an impl-owned sized parameter as the proof input would incorrectly authorize a
stronger bound than the trait's `?Sized` contract. Impl method requirements are
proved under the impl owner plus the substituted trait method contract. The impl
body continues through the ordinary pipeline under only its own assumptions;
a weaker override cannot borrow the trait declaration's Copy permission.

Own early lifetime arguments match positionally. Trait late slots become universal
proof inputs and impl late slots become existential instantiations. Unused late
declarations do not change that contract. The existing capture-safe folder keeps
nested callable binders intact. Shape equality excludes type/const mismatches and
ordinary value coercions; directed region relations check parameters
contravariantly and results covariantly, permitting stronger returned lifetimes
and weaker input lifetimes. Expected signature and impl input well-formedness
supply implied lifetime facts through one shared relation helper. Impl outlives
requirements and trait-equation equalities remain obligations of that proof.

Sixty-three source controls independently agree with pinned rustc. They cover
method coverage, overrides/defaults, receiver presence, type/const arguments,
implicit sizedness, weaker/stronger builtin and user bounds, early/late lifetime
classification, alpha renaming, unused late parameters, input/output subsumption,
nested function correlation and lifetime equations. Typed assertions retain the
actual override/default identities and confirm that required declarations lack
bodies. A double-move override fails under its own weaker assumptions.

The PA18 trait-method boundary still declares exit 1 and independently accepted
Rust source. Its valid impl now passes conformance, so rejection moves from the
impl declaration at byte 38 to the still-unsupported method call at byte 90.
The existing `no inherent method with this name` diagnostic is generated through
`ref-test-pa18`. Review confirms exactly one stderr oracle changes, with only
compiler-source hashes changing in the reference receipt. Source, manifest,
expected status and the documented PA19 ownership remain unchanged. This is not
native trait dispatch; the called boundary program must pass when PA19 selection
and dispatch are implemented.

General impl selection as bound evidence, trait method lookup and concrete
required/default dispatch remain outstanding. Predicate-sensitive coherence,
associated items and normalization remain PA20 work; PA21 is still required.
Conformance currently consumes the existing caller/owner assumption proofs and
will share the general solver when it is implemented. PA19 is still planned as a
course release, not promoted by these maintainer controls.

Validation passes all 838 debug frontend tests, 837 release tests before the final
lifetime-equation control was added, and all 36 affected release tests afterward,
including that control. The final full course report passes all 3,790 cases.
The independent rustc sweep, 19 maintainer native groups, 1,126 course executions
and three codegen controls pass; the sixteen declared PA13 runtime differences
remain explicit. Clippy all/pedantic, formatting, the 543-file audit, 27 harness
tests and upstream/Reference integrity pass. Course native outputs are unchanged;
no trait-call execution is claimed by the existing native sweep.

Twenty-two frozen A/A and five-block ABBA jobs retain 572 observations, with full
and count output equality and all source/binary/input hashes verified. No local
RustGM build, test or diagnostic probe overlaps timing. Existing workload median
wall-time changes range from -1.5% to +1.1%, with median RSS changes between -20
and +16 KiB. Several paired blocks and calibration intervals are noisy: ordinary
functions include a +11.5% block, trait bounds a -29.8% block, and identical new
binaries show calibration spreads up to 39.5%. The small median differences do
not establish stable speedups or regressions; all observations remain retained.

New same-binary 64/256/1,024-item cases take 6.81/15.05/51.58 ms for simple methods,
7.48/20.00/74.21 ms for generic weaker overrides, 8.27/21.66/80.58 ms for lifetime
methods and 8.60/21.22/78.92 ms for trait-bound method contracts. These are measured
scaling points on this host, not throughput guarantees. All 546 source/config/test
files are archived under `obj/pa19-work/conformance-source-v1`. Compiler SHA256 is
`5048340c4e2766dde10779cd79d09564e06c230a1b62a2669af00f7278e4edc5`.
The receipt is `obj/pa19-work/conformance-validation-v1.json`; failed checks,
independent qualifications and all measurements remain under
`obj/pa19-work/conformance-*`, `obj/pa19-work/method-binders-v1` and `obj/perf/`.


## Inferred goals on both sides of impl matching

The trait equation engine now accepts inferred type and const parameters on both
sides of a complete goal. Canonical caller assumptions enter the same scoped
callable relation. Shared Self/argument variables remain correlated; function-local
lifetimes permit alpha renaming without losing their relationships. Either inferred
side rejects a binding that would capture a surrounding callable's local lifetime.
Outer lifetimes rebase when a variable moves across callable scopes and remain
explicit equality obligations. A shared unresolved variable is reflexive even
inside a callable. Closed subtrees retain the existing cheap equality path.

`equate_predicates` is the scoped inferred-goal interface for implementation
selection. `equate_predicate_shapes` shares the structural argument traversal but
continues to erase lifetimes exclusively for possible-overlap queries. This
prerequisite does not yet select general impl evidence or dispatch trait methods;
PA19 remains planned, and PA20/PA21 remain required.

Five new typed tests cover inference in both directions, local capture rejection,
closed nested functions, repeated variables across callable scopes, outer lifetime
rebasing and correlated type/const arguments. Existing binder and isolated-probe
controls now exercise both inferred and canonical interfaces, including mismatched
traits/arguments, local lifetime correlations, wide shared graphs and work limits.
No course fixture or text oracle changes. The source-level trait/conformance
regressions continue to run their independent pinned-rustc checks.

Validation passes 843 frontend tests and all 3,790 PA1–PA18 course cases, plus the
final 18 affected debug tests and 57 affected release tests after extending the
existing controls. All 19 maintainer native tests pass. Clippy all/pedantic,
formatting, the 544-file audit and 27 harness tests pass with no lint exemptions.
The full course-native sweep was not repeated: lowering and dispatch are unchanged,
and no new native trait-call support is claimed.

Eighteen frozen A/A and five-block ABBA workloads, followed by two retained repeats,
record 520 observations with full/count output equality. All source, binary and
input hashes agree; no local RustGM build, test or compiler probe overlaps timing.
Initial workload median wall-time changes range from -2.7% to +3.3% except for a
noisy -17.9% callable-outer result. That apparent speedup is not supported by the
paired blocks (median -0.5%); its repeat has a +2.1% workload median and +1.5%
paired-block median. Callable-alpha repeats at +3.6% overall and +1.2% paired.
Other noisy controls include builtin Drop's 35.4% A/A spread. These measurements
do not establish stable speedups or a performance-equivalence guarantee.
Initial median RSS changes range from -36 to +276 KiB; the callable-alpha +276 KiB
observation becomes +8 KiB in its repeat. Every observation is retained.

All 547 source/config/test files are archived under
`obj/pa19-work/bilateral-source-v1`. Compiler SHA256 is
`588b8aa827b598fda634617288988deced0bab1efae80d01e5274d2c7cd94317`.
The receipt is `obj/pa19-work/bilateral-validation-v1.json`; checks and measurements
remain under `obj/pa19-work/bilateral-*` and `obj/perf/`. The preceding conformance
checkpoint c7dae8d passed CI run 38694839917 on main.


## Validated implementation evidence in bodies

Body trait obligations, canonical body checks and inherent-method applicability
now use one selector over validated impl records. Candidate lookup reuses the
complete closed-application and outer-head partitions built for coherence.
An inferred goal and a candidate header share type/const equations across Self
and every argument; failed probes cannot constrain the caller. Conditional Copy,
Clone and Sized requirements use the caller's actual environment. User-trait
conditions are proved recursively with an explicit worklist and a total work
budget. Required cycles fail instead of becoming their own evidence; an unused
recursive conditional impl remains legal.

Successful child proofs commit their resolved complete goals into the parent
candidate. Only an unambiguous selected proof commits to the body. Its impl owner
and typed arguments are retained at the use site. Impl lifetime captures use an
isolated view of the caller's next free region identity; selected variables and
all equality, outlives and type-outlives obligations enter the existing region
checker. Failed alternatives publish no variables. Tests inspect both discarded
allocations and distinct fresh captures from successive successful calls, including
unused existential lifetimes constrained to static.

Completed recursive requests that allocate no new lifetime identities share
proof nodes. The collector visits this proof graph once. The 4/16/32-level
A-and-B recursive diamond controls therefore avoid exponential proof expansion.
Evidence that creates fresh lifetime identities is deliberately not reused until
it can be rebased safely; general canonicalized solver caching remains PA20 work.
The 4,096-header index control additionally checks that a closed goal probes its
own partition and cannot select a merely prepared, unvalidated header.

Numeric inference is distinct from an unconstrained whole Self. Integer and
floating-point variables search only their compatible primitive heads and can
infer a unique implementation's type. When progress stops, ordinary numeric
fallback runs and pending goals are retried. Two initial scratch predictions
that a sole u8/f32 impl would reject an unsuffixed literal were contradicted by
pinned rustc and corrected before shipping the controls; those original results
remain retained. General unknown Self still cannot be guessed from a sole impl.

Fifty-four source controls independently agree with pinned rustc, covering
conditional and recursive evidence, caller permissions, inferred type/const
arguments, callable correlation and capture rejection, lifetime requirements,
numeric domains/fallback, inherent-method conditions, typed selected arguments
and failed-candidate isolation. Four new programs also execute through CPPGM
against rustc, including recursive wrappers and a const argument inferred from an
array. These are generic calls whose bounds now have implementation evidence;
they do not claim user-trait method dispatch.

The full debug run passes 855 tests and all 3,790 existing course cases; the final
lifetime-publication test also passes separately, bringing the verified frontend
total to 856. The affected release run passes 83 tests, followed by all 13 final
selection tests after that last control was added. All 20 maintainer native tests,
1,126 course executions and three codegen controls pass. The sixteen documented
PA13 runtime differences remain explicit. Clippy all/pedantic, formatting, the
549-file audit and all 27 harness tests pass. Course fixtures and text oracles
remain unchanged.

Four independently accepted source controls still identify the next declaration
work: a closed trait bound, supertrait impl requirements in both source orders,
and a method conformance requirement implied by an unconditional blanket impl.
They remain recorded as unsupported in `selection-probes-v2.json`,
not counted among the passing controls. Declaration/nominal proofs and Drop
coverage still need the shared selector and complete lifetime proof packets;
prepared headers must not silently become validated body evidence to bypass
ordering. General trait method lookup and required/default concrete dispatch,
PA19's course release, PA20 and PA21 are still required.

Thirty-seven frozen A/A and five-block ABBA jobs plus one retained repeat record
988 observations. Full and count output equality and every source/binary/input
hash pass; no local RustGM build, test or compiler probe overlaps timing. Existing
workload median wall-time changes are -2.5% to +1.6%, except an unstable -27.5%
callable-outer result whose paired-block median is +1.0%. Its repeat is +1.2%
overall. Calibration spreads reach 73.5%, so no stable speedup or performance-
equivalence guarantee is claimed. Existing median RSS changes are -30 to +48 KiB.

New 64/256/1,024-use workloads take 7.25/12.34/30.59 ms for concrete selection,
9.58/19.34/58.47 ms for conditional selection, 13.38/36.29/137.69 ms with lifetime
captures and 6.87/11.90/28.85 ms for numeric inference. Distinct closed const
header/use pairs take 11.69/27.01/97.40 ms. Recursive predicate diamonds at depths
4/8/16/32 take 4.47/4.89/6.12/10.64 ms. These are measurements on this host, not
throughput guarantees. Every observation and the noisy calibration blocks remain
retained rather than filtered away.

All 552 source/config/test files are archived under
`obj/pa19-work/selection-source-v1`. Compiler SHA256 is
`1e384f30f0d3630160053b59024822045a2bc699ad036396aa5145de0d7b14f3`.
The receipt is `obj/pa19-work/selection-validation-v1.json`; qualifications, earlier
failed checks and measurements remain under `obj/pa19-work/selection-*`,
`obj/pa19-work/impl-selection-v1` and `obj/perf/`.

## Implicit trait method paths

`T::method`, `Self::method` and `<T>::method` now select trait declarations while
retaining the written Self type. This fixes a false acceptance where a default
method's `Self::f(3u16)` could silently infer a different Self. Primitive, nominal,
reference, array and alias prefixes share ordinary annotation inference. Implicit
trait arguments are inferred; a trait's type-path defaults do not constrain an
unwritten qualifier. Selected operands reuse the existing trait dispatch path.

An applicable inherent method wins. If its impl conditions fail for an explicit
or rigid prefix, lookup can fall back to a scoped trait. A still-unknown prefix
cannot use subsequent call arguments to make that choice. Method-own bounds,
argument types and expected return types do not disambiguate names. Enum variants
keep their earlier lookup and constant-evaluation behavior.

Parameter bounds precede lexical trait candidates. Scope lookup uses final
resolver bindings, including anonymous imports, aliases and macro call-site
scopes. The pinned compiler retains one contribution from ambiguous glob imports
for method lookup; reversed import/declaration-order controls and execution checks
qualify that behavior. Inner type names do not erase an outer trait's eligibility,
while a direct same-scope binding can hide a glob-imported trait.

A first implementation scanned all matching trait names for every use. A declared
1,024-trait/1,024-call control exposed budget exhaustion despite all those traits
being unimported. The final implementation lazily indexes local methods by name
and shares inherited candidate lists. Caller parameter bounds acquire a reverse
index only when implicit lookup requires it. Both concrete and rigid versions of
that control pass. No source text is reparsed and failed candidate equations do
not enter the caller's inference graph.

The new maintainer tests contain 48 acceptance controls, two generated scaling
programs and 27 executable comparisons with pinned rustc and CPPGM. Three initial
scope predictions were corrected against rustc before becoming test expectations;
the original predictions and corrections remain in the local evidence. An earlier
full run caught enum-alias constants entering method fallback; the existing early
constructor suite and new variant-versus-method controls cover that regression.

Validation passes 908 debug tests, 91 affected optimized tests, all seven affected
trait native groups and the seven-program early-constructor native group. All
3,790 PA1–PA18 text-oracle cases pass unchanged. Formatting, Clippy all/pedantic,
the 571-file zero-finding audit and 27 harness tests pass. PA19 remains planned:
general dot trait lookup and its complete course release are still required.
PA20 and PA21 remain unfinished, as does the recorded redundant Fn-bound
qualification discrepancy that needs review before release.

Sixty-three frozen A/A and five-block ABBA jobs plus one retained repeat record
1,664 observations. Every source, binary and input hash and full/count output
comparison passes; no local RustGM build, test or probe overlaps timing. Existing
workloads' median paired changes range from -6.7% to +4.9%. The +4.9% deferred-pointer
outlier has a 32.9% A/A spread and becomes -0.7% on repeat with a 3.1% spread.
Other calibration spreads reach 41.7%; these results do not establish a stable
speedup or performance equivalence. Existing median RSS changes are -28 to +60 KiB.
Earlier method and conditional-selection performance follow-ups remain separate.

New 64/256/1,024-use candidate median wall times in milliseconds are:

| Workload | 64 | 256 | 1,024 |
| --- | ---: | ---: | ---: |
| Concrete type paths | 8.02 | 19.59 | 67.80 |
| Rigid parameter paths | 12.30 | 34.06 | 139.79 |
| Unimported homonyms, concrete calls | 9.71 | 26.77 | 98.56 |
| Unimported homonyms, rigid calls | 13.44 | 81.86 | 184.49 |
| Wide trait member sets | 8.63 | 42.48 | 87.20 |
| Conditional inherent fallback | 12.28 | 46.18 | 125.27 |

These measurements retain all observations and do not imply a throughput bound.
The 574 source/config/test files are archived in `obj/pa19-work/implicit-source-v1`.
Compiler SHA256 is
`3dff0d0a3d0f957107554930d2b95aacc7c7fc90f09dfeec4817a3725d71229e`.
The receipt is `obj/pa19-work/implicit-validation-v1.json`; original predictions,
failed probes, qualifications and timings remain under
`obj/pa19-work/implicit-methods-v1`, `obj/pa19-work/implicit-*` and `obj/perf/`.

## Trait receiver calls

Dot calls now search inherent methods, parameter-bound methods and lexical trait
methods at each built-in dereference/autoref candidate. Bound priority pins Self
to the parameter that supplies the bound. A method that cannot use that Self can
still participate through a lexical implementation for another Self, such as
`&T`; this distinction preserves genuine ambiguity between reference impls.
Static associated functions cannot become dot methods. Call arguments, expected
results, method-own bounds, lifetimes and write permission do not select a fallback.

Primitive, nominal, rigid, reference and numeric receivers share ordinary callable
inference and concrete trait dispatch. A unique implementation can infer a numeric
receiver; otherwise ordinary numeric fallback remains available. An unconstrained
whole type is not inferred solely from a trait implementation. Mutable-reference
receivers reborrow the original place, including when Self is itself a reference.
Both rules were corrected from initial predictions using pinned rustc; original
predictions and results remain in the evidence rather than being overwritten.

Array receivers can unsize after the final built-in dereference. The actual borrow
retains the array place type and the region contract records its conversion to a
slice reference. Existing body lowering then produces the slice value. No source
is reparsed, no second evaluation of the receiver is introduced and the backend
contract is unchanged. Trait calls infer Self through the receiver signature;
ordinary call arity counts the hidden receiver from its typed call node.

The first deep-reference control exposed 139,602 inference steps at 64 reference
levels. Receiver-shape checks and the existing trait head indexes now exclude
impossible candidates before importing graphs. The 64/96-level controls each need
one full candidate probe, while successful candidates still undergo complete trait,
borrow and lifetime proof. Existing inherent selection retains its indexed path.

The new tests have 93 independently qualified acceptance controls and 50 native
comparisons. They distinguish inherent/shared/mutable precedence and parameter
bounds from lexical reference impls, exercise defaults and recursion, and verify
source-place mutations, returned borrows, captured lifetimes and two-phase calls.
Two generated dot-call workloads add 1,024 independent scopes with 1,024 unimported
homonyms; they share the existing scope-index regression with type-path lookup.

A final control exposed a gap shared with the preceding implicit-path checkpoint:
lookup selected a bound method, but later inference could replace its trait arguments
with those of an unrelated blanket implementation. The final implementation retains
the selected origin and restricts exactly the callable's implicit Self predicate to
caller-assumption evidence. Additional method requirements keep ordinary impl proof.
Twenty-two controls cover types, consts, lifetimes, function values, supertraits and
multiple bound applications; six additional programs execute the selected dispatch.
One explicit-lifetime prediction was corrected against rustc before accepting the
control, and its original prediction and diagnostic remain retained.

The former PA18 trait-method boundary source is preserved verbatim in the accepted
maintainer/native checks. Its obsolete rejection and manifest entry are removed;
all remaining 432 PA18 oracle triples are unchanged, and the receipt is regenerated
through the explicit reference command. PA18 now has 268 accepted and 164 rejected
cases, including two documented later-stage acceptance differences. The independent
PA18 checks, 24 upstream fixtures and 126 vendored Reference files pass.

Validation passes 929 debug frontend tests, 117 affected optimized tests, all ten
affected native groups and 3,789 active course cases. Formatting, Clippy all/pedantic,
the 576-file zero-finding audit and 27 harness checks pass. The preceding implicit
path checkpoint f539916 passed CI run 38728011044 on main. PA19's semantic view and
complete grading contract remain required; PA20 and PA21 are still unfinished.

Seventy-nine frozen A/A and five-block ABBA jobs plus three retained repeats record
2,132 observations for the final source. All full/count output comparisons and
source/binary/input hashes pass. No local RustGM builds, tests or probes overlap
these timing runs. The earlier pre-fix snapshots and their 1,950 observations remain
retained separately; they do not validate the final bound-selection correction.

Against the published implicit-path compiler, initial paired medians range from
-5.7% to +5.5%, with A/A spreads up to 39.0%. The Drop, callable-inference and implicit-
fallback positive outliers become -0.4%, -0.9% and -4.6% on repeat. Every original
sample remains included. Against the pre-fix receiver compiler, paired medians
range from -1.4% to +2.7%, with A/A spreads up to 31.5%. These measurements do not
establish a stable speedup or performance equivalence. Median RSS differences are
-686 to +744 KiB against the published compiler and -20 to +398 KiB against the
pre-fix receiver compiler.

Final candidate median wall times in milliseconds are:

| Workload | 64 | 256 | 1,024 |
| --- | ---: | ---: | ---: |
| Value receivers | 7.79 | 18.28 | 62.65 |
| Borrowed receivers | 9.24 | 23.88 | 83.72 |
| Rigid receivers | 13.95 | 43.51 | 182.46 |
| Unimported homonyms | 9.31 | 30.54 | 93.64 |
| Array to slice receivers | 9.93 | 27.62 | 99.57 |
| Reference Self | 9.44 | 25.55 | 91.43 |
| Bound arguments, dot calls | 18.90 | 75.88 | 249.21 |
| Bound arguments, type paths | 16.89 | 78.66 | 220.88 |

The 579 source/config/test files are archived under
`obj/pa19-work/trait-receiver-source-v2`. Compiler SHA256 is
`dfd43e1a1604644f49c585b700104854794846ffe4a9a53ce91afa2f2075a9f0`.
The receipt is `obj/pa19-work/trait-receiver-validation-v2.json`; original predictions,
corrections, failed probes and validation logs remain under
`obj/pa19-work/dot-receivers-v1`, with frozen inputs, measurements and both source
snapshots under `obj/pa19-work/trait-receiver-*` and `obj/perf/`.


## Semantic inspection view

`--emit-semantics` and `--count-semantics` now expose the retained trait/method
facts needed for PA19's text-oracle contract. Owners, members, parameters,
defaults, complete predicates, validated impl method mappings, qualified paths,
receiver conversions and selected implementation arguments have explicit records.
Canonical type, definition, argument and region writers are shared with the type
view. Sorting occurs at the output boundary; compiler phases keep using typed facts.

The view runs declaration/body inference. Lifetime, ownership, loan, cleanup and
concrete-instance proofs remain in the complete LowIR pipeline; pending region
constraints are labelled accordingly. One focused control deliberately succeeds
at inspection and fails its later lifetime proof. Other controls verify actual
method/default mappings, bound versus lexical selection, explicit qualifications,
Drop/const handling, count agreement, output errors and deterministic bytes across
seven independently seeded CLI processes.

Four focused debug tests and seven affected optimized tests pass. All 3,789 active
PA1–PA18 text-oracle cases remain byte-identical; no references or receipts are
regenerated for this view. Formatting, Clippy all/pedantic, the 578-file zero-finding
audit and all 27 harness checks pass. PA19 remains planned until its complete
handout, fixture manifest and independent/native qualification are published.

Eighteen immutable jobs retain 468 observations with A/A calibration and five
paired ABBA blocks each. They cover 64/256/1,024 bound receiver/type-path calls in
old type-count mode, new semantic-count mode and full semantic rendering. Full
output hashes and count results agree, as do all source/binary/input hashes.
No local RustGM builds, tests or probes overlap timing. Existing type-count paired
medians range from -1.6% to +2.6% with A/A spreads up to 11.0%; identical-binary new
mode comparisons range from -3.3% to +3.9%, with calibration spreads up to 76.4%.
These observations do not establish a stable performance change.

Final median wall times in milliseconds are:

| Workload | 64 | 256 | 1,024 |
| --- | ---: | ---: | ---: |
| Receiver semantic count | 8.86 | 38.80 | 74.34 |
| Receiver full semantic view | 9.15 | 23.31 | 81.03 |
| Type-path semantic count | 10.30 | 28.14 | 101.84 |
| Type-path full semantic view | 10.92 | 30.06 | 111.29 |

The 581 source/config/test files are archived under
`obj/pa19-work/semantic-view-source-v1`. Compiler SHA256 is
`18e84f6d5d6c9c8227b6520378bb8035c8ec25703d49576befacd8a987cbc1ec`.
The receipt is `obj/pa19-work/semantic-view-validation-v1.json`; declarations,
validation and timings remain in `obj/pa19-work/semantic-view-v1`,
`obj/pa19-work/semantic-view-*` and `obj/perf/`.


## PA19 course release

PA19 is now implemented-draft with 1,070 declared fixtures and generated oracle
triples. The semantic lane contains 439 cases; the LowIR lane contains 631,
including 247 accepted programs that all execute through pinned rustc and CPPGM.
All installed oracle triples match the independently repeated pre-install outputs.
The complete handout, manifest, references, provenance, root test registration and
independent/native targets are published together.

Five future-stage boundaries, one pending-lifetime inspection and three redundant-
binder compiler differences are explicit. The latter preserve canonical equality
and retain the pinned default compiler's rejection in the independent metadata;
next-solver diagnosis does not replace that oracle. The nine differences are
excluded from acceptance-equivalence claims. Review also corrected one extraction
prediction that mistook a maintainer PA25 boundary for candidate acceptance; its
original declaration, failure and correction remain recorded.

All 4,859 active course cases, 1,070 PA19 independent language checks and 247 native
executions pass. Clippy all/pedantic, formatting, the 578-file audit, 27 harness
tests, upstream verification and backend qualification pass. Compiler sources are
unchanged from the measured semantic-view checkpoint. Both preceding main commits,
c3bb0dc and d28780f, passed CI runs 38735309406 and 38736772181. The course release
receipt is `obj/pa19-work/course-v1/release-validation-v1.json` and its public review
is [pa19-validation.md](pa19-validation.md). PA20 and PA21 remain required.

## PA20 scoped failure memoization

PA19's release commit `7931fc5` passed CI run 38739538725. The first PA20 solver
checkpoint extends the existing iterative trait search to reuse fully resolved
failed subgoals. The memo remains inside one immutable caller, query mode and
region scope; complete type/const arguments and callable binder identities remain
in its canonical keys. No answer is shared across declaration-only and validated
queries, across changed assumptions, or after implementation availability changes.

An inductive cycle cutoff is not a completed negative proof. A search-local cycle
counter prevents both the cutoff and its failed ancestors from entering the memo.
Deferred inference and work-budget errors remain separate outcomes. Successful
proof sharing retains the previous restriction against reusing fresh existential
lifetimes without rebasing. The outermost request is never memoized, since there
is no subsequent consumer in that search; this preserves the allocation-free memo
path for a simple leaf implementation. Assumption lookup still precedes impl search.

Six internal controls cover a 128-candidate/32-level repeated failure, inductive
cycles, caller trait assumptions, builtin permissions, provisional headers and
work exhaustion. Source controls declare twelve acceptance outcomes before pinned
rustc qualification. Two native programs verify the selected method values after
failed candidates and across distinct const arguments. The test-only source
expansion harness lives under `dev/tests/invariants`; compiler phase dependencies
and the audit rules remain unchanged.

The full maintainer debug run passed 940 tests before the final removal of unused
outermost memo entries. The final candidate passed ten optimized trait invariants,
117 affected optimized tests, both native programs, all 4,859 active course cases,
Clippy all/pedantic, formatting, the 580-file audit and 27 harness tests. No course
reference bytes or assignment status changed. Associated items, projection
normalization and the remaining PA20/PA21 requirements are still open.

The final frozen compiler has SHA256
`3d357d89cec39182e708a6872defc6aab03179f6dd4bee85ba9911225789d4be`.
Its 583 source/config/test files are archived under
`obj/pa20-work/scoped-cache-v1/source-v2`. Twelve frozen inputs were independently
accepted by pinned rustc and matched the published PA19 compiler in full and count
semantic output. A/A calibration and five paired ABBA blocks per input retained
312 observations; no local RustGM build, test or probe overlapped timing.

Repeated failed-bound workloads reduced paired median wall time by 56.1–87.0%,
with A/A spreads of 0.2–3.1%. The deepest case went from 167.66 ms to 21.85 ms.
Receiver and concrete-goal controls ranged from -1.3% to +0.6%, with A/A spreads
of 1.9–4.6%; these controls establish no stable change. All observations,
including a delayed baseline sample, are retained. Full/count output and frozen
source, input and binary hash checks passed. The receipt is
`obj/pa20-work/scoped-cache-v1/validation-v2.json`; the untimed predecessor snapshot
and its validation logs remain alongside the final evidence.

## PA20 canonical projection representation

The preceding solver checkpoint `c7a4958` passed CI run 38743603167. Associated
type applications now have a distinct canonical `Kind::Projection`, with the
declaring item, Self, full trait application and separate family arguments.
One immutable declaration schema fixes trait ownership and result sizedness;
conflicting redeclarations and invalid references fail. Storage is allocated
only when projections are declared. The canonical type enum remains 24 bytes.

A dependency bit retains projections even when their inputs contain no generic
parameters or lifetimes. The common bounded type fold substitutes Self, trait
arguments and family arguments at the current binder depth. Parameter and bound-
lifetime occurrence collection visits all inputs; these occurrence sets are not
injectivity proofs. Variance treats projection inputs invariantly. Ordinary
inference cannot reverse a projected equality into input equations. The type view
has an explicit projection spelling using the existing generic-argument writer.

Legacy layout, Copy/equality, destruction, shape and lifetime queries diagnose
missing normalization instead of classifying a new constructor through a catch-all
primitive case. This is a typed representation checkpoint: source elaboration,
associated-item conformance, projection equalities and normalization still need
implementation. No new source program, course oracle or assignment status is
claimed by this change; PA20 and PA21 remain required. In particular, source
generic-parameter constraint checks must distinguish occurrences inside a
non-injective projection from constraining inputs before the frontend publishes
these types. Associated constants and PA21 family rules also remain open.

Six typed invariants check schema/application identity, complete parameter/const
uses, binder capture and depth-specific substitution, lack of invented result
properties, invariant input positions and a 10,000-level shared type graph.
All 946 maintainer debug tests, 34 affected optimized tests (including two native
programs), all 4,859 course cases, Clippy all/pedantic, formatting, the 582-file
audit and 27 harness tests pass. Earlier oracle bytes are unchanged.

The frozen compiler SHA256 is
`aa5270879d99538e6f78bb226af98b444f9de87ff41622bf785c97f7bc457bf4`.
Its 585 source/config/test files are archived under
`obj/pa20-work/projections-v1/source-v1`. Twelve ordinary workloads compare full
and count semantic bytes with the published solver compiler. Paired median wall
changes range from -1.0% to +2.7%, with A/A spreads of 2.6–94.8%. Two retained
repeats yielded +2.6% and +1.6%, with calibration spreads of 3.6% and 5.7%.
These small positive observations remain visible; no overall speedup is claimed.

A separate frozen driver measures only canonical projection construction and
substitution through the reference library; it is not a new frontend CLI mode.
At depths 4,000, 16,000 and 64,000, work counts are 32,002, 128,002 and 512,002,
with median wall times 9.82, 31.87 and 133.96 ms and peak resident medians
3,616, 8,542 and 28,776 KiB. This is same-binary scaling evidence, with A/A spreads
up to 41.6%, not a comparison against a prior projection implementation.

All 442 observations are retained: 312 initial regression observations, 52
repeat observations and 78 direct-fold observations. No local RustGM build, test
or unrelated probe overlapped timing. Full/count or repeated-driver output and
source, input and binary hashes pass. The receipt is
`obj/pa20-work/projections-v1/validation-v1.json`.

## PA20 associated type declarations and conformance

Projection checkpoint `042661c` passed CI run 38749084673. Source trait declarations
can now require non-parameterized associated types, and trait implementations can
define them using concrete types or their own type, const and lifetime parameters.
Each declaration retains its canonical projection and result contract. Required
definitions are matched in the type namespace and published with the validated
implementation as declaration/definition identity pairs; the semantic view emits
`impl-type owner=... declaration=... definition=...` in declaration order.

Definitions must be present, belong to the trait, have inherited visibility and
provide a right-hand type without additional item bounds. Stable-profile checks
reject inherent associated types and associated type defaults in traits. Builtin
Sized, Copy and Clone requirements are checked by identity. The implicit Sized
bound can be relaxed with `?Sized`. A definition's well-formedness and lifetime
requirements are proved even when unused, under the impl environment. Unlike an
ordinary alias, an unused definition cannot defer an invalid underlying type.
Lifetime queries and their constraint environment are shared across all associated
definitions of one impl; another impl's successful proof grants no permission.

Thirty-one source acceptance/rejection controls agree with pinned rustc. Typed
tests separately check projection ownership, Self, result contracts and the selected
definition mapping, including reversed declaration order and distinct type/value
namespaces. New native controls execute default and overridden methods on traits
with associated types. The full debug maintainer run passed 949 tests before the
final shared-lifetime-query adjustment and four additional source controls. The
final candidate passed 141 affected optimized tests, including all 31 controls,
the new native programs and the existing trait native suite. Formatting, Clippy
all/pedantic, the 585-file audit, 27 harness tests and all 4,859 course cases pass.

The existing PA18 and PA19 associated-type boundary programs still reject: their
declarations now pass, and the unsupported projected-type annotation is diagnosed
at offsets 76 and 70. Both programs were independently rechecked as rustc-accepted.
Only those two diagnostic sidecars and the owning reference provenance records
were regenerated; source fixtures, statuses and other oracle bytes are unchanged.
The declared update review is in
`obj/pa20-work/associated-declarations-v1/declared-boundary-updates-v1.json`.

The frozen compiler SHA256 is
`8e0780114aeca3edbe3798189bbd89235f498fd2084ad69b8b5fec17c6fc9e6b`;
588 source/config/test files are archived under
`obj/pa20-work/associated-declarations-v1/source-v1`. Twelve ordinary workloads
match the published projection compiler in full and count semantic bytes. Six
new scaling workloads are independently accepted by pinned rustc and have exactly
the declared number of implementation-type records. A/A calibration and five
paired ABBA blocks per input retain 468 initial observations; three repeated
ordinary workloads add 78, for 546 retained observations. No local RustGM build,
test or unrelated probe overlapped timing. Source, binary and input hash checks
and every output equality check pass.

Initial ordinary paired medians range from -5.5% to +5.6%, with A/A spreads of
3.4–65.2%. Repeats for failed-width 128, receiver-rigid 1024 and concrete-goals 1024
yield +6.0%, +0.6% and +5.5%, with A/A spreads of 4.9%, 5.7% and 2.0%. These
positive observations remain part of the record; this checkpoint establishes no
speedup or absence of cost. Same-binary associated declaration widths 128, 512 and
2,048 take median 6.93, 13.55 and 39.99 ms, with peak resident medians 5,750,
7,142 and 9,972 KiB. Distinct associated impl counts 128, 512 and 2,048 take
11.18, 26.76 and 95.96 ms, with peak resident medians 6,448, 7,858 and 14,200 KiB.
Those new-feature scaling measurements have no older supported implementation to
compare. The validation receipt is
`obj/pa20-work/associated-declarations-v1/validation-v1.json`.

PA20 and PA21 remain planned. Source projection paths, normalization, equality
predicates, arbitrary user-trait/lifetime associated bounds and associated constants
still need implementation; unsupported contracts diagnose rather than disappear.
Associated type parameters and item where clauses remain the explicit advanced
generic boundary. Next, connect projection elaboration and normalization to the
typed implementation mapping and scoped selection evidence, preserving unresolved
generic projections without treating their inputs as injective. Concrete layout
and lowering must consume normalized results. Do not confuse the completed
declaration checks with complete projected-type usage or assignment release.


## PA20 qualified associated type normalization

Associated declaration checkpoint `aa44b01` passed CI run 38752633074. Explicit
qualified paths such as `<S as Trait<T, N>>::Item` now select an implementation,
substitute its arguments into the associated definition and normalize the result
before structural checks, layout or lowering. This includes concrete signatures,
aliases, generic impl substitutions, trait defaults, nested projections, static
references and higher-ranked function pointer results. All implementation contracts
still complete before bodies or executable facts are published.

The common bounded type fold now suspends at typed projection requests after
normalizing their inputs. Declaration work can finish a selected definition while
the fold is suspended; its replacement is recursively normalized before publication.
Completed work is shared only within the same proof scope and binder depth. Active
cycles, stale responses and exhausted work diagnose explicitly. A generated graph
with 10,000 projection levels verifies bounded iterative traversal and shared work.
Already-normalized structural annotations need no additional normalization walk;
instantiated type paths still check for projections introduced through defaults.

Projection occurrences cannot constrain an impl parameter merely by appearing in
its projected inputs. Separate constraining collectors preserve ordinary occurrence
queries while treating those inputs as opaque. Associated definitions also reject
an impl lifetime used in the result but unconstrained by Self or trait arguments.
Independent controls cover alias erasure, unused lifetimes, nested binders, outer
capture and mutually related but unconstrained lifetimes. Associated declarations
cannot masquerade as ordinary lexical aliases: bare names in traits and impls
reject, while an ordinary alias nested inside a method remains valid.

Thirty source path cases and 39 associated declaration cases agree with pinned
rustc. New native programs exercise scalar, aggregate and wide-reference results;
the associated declaration and existing trait native controls also pass. The final
candidate passes 161 affected optimized tests including those native checks,
formatting, Clippy all/pedantic, the 589-file audit with no findings, 27 harness tests,
upstream provenance and all 4,857 active course cases. The full debug run passes 955
tests; its compiler predates the final extension of the bare-name guard to impls,
which is covered by the final optimized run and three additional source controls.

PA18 and PA19's former associated-type rejection fixtures now succeed. Their exact
source bytes are retained in the independently qualified path tests, including
the different trailing-newline contracts. Their obsolete rejection sidecars and
manifest entries were removed; all remaining oracle bytes are unchanged. The
current counts are PA18 431 and PA19 1,069. Their reference provenance was regenerated
through the official targets. The declared migration review and original sidecars
remain under `obj/pa20-work/normalization-v1/`; these cases will enter the PA20
manifest when its complete grading contract is ready.

This is still a partial PA20 checkpoint. Scoped lifetime rebasing and retained
selection evidence, symbolic projection equalities, shorthand `T::Item`/`Self::Item`,
general projected generic bodies, associated constants and arbitrary associated
bounds remain required. Unsupported evidence is diagnosed, never discarded to
manufacture a normal form. Projection use in early headers/defaults also requires
pipeline integration. PA20 and PA21 remain planned; their full original scope is
unchanged.


The final frozen compiler SHA256 is
`a5cf9019698ea1f447189d2089a23c487896b8f1d1690ed6d72afe0f9a84df84`;
592 source/config/test files are archived in
`obj/pa20-work/normalization-v1/source-v2`. Eighteen existing workloads match the
published associated-declaration compiler in full and count semantic bytes. Six
new source workloads, independently accepted by pinned rustc, check projection
chains and repeated qualified annotations at scales 128, 512 and 2,048; their
semantic output contains exactly the declared implementation-type record counts.

Two frozen candidates retain 624 observations each, for 1,248 total. Every run
uses A/A calibration and five paired ABBA blocks per input. The initial ordinary
paired medians range from -2.2% to +3.1%. After avoiding redundant structural
annotation checks and correcting qualification, final medians range from -1.3%
to +4.3%, with A/A spreads of 0.7–62.7%. Concrete-goal cases remain 3.2–4.3%
slower, following the initial 2.6–3.1% observations. The removed redundant checks
do not establish elimination of that cost; no overall speedup is claimed.

Final same-binary chain medians are 8.87, 30.26 and 91.18 ms, with peak resident
medians 6,478, 9,982 and 21,090 KiB. Repeated-path medians are 5.78, 11.93 and
36.62 ms, with 5,720, 6,468 and 9,936 KiB. These are new-feature scaling evidence,
not a comparison with a previously supported implementation. The shared host had
unrelated compiler activity; all noise and observations remain recorded. No local
RustGM build, test or unrelated probe overlapped either timed run. Full/count
output equality and source, binary and input hashes pass. The receipt is
`obj/pa20-work/normalization-v1/validation-v1.json`.


## Projection lifetime constraints before normalization

Qualified projection checkpoint `6d04b06` is published on `main`. Independent
qualification for the next lifetime integration exposed Rust's E0581 constraint:
`for<'a> fn(<&'a u8 as A>::Out) -> &'a u8` is invalid even if the selected impl
normalizes its input to `&'a u8`. The occurrence inside the non-injective projected
input cannot constrain the output lifetime. An additional direct `&'a u8` input
makes the binder valid. Aliases must preserve this distinction as well.

The canonical function-pointer validator now consumes unreduced signatures,
collects constraining input positions separately from output occurrences and
checks every opaque input occurrence against the binder's valid slot range.
Six typed cases agree with independently compiled Rust programs, covering direct
and projected inputs/results and an unused projected input lifetime. A separate
malformed canonical control checks a lifetime slot hidden in trait arguments.
This establishes the typed query contract; source support for escaping lifetime
projections remains explicitly rejected until scoped rebasing, retained selection
obligations and unreduced annotation provenance are integrated. Normalizing first
and checking only the normal form would incorrectly accept the E0581 cases.

All 149 affected optimized tests, including existing projection and trait native
controls, 4,857 active course cases, 27 harness tests, formatting, Clippy all/pedantic
and the 589-file audit pass. Course fixtures, manifests and oracle bytes are unchanged.
The frozen compiler SHA256 is
`eaca62988472d0fc3254bcd03f42efc6e6a86a3e080eb3ed80e0953396209954`;
592 source/config/test files are archived in
`obj/pa20-work/projection-regions-v1/source-v1`. Twelve ordinary regression inputs
and three independently qualified pointer-binder workloads match `6d04b06` in
full and count semantic bytes. A/A calibration and five ABBA blocks retain all
390 observations. Ordinary paired medians range from -1.8% to +1.7%, with A/A
spreads of 1.8–7.2%; pointer-binder medians are +0.3%, -0.5% and +1.5% at 128,
512 and 2,048 aliases, with spreads of 3.0–6.6%. These small differences do not
establish a speedup. No local build, test or unrelated probe overlapped timing;
source, binary and input hashes and all output comparisons pass. The receipt is
`obj/pa20-work/projection-regions-v1/binder-validation-v1.json`.

Eighteen future source controls are independently qualified under the same local
evidence directory. They cover static and caller lifetimes, permuted trait
arguments, impl outlives requirements, strengthening failures, nested callable
binders and local annotations. Their positive outcomes are requirements for the
next integration, not current source acceptance claims. PA20 and PA21 remain
planned and required.


## Unreduced annotation identities and alias constraints

Qualified normalization `6d04b06` passed CI run 38762675396; binder constraints
`edff17c` passed CI run 38764378850. The declaration pipeline now retains original
projection identities separately from normal types. Sparse records cover source
annotations, alias definitions, supplied argument lists and generic defaults.
Their storage is allocated on first use; ordinary node facts and their layout
are unchanged, and ordinary programs allocate no such records.

Completed typed children carry these identities through aliases, references,
arrays, pointers, tuples and callable signatures. Alias paths reuse the existing
joint parameter/lifetime substitutions. Defaults use a separate append-only
unreduced argument prefix; the extra state is allocated only when an argument
actually differs. Nested projections retain original Self and trait inputs while
selection consumes their already-normalized children. Feeding the original inputs
back into selection would repeat work along alias chains; the final implementation
keeps those two responsibilities separate.

This fixes an observable source error: `type P<T> = <T as A>::Out` can use `T`
even when a blanket impl normalizes its result to `u16`. The alias occurrence check
now consumes the original definition instead of incorrectly diagnosing unused `T`.
Aliases with an actually unused parameter still reject. Normal results do not infer
projected inputs: a generic call whose signature erases its parameter still needs
an explicit argument. Inference, layout and lowering consume normal types.

Thirteen alias/function controls agree with pinned rustc. Additional independent
controls cover compound signatures, nested Self/trait projections and a 512-link
alias chain. Typed invariants verify distinct origins for equal normal forms,
default substitution, alias forwarding, callable item identities and absent storage
for ordinary programs. Two new native programs execute a demanded generic function
whose alias result erases its parameter and an aggregate using a dependent default.
Function-pointer binder checks consume unreduced completed signatures, preparing
for lifetime projection support without claiming that support is finished.

The full debug run passed 963 tests before the final work-sharing correction and
additional chain control. The final implementation passes 179 distinct affected
optimized tests, including the new and existing native controls, all 4,857 active
course cases, 27 harness tests, upstream provenance, formatting, Clippy all/pedantic
and the 593-file audit with no findings. Course fixtures, manifests and all oracle
bytes are unchanged. Escaping lifetime projections still diagnose explicitly;
scoped rebasing, retained obligations, partial inferred signatures, symbolic
equalities and the remaining full PA20/PA21 requirements remain open.


The final compiler SHA256 is
`0114551c47f32398ced97e50f7c3c8c2a8d82bfe7797f933e9b87b062042a728`;
596 source/config/test files are archived under
`obj/pa20-work/unreduced-v1/source-v2`. Fifteen existing workloads match the
published binder-check compiler in full and count semantic bytes. Nine new
retained-origin workloads vary alias declaration width, repeated uses and chain
depth at 128, 512 and 2,048; all are independently accepted by pinned rustc.
The revised compiler also matches the initial candidate's full/count semantic
bytes on all 24 inputs. Three previously accepted chain programs separately
match the published compiler's full/count LowIR bytes, allowing a meaningful
runtime comparison despite their intentionally richer semantic type graphs.

Both immutable candidates retain A/A calibration and five ABBA blocks for every
input, totaling 1,404 observations. Initial ordinary paired medians range from
-15.7% to +2.0%; final ordinary medians range from -3.1% to +3.0%, with A/A
spreads of 1.3–37.7%. The initial large negative concrete-goal result is retained
and is not reproduced by the final run; no general speedup is claimed.

The cost of retaining origins remains visible. Initial LowIR chain comparisons
are +6.8%, +8.1% and +12.2%. Removing redundant records at alias item nodes keeps
the RHS annotation and definition identity; final chain comparisons are +5.0%,
+8.9% and +11.2%, with A/A spreads of 2.4%, 0.9% and 6.4%. These are costs of
additional retained facts, not evidence of an absent regression or a demonstrated
speedup from the record change. The required original identities remain intact.

Final same-binary semantic medians for widths 128/512/2,048 are
6.90/15.84/53.15 ms, with peak resident medians 6,438/7,872/13,456 KiB. Repeated
uses take 4.98/8.02/21.19 ms and 5,794/6,426/8,526 KiB. Chains take
6.23/12.68/42.56 ms and 5,726/7,132/9,970 KiB. All observations and output/hash
checks are retained, and no local build, test or unrelated probe overlapped timing.
The receipt is `obj/pa20-work/unreduced-v1/validation-v1.json`.

## Scoped projection lifetime proofs

Qualified associated types now normalize selected impl lifetimes back into their
caller's scope. The common type fold opens surrounding lifetimes while preserving
static, existing body variables and nested local binders. Equality components
restore selected result identities and shift captures beneath nested callables;
they do not grant the equalities as assumptions. Every selection equality,
outlives and type-outlives obligation remains mandatory, including when the
normalized result erases the lifetime entirely.

Declaration annotations share one directed proof per binder with disjoint fresh
intervals, checked against the completed owning environment. Local annotations
retain evidence through the same body allocator and proof gate as trait calls.
This avoids both checking every annotation against a newly rebuilt declaration
graph and appending local evidence to an already completed declaration gate.
Closed, lifetime-free projections avoid allocating the opening/restoration folds.

Original associated RHS occurrences now detect unconstrained impl lifetimes even
when projection normalization erases them. Function binding-time classification
uses constraining original inputs and all original outputs; late instantiation
separately uses the normalized signature's live slots. This preserves explicit
early lifetime arguments without inventing slots in erased executable shapes.

The 18 previously declared projection-region cases now match pinned rustc.
`projection_lifetimes` contains 39 independently qualified source cases and a
24-compilation semantic determinism control. Five typed controls check binder
depth, simultaneous restoration, unknown identities, allocation limits and the
closed-query shortcut. Three new native programs execute reference results,
permuted tuple arguments and captured function-pointer results through CPPGM.

The determinism control exposed randomized assumption traversal during region
substitution. Both predicate and sized-assumption keys now have a stable typed
visit order before allocation. The independently accepted, structurally distinct
Copy-bound witness produced two semantic byte streams across 24 pre-fix runs.
An earlier probe used overlapping Copy bounds rejected by rustc; its original
source and stderr remain recorded as a separate compatibility issue, not as a
positive test or benchmark.

The full debug run passed 969 tests with 32 native tests ignored before the final
shortcut and deterministic-order refinements. The final optimized affected run
passed 208 tests, explicitly including native cases. All 4,857 active course cases,
Clippy all/pedantic, rustfmt, the 599-file audit and 27 harness tests passed on the
final implementation. Course fixtures, manifests and oracle bytes are unchanged.

PA20 and PA21 remain planned. Conditional ordinary aliases still need deferred
proof templates at their uses: rustc accepts the recorded unused alias definitions
but rejects uses missing the impl's outlives relation. RustGM currently rejects
such definitions conservatively. Signature/header environment dependencies,
symbolic projection equalities, partial pointer annotation provenance, remaining
associated-item contracts and advanced generic relationships also remain required.
The next implementation notes and independent alias controls are retained in
`obj/pa20-work/scoped-projections-v1/`.

The immutable final binary is
`e88890fc655d28db097f92ff1f44587affbe63b5e4d86d390bd801af6fe4fc58`;
602 source/config/test files are archived. Twenty-four existing workloads match
published full/count semantic bytes. Fifteen independently qualified new inputs
cover scoped width, repeated uses, alias chains, and distinct caller-environment
width/repetition at 128, 512 and 2,048. All frozen hashes and full/count outputs
were verified before and after A/A calibration and five paired ABBA blocks.

The first run retains 1,014 observations; six confirmation controls retain another
156. Initial ordinary paired medians range from -18.1% to +4.6%. The four repeated
ordinary controls range from -3.6% to +1.5%; the large pointer-control negative
moves from -18.1% to +0.4%, and the largest chain positive moves from +4.6% to
+1.5%. Host variability also appears in identical-binary controls, so these
measurements establish neither a general speedup nor the absence of small
regressions. Every observation is retained; no local build, test or unrelated
probe overlaps either timing run.

Initial same-binary scoped width medians are 13.71/41.77/133.14 ms, repeated-use
medians 14.07/34.15/125.78 ms, and chain medians 14.75/40.67/137.16 ms. Distinct
environment widths take 11.13/24.98/88.96 ms and repeated environments take
13.38/31.14/125.55 ms. Peak resident medians across these new inputs range from
6,450 to 15,618 KiB. Interpret the scaling observations with the recorded timing
noise rather than as a precise throughput guarantee. The validation receipt is
`obj/pa20-work/scoped-projections-v1/validation-v1.json`.

## Conditional alias lifetime contracts

Ordinary aliases now defer conditional projection lifetime requirements to their
uses. Generic defaults retain separate contracts, instantiated only when selected;
an explicit override does not acquire the unused default's requirements. Alias
bounds do not grant those requirements to callers, and associated definitions
continue to prove their contracts even when unused.

The implementation stores a sparse, sealed dependency graph. Dependencies survive
an enclosing projection that erases their result. Instantiation shares identical
(contract, arguments) pairs within the actual declaration binder or body, retains
all selection evidence, and allocates disjoint proof-private lifetime intervals
before substituting caller arguments. A failed expansion poisons its proof scope.
Typed dependency and argument work is bounded; alias chains do not copy their
entire transitive requirement set into each declaration.

Nested function-pointer annotations inherit their enclosing alias/default proof
owner. Proofs without references to intervening local binders can be lowered into
that template. Requirements involving local universal lifetimes still need an
explicit quantified proof representation and remain conservatively rejected.

The source matrix contains 39 independently qualified cases: 23 accepted and 16
rejected. Every negative candidate reaches the intended lifetime failure through
LowIR validation. Coverage includes unused conditional aliases, signature/local
uses, argument permutations, erased results/dependencies, nested pointers, alias
and nominal defaults, explicit overrides, distinct scopes, and associated-definition
conformance. Three new native programs exercise reference chains, selected defaults
and captured function pointers through CPPGM.

Typed controls exercise a 512-level diamond dependency graph, per-scope and
per-argument sharing, overlapping numeric caller/template identities, incomplete
or sealed graph mutations, failed-scope reuse, binder preservation and allocator
limits. The complete debug suite passes 976 tests with 33 native tests ignored;
the optimized affected run passes 88 tests with native cases explicitly included.
All 4,857 course cases, Clippy all/pedantic, rustfmt, the 604-file audit and 27
harness tests pass. Course fixtures, manifests and oracle bytes are unchanged.

Partial conditional aliases/defaults remain an explicit implementation boundary.
Independent experiments showed that eagerly applying normalized alias templates
can incorrectly infer projection inputs: three hypothesized positive cases are
rejected by pinned rustc with E0283. Their original declarations and stderr remain
recorded. The incomplete inferred-root experiment is archived, and the compiler
requires delayed projection normalization rather than publishing those results.
Some valid partial default uses therefore remain unsupported. Complete caller
environments, quantified proofs, symbolic equalities and the remaining PA20/PA21
contracts are still required; both assignments remain planned.

The immutable candidate is
`568d30eb3c019dd190a3c40e0c425c5565572f0ac71626405c326a216c60c3f2`;
607 source/config/test files are archived. Thirty existing workloads preserve
published full/count output bytes. Nine new, independently rustc-qualified
workloads measure conditional alias chains, repeated uses and widths at
128/512/2,048. Both timing runs verify frozen hashes and output equality before
and after measurement, with no overlapping local builds, tests or frontend probes.

A/A calibration and five ABBA blocks per workload retain 1,014 observations;
six confirmation controls retain another 156. Initial existing-workload paired
medians range from -4.8% to +3.7%. The four repeated comparisons range from -2.5%
to +1.1%; the largest initial negative moves to -0.3%, and both repeated scoped
chain positives change sign. Calibration and identical-binary controls also show
variability. These results establish neither a general speedup nor the absence
of small regressions; every observation is retained.

Initial candidate chain medians are 5.04/8.64/27.88 ms, repeated-use medians are
4.90/7.72/19.11 ms, and width medians are 9.27/25.32/91.32 ms. Their peak resident
medians range from 5,746 to 18,420 KiB. The source-level scaling controls and typed
diamond test cover sharing without claiming precise throughput from noisy wall
times. The receipt is `obj/pa20-work/alias-obligations-v1/validation-v1.json`;
remaining prerequisites are recorded beside it in `next-implementation.md`.

## Quantified alias lifetime requirements

Conditional alias/default contracts now retain nested universal binder scopes.
An unused alias may keep an unproved requirement; each actual use must discharge
it with distinct private witnesses beneath the appropriate universal binder.
Sibling binders cannot share witnesses. Caller inference variables remain outside
younger binders, and an enclosing projection or zero-length array cannot erase a
requirement that was needed to form its argument.

The shared typed lifetime evidence contains a flat, topologically ordered scope
plan and proof blocks with disjoint fresh intervals. Combining independent plans
remaps their scope IDs. Source binders acquire their final slot counts before a
contract is sealed; empty binders retain coordinates and consume bounded work.
Each source scope is cached after discovery, and packed ancestor jumps resolve
captured lifetimes without repeatedly walking a deep parent chain. Ordinary flat
proofs retain the allocation-free quantifier path.

Body collection, generic specialization and selected-item rewriting preserve the
structured evidence. Parameter substitution skips each block's local binder
prefix. The ordinary lifetime gate proves quantified requirements together with
caller constraints, preserving both body inference and diagnostic source identity.
Associated definitions still prove their own requirements even when unused;
ordinary alias bounds do not grant assumptions to an alias use.

The independently qualified source matrix covers 48 programs, including nested
captures, selected and overridden defaults, erased results, generic type-outlives
requirements, local lifetime inference and associated-definition conformance.
Its negative checks require the intended unsatisfied lifetime diagnostic.
Four native programs exercise private witnesses, generic Copy/type-outlives
requirements, erased results and trait dispatch after a quantified local alias.
Typed controls include sibling isolation, scope remapping, incomplete contract
rejection, numeric region overlap, empty-binder budgets and exact ancestor lookup
through 4,096 nested binders.

PA20 and PA21 remain planned. Delayed projection inference, complete declaration
environment dependencies, symbolic associated-type equalities, direct nested local
annotation scopes and the remaining advanced generic contracts are still required.
These changes do not promote a partial assignment or regenerate course oracles.

The full debug suite passes 986 tests with 34 native tests explicitly ignored.
The optimized affected suite passes 54 test functions with native cases included.
The final 48-case source matrix contains 30 accepted and 18 rejected programs;
every rejection reaches its intended lifetime solver failure. All 4,857 active
course cases, Clippy all/pedantic, rustfmt, the 612-file audit and 27 harness tests
pass. Course fixtures, manifests and oracle bytes are unchanged.

The final binary is
`fb0c25fedad24b1ec54307874f10d32ce31e5a81e037a922fb84619f053d2862`;
615 source/config/test files are archived. Thirty-nine existing workloads preserve
published full/count output bytes. Twelve new inputs cover quantified chains,
repeated uses, distinct widths and body proofs at 128/512/2,048. Three source-depth
inputs at 16/32/64 compare the initial implementation with its source-scope cache
and ancestor lookup refinements. An independently accepted depth-128 input exceeds
the frontend's existing parser nesting limit; its rejection is retained separately
and is excluded from successful timing. The 4,096-level typed control bypasses
source parsing to test the lifetime scope machinery directly.

A/A calibration and five ABBA blocks retain 1,404 observations; eight confirmation
controls retain another 208. Full/count outputs and frozen source, binary and
input hashes agree throughout both runs. No local RustGM build, test or frontend
probe overlaps timing. Existing-workload paired medians initially range from
-3.5% to +5.0%. The pointer-binder increases at 128/512/2,048 fall from
+1.7/+5.0/+2.2% to +0.9/+0.8/+0.6% in confirmation; the large conditional-width
comparison moves from -3.5% to -3.0%. Calibration still shows host variability.
These results establish neither a general speedup nor the absence of small
regressions.

Initial quantified chain medians are 10.04/9.17/25.00 ms; confirmation of the
nonmonotonic first two points gives 5.16/8.98 ms with identical candidate binaries.
Repeated-use medians are 4.97/7.57/18.37 ms, width medians 7.56/17.99/63.67 ms,
and body-proof medians 8.28/20.73/71.17 ms. New workload resident medians range
from 5,744 to 19,844 KiB. Depth-16 and depth-64 refinements measure +2.5/-3.8%
initially and +1.6/-2.6% in confirmation; startup and host noise limit conclusions
at these small source sizes. Every original observation and the excluded parser
boundary are retained. The receipt is
`obj/pa20-work/quantified-aliases-v1/validation-v2.json`; remaining implementation
work and the newly qualified declaration-environment probes are recorded beside it.

## Declaration environments for projection selection

Function signature projections now suspend until their owning parameter
environment is ready. Method signatures include inherited requirements; the
same dependency applies to lifetime-only goals. Nominal defaults demand the
existing nominal contract, preserving required owner bounds even when a default
is explicitly overridden. Ordinary alias bounds still grant no permissions.

Bound subjects retain typed projection identities while the complete bound list
is elaborated. Normalization uses a private immutable assumption snapshot, then
rebuilds the predicates and publishes the final environment. This supports either
written order of a projected bound and its selection requirements without
speculatively selecting an implementation from an incomplete environment.
Ordinary bounds keep their existing path, and typed source facts are reused.

Normalization preserves original predicates for two separate decisions. A bound
such as `<T as A>::Out: B` can normalize to `u8: B` and remain a conditional generic
requirement: pinned rustc accepts the unused generic definition even when `u8`
has no implementation of `B`. Calls must still prove the requirement. The initial
hypothesized rejection, its rustc diagnostic outcome, and the corrected positive
control are retained. Written closed bounds continue to require declaration-time
proof even if the surrounding function has an unrelated generic parameter.

Original bound lifetime occurrences also determine early binding. Diagnostic
review caught an explicit-lifetime call being rejected for the wrong reason after
normalization erased its bound's lifetime. Binding-time classification now retains
those occurrences, with a positive explicit-lifetime call and a negative control
that must reach its actual Copy requirement.

The final source matrix covers 47 independently qualified programs: signatures,
method inheritance, supertraits, complete trait arguments, bound order, composite
subjects, default selection/override, lifetime-only requirements, inference input
preservation and declaration/use rejection controls. Six new native programs
exercise projected signatures, method inheritance, normalized bound subjects,
selected nominal defaults and lifetime-only assumptions through CPPGM.

PA20 and PA21 remain planned. Conditional ordinary aliases still need deferred
trait normalization; projected trait arguments and other declaration environments
need the corresponding dependency support. Symbolic associated-type equalities,
delayed inference, direct nested local annotation proof scopes and the remaining
advanced generic relationships are still required. This work does not promote
partial assignment suites or regenerate course oracles.

The final debug suite passes 987 tests with 35 native tests explicitly ignored;
the final optimized affected suite passes 197 tests with native cases included.
An earlier complete optimized run also passed 987 tests, before the final early
lifetime binding correction. The final independently qualified matrix contains
32 accepted and 15 rejected programs. All 4,857 active course cases, Clippy
all/pedantic, rustfmt, the 615-file audit and 27 harness tests pass. Course
fixtures, manifests and oracle bytes are unchanged.

The frozen candidate is
`7ad7133cb6fc03dab9f88e650a8856c8d27b80e792db7aec2eaa1b3b98424d4f`;
618 source/config/test files are archived. Fifty-four existing workloads preserve
published full/count output bytes. Twelve new, independently rustc-qualified
inputs cover signature width, repeated signatures, projected bound subjects and
nominal defaults at 128/512/2,048. A/A calibration and five ABBA blocks retain
1,716 observations; nine confirmation workloads retain another 234. Frozen
source, binary and input hashes and output equality are checked throughout.
No local RustGM build, test or frontend probe overlaps either timing run.

Existing-workload paired medians initially range from -12.7% to +2.5%; four
confirmation comparisons range from -0.1% to +0.8%. The initial -12.7% body-proof
comparison has 63.2% calibration spread and moves to +0.8% in confirmation.
An identical-binary signature-width confirmation measures -23.9%, independently
demonstrating substantial host variability. These results establish neither a
general speedup, precise throughput nor the absence of small regressions.

Initial new-input medians are 16.96/51.96/198.76 ms for signature width,
10.82/27.07/90.26 ms for repeated signatures, 18.21/32.13/194.27 ms for bound
subjects and 11.29/35.69/172.57 ms for nominal defaults. Confirmation bound-subject
medians are 11.88/47.53/224.65 ms; the largest nominal-default and signature-width
inputs measure 118.77 and 116.17 ms. Initial resident medians span 5,872 to
22,068 KiB. All original observations are retained without treating these noisy
source-level controls as a complexity proof. The receipt is
`obj/pa20-work/projection-environments-v1/validation-v1.json`; remaining work is
recorded beside it in `next-implementation.md`.

## Projections in complete trait arguments

Function bound environments now retain projections in trait type arguments as
well as subjects. Typed roots are collected using the declared argument kinds,
preserving mixed lifetime/type/const applications. Nominal and ordinary alias
wrappers also wait for the raw assumption snapshot. Completed roots rebuild the
trait applications before the final environment and implied supertraits are
published. Genericity and early lifetime binding retain the original complete
predicate even when only an argument contains the erased parameter or lifetime.

The source controls include both bound orders, composite and zero-length-array
arguments, explicit lifetime calls, owner isolation, declaration/use failures and
method inheritance. Negative controls require their intended semantic diagnostic.
Native programs exercise generic trait dispatch, inherited method requirements,
normalized supertraits and mixed lifetime/type/const arguments through CPPGM.

One initial hypothesis about a projected bound depending on another projected
assumption was independently disproved: pinned rustc rejects it with E0277.
The original expectation and diagnostics are retained. RustGM currently reaches
an unsupported raw-projection selection path for that boundary, so it is excluded
from the passing language matrix. Symbolic equality support and nested callable
bound scopes remain required. PA20 and PA21 remain planned.

The complete optimized suite passes 988 tests with 36 native tests explicitly
ignored. The affected optimized run includes ten native programs across the two
projection-environment test modules, including four new programs. The final
source matrix has 34 independently qualified cases: 25 accepted and nine rejected
through LowIR, with intended diagnostics checked for every rejection. All 4,857
active course cases, Clippy all/pedantic, rustfmt, the 616-file audit and 27 harness
tests pass. Course fixtures, manifests and oracle bytes are unchanged.

The immutable candidate is
`6f91dd94db2ea361cc901026c37a387bce249d232d4822257a4e8060ab775cc7`;
619 source/config/test files are archived. Sixty-six existing workloads preserve
published full/count output bytes. Nine new, independently rustc-qualified inputs
exercise projected argument width, repeated bounds and nominal wrappers at
128/512/2,048. Remaining work and independently qualified alias boundaries are
recorded in `obj/pa20-work/projected-bound-arguments-v1/next-implementation.md`.

A/A calibration and five ABBA blocks per workload retain 1,950 observations;
seven confirmation workloads retain another 182. Both runs verify full/count
output equality and frozen source, binary and input hashes. No local RustGM
build, test or frontend probe overlaps either run. Initial existing-workload
paired medians range from -4.9% to +3.1%. The +3.1% conditional-width comparison
moves to -0.7% in confirmation, and the +1.2% bound-subject comparison also moves
to -0.7%. Confirmation includes a -14.3% depth comparison, while absolute times
and calibration show substantial host variability. These measurements establish
neither a general speedup nor the absence of small regressions.

Initial new-input medians are 12.13/36.83/139.37 ms for argument width,
7.61/18.71/64.11 ms for repeated bounds and 13.20/42.02/151.60 ms for wrappers.
Resident medians range from 5,920 to 23,496 KiB. The identical-binary width-512
confirmation measures 62.81 ms with 44.8% calibration spread, further limiting
throughput conclusions. All observations remain recorded; the receipt is
`obj/pa20-work/projected-bound-arguments-v1/validation-v1.json`.

## Deferred alias formation and erased Sized requirements

Ordinary aliases now retain unproved associated type formation until an actual
use supplies its complete environment. Dependent defaults use the same typed
contract graph; overriding a default avoids its requirements. An empty lifetime
proof no longer discards a projection requirement, including one whose result
an enclosing alias erases. Requirement-only normalization does not overwrite the
use's published annotation. Alias-written bounds remain unenforced and cannot
supply proof assumptions. Registered derive contracts permit early declaration
selection, with every selected derive still checked by the final nominal gate.

Independent checking corrected two initial expectations: pinned rustc accepts
unused dependent nominal and trait defaults. Closed invalid nominal/trait
defaults still reject; ordinary aliases can defer closed projection formation.
The original expectations and diagnostics are preserved beside the corrected
matrix. Associated definitions and missing associated members retain their
mandatory declaration checks.

A separate negative control exposed an existing acceptance bug: eager alias
normalization could erase an implementation's implicit Sized premise and accept
`Alias<str>`. Selected implementation evidence now retains these premises,
including recursively selected conditions. Alias uses substitute and prove them
in their actual environment. Typed tests check argument-sensitive DAG sharing,
sealed contracts, independent scopes and total callable depth across dependencies.
Partial aliases with only liftable Sized premises retain ordinary body inference
obligations, preserving valid inferred pairs while rejecting unsized arguments.
An erased, unconstrained input still requires an annotation; associated results
never determine noninjective inputs.

Bounds also preserve their original complete predicates through erased aliases.
This keeps projected generic bounds from being misclassified as closed and
preserves early lifetime arguments after normalization. The new source matrix
contains 62 independently rustc-qualified cases: 36 accepted and 26 rejected
through LowIR, with intended semantic diagnostics required for negatives.
Seven new native programs cover inference, nested function pointers, conditional
signatures, erased results, defaults and const arguments.

General alias/default projection inference and lifetime inference remain pending,
as does normalization of residual projections beneath callable scopes. Two
additional scoped probes require unsized-tail aggregates outside the current
profile and are excluded from the supported language matrix. PA20 and PA21
remain planned. The remaining implementation boundaries and original probe
records are in `obj/pa20-work/deferred-alias-projections-v1/next-implementation.md`.

The final optimized suite passes 992 tests with 37 native tests explicitly
ignored. The affected optimized run passes 50 tests, including 29 native programs
across seven modules. All 4,857 active course cases, Clippy all/pedantic, rustfmt,
the 620-file audit and 27 harness tests pass. Course fixtures, manifests and oracle
bytes are unchanged. The final candidate preserves every full LowIR output,
diagnostic and status in the independently qualified 62-case matrix.

The frozen final binary is
`9c0869c3abb5747468246f65188444c0d5ccdcd43a0f37954ca4a2b074c4750f`;
623 source/config/test files are archived. Seventy-five published workloads keep
their full/count output bytes. Twelve new independently qualified inputs cover
conditional alias width, repeated uses, erased dependencies and inferred Sized
premises at 128/512/2,048. Every timing run checks frozen source, binary and input
hashes and output equality, with no overlapping local RustGM build, test or probe.
All A/A calibration and paired ABBA observations remain retained.

Initial measurements and confirmation retain 2,470 observations and identify a
repeatable cost in aliases that erase generic inputs: confirmation measures
+10.1/+13.4/+18.8% at 128/512/2,048. Investigation found parameter-only environments
unnecessarily cloning and substituting lifetime state. They now preserve their
immutable parameter environment directly when the projection has no surrounding
lifetime occurrences; environments containing composite or trait predicates still
use the checked mapping.

Final measurements and confirmation retain another 2,600 observations, including
four direct comparisons with the first candidate. The optimization reduces the
512/2,048 alias-width costs by 3.2/5.4% in confirmation. It does not eliminate the
cost of retaining previously discarded requirements: relative to published main,
these alias-width inputs still measure +8.3/+9.9/+14.2%. The 2,048 input has 35.4%
calibration spread; the 512 input has 0.6%. Other positive confirmation comparisons
range from +2.3% to +7.8%, with substantial calibration noise in the argument cases.
These results document a remaining regression, not a no-regression or general
speedup claim. General delayed normalization/inference remains a possible way to
avoid eagerly proving unused aliases while preserving all use requirements.

Final new-input medians are 15.45/48.60/202.46 ms for conditional width,
6.28/14.69/43.88 ms for repeated uses, 15.78/51.90/200.33 ms for erased dependencies
and 5.47/10.64/30.39 ms for inferred Sized premises. Resident medians range from
5,908 to 31,938 KiB. The receipt is
`obj/pa20-work/deferred-alias-projections-v1/validation-v2.json`; the first candidate,
all 5,070 observations and the corrected initial expectations remain available.

## Alias projection normalization beneath callable binders

Residual projections now normalize inside callable aliases, including nested and
sibling function pointers, captured outer lifetimes, defaults and erased results.
Each callable receives a distinct normalization frame and typed scope identity.
Parent identities and local lifetime counts accompany selection requests; depth
alone never identifies a proof universe. The iterative fold preserves empty
binders and bounds combined work across suspended frames.

Deferred formation and Sized requirements retain their actual scope IDs. Pending
queries share immutable binder plans whose identities survive moving or remapping
the original proof. Selection shifts the caller environment beneath the complete
ambient binder prefix. New evidence is appended immediately to its declaration
or body sink, reserving each fresh interval before subsequent queries. Independent
or interleaved body allocators cannot share a proof interval. Requirement-only
normalization still leaves the actual annotation intact.

The new source matrix independently agrees with pinned rustc on 29 programs:
16 accepted and 13 rejected through LowIR. Negatives require their intended
missing-implementation, unconstrained-output or outlives diagnostic. The earlier
positive hypothesis for a short captured reference was incorrect; rustc requires
that reference to outlive static. Original expectations and diagnostics remain
preserved. Two additional direct source-annotation controls agree with both the
published compiler and this candidate; they are separate from the new matrix.
Six native programs compare callable values, generic signatures, erased results,
defaults, static captures and nested captures through CPPGM against rustc.

Seven typed normalization tests include independent sibling caches, stale request
rejection, selected results introducing callables and 1,024 empty nested binders
with bounded linear fold work. A separate unit test checks immutable scope plans
across proof remapping and subsequent extension. The complete optimized suite
passes 997 tests with 38 explicit native ignores; the affected run passes 60 tests
with native execution enabled, including 35 programs. All 4,857 active course
cases, Clippy all/pedantic, formatting, the 625-file audit and 27 harness tests
pass. Course fixtures, manifests and oracle bytes remain unchanged.

General delayed alias/default inference, symbolic assumption-selected associated
values and dependent projected assumptions remain unfinished. Source-local binder
routing still requires a broader audit, as do nonfunction bound lifecycles and
complete higher-ranked bounds. These checks do not complete PA20 or PA21; both
remain planned pending their remaining implementation and course contracts.

The frozen candidate is
`18d951b2906ae592d0a0937a5fedb4273a77aee9e95baf64db56f412bf8bfc94`;
628 source/config/test files are archived. Eighty-seven published workloads retain
all full/count output bytes. Fifteen new independently qualified workloads cover
callable width, repeated uses, captured lifetimes and alias chains at
128/512/2,048, plus nested callable depth at 8/32/64. A/A calibration and paired
ABBA runs retain 2,652 observations, with 130 more in focused confirmation.
Every run checks source, binary and input hashes and output equality; no local
RustGM build, test or frontend probe overlaps timing.

Existing-workload paired medians initially range from -7.2% to +4.5%. The +4.5%
conditional-repeated and +3.0% erased-dependency comparisons move to -0.8% and
+0.8% in confirmation. Nominal defaults move from +1.4% to +3.3%, with 7.1%
calibration spread. These observations do not establish a general speedup or the
absence of small regressions. They compare against the preceding published
compiler and do not erase its already documented alias-width regression.

Initial new-input medians are 13.22/43.27/164.70 ms for width,
8.31/20.50/72.00 ms for repeated uses, 12.52/39.52/139.12 ms for captured lifetimes
and 9.10/24.03/77.35 ms for alias chains. Depth inputs measure 4.04/4.46/5.80 ms.
Resident medians range from 5,904 to 22,792 KiB. Confirmation measures the largest
chain at 83.99 ms and depth-64 at 6.75 ms; the latter has 39.0% calibration spread.
These source measurements are distinct from the typed fold's bounded-work test.
The receipt is `obj/pa20-work/callable-alias-normalization-v1/validation-v1.json`.

Eight newly declared inference probes agree with pinned rustc's expected outcomes.
They identify three accepted forms still rejected by RustGM: inferred projected
pairs, callable pairs and dependent defaults. An explicitly overridden default
already works. The four rejection probes currently stop at an unsupported-feature
diagnostic, so they are not counted as successful semantic negative controls.
Their records and implementation notes remain beside the receipt for the next
batch of work.

## Inferred projection inputs and dependency watches

The inference graph can now retain an associated application's Self, trait inputs
and family inputs while type and const arguments are unknown. Mixed argument
layouts preserve the trait/family boundary and shared variable identities through
parameter substitution, lifetime mapping and isolated candidate imports. Result
comparison still requires normalization: matching two projections never equates
their inputs. Property queries and result-region freshening cannot substitute
facts about a projection's inputs for facts about its result.

Canonical-input watches suspend on unresolved type/const leaves. Equivalence-class
unions, bindings and explicit numeric/never fallback wake only affected watches.
Binding a leaf to a compound term exposes its children without rescanning resolved
ancestors. Each completed watch is delivered once; all visits and notifications
consume the existing inference work budget. Ordinary bodies allocate no watch
storage. Readiness has no proof authority and may still produce a canonical type
containing projections.

Thirteen new typed tests cover shared Self/trait/family inputs, mixed const slots,
substitution under callable binders, noninjective result equations, speculative
isolation, unions, defaults and readiness delivery. A 512-step compound dependency
chain checks bounded linear work, and 4,096 sleeping queries check that changing
one input does not scan unrelated queries. These are work-count assertions, not
source-level timing or conformance claims.

The full optimized suite passes 1,010 tests with 38 explicit native ignores. The
affected run passes 30 tests with native execution enabled, including 35 programs
across eight modules. All 4,857 active course cases, Clippy, formatting, the 630-file
audit and 33 harness tests pass. General inferred alias/default proofs and
suspended result equations remain unconnected; the existing source annotation
gate remains. No course fixture,
manifest or oracle changes, and PA20/PA21 remain planned. The next implementation
steps and retained validation records are in
`obj/pa20-work/inferred-projection-terms-v1/`.

The frozen candidate is
`2a370446719727a69a3698a94e327b413bc7a1af14cfffff369b79da0e01ce4c`;
633 source/config/test files are archived. All 102 previously qualified workloads
retain their full/count output bytes against the preceding published compiler.
A/A calibration and five paired ABBA blocks per workload retain 2,652 observations.
Each run verifies frozen source, input and binary hashes. No local RustGM build,
test or frontend probe overlaps timing; other projects on this host remain active.

Paired medians range from -6.8% to +2.5%. The largest increase is unreduced repeated
aliases at 2,048 (+2.5%, with 2.1% calibration spread); quantified width at 2,048 is
+2.2%, with 3.5% spread. These observations do not prove a general speedup or rule
out small regressions, and do not erase the earlier documented alias-width cost.
The old candidate-only workload label on 15 inherited jobs is corrected in the
derived analysis using the actual binary digests: all 102 current jobs compare
distinct A/B binaries. Original receipts and observations remain unchanged.

Timing reuses the two pinned executable paths instead of creating a binary copy
per job. Rendering scratch files are removed after each job. Guarded validation
prunes disposable generated executables while preserving source, diagnostics,
receipts and observations. The receipt is
`obj/pa20-work/inferred-projection-terms-v1/validation-v1.json`.

## Deferred equations and partial alias normalization

Partial alias annotations now preserve projection inputs through body inference.
Ordinary sibling equations determine type/const arguments; comparisons involving
associated results suspend without equating those inputs. Normalization wakes
only the blocked leaves that changed, retaining equality versus shape semantics.
Finishing a body with unresolved equations or formation requirements fails.
A 4,096-leaf typed test bounds each single-leaf resume without rescanning siblings
or repeating their lifetime equalities. Thirteen equation tests cover ordering,
noninjectivity, defaults, chained dependencies and mismatched selected results.

Alias contracts retain their original canonical proofs and erased formation
requirements separately from inferred output terms. Parameter-use collection
identifies the actual input dependencies. Source callable scopes and alias-local
scopes retain their own binders; private proof identities are rebased before
actual arguments enter. Canonical body normalization selects only validated
implementations and retains the complete scoped lifetime evidence. Four new
proof tests cover private-ID collisions, ambient substitution, scoped interval
gaps and copying a chosen ancestor chain after 8,192 earlier sibling scopes.

Eighteen declared source cases match pinned rustc: ten accept and eight reject.
They cover inferred tuples in either field order, callable parameters, mixed
const arguments, dependent defaults and overrides, numeric fallback, captured
body lifetimes, static-only implementations and invalid lifetime escape.
The erased-inner-input rejection still reports a normalization demand, so it is
not evidence that all delayed operational demands are implemented. Six new
native programs execute the selected results and callable/lifetime paths through
CPPGM and rustc. Together with eight affected existing modules, 41 native programs
pass. The full optimized suite passes 1,028 tests with 39 explicit native ignores;
all 4,857 active course cases, Clippy all/pedantic, formatting, the 639-file audit
and 33 harness tests pass. No course fixtures, manifests or oracle bytes change.

PA20 and PA21 remain planned. Isolated inferred method prefixes still reject
pending projection proofs. Symbolic assumption-selected associated equalities,
remaining delayed operational demands, associated constants/result bounds,
shorthand paths, the remaining coherence work, GATs, opaques, complete higher-ranked
relationships and generic drop checking remain required. This checkpoint does
not promote an assignment or reduce the active goal.

Evidence and immutable source/binary snapshots are retained in
`obj/pa20-work/deferred-inference-equations-v1/`. A mistyped native test target in
the first batch was corrected to `unreduced_annotations`; the failed invocation
ran no native tests. The corrected native run and subsequent course checks pass.

The frozen candidate is
`8d8b0d661faa3f1af39a7d9c9a716c26f6276143aba21a1cd73cc55280c9dd8d`,
with 642 source/config/test files archived. All 102 previous workloads retain
identical full/count output bytes against the published `2a370446...` baseline.
Twelve new, independently rustc-qualified workloads measure repeated inferred
aliases and source callable scopes at 64/256/1,024, plus callable width and
dependent-default chains at 64/256/512. A/A calibration and five ABBA blocks per
workload retain 2,964 observations; five focused confirmations retain 130 more.
No local RustGM build, test or frontend probe overlaps timing. Other projects
remain active on this host, and every raw observation is retained.

Initial existing-workload paired medians range from -5.7% to +6.6%. Quantified
width's +6.6% has 64.4% calibration spread and becomes +0.5% in confirmation.
Quantified repeated uses move from +2.6% to +0.4%; nominal defaults, argument width
and inferred Sized controls confirm at -0.4%, +0.9% and -1.0%. Confirmation
calibration spreads are 8.9–15.2%. These measurements neither establish a general
speedup nor rule out small regressions; the earlier alias-width cost remains.

New workload median wall times are 6.82/15.17/47.90 ms for repeated inferred
aliases, 9.71/26.35/104.02 ms for repeated source callable scopes,
6.07/12.64/19.64 ms for callable width, and 4.49/6.46/8.68 ms for default chains.
Their resident medians range from 5,896 to 13,642 KiB. These timings complement
the typed bounded-work checks; they are not a proof for arbitrary programs.

Six further capture probes were independently qualified after timing. Five
match rustc, including rejection of a static-only function pointer. The remaining
accepted program puts a generic function pointer before the tuple field that
determines its projected input; RustGM still rejects it as unresolved. The
captured-lifetime transformation creates a new projection occurrence whose
normalization is not yet scheduled. Its source, expectations and diagnostics are
retained in `capture-followup-declared-v1.json` and `capture-followup-v1.json`.
That transformed-occurrence scheduling is the next body-inference fix, rather
than completed conformance coverage in this checkpoint.

## Transformed projection occurrences

Captured-lifetime mapping now registers the projected terms it creates. Alias
arguments and dependent defaults also register projections introduced through
substitution, including when the outer template itself has none. A query waits
on its projected inputs rather than the whole output aggregate, allowing an
independent output hole to be solved by the selected associated result. Repeated
registration of the same term in the same scope shares its normalization query;
the separate formation contracts remain required.

The occurrence walk preserves callable binder scopes, including empty and sibling
binders. It visits shared ordinary structure once per scope and leaves a nested
projection's inputs to its semantic query. Three typed tests cover unrelated
output holes, scope identities, 512 nested projection inputs and a 512-level
ordinary shared graph with bounded traversal work.

The expanded source matrix independently agrees with pinned rustc on 31 programs:
20 accept and 11 reject. Eight native programs exercise inferred aliases,
defaults, captures and independent output holes; the complete affected native run
executes 43 programs across nine modules. One original positive test expectation
was wrong: a type parameter used only in another parameter's default is unused
in the alias body. Rustc's E0091 rejection is preserved as a negative control;
the corrected positive also uses that parameter in the body. The initial failed
test and the expectation correction remain in the evidence directory.

The full optimized suite passes 1,031 tests with 39 explicit native ignores.
All 4,857 active course cases, Clippy all/pedantic, formatting, the 641-file audit
and 33 harness tests pass. No course fixture, manifest or oracle changes.
The earlier erased-inner-input negative still stops at a normalization demand;
its rejected status does not establish complete delayed-operation support.
PA20 and PA21 remain planned, with the remaining responsibilities listed above.

The immutable candidate is
`5aa15338213dcd21f56c063cc0be00171c03131efd66455b812656abeef8e0b5`;
644 source/config/test files are archived under
`obj/pa20-work/transformed-projection-occurrences-v1/`. Of 114 inherited
performance inputs, 105 retain identical full/count semantic output. Nine omit
unused intermediate aggregate/callable types, shifting later type identities
in the semantic dump. Their count views remain identical. The initial strict
comparison correctly failed; original dumps, diffs and the reviewed change are
retained in `output-review-v1.json`. Those nine comparisons now use full/count
LowIR, whose bytes match exactly. This is an explicit change of measurement
stage, not semantic-dump byte equality or an oracle update.

Twelve new independently rustc-qualified workloads vary repeated captured generic
callables, callable defaults, wrapped projected arguments and unrelated output
holes at 64/256/1,024. The full run retains 3,276 observations with A/A calibration
and five ABBA blocks per workload. Every run verifies source, input and binary
hashes. No local RustGM build, test or probe overlaps timing; other projects on
this host remain active. Measurements reuse the two pinned executable paths,
remove rendering scratch and retain all observations and diagnostic evidence.

Inherited-workload paired medians range from -7.2% to +5.3%. Four focused
confirmations retain another 104 observations: conditional width moves from
+5.3% to -1.0%, quantified chains from +4.6% to +2.0%, and callable repeated
uses from +3.8% to -2.0%. Conditional repeated uses remain slower, moving from
+2.9% to +3.7%, with 1.9% confirmation calibration spread. This small measured
regression remains recorded; the noisy comparisons do not prove a general
speedup or erase the earlier alias-width cost. Conditional width's confirmation
has 96.2% calibration spread and cannot establish an improvement.

New-input median wall times are 12.63/40.91/157.88 ms for captured generic
callables, 8.22/20.55/66.47 ms for callable defaults, 7.80/20.06/67.79 ms for
wrapped projections and 7.73/18.00/58.83 ms for unrelated output holes. Resident
medians range from 5,938 to 15,106 KiB. These are source-level measurements,
distinct from the typed bounded-work assertions. The receipt is
`obj/pa20-work/transformed-projection-occurrences-v1/validation-v1.json`.

After timing, fifteen further probes independently match their declared rustc
expectations. Two accepted controls also pass RustGM. Ten other accepted programs
still fail, covering projected values used before later statements constrain their
inputs, delayed result-region freshening and a projected generic function argument.
The three negative controls currently stop at an earlier normalization demand,
so their rejection is not counted as validation of the intended negative reason.
Inputs and results remain in `obj/pa20-work/delayed-projection-demands-v1/`.
These operational demands and transformed generic signatures are follow-up work,
not completed support in this checkpoint.

## Projected callable arguments and nominal fields

Generic function signature and nominal field instantiation register projected
occurrences in their resulting scopes. A query for the original argument cannot
normalize a different graph produced by substitution beneath a callable binder.
Both sites reuse the scoped occurrence walk and its query deduplication; they
do not select implementations or discharge formation requirements themselves.

Partial callable inference opens source lifetime binders only where the
expected type supplies matching callable structure. A hole for an entire
higher-ranked function-pointer type keeps that type's own binders. Sibling
callable occurrences receive independent lifetime instances; unchanged shared
subtrees are visited once. The original source type remains subject to a
separate directed subsumption proof. All traversal and substitution work is
charged to the inference budget.

Generic inference first detects binders in the current typed graph without
resolving holes or interning canonical types. Known scalar types need no worklist
allocation; shared constructed shapes use a bounded walk. Answers are not cached
across inference changes. The original broader activation eagerly resolved
ordinary types and reordered eight existing PA19 text dumps. The strict course
check caught this; the binder-presence query restores their exact bytes. No PA19
oracle is changed. Original failures remain in the validation evidence.

Thirty-four declared source controls agree with pinned rustc in the focused
maintainer checks: 24 accept and ten reject. They cover projected argument order,
nested function pointers, captured caller lifetimes, late binders, borrowed
results, constructors, nominal fields and return values, inherent methods and
qualified trait methods. Seven native programs check selected function behavior
and returned values. Five typed controls cover whole-type binder preservation,
partial shape opening, independent sibling instances, shared graph work bounds
and binder detection after new bindings without canonical type insertion.

The stronger native cases exposed a source lifetime leaking into inferred type
holes. Opening every nested binder corrected that case but incorrectly erased
whole higher-ranked types. The final shape-directed walk handles both. One
original positive hypothesis with a nested higher-ranked input is rejected by
rustc; its expectation was corrected before final validation. Earlier experimental
sources, declarations, diagnostics and measurements are retained under
`obj/pa20-work/projected-callable-arguments-v1/` and `-v2/`; the earlier experimental
binary is not the published performance baseline.

Three source regressions belong to PA18 and are now course fixtures. Pinned rustc
qualifies their two accepted cases and one lifetime rejection. The accepted
programs run value assertions through both rustc and CPPGM. Their nine sidecars
are generated only after independent checks; all 1,293 preexisting PA18 sidecars
remain byte-identical. The active PA18 suite now has 434 cases, 267 executable
comparisons and three deliberately divergent codegen-only comparisons, with its
one documented later-stage rustc acceptance difference.

Delayed projected values and operational coercions remain follow-up work.
Unconstrained inputs must not be inferred from projection results; queued
conversions must retain reborrow, two-phase and discarded-repeat metadata. The
erased-input negative here still stops at a normalization demand, so it is not
proof of the eventual unconstrained-input diagnostic. Associated equalities,
associated constants, general coherence and the remaining PA21 responsibilities
are also unfinished. PA20 and PA21 remain planned.

Final optimized validation passes 1,037 tests with 40 explicit native ignores.
The affected ten-module native batch executes 50 programs; PA18 independently
checks all 434 declarations and executes its 267 runnable cases. All 4,860 active
course cases pass. Clippy all/pedantic, formatting, the 644-file audit with zero
findings, 33 harness tests and upstream verification pass. The 34 new maintainer
source controls also agree with rustc through the CLI's complete LowIR emission.
The full suite failure before the binder-presence fix is retained separately
from this final passing run.

The frozen candidate is
`ac4a62b05d79c05c4b626f11fe93feb4ca57cb0469dea864e1b83ad2675f7a2f`,
with 647 source/config/test files archived in
`obj/pa20-work/projected-callable-arguments-v2/`. Its baseline is published commit
`477d1ba`, binary
`5aa15338213dcd21f56c063cc0be00171c03131efd66455b812656abeef8e0b5`.
All 126 inherited workloads retain identical full/count output in their previously
selected modes: 117 semantic views and nine LowIR views. No comparison mode was
changed for this checkpoint.

Twenty-one independently rustc-qualified new scaling inputs cover projected
function arguments, constructors and captured callers at 64/256/1,024, argument
width at 64/256/512, and three binder families at 64/256/1,024: projected late
inputs, whole higher-ranked arguments and ordinary partial tuple inputs. Their
candidate-only measurements establish scaling and calibration, not comparisons
against an older compiler that rejects them. Every measurement uses immutable
binaries and inputs, verifies full/count output equality, and retains raw A/A
calibration and paired ABBA observations. No RustGM build, test or probe overlaps
timing; unrelated projects remain active on this host.

The timing batch retains 3,822 observations. Inherited-workload paired medians
range from -12.6% to +7.4%. Two comparisons exceed +4%: unreduced chains at 128
(+4.8%, 19.7% A/A spread) and inferred Sized at 128 (+7.4%, 55.3% spread).
Focused confirmations retain another 52 observations and measure -2.1% with
1.3% spread and -0.9% with 10.7% spread, respectively. Those initial regressions
do not persist in the confirmations; these noisy measurements do not establish
a general speedup. Every initial observation remains in the record.

Candidate-only median wall times, in milliseconds, are:

| Workload | 64 | 256 | 1,024 (512 for width) |
| --- | ---: | ---: | ---: |
| Projected function arguments | 9.97 | 27.79 | 98.02 |
| Projected constructors | 11.01 | 29.35 | 103.08 |
| Captured projected callers | 16.91 | 57.57 | 260.47 |
| Projected argument width | 8.25 | 18.82 | 35.72 |
| Projected late-bound inputs | 17.76 | 62.65 | 275.11 |
| Whole higher-ranked arguments | 11.83 | 32.85 | 135.89 |
| Ordinary partial tuple inputs | 9.06 | 23.63 | 89.68 |

Their resident medians range from 5,960 to 19,970 KiB. Source-level timing remains
separate from typed work-bound assertions. Binaries are reused across samples;
rendering scratch and probe metadata are removed. The successful validation
wrapper pruned 3,031 generated files (2.11 GiB of unshared blocks), recorded in
`obj/cleanup/1792111513060885191.jsonl`. The superseded experimental pin is retired;
only the current comparison baseline and candidate remain pinned until publication.
The final receipt is
`obj/pa20-work/projected-callable-arguments-v2/validation-v1.json`.

## Tuple coercion order and isolated result transport

Tuple fields now finish their coercions in source order, before the next field
is entered. In `Pair<T> = (T,T)`, initializing the first field from a mutable
reference establishes reference context for the second. That second field then
reborrows, including through block tails and nested tuples. The first field still
moves when its expected type was unknown, and independent holes remain independent.
Arrays retain their existing common-type join and final coercion pass.

Seven declared source controls agree with pinned rustc: five accepted programs
and two move rejections. Four native programs verify ordinary, block-tail, nested
and projected reborrows with value assertions through rustc and CPPGM. Three
ordinary alias controls join PA18, whose 437 cases comprise 272 accepted and 165
rejected programs. The independent course run checks 269 executions and three
codegen-only comparisons. Oracle generation adds nine sidecars and preserves all
1,302 preexisting PA18 sidecar hashes. The one documented later-stage acceptance
difference remains explicit in the manifest.

The inference layer also gains isolated symbolic-result transport for upcoming
partial projection queries. A probe copies all roots with shared type and const
mappings. Export rejects input binding, domain narrowing, merging distinct input
holes, changed source equations and fresh escaping query holes before allocating
destination terms. Accepted results reuse original input identities; a concrete
result alone cannot choose an unresolved input. Lifetime equations remain in the
probe and carry no implementation-selection or formation-proof authority.
Eight typed tests cover these contracts, const slots and bounded shared graphs.
Ordinary imports do not pay the additional tracking cost. This API is not yet
connected to projection selection; partial normalization remains unfinished.

Follow-up investigation retains 36 independent source probes, including original
hypotheses and three expectation corrections, under
`obj/pa20-work/pending-projection-coercions-v1/`. Rustc rejects reversed tuple
initializers requiring an implicit function-item or shared-reference conversion,
even when a later numeric field fixes the input. Explicit conversions and fully
specified alias inputs provide separate accepted controls. Retrying all deferred
conversions after input resolution would incorrectly accept those programs.
The current implementation still needs to retain the original projected context
when a declaration template has normalized early, and to normalize suitable
partially known inputs without inferring them from output equations. Symbolic
associated equalities, associated constants, general coherence and the remaining
PA21 responsibilities are still required. PA20 and PA21 remain planned.

Final optimized validation passes 1,046 tests with 41 explicit native ignores.
The affected eleven-module native batch executes 54 programs separately; all
4,863 active course cases pass. Clippy all/pedantic, formatting, the 647-file
audit with zero findings, 33 harness tests and upstream verification pass.
The successful wrapper removes 3,044 generated files (2.12 GiB of unshared
blocks), recorded in `obj/cleanup/1792114155721374176.jsonl`.

The frozen candidate is
`99e4c8a556b48491be92f50d26ff072d5e707fd29933e7f10e05b14a3ee31974`,
with 650 source/config/test files archived under
`obj/pa20-work/partial-projection-normalization-v1/`. The comparison baseline is
published commit `948d628`, binary
`ac4a62b05d79c05c4b626f11fe93feb4ca57cb0469dea864e1b83ad2675f7a2f`.
All 147 inherited workloads retain identical full/count bytes in their existing
modes. Nine new rustc-qualified source workloads cover repeated tuple reborrows
and projected reborrows at 64/256/1,024, and wide shared-type tuples at 64/256/512.
These newly accepted inputs use candidate-only measurements. Their timing is
separate from the typed probe work-bound assertions. Every job uses immutable
binaries and inputs, A/A calibration and paired ABBA observations, with no RustGM
builds, tests or probes overlapping timing. Unrelated projects remain active on
this host.

The initial timing batch retains 4,056 observations. Inherited paired medians
range from -15.4% to +15.2%. Six comparisons exceed +4%: concrete goals at 256,
pointer binders at 512, unreduced repeated uses at 512 and 2,048, callable width
at 512, and captured projected arguments at 256. Their focused confirmation
retains another 156 observations, with respective deltas of -1.3%, -1.2%, -1.0%,
-1.0%, +0.6% and -0.9%. Confirmation A/A spreads range from 1.4% to 7.0%.
The initial regressions do not persist in this confirmation; these measurements
do not establish a general speedup. Every initial sample remains retained.

Candidate-only semantic-view median wall times, in milliseconds, are:

| Workload | 64 | 256 | 1,024 (512 for width) |
| --- | ---: | ---: | ---: |
| Repeated tuple reborrows | 6.49 | 14.81 | 50.74 |
| Shared-type tuple width | 4.42 | 6.38 | 9.49 |
| Repeated projected reborrows | 12.72 | 39.45 | 165.68 |

Their resident medians range from 5,898 to 17,212 KiB. Measurement reuses the two
pinned compiler binaries and removes large rendering scratch after each job.
The final receipt is
`obj/pa20-work/partial-projection-normalization-v1/validation-v1.json`.


## Partial projection selection and scoped input inference

A unique validated implementation header can now determine partially known
projection inputs before numeric fallback. This supplies source-time function
pointer or reference context without using an associated result to choose an
implementation. Competing headers remain deferred. The existing full selector
proves implementation premises; partial selection does not create a second
trait solver or treat a candidate header as evidence.

Each query imports its inferred input graph into an isolated context. One
lifetime opening map covers both the input and its shifted caller environment.
Selected inputs are restored before their equations reach the body. Enclosing
binder slots retain rigid identities, nested callable binders may alpha-rename,
and outer inference variables cannot capture younger universal slots. Selection
requirements remain attached to the original proof scope. Both canonical and
partial paths share associated-definition substitution and result restoration.

The body caches its projection environment after substituting established
universal aliases. A caller bound and its existing aliased copy must not become
two competing assumptions when opened. This correction accepts a matching
caller bound while preserving rejection for a bound on an unrelated lifetime;
it does not turn pending lifetime equations into permissions.

Dependency tickets now report coalesced input changes and support cancellation.
Only changed leaves are revisited. A scheduling certificate can switch a query
to canonical-only notifications when exactly one validated header preserves all
input holes and has only builtin premises. An isolated probe rejects bindings,
domain narrowing and merged holes. This certificate never supplies proof: the
ordinary normalizer must still discharge the complete implementation contract.
The certificate is checked before opening a full query's lifetime environment.
For a single independent type parameter beneath already-known reference or
pointer heads, with no trait arguments or recursive trait premises, the checked
header establishes input preservation without copying the input graph. Other
layouts retain the isolated probe, whose preservation check does not export an
unused result. The correction handles 64/256/512/1,024-field inferred tuple
inputs within the existing work budget. An earlier candidate rejected the valid 512-field case;
its failure receipt remains retained and its executable has been retired.

Output comparison found two additional scheduling defects before publication.
Queries on entirely unknown Self allocated lifetime aliases before they could
select anything. Change notifications could also enter partial selection while
their dependency counters still reflected queued updates. Unknown Self now
waits for a head or numeric domain. Canonical completion retains its existing
queue order; partial events arrive only after pending dependency updates. A
naive all-events drain changed region-allocation order and was rejected by the
same comparison. The corrected implementation matches every full output in the
159 baseline comparisons (156 inherited inputs and three new wide inputs),
including captured callable inputs and reborrows.
All intermediate mismatch receipts and observations remain available under
`obj/pa20-work/partial-projection-selection-v2/`.

The new maintainer tests include 25 source controls with declared acceptance
and rejection independently checked against pinned rustc, six native programs,
seven dependency-ticket tests and four scoped-input tests. They cover unique
and ambiguous headers, type/const inference, source-time reborrowing, caller
bounds, higher-ranked slots, capture rejection, const dependencies, coalescing,
canonical completion order and bounded work on shared graphs. The wide-input
source regression also checks complete instance construction against rustc.
These tests develop the future contracts without marking PA20 or PA21 active.

This checkpoint does not implement general symbolic associated normal forms or
resolve the 16 known contextual-coercion mismatches among 43 retained follow-up
controls. Input-preserving symbolic transport must retain original formation
obligations even when a result erases its inputs. Symbolic associated equalities,
associated constants, general coherence and the remaining PA21 variance,
higher-ranked, GAT, opaque-type and generic-drop responsibilities remain open.
PA18 and PA19 remain implemented drafts; PA20 and PA21 remain planned.


Optimized validation passes 1,059 maintainer tests, with 42 backend-dependent
tests explicitly ignored there and all 42 executed successfully in the native
run. All 4,863 active course cases retain their existing text oracles. Formatting,
Clippy all/pedantic, the 652-file audit with zero findings, 33 harness tests,
backend qualification and upstream verification pass. The 43 retained follow-up
controls still have the same 16 mismatches as the published baseline; none is
counted as a passing language control. The validation wrapper removes 4,124
files and 4.36 GiB of unshared blocks, recorded in
`obj/cleanup/1792122005325536086.jsonl`.

The validated compiler is
`feb02e8ce952f95cb603ee3139963dc837831e8b97a1291cbf213b4c39bb9fcf`,
with 655 source/config/test files frozen under
`obj/pa20-work/partial-projection-selection-v2/source-v5/`. The comparison
baseline is published commit `e2fcd9f`, binary
`99e4c8a556b48491be92f50d26ff072d5e707fd29933e7f10e05b14a3ee31974`.

The initial scheduling version retained 2,964 timing observations before full
output comparison stopped it. Its corrected successor retained 4,446 samples
and 208 confirmation samples, exposing persistent 11–15% overhead for repeated
inferred callable inputs and 8–10% for wider partial tuples. Both intermediate
optimization experiments retain another 208 samples each. Moving the certificate
before lifetime opening, avoiding unused result export, and recognizing the
checked independent-parameter header remove unnecessary query preparation.
The rejected candidates and every original observation remain documented;
superseded executables are retired rather than accumulating under storage pins.


The final batch retains 4,446 observations across 171 workloads: 156 inherited
inputs and three new wide inputs compare the published baseline with the
candidate, while twelve newly accepted inputs use candidate-only measurements.
All jobs use frozen binaries and inputs, A/A calibration and paired ABBA blocks;
full/count output bytes agree, and frozen source/binary/input hashes remain
unchanged. No RustGM build, test or probe overlaps timing. Unrelated projects remain active on the host.

The 159 comparison paired medians range from -16.5% to +3.0%. None exceeds the
+4% confirmation threshold; the largest increase has 4.2% A/A spread. These
measurements do not establish a general speedup. All observations from the
stopped and rejected candidates remain in the record.

Candidate-only semantic-view median wall times, in milliseconds, are:

| Workload | 64 | 256 | 1,024 |
| --- | ---: | ---: | ---: |
| Unique concrete headers | 10.23 | 29.03 | 100.23 |
| Unique reference reborrows | 13.52 | 41.12 | 158.27 |
| Unique higher-ranked inputs | 11.33 | 32.45 | 118.47 |
| Unique headers with caller bounds | 16.99 | 55.29 | 211.57 |

Their resident medians range from 6,086 to 20,252 KiB. Timing remains separate
from the typed work-bound assertions and independent source/native checks.
Measurement reuses the two current compiler binaries and removes rendering
scratch. The final receipt is
`obj/pa20-work/partial-projection-selection-v2/validation-v5.json`.

## Symbolic associated results with retained input proofs

A unique compatible implementation header can now expose its associated result
while original type or const inputs remain unresolved. Header matching uses an
isolated graph, counts every compatible alternative and exports only equations
that preserve the original inputs. A result cannot choose between competing
implementations. The original application survives publication with shared input
identities; ordinary selection must eventually prove all implementation and
formation requirements and agree with the published result. Output completion
alone cannot discharge those obligations, including erased const inputs.

Partial alias annotations now use their retained unreduced definitions. This
preserves source-time reference, function-pointer and array conversion context.
Unresolved projections defer result equality without retroactively introducing
an implicit conversion or reborrow. Scoped equations intersect numeric variable
domains in either argument order. The former 16 contextual-coercion mismatches
among 43 retained controls now agree with pinned rustc.

Nested projection inputs register their own normalization queries and notify only
dependent applications. Inner publication may erase holes from an outer input,
while the inner query still owns the original proof. Cached callable scopes retain
empty frames and ambient lifetime identities across retries. This fixes a concrete
outer-header regression found by independent comparison during development;
its original failure evidence remains retained.

Repeated canonical queries share completed normal forms within the body's
immutable environment when they have no ambient/free lifetimes and leave no
lifetime obligations. Only successful complete selections enter that cache.
A single independent header parameter beneath exact known reference/pointer
heads can also reuse the original input term and actual captures directly.
Repeated captures must agree and every capture must be supplied; other headers
retain the isolated probe. Readiness and symbolic publication share that header
match, while original implementation premises still require the full selector.

The maintainer tests independently qualify 80 source outcomes with pinned rustc
and execute eight new native programs through both compilers. Additional typed
controls verify publication identity, cyclic-result rejection, erased const
requirements, integer/float domain merging and coalesced dependency notifications.
The optimized suite passes 1,066 tests; all 43 explicitly backend-dependent groups
also pass. All 4,864 course cases pass. The new PA18 ordinary-alias fixture preserves all
1,311 preexisting PA18 sidecar hashes; no existing course oracle changes.
Formatting, Clippy all/pedantic, file and architecture audits, harness checks,
backend qualification and upstream checks pass on the frozen candidate.

The validated compiler is
`9436423ce79b1be47eafb4a472ee6e9d0a728d35af4c4033af2adba4f95ef49b`,
with 659 source/config/test files frozen under
`obj/pa20-work/symbolic-projection-integration-v1/source-v5/`. Its comparison
baseline is published commit `0fbaba0`, binary
`feb02e8ce952f95cb603ee3139963dc837831e8b97a1291cbf213b4c39bb9fcf`.

PA20 and PA21 remain planned. General symbolic associated equalities, associated
constants and coherence, followed by the remaining variance, higher-ranked,
GAT, opaque-type and generic-drop responsibilities, still require implementation
and complete grading contracts.

The initial candidate retained 4,836 timing observations over 186 jobs and
exposed about 105% overhead at the largest repeated inferred-alias scale. A
234-observation focused run measured the first normal-form cache improvement.
The final candidate retains another 4,836 observations over the same jobs, followed
by 468 observations confirming all increases above 4% and the related inferred
callable/wide-input controls. Every observation and rejected prototype receipt is
retained; superseded candidate binaries have been retired.

Of 171 inherited A/B inputs, 168 preserve full and count output bytes. The three
inferred-Sized inputs retain one additional canonical projection type because
the original alias application now participates in formation proof; the immediately
following tuple ID shifts by one. An exact structural diff check permits only
that addition, and count bytes remain identical. Adding an ordinary entry calling
the measured function gives independently rustc-qualified programs with identical
full/count executable LowIR from both compilers. The three semantic output changes
are recorded explicitly in `output-review-v5.json`, not silently normalized away.

Fifteen additional workloads independently qualify with pinned rustc and reject
in the published baseline. They therefore use candidate-only measurements. All
jobs use frozen binaries and sources, A/A calibration and paired ABBA blocks;
full output hashes are checked before and after timing, count outputs agree, and
source/binary/input hashes remain unchanged. No RustGM builds, tests or probes
overlap timing. Unrelated projects remain active on this host.

The final confirmation leaves repeated inferred-alias overhead of 11.8%, 26.0%
and 33.8% at 128/512/2,048 uses, with candidate medians of 6.45/13.51/41.60 ms.
These annotations now retain source-time projection context and their original
queries. The remaining overhead is an optimization target. Repeated inferred
callable inputs show 4.7%, 3.6% and 4.3% at 64/256/1,024 uses, with medians of
11.00/30.17/117.42 ms. All other confirmation deltas are at most +1.5%; the wide
partial-input cases range from -5.9% to -1.3%. These observations do not establish
a general speedup. Calibration spreads and every paired block remain in the
receipts, including noisy initial observations.

Candidate-only semantic-view median wall times, in milliseconds, are:

| Workload | 64 | 256 | 1,024 (512 for width) |
| --- | ---: | ---: | ---: |
| Later projection inputs | 8.85 | 23.57 | 84.15 |
| Shared result identity | 11.03 | 29.58 | 105.78 |
| Shared projection width | 5.22 | 9.49 | 15.66 |
| Nested projected inputs | 14.17 | 44.31 | 162.86 |
| Nested callable inputs | 21.57 | 74.20 | 299.37 |

Resident medians range from 5,884 to 19,932 KiB. The final correctness receipt is
`obj/pa20-work/symbolic-projection-integration-v1/validation-final-v5.json`; timing
and confirmation summaries are `analysis-v5.json` and `analysis-confirm-v5.json`
in the same directory. Cleanup after the validation batches removes 10,422
generated files and 11.70 GiB of unshared blocks, recorded in
`obj/cleanup/1792128822961449804.jsonl`.

The next associated-equality investigation independently qualifies 17 declared
source outcomes with pinned rustc. Nine accepted programs remain unimplemented:
opaque projected identity, explicit associated equalities in inline/where bounds,
projected Copy assumptions, distinct trait arguments, shorthand associated paths,
default methods, inherited supertrait equalities and unused contradictory bounds.
The eight rejection controls also reject in the current compiler, but its early
boundary diagnostics do not establish the future solver's correctness. In
particular, pinned rustc accepts an unused contradictory associated-type bound;
an actual call must still satisfy it. The original incorrect rejection hypothesis
and its correction are retained under
`obj/pa20-work/symbolic-associated-equalities-v1/`. These exploratory controls are
not included in the active assignment or maintainer pass counts.


### Typed associated equality obligations

PA20 now has a canonical projection-equality predicate and an inferred result
term that shares the subject's parameter and const graph. Parameter/const uses,
free-lifetime and binder-capture queries, canonical substitution, inferred
instantiation, probe import and completion all retain both roots. A normalizer
may replace a root with its proved normal form without losing the remaining
equality. Result equations do not infer unrelated projection inputs.

Immutable environments retain these equalities separately from builtin abilities
and trait membership. They preserve competing results, deduplicate identical
assumptions, reject parameters or constants outside the actual scope, and open
result-only lifetime captures with the same mapping as the rest of the caller.
No equality grants Sized, Copy or Clone by falling through the builtin path.
Candidate imports preserve the whole obligation, and unsupported proof paths
reject explicitly. Both readiness paths exclude equality premises from the
builtin-only scheduling certificate.

Seven integration controls and one environment unit control cover these
invariants, including a const used only on the right, independent input/result
variables, a pending result equation, isolated probes and nested callable binders.
They exercise the typed interfaces directly. The earlier 17 rustc-qualified
source cases remain pending: this checkpoint does not yet elaborate source
associated bindings or solve them. PA20 and PA21 remain planned, with their full
contracts and implementation still required. Existing course fixtures and text
oracles are unchanged.

The implementation and validation artifacts are retained under
`obj/pa20-work/typed-projection-obligations-v1/`. Initial build, lint and audit
failures remain recorded: adding the enum variant required explicit consumers;
method applicability now delegates its builtin proof to a cohesive helper; the
new integration file is `equality_predicates.rs`, keeping the projection filename
family within the existing placement limit. No lint or audit suppression was
added.

The frozen candidate is
`63c353c3db2e07f88eb7ba706f842f919d514f58c28603a7e246cfaf4cfff096`,
with 660 source/config/test files retained beside it. All 28 validation commands
pass: formatting, strict Clippy, the eight targeted controls, the 657-file audit
with zero findings, 33 harness tests and assignment discovery, all 4,864 course
cases, 1,074 release tests, backend qualification and all 43 explicitly native
groups. The default release run reports those 43 backend groups as ignored;
the separate native run executes all of them successfully. Receipts are
`validation-v2.json` and `checkpoint.json`; the initial placement failure remains
in `validation.json` and its log.

Compared with published `d17ab0c`, 36 jobs retain 936 observations over eighteen
frozen inputs: concrete goals, bound subjects, receiver selection, inferred
aliases, repeated inference and nested callable projections, each at three
scales and in full/count modes. Every job includes A/A calibration and five ABBA
blocks. Full and count outputs match byte-for-byte; source, input and binary
hashes are checked before and after measurement. Median paired wall-time deltas
range from -3.48% to +2.88%; this is not evidence of a general speedup. These
measurements cover the existing paths changed by the representation and proof
routing; source equality solving still needs its own workloads when integrated.
The complete observations and summaries are in `measurements/`,
`perf-results.json` and `performance-summary.json`. No RustGM builds or tests
overlap timing. Validation cleanup removes 4,188 generated files and 4.39 GiB of
unshared blocks, recorded in `obj/cleanup/1792131801352781286.jsonl`.
Resident medians differ by -22 to +756 KiB across these pairs. A/A wall-time
spreads range from 0.86% to 78.00% on the shared host; individual noisy samples
and all block ratios remain in the receipts. These measurements do not establish
unchanged memory use or a strict zero-regression performance guarantee.


### Source associated equality bindings

The maintainer now elaborates equality bindings such as `T: Trait<Out = U>`
and the corresponding where-clause forms into separate typed predicates. Member
names retain resolved identities; positional arguments, defaults and raw names
keep their existing rules. Ordinary trait headers are completed before inherited
binding lookup, and all equality requirements are published before environments
freeze. A direct member of the named trait takes precedence; otherwise competing
inherited declarations are ambiguous. Lookup caches are discarded at this phase
transition.

Normalization proves the trait premise separately, then matches complete input
equations against indexed equality assumptions. A selected equality may determine
the result; equal outputs do not imply equal input parameters. Scope opening,
callable binders and result-only lifetimes retain their proof obligations. Calls
instantiate both sides in the existing inference graph and schedule normalization
before the suspended equality completes. Closed declaration equalities require an
independent proof, without assuming themselves. Result formation and outlives
requirements remain separate from the original projected subject's trait premise.

Unused generic equalities may be contradictory, even with declared Sized or Copy
promises. They are retained rather than rejected eagerly or used to grant arbitrary
properties. A demanded normal form must be unambiguous, and an actual call must
satisfy its requirements. Opaque normalizer responses still encounter the existing
shape, lifetime, ownership and execution gates.

The new `binding_contracts.rs` integration group independently qualifies 42 source
outcomes with pinned rustc before checking complete template proofs. Nine native
programs compare RustGM/CPPGM execution with rustc, covering equality parameters,
where clauses, trait arguments/defaults, inherited equalities, closed predicates,
raw names, borrowed results and higher-ranked function-pointer results. Negative
controls include missing members, mismatched calls, environment isolation,
noninjective results, inherited ambiguity, lifetime escape and missing result
well-formedness. Existing course fixtures and text oracles are unchanged.

Artifacts are retained under
`obj/pa20-work/source-associated-bindings-v1/`. Final validation is recorded in
`checks-v8.json` and `validation-v8-counts.json`: all 28 commands pass, including
formatting, strict Clippy, architecture/file audits, 33 harness tests, all 4,864
active course cases, 1,075 release tests, backend qualification and all 44 separate
native groups. The default release suite reports those backend groups as ignored;
the explicit native run executes them. The frozen candidate is
`13f5d97df19e87185eb7100114d1a8341d56c708f4cc550b1a482238eb452db4`,
with 663 source/config/test files frozen for the measurement comparison.

Initial failed checks remain in the receipts. The new test was renamed as one
cohesive group after the filename-family placement audit rejected the original
name. A new unit assertion incorrectly treated static lifetimes as absent from
the type store's conservative free-region summary; the corrected test checks the
substituted type, the region-free fast path and deduplication without changing
that established summary. No lint or audit suppression was added.

This remains a partial PA20 checkpoint. The original positive investigations for
opaque projected identity, projected Copy assumptions, equality-driven default
methods and result-declared Copy behavior still expose missing integration.
General equality premises in implementation search and method applicability,
associated colon bounds/constants and opaque result properties remain required.
PA21's full variance, higher-ranked relationships, GATs, opaque types/captures and
generic drop checking also remain required. Neither planned assignment is
activated or counted as passing.

Validation cleanup removed 4,272 generated files and 4.47 GiB of unshared blocks;
the receipt is `obj/cleanup/1792136160597780461.jsonl`. Frozen inputs, diagnostics,
source snapshots and observations remain available after executable cleanup.


Performance uses two explicit comparisons, with immutable binaries, frozen inputs,
A/A calibration, five paired ABBA blocks per job and full/count output equality:

- Against published `b7f371c`, 36 jobs retain 936 observations across eighteen
  existing-path inputs. Median paired wall-time changes range from -9.70% to
  +1.94%, with resident-median changes from -222 to +492 KiB. A/A spreads range
  from 0.75% to 90.26% on the shared host; the negative deltas do not establish
  a general speedup or a strict zero-regression guarantee.
- Six additional jobs retain 156 observations for 64/256/1,024 associated equality
  bindings, comparing the validated preoptimization implementation (`candidate-v5`)
  with the final candidate. Each input is independently accepted by pinned rustc.
  The final implementation indexes equalities by associated declaration and uses
  a cached certificate to skip unnecessary whole-environment region remapping.
  Full-view resident medians are approximately 6.4/7.8/10.6 MiB. Full-view wall-time
  medians fall from 7.12/27.25/294.59 ms to 6.08/12.03/36.02 ms, with median paired
  deltas of -13.85%/-56.27%/-87.94%. These are semantic-inspection workloads, not
  end-to-end native compile measurements. Calibration spreads, every paired
  block and both count/full modes remain recorded.

All 1,092 timing observations are retained in `measurements/`; `perf-results.json`
and `performance-summary.json` contain summaries. Expected binary hashes are
checked against `binary-sha256.json`; source, input, binary and runner hashes are
verified before and after measurement. No RustGM builds, tests or probes overlap
timing. The wide-list comparison is against an earlier implementation of this
new feature, not the published compiler that does not yet support these inputs.


### Equality premises in implementation selection

Associated equalities now participate in the existing iterative trait proof stack.
A normalizer suspends at each trait premise; the selected implementation or caller
assumption supplies its result, and the original query retains its complete input
equations. Normal forms may determine result holes without inferring unrelated
projection inputs. Unresolved inputs defer while other premises progress. Receiver
method applicability uses this same selection path, and the ordinary callable and
lifetime gates still discharge its actual requirements.

Equality queries share the immutable caller's canonical memo table. Negative
answers depend on both roots and are not retained after an inductive cycle cutoff.
Proofs with fresh existential lifetime allocations remain nonreusable without
rebasing. Private result lifetimes stay attached to the selected child proof until
the enclosing result comparison and final body gate discharge them. A distinct
equality evidence variant cannot stand in for a trait implementation or bypass
ordinary call validation.

`premise_contracts.rs` independently qualifies fifteen accepted/rejected sources
with pinned rustc, then runs the complete LowIR construction pipeline. Nine
accepted programs also compile and execute under rustc and the qualified CPPGM
backend. They cover conditional trait implementations, wrapper subjects, caller
assumptions, inherent and receiver methods, output inference, borrowed results,
callable alpha equality and independent associated equalities. Negative controls
cover wrong results, missing caller equality, receiver rejection, lifetime escape
and inductive cycles. A new internal cache regression uses 64 failed equality
conditions and one applicable candidate: the failures share one completed
negative equality proof, do not constrain the originating output hole, and cannot
hide the valid implementation. All seven selection-cache invariants pass.

The initial sixteen-source investigation and all intermediate results are retained
under `obj/pa20-work/solver-equality-premises-v1/`. Its accepted nested-projected-
subject case remains unsupported and is not counted as a passing regression.
The first test-harness attempt used the generic-template helper, which correctly
does not check concrete bodies; the lifetime-only negative control therefore
needed the full LowIR pipeline. The corrected test retains its original source and
rustc-declared rejection. The initial Clippy failure for unnecessary raw-string
hashes was corrected without a suppression.

This is still partial PA20 work. Nested projected subjects, quantified projection
normalization within implementation equality premises, associated output
parameters that constrain an impl, associated colon bounds/constants and opaque
result properties remain required. PA21 retains its full pending contract. No
course fixture or text oracle is changed, and neither assignment is activated.

The six targeted commands and all 23 final validation commands pass: formatting,
strict all/pedantic Clippy, the 662-file architecture audit with zero findings,
33 harness checks, 4,864 course cases, 1,077 release tests and all 45 separate
native groups. The final validation verifies a frozen snapshot of 665 source and
configuration files. The candidate's SHA-256 is
`c83b1415f2bbd3faf04990342d9f1ef60b7dcceb70a02d6ed89c99e4dc1f1d2f`.
`validation-summary.json` records the counts without conflating ordinary tests,
native groups and the programs within those groups. Successful validation
cleanup removed 4,257 generated files and 4.47 GiB of unshared blocks; its receipt
is `obj/cleanup/1792138407563124856.jsonl`.

Six additional source investigations independently confirm the next parameter
constraint boundary: pinned rustc accepts type results, array-length const
results and inherent impl results that are determined by an associated equality,
while rejecting unconstrained inputs, reverse inference through non-injective
projections and a self-dependent implemented-trait equality. RustGM still rejects
the three accepted cases. Sources, declared outcomes and both compilers'
diagnostics are retained in `followup-qualification.json`; these pending cases
are not included in the passing regression counts.

Performance retains 1,404 observations across 54 jobs with frozen inputs,
immutable binaries, A/A calibration, five paired ABBA blocks per job, and full
and count output equality. The 42 existing-path jobs compare against published
`aa958af`; paired wall-time medians range from -3.33% to +4.78%, and resident
median differences from -642 to +550 KiB. A/A spreads range from 0.56% to
113.94% on this shared host. The twelve new-feature jobs compare against the
immutable first working equality implementation, not a published or fully
regression-validated baseline. At 1,024 repeated premises, canonical proof reuse
reduces full-view median time from 22.77 to 19.25 ms (-15.25% paired) and count
time from 22.19 to 18.72 ms (-15.78% paired). Independent equality-call workloads
range from -2.55% to +1.85%; all six new inputs are accepted by pinned rustc.

Four confirmation jobs retain another 216 observations, with twelve A/A samples
and ten ABBA blocks per job. The 256/1,024 concrete-goal workloads still show
small positive median deltas: +1.29%/+2.07% in count mode and +2.05%/+3.88% in
full mode. Their A/A spreads are 9.20%–42.46%. These results do not establish a
zero-regression claim; the concrete-goal overhead remains visible alongside the
new equality functionality. Raw observations and both result sets remain in the
checkpoint directory. A confirmation-script preflight initially compared its
two selected binaries against the complete three-binary inventory and stopped
before timing; the corrected subset check still verifies every inventory hash.
Source, input, executable and runner hashes are verified before and after each
completed run, with no overlapping RustGM builds, tests or probes.

After published `aa958af` passed CI run `38868912591`, the superseded `b7f371c`
executable was retired with its hash and reason retained in
`retired-baseline.json`. Cleanup continues to preserve receipts and raw
observations while removing rebuildable outputs.

### Parameters determined by associated results

Trait and inherent implementations now share parameter determination after their
typed bound environments are complete. Injective header positions seed the
analysis. Each associated equality supplies a directional dependency from all
of its type, const and lifetime inputs to injective result positions. Original
projection occurrences are retained, so normalization cannot accidentally make
a non-injective result constrain its inputs. An equality on the implementation's
own complete trait predicate cannot justify that implementation's parameters.

The dependency graph uses a worklist indexed by newly determined parameters.
Reverse declaration order does not force repeated scans, and unseeded cycles
cannot bootstrap themselves. Headers which already determine every owned type
and const parameter skip graph construction. Actual implementation selection
still proves the equality conditions, infers the selected arguments, and retains
the existing callable and lifetime checks before lowering.

The twenty-two source controls in `impl_parameter_contracts.rs` are independently
qualified with pinned rustc. Fourteen accepted programs construct LowIR and run
under both rustc and the CPPGM backend. They cover result types, array-length
consts, inherent and receiver methods, forward/reverse dependency chains,
parameters seeded by trait arguments, tuple/callable results, trait-argument
dependencies, distinct applications of the same trait, and lifetime dependencies.
Rejections cover unseeded inputs and cycles, reverse inference, missing argument
or lifetime inputs, non-injective results, self-dependent trait equalities and
an invalid call after an otherwise valid impl. A separate 10,000-link reverse-order
graph regression verifies bounded work and rejection of an unseeded cycle.

The initial implementation passes all twenty-two declared source outcomes, strict
Clippy, the graph regression and the native controls. Investigation inputs,
diagnostics and receipts are retained under
`obj/pa20-work/impl-parameter-constraints-v1/`. The preceding equality-solver
checkpoint `7dae890` passed CI run `38873338160`.

This remains partial PA20 work: projected impl-bound subjects, quantified
normalization, associated colon bounds/constants and opaque result properties
remain required. PA21's full advanced-generic contract remains pending. No
course fixture, oracle, handout status or assignment activation changes here.

All 23 final validation commands pass: 4,864 course cases with unchanged oracles,
1,079 release tests, all 46 separate native groups and the backend bridge checks.
Targeted validation also passes formatting, strict Clippy, the 665-file audit with
zero findings and 33 harness checks. The final candidate verifies a frozen
668-file source/configuration snapshot and has SHA-256
`3b48fc96872b9b0b6b403793e2c507d58ac51ce31e4b03909c6bcfd7b32382d5`.
`validation-summary.json` retains the exact command and test counts. Cleanup
removed 4,300 generated files and 4.53 GiB of unshared blocks, recorded in
`obj/cleanup/1792139921611575015.jsonl`.

Six follow-up coherence probes preserve the distinction between overlapping and
provably disjoint impls. Both compilers reject resolved overlaps, renamed-output
duplicate impls and duplicate inherent methods, and accept distinct inherent
methods. RustGM still rejects two valid disjoint trait impl pairs: one whose
required local trait has no implementation, and one whose closed associated
result contradicts a fixed equality. The latter was initially hypothesized to
overlap; pinned rustc instead accepts it. That original hypothesis and diagnostic
are preserved in `coherence-declared.json` and `coherence-qualification.json`,
without generating or blessing an oracle. These are pending coherence cases,
not passing regression claims.

The next projected-impl-bound investigation retains twelve independently
qualified sources under `obj/pa20-work/projected-impl-bounds-v1/`. RustGM rejects
all seven accepted sources at the existing declaration/proof boundary; five
negative controls are also retained. No new implementation or claimed support
for those cases is included in this checkpoint.

Performance retains 1,872 observations across 72 jobs. The 54 existing-path jobs
compare immutable published `7dae890` and candidate binaries with frozen inputs,
A/A calibration, five paired ABBA blocks per job and full/count output equality.
Median paired deltas range from -2.84% to +7.30%, resident-median differences from
-96 to +668 KiB, and calibration spreads from 0.74% to 61.58%. The largest positive
delta, a 128-function projected-bound workload, was investigated in four
confirmation jobs retaining another 216 observations with twelve A/A samples and
ten ABBA blocks per job. Confirmation full/count deltas are +0.47%/+1.47% at 128
functions and +1.93%/+1.80% at 2,048 functions. Calibration spreads of 7.28%–84.41%
and other host activity preclude a strict zero-regression claim. Both the initial
and confirmation observations are retained.

Eighteen additional jobs measure nine independently rustc-qualified scaling
inputs at 64/256/1,024 constraints. Their A and B labels intentionally use the
same immutable candidate: these are repeated scaling controls, not before/after
speedup comparisons. Forward-chain full-view medians are 5.46/9.37/24.31 ms;
reverse-chain medians are 5.32/9.07/24.26 ms. Resident medians at 1,024 links are
about 9.1 MiB for both orders. Many associated results sharing one input measure
5.16/8.49/33.57 ms, reaching about 8.4 MiB at 1,024 results. These measurements
include semantic inspection and text rendering, not just dependency traversal or
end-to-end native compilation. `performance-summary.json`, `perf-results.json`,
`confirmation-results.json` and raw observations preserve the distinctions.

All measured source, input, binary and runner hashes remain unchanged. RustGM
builds, tests and probes do not overlap timing. Obsolete `aa958af` and the first
equality-solver prototype binaries were retired after `7dae890` passed CI;
`retired-baselines.json` retains their hashes and reasons. Current baselines remain
pinned, while cleanup removes generated executables and retains evidence.

### Opaque projected bounds in declarations

Trait and inherent impl bounds now use the same staged declaration environment
as function bounds. Their raw typed predicates exist before projected annotations
normalize. A completed opaque bound retains its projection without re-enqueuing
the same environment indefinitely. Logical shape checks require an exact trait
premise, traverse all projected type inputs, and grant only declared or explicitly
assumed Sizedness. They do not invent layout, Copy or result lifetime properties.

The well-formedness walk separately visits projection inputs at their original
callable depth. Input reference requirements remain distinct from the associated
result's outlives and live-value components. Those result queries retain their
normalization guards. This distinction prevents an opaque result from acquiring
lifetime permissions merely because a lifetime appears in its inputs.

Seven declaration sources are independently qualified with pinned rustc in
`projected_bound_declarations.rs`, then checked through LowIR construction.
They cover function, trait impl and inherent impl bounds, unsized projected array
elements, explicit Sized evidence, and distinct arguments of the same trait.
Two structural regressions check exact premise identity, Sized versus ?Sized,
unchanged Copy/layout guards, input lifetime requirements and the separate
result lifetime guard. No course fixture or text oracle changes here.

Investigation sources, diagnostics and intermediate candidates remain under
`obj/pa20-work/projected-impl-bounds-v1/`. Its twelve initial call-site probes
still expose unsupported normalization of projected impl conditions: declaration
acceptance alone does not establish executable support for those seven positive
programs. Five additional lifetime probes include a corrected initial hypothesis:
pinned rustc accepts a function that uses an implied input outlives bound even
when the associated result normalizes to an integer. RustGM still loses that
original input fact at the later lifetime gate. The original hypothesis and
diagnostics are retained; this case is not counted as a passing regression.
References to opaque associated results also need further result lifetime work.

PA20 remains partial, with projected conditions at selection, coherence,
quantified normalization, associated colon bounds/constants and opaque result
properties still pending. PA21's advanced generic contract remains pending.
The preceding parameter-determination checkpoint `a2c9fc7` passed exact-head
CI run `38875981260`.

All seven targeted commands and 23 final validation commands pass. The final
suite has 4,864 course cases, 1,082 release tests and 46 separate native groups;
the architecture audit checks 666 files with zero findings, and all 33 harness
checks pass. Validation verifies a frozen 669-file source/configuration snapshot.
The candidate SHA-256 is
`d74d1b36d03eacb37f6685648ce80705e28a5c43447620f0328fc128107e0609`.
Exact counts remain in `validation-summary.json`, without combining course
cases, ordinary tests and native groups into a misleading assignment total.

Performance retains 2,184 observations across 84 jobs, with immutable binaries,
frozen inputs, A/A calibration, five paired ABBA blocks and full/count output
equality. The 72 existing-path jobs compare published `a2c9fc7` against this
candidate. Paired median wall-time changes range from -4.33% to +2.39%, with
resident-median differences from -620 to +784 KiB. Calibration spreads range
from 0.78% to 121.02% on the shared host. The largest positive median is the
1,024-receiver full-view case; these results do not establish zero regression.

The remaining twelve jobs measure six new inputs independently accepted by pinned
rustc. Both timing labels intentionally use the same final candidate, so these
are scaling controls rather than speedup comparisons with an unsupported old
compiler. Full LowIR output for 64/256/1,024 opaque function bounds takes median
11.54/18.80/67.89 ms; corresponding impl bounds take 13.14/37.52/140.07 ms.
At 1,024 declarations, resident medians are approximately 14.6 MiB and 17.8 MiB.
These timings include the complete LowIR pipeline and output rendering, rather
than isolated bound checking. Sources, executable and runner hashes are verified
before and after timing, with no overlapping RustGM builds, tests or probes.
`performance-summary.json`, per-job receipts and every raw observation remain.

Validation cleanup removed 4,304 generated files and 4.53 GiB of unshared blocks,
recorded in `obj/cleanup/1792142872116185774.jsonl`. Intermediate binaries were
retired once the final candidate passed; `retired-prototypes.json` retains their
hashes and reasons. Only binaries needed for the current comparison remain pinned.

### Projected implementation conditions and inference

Implementation selection now normalizes projected trait subjects and type
arguments on the existing iterative proof stack. Equality premises and projected
builtin conditions share that machinery. The original query retains its complete
normalization evidence, selected implementation and private lifetime interval.
Ordinary builtin premises retain their direct proof path. A suspended builtin
continues after earlier conditions without repeating completed conditions.

A normalized input view preserves the original projection graph and shares its
ordinary inference variables. This permits an associated result beside an
unresolved tuple element to select a trait implementation and determine that
element. It does not infer a projection's inputs from an equal result. Canonical
operation identity supplies congruence for unchanged opaque inputs. The original
canonical predicate owns the memo key; proofs with private lifetime allocations
remain ineligible for reuse across queries.

An early caller-assumption lookup now preserves its typed normalization-needed
result. The solver normalizes the requested argument and retries against the
caller's concrete bound, including `T: A<Out = u16> + C<u16>` satisfying a
conditional implementation that requires `T: C<<T as A>::Out>`. An opaque
comparison that still cannot complete defers without establishing negative
cache evidence.

The permanent `projected_impl_conditions.rs` matrix has 28 independently checked
sources. It covers nested trait/equality conditions, inferred associated outputs,
partial and nested argument inference, inherent methods, mixed trait arguments,
Copy/Clone/Sized conditions, exact caller assumptions, missing premises,
inductive cycles and captured-region escapes. Two structural tests preserve
original projection roots and shared holes while rejecting inference from unrelated
operations. A solver invariant checks repeated unsatisfied projected conditions
across 64 candidates reuse one negative proof. Six separate native sources check
selected calls and methods through the external CPPGM backend against pinned
rustc. These are maintainer regressions; no course text oracles are regenerated.

Research evidence and immutable candidates are retained under
`obj/pa20-work/solver-projected-inputs-v1/`. Earlier variants exposed the need to
commit original input identity, normalize builtin conditions, preserve independent
holes and retry early assumption lookup. Explicit opaque projected outlives
bounds still encounter an earlier declaration property guard. Original callable
formation contracts, quantified normalization, general opaque properties,
associated colon bounds/constants and the remaining coherence/PA21 contract
remain required. PA20 and PA21 are not activated by this checkpoint.

All six targeted commands and 23 final validation commands pass, along with the
standalone `make lint` target. The final suite has 1,086 release tests across 197
result groups, 4,864 course cases and 47 separate native groups. The architecture
audit checks 671 files with zero findings; all 33 harness checks pass. A frozen
674-file source/configuration snapshot is verified after validation. The final
candidate SHA-256 is
`06dffa2e18642e66d3532def0d7647617bdf5432914bb31c3a8de9fa09a5c088`.
`validation-summary-v9.json` preserves the separate counts and command statuses.

Final validation cleanup removed 4,333 generated files and 4.55 GiB of unshared
blocks, recorded in `obj/cleanup/1792147928176692023.jsonl`. Intermediate candidate
binaries were retired with hashes and reasons in `retired-prototypes.json`.
Only the published `92c75a5` baseline and final candidate were retained for the
comparison; historical source snapshots, diagnostics and hashes remain available.

Performance retains 3,120 observations across 120 jobs. The initial 96 jobs
comprise 84 existing-path comparisons with `92c75a5` and twelve same-candidate
scaling controls. They use immutable executables, frozen inputs, A/A calibration
and five paired ABBA blocks, with full/count output equality and all source,
executable and runner hashes verified. RustGM builds, tests and probes do not
overlap timing. Existing-path paired median changes range from -9.40% to +14.59%,
with calibration spreads of 1.70%–48.54% on the shared host.

Every measured size and mode of the four families showing an increase above 3.5%
was repeated in 24 confirmation jobs; the original observations remain intact.
Confirmation paired medians range from -2.98% to +2.94%, with calibration spreads
of 2.63%–88.94%. The largest initial increases did not recur at the same magnitude.
These results do not establish zero overhead. Initial resident-median differences
range from -774 to +628 KiB; confirmation differences range from -68 to +582 KiB.
`confirmation-plan.json`, `performance-summary.json`, `confirmation-summary.json`
and the raw observation files preserve the selection rule and both runs.

Six new scaling inputs are independently accepted by pinned rustc. Their A/B
labels intentionally use the same candidate and do not measure improvement over
an unsupported predecessor. Combining paired observations from both labels, full
LowIR medians at 64/256/1,024 constraints are 6.83/15.22/48.01 ms for wide projected
bounds and 8.48/21.90/72.43 ms for nested inference queries. Resident medians at
1,024 constraints are approximately 12.5 MiB and 10.4 MiB. These are complete
frontend/LowIR measurements including rendering, rather than isolated solver
measurements. `scaling-summary.json` retains the exact values.

### Shared projected predicate inputs

Repeated trait arguments now preserve one inferred graph through projection
root discovery, normalized-view rewriting and canonical predicate resolution.
Complete input matching shares successfully related root pairs within one
predicate relation and enclosing lifetime scope. The cache cannot cross proof
queries or make different projected operations injective. Known canonical roots
also avoid allocating traversal scratch on the ordinary resolution path.

A structural invariant repeats one 256-wide projected tuple through 256 trait
arguments. It checks a single discovered projection, identical rewritten root
identities, unchanged original projection roots, and linear work bounds for
rewriting, canonicalization, inferred matching and canonical matching. A permanent
source regression compiles a 1,024-wide shared alias through 1,024 trait arguments;
a negative control changes the last implementation argument and must still fail.
Both source expectations are checked independently with pinned rustc.

The investigation at `obj/pa20-work/projected-input-sharing-v1/` retains twelve
independently object-qualified source probes. Published `caa3808` accepts all
unused controls and calls through width 256, but exhausts its work budget at
widths 512 and 1,024. The candidate accepts all twelve without increasing the
budgets. An intermediate prototype fixed only discovery and rewriting; its large
calls still failed, identifying repeated complete-predicate equations and
resolution as additional work. Diagnostics and versioned receipts retain those
observations. These are maintainer regressions; course oracles are unchanged.

All 25 final validation commands pass: strict Clippy, architecture/harness checks,
1,088 release tests across 197 result groups, all 4,864 PA1–PA19 course cases,
backend qualification and 47 separate native groups. The Rust audit checks 671
files with zero findings; all 33 harness checks pass. The frozen 674-file source
snapshot matches after validation. Candidate SHA-256:
`4530c8d162441dd7368bf023745c45679165a09aeac7f4e46f1f6a402fee9a06`.
`validation-summary.json` preserves counts and command statuses. A first runner
attempt nested the storage guard through `make lint` and was correctly refused;
its receipt is retained. The corrected batch runs direct Clippy beneath the
outer guard. Validation cleanup removed 4,334 generated files and 4.55 GiB of
unshared blocks (`obj/cleanup/1792150242791762521.jsonl`).

The standalone `make lint` target also passes. Performance retains 3,016
observations in 116 jobs: 106 initial comparisons with immutable published
`caa3808`, four same-candidate controls for newly accepted large calls, and six
confirmation comparisons. Full/count output equality and all executable, frozen
input, source and runner hashes are verified. RustGM builds, tests and source
probes did not overlap timing. Initial paired median changes range from -50.76%
to +5.08%, with A/A spreads of 0.71%–107.36% on the shared host. The predeclared
3.5% rule selects every size/mode of `inferred-sized` for confirmation; those six
paired medians range from -4.28% to -0.21%, with A/A spreads of 3.33%–52.72%.
The initial increase did not recur. Initial resident-median differences range
from -98 to +688 KiB; confirmation differences range from +138 to +228 KiB.
Raw observations and both sets of results remain intact in the evidence directory.

Full LowIR median times for repeated calls decrease from 9.73 to 7.88 ms at
width 64 and from 49.45 to 24.65 ms at width 256. The newly accepted 512/1,024
calls have same-candidate scaling controls, not comparisons against a successful
predecessor. Their medians are approximately 62/202 ms, with about 8.6/10.6 MiB
resident memory. Unused width-1,024 declarations remain near 201 ms. These complete
frontend measurements include declaration checking and rendering; the linear
structural invariants apply to the shared predicate operations, not the entire
source pipeline. `performance-summary.json` and `scaling-summary.json` retain
exact measurements. PA20 and PA21 remain incomplete and are not activated here.

### Associated result-bound declaration contracts

Associated type declarations now elaborate ordinary user-trait bounds through
the existing bound worklist. Result contracts retain complete predicates, including
separate projected subjects for nested result equalities. Builtin bounds keep
their direct path. A trait's associated declaration identities exist before its
bound arguments are elaborated; unsupported projected argument scopes retain an
explicit boundary instead of losing their obligations.

An implementation checks each required result in its own environment. One lazy
substitution maps the trait's Self, type, const and lifetime arguments across all
result contracts of that implementation. Actual result types already belong to
the implementation's scope and are not substituted a second time. Definition
well-formedness and every selected proof's lifetime obligations remain mandatory.

The bound declaration itself also needs validation, independently of any impl
or caller. Retained associated-owner predicates pass the ordinary bound checks
and final trait-argument lifetime gate. This rejects an unsized Self or type
argument passed to a Sized parameter, a missing trait argument bound, and a
missing lifetime requirement even in an unused trait. An intermediate prototype
missed those four cases; independent rustc checks exposed the gap before release.
Their diagnostics and the corrected results remain in
`obj/pa20-work/associated-result-bounds-v1/`.

`result_bound_contracts.rs` contains 23 independently qualified declaration
programs. They cover forward and recursive declarations, mixed substitutions,
invalid definitions, nested equality bounds and declaration-time well-formedness.
Three separate native programs check concrete selection, type-argument mapping
and a nested associated local type through CPPGM against pinned rustc. The latter
uses an ordinary method return type; associated-type method return signatures
still require conformance normalization with retained formation evidence.
Shorthand associated Self paths also remain required work.

The full investigation has 31 object-qualified rustc programs. Eight are outside
this declaration checkpoint: generic-body result permissions, transitive and
recursive result-bound derivation, Copy permissions, nested generic equality
use, their paired negative controls, and explicit result outlives. Do not add
recursive result clauses to the eager ordinary-supertrait closure: valid
`type Out: A` declarations require demand-driven proof. These remaining tasks,
original callable formation contracts, associated constants, remaining coherence
and PA21 are still part of the goal. PA20 and PA21 remain planned; course
fixtures and text oracles are unchanged.

All 25 final validation commands pass: strict all-target Clippy, architecture and
harness checks, 1,089 release tests across 198 result groups, all 4,864 PA1–PA19
course cases, backend qualification and 48 separate native groups. The syntax-aware
Rust audit checks 672 files with zero findings; all 33 harness checks pass.
The frozen 675-file source snapshot matches after validation. Candidate SHA-256:
`527952ec921a25f1258b3fdd56a1e74bf22d524176362fcd16c5297daaa7564b`.
The evidence directory's `validation-summary.json` records counts and command
statuses. A preliminary Clippy check caught a 101-line conformance function;
extracting the cohesive result-contract instantiation operation fixes the limit
without a lint allowance. Its original snapshot and diagnostic remain available.

Standalone `make lint` also passes. Validation cleanup removed 4,353 generated
files and 4.56 GiB of unshared blocks
(`obj/cleanup/1792153143909519219.jsonl`). Superseded prototype executables are
retired with hashes and reasons in `retired-prototypes.json`; diagnostics, inputs
and receipts remain. The final candidate and current published comparison baseline
remain pinned. The local 40 GiB reserve is unchanged.

Performance retains 3,796 observations in 146 jobs: 110 initial comparisons with
immutable published `b579008`, twelve same-candidate scaling controls for the new
result bounds, and 24 confirmations. All executable, input, runner and frozen
source hashes match, as do full/count output hashes. RustGM builds, tests and
source probes did not overlap timing. Initial paired median changes range from
-6.36% to +7.19%, with A/A spreads of 0.55%–182.52% on the shared host. The
predeclared 3.5% rule repeats every size/mode of `forward-chain`, `shared-input`,
`wide-equalities` and `wide-projected-bounds`. Confirmation paired medians range
from -2.82% to +3.19%, with A/A spreads of 1.30%–26.68%; no confirmation exceeds
the threshold. This does not establish zero cost. Resident-median differences
range from -138 to +1,348 KiB initially and -106 to +1,272 KiB in confirmation;
the width-1,024 shared-input family retains the largest increase, about 1.2 MiB.
Both sets of raw observations remain intact, including noisy observations.

All six new scaling sources independently pass pinned rustc object generation.
Full LowIR median times at 64/256/1,024 associated members are approximately
5.03/9.00/27.29 ms; the corresponding independent-implementation cases take
5.25/13.63/37.50 ms. At 1,024, resident medians are approximately 9.9 MiB and
12.0 MiB. These are complete frontend measurements, not an isolated complexity
proof. `performance-summary.json`, `scaling-summary.json` and the measurement
receipts retain exact values. PA20 and PA21 remain incomplete.

### Deriving user-trait result permissions

The selector can now use a user-trait bound declared on an opaque associated
result. It first normalizes the original query inputs and proves their parent
trait applications on the existing iterative proof stack. A separate result-rule
source retains the actual selected predicate. Normalization and final body-bound
checks preserve this evidence and its lifetime obligations; it never supplies an
implementation body by itself. Direct, transitive and recursive result bounds use
finite demands rather than expanding an unbounded associated-result closure.

One capture-safe mapping serves both implementation conformance and result-rule
instantiation. The ordinary-supertrait closure of each result declaration is
indexed by trait identity before caller substitution. A query maps only candidates
for its requested trait, retaining alternatives with distinct arguments. Pure
mapped clauses share a cache only after the complete result and ordinary trait
declarations are available. Full projected type identities and the requested
trait preserve Self, trait/family arguments and lifetimes. Header changes invalidate
that epoch. A structural invariant prewarms the cache, then checks callers with,
without and again with the parent permission: only the permitted callers succeed.

The first working selection prototype exposed a later boundary in generic
function-item value liveness. Symbolic liveness now follows the sufficient route
of keeping all projection input components live. It grants no explicit result
outlives assumption, concrete property or reverse implication. Closed component
queries still reject opaque results. The invariant tests all input regions and
rigid type parameters, zero-length runtime arrays, shared graph work and cache
reuse. Exact output components and alternate declared-result lifetime proofs
remain required work.

`derived_permissions.rs` includes 36 independently object-qualified source cases
and six separately executed native comparisons with pinned rustc and CPPGM. They
cover direct/supertrait/recursive result permissions, mixed arguments, implementation
premises, unsized results, inference ambiguity and caller-bound preference. An
initial expectation that caller and result clauses must be ambiguous was disproved
by pinned rustc; the corrected accepted case and original qualification receipts
remain in `obj/pa20-work/result-bound-derivation-v1/`. The native preference control
uses two different return values to check the selected body.

Nine independently object-qualified scaling sources combine widths 64, 256 and
1,024 with repeated calls, distinct members and repeated queries over a wide
result-bound list. Rebuilding the mapped index on each query exhausted the prior
prototype's work budget at the largest wide case. Reusing pure clause mappings
accepts all nine with unchanged budgets. Three further rustc-qualified sources
use independent generic functions at the same widths. The per-projection cache
still exhausted the trait work budget at width 1,024 because each distinct Self
expanded the whole declaration. Indexing declaration templates before mapping
addresses that separate scaling problem. Permanent width-1,024 source regressions
and smaller warmed-cache unrelated-trait rejections cover both repeated queries
and independent caller scopes.

The broader source investigation has 39 independent outcomes. Three positive
cases remain outside this checkpoint: builtin Copy permission, nested result
equalities and explicit result outlives. Original callable formation contracts,
associated-type method signatures, shorthand paths, associated constants,
remaining coherence and PA21 also remain required. PA20 and PA21 are still planned;
course text oracles are unchanged.

The final 25-command validation passes: strict all-target Clippy, architecture
and harness checks, 1,093 release tests across 200 result groups, all 4,864 active
PA1–PA19 course cases, backend qualification and 49 native groups. The syntax-aware
audit checks 677 files with zero findings; all 33 harness checks pass. Standalone
`make lint` also passes. The final 680-file source snapshot matches after validation.
Candidate SHA-256:
`0a3f2845dc3dbcfd5ca71eca4821f01de63230d89ef9d4006bec7dd0651cebc5`.
The evidence directory retains the earlier per-projection prototype's complete
validation separately under `v4-full-*`; that passing suite did not cover the
independent-scope scaling failure. Neither prototype failure was resolved by
raising a work budget or changing an oracle.

Performance retains 4,160 study observations in 160 jobs, plus all 640 full-output
qualification observations. Initial jobs comprise 122 comparisons with immutable
published `89fb1fd`, sixteen with the first working selection prototype, four with
the per-projection cache prototype, and four same-candidate controls where an
earlier prototype exhausted its work budget. All binary, input, runner and frozen
source hashes match; full/count outputs match throughout. Builds, tests and source
probes did not overlap measurements.

Published-baseline paired median changes range from -4.03% to +4.43%, with A/A
spreads of 0.49%–272.06% on the shared host. The predeclared 3.5% rule repeats all
sizes/modes of `forward-chain` and `repeated-input-called`: fourteen confirmations
range from -3.77% to +1.79%, with A/A spreads of 1.39%–10.91%. No confirmation
exceeds the threshold; this is not a zero-cost claim. Published-baseline resident
median differences range from -772 to +534 KiB initially, and -626 to +290 KiB
in confirmation. Every initial and confirmation observation remains available.
The same-binary controls include a +6.90% paired change, another indication of
measurement noise; they are not comparisons with a working older implementation.

At width 256, full LowIR generation for repeated queries over a wide bound list
falls from about 108.53 ms in the first prototype to 20.61 ms. Independent caller
scopes fall from 135.15 ms in the per-projection cache prototype to 31.07 ms, with
resident medians falling from 31,754 to 9,384 KiB. Width-1,024 independent scopes,
which previously failed, take about 115.40 ms and 18.1 MiB; the corresponding wide
repeated-query control takes about 71.73 ms. These are whole-frontend observations,
not isolated complexity proofs. Distinct-member width-1,024 retains a small measured
cost against the first prototype: about +1.37% and +620 KiB in full-output mode.
`performance-summary.json`, `scaling-summary.json` and their receipts retain the
exact values and comparison identities.

Validation cleanup removes 4,387 generated files and 4.59 GiB of unshared blocks
(`obj/cleanup/1792156895917841157.jsonl`). The local 40 GiB reserve remains unchanged.
The current final binary and required comparison binaries stay pinned until
publication checks finish; superseded executables are then retired with hashes
and reasons while source, diagnostics and measurement receipts remain.

### Associated types in method contracts

The maintainer now normalizes a trait's own associated types in qualified method
parameters and results before comparing an override. Associated definitions pass
their own conformance first; a local index shares those checked definitions across
the impl's methods. Each lookup matches the full parent application and preserves
its callable depth. The implementation becomes globally validated only after all
methods pass. Ordinary selections retain their lifetime evidence, and generic
requirements still cannot strengthen the trait's contract.

Mapped conformance signatures already use the method proof's region scope. Their
components therefore normalize independently of the outer function wrapper, with
nested callable binders intact. One semantic normalization session shares completed
graph work across those components while holding the environment, proof scope,
availability and local provider fixed. It retains the proof and nested scope map
alongside cached normal forms. Body instantiation now schedules normalization when
the callable signature contains projections, even when no earlier projection task
is pending. That missing trigger was a second boundary after declaration conformance.

`method_contracts.rs` contains 34 independently object-qualified source outcomes
and twelve native comparisons against pinned rustc and CPPGM. They cover concrete
and generic impls, projected inputs/results, references, mixed trait arguments,
function-pointer values, dot calls, equality-constrained generic calls, nested
callables and independent method lifetimes. An ordinary input can infer `Self` and
wake projection normalization; an associated result alone cannot infer `Self`.
Swapped lifetime contracts, narrowed
universal results, wrong projected types, stronger generic requirements and unchecked
associated definitions reject. Native controls use different return values where
needed to check which callable executes. Focused tests, strict Clippy, the file audit
and harness checks pass. The audit caught a seventh `associated_` filename prefix;
renaming the test to its method-contract responsibility resolves that finding
without an exception. The original diagnostic and test source remain in
`obj/pa20-work/associated-method-signatures-v1/`.

The initial source investigation has 43 independently qualified outcomes. Six valid Rust
programs remain outside this checkpoint: three shorthand associated-path cases,
a call requiring original erased-input lifetime facts, opaque generic result
forwarding, and a nested callable combining trait lifetime arguments with multiple
impl captures. The negative erased-input escape still fails at the same body
lifetime boundary as its valid control; it does not demonstrate completed call-site
formation support. The multiple-capture negative also fails before conformance,
so the successful paired method-lifetime controls provide the conformance evidence.
These distinctions remain explicit in the qualification receipts.

Four further independently object-qualified controls check inherited and unrelated
associated results. Concrete and generic supertrait-result overrides remain
unsupported on both published `70963bf` and this candidate. The wrong-result
control rejects at the same normalization boundary, so it does not establish
inherited method conformance. A closed unrelated associated result already works
on both binaries. These four controls bring the source investigation to 47 outcomes;
their diagnostics remain in `qualification-supertraits.json`.

Seventeen scaling sources independently pass rustc object emission. Thirteen
compile before and after session sharing: wide associated-member/method sets at
64/256/1,024, distinct captured parameters at 64/256, and shared captured parameters
at 64/256/1,024, including nested function components. At 1,024 distinct type
parameters, the two projected cases reach the existing runtime-lifetime storage
limit. Two ordinary-signature controls reproduce that limit on published `70963bf`
as well as the prototypes. All four diagnosed failures remain recorded; they are
excluded from successful timing comparisons. Budgets have not been raised.

Final validation passes 1,094 release tests in 201 result groups, all 4,864 PA1–PA19
course cases, backend qualification and all fifty explicit maintainer native test
groups. Strict all-target Clippy, standalone `make lint`, 679 audited Rust files
with zero findings, and 33 harness checks pass. The frozen snapshot contains 682
source/configuration/test files. The final compiler matches the session-sharing
prototype byte-for-byte, with SHA-256
`70552976c26df34688b0d7261db9dbf0543a702da5e263edf0a5d0b3071e8f66`.
`validation-summary.json`, source hashes and individual logs retain the evidence.

Performance retains 6,284 study observations across 214 jobs, plus 856 full-output
qualification observations. The initial study compares 146 existing jobs with
published `70963bf` and 26 new scaling jobs with the first working method-signature
prototype. All binary, input, runner and frozen-source hashes match; full/count
outputs match throughout. No builds or source probes overlap measurement.

Initial published-baseline paired changes range from -6.32% to +5.75%, with A/A
spreads of 0.31%–82.58% on this shared host. The declared 3.5% rule repeats every
size and mode of three published-baseline families and one prototype family:
24 confirmations. The eighteen published-baseline confirmations range from
-11.46% to +4.06%; the six wide-method prototype confirmations range from -2.88%
to +3.30%. Every initial observation remains available.

The remaining +4.06% confirmation, full semantic output for 256 concrete goals,
prompted eighteen longer jobs: all six sizes/modes versus the published compiler,
plus the same inputs with each binary compared to itself. Each retains fifteen
ABBA blocks after warmup and A/A calibration. Genuine comparisons range from
-2.24% to +3.40%, with A/A spreads of 3.03%–7.10% and resident median increases of
184–584 KiB. Identical-binary paired changes range from -1.52% to +0.90%. The
256-goal full-output comparison is +0.92% in this study. These results retain a
small measured cost; they do not establish zero overhead. `focused-plan.json`,
`focused-summary.json` and raw observations record the follow-up separately.

Session sharing reduces full LowIR time for 1,024 repeated captured-parent inputs
from about 109.82 ms to 33.00 ms; nested callable inputs fall from 111.75 ms to
34.74 ms. Resident medians remain about 11.3 MiB. At 256 distinct captures,
ordinary/nested cases fall from 70.75/63.49 ms to 52.36/47.55 ms. These are measured
whole-frontend improvements over the working prototype, not comparisons with the
published compiler, which lacks the new method-signature behavior.

Full-validation cleanup removes 4,430 generated files and 4.64 GiB of unshared
blocks (`obj/cleanup/1792160880664633141.jsonl`). The local 40 GiB reserve remains
unchanged. Comparison executables stay pinned through publication checks; replaced
binaries are then retired while sources, diagnostics, hashes and observations stay.

Original callable formation, remaining opaque-result properties and lifetime
proofs, inherited method contracts, shorthand paths, associated constants, remaining
coherence and PA21 remain required work. PA20 and PA21 are still planned; course
text oracles are unchanged.

### Inherited method contracts and closed caller bounds

The maintainer compares inherited associated method signatures using the actual
impl's bounds and the trait method's own predicates. It no longer adds the
containing trait's inherited predicates as new conformance assumptions: the impl
header has already proved those requirements. Previously a synthetic `Self: A`
assumption could hide the concrete result selected by an `A` implementation.
Method bodies retain their own declared environments, and every impl still passes
its complete header, associated-definition and method checks before publication.

An explicit method bound remains significant. Independent rustc object checks
rejected two initially hypothesized acceptances: a repeated bound on generic
`[T; 1]`, and a repeated lifetime-dependent trait application on concrete `u8`.
Both retain opaque associated results. A proposed general duplicate-bound filter
was discarded before building it. The initial declarations, diagnostics, rejected
proposal and reviewed expectations remain in
`obj/pa20-work/inherited-method-contracts-v1/`; these were investigation corrections,
not oracle updates.

The supported fallback is narrower. After proving the trait premise, explicit
associated equalities and ambiguity keep their existing priority. An absent
equality can fall back to an independently selected implementation only when the
entire projection application has no type/const parameters, external lifetime variables
or projection inputs. Selection uses no caller assumptions and retains its full
region evidence under the existing implementation-availability mode. Generic and
lifetime-dependent explicit bounds remain opaque. The fallback also handles
ordinary function signatures with repeated closed bounds.

`inherited_methods.rs` passes 32 independently object-qualified source outcomes
and twenty native comparisons against pinned rustc and CPPGM. Coverage includes
concrete and generic impls, declaration order, trait arguments, reference inputs,
captured results, own/inherited results together, method-specific bounds/equalities,
repeated closed and transitive bounds, function pointers and dot calls. Wrong
results and stronger generic/lifetime contracts reject. The broader investigation
has 34 qualified outcomes: opaque forwarding and opaque destruction still reject
two valid programs at their later body boundaries. Neither rejection establishes
failed declaration conformance.

Twenty-one scaling sources pass rustc object qualification and compile with the
candidate. They cover 64/256/1,024 associated members, wide supertrait lists,
repeated captured-parent parameters and nested callable components, plus ordinary
signature controls. The first inheritance prototype rejects the six repeated-bound
sources, which the closed fallback now accepts; all its earlier successful inputs
remain accepted. These inputs check declarations and method bodies with an empty
`main`; they are not native execution cases. Strict Clippy, seven existing focused
modules, 681 audited Rust files with zero findings, and 33 harness checks pass.
The final frozen candidate passes 1,098 release tests in 202 groups, all 4,864
PA1–PA19 cases, backend qualification and 51 maintainer native test groups.
All 25 final commands pass, and the 684-file source snapshot matches the checked
tree. Standalone lint also passes. Cleanup removes 4,489 generated files (4.72 GiB
of unshared blocks), retaining inputs, diagnostics and qualification receipts.
PA20/PA21 and their course oracles remain planned and unchanged.

A late lifetime-evidence control exposed a conservative restriction in the first
candidate: the existing free-region metadata includes static lifetimes for
substitution, so it also excluded a concrete static-reference receiver. A separate
cached flag now distinguishes non-static free regions while retaining the existing
substitution behavior. The record remains four bytes. Three typed invariants check
that size, mixed-region accumulation order, callable binder boundaries, full trait
arguments and unresolved projections. A static-reference program now executes
through an implementation with a lifetime parameter; its local-reference escape
control rejects at the lifetime proof. The earlier complete validation is retained
under `full-v4/`; the final validation above covers the metadata change. Its
immutable compiler hash is
`5b539396c4d3730c1918722fee5fc7734d3b5de43158c257134f179af0774e89`.

Immutable performance measurements retain 238 jobs, 6,188 study observations
and 952 full-output qualification observations. The initial study compares 178
jobs with published `4472116`, 24 with the earlier inheritance prototype, and
twelve repeated-bound jobs with the earlier closed-fallback prototype. Every
input, source snapshot and binary hash remains unchanged; full and count outputs
match throughout. Each job includes A/A calibration and five paired ABBA blocks.
No local RustGM builds or qualification probes overlap the timed work.

Initial paired medians against published main range from −45.03% to +9.99%,
with A/A spreads of 0.34–104.22%; all original outliers remain recorded. The
predeclared 3.5% repeat rule selects four families and all their sizes/modes.
The eighteen main-baseline confirmations range from −2.40% to +3.41%, and the
six inheritance-prototype confirmations range from −1.07% to +1.59%. All are
below the repeat threshold; this is not a claim of zero overhead. Confirmation
RSS differences range from −18 to +130 KiB. The static-aware metadata's twelve
comparisons against the closed-fallback prototype range from −3.22% to +2.67%.

Removing synthetic inherited assumptions also reduces work on ordinary method
signatures with wide supertrait lists. At 1,024 parents, the full-view whole
frontend median changes from 1,067.36 ms to 594.64 ms against published main
(paired median −44.03%), with matching output. Frozen sources, runners, raw
observations, summaries and the original validation remain under
`obj/pa20-work/inherited-method-contracts-v1/`.

## Opaque body values and concrete instance normalization

The next checkpoint keeps a proved, unchanged associated result as an opaque
inference value. Equal result identities can flow through expressions without
equating their operation inputs. Each deferred occurrence retains its original
application and formation obligation separately from the shared result. Result
publication wakes dependent equations; changing lifetime inputs grants no new
normalization authority. Ordinary known types retain their existing fast path.

Written body types use their declaration environment; free caller types use the
existing aliased environment. The first complete regression run exposed duplicate
original/aliased assumptions in `derived_permissions`. That failed run remains
under `full-v10-failed/`; the final implementation fixes environment selection
without changing the existing test expectation.

Concrete instantiation normalizes the substituted signature, typed CFG roots and
lifetime contract in one fixed session using validated implementations. It keeps
selection debts even when a result erases its input lifetimes. Normalized
signatures must still agree before layout and destruction selection. Three typed
instance controls include forged mismatched signatures and an erased input whose
selected implementation requires a static lifetime. Existing projected type debts
under non-root quantified proof scopes retain an explicit unsupported gate:
creating sibling binders would not preserve the original private variables.

`opaque_body_values.rs` contains 26 declaration/body type-checking controls,
fifteen LowIR controls and six native callable programs. Early type checking does
not claim ownership, lifetime or destruction acceptance. Five inference invariants
check separate occurrence authority, wakeups, result identity and lifetime mapping.
Independent Rust 1.98 object checks qualify the reviewed language outcomes before
maintainer expectations are added. Nine new scaling inputs, at 64/256/1,024
elements or instances, also pass rustc object generation and RustGM LowIR emission.
Additional valid borrow/captured-callable investigations that still fail are kept
as investigation evidence, not successful coverage or course fixtures.

The final immutable compiler is
`7ec7ef7b3ed94f8dd1ca57586e3355d739938bf78302fc0961ab5e3c0389d30f`.
All 25 final commands pass: strict Clippy, 1,108 release tests in 204 groups,
4,864 PA1–PA19 cases, backend qualification and 52 maintainer native test groups.
The syntax-aware audit checks 684 Rust files with zero findings; all 33 harness
checks pass. The 687-file source snapshot matches the validated tree. Standalone
lint passes. Validation cleanup removes 4,541 generated files (4.75 GiB of
unshared blocks); the following tidy removes another 3,217 files (1.92 GiB).

General owned associated values still need opaque lifetime/drop components and
result Copy/Clone permissions. Nested result bounds, lifetime-equivalent inputs
and extensions of existing quantified instance proofs also remain unfinished.
PA20/PA21, their course fixtures and their release statuses remain planned.

Performance evidence retains 106 jobs, 2,756 study observations and 424 full-output
qualification observations. The initial 50 comparisons use published `01163cb`;
eighteen identical-candidate controls measure the nine newly supported scaling
inputs without claiming a speedup against an unsupported baseline. The predeclared
3.5% repeat rule selects six families and all 38 of their sizes/modes. Every binary,
source and input hash stays fixed, and all full/count output comparisons agree.
No local builds or qualification probes overlap the timed work.

Initial paired medians range from −1.89% to +11.94%; confirmations range from
−1.03% to +10.65%. Confirmation A/A spreads range from 0.39% to 120.68%, and RSS
differences range from −392 to +564 KiB. Twelve confirmation jobs remain above
3.5%, including repeated inferred applications and the largest shared-input call.
These are recorded overhead, not a clean performance pass. The independent
occurrence proofs are required for correctness; reusing inference-resolution
scratch is a follow-up investigation, without discarding these observations.
At 1,024 elements, the new repeated-value, wide-tuple and distinct-instance full
views take 38.59, 102.94 and 122.35 ms respectively in the initial study. All
evidence, including original failures and scaling qualification, is retained in
`obj/pa20-work/opaque-body-values-v1/`.

## Reuse bounded inference resolution scratch

Demand resolution now retains collection capacity within its inference body.
It clears every result, active marker, pending node and child after successful
or failed traversal. Known types allocate no scratch. A later equation or
projection publication must be observed by the next request; no semantic result
cache survives between requests. Hash tables disproportionate to the current
traversal are discarded before clearing, avoiding repeated large clears after a
wide query. Vectors retain their spare capacity.

Three new invariants cover failed traversal followed by unrelated work and later
progress, a composite type changed by projection publication, and bounded hash
storage after wide/deep queries followed by small successful/failed requests.
Source review stopped the first prototype's release build to address the large
table clearing issue. Its interrupted logs and snapshot remain under
`unbounded-capacity-interrupted/`; no completed validation is claimed for it.

The final compiler hash is
`7b65edeb261d54cf666e633fbee69ad9d567a955bb196f14553c8bf2fba791b2`.
All 25 validation commands pass: 1,111 release tests in 204 groups, 4,864 course
cases, backend qualification and 52 native test groups. Strict Clippy, standalone
lint, the 684-file audit with zero findings and all 33 harness checks pass. All
687 frozen source hashes match. Cleanup removes 4,532 generated files (4.75 GiB
of unshared blocks). Eight superseded opaque-value prototype binaries are retired
with hashes and receipts after published `923d569` passes exact-head CI.

The performance study retains 170 jobs, 4,420 study observations and 680 full-output
qualification observations. It compares fifty jobs with pre-feature `01163cb` and
68 with published `923d569`, including the nine new opaque-value scaling inputs.
The predeclared rule repeats all sizes/modes for each affected comparison/family:
38 confirmations against `01163cb` and fourteen against `923d569`. All input,
binary and source hashes and every full/count output comparison agree. No local
build or qualification probe overlaps timing.

Against `923d569`, initial paired medians range from −10.29% to +7.53%; the
fourteen confirmations range from −7.00% to +4.73%, with one small receiver count
case above 3.5%. Confirmation A/A spreads are 1.38–64.64%, and RSS differences
are +60 to +236 KiB. Repeated inferred applications improve in all six initial
comparisons, including −4.09% at 1,024 applications in count mode. This is a
targeted allocation improvement, not a uniform speedup or zero-overhead claim.
The pre-feature confirmations still range from −7.27% to +9.00%, with seven jobs
above 3.5%; the earlier occurrence-proof overhead remains an open performance
concern. Raw evidence and all original outliers remain under
`obj/pa20-work/resolution-scratch-v1/`. PA20 and PA21 remain planned.
