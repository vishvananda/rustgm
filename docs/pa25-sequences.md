# PA25 constant sequence execution

This is an implementation checkpoint, not an activated assignment contract.
The independently declared inputs are in
[`sequence-outcomes.json`](../pa25/tests/sequence-outcomes.json).

Checked array-to-slice conversions retain length metadata on the pointer while
preserving the original allocation and its sized type. Both the typed CFG's
direct reference adjustment and explicit unsizing use the shared constant
representation. Mutable reborrows preserve write permission; shared references
restrict it. Promotion uses the same adjustment and backing type.

Array-backed slice indexing and suffix selection execute with bounds checks.
Reads walk existing mutable cells only as far as necessary, then select fields
directly from an immutable constant. They do not expand an entire repeated array
to read one element. A write to another field of the containing tuple does not
force that immutable array to expand either. Writes continue to use the ordinary
mutable cell path and retain its resource limits.

## Qualification

The six accepted programs cover indexed reads, static backing, mutable writes,
suffix patterns, empty slices and repeated arrays. Their original source bytes
pass pinned Rust 1.98.0 object compilation, all three candidate inspection/
lowering views, and native execution with both rustc and the CPPGM backend.
Out-of-bounds reads (including empty slices) and escaping local references are
independently rejected. Inputs and declared hashes remain unchanged from the
initial probe.

The compressed-read invariant varies array size through 8, 1,024 and 1,000,000
elements under a 2,048-step/128-slot interpreter budget. Within each family,
steps and peak slots are identical: direct reads use 27/7, slice reads 56/16,
and reads after changing an unrelated tuple field 66/16. These are interpreter
work/storage counters, not whole-compiler timing or allocator measurements.

The combined gate at `obj/parallel/sequence-integration-v3/` passed 91 affected
test functions, including native groups, strict all-target/all-feature release
Clippy, file/architecture audit (1,494 files, zero findings), layout validation
and all PA1–PA23 text suites. Its source fingerprint and binary SHA-256 are
retained with the receipts. Full release and paired performance qualification
remain integration requirements before publication.

## Remaining coverage

Subslice/subarray rest views remain implementation gaps for Rust-accepted inputs.
They require preserving backing identity and
aliasing, rather than allocating unrelated copies for views. Initial positive
hypotheses for constant `str` matching were rejected by pinned rustc with E0658;
the unchanged sources and failed hypotheses remain evidence. The current
interpreter's rejection of those operations does not yet establish whole-body
constant eligibility, including unreachable operations. None of these boundaries
is a passing PA25 grading oracle.

## Literal backing checkpoint

Typed dereferences now materialize shared immutable backing for canonical byte
literal identities. Repeated dereferences reuse one backing record; its sized
array type and contents remain separate from each pointer's selected metadata.
Thin pointers do not recover discarded length, and shared backing cannot gain
write permission from a raw mutability cast. Literal bytes are not copied into
the backing table. The lazy table has a logical 64 MiB limit charged at 128 bytes
per distinct backing record; this is not an allocator/RSS measurement.

Reading an entire byte array produces a by-value constant. Mutating a local copy
uses the ordinary owned-cell path and leaves the original literal unchanged.
Length inspection preserves explicitly selected pointer metadata. Empty literal
backing remains addressable with zero logical length.

`literal-backing-outcomes.json` declares eight additional accepted programs and
two bounds rejections. The raw literal-write rejection belongs to PA27 and is
declared in `pa27/tests/constant-literal-outcomes.json`. The original
`slice-bytes` boundary now passes without changing its source. All fifteen
accepted sequence/literal programs execute unchanged with both rustc and CPPGM;
all new bounds and write rejections agree with pinned rustc. Private controls
verify backing reuse at 8/32/128 distinct literals with 128 requests each,
metadata separation and shared-derived permission. Pointer, constant value and
reference-place sizes remain 24, 32 and 64 bytes on this target.

The gate in `obj/parallel/pa25/literal-backing-check-v2/` passes three reference
invariants, three CFG execution test functions, five sequence test functions
(including the fifteen native programs), strict Clippy, the 1,494-file audit
with zero findings, layout validation and all PA1–PA23 text suites. The earlier
gate stopped at a `manual_let_else` lint, which was corrected without changing
the declarations or fixtures. Full publication qualification and paired timing
remain pending. The initial independent hypotheses and old compiler failures
are retained in `obj/parallel/pa25/literal-backing-v1/`.
