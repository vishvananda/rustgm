# Checked memory and caller checkpoint

This checkpoint extends the compiler published at `7117fe9f` with selected
initializer execution, canonical pointer offsets and symbolic byte-array
transport. It also removes repeated shared-requirement expansion during cleanup
liveness recording. PA1–PA24 retain their existing grading contracts; PA25–PA30
remain planned.

## Compiler behavior

- Projected tuple/record constant operands and nested associated initializer
  dependencies retain their exact caller and original declaration. Publication
  resolves only the newly produced dependency rows through that body's retained
  equalities before substituting caller arguments. Original rigid operands are
  validated once before execution dispatch, with full instance proof retained.
- Public generic callable signatures prepare their own aggregate storage roots
  separately from opened body types. Canonical iteration trait visibility follows
  the real prelude/import/implementation identities, including inherited `Self`.
- Raw pointer `add` uses registered inherent method identities and preserves
  const/unsafe permission, checked multiplication, original allocation bounds
  and zero-effective-offset identity. Runtime lowering uses the existing backend
  operations.
- Complete initialized `[u8; N]` values may transport symbolic pointer fragments
  through existing constant Images. A demanded scalar byte still rejects
  provenance, references still require their own validity checks, and published
  integer arrays cannot retain pointer provenance. The implementation adds no
  second pointer or allocation model.
- Cleanup recording shares immutable region/parameter component sets and updates
  multiplicities only when entity liveness changes. The existing fixed-point
  solution, entity identity, cleanup order and complete loan proof remain intact.
  Distinct requirements still produce their required output, including cases
  whose actual output is quadratic.

The owning development records are `pa25-readiness.md`,
`pa25-cleanup-liveness.md`, `pa26-iterator-public-signature.md`,
`pa26-iteration-method-visibility.md`, `pa27-array-fragments.md` and
`pa28-pointer-add.md`. Their independently declared positive and negative
fixtures are exercised by maintainer tests while the assignments remain planned.

## Qualification

The frozen tested snapshot is `33750a623edcd19e64ed293a74a3c987cbafe8a3`.
Its immutable default-feature binary has SHA-256
`5af2cfbfeb0e57c19f2c7c7e54252b1407759593d7a67c8cc7cd51c69f90cd67`.
The later documentation and history merge preserve all 1,767 recorded compiler,
test and harness source hashes. The history receipt is
`obj/parallel/checked-memory-callers-checkpoint-v2/history-reconciliation.json`.

Combined qualification in the `rustgm-pa29-work` checkout includes:

- 2,588 regular tests and 267 native tests.
- All 8,360 PA1–PA24 text cases matching committed references exactly; no new
  reference generation or diagnostic normalization.
- Strict all-target/all-feature release Clippy, formatting, the architecture/file
  audit (1,735 files, zero findings), harness, layout, upstream and both backend
  capability checks.
- Thirteen affected independent suites: PA6 source, plus source/native suites
  for PA16, PA17, PA19, PA20, PA21 and PA24. Their declared stage differences and
  original source bytes remain unchanged.

The initial complete regular run passed 2,587 tests and failed one exact layout
assertion. The measured extra eight bytes belong to the lazy pointer-method
catalog. The corrected expectation is 8,544 bytes for Declarations. All 832
library tests were rerun; the thirteen unchanged passing integration groups
retain their original evidence. No production fix was needed for that failure.
Receipts and the complete regular inventory are in
`obj/parallel/checked-memory-callers-checkpoint-v{1,2}/`; affected independent
receipts are in `obj/parallel/checked-memory-callers-independent-v1/`.

## Performance

The main checkout's `obj/parallel/checked-memory-callers-performance-v1/`
contains the frozen 70-input combined comparison. It reuses 51 independently
qualified source inputs and adds 19 independently object-qualified controls.
Preflight confirms 54 exact-output A/B pairs and 16 newly supported inputs.
The baseline is the immutable public `7117fe9f` compiler. Newly supported inputs
receive scaling/resource observations, without a baseline speedup claim.

The matrix uses one exclusive registered-job window, A/A calibration, five
paired ABBA blocks with three executions per observation, eight RSS observations
and Callgrind instruction counts. Every raw sample is retained. All 70 inputs
completed with output equality preserved; there were no instruction or scaling
attribution triggers. Across the 54 shared inputs, instruction deltas range from
72.74% lower to 2.98% higher. All complete 8/32/128 series have incremental
instruction ratios between 2.26 and 4.52 per fourfold input increase.

The two width-128 cleanup families use 12,018,029 and 17,990,560 instructions,
down from 44,082,796 and 49,978,092. Their paired median wall times improve 46.7%
and 37.3%, with A/A spreads of 4.0% and 3.1%. The observer-context family also
benefits from the shared cleanup change. These results describe these frozen
programs and do not establish a whole-language speedup.

Two RSS medians cross the predeclared attribution threshold: ordinary-lowir128
increases 626 KiB (5.66%) and prefix-arguments-8-shared increases 668 KiB (6.09%).
The one permitted attribution pass, including ordinary512 as a control, is
declared in `obj/parallel/checked-memory-callers-memory-attribution-v1/` in the
main checkout. It retains the original observations and compares heap/stack,
mapped-page, calibrated RSS and page-fault measurements. That pass is complete:
the two RSS changes do not reproduce, with paired medians instead falling 638
and 186 KiB; the ordinary512 control falls 1,440 KiB. Peak tracked heap is
unchanged for both ordinary controls and 782 bytes lower for prefix arguments.
The combined heap/stack peaks rise 400, 6,208 and 304 bytes respectively, while
mapped-page peaks rise one 4,096-byte page on all three inputs. Prefix arguments
has a measured 6,208-byte stack increase; this is not a memory-neutrality claim.

These observations do not support retained allocation growth matching the
original roughly 0.6 MiB RSS changes. The RSS reversal and control variation are
consistent with process residency/run variation, without proving its precise
cause. The original matrix remains authoritative evidence alongside this one
predeclared attribution pass; neither is replaced by favorable observations.
All measured outputs remain equal. The attribution `analysis.json` binds the
original analysis, declaration, results and retained raw profiles. The completed
performance review permits publication with these limitations.

Generic inline output contracts, open type arguments and selected reference
patterns remain required PA25 work. Shared collection iteration, the remaining
unsafe/intrinsic families and full crate/foreign contracts are separate unfinished
work. This checkpoint does not claim assignment completion.
