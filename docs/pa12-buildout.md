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
These facts do not yet implement constructor expression evaluation.

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

## Evidence at this checkpoint

- All 1,547 PA1–PA11 fixture outputs are unchanged.
- Debug and release suites each pass 221 Rust tests. The declaration acceptance
  matrix checks the pinned rustc with default lints, including representation
  conflicts, duplicate fields, recursive storage and discriminant failures.
- Guaranteed layout vectors compare sizes, alignments and field/payload offsets
  independently with rustc. Deep nesting checks 20,000 layouts without recursive
  traversal, and enormous zero-sized arrays require constant layout work.
- Strict Clippy/rustfmt pass. The file/phase audit checks 147 Rust files with
  zero findings; two audit tests and 19 harness tests pass.
- The typed object-memory probe in `dev/tests/lowir_lowering.rs` writes
  `obj/pa12-work/memory.lowir`. The pinned bridge validates it and its executable
  returns exactly 42 with empty streams. This is an IR-contract check, not
  evidence of completed Rust aggregate lowering. The existing backend smoke
  check also passes.

Raw logs and frozen binaries are in `obj/pa12-work/`. Existing oracles remain
authoritative until a new assignment has declared outcomes, independent source
execution checks and reviewed generated text.

The frozen candidate SHA-256 is
`48ab2428f14eacdff8ff084cf98efd9ccb77f210e458e132b4542ab0278471c7`.
LowIR performance uses the published PA11 binary as baseline, CPU 10, frozen
inputs, A/A calibration and three ABBA blocks at each scale. All 54 observations
and complete-output hashes are retained in
`obj/perf/1791813976149095594/samples.json`. Baseline/candidate paired median
times for 4,000, 16,000 and 64,000 demanded procedures are 195.66/194.62,
773.98/759.34 and 3461.89/3442.62 ms. Peak RSS at the largest scale is
693,480/695,480 KiB. Outputs are identical; these observations show no measured
time regression and a small memory increase, not a claimed speedup.

## Required continuation

1. PA12: check aggregate literals, constructors, projections, indexing and
   assignments; retain selected fields/variants and once-evaluated indices in
   typed body places. Implement enum discriminant constant expressions. Lower
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
