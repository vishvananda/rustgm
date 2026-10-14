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
are evaluated values at this checkpoint; const parameter terms are still needed.

Source type/const generics are still rejected at the existing semantic boundary.
The new representation alone does not instantiate field definitions, prove
generic bounds or produce native code. Layout, Copy/equality and destruction
queries reject mixed-argument nominal types until instantiated fields and
obligations are supplied. No course references are regenerated for this change.

## PA18: generic templates and demanded instances

1. Retain type/const parameter identities by owning declaration and ordinal.
   Validate kinds, declaration order, defaults, arity and const parameter types.
   Resolve each syntax argument once, including inferred arguments and the
   type/const ambiguity of single-segment paths. Distinguish early lifetime
   arguments from late callable binders.
2. Elaborate type and const parameter terms and substitute them through types,
   signatures and aggregate field templates. Cache substitution in one argument
   environment. Keep lifetime shifting correct through nested callable binders.
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
