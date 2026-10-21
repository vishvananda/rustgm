# Captured execution publication checkpoint

Compiler snapshot `68d3a32e` adds direct captured closure execution, adjusted
capture facts, scalar union constant images, AtomicU32 swap/fetch_add, and
borrowed receiver binding corrections. PA24–30 remain planned; these development
checkpoints do not establish assignment completion.

Combined qualification finished at `171a3bda`. All active PA1–23 text suites,
166 maintainer native test functions, independent source suites PA9–23, and
independent native suites PA11–14 and PA16–23 passed. Formatting, strict release
all-target/all-feature Clippy, file/layout/upstream audits, harness and backend
checks passed. The full release Cargo run completed every target; its only
failures were two stale assertions in `owned_operands`. The corrected target
passed all five tests, with unchanged compiler sources and other test targets.

The PA16 zero-length named-constant repeat oracle now omits an unused object
and three address/copy instructions. The pinned Reference, rustc object
compilation and both native executions independently support that change.
The original failed byte check and stale test failures remain in the evidence,
with their precise reviewed replacements. No fixture outcome was changed.

Local publication evidence is
`rustgm-validation-work/obj/parallel/publication-captures/complete.json`.
The authentication pause interrupted source PA21; its partial evidence remains
separate, and only unfinished suites were resumed. The validation executable
has SHA-256 `041153d5ff8c1f7c71f71c67818fac5d74be623f75c64252987bddf5278aefd8`.
The performance and independent zero-repeat pin has SHA-256
`76a94721c579c2e008d0c6b1d3b873c0fb96058d975292cc608c709781c3bff2`.
Their executable code and allocated data sections match; whole ELF hashes
differ. The comparison receipt records each section and both exact binaries.

The completed 18-input development performance matrix retains immutable pins,
frozen rustc-qualified inputs, output comparisons, calibration, paired timing,
RSS and instruction observations under
`obj/parallel/capture-performance/measurement-v1/` in the integration checkout.
It exposed superlinear precise-capture borrow work. The separate loan-expiration
optimization and its follow-up measurements are not part of this compiler
publication checkpoint.

The subsequent controller-only change permits the user-authorized three-job
trial. All 72 Python harness tests pass, including the additional slot,
per-worktree exclusion, nested third-slot inheritance, exclusive admission,
idle configuration and stale-policy rejection. Compiler sources remain those
of the qualified snapshot. Per-job limits and disk safeguards are unchanged.
