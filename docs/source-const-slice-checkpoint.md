# Source const calls, slice cleanup and edition checkpoint

This checkpoint checks conditional const obligations at each reached call,
validates actual source callable const eligibility, lowers source slice cleanup,
borrows validated source-role headers, and selects never-coercion fallback from
the compiling crate's edition. Ordinary library implementations remain Rust
source. No planned assignment is activated and no text oracle is regenerated.

Whole upstream core, alloc and std builds remain incomplete. Source-client
metadata, actual FnOnce const invocation, runtime const_eval_select, library
stability, owned-box cleanup and the remaining library/target contracts are still
required. The provider-capability, dynamic-object cleanup and return-capture
edition successors are separate, unqualified integrations at this publication.
The CLI continues to expose only its qualified Rust 2024 profile.

## Frozen functional checks

Compiler: `cd6ce633078df54ddeee368f1547fa60070a56f0`.
Release CLI SHA-256:
`b70e362fb8dd6d7680ca76296aa1fa14acefcffa4ab17e667ae1ecbcd7cc4588`.

The fresh combined release run passes **3,400 regular and ignored maintainer
tests**, including native tests. All **8,382 active text cases** pass. Affected
independent source and native suites PA17–PA24 and seven transitive-metadata
native consumers pass. Formatting, all-target/all-feature strict Clippy, the
2,101-file architecture audit, 164 harness tests, registration of 371 roots in
13 binaries, upstream checksums and the backend gate pass.

The debug private preflight passed 1,199 tests; the full release library lane
includes one additional ignored control. Worker failures and scoped recoveries
remain separately recorded in their owning checkpoint documents. The merged
release result is a fresh complete pass, not a sum of worker results.

| Component | Observed shared-machine wall time |
| --- | ---: |
| Debug private preflight, including compilation | 135.67 s |
| Full release regular/native Cargo command | 1,642.16 s |
| Optimized compilation within that command | 15 min 13 s |
| All active text cases | 19.33 s |
| Affected independent source checks | 198.28 s |
| Affected independent native checks | 243.65 s |

These are observed costs, not a controlled harness speedup. CI now runs
formatting, architecture checks and strict Clippy in that order, without
repeating formatting in the same Make invocation. Focused development batches
remain the first integration check; the full release is a publication gate.

Receipts: `obj/parallel/source-const-slice-close-v1`. Functional receipt SHA-256:
`4d32fde9c198a9342c9dd9b37b5d062e3f866c64caf65091320414ab7a95d39e`.
The successful storage wrapper removed 13,564 generated files and reclaimed
17.13 GiB. Sources, diagnostics, receipts and current comparison pins remain.

## Performance and algorithmic result

The exclusive matrix freezes 60 inputs. **39 comparisons** against the preceding
published compiler have byte-identical output. The ten new reached-const and
eleven new slice inputs have candidate-only measurements; acceptance does not
constitute a speedup against a failing baseline. All fourteen callable scaling
programs and eleven slice scaling programs pass candidate native execution
against their previously declared, independently qualified observers before
timing starts.

The source-lowering adapter calls the checked provider pipeline and lowers
prepared roots. It makes no public source-metadata export claim. Its binary hash
is `3793edccfb215d1623a0d1aef31dd445a23296feee111f5f1a618c70e7c10474`;
both compiler versions use the identical adapter source. The frozen ordinary
CLI baseline is the published `4f0278e3…` binary documented in the
[preceding checkpoint](source-calls-sizing-checkpoint.md).

Across comparable inputs, instruction changes range from **−1.92% to +1.90%**,
with a median of +0.38%. All declared 4x isolated-axis instruction ratios range
from **1.02x to 3.68x**, below the 6x investigation threshold. No instruction,
RSS or scaling trigger fires; the largest comparable median RSS increase is
348 KiB. Ordinary controls change instructions by −1.92% to +0.66%.

| Changed-path input | Size | Candidate instructions | Candidate wall time |
| --- | ---: | ---: | ---: |
| Repeated reached const calls | 8 / 32 / 128 | 2,890,690 / 5,750,511 / 17,116,592 | 4.70 / 5.58 / 8.48 ms |
| Distinct reached const callers | 8 / 32 / 128 | 5,177,169 / 14,918,531 / 54,259,746 | 6.02 / 14.07 / 27.95 ms |
| Slice element width | 8 / 32 / 128 | 3,923,149 / 4,508,599 / 6,761,103 | 6.51 / 9.28 / 10.68 ms |
| Shared callable signature depth, eight calls | 8 / 32 / 128 | 7,679,483 / 16,224,786 / 50,468,966 | 8.65 / 11.64 / 23.73 ms |

The [callable eligibility checks](pa31-callable-const-eligibility.md) separately
measure newly charged reconstruction work: 96 steps at all three shared
signature depths, zero retained new types, and zero new eligibility work for
ordinary pointer controls. The whole-compiler measurements above include
parsing and semantic work and must not be confused with those counters.
Slice recipes retain one element dependency and emit one runtime loop; runtime
length and zero stride do not expand compile-time traversal. Reached const
payment remains specific to each call's caller context and cannot be reused as
permission merely because its structural code is warm.

Paired wall-time medians increase by **0.59% to 19.68%**, with a median of
**5.52%** across comparable inputs. This checkpoint does not claim a wall-time
improvement. A/A spreads of 0.89%–36.54% and other host activity limit
interpretation of these short observations; they do not erase the increases.
Warmup, calibration, five ABBA blocks, RSS and Callgrind observations are all
retained without rerunning for a more favorable result.

## Retained harness failure and recovery

`obj/parallel/source-const-slice-perf-v1` passed all 60 source/output preflights,
then failed its first native backend invocation because the saved LowIR had a
`.stdout` suffix. The object bridge correctly requires `.lowir`. No timing had
started and no backend compilation occurred for that invocation.

The separate `source-const-slice-perf-v2` recovery copies the identical saved
bytes to `candidate.lowir`, retains the successful preflight receipts, and runs
all 25 native programs before the unchanged timing policy. Compiler, inputs,
baselines and expected results are unchanged. The original failure is retained;
the successful recovery removes its 50 generated object/native files after
recording their hashes.

Performance results SHA-256:
`07e92586ef1de90af874d5b46a6bde7fec137f6d26ae99db8ab9c1be95db4b04`.
Native preflight SHA-256:
`4c41bbc81d4e4a66f86ae57041b286400b1d02498c021f5c35b005d708b21ff0`.
Analysis SHA-256:
`b76714ef49a8cef7db0085c80f590b523898f8a52f2352238738fdd6fe9a9efb`.
