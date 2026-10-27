# RustGM Programming Assignment 28: Intrinsics and atomics

Status: **planned**. This is a buildout brief, not a released grading contract.
Work prerequisites: [ordered dependency units](../docs/assignment-order.md). The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.

## Required implementation

Memory and numeric intrinsics, atomics and volatile operations.

The initial capability checkpoint uses the existing pinned LowIR vocabulary and
backend without extending either. The independent inputs in `tests/backend/`
qualify scalar volatile loads and stores for integer widths 8 through 128,
`f32`, `f64` and pointers. They also qualify `u32` atomic load, store, exchange,
compare-exchange success and failure (including the expected-value update), and
add-and-fetch. Run the ignored `intrinsic_backend` maintainer test with the
configured backend and GNU objdump to execute these controls and inspect the
atomic machine instructions. This establishes backend capabilities only.

The permanent [atomic backend matrix](tests/backend/atomics/README.md) extends
that evidence to 8/16/64-bit signed/unsigned storage with a U32 control, pointer
and valid boolean words, and thread/signal fences. It preserves separate
independent Rust outcomes and validator-only invalid-order probes. It supplies
no additional Rust frontend bindings and makes no concurrency-correctness claim.

The initial frontend checkpoint implements canonical `core::ptr` and `std::ptr`
`read_volatile` and `write_volatile` for primitive scalars and thin raw-pointer,
reference and function-pointer values. Imports and reified unsafe function
pointers retain their canonical callable identity. Reification shares a shim per
selected instance; unused direct reads retain their volatile access. Ordinary
loads and stores keep their existing LowIR spelling.

Typed intrinsic descriptors track unsafe permission independently of constant
operation eligibility. Volatile calls require the ordinary unsafe context and
are runtime-only, including skipped branches in ordinary and generic const
functions. Forming an intrinsic function-pointer constant is permitted; calling
that pointer during constant evaluation remains forbidden. As for raw reads and
writes, the source program must meet the selected type's validity and alignment
requirements; an unsafe block does not validate a pointer or its contents.

The ordinary memory slice adds canonical `core::ptr` / `std::ptr` `read` and
`write` with their real Sized-only signatures. Concrete non-Copy transfers now
use ordinary source move/drop rules and the existing owned ABI snapshots.
`write` consumes its value argument without dropping overwritten storage;
`read` returns an owned snapshot without marking the source local moved.
Unsafe caller obligations remain, including avoiding invalid duplicate ownership.
The earlier Copy-only capability restriction, query, cache and counters are
superseded; they are retained only in the historical checkpoint review.

Both operations require unsafe permission and permit const execution through
existing symbolic raw-access checks. Per-pointer authority, exact selected
storage, initialization, bounds, raw-target alignment and liveness remain checked
at every access. No successful read or final `*mut` type grants write authority.
Runtime object transfers use completed CopyObject spans and owned ABI snapshots,
preserving padding and source/destination overlap. Reified calls use the same
ordinary hidden-result and argument ABI. CTFE preserves typed values and scalar
union Image initialization masks without creating a reference.

Canonical `read_unaligned` and `write_unaligned` use the same Sized-only owned
transfer rules. Their alignment relaxation belongs to that one raw operation;
exact storage type, bounds, initialization, lifetime, write authority and static
evaluation context remain checked. Loading a pointer value unaligned does not
relax a later dereference. Reference formation and ordinary/volatile/atomic
operations retain their independent alignment rules. Runtime aggregate copies
use conservative alignment one over the existing ABI snapshots. The declared
32 source outcomes and ten native comparisons are recorded in
`tests/unaligned-memory-contract.json`; general reinterpretation and the retained
numeric/ZST pointer profiles remain separate boundaries.

The bounded counted-copy slice adds canonical unsafe const
`copy_nonoverlapping<T: Sized>(*const T, *mut T, usize) -> ()`, without a Copy
bound. Runtime loops use existing scalar transfers or completed CopyObject
spans; zero-size operations have no transfer loop. CTFE proves exact symbolic
scalar/object or contiguous-array spans once, snapshots the selected source
before stores and rejects physical overlap. Scalar union Images retain their
initialization masks. No overwritten value is dropped and no source move is
invented. Unsafe callers retain the ordinary ownership obligations.

Zero effective bytes use an explicit address-only path: alignment and retained
symbolic ancestry stay checked while contents, liveness and write permission
are not accessed. This does not grant permission for a later nonzero operation.
`tests/counted-memory-contract.json` distinguishes 45 declared object/three-view
outcomes and 11 native comparisons from the required remaining byte-transport,
numeric/null provenance and `invalid_null_arguments` source-lint contracts.
Arbitrary uninitialized/invalid byte copying, general offset/reinterpretation
and overlapping moves/fills remain open work. See
`docs/pa28-counted-memory.md` for qualification and retained failed hypotheses.

Typed initializer owners determine static-read eligibility. Const functions
inherit the caller's evaluation context; named, associated and inline constants
establish constant context, including a whole-initializer inline block. Nonzero
constant reads reject interior-mutable static roots even through ordinary fields
or inactive variants. Static initializer reads, zero-byte reads, local interior
values and independent reference validity retain separate controls.
`tests/owned-memory-contract.json` declares 52 object/three-view outcomes and
14 native comparisons. Three Rust-positive profile boundaries remain explicit:
two nominal mutable-reference lifetime cases and mutable-static initializer
reads. Numeric/expired zero-size pointer profiles and general reinterpretation
remain open. Original slice-index/.len consumer controls are retained pending
combined-main qualification. See `docs/pa28-owned-memory-review.md` and the
historical `docs/pa28-memory-review.md`; these checkpoints do not activate PA28.

Aggregate, zero-sized and wide-pointer volatile transfers remain explicitly
unsupported. Ordinary object copies or zero initialization cannot substitute for
volatile accesses or padding. The remaining atomic methods and intrinsic families are later slices. Backend
capability controls alone do not establish source-language support.

A separate constructor/storage slice introduces the exact canonical
`core::sync::atomic::AtomicU32` and `std::sync::atomic::AtomicU32` identity and
its const `new(u32)` factory. Its private storage has natural `u32` layout;
the type is neither Copy nor Clone. The factory remains an ordinary const
function for promotion eligibility. Shared mutation is a typed storage fact,
independent of destruction, auto traits, unsafe permission and const writes.
Promotion refines actual empty arrays and inactive variants, while constant
reference escape checks inspect the selected referent and its symbolic origin.
Each emitted allocation determines its own readonly placement, so an immutable
reference slot can point to writable atomic storage.

The draft `tests/atomic-storage-outcomes.json` records 27 independent source
outcomes and five native storage comparisons using the already qualified
volatile operations. PA25 closed inline execution now supports the unchanged
zero-array-repeat-return fixture; its former explicit boundary is superseded
by independent source and native qualification.
Initializer-local borrow controls consume PA25's actual body regions, while
escaping local references remain invalid. These storage comparisons establish no atomic-operation or concurrent-execution
claim.

The method slice introduces canonical `Ordering` with its non-exhaustive match
contract and distinct Rust discriminants. Safe runtime `AtomicU32::load` and
`store` use operation-specific LowIR ordering types; a bounded switch translates
Rust tags rather than forwarding them as backend order encodings. Invalid
variable, aliased, UFCS and reified-call orders abort at runtime. Direct method
syntax with a canonical variant obeys the scoped `invalid_atomic_ordering` lint,
including allow/warn/deny/forbid inheritance. Atomic operations remain runtime-only
through the existing whole-CFG constant eligibility gate. Packed receiver borrows
retain the ordinary alignment check; raw address formation is a separate rule.

The RMW slice adds canonical safe runtime `swap` and `fetch_add`, including
reified method pointers. Every Rust Ordering is legal for these operations.
A typed RMW ordering translates the same five Rust tags to backend codes.
`swap` returns the backend exchange's old value; `fetch_add` subtracts its
operand with wrapping u32 arithmetic from the captured add-and-fetch result.
It performs no extra memory read. Shared static storage, nested aggregates,
ignored results, variable/aliased orders and integer wraparound retain their
runtime effects. Constant calls, including retained skipped branches and
function-item aliases, remain forbidden independently of unsafe permission.
The subtraction slice adds canonical safe runtime `fetch_sub` for the same five
orderings. It passes a wrapping u32 negation to add-and-fetch, then adds the
original operand to the captured result to recover the old value. Both arithmetic
steps use u32 width, and no additional atomic-object read occurs. Independent
source, native and shim-growth controls are recorded in
`tests/atomic-subtract-outcomes.json`.
The CAS slice adds canonical safe runtime `compare_exchange` and
`compare_exchange_weak` returning the exact `Result<u32,u32>` identity. Typed
success/failure orders preserve all fifteen valid backend pairs: success accepts
all five orders, while failure accepts only Relaxed, Acquire and SeqCst. Direct
invalid failure variants obey the source-sensitive lint; dynamic, aliased, UFCS
and reified invalid orders abort at runtime. Constant eligibility remains
RuntimeOnly. The expected scratch slot supplies the old/observed Result payload
without an extra atomic-object read. Weak currently uses the strong operation,
which is permitted to avoid spurious failures. Ordinary variant layout and the
hidden-result ABI determine Result storage, without assumed tags or offsets.

The CAS source/native/scaling receipt also qualifies lazy preparation of fully
qualified AtomicU32/Ordering expression paths, explicit core aliases, no implicit
prelude and local shadowing. Existing canonical identity checks remain in force.
See `tests/atomic-cas-outcomes.json` for exact outcomes and phase distinctions.
The bitwise slice adds canonical safe runtime `fetch_and`, `fetch_or`,
`fetch_xor` and `fetch_nand` for U32. Each uses one initial Relaxed atomic load
and a compare-exchange retry loop with the requested success order and Relaxed
failure order. Failed attempts consume the backend-updated expected scratch;
success returns its captured old value. Source arguments are evaluated once,
and unchanged desired values still perform CAS. Nand complements at U32 width.
`tests/atomic-bitwise-outcomes.json` records source/native and typed loop
invariants; the native evidence is uncontended. No direct bitwise atomic opcode
or backend change is introduced.
The extrema slice adds canonical safe runtime `fetch_min` and `fetch_max` for
U32 and all five orderings. Explicit unsigned comparisons select desired bits
through a wrapping U32 mask; the same CAS retry scaffold serves bitwise updates.
Success returns the captured previous value, while failure reloads only the
backend-updated expected scratch. Unchanged desired bits still perform CAS.
Source operands are evaluated once, reified calls share canonical shims and
constant eligibility remains RuntimeOnly. The six ordinary source/native
comparisons are uncontended. Separate controlled LowIR tests inject one store
after the seed to force a failure; expected old/final values and stale-return
negative controls check retries on both sides of the high bit and at equality.
See `tests/atomic-extrema-contract.json` and `docs/pa28-atomic-extrema.md`.
Additional atomic widths and intrinsic families remain separate work.

The canonical `ptr::copy` slice adds overlap-safe runtime byte loops and
constant typed snapshots, retaining the real Sized-only unsafe const signature.
Both directions and identical pointers preserve source snapshots; overwritten
values are not dropped. `copy_nonoverlapping` still rejects constant overlap.
Runtime controls include invalid scalar bits transported without interpretation,
while arbitrary invalid/uninitialized-byte constant transport remains required
work. See `tests/overlapping-memory-contract.json` and
`docs/pa28-overlapping-memory.md` for the exact profile and qualification.

A scalar byte-backing slice now preserves invalid bool/char bits and uninitialized
bytes during constant copies without decoding them. Converted allocations retain
one mutable backing shared by ordinary field accesses, range aliases and raw
transfers. Selected typed reads still enforce initialization and scalar validity;
length metadata does not decode unrelated elements. Repeated fills encode once
and replicate bytes/mask with bounded work. The 22 source outcomes and 15 native
comparisons are declared in `tests/byte-transport-contract.json`; original
uninitialized-local hypotheses are retained as E0381 source rejections, without
native-positive claims. Pointer-bearing/enum backings, symbolic relocations,
fills and general reinterpretation remained follow-ups at that checkpoint. See
`docs/pa28-byte-transport.md` for the exact scope and superseded boundaries.

The symbolic-fragment slice supersedes the pointer-bearing scalar-only boundary
for its qualified thin/function/slice/vtable word families. Data, length and
vtable words retain separate identities; copies preserve initialization masks
and original write restrictions. Partial words may exist transiently but must
be complete before typed demand or immutable publication. Constant views and
native emission consume the same fragments, without an active union member or
invented numeric provenance. Enum/opaque/closure/string backing, general raw
slice and object-reference reconstruction, and PA29 image export remain required
work. See `docs/pa28-relocation-design.md` for exact supported profiles.

The current implementation and release gaps are reconciled in
`docs/pa27-pa28-remaining.md`; historical checkpoint reviews preserve their
original observations and explicitly superseded boundaries.

External-open coverage keeps an unknown case separate from real variant IDs and
runtime discriminants. Matching every current Ordering variant still needs a
wildcard, including nested and refutable patterns. The general scoped
`unreachable_patterns` policy consumes existing whole-arm usefulness facts;
partial unreachable alternatives within an otherwise useful arm remain outside
that retained fact contract. `tests/atomic-method-outcomes.json` records the
independent source and native method controls. No concurrent-execution suite or
additional atomic operation family is claimed by this slice.

The draft `tests/manifest.draft.json` declares 53 independently qualified source
outcomes, including nine native comparisons and five valid Rust inputs that
exercise the explicit unsupported boundary. The maintainer tests
`volatile_intrinsics` and `intrinsic_backend` retain their new outputs under
`obj/parallel/pa28/`. These checkpoints do not activate PA28 or provide generated
text references.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

## Tests to develop with the reference

additional atomic widths and methods; casts; u128; memory overlap.

Add spec fixtures, positive/negative controls and cross-feature regressions in
the earliest owning assignment. Use the course reference for text oracles and
pinned rustc for independent language checks. From PA11, execute emitted LowIR
with the configured CPPGM backend. Do not require new future-stage behavior
implicitly. An empty suite or a not-implemented result cannot pass.

## Completion and review

Publish the semantic and lowering boundary, tests/manifest.json and reference
sidecars together. Run all active earlier suites, lint, file/architecture audits,
and affected scaling measurements. Record independent checks before setting
status active in `assignments.json` and adding the stage to the root report.
Student export is a later operation: this repository owns the full solution.

The canonical `ptr::write_bytes` development slice retains its actual Sized-only
unsafe const signature. It fills checked symbolic spans with one byte/mask
update and one provenance interval edit; runtime uses the frozen U8-store loop.
Initialization, lifetime, bounds, alignment, per-pointer write authority and
static-context checks remain independent. Typed demand rejects invalid bool/char
bits and incomplete pointer words, while transport can preserve or repair them.
`tests/byte-fill-contract.json` declares the exact supported profile, numeric/MAX
library boundaries and pinned-CTFE alignment differences; no invalid-precondition
native result is claimed. See `docs/pa28-byte-fill.md`.

Canonical `*const T::add` and `*mut T::add` are unsafe const inherent methods
with inherited `T` and a method-local `T: Sized` requirement. Lookup selects the
inherent method before proving that requirement; it cannot silently select a
same-named trait method when `T` is unsized. Direct, qualified, generic and
reified runtime calls share the selected intrinsic identity. Reified calls
remain independently forbidden during constant evaluation.

Constant arithmetic checks the mathematical byte displacement and allocation
bounds for nonzero effective offsets. It preserves the incoming pointer's write
restriction and selects original backing coordinates without reading contents
or creating a reference. A zero effective offset, including zero-sized types,
returns the original value without an allocation/liveness demand. Final escape
and later memory accesses retain their own validity gates. Numeric/null and
expired zero-offset CTFE controls make no native-validity claim. Runtime shims
use the existing byte index operation and no loads/stores. See
`tests/pointer-add-outcomes.json` and `docs/pa28-pointer-add.md`; whole byte-array
fragment snapshots and the remaining memory/provenance families stay open.

The explicit integer-tag payload enum development slice reuses retained variant
field offsets and existing recursive byte storage. Typed reads validate only the
selected active payload; inactive bytes/padding retain independent initialization
facts. Canonical field pointers preserve their physical offsets across tag
changes. `tests/payload-enum-byte-outcomes.json` and
`docs/pa27-payload-enum-bytes.md` record the independently qualified profile.
Default-layout/niche enum and other required backing families remain open.

The C enum byte-backing slice admits repr(C) without a written integer tag by
using retained target ABI facts. Existing tag/payload validity, physical field
addresses and raw transport rules remain separate. The draft
`tests/c-enum-byte-outcomes.json` records the original default/tagless
Rust-positive gaps; the tagless followup below supersedes that one boundary.
See `docs/pa27-c-enum-bytes.md`.

The unique no-tag enum byte-backing slice reuses the exact retained sole variant
and physical fields as a product. It preserves ordinary field projections,
initialized-byte masks, symbolic fragments and reference validity; the original
tagless positive is now supported. Multi-variant default/niche layouts remain
required. See `tests/tagless-enum-byte-outcomes.json` and
`docs/pa27-tagless-enum-bytes.md` for the independently declared scope.

The ordinary direct-tag enum storage slice also applies to counted copies and
field-pointer fills through the existing byte/provenance engine. It preserves
selected payload validity and physical field addresses without changing the
retained ABI or granting new raw access permission. See
`tests/default-tag-enum-outcomes.json` and `docs/pa27-default-tag-enum-bytes.md`.
Broader niches and unsupported payload backing remain required work.

The retained closure capture slice also supports canonical memory transfers over
qualified capture children, preserving pointer restrictions and actual ownership.
Its Non-Copy source uses ManuallyDrop and observes one runtime destructor.
See `tests/closure-byte-outcomes.json` and `docs/pa27-closure-byte-storage.md`;
unsupported capture-child families remain required followups.
