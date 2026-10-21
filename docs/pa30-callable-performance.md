# PA30 callable checkpoint performance

This bounded check compares production checkpoint `6003e275` with `d60b8d52`.
Both immutable compilers use the repository's release profile (`opt-level=3`,
`debug=1`) and Rust 1.98.0. No RUSTFLAGS, encoded Rust flags, toolchain override
or release-profile environment override was set. The shared resource controller
forced two build jobs and disabled incremental compilation. Measurement ran on
CPU zero under an exclusive lease.

Baseline SHA-256:
`584e421a01ccaef3c81f9e37beeb9b2f7810505d1b4d999cc447c49643fe2f47`.
Candidate SHA-256:
`3107611356b7f75670897ae976406046a04de316b0967c4918e0303431937b8c`.

The predetermined inputs repeat scalar assignments, Rust function-pointer
coercions/calls and contextual reference closures at 64 and 256 occurrences.
Four candidate-only controls repeat C function-pointer calls and safe-to-unsafe
C pointer coercions at the same sizes. All ten inputs were independently
qualified by pinned rustc object generation before measurement. Shared LowIR,
stderr and exit status agree exactly for every observation.

Each shared input has warmups, two A/A baseline pairs and three ABBA blocks,
with three compiler executions per observation. All 124 observations (372
compiler executions) retain wall time, user/system CPU time, peak RSS, status
and output hash. These are short millisecond inputs: A/A wall ratios range from
0.717 to 1.049, and one scalar-256 ABBA block has ratio 1.165. They do not support
a tight percentage regression claim.

| Shared control | Count | Median B/A wall | Median B/A RSS |
| --- | ---: | ---: | ---: |
| Scalar | 64 | 1.0063 | 0.9953 |
| Rust pointer | 64 | 1.0036 | 1.0100 |
| Reference closure | 64 | 0.9995 | 1.0170 |
| Scalar | 256 | 1.0129 | 0.9998 |
| Rust pointer | 256 | 0.9972 | 1.0092 |
| Reference closure | 256 | 0.9904 | 1.0032 |

Candidate C pointer controls scale from 4.15 to 8.72 ms and 9,136 to 10,244 KiB
peak RSS. The safe-to-unsafe C controls scale from 4.52 to 9.61 ms and 9,584 to
10,268 KiB. These observations describe bounded controls, not a release-wide
performance guarantee.

The immutable pins, frozen input hashes, build log, exact plan, every raw
observation and block summaries remain under
`obj/parallel/pa30/performance-d60/`. Resource receipt
`fec3fa23d8244367928636110a5282a9` returned zero. An earlier queued attempt was
interrupted with status 130 before admission to correct the proposed candidate
profile from debug to release; it produced no build or measurement observations.
