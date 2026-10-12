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
scalar executable proof deliberately rejects aggregate values and projections
until the move/initialization proof and aggregate lowering are integrated.

## Evidence at this checkpoint

- All 1,547 PA1–PA11 fixture outputs are unchanged.
- Debug and release suites each pass 227 Rust tests. The declaration acceptance
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
- Strict Clippy/rustfmt pass. The file/phase audit checks 149 Rust files with
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

## Required continuation

1. PA12: integrate aggregate ownership checks and executable lowering; the
   expression/member selections and typed places are implemented. Implement
   enum discriminant constant expressions. Lower
   aggregate values/calls with owned snapshots and explicit bounds checks,
   including zero-sized values. Add source fixtures, native comparisons and
   a deterministic layout inspection contract before activating the assignment.
2. PA13: use the same field/variant identities for binding modes, match CFGs,
   guards, or-patterns, slices and usefulness/exhaustiveness. Retain move and
   reference semantics for the subsequent checks; do not erase unproved borrows
   merely to reach LowIR.
3. PA14: generalize the scalar initialization proof to move paths, structural
   Copy rules, partial moves, disjoint fields, reinitialization and CFG joins.
   Supply the ownership text view and independently checked rejection cases.
4. PA15: discharge lifetime contracts through region parameters, elision,
   outlives constraints and caller/callee relationships, including nested
   reference returns. Full loan checking remains PA16's responsibility.

The proposed private Rust calling convention passes nonzero aggregate arguments
via pointers to evaluated snapshots and aggregate results via a hidden output
pointer. Zero-sized parameters/results require no physical payload but retain
their effects and initialization/move obligations. Function-pointer signatures
must reflect the same convention. This is a pending implementation choice, not
an assertion of rustc's unspecified native ABI or a backend change.

The next ownership proof must distinguish moving a whole local from moving one
field: a partial move retains the parent's structure and permits reinitializing
that field, whereas assigning into a wholly moved/uninitialized aggregate has
different requirements. Use sparse, interned move paths with typed field keys,
and retain state for unmentioned fields when checking whole-value reads. Do not
expand large arrays into per-element records. Built-in Copy rules and explicit
nominal Copy selection must precede erasure to `copyobj`; a physical memory copy
alone is not permission to reuse a moved Rust value. Ordinary indexing cannot
move a non-Copy element out of an array; pattern projections are a separate
operation. The current scalar rejection gate must be replaced by that proof,
not removed to make aggregate examples compile.
