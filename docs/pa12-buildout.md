# PA12–PA15 implementation handoff

PA12 is now an implemented draft; [its validation record](pa12-validation.md)
and handout publish the grading contract. PA13–PA15 remain planned. This file
retains the implementation history and continuation requirements; the active
objective includes all four assignments.

## Completed aggregate foundation

`types::Kind::Adt` carries a nominal declaration identity. Field definitions
live in `types::aggregate`, separately from canonical identity, so resolving
`*const Node` does not demand `Node`'s complete storage. Declaration checking
reserves identities, elaborates field annotations through the existing alias
worklist, publishes shapes and constructor identities, solves discriminants,
and then checks every aggregate's layout. Direct, indirect and zero-length-array
by-value cycles are rejected.

`sema::members` retains flat variant/field arenas, source provenance, canonical
field types, constructor identities and resolved field access. Member names use
their own NFC interner; its IDs must not be confused with lexical binding name
IDs. Tuple constructors have function-item types with canonical signatures.
Expression checking consumes these facts for tuple/array/struct construction,
unit and tuple constructors, field projections, indexing and struct updates.
It retains selected fields, variants and array bounds on the expression facts,
including private-field checks and expected-type/function-pointer coercions.
The dense node fact remains 80 bytes.

Representation hints consume configured attribute token trees, including
selected `cfg_attr` and macro-generated declarations. Supported hints include
C, Rust, transparent, integer enum tags, packed and align. The initial nominal
profile requires concrete sized fields. Explicit discriminants currently accept
typed scalar constant expressions, including checked integer arithmetic,
floating arithmetic, casts, booleans, characters, lazy operators, conditional
expressions, expression-only blocks, named constants, immutable Copy statics
and casts of field-less variants. Duplicate and implicit-successor checks use
their evaluated target-width values. General constant functions, statements,
loops and allocations remain PA25 work.

`abi::layout` is an iterative, cached x86-64 query. It retains source-order
fields for unspecified Rust layout, honors guaranteed C/transparent/packed/tag
representations, checks target-size arithmetic, and distinguishes zero size
from uninhabitedness. Array layouts retain element/stride/count rather than one
entry per element. Pointers break storage dependency cycles. There is no niche
optimization. The cache belongs to one type store and immutable completed
definitions, and caches failures as well as successful layouts.

The typed LowIR builder/writer supports positive-sized object slots, scalar
pointer indexing and object transfers. Object extents/alignment are interned
per function; scalar instructions do not carry full memory-span payloads.
`copy_object` requires the caller to prove non-overlap and actual address
alignment. Rust zero-sized values must not be represented as zero-byte LowIR
object slots. These operations use the existing qualified C++ backend without
any backend source or lock changes.

The typed body IR now has a flat projection arena and an explicit place-formation
action. Field access does not first read its entire aggregate. Index expressions
are evaluated once; their bounds check belongs to place formation, including
wildcard discards. Primitive assignment evaluates its RHS before the LHS place.
Struct updates evaluate explicit fields before the base and read only inherited
fields. Constructor functions are built directly from typed member facts in
declaration order, without synthesizing or reparsing source. Array repeats keep
one operand and reuse the length in their canonical array type, including
enormous zero-sized arrays. Ordinary body values remain at most 24 bytes.

Place construction uses a reusable operand stack rather than another dense
per-syntax-node side array. Unreachable work still balances that stack; tests
cover divergence in RHS expressions, indices, updates and nested branches.
Aggregate expressions are available in the type/body inspection views. The
executable path now consumes the owned-value proof below for both scalar and
aggregate bodies. The former scalar-only checker has been removed.

## Owned-value proof

`types::copy` caches closed-profile Copy/Clone obligations independently of
layout. Tuples and arrays inherit their fields' abilities, including zero-length
arrays. Nominal types require explicit built-in derives. Shared references,
raw pointers and function values are Copy; mutable references are not.
Retained `derive(Clone, Copy)` attributes use the same configured token-tree
pass as representation hints. Field obligations, duplicate implementations,
the Clone prerequisite and packed-struct Clone/Copy restrictions are checked
even on unused declarations. This does not implement general trait selection,
Clone method calls, imported/qualified derives or procedural derive macros.

`borrow::ownership` interns sparse local/temporary field paths and numbers
subtrees into bitset intervals. One shallow bit represents an initialized
aggregate structure and its unmentioned fields. A field move clears its subtree
while retaining its parent's structure; a whole move clears both. Field writes
require an initialized parent. This permits partial reinitialization without
accepting writes into wholly moved or never-initialized aggregates. Arrays stay
atomic for ordinary indexing; their lengths never expand the path arena.

The CFG fixed point intersects definitely initialized bits and unions possibly
written bits. Moves do not reset immutable-write history. Storage markers reset
state for fresh loop iterations; temporary snapshots gain state on evaluation.
Diagnostics inspect converged reachable states. Budgets account for actual
bitset words touched and bound both state memory and solver work. Ordinary
indexing requires an initialized array and cannot move a non-Copy element.
Repeats evaluate their operand once, including lengths zero/one; larger repeats
require Copy or a constant operand. The proof retains selected Copy/Move uses.
The repeat bound is also discharged by semantic analysis before unreachable
body code is discarded: Rust requires this type obligation even after `return`.
Pinned-rustc controls cover divergent operands and unreachable repeats, while
ordinary unreachable move/read checks retain their CFG behavior.

`--emit-ownership` and `--count-ownership` expose this proof for maintainer
inspection. These are provisional views while PA14 remains planned. They do
not discharge reference adjustments, loan or region obligations. Executable
lowering accepts only the checked program witness, then separately checks its
demanded storage representations before erasing source values.
Inspection returns an `Analysis` containing the checked witness and per-body
records. Executable verification uses the identical checking engine but releases
each body's inspection records immediately; lowering needs the witness rather
than retaining all move-path arenas beside its growing output.

## Executable aggregates

`lower::function::aggregate` lowers owned tuples, arrays, structs and enums
through the typed LowIR object operations. By-value reads create independent
snapshots; constructors and calls produce fresh storage. Assignments, callee
parameter initialization and returned values copy only between these distinct
allocations. Copy/move legality comes from the preceding proof, not from the
physical `copyobj` operation. Transfers retain the actual minimum address
alignment, including packed fields.

The private calling convention passes nonzero aggregate arguments as pointers
to evaluated snapshots and nonzero aggregate results through a hidden first
output pointer. Function pointers use the same signature transformation.
Zero-sized parameters/results carry no physical payload, while preserving
evaluation effects, moves, initialization and array bounds. This is RustGM's
internal convention, not a claim about rustc's unspecified native ABI.

Field offsets consume retained member ordinals and cached layouts. Array place
formation emits an unsigned bounds guard, including wildcard discards and
zero-sized elements. Repetition emits a constant-size loop for nonzero storage;
huge zero-sized arrays require no loop or object allocation. `Option<Selected>`
remains at most 24 bytes, so aggregate addresses do not widen ordinary snapshots.

Field-less enum casts retain a semantic eligibility fact, honoring the rule
that only unit variants may have explicit discriminants when non-unit empty
variants are present. Lowering reads the selected physical tag, or materializes
the fixed discriminant of an untagged single-variant enum, then uses the existing
integer conversion rules. Layouts also retain a by-value `contains_reference`
fact. A nested reference cannot evade the pending ownership/region boundary by
being copied as aggregate bytes, including inside a zero-sized array.

## Typed discriminants

The discriminant query engine reuses the ordinary body type checker. Constants
already checked for a query retain that fact and are not inferred again during
body analysis. Expression scratch is allocated lazily per source file and reused
across queries. Constant eligibility validates both conditional branches, while
evaluation visits only the selected branch. The explicit work stack and sparse
query cache include constant/variant cycle detection and an input-scaled work
budget. Implicit-only enums do not allocate a constant expression checker.
Nominal shapes and constructors precede evaluation; layouts cannot cache
placeholder discriminants. No source is reparsed and scalar operations consume
the existing selected types/casts/literal bits.

The scalar discriminant matrix includes independently executed rustc values,
invalid cycles and arithmetic, 10,000 chained constants and 5,000 implicit
variants. Four additional source programs execute through both rustc and the
pinned backend, covering folded tags, cross-enum signed casts, lazy branches,
floating saturation and forward references. Evidence is retained in
`obj/pa12-work/discriminant-vectors/` and `discriminant-native/`.

Those tests exposed a resolution defect: enum member bindings were incorrectly
considered lexical bindings inside discriminants, shadowing outer constants and
types. Ordinary and hygienic lookup now share the rule that direct variant
members require qualification; explicit imports remain lexical. Four declared
PA6 fixtures and one PA8 expansion fixture own the regression. Their independent
checks pass (170 PA6, 88 PA8); the existing documented PA6 phase difference is
unchanged. No previous fixture sidecar changed.

## Evidence at this checkpoint

- All 1,547 pre-existing PA1–PA11 fixture outputs are unchanged. Five new
  PA6/PA8 name-resolution regressions raise the active total to 1,552.
- Debug and release suites each pass 240 Rust tests. The declaration acceptance
  matrix checks the pinned rustc with default lints, including representation
  conflicts, duplicate fields, recursive storage and discriminant failures.
- Guaranteed layout vectors compare sizes, alignments and field/payload offsets
  independently with rustc. Deep nesting checks 20,000 layouts without recursive
  traversal, and enormous zero-sized arrays require constant layout work.
- Aggregate expression cases independently check rustc acceptance. Body
  invariants check once-only index evaluation, RHS-first assignments, inherited
  field reads, generated constructor bodies, wildcard places and control-flow
  divergence. A 10,000-field projection chain constructs iteratively and reads
  only its selected leaf.
- The ownership matrix checks 74 declared acceptance/rejection cases against
  pinned rustc. A deterministic additional differential run checks 256 operation
  sequences with straight-line, branch and loop flow: 177 accepted, 79 rejected,
  no disagreement. Sources and both diagnostics remain in
  `obj/pa12-work/ownership-differential/`. Structural tests cover 20,000 nested
  Copy queries, cached failures, 10,000 projections, 40,000 independent locals,
  repeated projections and an array of `u64::MAX` zero-sized elements.
- Strict Clippy/rustfmt pass. The file/phase audit checks 160 Rust files with
  zero findings; two audit tests and 19 harness tests pass.
- The typed object-memory probe in `dev/tests/lowir_lowering.rs` writes
  `obj/pa12-work/memory.lowir`. The pinned bridge validates it and its executable
  returns exactly 42 with empty streams. The existing backend smoke check also
  passes. A separate address probe verifies actual 64-byte object-slot alignment.
- Twenty-four Rust aggregate programs independently compile and execute through
  rustc and the pinned C++ backend with matching clean exit zero. They exercise
  snapshot order, direct/indirect calls, packed and over-aligned objects, nested
  arrays, struct updates, scalar field access, repeats, zero-sized operand
  effects and enum casts. Eight additional bounds probes match the qualified
  panic-abort behavior, including wildcard, zero-sized and `u64::MAX` cases.
  Evidence is in `obj/pa12-work/aggregate-native/`, `aggregate-bounds/` and
  `lowering-vectors/`. The complete PA11 native suite was rerun: 194 independent
  checks, including all 180 executable comparisons, pass.

Raw logs and frozen binaries are in `obj/pa12-work/`. Existing oracles remain
authoritative until a new assignment has declared outcomes, independent source
execution checks and reviewed generated text.

The layout-foundation candidate SHA-256 is
`48ab2428f14eacdff8ff084cf98efd9ccb77f210e458e132b4542ab0278471c7`.
LowIR performance uses the published PA11 binary as baseline, CPU 10, frozen
inputs, A/A calibration and three ABBA blocks at each scale. All 54 observations
and complete-output hashes are retained in
`obj/perf/1791813976149095594/samples.json`. Baseline/candidate paired median
times for 4,000, 16,000 and 64,000 demanded procedures are 195.66/194.62,
773.98/759.34 and 3461.89/3442.62 ms. Peak RSS at the largest scale is
693,480/695,480 KiB. Outputs are identical; these observations show no measured
time regression and a small memory increase, not a claimed speedup.

The subsequent expression/body candidate is frozen as
`947c986c71d9981fe2d5625f854c9b31fbb900363eb99dbde247003dccb35dda`.
Its baseline is the layout-foundation candidate above, with the same immutable
inputs, CPU pinning, A/A calibration and three ABBA blocks. At 64,000 procedures,
body construction baseline/candidate medians are 2799.68/2793.55 ms and peak RSS
543,332/544,328 KiB. End-to-end LowIR medians are 3356.62/3445.17 ms and peak RSS
695,512/696,928 KiB: about 2.6% additional elapsed time in that observation.
Every count and full output hash agrees. All 108 observations remain in
`obj/perf/1791816073922580496/samples.json` and
`obj/perf/1791816151123874056/samples.json`.

Earlier measurements exposed a redundant repeat-length field that widened all
body values. Removing it restored compact common records and substantially
reduced memory overhead. Those pre-fix observations are retained too, in
`obj/perf/1791815793886761711/samples.json` and
`obj/perf/1791815869736630624/samples.json`; they are not the final candidate.

The ownership checkpoint binary is
`3fe4f24061576023d1f9fb1ae2254779870f916f01ad9e40ab233fcc99d913f8`.
Compared with the expression/body candidate, LowIR baseline/candidate medians
at 4,000, 16,000 and 64,000 functions are 201.22/201.97, 805.58/806.60 and
3411.84/3397.71 ms. Largest-scale peak RSS is 696,928/696,968 KiB. Counts and
complete output hashes agree at every scale; the observations show no measured
regression, not a claimed speedup. All 54 samples remain in
`obj/perf/1791817977115983992/samples.json`.

`make perf-ownership` exercises partial moves, branch reinitialization and
consumption of distinct nominal fields. A/A scaling with the same frozen
binary, CPU 10 and three paired blocks gives 261.50, 1091.65 and 4444.72 ms for
4,000, 16,000 and 64,000 functions; peak RSS is 55,280, 209,780 and 825,932 KiB.
These measure the entire frontend through ownership, not the solver alone.
All 54 observations and full-output checks remain in
`obj/perf/1791818122849043847/samples.json`. There was no preceding ownership
binary, so this is scaling evidence rather than an A/B regression claim.

The aggregate-lowering checkpoint binary is
`53a6deffe50d39001774c0bbe27bcec7919b1f119df4528819775a4c1dfad39d`.
Against the published ownership checkpoint, scalar LowIR baseline/candidate
medians at 4,000, 16,000 and 64,000 functions are 196.67/208.72, 827.64/840.07
and 3458.12/3572.63 ms. Largest-scale peak RSS is 696,928/698,344 KiB.
These observations include about 3.3% elapsed overhead at the largest scale;
they do not establish a speedup. Counts and full output bytes agree. The same
CPU pinning, A/A calibration and three ABBA blocks retain all 54 observations
in `obj/perf/1791821870358292310/samples.json`.

The initial executable aggregate candidate retained ownership inspection arenas
through lowering and increased largest-scale scalar peak RSS by about 28 MiB.
Streaming the identical proof and releasing each body's records removes most
of that increase. Copy-cache hits also avoid resetting query scratch, and
root-only move paths avoid allocating a traversal stack. Intermediate binaries
and every earlier observation remain in `obj/pa12-work/` and
`obj/perf/1791820338675201752/`, `1791820505192926655/`,
`1791821183202994711/` and `1791821293117823723/`; they are not the final
candidate. The aggregate workload is selectable with
`scripts/measure_lexer.py --stage lowir --aggregate-input`.

The final aggregate A/B run compares the initial executable candidate with
this checkpoint at 1,000, 4,000 and 16,000 demanded procedures. The
baseline/candidate medians are 86.62/83.08, 370.19/378.56 and
1480.62/1510.16 ms. Largest-scale peak RSS falls from 321,904 to 310,528 KiB,
with about 2.0% measured elapsed overhead. Complete LowIR outputs agree at every scale. All 54 samples,
including both baselines and calibration, remain in
`obj/perf/1791821983967599492/samples.json`. This measures the entire frontend
through LowIR, not just the ownership solver or object transfers.

The typed-discriminant checkpoint is frozen as
`d1fc49196f692158451a9480f946995f79638031080a3e5aefe2976ff9ca8748`.
Against the aggregate-lowering checkpoint, scalar LowIR baseline/candidate
medians are 206.79/208.00, 840.29/843.34 and 3491.34/3498.57 ms at 4,000,
16,000 and 64,000 procedures. Largest-scale peak RSS is 698,332/698,368 KiB;
the elapsed medians differ by about 0.2%. Counts and complete outputs agree.
All 54 observations, A/A calibration and frozen input hashes remain in
`obj/perf/1791823200682952616/samples.json`.

The new scalar-discriminant workload uses 1,000, 4,000 and 16,000 enums,
with named constants, selected arithmetic, casts and cross-variant dependencies;
`--discriminant-input` selects it in the measurement script. End-to-end LowIR
A/A medians are 44.02, 187.30 and 860.61 ms, with peak RSS 12,824, 40,876
and 146,212 KiB. All 54 observations and full-output checks remain in
`obj/perf/1791823294852868003/samples.json`. The preceding compiler did not
support these inputs, so this is scaling evidence, not an A/B regression claim.

The PA6 name-resolution A/B run also preserves counts and complete output
bytes. At 4,000, 16,000 and 64,000 functions, baseline/candidate medians are
51.70/52.38, 206.98/207.00 and 881.43/893.34 ms; largest-scale peak RSS is
138,272/138,316 KiB. All 54 observations and calibration remain in
`obj/perf/1791823353138107656/samples.json`.

## Required continuation

1. PA12 is published with 110 declared fixtures, text oracles, independent
   aggregate execution and a deterministic layout inspection contract. Preserve
   those behaviors while extending the same typed facts in subsequent stages.
2. PA13: use the same field/variant identities for binding modes, match CFGs,
   guards, or-patterns, slices and usefulness/exhaustiveness. Retain move and
   reference semantics for the subsequent checks; do not erase unproved borrows
   merely to reach LowIR.
   The concrete sequence and completed binding-identity prerequisite are in
   [the PA13 handoff](pa13-buildout.md).
3. PA14: extend the owned-value proof to PA13's pattern projections and binding
   modes, and retain maybe-initialized path facts separately from write history.
   Freeze the ownership view, publish assignment fixtures and their
   independently checked outcomes, and integrate the executable proof boundary.
4. PA15: discharge lifetime contracts through region parameters, elision,
   outlives constraints and caller/callee relationships, including nested
   reference returns. Full loan checking remains PA16's responsibility.

PA12's handout freezes the scalar constant-expression profile, including `Self`
in nominal declarations and literal/grouped array lengths. Its declared negative
controls keep PA25's full evaluator boundary explicit.

Keep ordinary indexing separate from PA13's pattern projections, which
will need constant-index/downcast path identities. Reference adjustments and
region obligations need their own checked facts before reference erasure. The
current reference rejection gate must be replaced by these proofs, not removed
to make reference examples compile.
