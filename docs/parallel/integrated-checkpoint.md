# PA24–PA30 integrated development checkpoint

Compiler commit `abeedb233ea414642819f5127877d28e66cad406` passed the combined
publication gates. This is a development checkpoint: PA1–PA23 remain active;
PA24–PA30 remain planned. Captured closure execution, complete constant
evaluation, library hooks, unsafe validity, the full atomic family, general
crate metadata and the remaining foreign ABI work still need implementation
and their assignment release checks.

The checkpoint combines generic noncapturing closures and their inherited
lifetime proofs, checked constant function-pointer identities, constant
inspection, assignment-trait selection, mutable indexing, canonical AtomicU32
load/store, scalar crate metadata/imports and scalar foreign linkage. Earlier
development records describe their supported subsets and explicit boundaries.
The later capture schema, inline-constant scheduler, primitive assignment
factories and symbolic raw-read work are not part of this qualification.

## Functional evidence

The isolated validation checkout retained source hashes and checked the compiler
binary before and after validation. All 62 recorded gates passed:

- 7,786 byte-oracle cases across all 23 active assignments.
- 2,026 passing release Cargo test functions and 158 passing native maintainer
  test functions. These are harness counts, not individual PA case counts.
- 6,593 independent source checks across PA9–PA23.
- 4,282 independent native-suite checks, including 2,364 executable comparisons
  and 314 codegen-only comparisons, across PA11–PA14 and PA16–PA23.
- 38 focused test functions and 23 independently qualified draft source cases.
- Formatting, strict all-target/all-feature release Clippy, file/architecture
  audits, audit tests, layout, harness, upstream and pinned backend checks.

Evidence lives in `obj/parallel/publication-integrated/` in the validation
checkout. `complete.json`, `checks.json` and `sources.json` retain the exact
commands, statuses, timings and hashes. An initial release build was interrupted
for cleanup before approaching the worker storage allowance; `maintenance.json`
records that interruption. Completed checks were preserved, the release gate
was rerun successfully, and later native gates completed. Cleanup reclaimed
9.06 GiB of investigated failed-run artifacts, then 1.88 GiB and 7.68 GiB after
the successful release/native batches. Source inputs and diagnostics remain.

## Immutable performance evidence

`obj/parallel/integrated-performance/measurement-v1/` in the main checkout
compares baseline `4a449961` with the qualified candidate. Binary SHA-256:

- A: `38ac5f0d73fc5a40ccb0a6c794d02cead90cf9ea23ecf516fed358f5ff49483a`
- B: `0137ba89180d37f67bdd320ed57d6d651757e33e66c994acd2202b0065e4519e`

Fifteen frozen inputs, independently accepted by pinned rustc, exercise widths
8, 32 and 128. Ordinary functions and generic noncapturing closures have shared
accepted A/B output; all six LowIR outputs were byte-identical. Constant
closures, assignment traits and atomics are candidate-only workloads because
the baseline rejects those inputs. They are scaling observations, not speedups.

The exclusive RustGM lease used CPU 2, two warmups, four A/A observations,
five paired ABBA blocks, three executions per observation, eight RSS observations
and Callgrind instruction counts. New workloads used the same observation count
with B/B calibration. Every timing observation and output hash is retained.
Other projects can still load the shared machine.

Shared instruction changes ranged from -0.433% to -0.133%. Paired median wall
changes ranged from -7.47% to +15.14%, with A/A spreads up to 28.69%; these
measurements do not establish a reliable wall-time improvement or regression.
Shared median RSS changes ranged from -114 to +198 KiB. The executable text
section grew 3.50% as the supported functionality expanded.

| Candidate-only workload | Instructions, width 8 | Width 32 | Width 128 | Width 128 wall median | Width 128 RSS |
| --- | ---: | ---: | ---: | ---: | ---: |
| Constant closures | 4.324 million | 15.353 million | 60.205 million | 19.77 ms | 11,724 KiB |
| Assignment traits | 5.302 million | 17.313 million | 65.593 million | 28.15 ms | 10,972 KiB |
| Atomic load/store | 5.375 million | 17.743 million | 67.133 million | 19.83 ms | 11,006 KiB |

Instruction growth is approximately linear over these measured sizes. This
bounded development matrix does not replace the broader assignment release
matrix. A first measurement admission failed because a cleanup command still
held the storage lock; it produced no timing observations. The subsequent
exclusive run completed all 15 inputs and exited successfully.
