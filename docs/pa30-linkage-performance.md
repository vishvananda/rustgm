# PA30 exact function linkage performance

This fixed study compares clean shared-root baseline
`f1fec3035466a1101f768aef27da816973bbdc4f` with linkage checkpoint
`001022afd38763e824cb49bdd6aa9162e5668c8b`. Both include the `7e80bff0`
lifetime correction. The comparison isolates retained function attributes and
the canonical LowIR function-name index, before constructor-promotion changes.

Both immutable binaries use Rust 1.98.0 release optimization level 3, debug
information level 1, default repository flags, two Cargo jobs and disabled
incremental compilation. SHA-256 values are:

- Baseline: `98948339adc76e7bf031074f1ed7f26b81ad6d9f289e7198e9b1737248a0566c`.
- Candidate: `a2379037fbb0c97586aa608e8ff81280b6b553664f21b36027252eb0d56f101f`.

The plan was recorded before exclusive admission. Six frozen inputs received
independent Rust 1.98.0 object qualification: scalar statements and macro-built
functions at 64/256 elements shared by both versions, and candidate-only exact
exports with matching imported self-aliases at 64/256 elements. Each shared
control has one warmup per binary, two A/A pairs and three ABBA blocks, with
three executions per observation. Candidate-only controls have one warmup and
three observations. All 80 observations (240 executions), statuses, stderr and
output hashes remain available. Shared outputs agree byte for byte.

| Shared control | Paired wall B/A median | Paired RSS B/A median | Callgrind instructions A | Instructions B | Instruction B/A |
| --- | ---: | ---: | ---: | ---: | ---: |
| Scalar 64 | 1.02499 | 0.99873 | 4,775,986 | 4,766,494 | 0.99801 |
| Macro 64 | 1.02473 | 1.01263 | 15,700,531 | 15,788,323 | 1.00559 |
| Scalar 256 | 0.97112 | 0.99725 | 17,100,676 | 17,084,623 | 0.99906 |
| Macro 256 | 0.98374 | 0.99965 | 61,654,461 | 62,018,232 | 1.00590 |

Exactly one matched Callgrind observation was collected per shared input and
binary, with complete profiles/logs and output equality checks. There were no
follow-up repetitions. These simulated instruction counts show about 0.56–0.59%
extra work on the macro/function controls; scalar changes are below 0.2%.
A/A wall ratios range from 0.91598 to 1.04420. These short timings do not support
a tight wall-time threshold claim. RSS ratios are close to one, with Macro 64
about 1.26% higher in its paired median.

Candidate-only exact export/self-alias median wall time rises from 9.385 ms at
64 elements to 24.546 ms at 256. Median RSS rises from 10,020 to 12,000 KiB.
These are scaling observations, not a comparison against an implementation
without the feature.

Evidence is under `obj/parallel/pa30/linkage-performance/`: `study-plan.json`,
`build.json`, `plan.json`, `observations.jsonl`, `summary.json`, `instructions.json`,
Callgrind profiles/logs, input/output files and immutable pins. Build receipt
`d9a3635202794f848c68774ce40a562c` records matching profiles and target cleanup;
exclusive receipt `eba076434f8d43e999147f5ee90e0e90` records the successful study.
The first qualification attempt stopped before any observations because the
input generator added a closing parenthesis to `link_name`. Its generator,
inputs, diagnostics and failed receipt `2838f42c5ef241ea8dcd09865f695c6c` are
retained separately. Correcting that generated input did not change either
compiler binary or the fixed observation plan.

This is a bounded checkpoint measurement. Full PA30 release/performance
qualification remains outstanding.
