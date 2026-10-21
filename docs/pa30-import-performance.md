# PA30 foreign ownership performance qualification

The isolated comparison uses the clean post-Naming baseline
`5f1945cb4e3fe978fa0c4089aa5d24c2c0352e00` and scalar-import candidate
`8de31d0521c119ddf234c1a13ee1041358304142`. Both include the parent's earlier
`6b684da5` callable implementation. Its temporary type-bundle inspection churn
is shared by the two binaries; the later correction is outside this comparison.

Both binaries were built with Rust 1.98.0 in release mode, optimization level 3
and debug information level 1, default repository flags, two build jobs and
incremental compilation disabled. The baseline was built from an immutable Git
archive under this worker's output directory. The build receipt retains archive
hash, exact commands, compiler version and relevant environment values. Immutable
binary SHA-256 values are:

- Baseline: `5bfaa568c3c052ad66d82974be8ff12cc504ba646a00100a6b545c3d5076a8ec`
- Candidate: `637e1bd0c9462235487969d5a6189146d35f78f4dd09288b19bec642d9cb6f5a`

Disposable Cargo artifacts were pruned before requesting the exclusive lease.
The measurement pins remained read-only and were hash-verified before execution.
Eight frozen inputs were independently qualified by pinned rustc object generation
before timing. Shared scalar and macro-declaration controls use 64 and 256 units;
candidate-only controls use the same counts of imported calls and callbacks.

The study ran on one fixed CPU with warmups, two A/A pairs and three ABBA blocks,
three compiler executions per observation. All 88 observations and 264 executions
are retained. Every successful shared execution had identical output, stderr and
exit status. The paired median ratios are candidate divided by baseline:

| Shared input | Wall time | Peak RSS |
| --- | ---: | ---: |
| Scalar 64 | 0.9996 | 0.9897 |
| Macro 64 | 1.0155 | 1.0024 |
| Scalar 256 | 0.9746 | 0.9990 |
| Macro 256 | 1.0140 | 1.0227 |

Macro wall-time medians were about 1.4–1.6% higher in this run. These short
millisecond controls do not support a tight threshold claim: A/A wall ratios
range from 0.9866 to 1.2543, and individual ABBA blocks remain in the raw record.
RSS ratios vary between blocks; the table reports paired block medians rather
than a ratio of pooled observations. This is a bounded changed-path study, not
the assignment's full performance matrix.

Candidate import median wall time grows from 4.055 to 5.431 ms for 64 to 256
calls, with peak RSS medians 9,432 to 10,040 KiB. Callback controls grow from
4.369 to 6.450 ms, with RSS 9,380 to 9,876 KiB. These are measurements of the
frontend producing LowIR, separate from native interoperability execution.

Evidence remains in `obj/parallel/pa30/import-performance/`: `build.json`,
`plan.json`, `observations.jsonl`, `summary.json`, immutable pins, frozen sources,
independent diagnostics and representative identical outputs. Ordinary build
receipt `21615e5d23544be69895f21ea2ece801` and exclusive measurement receipt
`d9fb344d644e4c5da18cba43a639f40d` both finished with status zero. No observations
were discarded or overwritten. No assignment activation or oracle changed.
