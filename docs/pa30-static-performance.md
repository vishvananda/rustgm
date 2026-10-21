# PA30 foreign storage and shared namespace measurement

This fixed comparison isolates the foreign-static checkpoint and the extension
of the canonical name index to a typed function/global namespace. Both sources
already contain exact function linkage, the shared callable roots API and the
ValueScope correction. No combined-main performance claim is implied.

The clean baseline is `e0fb5c82611c45aa8af512a435b40db846d37ce9`, executable
SHA-256 `2ae11cb15cf10574c1d0eb70e7e4988e8780d8b2803a0d8cce537420bec6488e`.
The clean candidate is `9451a93d8875c720a3715a057f8e93e7014626e9`, executable
SHA-256 `376e9edea988c8b8e3c9511d35e9015627749ee25c817cdebada0807dd0c1392`.
Both immutable pins were built with Rust 1.98.0, release optimization level 3,
debug level 1, default flags, two Cargo jobs and incremental compilation disabled.
The baseline is a retained Git archive, not a live checkout. Disposable A/B
Cargo artifacts were pruned through the storage guard before measurement.

Six inputs and their source hashes were declared before qualification. Shared
controls contain ordinary scalar functions and local static objects with
relocations, each at 64 and 256 items. Candidate controls contain foreign
static imports plus exported generic callback storage at those sizes. Every
input was independently accepted by Rust 1.98.0 object emission before timing.
All shared frontend status, stderr and LowIR bytes agree across A/B. The
inherited PA20 inspection delta also has exactly equal A/B output, confirming
its independence from this checkpoint.

The exclusive lease retained exactly 80 observations and 240 executions: one
warmup per applicable binary/input, two A/A pairs, three ABBA blocks for each
shared input, and three measured candidate observations for each new input.
Each observation retains three executions. Every status, empty stderr and
output hash was checked. One predeclared Callgrind observation per shared input
and binary produced eight instruction traces; no repetitions or follow-ups ran.

| Shared input | A instructions | B instructions | B/A | Median paired wall B/A | Median paired RSS B/A |
| --- | ---: | ---: | ---: | ---: | ---: |
| Scalar 64 | 13,492,401 | 13,488,990 | 0.99975 | 1.01415 | 0.92111 |
| Storage 64 | 18,136,489 | 18,341,757 | 1.01132 | 1.04776 | 0.97171 |
| Scalar 256 | 52,078,399 | 52,083,546 | 1.00010 | 1.01413 | 0.92319 |
| Storage 256 | 72,019,117 | 72,823,203 | 1.01116 | 0.91492 | 0.98109 |

The ordinary scalar instruction observations are effectively unchanged; local
storage adds about 1.12–1.13% instructions in these controls. That bounded cost
accompanies canonical global-name validation and declaration/definition state.
The experiment does not attribute individual instructions to a sub-operation.

A/A wall ratios span 0.87431–1.12477. Paired wall blocks are also noisy, so these
measurements do not establish a tight wall-time threshold. Unrelated host
workloads, process layout and allocator behavior limit wall/RSS inference even
though repository heavy jobs were excluded. Lower observed RSS is not a claim
of an implementation memory improvement. Raw observations and calibration are
retained without dropping outliers.

Candidate-only median wall time grows from 8.799 ms at 64 to 14.970 ms at 256;
median maximum RSS grows from 10,220 to 11,748 KiB. These are scaling observations
for the declared compile-only controls, not native execution or a constant-factor
comparison with an unsupported baseline feature.

Evidence is under `obj/parallel/pa30/static-performance/`: the declared study
plan, source hashes, Git archive, build settings/logs, immutable pins,
qualification outputs, `observations.jsonl`, `summary.json`, and all Callgrind
traces, logs, outputs and counts in `instructions.json`. Build resource receipt
`023529acfc824e5cb983435b07884dba` and exclusive receipt
`0d73754bb898424caf5f914c537015c1` both complete successfully. Qualification and
timing finished on their first attempt; there were no failed performance
observations to exclude. Functional implementation failures are separately
recorded in the static checkpoint. No benchmark rerun is requested.
