# PA28 ordinary pointer transfers

This isolated development checkpoint adds exact canonical `core::ptr` / `std::ptr`
`read` and `write`, appended at Intrinsic indices 13 and 14. Both are unsafe and
const eligible; source unsafe permission remains independent of memory validity.
The actual signature is `T: Sized`, with no invented Copy requirement. The initial
implementation capability accepts concrete Copy types and reports an explicit
boundary for non-Copy transfers. Generic defining bodies retain their real
contract, and selected operations check the concrete type.

Runtime scalar transfers use ordinary loads/stores. Object transfers use completed
layout size/alignment and existing CopyObject. Reads target fresh hidden-result
storage; writes consume an owned by-value snapshot, including when the original
source expression aliases the destination. Padding travels in the object span.
No field-by-field reconstruction, zero-fill, backend extension or volatile
substitution is involved. Ordinary Rust parameter/result classification supplies
reified shims, cached by full canonical callable instance.

CTFE forms an existing Resolved raw access from the full symbolic pointer value.
It reuses exact storage-type compatibility, bounds, raw-target alignment,
initialization, static readiness, lifetime and per-pointer write checks. It creates
no reference and never derives authority from a final `*mut` spelling. Every access
checks permission/validity; only immutable successful Copy capability is cached.
Scalar union Images retain byte initialization masks and overlapping field aliases.
A write can repair invalid bool bytes or initialize a wider field without an eager
reference read; copying an Image neither validates alternative fields nor changes
its initialization mask. Pointer-bearing and general reinterpretation profiles
remain outside this representation.

The additive `Copies::bounded` uses sparse state, no assumptions and
`declaration=false`, reusing the existing iterative traversal. Pending edges are
charged before scheduling. Failed work clears active/pending entries, while
unrelated queries remain usable. The ordinary query keeps its existing behavior,
with an added optional budget branch; this is an explicit later measurement
surface, not a neutrality claim. The interpreter's success-only full Instance set
survives suspension and charges one logical retained slot per entry, consistent
with its existing slot budget (not a complete heap-byte measurement).

## Qualification

`pa28/tests/pointer-memory-contract.json` declares 42 independently object-qualified
inputs: 28 accept in pinned Rust 1.98.0 and 14 reject. Twenty-three accept through
all three RustGM views; five valid inputs retain explicit profile boundaries.
The rejected numeric unaligned ZST input reaches an earlier unsupported numeric
provenance gate in RustGM, so it does not establish alignment diagnostic priority.
The other controls distinguish unsafe calls, type/arity, shared-derived writes,
stale storage, packed field pointers, invalid bool, static writes, reified const
calls and uninitialized Image bytes. Whole packed aggregates and live symbolic
aligned ZST transfers accept independently of separately formed field pointers.

The ignored native test executes eleven rustc/CPPGM comparisons: ten succeed and
one independently expected slice-bounds panic aborts. Controls cover operand
evaluation once, u128/f64, padded snapshots and overlap, ordinary/reified/generic
ABI paths, arrays, wide reference values, aligned ZSTs, scalar Image repair/masks,
packed aggregates and const generic transfers. The bounds comparison records rustc
SIGABRT versus backend status 134. Six private invariants cover typed transfer
spans/ABI/flavor and sparse/deep/budgeted Copy queries. Eleven related source and
scaling groups pass. Strict all-target/all-feature Clippy, syntax audit (1,377
files, zero findings), build and 24+126 upstream provenance checks pass.

The N=8/32/128 loop control records ordinary steps 343/1159/4423 and 30 slots;
ordinary pointer transfers use 509/1781/6869 steps and 38 slots. The separate
18-point graph matrix varies width 2/16/64, repeated calls 1/8/32, and unrelated
aliases 0/64. Exact Copy visits remain 36/148/532 at each width, independently of
repetition/unrelated types; four distinct typed instances emit four shims. Observed
logical slots are `4*width + 18*repeats + 23`, bounded explicitly by the test.
These are actual traversal/storage counters, not compile-time or RSS benchmarks.
The earlier unrolled ordinary control itself used 133 slots at N=8 and falsified
the reused `N+96` hypothesis; its source and failed evidence remain retained.

The ordinary scalar control is byte-identical to baseline 62542d22 in types,
LowIR and constant views. Volatile preparation now registers four pointer
functions rather than two; its bounded registry invariant is updated explicitly,
while atomic preparation selects only the explicit atomic family.

Source fingerprint:
`9083fafbdaa6ad164c001d7ebdd213af281ebee424ca9f1ccd02698d5a1e8338`.
Compiler hash:
`4ba7a063d7e214e19c1bfb442829e81ac803393f23095f9b4b183c69ee09f6a2`.
Raw final evidence is `obj/parallel/pa28/pointer-memory-close`; private invariants
are retained under `pointer-memory-seventh`. The tracked outcomes bind each input,
contract and observed output to these snapshots.

## Retained disagreements and boundaries

The original null-unit read hypothesis predicted rejection, but pinned rustc
accepts that read, a null-unit write and an expired ZST read. The initial stricter
symbolic provenance/liveness profile rejects these; they remain Rust-positive
boundaries, not language-invalid cases. Live aligned symbolic ZSTs are supported.
The original declarations and object evidence are preserved unchanged.

The unchanged `pa28/tests/spec/pointer-memory/wide-value.rs` reaches
`ERROR: file 0: 37: constant slice projection is not implemented` on this worker
cut. The retained original native source
`obj/parallel/pa28/pointer-memory-native-2914072/arrays-and-wide-values.rs`
reaches `ERROR: file 0: 321: receiver has no inherent nominal method` for `.len()`;
its original log is `pointer-memory-sixth/source-native.log`. The combined
integrator confirms both diagnostics in all three views under
`obj/parallel/pa24/captured-objects/checks-sixth/`. Separately qualified whole-wide transfers
and runtime index/bounds controls do not implement either missing slice consumer.

Non-Copy transfers, counted copies/overlap APIs/byte fills, unaligned operations,
numeric families and the existing general pointer/union representation limits
remain required follow-up work. No planned assignment is activated and no oracle
is generated. The inherited PA16 receipt/layout discrepancy is integrator-owned
and already corrected in its combined checkout; this worker does not regenerate
or repeat that known stale check.

The later Sized non-Copy checkpoint supersedes this review's concrete Copy-only
profile, capability cache, bounded Copy API and copy_steps counter. Their original
measurements and hypotheses above remain historical evidence. Current transfer
and static-context contracts are tracked in `pa28-owned-memory-review.md`.
