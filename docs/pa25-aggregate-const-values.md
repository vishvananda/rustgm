# Canonical aggregate const values and evaluator handoff

Original core's `TransmuteFrom<Src, const ASSUME: Assume = { Assume::NOTHING }>`
requires a genuine structured const argument. `Assume` is a real four-boolean
struct with source-owned trait implementations, not a compiler bit mask. PA25
owns evaluation/value production; PA21 owns typed generic arguments, PA29 owns
transport and PA31 owns the actual source language bindings. All these parent
requirements remain open.

The canonical representation, evaluator/reader handoff and local source parameter
profile are implemented. Actual source providers can use struct, enum, tuple and
fixed-array parameters after real feature and marker proofs. The source
transmutability relation, broader value profiles and imported transport remain
unimplemented. No original crate artifact or whole assignment is completed.

## Representation

`ConstKind::Aggregate` holds a dense canonical node identity alongside existing
scalar and parameter cases. Ordinary scalar bits are never repurposed as object
IDs. A cold optional owner beneath the already optional argument storage retains
typed products and immediate canonical child IDs. Nominal type, enum variant,
field order and scalar type all participate in identity. Nominal field checking
belongs to the semantic producer; these data constructors establish no feature,
`ConstParamTy`, layout, visibility or caller permission.

Parents are appended only after their complete children exist. Interning hashes
immediate IDs and compares complete slices on collisions, so equality reuses
canonical identities rather than recursively expanding a value. A 128-level
binary shared product retains 128 nodes and 256 child edges. Canonical repeated
arrays retain one child even at length one billion. Explicit uniform arrays and
compressed repeats normalize to the same identity; empty arrays do not retain an
irrelevant repeated element.

The producer charges validation, collision probes and prospective capacity
growth before publication. Warm lookups charge work and allocate no additional
payload. Bad identities, sequence types, unclosed types and failed payments
cannot become canonical value nodes. Reusable field slices may survive a later
failed publication after their allocation has been paid; they grant no value or
semantic authority. Node/edge limits remain finite.

The payload belongs to its const-value producer. It is excluded from unrelated
trait-input storage charges and argument-cache invalidations. Existing canonical
constant IDs still participate in their normal store extent. No field is added
to the outer type store, declaration result, body or execution frame. The cold
argument-storage owner gains one optional pointer. Existing scalar APIs remain
scalar; an aggregate cannot masquerade as an array length or scalar VM operand.
Old metadata formats remain closed to aggregate arguments.

## Evaluator and reader boundary

Both ordinary and suspended generic-value queries publish a `ConstId`, rather
than scalar bits that callers reinterpret. Scalar results retain their direct
path without graph scratch. Array lengths still require an actual `usize`
scalar. Closed aggregate operands are distinct from unresolved parameters;
existing caller, body, initialization, lifetime and source permissions remain
separate checks.

Completed products are converted iteratively using memoized `(TypeId, Constant)`
identities and their published field recipes. The producer verifies exact field
arity, scalar kind and actual nominal/enum ownership before interning children.
It preserves compressed repeats, including empty arrays, and rejects unsupported
pointer, union and other nonstructural value profiles. Initialized byte-array
images must satisfy their existing publication check. Each conversion has its
own finite work/storage budget, independent of source AST size; actual source
contexts additionally pay their completed local header producer before growth.
Failed work remains charged and cannot publish a completed root.

`Constant::argument` returns a compact canonical aggregate handle without
reconstructing the product arena. `Constant` remains 32 bytes. Shared readers
resolve immediate fields and actual declaration-owned variants on demand, retain
exact storage-type checks and preserve out-of-bounds faults. Pattern expansion
memoizes canonical handles alongside ordinary product identities; struct updates,
enum casts and symbolic projections use the same typed reader. Immutable operands
can initialize independent VM allocations: reads share the operand, while a field
write thaws only that allocation. Existing raw-write authority and expired-storage
checks still apply at their normal execution boundaries.

## Source declarations and actual marker proofs

Header registration reserves a typed rigid parameter without treating it as an
evaluated value. Tuple/array/ADT identity and inference support supplies no source
permission. Semantic registration separately requires the real
`min_adt_const_params` or `adt_const_params` request for aggregate types and rejects
dependent parameter types, projections and free regions outside this profile.
After all ordinary implementation validation completes, every local const
parameter under that feature, including unused and primitive parameters, must
prove the validated source `ConstParamTy` application. Closed types cannot justify
themselves with a caller's assumed trait premise. Proof scratch is shared within
one owner's environment; the declaration inventory is traversed once.

The actual marker header retains its required `StructuralPartialEq` and `Eq`
identities. Its implementations inspect every real struct/enum variant and
substituted field through the existing shape producer. Each child needs the same
actual marker; selecting a benign active variant cannot hide an invalid sibling.
`PhantomData` retains its existing empty physical fields and separate logical
ownership. Field/generic derives now use the shared constrained header writer and
ordinary implementation validation. Generated syntax confers no marker permission.
The former Rust-positive `wider-generic` and `nominal-const` fixtures are accepted;
a derived `Payload<Unkeyed>` consumer still fails the actual child requirement.

Both aggregate feature spellings are language requests even when a provider
declares no unstable library API. They do not add library policy definitions:
`implied_by` still requires its real target. This distinction is covered alongside
the existing staged-policy opponents. No global or transitive feature premise is
created by a relationship.

Runtime const operands use the common typed constant initializer, keyed by their
canonical `ConstId`, with fresh owned snapshots where required. Semantic generic
substitution likewise retains the typed value. Fieldless enum casts consume the
actual reader variant rather than assuming an arena-specific representation.

`pa25/tests/spec/aggregate-parameters/contract-v1.json` declares fourteen cases
before candidate/reference qualification. Eight positive native observers cover
both gates, records, enums, four-boolean `Assume`, arrays, tuples, an actual forward
default used by an associated call and const-function local storage. Six opponents
cover a missing gate, missing or same-spelling counterfeit markers, an invalid
field, a primitive lacking its real marker and an unused invalid declaration.
Every source is listed in the existing const-machine manifest; its parent PA stays
planned. The course runner's explicit `source_provider` context retains the
checked provider identity, preserving the fixture bytes. These fourteen text
cases select constant inspection without requesting library artifact publication;
their genuine LowIR/native executions use the full maintainer pipeline. General
nominal metadata publication remains an explicit PA29 requirement.

Independent pinned Rust object compilation and C-linked execution pass all
fourteen outcomes and all eight exact native observations (`7`, `7`, `9`, `11`,
`13`, `15`, `17`, `19`). Receipt:
`obj/parallel/aggregate-parameters-v1/reference-native-v3/receipt.json`.
Private gate `1793730897305304114` passes twelve derive, caller, policy, failed-work
and ordinary-root controls in 94.19 seconds with strict private Clippy, formatting
and architecture checks. The final compiler snapshot passes production34 and
PA18 502/502 plus PA21 758/758 in 132.87 seconds (`1793732280416643837`), private12
in 67.77 seconds (`1793732438140196812`), and the eight native observations in
77.41 seconds (`1793732519646561584`). The combined cold predecessor exceeded
its five-minute link budget and is retained as a failed run, not qualification.

The final course pass compares all 109 cases in 1.87 seconds; only the new
fourteen oracles were generated, and all 285 earlier sidecars remain byte-equal.
All 276 harness tests, suite registration, placement and pinned-source checks
pass. The source-provider adapter now separates semantic inspection from artifact
publication. This Python/manifest correction is qualified by the actual CLI and
harness pass; the joined receipt records the exact two adapter/manifest deltas
and verifies that compiler and Rust-test sources did not change.

Candidate CLI SHA-256 is
`466cf52ed74ecabfd7ba6a1f618a450f5b19564698dd5c2c27ad8e750a0954af`.
The independent/native/golden/course/harness and source-matched gate join is
`obj/parallel/aggregate-parameters-v1/joined/receipt.json`, SHA-256
`5dead2a35f6926c675a020b57cc4c66b674621c3ab7a3bc39bbcf87a6320c2a2`.
Its 112 immutable A/A and ABBA observations preserve output equality against
the previous evaluator CLI. Median candidate/baseline ratios are 0.937 for the
ordinary control and 1.017 for 64 scalar arguments. A/A spans 0.981–1.337, so
these noisy process measurements establish no precise speedup or regression.
The source algorithm reuses one proof per owner and traverses the declaration
inventory once; existing shared-value/repeat bounds remain covered separately.

## Focused evidence

`obj/qualification/fast/1793720963537416621` passes five exact private tests in
121.09 seconds, with strict private Clippy, formatting and architecture checks.
The controls cover nominal/variant/type identity, uniform and compressed arrays,
deep sharing, bounded warm work, unchanged unrelated trait storage, failed payment
and malformed input. The ordinary-source catalog control remains present.

`obj/qualification/fast/1793721130316649825` passes nine existing product/operand
tests, strict production Clippy and PA21 758/758 in 202.68 seconds. This includes
independent Rust comparisons, generic operand substitution, primitive const
kinds, checked pattern borrows and bounded repeats. Every local batch uses the
existing five-minute guard, two threads, 16 GiB output cap, 150 GiB reserve and
warm-cache cleanup. No release build or broad native sweep is added here.

The evaluator successor's private gate
`obj/qualification/fast/1793724807787733625` passes 11 exact controls with strict
Clippy, formatting and architecture checks in 119.22 seconds. They cover actual
source struct/enum evaluation and owner mismatches, suspended initializer queries,
source growth/warm/failed payments, billion-element repeats, failed publication,
compact values and VM mutation/permission/liveness. Converting a 128-level binary
shared value completes below 32,768 charged steps and publishes at most 129 const
identities, rather than traversing its exponentially expanded tree. Reusing its
canonical root takes one local work step.

`obj/qualification/fast/1793724963740747337` passes 22 affected production tests
and PA21 758/758 in 117.65 seconds, including nominal/product patterns, updates,
discriminants, symbolic operands and real source wrapper/NoDrop/MaybeUninit
controls. Native gate `obj/qualification/fast/1793725167262241328` passes three
exact tests (eight actual executions) in 33.65 seconds: the three MaybeUninit
value/destruction/panic observers, four source wrapper observers and the NoDrop
mutation/ownership observer. Earlier failed private checks exposed test setup errors; the qualified
tests use actual unfinished initializer readiness, actual payer validation costs
and the VM's existing separate raw-write authorization boundary.

Immutable candidate
`b77fd3e812408150a2ebcd1b9a131f43bcd0c0645e7625b376439bcfc5ae0e44`
is compared with the preceding native checkpoint's immutable `f5ae84eb` binary.
Two frozen inputs each retain three A/A pairs and three paired ABBA blocks, with
every observation and equal output checks in
`obj/parallel/aggregate-handoff-v1/perf/receipt.json` (SHA-256
`fecc1a0de5bbeae26344aaf40a376270165ab1684f4d94d41a7251a13b798315`).
The ordinary-control median candidate/baseline ratio is 0.985; 64 scalar const
arguments give 1.025. A/A ratios range from 0.924 to 1.015 across these short
process measurements, so no speedup or precise regression conclusion is claimed.
The graph/repeat work bounds directly qualify the new conversion algorithm.
This candidate has not been used for a new original-core probe; the prior
`f5ae84eb` original-core frontier remains the only such evidence here.
The joined source/binary/three-gate/performance receipt is
`obj/parallel/aggregate-handoff-v1/joined/receipt.json`, SHA-256
`d052f68d58b1816f1fe0985d5bb26c1279f2cb1c17047f278729ad6264e7093c`.

## Remaining implementation order

1. Complete the remaining const-parameter profiles: source macro allowance
   contexts, unsized/string/reference values, dependent parameter types and wider
   recursive/transport contracts. Root-provider aggregate admission and real
   marker/default readiness are qualified independently of those requirements.
2. Implement genuine `transmute_trait`/`transmute_opts` declaration authority and
   the coinductive transmutability relation over source/destination layouts,
   validity sets, padding, references and lifetimes under the actual `Assume`
   value. Reinterpreting one value successfully is not proof for all values of a
   source type. Preserve the actual unsafe trait and source default method;
   registering the language-item spellings alone does not close LIB-124.
3. Transport actual nominal declarations, structured constants, generic defaults
   and body inputs through a versioned PA29 profile with deterministic identities
   and fresh consumers. Remap type/value/variant identities once at the boundary;
   old scalar-only formats must not encode aggregate IDs as scalar bits. Original
   core, alloc and std artifacts and all assignment release requirements remain.
