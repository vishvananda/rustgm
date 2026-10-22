# Constant evaluation, slice operations and counted-copy checkpoint

This development checkpoint integrates selected PA25–PA28 functionality after
the published PA24 release. PA1–PA24 remain the active course assignments;
this checkpoint does not complete or activate PA25–PA30.

The production compiler is `b0d85e2809588ef1adf74cbbd0fdba43c5b5a0d9`.
Commit `9314f2ef72d62f91481e32921a097445d75e7785` changes only a maintainer
test's expected pointer-intrinsic count. Commit `3e7a8847` publishes the
independently qualified PA22 regression fixtures and their documentation.
The default-release compiler used for qualification and measurements has SHA-256
`228f9496bb6d9424bfd2ce89fb70cda6c6a63ef7c80358e827ee4f9df9650a05`.

## Included behavior

- Declaration-time constant demands retain their actual selected instance,
  caller scope and proof obligations. Closed aggregate and static shared-reference
  source results can complete without inventing a concrete generic result type.
- Slice length uses the retained slice metadata. Explicit array-reference to
  slice casts reuse ordinary reference coercion and lifetime/loan validation.
- Counted nonoverlapping copies and overlapping copies share checked typed
  spans and preserve source snapshots. Runtime overlapping copies use
  directional byte loops. Zero-byte operations retain address/alignment checks
  without claiming content access.
- Earlier PA20 ownership regressions are repaired. Existing published text
  outputs remain unchanged.

The detailed contracts and remaining boundaries are recorded in the owning
constant-evaluation, slice and transfer checkpoint documents. Open/projected
result proofs, broader language hooks, byte storage with pointer provenance,
additional atomic operations, metadata and foreign interfaces still need work.
Later worker commits are deliberately excluded from this qualified snapshot.

## Functional qualification

The isolated validation checkout retains evidence under
`obj/parallel/pa25-pa28-grouped-v1/` and
`obj/parallel/pa25-pa28-grouped-v2/`:

- All 336 maintainer roots are registered exactly once in 13 test binaries.
- Formatting, strict release all-target/all-feature Clippy, file and architecture
  audits, 89 harness tests, layout checks and upstream pin checks pass.
- The complete regular suite initially passed 2,398 functions and failed one
  stale expected registry count: the two added copy operations increased the
  count from six to eight. The test-only correction preserves its identity and
  body-work assertions. All 95 functions in the affected memory group pass on
  rerun, completing 2,399 regular functions across the original full run and
  corrective group run. The original failed log is retained.
- Inventory review accounts for three renamed tests whose earlier unsupported
  positive cases now complete. Original positive sources and genuine cycle or
  duplicate-tag negatives remain covered; no missing baseline tests are waived.
- All 234 ignored native maintainer functions pass, including optional
  interoperability checks. The default-feature release binary builds separately.
- All 8,255 existing PA1–PA24 text comparisons pass.

The full regular log SHA-256 is
`c539b4a9f237040c5b00238eff97020cf797e09d72f52fc8e50ea3ab90683415`;
the corrective memory-group log is
`2d687fa2aed5e1d53a15b99248b3125f850881c8c7e4d9e28dbe3168fcbf53dd`.
The reconciled regular receipt is
`c52c630a4a425975e3f9551d9a5a66275c2bf4846ce90fc8f55149e092678fd3`.
The successful qualification cleanup reclaimed 11.19 GiB while retaining
inputs, logs, receipts and the current compiler pin.

PA22 additionally gains 28 explicit reference-unsizing cast cases, independently
qualified before oracle generation: 18 accepts and 10 genuine language rejects.
The complete expanded 356-case PA22 source, native and text suites pass.
Its native matrix includes 120 executions and 134 compile-only comparisons.
All 984 earlier sidecars remain byte-for-byte unchanged. Every new oracle was
reviewed against the independent run, and every new positive LowIR output equals
the bytes used by the native checker. The active course now has 8,283 cases.
See [the cast qualification record](pa22-reference-unsizing-casts.md) for
generation, review and installation receipts, including the retained failed
attempt with an incorrectly placed evidence directory.

## Immutable performance comparison

Main checkout evidence is in `obj/parallel/checkpoint-performance-v2/`.
The baseline default-release binary has SHA-256
`c408b1205c7815a004309b471ee115db532c7dd3c3a070c0ca2e2904d858cd10`;
its production source map matches the published PA24 compiler. Both executables
and all 33 inputs were frozen and hash-checked before and after measurement.
All sources independently compile to objects with pinned Rust 1.98.0.

Six inputs succeed on both versions and produce identical output bytes. The
other 27 exercise newly supported behavior: the baseline rejects them, so they
provide candidate scaling evidence, not comparative speedups. The first source
qualification attempt incorrectly treated two entry-free controls as binaries;
the retained retry corrects their crate-type metadata without changing sources.

Measurements use an exclusive resource lease, CPU affinity, two warmup
observations, four A/A calibration observations and five paired ABBA blocks.
Each observation contains three executions. Candidate-only cases use the same
count of B executions. Every output is checked. Eight RSS observations per case
and Callgrind instruction profiles are retained alongside every wall-time sample.

| Shared input | Instructions, B vs A | Paired median wall change | A/A spread | Median RSS change |
| --- | ---: | ---: | ---: | ---: |
| ordinary512 | −0.138% | −2.201% | 6.577% | −100 KiB |
| constcall64 | +0.614% | +0.456% | 4.825% | +126 KiB |
| ordinary-lowir128 | −0.175% | −0.990% | 1.694% | +98 KiB |
| ordinary-transfers-8 | −0.143% | +4.440% | 4.482% | +24 KiB |
| ordinary-transfers-32 | −0.030% | −1.018% | 5.017% | +72 KiB |
| ordinary-transfers-128 | +0.016% | −0.848% | 4.125% | +66 KiB |

The small instruction changes and RSS deltas do not indicate a substantial
common-case regression on these controls. Wall observations are noisy and do
not establish a speedup or exact neutrality. In particular, the smallest ordinary
transfer case has a positive wall delta despite slightly fewer instructions.
The executable text section grows from 10,030,514 to 10,161,454 bytes; RSS alone
cannot distinguish code pages from heap allocations.

| Candidate family | Instruction ratio for first 4× input step | Second 4× step |
| --- | ---: | ---: |
| Open source demands: 16, 64, 256 | 3.636× | 3.894× |
| Overlapping transfers: 8, 32, 128 | 2.745× | 3.503× |
| Nonoverlapping transfers: 8, 32, 128 | 2.610× | 3.473× |
| Slice length calls: 8, 32, 128 | 2.981× | 3.662× |
| Reference casts: 8, 32, 128 | 3.061× | 3.667× |
| Ordinary transfer control: 8, 32, 128 | 2.693× | 3.520× |

The source-width matrix separately varies width 8/32/128 and calls or external
preparations 8/32. Fourfold width steps use 1.209–2.364× instructions; fourfold
call/preparation steps use 1.301–2.530×. These bounded observations do not prove
asymptotic complexity or eliminate a smaller width-times-resumption term.
Any proposed optimization of that term must first attribute work with structural
counters while preserving complete proof revalidation.

The predeclared analysis policy calls for attribution above a 5% common-case
instruction increase, an RSS increase exceeding both 512 KiB and 5%, or a 6×
instruction increase for a 4× input step. None is triggered. These are
investigation thresholds, not permission to discard smaller differences.
No measurements were repeated to select a favorable result.

Performance result SHA-256:
`71307e4631b3df848a9578f09e54dd1382b846693d1f5aa1d861e31b7b6690e0`.
Plan SHA-256:
`5ae45071d47c369bb107643933c36708952f031c85c59db21c1b2146c327b8ed`.
Completion SHA-256:
`3afbdc53649d76e292643cf4dfdfe4d773a15e8fe4caa3d01b84199eb218256e`.

## Development throughput

The grouped maintainer harness removes repeated compilation of shared test
helpers. Recorded full regular runs fell from 35m50s to 17m29s with identical
coverage; these are shared-machine observations, not isolated timing claims.
The combined checkpoint above took 16m14s for its first complete regular command,
followed by the necessary affected-group correction. Full native qualification
took 5m23s. The separate default-feature build took 3m30s.

Three workers continue in isolated checkouts under the existing three-job,
memory and disk limits. Full integration qualification uses a fixed cutoff while
workers proceed with the next checkpoint. Further test-build savings should
target the test-only interoperability feature that currently causes a separate
default frontend build, without dropping either feature configuration's checks.
