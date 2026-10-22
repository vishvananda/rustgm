# PA25 constant rest storage

This development checkpoint implements checked `Subslice` and `Subarray`
projections. It does not activate PA25 or change a generated reference.
The independently declared inputs, original hashes and route distinctions are
in `pa25/tests/rest-view-contract.json`.

A rest reference selects its original allocation and an absolute element
range. Nested ranges normalize once; an element projection normalizes to the
original base and absolute index. Pointer write permission follows the incoming
pointer value, so normalization cannot turn a shared-derived pointer writable.
Expiry follows every original storage ancestor. `StoragePath` carries a typed
optional terminal range; consumers must handle it or explicitly reject it.
There is no implicit conversion of a range path to an ordinary field path.

A by-value array rest is an immutable snapshot, independently owned when a local
later mutates it. Product snapshots share an arena field interval; repeated
arrays keep one fill value and a selected length. Byte snapshots retain an
explicit `ByteRange` recipe into canonical literal storage. Nested byte ranges
adjust offsets without copying bytes or creating a Constant per byte.
Mutable rest references select the original cells, including whole-array stores.
Mutable dense snapshots/writes keep the interpreter's existing storage limits.

Address formation does not demand contents. Completing an immutable range
reference uses the existing contents dependency scheduler and a narrow range
completion consumer. Actual final-reference inspection visits the selected
storage, including zero-length and compressed repeat handling. Canonical static
backing remains distinct from a fresh interior-mutable allocation. Raw whole-range
reads/reconstruction remain an explicit unsupported profile; normalized element
pointers retain ordinary exact type, bounds, alignment, initialization, lifetime,
write-authority and static-context rules.

`Rvalue::Length` uses the selected array type or retained slice metadata, checks
storage liveness and the initialized local container, and does not read elements
or demand a static initializer. Pinned Rust independently accepts metadata-only
pattern controls for ordinary, interior, mutable, external and self-referential
statics; source unsafe and mutable-static reference lints remain independent.
Ordinary array/slice `.len()` method preparation is a separately tracked PA26
gap. Exact accepted method-route inputs and their earlier unsupported diagnostics
are retained; pattern controls do not claim that the method route is implemented.

On the qualified x86-64 target, measured production layouts keep Pointer 24,
Constant 32, reference Place 64, public Location 64, ordinary projection 16,
Products Contents 48 and public View 48 bytes. Public Origin grows 16→24 bytes.
A tagged optional projection identity stays 8 bytes; cold range records are 24
bytes in a lazy arena, with no range-arena allocation for ordinary programs.
Range records charge a conservative 256 bytes each against 64 MiB, including
capacity slack and their Place, plus one fixed header allowance. Snapshot recipe headers share payload and use
a separate conservative 128-byte header allowance against 64 MiB, also retaining one fixed allowance. Mutable range
operations charge selected elements and temporary retention before cloning;
read-only structural inspection retains its existing bounded work walk.

The new constant-view vocabulary is `byte-range length=N data=HEX` for an owned
byte snapshot, and `origin=subsequence:aBASE:FROM:LENGTH contents=alias` for a
range storage view. Runtime pointer serialization remains a relocation to the
original allocation plus the checked element-stride offset. Zero filling native
storage does not change constant-time initialization or pointer permission.

Initial focused evidence is under `obj/parallel/pa25/rest-views/`. Work controls
vary backing lengths 8/1024/1000000 and repetitions 8/32/128, paired with ordinary
indexing. Work/storage is identical across backing widths within each family:
ordinary 387/1299/4947 steps with 34 peak slots, rest 1029/3837/15069 steps with
76/172/556 peak slots. These are logical interpreter counters, not allocator,
RSS, wall-time or whole-compiler performance claims. Original failed hypotheses,
including an initial parent-derived raw alias source that is not native validity
evidence, remain retained separately. Full integration/release qualification
and performance remain the integrator's responsibility.

Closing qualification uses compiler source fingerprint
`99da51530380cb87e60f2fecf6ffc0934d84e0efcc4fa927be5ef9bf575e901f`
and development binary
`bc1fd81c36ff254b30fb8962e8f07fcffe6c1051dac4180ececdbf8e6f9b5787`.
All 46 pinned-Rust object declarations match. All 138 candidate view outcomes
are recorded, with seven method-route gaps explicitly separated from the
implemented profile. In particular, rejection before `.len()` lookup does
not qualify a mutable-static lint. The 12 native comparisons pass, including
owned destructor counts, mutable aliases, independent snapshots and reference
arrays. Temporary reference-array backing originally predicted to escape
instead rejects in Rust with E0716; its exact input is retained separately,
and the compiler rejects it for the source lifetime rule. Actual static array
backing and local arrays with valid lifetimes succeed.

Reference-element storage uses the preexisting physical compatibility rule:
identical outer reference mutability and identical referent TypeId may differ
only in the outer region. The shared Reader predicate also serves the existing
raw checks. Nested reference structure, mutability, referent mismatch, bounds,
source conversion and lifetime proofs are not relaxed. Reachable-value inspection
uses the actual stored element type, and relocation uses the backing array's
original layout and stride.

The closing batch passes strict all-target/all-feature Clippy, the 1500-file
architecture audit with zero findings, four rest layout/compression/compatibility
invariants and the existing physical-reference invariant. Related sequence and
CFG regressions pass; the compatibility tail also passes ordinary pointer and
unaligned transfer source/native/work groups, canonical spelling preparation
and ordinary static-location tests. Every active PA1–PA23 text comparison,
assignment layout and pinned upstream verification passes. Receipts are in
`obj/parallel/pa25/rest-views/{close-v2,close-v4,compatibility}/`; earlier failed
runs remain retained and are not claimed as passes. No compiler source changed
throughout the successful closing and compatibility batches.
