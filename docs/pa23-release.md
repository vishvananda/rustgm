# PA23 release qualification

The complete functional release qualification and frozen performance study
are complete. The results and observed overhead are recorded below. PA24 and PA25 remain planned.

## Contract and fixture evidence

The matrix has 399 course-authored LowIR sources: 262 accept, 137 reject;
85 accepted programs execute and 177 explicitly check compilation only.
All outcomes were independently qualified before generating byte oracles.
`obj/pa24-pa25-work/public-inputs-v2/qualification-v1/` records the first 396
sources and `qualification-supplement-v1/` records the final three. Their
manifests and original source hashes remain alongside the receipts.

The former PA19 future-object rejection moves to PA23 acceptance with identical
source bytes. PA19 therefore contains 1,104 cases. No artificial stage gate
retains rejection of an implemented language feature.

The only changed existing output is PA21's semantic view of
`contracts/polymorphic-outputs/accepted/semantics/universal-forward.rs`.
Its opaque-bearing quantified return obligation now uses the resumable canonical
selection path. The proof source is the actual annotation, the local region
block is ordinary scoped storage, and an explicit equality to `bound:0:0`
replaces the implicit alias interpretation. The proof keeps its existing
region equality and scope; the source count decreases by one and the relation
count increases by one. Acceptance, source bytes and its LowIR counterpart
remain unchanged. All other active outputs matched before reference generation.

The qualified compiler candidate is
`ac46cc710053ff4b0638e63345baf43de2dac21516d41505894a49002395e406`,
pinned in `obj/pa24-pa25-work/defining-regions-v4/binaries/`. The focused
checkpoint passes strict Clippy, all 472 library tests, 17 independently checked
defining-region outcomes, ten native comparisons, the 1,127-file audit,
four audit-boundary tests and the harness. These focused results preceded the complete release batch below.

## Algorithmic evidence

At 8/32/128 dependent defining bodies, V4 builds exactly 8/32/128 original CFGs,
consumes 191/767/3,071 lifetime facts and performs 81/153/441 constant preparation
steps. With 32 defining bodies and 32/128/512 unrelated constants, preparation
is 209/401/1,169 steps: two additional steps per constant across the entire chain.
Completed CFGs and contracts survive into ordinary lowering, which still runs
its full proof gates. Evaluator caches retain only report-owned indexes and
cleared storage; active queries and body permissions remain local.

`defining-regions-v4/performance-v1/` compares immutable V3 and V4 binaries on
six identical inputs, with A/A calibration, five ABBA blocks, three executions
per observation, eight RSS observations and Callgrind profiles. Every LowIR
output matches exactly. Instruction changes are +0.051% for the ordinary control,
-0.044%/+0.083%/+0.016% at dependency widths 8/32/128, and -0.197%/-0.811% for
128 bodies with 512/2,048 unrelated constants. The eliminated product-cost setup
was a small fraction of these complete compilations. Wall-time observations
are noisy and do not establish an overall speedup.

Cleanup receipts `1792533145901666045`, `1792533185494345936`,
`1792533296344255644` and `1792533356528531368` are retained under `obj/cleanup/`.
Temporary native binaries and rustc object files were removed; raw inputs,
diagnostics, timing observations and the current immutable pins remain.


## Complete functional qualification

`obj/pa24-pa25-work/pa23-release-v1/validation-v1/` passes all 70 gates in
950.22 seconds against the exact candidate above. Its source snapshot and final
hash checks confirm that compiler and fixture bytes did not change during the run.

| Gate | Result |
| --- | --- |
| Release maintainer tests | 1,827 passed |
| Explicit native maintainer groups | 122 passed |
| PA1–PA23 byte oracles | 7,784 passed |
| Independent source checks | 7,583 passed; 11 declared stream fragments |
| Independent native checks | 4,285 passed; 2,364 executions and 314 compile-only comparisons |
| Formatting, strict Clippy, file/phase audit | Passed; 1,127 audited files, zero findings |
| Audit unit tests, harness, discovery, upstream, backend, token/literal/layout checks | Passed |

These totals preserve earlier assignments' documented language/runtime differences;
they are not an unqualified claim of rustc parity. PA23 itself has no declared
source-outcome differences. Its 85 executions and 177 compile-only controls pass
through the unchanged qualified CPPGM backend.

Generated LowIR was reviewed for 16-byte data/vtable references, destructor/size/
alignment headers, conditional vacant entries, aggregate calls and deterministic
upcast targets. All rejected PA23 inputs have empty stdout and ordinary diagnosed
failures; none relies on a not-implemented or later-assignment diagnostic.
Reference regeneration changed only the reviewed PA21 semantic output and removed
the migrated PA19 rejection; the remaining existing output bytes are unchanged.

The successful full-run cleanup removed 9,158 generated artifacts and recovered
8.50 GiB of unshared blocks. Receipt: `obj/cleanup/1792534455827460309.jsonl`.
Its broad historical scan was slow; subsequent measurements use explicit cleanup
roots, preserving source, diagnostics, raw observations and current pins.


## Broad performance qualification

`obj/pa24-pa25-work/pa23-release-v1/performance-v1/` retains 38 frozen inputs,
every observation, RSS samples and instruction profiles. Baseline A is published
`a8219d0f`, SHA-256
`7beb2470a90c1afd68db18b1846db17fcada873c08589aae081c5a2e717a3997`.
Candidate B is the exact functionally qualified compiler above. All 29 shared
outputs are byte-identical. The other nine inputs require newly implemented
object or defining-body behavior, so their timings are candidate-only. All
38 sources pass pinned rustc object compilation.

Shared instruction changes range from -0.214% to +2.596%. The ordinary 256-function
control is +0.169% in instructions, with a 15.00 ms candidate median and +4.33%
paired wall median. Across the shared set, paired wall medians range from +0.97%
to +15.97%; the median of those percentages is +6.33%. Small inputs show the
largest percentages. RSS changes range from -306 to +868 KiB. These observations
include measurable overhead; they do not establish performance neutrality or a
speedup. Wall-time noise and code-layout/startup effects have not been isolated
as causes. No follow-up was selected merely to obtain a favorable timing.

| New workload | Widths | Instructions (millions) | Candidate wall medians (ms) |
| --- | --- | --- | --- |
| Object dispatch | 8 / 32 / 128 | 7.99 / 26.54 / 100.67 | 5.67 / 15.35 / 42.63 |
| Object upcasts | 8 / 32 / 128 | 9.65 / 31.91 / 120.99 | 6.74 / 18.36 / 52.41 |
| Captured defining dependencies | 8 / 32 / 128 | 18.51 / 69.87 / 276.05 | 8.79 / 35.95 / 71.56 |

These instruction-growth samples complement the actual-work invariants; they
are not a proof about every Rust program. The raw A/A calibrations, five ABBA
blocks, three executions per observation and eight RSS samples remain available.
The candidate-only rows use repeated candidate observations rather than a
speedup comparison with a compiler that rejects their inputs. Cleanup removed
38 temporary object files, receipt `obj/cleanup/1792535010547426074.jsonl`.
