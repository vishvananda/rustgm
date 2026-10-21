# Generic closures and callable object measurements

This development checkpoint measures public `bbe280e2` against candidate
`be2e9cbca706938a6c881daac282606c0093874c`. It covers captured generic closures,
captured inline constants and borrowed callable objects. The later generic
receiver-adapter and pointer-memory checkpoints are not included. PA24 remains
planned; these measurements do not qualify the complete assignment.

Both executables are immutable default-feature release builds. Their SHA-256
hashes are `5bff94e7d90773adeb57382f74497347a477ae5e7b5e16c803172fdaa71edf0b`
(baseline) and
`a1ddeb165396a057719cfce797408b18a7f10ff944a05207e55a634da05bc739`
(candidate). Raw inputs, declarations, every timing sample, memory observations,
Callgrind profiles, outputs and completion receipts remain under
`obj/parallel/generic-object-performance/measurement-v1/`. The generating script
is retained beside that directory as `measure.py`.

All 24 frozen programs first pass pinned rustc 1.98.0 object generation and
independent rustc-versus-CPPGM native execution, including source assertions of
the complete expected values. Each shared input has byte-identical LowIR between
the two compilers. Candidate-only inputs reject on the baseline and execute on
the candidate; no speedup ratio is assigned to those unsupported baseline cases.
Every timed output matches its qualified bytes, and executable/input hashes are
verified again after measurement.

The run uses exclusive admission among registered RustGM jobs and CPU affinity
to CPU 2. Each shared case has initial AB observations, four A/A observations,
then five ABBA blocks, with three executions per observation. Candidate-only
cases retain the same observation count. There are 624 observations containing
1,872 timed frontend executions, plus eight peak-RSS samples per case and
separate instruction profiles. Unrelated host work remains outside the resource
controller and can affect wall time. No samples are discarded or rerun to
obtain a more favorable result.

## Shared controls

Positive percentages mean the candidate costs more. A/A spread is the range
relative to the smallest calibration observation, not a confidence interval.
The table includes all nine shared cases, including noisy small inputs.

| Family | Width | Instructions | Paired median wall time | A/A spread | Candidate milliseconds |
| --- | ---: | ---: | ---: | ---: | ---: |
| Ordinary functions | 8 | +1.82% | -2.58% | 19.44% | 4.635 |
| Ordinary functions | 32 | +1.25% | +4.65% | 4.35% | 5.100 |
| Ordinary functions | 128 | +1.12% | +3.77% | 2.71% | 9.482 |
| Generic functions | 8 | +1.87% | +15.77% | 32.81% | 5.228 |
| Generic functions | 32 | +1.59% | +4.08% | 8.52% | 4.996 |
| Generic functions | 128 | +1.11% | +5.00% | 11.46% | 9.411 |
| Generic empty closures | 8 | +1.81% | +11.93% | 34.84% | 6.001 |
| Generic empty closures | 32 | +1.51% | +5.31% | 1.08% | 6.402 |
| Generic empty closures | 128 | +1.54% | +4.70% | 1.60% | 13.360 |

This is a measured ordinary-path instruction regression of 1.11–1.87%, not a
neutrality claim. In particular, the 32/128 empty-closure wall increases exceed
their calibration spreads and remain a regression to track. Peak RSS increases
by 82–1,100 KiB across the shared cases. Executable text increases from 9,455,634
to 9,767,586 bytes. Timing includes process startup and complete LowIR output;
these figures are not isolated semantic-query costs or a rustc speed comparison.

## Newly executable paths

Widths are distinct specializations/targets or repeated uses, as named. Values
below report candidate measurements only. The final column compares instruction
counts when input width grows fourfold from 32 to 128.

| Family | 8: milliseconds | 32: milliseconds | 128: milliseconds | Instructions, 128 / 32 |
| --- | ---: | ---: | ---: | ---: |
| Distinct captured specializations | 4.443 | 7.258 | 16.979 | 3.648 |
| Captured inline constants | 4.967 | 8.578 | 20.823 | 3.686 |
| Repeated captured specialization | 5.290 | 5.011 | 7.507 | 3.184 |
| Repeated dynamic callable | 4.651 | 7.065 | 29.130 | 3.606 |
| Distinct dynamic targets | 6.292 | 13.631 | 41.332 | 3.902 |

These instruction ratios are consistent with linear work over the tested range;
they do not establish an asymptotic bound for every program. Peak RSS ratios over
the same widths range from 1.047 to 1.186. The 128-wide distinct capture and
dynamic-target timing calibrations are noisy (32.09% and 36.37%); their wall
numbers should not support fine-grained performance conclusions.

Separate typed invariants in `erased_dispatch` retain one table, concrete target
and physical forwarding function for 8/32/128 repeated calls. Planning work is
106/274/946 steps, seven additional steps per added call. Those checks establish
the intended cache reuse independently of whole-process timing. Native execution,
cache invariants and measurements all remain separate from the full publication
gates for this exact candidate.
