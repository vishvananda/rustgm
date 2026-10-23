# Source capabilities, dynamic cleanup and return captures

This checkpoint validates source-provider attributes against their actual language
roles, lowers dynamic-object cleanup through exposed vtable recipes, and selects
implicit return-opaque captures using the defining token's edition. Ordinary
library implementations remain Rust source. PA25–PA44 remain planned; no assignment
is activated and no text oracle is regenerated.

Whole upstream core, alloc and std builds remain incomplete. Source-client
metadata, callable const execution, runtime const_eval_select, library stability,
owned-box cleanup and the remaining library/target contracts are still required.
The CLI still exposes only its qualified Rust 2024 profile. The Rust 2021
measurements below use an explicit edition API adapter, not a new CLI promise.

## Frozen functional qualification

Production snapshot: `14c2fe3fe8714e80fc32c44cf3e4ddfd348a58e1`.
Qualified snapshot: `b2e254179e78d176c4a54a82499be4f3b6285dda`, which changes only
one private layout invariant after that production snapshot. Release CLI SHA-256:
`d5555e97d272a3cde262ae262fb9bc676224ecf4d732ee15a8c351e8711ebe4b`.

All **3,424 regular and ignored maintainer tests** pass, including native tests.
This comprises **2,206 retained tests in 13 integration targets** and **1,218
fresh library tests** after the isolated test correction. It is not a fresh full
suite run. All **8,382 active text cases**, affected independent source/native
suites PA17–PA24, and seven transitive-metadata native consumers pass. Formatting,
all-target/all-feature strict Clippy, the architecture/file audit, harness,
registration and pinned upstream checks pass. Focused integration also exercises
the new dynamic cleanup and return-edition native programs.

The initial full release run passed 3,423 tests and failed one stale assertion:
the destruction query's cold Scope now owns an optional binding pointer and is
56 bytes rather than 48. The correction accounts explicitly for that field and
keeps the outer NeedsDrop size at 72. Before rebuilding, qualification retained
each completed target's binary hash, exact inventory, result and source hashes.
The complete library target was rebuilt and rerun; the production compiler and
13 integration targets were unchanged. The original failure remains recorded.

| Component | Observed shared-machine wall time |
| --- | ---: |
| Original full release regular/native command, including its failure | 1,652.41 s |
| Optimized compilation within that command | 15 min 11 s |
| Corrected full release library build/run | 437.88 s |
| Library test execution within that recovery | 3.81 s |
| All active text cases | 26.12 s |
| Affected independent source checks | 235.30 s |
| Affected independent native checks | 284.94 s |

These are observed costs, not a controlled harness speedup. Receipts are under
`obj/parallel/source-capabilities-dyn-returns-focused-v1`, `-close-v1` and
`-close-v2`, with the common `source-capabilities-dyn-returns` prefix. Original
target-inventory SHA-256:
`ea0e1503598f90ff5f8a7fd34c056c6039271479eff90f2ebd74a4487bff7b36`.
Recovery checks SHA-256:
`2d94870dfe8385fa9cb390c8f9dd40d1adfda0d90b47c795f59666d8313c455a`.
Diagnosed failed-run cleanup reclaimed 27.66 GiB from 15,148 generated files;
sources, diagnostics, receipts and current comparison pins remain.

## Performance and algorithmic boundaries

The exclusive matrix freezes **37 inputs** against the preceding published
`cd6ce633…` compiler: **26 byte-equal A/B comparisons** and **11 candidate-only
measurements**. All 27 declared candidate native programs pass before timing.
Candidate-only acceptance is not a speedup over a failing baseline. The source
and edition adapters use identical driver bytes for both compiler versions and
call the real expansion, resolution, proof and lowering paths.

Comparable instruction changes range from **−11.40% to +3.31%**, with a median
of **−0.007%**. The largest median RSS increase is **440 KiB**. All declared 4x
isolated-axis instruction ratios lie between **1.15x and 3.78x**. No instruction,
RSS or scaling investigation threshold fires. Adding 128 unrelated declarations
costs 1.51x for the slice control and 1.28x for the dyn control; this cost is
reported separately and does not claim independence of whole-frontend work.

| Changed-path input | Size | Candidate instructions |
| --- | ---: | ---: |
| Repeated dyn root | 8 / 32 / 128 | 7,489,659 / 19,572,612 / 68,099,917 |
| Distinct dyn roots | 8 / 32 / 128 | 10,328,854 / 32,254,755 / 120,214,673 |
| Return bodies, Rust 2021 | 8 / 32 / 128 | 4,934,994 / 15,910,655 / 59,913,801 |
| Return bodies, Rust 2024 | 8 / 32 / 128 | 5,439,019 / 17,884,561 / 67,561,490 |

The Rust 2021 return family reduces instructions by 9.31%–11.40%. Its lifetime
inventory is retained during the existing bound walk, before alias normalization;
there is no extra source parse or whole-tree walk. The dyn family shares bounded
sparse structural queries and retains exact exposed recipes. Warm structure
never grants caller, cleanup-proof or const permission. Detailed work/storage and
record-size contracts remain in the owning documents linked below.

Paired wall-time medians range from **−7.84% to +6.53%**, with a median of
**−0.58%**. A/A spreads range from **1.02% to 38.19%**, with median **4.64%**;
these short shared-host observations do not establish a general wall-time gain.
Warmups, calibration, five ABBA blocks, RSS and Callgrind observations are retained
without rerunning to obtain a more favorable result.

## Retained benchmark preflight corrections

The first matrix stopped before native execution or timing. Its six return inputs
used an unavailable standalone `assert!` macro, and its original-provider input
required `fundamental`, unsupported by the baseline. Both failures remain recorded
in `obj/parallel/source-capabilities-dyn-returns-perf-v1`.

A separately declared v2 replaces the six return inputs with the established
explicit check helper, independently qualifying all six new sources through
metadata, object generation and native execution. The original sources and their
independent outcomes remain unchanged. The provider case becomes candidate-only;
it supplies no A/B speedup claim. A header-only control unexpectedly accepted by
the baseline remains candidate-only as originally declared. No compiler changes
or timing-policy changes were made to recover the benchmark.

Final evidence: `obj/parallel/source-capabilities-dyn-returns-perf-v2`.
Results SHA-256:
`4f6bcadd76253e734efc337c1db118dc70e72f8f7acc6006d8d1d4cbb79c7b05`.
Analysis SHA-256:
`8048205ed1347b914bd6c78546f090628c8a01189c47b2e0e1a3e19b0a34d2d2`.
Successful cleanup removes generated native objects/executables after hashes
are recorded. Current compiler and adapter pins remain for the next comparison.

Feature contracts: [provider capabilities](pa29-provider-capability-checkpoint.md),
[dynamic cleanup](pa31-dyn-drop-glue.md),
[return captures](pa34-return-editions.md). The
[library-stability contract](pa31-library-stability.md) is independently probed
preparation; this checkpoint does not implement library stability.
