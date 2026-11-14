# Source unchecked integer shifts

Clean release23 from36b794b08 clears original core's source layout work guard.
Its unchanged diagnostic checks42310 declarations and25858 bodies, with no
failed/blocked functions. Completion reaches the genuine unchecked_shl declaration
through generated usize::unchecked_shl at expanded file12:1858633. Postcheck
passes722 input hashes,375 opens, six configuration pins and3957 compiler hashes
under the same120s/work/byte limits. No original library artifact exists.

The exact source-selector guard is captured under
obj/parallel/intrinsic-completion/debug23/. Current expanded23's complete output
has SHA256f5b6fb7ad812f41193314f6211950c1329a64442d78b0bcb9d224402ea262659,
identical to the preserved historical expansion. This independently binds the
call to the original unsigned macro and64-bit usize substitution. The large
expanded offset is not an offset into the65KiB physical num/mod.rs.

The [PA31 packet](../pa31/tests/spec/unchecked-shifts/README.md) retains attributed
intrinsic declarations and the original unsigned wrapper cutout. All576 integer
results (12 LHS by12 RHS types, two directions and count0/width-1) were declared
before execution. Pinned Rust/C observes every exact result, including128-bit
values and signed right extension. The actual wrapper independently returns2^63.
Six runtime calls separately cover mixed RHS widths and signed/128-bit results.
An unmarked ordinary function must retain its own99 result.

Eight type/safety/const/Copy/prototype/count opponents independently reject.
The noninteger f32 opponent instead causes pinned Rust ICE101 in both CTFE and
reached runtime forms. Those are retained infrastructure failures, never semantic
rejections or passing reference results. RustGM must reject that unsupported
scalar. Initial no_core Neg/equality setup errors are retained; equivalent casts
and a plain99 authored body corrected setup without inverting required positives.
All first inputs and the explicitly corrected initial status classifier remain
under oracle1/. oracle2/ observes576 values and the actual wrapper; runtime3/
observes six runtime results. Baseline23 reproduces the precise missing intrinsic
for wrapper/matrix/scaling sources; the ordinary control succeeds.

The implementation reuses the existing typed integer-arithmetic selection and
constant shift kernel. The selection carries the exact RHS type, and each
consumer authenticates both evaluated arguments against the complete current
signature. Check both actual own Copy premises with the shared bounded checker;
its one-input comparison/arithmetic contract still permits only its original
subject and premises. CTFE uses the full RHS value and always rejects a negative,
equal-width or larger count before narrowing. Runtime valid unsafe calls use
existing LowIR shift/cast operations, including signed right extension. No new
source-name authority, fabricated function body, quota, dense AST/body/value
field, wire/backend/bridge or storage.py change is added. Existing imported
intrinsic transport fences remain; the backend stays frozen v3/17b024f69.

Focused1794643393395070066 passes six exact tests in127.47s: all576 shift bits,
ordinary99, prepared wrapper/runtime programs, nine candidate guards, bounded
8/32/128 preparation and old product/layout/wide-constant controls. Strict
selected/CLI Clippy, full formatting, architecture and metadata pass. Inventory
is3958 Rust files, with unchecked_shifts.rs registered once in frontend. Earlier
attempts remain: invalid selection, the one-subject Copy assumption, the last
same-type dependency check and a test-edit format error. No outcome was hidden.

Native1794644281771914889 passes three exact tests/five actual programs in78.27s:
the576-value matrix, actual usize wrapper, six mixed-type runtime calls, the old
156-product observer and original complete Field offsets. Strict checks pass.
The first native attempt was refused before launch; a later admission succeeded
without changing limits. Its successor hash/counter replay was then refused
before launch at152.8GiB free versus153.8GiB required including reservations.
Authenticated replay and final local harness/upstream remain pending headroom;
CI will run the complete harness/upstream and source tests on publication.

Full43570894822 PASSED EVERY STAGE on preceding36b794b08, including3077 private
tests in release/debug and the persistent layout guards. This shift checkpoint
requires its own successor. Controlled timings and a fresh unchanged original
core attempt remain pending; no speed claim, original artifact or whole PA31
release is made. Preserve every earlier required positive.
