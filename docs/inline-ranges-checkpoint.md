# Inline execution, byte ranges and shared iteration

This checkpoint extends public `08ad7822` with checked inline initializer
contexts, owned symbolic byte ranges, explicit fieldless enum tags through the
full integer widths, and shared slice/array iteration. PA1–PA24 remain active;
PA25–PA30 remain planned and incomplete.

## Behavior and qualification

The compiler retains the original initializer operands and checked caller
context across inline execution. Owned byte-array ranges preserve symbolic
fragments through the existing storage model. Explicit fieldless enum tags use
their actual integer width and validity checks. Shared collection iteration
uses canonical language/library identities and ordinary checked bodies.
These changes add no backend operations or alternate pointer representation.

The tested source snapshot is `c3b1f97bd68847368229cd1b8c9f18879c8830ec`.
The immutable default-feature compiler has SHA-256
`1d655c47c65b62a685b41bc0f8327077cd451e2224aa2f22f892dcc549cff497`.
All 1,795 recorded compiler, test and harness source hashes remain unchanged
after qualification. Later reference and documentation edits are separate.

The validation checkout's `obj/parallel/inline-ranges-checkpoint-v3/` records:

- 2,631 regular tests and 272 native tests passing.
- Strict all-target/all-feature release Clippy, formatting, the file/architecture
  audit (1,763 files, zero findings), 90 harness tests, registration, layout,
  upstream integrity and both backend capability checks.
- All 8,360 active text cases qualified, with the single reviewed PA6 update
  described below. No semantic acceptance or diagnostic changes were blessed.

The first full regular run found two stale private size assertions. Those
assertions now account for the source-initializer and pointer-method catalogs;
all 869 private tests were rerun. The unchanged thirteen integration groups
retain their 1,762 passing results from the full run. Production was unchanged
by the assertion corrections.

The affected independent checks cover 24 source/native groups: PA6 source,
PA13–PA14 source/native, PA15 source, and PA16–PA24 source/native. All passed.
The initial request for PA15 native checks failed because PA15 has no LowIR
cases; that failure is retained and no empty suite is counted as successful.
Receipts are in `obj/parallel/inline-ranges-independent-v{1,2}/`.

PA6's unchanged `for_binding.rs` now appends three canonical names, seven scopes
and 25 definitions when collection metadata is demanded. All prior records
retain their order, and both source references retain their target identities.
The owning contract and independent source outcome were reviewed before scoped
generation. Commit `5a1ae8ef` contains the generated reference and receipt;
the entire PA6 text suite and layout check then passed. The other 23 active text
suites retain their unchanged results. The review is recorded in
`obj/parallel/inline-ranges-reference-v1/`.

## Performance review

The main checkout's `obj/parallel/inline-ranges-performance-v1/` retains the
88-input frozen comparison against public `08ad7822`, whose immutable binary
hash is `5af2cfbfeb0e57c19f2c7c7e54252b1407759593d7a67c8cc7cd51c69f90cd67`.
The 73 shared inputs preserve exact output and metadata bytes. The 15 newly
supported inputs receive scaling/resource measurements without a baseline
speedup claim. Eighteen added source inputs were independently object-qualified
before candidate observations; retained inputs preserve their earlier hashes
and qualification.

The matrix uses A/A calibration, five paired ABBA blocks with three executions
per observation, eight RSS observations and Callgrind counts. Every observation
is retained. Instruction changes across shared inputs range from 0.04% lower
to 2.28% higher, with a median increase of 0.50%. No declared instruction or
scaling threshold was crossed. Complete scaling series have incremental work
ratios of 2.79–4.51 for fourfold input increases; these bounded measurements do
not prove asymptotic behavior for every possible program.

Six RSS medians crossed the review threshold, increasing 578–892 KiB
(5.26–7.23%). The single predeclared follow-up includes all six plus an ordinary
control. `obj/parallel/inline-ranges-memory-attribution-v1/` retains calibrated
RSS, page faults, heap/stack profiles and mapped-page profiles. Peak tracked
heap increases eight bytes on six inputs and decreases 1,978 bytes on the
remaining input. Independent stack maxima decrease on all seven. Mapped-page
peaks increase 188 or 196 KiB, consistent in scale with 200 KiB of added
executable sections.

The follow-up still observes RSS increases of 730 KiB for repeated byte access
and 596 KiB for prefix arguments. The evidence does not establish the precise
cause of the residual RSS differences, but does not show retained allocation
growth matching them. Both the original matrix and follow-up remain evidence;
neither is replaced by favorable samples. Publication accepts the documented
code-size and residency cost and makes no memory-neutrality claim.

The exclusive lease excludes registered RustGM jobs, not unrelated host work.
A/A spreads range from 0.68% to 69.93%, so these observations do not establish
a broad wall-time speedup. Host-load observations and the predeclared analysis
policy are retained alongside the results.

## Remaining work

Later projected-pattern and open-type caller changes, executable iterator
cloning and mutable iteration, payload/tagless enum storage, public reexports
and aggregate metadata are separate checkpoints. Some are locally qualified;
others still have demonstrated implementation gaps. Full PA25–PA30 contracts,
including generic crate bodies and remaining foreign interfaces, are unfinished.
This publication does not activate or declare completion of those assignments.
