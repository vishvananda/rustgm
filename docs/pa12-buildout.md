# PA12–PA15 implementation handoff

PA12–PA15 remain planned. This records the maintainer's implementation in
progress; it is not a released student contract or a claim that these stages
pass. The active objective includes all four assignments.

## Completed aggregate foundation

`types::Kind::Adt` carries a nominal declaration identity. Field definitions
live in `types::aggregate`, separately from canonical identity, so resolving
`*const Node` does not demand `Node`'s complete storage. Declaration checking
reserves identities, elaborates field annotations through the existing alias
worklist, publishes shapes and then checks every aggregate's layout. Direct,
indirect and zero-length-array by-value cycles are rejected.

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
integer literals and unary-negative literals, with suffix/range, duplicate and
implicit-successor checks. General constant expressions are still pending.

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
scalar executable proof still rejects aggregate values and projections until
the owned-value proof below is connected to aggregate lowering.

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

`--emit-ownership` and `--count-ownership` expose this proof for maintainer
inspection. These are provisional views while PA14 remains planned. They do
not discharge reference adjustments, loan or region obligations. The existing
scalar LowIR boundary remains until aggregate lowering consumes the new proof;
successful inspection alone does not enable executable aggregate output.

## Evidence at this checkpoint

- All 1,547 PA1–PA11 fixture outputs are unchanged.
- Debug and release suites each pass 233 Rust tests. The declaration acceptance
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
- Strict Clippy/rustfmt pass. The file/phase audit checks 156 Rust files with
  zero findings; two audit tests and 19 harness tests pass.
- The typed object-memory probe in `dev/tests/lowir_lowering.rs` writes
  `obj/pa12-work/memory.lowir`. The pinned bridge validates it and its executable
  returns exactly 42 with empty streams. This is an IR-contract check, not
  evidence of completed Rust aggregate lowering. The existing backend smoke
  check also passes.

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

## Required continuation

1. PA12: connect the owned-value proof to executable aggregate lowering; the
   expression/member selections, typed places and proof are implemented. Implement
   enum discriminant constant expressions. Lower
   aggregate values/calls with owned snapshots and explicit bounds checks,
   including zero-sized values. Add source fixtures, native comparisons and
   a deterministic layout inspection contract before activating the assignment.
2. PA13: use the same field/variant identities for binding modes, match CFGs,
   guards, or-patterns, slices and usefulness/exhaustiveness. Retain move and
   reference semantics for the subsequent checks; do not erase unproved borrows
   merely to reach LowIR.
3. PA14: extend the owned-value proof to PA13's pattern projections and binding
   modes, and retain maybe-initialized path facts separately from write history.
   Freeze the ownership view, publish assignment fixtures and their
   independently checked outcomes, and integrate the executable proof boundary.
4. PA15: discharge lifetime contracts through region parameters, elision,
   outlives constraints and caller/callee relationships, including nested
   reference returns. Full loan checking remains PA16's responsibility.

The proposed private Rust calling convention passes nonzero aggregate arguments
via pointers to evaluated snapshots and aggregate results via a hidden output
pointer. Zero-sized parameters/results require no physical payload but retain
their effects and initialization/move obligations. Function-pointer signatures
must reflect the same convention. This is a pending implementation choice, not
an assertion of rustc's unspecified native ABI or a backend change.

The lowering boundary must consume the owned-value proof before erasure to
`copyobj`; a physical memory copy alone is not permission to reuse a moved Rust
value. Keep ordinary indexing separate from PA13's pattern projections, which
will need constant-index/downcast path identities. Reference adjustments and
region obligations need their own checked facts before reference erasure. The
current scalar rejection gate must be replaced by these proofs, not removed to
make aggregate examples compile.
