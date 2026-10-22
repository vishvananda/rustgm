# Counted non-overlapping memory development checkpoint

This is the declared, in-development PA28 `copy_nonoverlapping` slice on the
accepted rest-view base `5bdcd0c6`. It does not activate the assignment or claim
completion of untyped memory transport. Qualification below distinguishes
completed independent evidence from checks awaiting resource admission.

The canonical intrinsic has the real unsafe const signature
`<T: Sized>(*const T, *mut T, usize) -> ()`, with stable vocabulary index 19.
There is no Copy requirement. Runtime scalar loads/stores and fixed-layout
CopyObject operations use the existing pinned backend. A counted shim evaluates
source operands once, tests an unsigned cursor against count before touching
external memory, and performs no external transfer for zero count. Zero-size
types have no transfer loop. These operations neither drop overwritten storage
nor mark source locals moved; the caller retains Rust's ownership obligations.

The first CTFE profile snapshots initialized typed values over exact symbolic
scalar/object storage or canonical contiguous array elements. A private
TransferSpan retains the original pointer capability, actual root, normalized
field path and optional array interval, selected type, byte offset and extent.
Paths, layout, extent and authority are checked once per span. The complete
selected source is frozen before any destination store. Contiguous array writes
reuse the existing range cell consumer and update original storage. Whole scalar
union Images preserve their byte initialization masks; scalar union-field source
byte transport remains explicit unsupported behavior.

Span headers and retained paths contribute to the existing machine slot count
throughout snapshot/store execution. Nested range-buffer reservations therefore
include the simultaneously retained span proofs. The prior slot count is
restored on success, error and initializer suspension; no proof is cached across
a resumed operation.

The crate-private projection-parent operation follows only a pointer's actual
retained ordinary projection. It preserves that pointer value's write
restriction, clears sized metadata and rejects terminal ranges or root locations.
It grants no arbitrary rerooting authority. The shared storage-read helper retains
the ordinary static-read eligibility and initializer readiness path. No global
Value, Origin, Pointer, Target or provider schema changes are introduced.

Zero effective bytes select AddressOnly traversal explicitly at the intrinsic
call. It checks retained symbolic ancestry, exact selected type, checked offsets
and target alignment without reading contents, demanding static initialization,
checking expired storage or granting write authority. Nonzero spans and ordinary
raw reads/writes retain Required traversal. Count times nonzero element size must
fit the allocation-size limit. Count times zero size does not overflow and does
not cause iteration. This is separate from forming a reference.

## Independent contract findings

The first 22 pinned Rust 1.98 object observations are retained under
`obj/parallel/pa28/counted-memory/first/`. Twenty-one matched their predeclared
expectations. The null zero-size case was initially predicted to reject but
accepted; its unchanged source and failed prediction remain in the contract.
That observation supplies no numeric/null native validity claim.

The pinned library contract at commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea` describes untyped copying with
initialization-state preservation. Zero effective bytes waive access validity
requirements while retaining alignment. Its precondition helper separately
handles the zero-effective-size case. The exact source links are recorded in
`pa28/tests/counted-memory-contract.json`.

Two null count-zero cases reject under Rust's default
`invalid_null_arguments` lint. This is a source-lint result, not CTFE evidence of
an actual memory access. Separate unchanged-call fixtures allowing that lint are
predeclared to isolate the distinction. Numeric/null provenance and that lint
remain required frontend work; an explicit numeric-pointer unsupported diagnostic
does not qualify either rule.

The existing generic ptr read/write aggregate loop independently built and ran
with Rust and the locked CPPGM backend for count zero and three. It preserved
source and suffix values and emitted CopyObject operations. This establishes the
existing backend loop capability only, using the immutable qualified `109be537`
compiler; it is not qualification of the new intrinsic implementation.

## Focused qualification

The companion contract declares 45 whole-source cases and 11 native controls.
The native set covers scalar/aggregate and non-Copy ownership, disjoint ranges,
generic/reified calls, one-time argument evaluation, reference elements,
rest-view destinations and no-access cases. Numeric/null sources are not run
natively. Source outcomes distinguish semantic negatives from accepted-Rust
profile boundaries, including partial union-field transport.

Private invariants check parent-projection permissions, zero-access versus
ordinary expired accesses, typed loop control flow and zero-size shim shape.
Work controls independently vary count, ancestry depth, element width and
unrelated type-store size. A separate compressed-source control copies one
element from arrays of length 256 and 2^40, comparing actual work/storage without
materializing the source. Whole-source controls vary repeated operations and
two different instantiated types alongside ordinary assignment controls. These
are logical work/storage checks, not wall-time or RSS neutrality claims.

The admitted preflight passed formatting, the all-target/all-feature compiler
check and six matched private invariants. All 45 current independent object
declarations agree with pinned Rust; the 22 unchanged first observations are
reused by exact source hash, with 23 new observations in the preflight receipt.
Both lint-allowed null count-zero cases accept, independently establishing the
default-lint distinction. The original failed ZST-null prediction stays retained.

The compressed-source pair both measured 17 steps and 12 logical slots. All 54
depth/width/count/noise combinations passed, with identical count-related work
and storage increments across ancestry depth and element width. Their evidence
is under `obj/parallel/pa28/counted-memory/preflight/`. These are direct checked
memory invariants, not independent whole-program runtime comparisons.

After the documented disk-admission refusals, the admitted strict pass found one
Clippy checked-conversion spelling, fixed without changing the bound. Its failed
log is retained under `second/`. The `third/` batch passed strict all-target and
all-feature Clippy, six private invariants, all 45 three-view source outcomes,
11 independent Rust/locked-CPPGM native comparisons, and 24 whole-source work
controls. Both concrete tuple types produce one shim each; unrelated type noise
does not change measured constant work/storage. The 4x operation-count steps stay
within the declared 5x ceiling for both counted transfers and ordinary controls.

The final loader uses ordinary fixture reads after the audit rejected eleven
unreviewed include macros; that failed audit is retained under `final/`. The
`final2/` pass qualified the unchanged eleven native inputs again and passed
strict Clippy, the 1529-file audit with zero findings, layout and upstream
verification, eight matched raw invariants and 27 test functions across seven
related memory source/native/work binaries.

PA1–19 and PA21–23 text compare exactly. PA20 passes 1379/1381: its existing
`constants/pattern/{lowir,semantics}/generic-default-dependency.rs` cases reject
with `ERROR: file 0: 83: shape parameter is outside its environment`. The
integrator confirmed this is the earlier pattern-scope regression in this
checkpoint's `5bdcd0c6` base; its separate repair `1a2536cb`, integrated as
`c47a8e88`, already passes all 7788 PA1–23 cases. This slice does not change
pattern scope or regenerate its references. The immutable qualified `109be537`
compiler still accepts the unchanged source; receipts remain under `tail/`.
The integrator owns combined repaired PA1–24 qualification. Subsequent worker
checkpoints include PA24's newly published 467 text cases.

No references are generated. The source negatives that currently reach a
numeric-pointer unsupported diagnostic do not qualify null/alignment rules;
only the independently opposed symbolic packed cases qualify this slice's
zero-access alignment check. Resource refusals, the initial prediction mismatch,
the lint correction and the failed test-loader audit are all retained.

The focused commands are `cargo test --lib counted_`,
`cargo test --test counted_transfers -- --include-ignored` and the related
`memory_transfers`, `unaligned_transfers`, `rest_views`, `sequence_constants`,
`union_references`, `union_addresses`, `direct_union_borrows` binaries. Every
command uses the shared resource/storage wrappers, Cargo/test threads two and
scoped `obj/parallel` cleanup. Native tests remain ignored in ordinary CI.

The final receipt is `obj/parallel/pa28/counted-memory/qualification.json`.
Its compiler-source fingerprint is
`de5ff20be020f27ec17a0ed891a77a8f25b1f476f4e75b4f26b1c704b64fadea`;
the immutable candidate SHA-256 is
`c990ea78ad41da8b010936305932b30f56c110250ac131a400e51dda8d5bd698`.
The current candidate and baseline pins live under the same evidence root.
No full Cargo suite, broad native matrix or exclusive measurement was run for
this worker checkpoint; those combined release checks remain integrator-owned.

Arbitrary invalid/uninitialized byte transport, overlapping copies, fills,
general offset/reinterpretation provenance and numeric/null routes remain open
required PA27/PA28 work. The complete inventory is maintained in
`docs/pa27-pa28-remaining.md`.
