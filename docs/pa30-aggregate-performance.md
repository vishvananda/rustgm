# PA30 aggregate ABI frozen comparison

The aggregate callable checkpoint adds a small, measurable ordinary-code cost.
Across five shared inputs, Callgrind instructions increase by 0.70–1.38% and
median peak RSS increases by 94–474 KiB. Wall observations are too noisy to
claim either a general speedup or a precise slowdown. The two new aggregate
inputs show approximately proportional instruction growth over the measured
64-to-256 declaration range; they have no executable baseline comparison.

The integrator completed the predeclared exclusive run with exit zero. All
594 compiler observations succeeded and matched their frozen LowIR hashes,
with empty stderr. Every observation, including outliers, remains in
`obj/parallel/pa30-aggregate-measurement-v1/` in the main worktree: `plan.json`,
`observations.jsonl`, `summary.json`, raw outputs, time records and Callgrind
traces. Preparation evidence remains in the worker's
`obj/parallel/pa30/aggregate-performance/complete.json` and `preparation.json`.
The run needed no automatic repetitions or discarded measurements.

## Attribution and protocol

Both immutable binaries use default release settings, Cargo `debug=1`, and no
feature or RUSTFLAGS override:

| Version | Source | Binary SHA-256 |
| --- | --- | --- |
| A | `84f6f84e9ef1d74b5d7c29a565edcb7451b0ffac` | `936856895c4a75957f23b115c4225f9c7dd505025290c19face727fd19a24e6d` |
| B | `07037425c40574de2743f91a5cdd82eead9ef7fb` | `9e3f9ce91dea488600d3f8d3b3d2e403c1113c894dbb96f21a0128324aa1e6d6` |

The pins isolate the aggregate implementation, before the later malformed
object-address validator and callable-selector changes. Inputs and executable
hashes were checked before and after measurement. All seven inputs were first
independently accepted and executed by Rust 1.98.0 and the candidate's pinned
object backend. The five shared inputs have identical A/B LowIR bytes, status
and stderr. A rejects both aggregate inputs at its declared missing aggregate
result-classification boundary.

Each shared input has four A/A pairs and five ABBA blocks, with three compiler
executions per position. Each label also has eight separate peak-RSS runs and
one Callgrind instruction observation. Each candidate-only input has fifteen
timing executions, eight RSS runs and one instruction observation. The process
was pinned to CPU 0. The exclusive controller excluded other guarded RustGM
jobs; unrelated Vizier/v5rust activity could still affect wall time.

## Shared inputs

Wall delta is the median of the five per-block B/A ratios minus one. A/A and
ABBA ranges retain all four and five ratios respectively. RSS columns are
medians of eight separate process maxima, not paired differences.

| Input | A/A ratio range | ABBA ratio range | Median wall delta | Instructions A → B | Instruction delta | RSS A → B (KiB) |
| --- | --- | --- | ---: | --- | ---: | --- |
| scalar | 0.968–1.089 | 0.927–1.192 | +4.73% | 1,287,001 → 1,296,024 | +0.70% | 10,162 → 10,636 |
| macro64 | 0.997–1.106 | 0.971–1.042 | +0.68% | 14,971,931 → 15,157,821 | +1.24% | 11,064 → 11,158 |
| scalar-c64 | 0.818–1.077 | 0.782–1.047 | −3.48% | 14,567,585 → 14,768,780 | +1.38% | 10,878 → 11,276 |
| macro256 | 0.984–1.028 | 1.002–1.028 | +1.32% | 57,057,659 → 57,846,360 | +1.38% | 12,324 → 12,716 |
| scalar-c256 | 0.993–1.054 | 0.997–1.026 | +1.77% | 55,325,782 → 56,059,539 | +1.33% | 12,400 → 12,802 |

The instruction increase appears on every shared input, including ordinary Rust
scalar code. It is a bounded implementation cost in these controls, rather than
zero overhead. The RSS deltas are +474/+94/+398/+392/+402 KiB, or
+4.66/+0.85/+3.66/+3.18/+3.24%. A single instruction observation per label and
eight RSS observations do not establish population confidence intervals.
The larger inputs' wall medians are consistent with a small added cost, but A/A
drift and the limited number of blocks prevent a stronger timing conclusion.
In particular, the scalar-c64 wall decrease is not evidence of a speedup.

## New aggregate inputs

| Input | Median wall (ms) | Median RSS (KiB) | Instructions |
| --- | ---: | ---: | ---: |
| aggregate64 | 8.950 | 12,020 | 36,040,446 |
| aggregate256 | 27.304 | 14,462 | 140,909,255 |

For four times as many distinct aggregate declarations, instructions grow
3.910×, median wall grows 3.051×, and median RSS grows 1.203×. This is a bounded
scaling observation at two sizes, not an asymptotic guarantee. It does not
measure the native C call cost or compare aggregate compilation against an
older functioning aggregate implementation. These results support retaining
the sparse object representation while acknowledging its ordinary frontend
cost; they do not complete or activate PA30.
