# Compressed constants and captured-input checkpoint

The completed comparison is in
`obj/parallel/pa24-sequence-performance/measurement-v1/`. A uses compiler
`1857ac3983632f5dfafe120f1b2d89d93f7208d6`, whose production sources match the
previous public checkpoint. B uses `1b93560f110c67c706a69c6748538289ab3bb028`;
the subsequent `6e88cdb6` repair changes only tests. Later library proof-order,
comparison helpers, unaligned transfers and named-pattern changes are outside
this comparison.

The immutable release binary SHA-256s are:

- A: `8091103388ad9be595cbe2fe7a40e2f0cb64cd66009bbe7f722abb789319148e`
- B: `e6308578034792cd915c95a7564f96abbbd3cb5b13284a6056c06eeac9dd1db2`

All 45 frozen inputs were independently compiled and executed with pinned rustc
and RustGM/CPPGM before measurement. Thirty-three are accepted by both compilers
with byte-identical LowIR; twelve newly supported inputs are candidate-only,
with their previous rejection preserved. Every measured execution rechecks status,
diagnostics and output equality. Input and binary hashes are checked again at
completion.

For each shared input the order is an AB warmup, four A calibration observations,
then five ABBA blocks, with three executions per observation. Candidate-only
inputs have the same number of executions and no invented baseline ratio.
Separate RSS samples and Callgrind instruction counts accompany every input.
The exclusive resource lease excludes registered RustGM jobs; unrelated host
workloads remained active. All raw observations are retained.

## Observations

| Input | A instructions | B instructions | A/B peak RSS, KiB | Paired wall-time median change |
| --- | ---: | ---: | ---: | ---: |
| Ordinary, 128 items | 32,192,078 | 32,345,206 | 11,392 / 11,668 | -0.40% |
| Repeated quantified calls, 128 | 229,286,825 | 234,270,554 | 14,230 / 14,536 | +2.05% |
| Receiver depth, 128 | 330,817,014 | 330,479,435 | 18,422 / 18,778 | -2.19% |
| Constant array, 1,000,000 elements | 106,659,459 | 1,530,092 | 64,896 / 10,248 | -94.00% |
| Mixed constant array, 1,000,000 elements | 106,802,385 | 1,674,621 | 65,002 / 10,282 | -93.55% |

These RSS entries are medians of separate peak-RSS executions. The shared-input
A/A spread ranges from 1.11% to 71.22%, with a median of 7.26%; small wall-time
differences do not establish speedups. Instruction counts are separate evidence:
ordinary-128 grows 0.48%, repeated quantified calls grow 2.17–2.45% across the
three widths, and the smallest constant controls grow about 71–73 thousand
instructions (4.55–4.90%). These modest costs accompany the added source-contract
checks and supported behavior; they are not hidden by an aggregate speedup.

Compressed constant storage removes the large array's element-count traversal:
B uses 1,528,270 instructions at eight elements and 1,530,092 at one million.
The new slice case similarly uses 2,004,489 and 2,006,000 instructions. The
million-element shared cases reduce instructions by 98.43–98.57% and peak memory
by about 53 MiB. Their large timing improvement exceeds the measured calibration
noise. No element-count-independent claim is made for arbitrary constant programs.

The nested receiver family still grows superlinearly. Its existing quadratic
region/constraint producer is documented in `pa24-receiver-depth.md`; this
checkpoint neither repairs it nor claims linear whole-frontend complexity.

Call-edge analysis of these retained profiles narrows that follow-up without a
new benchmark. At depths 8/32/128, body proof remains six calls and runtime point
inference remains three. Propagation transfers along component edges grow from
86 to 1,848 to 77,972 word visits; its inclusive instructions grow from 75,646
to 1,812,031 to 77,280,473. Expanding equivalent-region results costs only 190,264
instructions at depth 128 (0.058% of the total), ruling it out as the main target.
Structural requirement queries also become more expensive despite linearly
growing call counts. The next investigation needs graph-edge, seed and retained
word counts to distinguish necessary output size from redundant propagation and
repeated shared-tail materialization. These findings do not justify weakening
lifetime constraints. Profile hashes and parsed counts, whose self-cost totals
match each complete profile, are retained in
`obj/parallel/receiver-depth-investigation-v1/call-counts.json`.

The frozen checkpoint also passed the full release Cargo suite (2,316 tests),
native maintainer suite (219 tests), affected independent source/native suites,
all 7,787 active text cases and required lint/audit/harness/backend checks. It was
published to public main as `6e88cdb6`. PA24 remains planned while its later
library-proof correction and complete release candidate finish qualification.
