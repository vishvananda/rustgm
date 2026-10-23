# Source traits, destruction queries and edition context checkpoint

The integrated compiler now handles source-declared ordinary Copy/Clone/FnOnce
prerequisites, sparse concrete destruction queries, and defining-edition macro
fragment grammar with token-origin identifier context. This is prerequisite
work toward original upstream core, alloc and std. PA25–PA44 remain planned.
Rust 2021 is not yet enabled in the CLI; whole-unit configuration, imported
generic body transport, complete source Drop/Destruct and the remaining const
effect permissions are still required. See the [source trait boundary](pa31-source-trait-prerequisite.md)
and [edition context boundary](pa34-edition-context.md).

## Frozen qualification

Compiler commit: `aad1402569973707cf9799a8af7a0664ae35dfd2`. Immutable release executable SHA-256:
`f520dc5f488d13a95e3502a36c8973069d58471de6ac54d729a912ed84bd2d38`.

All 3,209 regular and ignored maintainer tests passed in one release execution.
All 8,362 active text cases passed without reference changes. Affected independent
source and native suites PA18–PA24 passed, along with strict release Clippy and
the backend check. Prior unchanged debug strict, architecture, harness and
focused receipts are retained and bound by the checkpoint. No assignment was
activated. Raw observations live in
`obj/parallel/source-traits-editions-combined-v1/`; its functional receipt hash is
`0c30536e44a16511ca0d649ed89fbed5f9ddd1dff0554d439e3c054f0cf5ebd5`.

| Qualification component | Observed time |
| --- | ---: |
| Complete maintainer Cargo command, including compilation | 28m 00.7s |
| Optimized compilation within that command | 14m 43s |
| All active text cases | 19.23s |
| Affected independent source suites | 199.88s |
| Affected independent native suites | 237.28s |
| Complete functional steps before final cleanup | 36m 32.2s |

These are observations on a shared machine under the existing resource limits,
not a controlled old/new harness speedup. The 1,079 private library tests took
3.32s; maintainer integration groups include external compiler/backend work.
The longest execution groups were traits (174.22s), memory (140.96s) and
projections (90.28s). The frequent text regression check is already below the
one-minute CPPGM comparison. Optimized build cost and independent execution
explain why a full qualification is longer. The final cleanup removed 13,105
generated files (16.93 GiB unshared blocks), retaining evidence and current pins.

The [checkpoint cadence](../TESTING_AND_REFERENCES.md#development-checkpoints)
uses exact focused debug batches while editing, early all-target strict Clippy
and audits, and one full merged release qualification. It avoids duplicate
library runs, separate regular/native builds and repeated cleanup scans.
CI keeps full release execution and debug library invariants; complete debug
integration execution remains an assignment-release check. Compiler-specific
results are never cached as permission to skip changed-candidate execution.

## Performance

Five common inputs compare against the published string-prerequisite binary
with byte-exact output equality. AABB warmups, AAAA calibration and five ABBA
blocks retain every observation. Each observation runs three executions;
RSS and Callgrind measurements are separate. An exclusive RustGM lease prevents
our workers from overlapping, but unrelated host load remains. A/A spreads of
3.16–14.34% and observed external load limit wall-time interpretation.

| Existing control | Instruction change | Median wall change | A/A spread | RSS delta |
| --- | ---: | ---: | ---: | ---: |
| ordinary-scalar | +0.86% | +9.54% | 4.75% | -794 KiB |
| opaque-wide-128-repeat-128 | +1.33% | +10.87% | 13.45% | -124 KiB |
| ordinary-string-control | +0.64% | -0.05% | 7.31% | -368 KiB |
| generic-empty-128 | +0.63% | +1.40% | 14.34% | -388 KiB |
| ordinary512 | +0.51% | +5.64% | 3.16% | -536 KiB |

The other 21 inputs are new source-trait cases rejected by the old binary.
They have candidate-only observations, not A/B speedup claims. Seven distinct
axes use widths 8, 32 and 128: repeated Copy/Clone/FnOnce calls, distinct Copy
const instances, distinct Clone wrapper instances, distinct FnOnce instances
and unused nominal definitions. Copy varies const N with fixed u8; Clone has
one actual u8 Clone target behind multiple wrappers. These are not measurements
of many different nominal Clone implementations. Frozen worker work counts
confirm reached preparation and specialization, not just parsing. The 4× width
instruction ratios range from 1.69× to 3.52×.
No declared trigger fired: >5% instructions, RSS growth both >512 KiB and >5%,
or >6× instructions for a 4× isolated width increase. This supports the tested
scaling dimensions; it is not a proof of global linear complexity.

The 21 exact inputs retain independent rustc object/native observations and
merged-candidate constants-inspector agreement. Actual trait native consumer
controls also ran in the full release. This does not claim merged native
execution of every scaling input. The separate Self-constructor successor is
not part of this frozen compiler or its measurements.

Raw data and declaration: `obj/parallel/source-traits-editions-perf-v1/`.
Analysis SHA-256: `230a5e1f78663b7edcde81f528259adc1937203875993fb2104a723453248c84`.
Measurement completion SHA-256: `d7da3f2ecf38c64d5ccb460e02e6d70d92e6beb3700b58f6b073dae5f0c0690c`.
All observations, rejected-baseline diagnostics and prior failed hypotheses
are retained. Later changes in this publication are documentation only.
