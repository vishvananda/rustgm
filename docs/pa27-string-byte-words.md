# String storage and word transport

This checkpoint consumes the retained `str` identity as a wide data pointer plus
independent byte length. Symbolic data provenance and per-pointer write authority
survive union/byte transport. String literals acquire their existing shared
immutable `[u8; N]` backing only when a physical word/address is demanded. Ordinary
literal representation and ordinary aggregate layouts remain unchanged.

The physical sequence element is exactly `u8`. Reference formation validates the
selected range against the original physical array, current initialized-byte and
provenance facts, actual allocation liveness/alignment, static context and incoming
write restriction. Raw pointer metadata is retained without forming a reference;
an oversized raw length cannot enlarge the allocation or authorize a larger read.
Ranges reuse canonical original-base offsets and immutable range recipes. Existing
pending-read duties handle static readiness without a second cursor or cache.

UTF-8 is an ordinary library invariant, separate from raw storage validity. The
[pinned primitive documentation](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/primitive_docs.rs#L1010-L1017)
explicitly distinguishes construction from UTF-8-assuming library operations.
The vendored Reference's `undefined.validity.str` requires initialized `[u8]`
bytes. The actual unsafe conversion contracts and bodies remain in
[core/str/converts.rs](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/str/converts.rs#L159-L213).
The checked UTF-8 algorithm remains ordinary Rust in
[core/str/validations.rs](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/str/validations.rs#L126-L251),
including its still-required source `const_eval_select` dependency. This work does
not add string API factories, an eager Unicode scan or a Unicode-validity cache.

The original three eager-UTF8-rejection hypotheses were falsified by pinned Rust
and the documented language contract. Their exact bytes, first expectations and
object receipts remain under `obj/parallel/pa27/string-byte-words/independent-v1`.
Their later raw-storage acceptance controls make no native-safety claim. The
original empty-string hypothesis instead rejected for E0745 (a raw address of a
const temporary); a distinct corrected local-reference-cell control independently
passes Rust/native. Neither discrepancy is relabeled as a string validity error.
The four original string-union/raw-word sources, including their panic-macro forms,
remain unchanged alongside their separately qualified division-helper controls.

Sources derived from the pinned conversion body retain the exact reduced mutable
cast/reborrow structure. They are reductions, not claims that the complete source
library or its transmute binding is implemented. Full upstream string routines,
other unsized tails, numeric/null provenance and the remaining PA27/28 contracts
stay required. Tuple-word construction controls are qualified for the explicit
Linux x86-64 profile, without promising universal Rust tuple layout.

Worker qualification is focused: declared source/checked-view outcomes, valid native
controls, malformed sequence/initialization/write/liveness invariants, ordinary
slice/literal controls and width/repetition work. The integrator's subsequent
combined release and performance results are recorded below. No text references
are edited.

The original panic-macro forms remain required successful inputs in the draft
manifest. The focused supported-storage test excludes those two sources while
their ordinary source-library `panic!` dependency is unresolved. Complete diagnostic
replay retains their current failure explicitly; it is not an expected rejection
contract or a claim of original-source acceptance.

The additional frozen `published-string-runtime.rs` input passes nonempty and
empty published constant string references through an ordinary runtime function.
Its scalar observations depend on both emitted byte storage and wide length
metadata; pinned Rust and CPPGM both execute successfully. This is actual data
emission/argument coverage, separate from const-computed scalar checks.

At widths 8/32/128, warm selected-byte reads cost exactly 13 logical steps each:
104/416/1,664 for 8/32/128 repetitions, with no retained growth during repetition.
Root backing retention is 18/19/22 existing value slots, respectively. Constant
and Location sizes remain 32 and 64 bytes. No representation record or Frame field
changes. Nine frozen raw-Str source controls vary width and reuse independently;
all Rust object/three-view checks pass, with the three diagonal native controls
also passing. These are logical-work/storage and functional results, not wall-time
or RSS measurements. Inputs/receipts remain under the string study's
`scaling-inputs`, `scaling-v1` and `closing-v1/logical-work.log` paths.

Final focused qualification passed 18 supported source inputs across all three
checked views and 11 literal native comparisons, including runtime publication.
The complete inventory has 20 inputs: the two unchanged panic-macro forms retain
their required-zero outcomes and their observed unresolved-macro failures. Seven
private memory invariants pass. Affected byte-access, fragment-range and relocation
source groups pass; the raw-slice group passes after preserving its existing
`constant range is out of bounds` diagnostic. The earlier diagnostic-only failure
is retained. Final all-target/all-feature strict Clippy, the 1,927-file audit with
zero findings, 117 harness tests, 358 registered roots in 13 binaries, assignment
layout, 24 upstream/126 Reference pins and all 8,362 active text comparisons pass.
No generated reference changed. The final compiler pin is recorded in
`obj/parallel/pa27/string-byte-words/closing-v2/binary.json`; the fixture-only
runtime registration tail is `registration-v1/checks.json`.

## Combined publication qualification

The integrated compiler snapshot is `aa6a854b378953c760f5b6e72416151bfc536297`,
with immutable release binary SHA-256
`f70699200b4f5535a35577238e62030e542ee1337156765c9e6a7499238f88fc`.
Validation evidence is retained in the validation checkout under
`obj/parallel/string-prerequisites-combined-v1/`:

- All 3,167 regular and ignored maintainer tests pass in one release execution.
  That command took 1,600.28 seconds, including 14m34s of code generation.
- All 8,362 active text comparisons pass in 18.99 seconds.
- PA18 and PA24 each pass 470 independent source checks. Their native checks
  pass 290 and 313 executable comparisons respectively, three codegen-only
  comparisons each, and ten PA24 object comparisons.
- Formatting, strict all-target/all-feature Clippy, architecture/file audit,
  harness, upstream bytes and backend qualification pass.

The full guard removed 10,334 generated files, reclaiming 16.91 GiB, and
losslessly archived one large diagnostic. Twelve superseded compiler pins were
then retired, reclaiming another 1.01 GiB; their hashes and receipts remain.
An idle cleanup also removed 9,664 older unpinned generated files (16.84 GiB)
and losslessly archived one older diagnostic. The current baseline, candidate
and backend pins are retained; source inputs, diagnostics and measurements remain.

`obj/parallel/string-prerequisites-perf-v1/` freezes twelve inputs before
measurement. The nine width/reuse grid inputs are newly supported; the old
compiler rejects them, so they have no A/B speedup result. Three ordinary/opaque
controls have byte-identical A/B output. The combined candidate also passes four
native consumers against retained independent outcomes for identical inputs.
The exclusive comparison retains A/A calibration, five ABBA blocks, raw RSS
observations and instruction profiles. Shared instruction counts decrease
0.085–0.117%; measured median RSS does not increase. Increasing one grid
dimension fourfold produces instruction ratios of 1.003–1.039 for width and
1.391–2.183 for reuse. These whole-compiler observations complement the exact
logical-work invariants above; they do not prove complexity by timing alone.

Primary shared wall medians are 4.84–6.52% slower. The scalar observation exceeds
its 3.39% A/A spread, so a single declared attribution batch records child CPU
time, faults and context switches for all three shared controls. Its paired
wall medians range from -1.32% to +0.87%, and child CPU from -1.72% to +1.79%,
within that batch's A/A spreads. No major faults occur; scheduling counts are
comparable. Both batches remain recorded, including the original slower results.
Shared-machine launch/scheduling variation limits wall-time attribution; this
checkpoint makes no wall-time speedup claim. No further retiming was performed.

The primary analysis SHA-256 is
`a8b0006537255b293ff0396c8a3c0e7fd13ec05c68a965eaad81d410b32a5680`;
the separate attribution result SHA-256 is
`62ac94518706f1ea1913ef1c8a3f153206d91250b58d298a90ad0360ad8162e1`.
The later merge of main's already-qualified PA34 Python tooling changes no
`dev/` bytes, so it does not rebuild or repeat the completed compiler suite.
The merged tree passes all 152 harness tests, registration of 358 roots in 13
binaries, all 44 assignment layouts and the 24 upstream/126 Reference file pins.
PA25–PA44 remain planned, including both original panic-macro storage inputs
and the complete upstream library requirements.
