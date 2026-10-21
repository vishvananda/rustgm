# RustGM Programming Assignment 28: Intrinsics and atomics

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA27. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

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
