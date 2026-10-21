# PA28 unaligned pointer transfers

The canonical core/std ptr registry adds `ReadUnaligned=17` and
`WriteUnaligned=18`, preserving indices 0–16. Both have the real Sized-only
unsafe Rust signatures and are const-allowed. They reuse the existing typed
Intrinsic payload, full selected Instance, ordinary generic signature factory,
owned arguments/results and reified shim cache. No Copy restriction, shared
value/provenance schema, backend vocabulary or CPPGM modification is introduced.

CTFE carries a private RawAlignment policy on the selected RawAccess. Ordinary
source raw dereferences and ptr read/write require alignment; only these two
canonical operations select the relaxed policy. Every access still checks exact
physical storage type, symbolic origin/liveness, leaf and anchor bounds,
initialization and scalar validity, per-pointer write authority and static
evaluation context. A later raw dereference creates a fresh aligned anchor.
Reference construction separately checks actual leaf alignment even if a private
raw access permits an unaligned transfer. Pointer values acquire no permission
or alignment promise from a successful read.

Runtime scalar transfers use the unchanged ordinary load/store vocabulary.
Aggregate transfers use existing non-overlapping CopyObject operations with
conservative alignment one. The Rust ABI still supplies the naturally aligned
owned argument snapshot or result destination; neither source operands nor
stored references are turned into references to packed fields. Writes consume
their value without dropping overwritten contents, and reads preserve the
source storage under the caller's unsafe ownership obligations.

The 32 source controls cover packed scalar, nested, aggregate, array, reference,
pointer and naturally aligned ZST fields; generic const calls, canonical aliases,
no-prelude resolution, shadowing and reification; and opposing unsafe, type/arity,
shared-write, expiry, initialization, union validity and ordinary alignment
failures. Packed-reference rejection remains deferred from type inspection to
executable checking. A dedicated pointer-valued read followed by ordinary raw
dereference rejects for its fresh alignment requirement. Existing exact-type
reinterpretation, static, nested-lifetime and numeric/ZST profiles remain open.

Ten independent rustc/CPPGM native comparisons check unaligned integer widths
through high-bit u128, floats, non-Copy padded objects, stored and wide references,
overwrite/destructor counts, explicit forgotten-source ownership, generic and
reified aggregate ABI, once-only operands, const results and self snapshots.
The original explicit-align ZST hypothesis failed Rust E0588; its bytes remain
as an independent rejection, alongside the separately qualified natural-alignment
empty-array ZST. The original ownership fixture used unresolved `core::mem::forget`.
That input/evidence is retained; the accepted control uses the already supported
canonical ManuallyDrop factory in a local Sized forget helper. This does not add
the library forget function.

At widths 8/32/128, repetitions 1/8/32 and unrelated aliases 0/64, each input has
exactly two demanded shims. Observed CTFE steps equal `4*width+54*repeats+46` and
retained logical slots equal `2*width+9*repeats+11`. Tests check separate linear
increments, repetition cost independent of width and exact invariance with
unrelated types. Ordinary packed loops at 8/32/128 iterations record
437/1493/5717 steps and 34 slots; unaligned pointer loops record
651/2331/9051 steps and 40 slots. These are logical execution counters, not whole
frontend timing, allocator measurements or RSS neutrality claims.

Six nonzero-selected private tests cover exact spans/ABI positions, no ZST
transfer, ordinary access flavor, physical reference alignment, initialization,+bounds under both raw policies and preserved write restrictions. The pointer
registry grows from four to six fixed entries; the earlier volatile registry
invariant now checks that expected growth independently of caller count.
Existing ordinary, owned and volatile source/native suites pass again.
Strict all-target/all-feature Clippy, syntax audit (1,492 files, zero findings),
layout and upstream verification pass. Every active PA1–23 text comparison passes
with zero differences. The final receipt records exact input/output/compiler
hashes. No references,
assignment status or public branch were changed.

All failed receipts remain under `obj/parallel/pa28/unaligned`: omitted native
check-helper setup, a missing system-Python audit dependency (the existing audit
venv succeeds), and two test-only Clippy issues. The first shim unit selector
matched zero tests and is superseded by a verified three-test module selector;
it is not counted as qualification. A later assertion uses iterator zips without
lint allowances. None of these setup failures is a language rejection.

Focused commands, with `CARGO_TARGET_DIR=obj/rust`, Cargo jobs two and test
threads two, must run through the shared storage/resource guard:

```sh
cargo test --locked --manifest-path dev/Cargo.toml --test unaligned_transfers -- --include-ignored --nocapture
cargo test --locked --manifest-path dev/Cargo.toml --lib lower::intrinsics::memory::tests -- --nocapture
cargo test --locked --manifest-path dev/Cargo.toml --lib raw_reads::reference_tests -- --nocapture
cargo test --locked --manifest-path dev/Cargo.toml --lib raw_reads::tests -- --nocapture
```

The remaining release contracts are tracked in `docs/pa27-pa28-remaining.md`.
Full release and immutable performance qualification remain integrator-owned.

Source fingerprint (sorted dev/**/*.rs path, NUL, bytes, NUL):
`0129d266b263b7ccd3ef04227307a7b1dd50d6cb9cf0974f5624da20b8708be8`.
Compiler SHA256:
`07d3cbd1f7e1615bf0dde01bf4cd1f114cce2d57ae38b2441c6f9530479fe227`.
Contract SHA256:
`68ae3c53ff3cbf5cb4ee79be852ce6a5386ffbff23b2a318cbc1a07542c8cf3d`.
Final strict/native/regression evidence is `obj/parallel/pa28/unaligned/fifth`;
active text and snapshot evidence is `obj/parallel/pa28/unaligned/close`.
`pa28/tests/unaligned-memory-outcomes.json` records all 32 object/three-view
outcomes (16 accepted, 16 rejected), ten asserted native comparisons and the
18-point logical width/repetition/unrelated-type matrix. Successful native ELF
files were pruned by the storage guard; source, LowIR, run diagnostics and
assertion receipts remain.
