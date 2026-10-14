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
