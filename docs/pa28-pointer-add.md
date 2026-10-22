# Canonical raw-pointer add

This development slice closes the unchanged
`pa28/tests/spec/byte-access/library-add-offset.rs` requirement. Whole byte-array
symbolic-fragment snapshots remain the next separate requirement; this slice
does not claim PA27 or PA28 completion.

`ConstPointerAdd` and `MutPointerAdd` append intrinsic indices 22 and 23. Each
has a distinct canonical impl owner and method DefId. The owner retains an
unsized-capable inherited `T`, while the unsafe const method has no own generic
parameters and requires `T: Sized` through ordinary selected-call obligations.
An unsized receiver therefore cannot fall back to a local trait method. Method
lookup, explicit qualification, aliases and reification use the typed registry;
there is no free `core::ptr::add` binding. An unrelated user method does not
publish either intrinsic's typed signature.

The CTFE helper validates the full selected callable identity and signature,
charges completed layout work, and checks count times size against target
`isize::MAX`. Address-only layout shares the existing readiness/cache traversal
but does not assert inhabitedness. Ordinary storage reads retain that assertion.
Zero effective displacement returns the exact incoming Constant, including its
metadata and write restriction, without traversing its allocation or contents.
Final escape validation still applies. Nonzero displacement uses the existing
checked live Byte projection and original allocation bounds, preserving write
restrictions. Arithmetic grants no read, initialization, alignment or reference
formation permission. Later accesses retain all those independent checks.

Runtime shims use the frozen LowIR multiply and byte-index vocabulary, with no
loads, stores or loops. Zero-size shims return their input directly. Selected
instances share the existing intrinsic shim cache. Generic and reified calls
use the ordinary ABI; const invocation through a function pointer remains
independently forbidden. No backend, public Value, Origin or Pointer schema
changes are introduced.

The pinned contract is Rust commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`,
[`library/core/src/ptr/docs/add.md`](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/ptr/docs/add.md),
lines 12–19, and the method-local Sized contracts in
[`const_ptr.rs`](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/ptr/const_ptr.rs)
and [`mut_ptr.rs`](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/ptr/mut_ptr.rs).
Nonzero byte displacement requires allocation provenance and an in-bounds
range; zero byte displacement does not demand allocation provenance. Numeric,
null and expired zero-offset CTFE acceptance controls have no native-validity
claim. No safety permission is inferred merely from rustc acceptance.

Independent outcomes were declared before candidate checks in
`obj/parallel/pa28/pointer-add/declared-first.json` and
`declared-followups.json`. The tracked companion is
`pa28/tests/pointer-add-outcomes.json`. Frozen sources include packed
address-only versus aligned/unaligned reads, non-Copy owned runtime values,
receiver/count evaluation once, one-past versus beyond-end addresses, unsigned
count overflow, shared-derived writes, unsafe permission, method turbofish,
unsized receivers, trait shadowing and explicit trait qualification. Literal
sources are loaded without rewriting their bytes. All use the explicit
independent `--cap-lints=allow` policy for the existing CHECK helper; original
rustc diagnostics remain in the independent receipts.

The original `trait-unsized-shadow` source was falsely accepted by baseline
073f160b because it selected a safe local trait method. It now selects the
unsafe inherent method and rejects. The additional `trait-unsized-unsafe`
control places that call in an unsafe block and independently rejects for the
method's Sized requirement. The original evidence is retained separately.

## Qualification

The final focused receipt is
`obj/parallel/pa28/pointer-add/closing-third/`: 43 pinned-Rust objects and 129
frontend views agree (28 accepts, 15 rejects), all 22 declared literal native
controls agree, and both private authority/shim invariants pass. The affected
67-case byte-access source group also passes. The first independent receipts
and frozen 073f160b baseline remain in `first/` and `followups/`. No rejected
numeric/null/expired runtime hypothesis was promoted to a native control.

`closing-second/` retains passing strict all-target/all-feature Clippy and the
1,693-file zero-finding audit. Its obsolete private selector matched zero tests;
the harness rejected that as incomplete. `closing-third/private.log` reruns the
correct `offsets::tests` selector and records both real tests. No production
change or repeated broad check was needed. Layout, upstream checks and suite
registration pass in the closing tail. Earlier retained failures cover test
wiring, function layout and fixture loading; they are not language outcomes.

All 20 source-work inputs independently qualify at widths 8/32/128, depths 0/8
and repeated calls 8/32/128, with a separate 64-unrelated-alias control. Repetition
increments are exactly 1,128 then 4,512 steps at depth zero and 1,704 then 6,816
at depth eight, independent of root width. The noise pair is exactly 2,568
steps/410 slots. Actual rows are in `closing-third/observations.json`. These
are bounded logical-work controls, not wall-time/RSS neutrality claims. Runtime
shims are checked for one block, no slots/spans, zero instructions for a ZST,
and just scale/index for positive strides 1/4/24.

The one active text pass checks all 8,360 PA1–24 cases against unchanged
manifests/inputs and the immutable published `0b995f43` archive. Every output
matches that published snapshot; the same 51 locally stale baseline sidecars
remain, with no unexpected differences. No reference is written. Receipts live
under `text-first/`, including the archive, manifest/input and output hashes.
Production fingerprint is
`9bbd17139b08fa3c172c61d35efcfd837e6c260bfffb17121f0111f345fb37ee`;
immutable candidate binary SHA-256 is
`409544aec57728d804540de31844da14514e5b263b6d6ab0d404bcc941859062`.
The backend remains the locked LowIR/lowir2native pair, hashes
`2acc7cefeecc3752c4a15da4ddb1e65c9cb2b88e0032a23d1eca7ff36a005882` and
`57f2c1156168eac7e653a4a193f27f124b635413e07de29940daf34a0b831fec`.

The next substantial missing original is
`pa27/tests/spec/byte-access/offset-word-roundtrip.rs`: a whole byte-array value
must carry symbolic fragments without forcing every byte to an integer.
Current word-only and raw-copy counterfactuals do not close it. Unit comparison
remains a separate PA26 dependency. Broader provenance, raw memory, numeric,
volatile and atomic families remain listed in `pa27-pa28-remaining.md`.
