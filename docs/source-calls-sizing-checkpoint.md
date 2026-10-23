# Source sizing, generic calls and edition checkpoint

This checkpoint transports checked generic helper calls, reuses complete actual
source sizing proofs within each caller's proof context, validates actual Unsize and
CoerceUnsized roles with both endpoints' formation debts, and selects external
safety/linkage and temporary lifetimes from their defining editions. It includes
the preceding source Freeze, sized drop-glue and generic-original transport
checkpoints. Ordinary library algorithms remain in Rust source.

The driver still exposes only the qualified 2024 profile. Original core, alloc
and std remain incomplete. Source trait/header and sysroot-client transport,
const-call closure, unsized slice/dyn/Box cleanup, remaining active source
attributes/features, edition semantics and target crate orchestration remain
required. No planned assignment is activated.

## Frozen functional checks

Compiler: `e3db713f4d472bc086bf6aff8ef9d7c2b932d449`.
Release CLI SHA-256:
`4f0278e33d66f7fbcc65b8d7f9b0f3860d21f2c0dfe86300807292973f6405be`.

The fresh combined release run passes **3,362 regular and ignored maintainer
tests**. All **8,382 active text cases** pass without reference changes. Affected
independent source suites PA13–PA24, corresponding native suites except PA15,
and seven transitive-metadata native consumers pass. Formatting, all-target
strict Clippy, the 2,078-file architecture audit, 164 harness tests, registration
of 370 roots in 13 binaries, upstream bytes and the backend gate pass.

The selected debug library preflight passed 1,175 cases before release code
generation, settling the merged private proof/layout and paired-origin tests.
The release library lane includes an additional ignored control. This is a fresh
combined qualification, unlike the separately retained scoped recovery on the
preceding compiler. Old failed runs and their exact corrections remain recorded.

| Component | Observed shared-machine wall time |
| --- | ---: |
| Debug private preflight, including compilation | 131.18 s |
| Full release regular/native Cargo command | 1,752.10 s |
| Optimized compilation within that command | 17 min 13 s |
| All active text cases | 19.29 s |
| Affected independent source checks | 248.28 s |
| Affected independent native checks | 292.73 s |

These observations separate compilation, execution and text costs; they are not
a controlled old/new harness speedup. Focused debug development, audit before
Clippy, exact recovery and one merged release avoid repeating the optimized
build for routine edits. Full Cargo targets now collect every target's failures
with `--no-fail-fast` while still returning failure.

Receipts live in `obj/parallel/source-call-sizing-close-v1`. Functional receipt
SHA-256: `5bbee7480c7e3f2b55f820f3d2827d94f4eebd0a685cedd6342a644138759359`.
Automatic cleanup removed 14,497 generated files, reclaiming 17.20 GiB, and
losslessly archived one large diagnostic with its original-byte checksum.
Sources, diagnostics, receipts and current comparison pins remain available.

## Performance and algorithmic result

The exclusive run in `obj/parallel/source-call-sizing-perf-v1` freezes 59 inputs.
Twenty-one common inputs compare with the published source-destruction binary;
sixteen accepted Freeze/v0-metadata inputs compare with the preceding generic
checkpoint. All 37 comparisons have byte-identical output. The two previously
rejected depth-128 inputs and twenty new call/Unsize inputs have candidate-only
observations, not speedup claims against a failing baseline. Produced generic
metadata also matches the frozen worker artifacts on every invocation.

The source sizing correction removes repeated fresh canonical proof roots for
nested aliases. Exact frozen programs retain semantic depth while avoiding the
separate unchanged syntax-nesting limit. Prior parser and work-budget failures
remain preserved; neither input budgets nor language requirements were relaxed.

| Alias depth, no unrelated types | Before instructions | After instructions | Candidate wall time |
| --- | ---: | ---: | ---: |
| 8 | 7,233,352 | 4,925,906 | 6.84 ms |
| 32 | 62,565,669 | 13,925,925 | 6.81 ms |
| 128 | Failed work budget | 49,586,049 | 14.08 ms |

Depth 32 uses 77.74% fewer instructions and has a paired median wall reduction
of 55.97%. Depth 128 now completes. Its earlier failed instruction observation
is retained separately and is not an equal-outcome A/B comparison. Whole-compiler
instruction ratios for 4x depth increases are 2.83x and 3.56x. Worker inference
work is 2,691 / 10,587 / 42,171, unchanged by 128 unrelated types; those counters
remain distinct from whole-compiler parsing and declaration work.

All declared 4x isolated-axis instruction ratios are between 1.02x and 3.67x,
below the 6x investigation threshold. The new producer/import axes distinguish
repeated calls from distinct helpers. Unsize axes distinguish associated-bound
width from caller count and include region-bearing endpoints. The eight
entry-free Unsize inputs measure the typed semantic view; their native behavior
is qualified separately by the combined release.

Five ordinary controls change instructions by -0.09% to +0.64%. The three
128-caller `needs_drop` controls trigger review at **+5.24% to +5.35%**. Offline
attribution places approximately 1.65 million added instructions in trait-goal
selection across widths 8, 32 and 128. Each fixture has 128 distinct constant
initializer owners. The new source nominal-formation checks precede physical
shape selection and retain caller-specific proof graphs, including when a
nominal has no header clauses. Source/profile inspection supports a bounded
per-caller correctness cost, not a return of the nested-depth recurrence; this
is an attribution inference, not a new call-count measurement. The cost is
accepted and retained as a future empty-preparation optimization target without
sharing caller proof or bypassing formation checks. The full review is in
`attribution-v1/review.json`; no instrumented rerun was performed.

No memory or scaling trigger fires. Other paired wall observations range
from +0.49% to +16.40%; they are retained rather than represented as an overall
wall-time improvement. A/A spreads of 1.52%–62.63% and unrelated host load limit
interpretation of these short timings. Every observation, including warmup,
calibration, all five ABBA blocks, RSS and Callgrind, remains available. The
performance results SHA-256 is
`8e5cdc5e382f0e0f05f0cbdea04eba9d9635c85d3f3025492d85dd3c66c04b69`.
