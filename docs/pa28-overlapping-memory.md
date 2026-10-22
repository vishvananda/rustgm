# Overlapping memory development checkpoint

This PA28 checkpoint adds canonical `core::ptr::copy` and its `std` re-export,
with stable intrinsic index 20 and the real unsafe const signature
`<T: Sized>(*const T, *mut T, usize) -> ()`. It has no Copy bound. The parent
`129530ba` contains the separately qualified counted non-overlapping checkpoint,
the PA20 pattern repair and released PA24. This checkpoint does not activate PA28.

## Transfer semantics

Runtime lowering uses a byte loop because the frozen LowIR CopyObject contract
forbids overlap. A zero-byte branch precedes pointer comparison and external
access. An unsigned pointer comparison selects forward or backward traversal;
the backward loop decrements before indexing. Neither direction computes a
pointer end. Each iteration loads one U8 before storing it. Zero-size types
return without arithmetic, loops or external accesses. The caller retains the
unsafe allocation, count, alignment and ownership obligations. The shim does not
drop overwritten values, invalidate source storage or reevaluate source operands.

Constant execution reuses the checked TransferSpan and complete source snapshot
from `copy_nonoverlapping`, permitting overlap only for the exact Copy identity.
Non-overlapping copies retain their overlap rejection. Span normalization,
layout, bounds, initialization, liveness, per-pointer write restriction and
static-evaluation context are unchanged. Every selected typed source value is
snapshotted before destination writes, including overlapping array/rest ranges.
Zero effective bytes retain explicit AddressOnly traversal and alignment checks;
nonzero accesses retain Required traversal. Count times nonzero stride must fit
the constant allocation-size bound. No Value, Origin, Pointer, Target or provider
schema changes are introduced.

The runtime byte loop transports invalid scalar bits and initialization state
without typed interpretation. The current constant profile still requires
initialized typed snapshots. Union-field byte transport and intra-object
reinterpretation remain explicit unsupported profiles. Their rejection does not
prove scalar validity; the demanded invalid-bool fixture remains distinguished
from an accepted transport that never interprets the bits. Whole scalar union
Images retain existing byte masks without introducing an active member.

The pinned Rust source at commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`,
`library/core/src/ptr/mod.rs`, specifies overlap-safe untyped copying through a
temporary snapshot and preserves initialization state. Its alignment requirement
still applies when effective bytes are zero. The frozen `shared/lowir.md`
CopyObject contract independently prohibits overlap. Numeric/null count-zero
source-lint and CTFE findings are retained from the counted contract; no such
input is executed natively.

## Qualification

`pa28/tests/overlapping-memory-contract.json` declares 55 independent source
outcomes, 23 native inputs and six backend controls. The first batch qualified
53 source inputs and 18 Rust executions before implementation qualification.
Two shadowing source controls and five raw-bit native controls were subsequently
added with declared expectations before their first execution. All 55 source
outcomes match pinned Rust and all three frontend views. The 23 Rust/locked-CPPGM
native comparisons pass, including left/right and identical copies, intra-object
byte shifts, generic/reified calls, one-time argument evaluation, reference
fields and owned destructor counts. The ownership controls replace or forget
duplicated values before drop; they make no invalid double-ownership claim.

The raw-bit controls cover invalid bool/char bits transported without typed
reads, and existing non-overlap copies of invalid bool bits, signaling-NaN bits
and raw invalid reference bits. They pass without constructing or dereferencing
an invalid Rust reference. The six direct backend controls separately verify
both overlap directions, identical/disjoint pointers, zero count and a zero-size
maximum count; these establish the pinned backend vocabulary rather than the
frontend implementation.

Five matched `overlapping_` private tests and six matched `counted_` private tests
pass. New invariants check complete snapshot results in both directions, old
non-overlap rejection, monotone per-pointer authority and explicit intrinsic
identity. Shim invariants check nine blocks, one cursor slot, guarded U8 accesses,
unsigned direction choice, loop backedges and an empty zero-size body. The private
54-point overlap grid varies count, ancestry depth, element width and unrelated
store size; ordinary counted controls remain intact. Twenty-four independently
Rust-qualified whole-source work controls vary repetition 8/32/128, tuple width,
unrelated declarations and two concrete shim types. Unrelated noise leaves
constant work/storage unchanged; 4x repetition stays below the declared 5x
logical-work ceiling. These are work/storage invariants, not timing neutrality.

The first strict preflight failed only the doc-markdown spelling of CopyObject;
its log remains under `preflight/`. After that correction, `preflight2/` passed
strict all-target/all-feature Clippy and the private invariants. `functional/`
passed strict Clippy, all new source/native/backend/work tests and the existing
counted source/native/work regression binary. The first final audit found a duplicated declared-case table with the old
counted tests. The common storage/no-access matrix is now shared, with explicit
overlap-specific overrides; both loaders are requalified. The closing batch passed strict Clippy, both source loaders, the 1544-file
audit with zero findings, layout/upstream verification and all 8255 PA1–24 text
comparisons, including the released 467 PA24 cases. Its evidence is under
`final2/`; the final receipt is `qualification.json`.
No reference generation, backend change, full release matrix or exclusive
measurement is performed by this checkpoint.

Evidence lives under `obj/parallel/pa28/overlapping-memory/`. Reproduction uses
`cargo test --lib overlapping_`, `cargo test --lib counted_`,
`cargo test --test overlapping_transfers -- --include-ignored --nocapture` and
`cargo test --test counted_transfers -- --include-ignored`, through the shared
resource/storage wrappers with two Cargo/test threads. Native tests remain
ignored in ordinary CI. The complete remaining contract list is maintained in
`docs/pa27-pa28-remaining.md`; fills and arbitrary invalid/uninitialized-byte
constant transport are required follow-ups.

The compiler-source fingerprint is `2aab17635abc171033a13795d2f1c64c4fa5fb1a7550ffdaaab9180a5250e2c1`;
the immutable candidate SHA-256 is
`a4b92ff263569730a80aece128c16ecb1e4623b3b3d4dd08ceed1d55f979a5d7`.
