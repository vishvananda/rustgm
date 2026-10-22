# Open constant results, impl lifetime inputs and scalar byte storage

Compiler/test cutoff `6fca476ba81a425ec340a6b0ee4bf9eda5437899` integrates
three qualified changes: open-lifetime source initializer result conversions,
completion of impl-owned omitted lifetime inputs before header selection, and
scalar byte storage that preserves initialization and invalid scalar bits during
constant memory transfers. Ordinary typed reads still enforce validity. Original
and instantiated body proofs remain required; copying bytes creates no lifetime
or pointer authority. See [source computations](pa25-source-computation.md),
[impl lifetime sealing](pa19-impl-lifetime-sealing.md) and
[byte transport](pa28-byte-transport.md) for their exact contracts and gaps.

This publication also removes the empty test-only interoperability Cargo feature.
The same five native tests remain registered with their original bodies and
ignore markers. Explicit native execution still requires the pinned backend.
This is a development checkpoint: PA25–PA30 remain incomplete and planned.

## Combined qualification

The frozen validation checkout passed formatting, strict release all-target
Clippy, architecture/file audits, 90 harness tests, layout and upstream checks.
All 337 integration roots are registered once in 13 test binaries. The complete
release run passed 2,413 regular test functions; explicit native execution passed
237 functions. Inventory comparison against the preceding published checkpoint
found no missing or changed baseline test outcomes. All 8,283 existing active
course cases matched their text oracles exactly.

Regular qualification took 17m46s, including 12m29s of compilation; native
qualification took 6m06s. The subsequent default production build took 0.166s:
Cargo reported both the library and executable fresh, with empty feature lists,
and the executable hash was unchanged. This removes the redundant build observed
at 3m30s in the preceding checkpoint; these elapsed times are shared-host
observations, not an isolated timing comparison between equivalent full suites.

Candidate SHA-256:
`1d7fe2a2af3a8f956dc1e928130cb6a22eb6c496a3fdef123464dc1cb921a3e9`.
Evidence is in `rustgm-validation-work/obj/parallel/pa25-pa28-open-byte-v1/`,
including `complete.json`, exact test inventories, source hashes and
`default-reuse.json`. Successful cleanup reclaimed 13.28 GiB and losslessly
archived one large diagnostic while retaining its original-byte hash.

## Owning fixtures and reference review

PA19 gains 21 impl-lifetime cases: 16 executable positives and five required
rejections. PA20 gains one associated output-lifetime rejection. The expanded
suites passed 2,506 independent source checks, retaining their six documented
acceptance differences rather than claiming universal rustc acceptance parity.
Their 1,520 LowIR/native checks include 793 executed comparisons, with no
compile-only positives. Reference generation followed these independent checks.
All 7,452 prior reference sidecars are byte-identical; every new oracle triple
matches the independently qualified frontend output, and positive LowIR is the
exact text executed by the native checker.

The original worker payload and its hash remain recorded. Review corrected its
Boolean native metadata to explicit execution contracts for positives and no
native declaration for negatives; no source or expected acceptance was changed.
The single canonical comparison case remains in PA26's planned outcome inventory.
PA19's regenerated receipt additionally records the currently required case
contracts; PA20 preserves every prior contract hash. Both receipts record the
actual generating compiler sources. No references were edited by hand.

The active course now has 8,305 cases: PA19 has 1,124 and PA20 has 1,382.
Installed fixtures passed the 90-test harness, layout and upstream checks.
Source, native, generation, review and installation evidence is retained under
`obj/parallel/impl-lifetimes-publication-v1/`.

## Performance and algorithmic checks

The exclusive frozen study compares the previous public compiler
(`228f9496bb6d9424bfd2ce89fb70cda6c6a63ef7c80358e827ee4f9df9650a05`)
against the candidate above. All 32 inputs independently compile as Rust objects.
Fifteen inputs succeed under both frontends with identical output; seventeen
are newly supported and have no claimed speedup against the rejecting baseline.
The retained matrix has 832 observations of three executions each, 256 RSS
samples, instruction profiles and A/A calibration followed by five paired ABBA
blocks for shared inputs. Inputs and binaries were checked before and after.

Shared instruction deltas range from -2.107% to +1.324%; median RSS changes range
from -88 to +484 KiB. Neither exceeds the predeclared attribution thresholds.
The ordinary transfer series has no worsening asymptotic instruction trend.
Paired wall medians range from -9.765% to +3.657%, with shared A/A spreads up to
91.869% and substantial block variation. These samples do not establish tight
wall-time neutrality or a general speedup; all observations are retained.

For converted byte storage, increasing backing width from 64 to 4,096 bytes
changes the instruction increment for 8 to 128 repeated operations by +0.042%
for records and -0.093% for unions. The sparse aggregate count series 8/32/128
has instruction ratios 1.0006 and 1.0025 per fourfold increase. These observations
accompany the private work/storage invariants for conversion once, bounded range
access and repeat encoding once. They do not prove whole-compiler complexity.
Untouched ordinary immutable union Images retain their known width-times-repeat
copy cost; instruction counts alone cannot exclude byte-volume or bandwidth
costs. Relocation-bearing storage and its consumers remain separate required work.

The analysis policy was frozen before measurement; no attribution trigger fired
and no favorable-result rerun was performed. Evidence is retained under
`obj/parallel/open-byte-performance-v1/`, including the plan, all source outcomes,
raw observations, RSS samples, Callgrind profiles and `analysis.json`.
Results SHA-256:
`b4ba53cf908b89004c0c43368d0e2c1f38c8c1391213b2b3a42381a581a518ea`.
Plan SHA-256:
`6ae3d12e9b8fda42a88a8d6a0ddc0a99271fa617c7e5d542bc97b59b59b8aa2c`.

Later projected-pattern and call-argument inference worker checkpoints are not
part of this frozen compiler cutoff. Their qualification and any reference
changes belong to the next integration cycle.
