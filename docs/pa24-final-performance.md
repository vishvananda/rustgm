# PA24 final candidate performance

This comparison closes the changed-path measurement for the PA24 release
candidate. Baseline A is `1b93560f110c67c706a69c6748538289ab3bb028`, whose
production sources match public checkpoint `6e88cdb6`. Candidate B is
`0b89bf3166f11740f9925bb5006002468cd2c486`; the final functional qualification
at `c7fe46c0` has the same compiler and tests. Later PA25 development is outside
this comparison.

The immutable default-feature release binary SHA-256s are:

- A: `e6308578034792cd915c95a7564f96abbbd3cb5b13284a6056c06eeac9dd1db2`
- B: `c408b1205c7815a004309b471ee115db532c7dd3c3a070c0ca2e2904d858cd10`

All 18 frozen inputs passed independent pinned-rustc and CPPGM qualification
before measurement. Executable programs check their declared values; entry-free
libraries produce verified ELF relocatable objects without a synthetic main or
a runtime claim. The nine shared inputs preserve identical LowIR bytes. Nine
new inputs retain the baseline's declared rejection and have no baseline timing
ratio. Families use widths 8, 32 and 128: ordinary functions, scalar libraries,
closure libraries, owned Ord helpers, Ordering values and unaligned transfers.

The shared plan is AB warmup, four A calibration observations, then five ABBA
blocks, with three executions per observation. Candidate-only inputs retain the
same observation count. All 468 observations (1,404 compiler executions) check
status, diagnostics and frozen output bytes. Separate peak-RSS measurements and
27 Callgrind profiles supplement the timings. Binary/input hashes are rechecked
at completion. The exclusive lease excludes registered RustGM jobs; unrelated
host workloads remained active.

| Input | A instructions | B instructions | A/B peak RSS, KiB | Paired wall median change |
| --- | ---: | ---: | ---: | ---: |
| Ordinary, 128 | 32,335,166 | 32,278,993 | 11,688 / 11,876 | -2.56% |
| Scalar library, 128 | 20,546,561 | 20,527,316 | 10,984 / 11,140 | -2.62% |
| Closure library, 128 | 121,128,889 | 121,082,585 | 13,428 / 13,358 | -0.57% |
| Owned Ord, 128 | unsupported | 60,328,712 | — / 11,934 | — |
| Ordering values, 128 | unsupported | 95,412,131 | — / 12,654 | — |
| Unaligned transfers, 128 | unsupported | 37,004,768 | — / 11,288 | — |

Shared instruction changes range from -0.174% to +0.951%. The largest percentage
increase is the smallest scalar library, with 17,876 additional instructions;
its peak RSS rises from 10,350 to 10,676 KiB. No shared input adds more than
392 KiB median peak RSS. The A/A timing spread ranges from 1.82% to 58.49%, with
a median of 5.37%, so the small wall-time differences do not establish speedups.
These observations support accepting the changed paths without a material
ordinary-path regression; they do not establish performance neutrality.

The broader 45-input comparison remains in `pa24-sequence-performance.md`,
including repeated/distinct captures, quantified/projected inputs and compressed
constant storage. Its nested-receiver superlinear growth remains an explicit
optimization follow-up. No whole-frontend linearity claim follows from these
measurements.

Evidence is retained in `obj/parallel/pa24-final-performance/measurement-v1/`.
Resource receipt `1650c5877e92473bbecbce8356095cb9` closed with status zero.
The results SHA-256 is
`e5adb3a23b3d6e041d5c9c6d09c866d018f1f5e1dc91414072466ffda52d8e64`;
the completion receipt SHA-256 is
`f9878458d2f30e601af3e784f32c47b9ebd95278acc8610f495df010705f8755`.
