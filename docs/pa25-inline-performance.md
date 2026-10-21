# Closed inline constant performance checkpoint

This bounded immutable study compares the scheduler-only checkpoint `1873a93c`
with closed inline execution `11003689`. It is not a full release matrix.
Compiler source SHA-256 is
`c0e1f539bb5d4fb751220077149b9cd87b702e80e037dfc1604891d200b671e0`.
The baseline executable SHA-256 is
`b8484baba83a1e788c61b328afc43679b4dbc0d9dd5f138ad5d3dd9e34be3f6f`;
the candidate is
`9996808ba86b87c5628284e777719d8c6adf2f8964354532761cb6ed53fa8ebe`.
Pins and input hashes were verified before and after measurement. All common
`--emit-constants` outputs matched byte for byte, and every observation retained
its output hash, exit status and stderr count.

The exclusive controller wrapper exited zero. Four A/A calibration blocks and
five paired ABBA blocks used three repetitions per position. Candidate-only
families used corresponding B/B and BBBB blocks. All 427 observations, including
warmups, remain in
`obj/parallel/pa25-inline-const/performance/observations.jsonl`; inputs, receipts,
pins, summary and actual work counts remain in that evidence directory. No
observations were discarded or repeated for favorable results.

| Common control | Median paired B/A wall ratio | A/A ratio range | Median RSS A / B (KiB) |
| --- | --- | --- | --- |
| 512 ordinary functions | 1.0642 | 0.8709–0.9972 | 11336 / 11344 |
| 64 scalar const calls | 1.0139 | 0.9070–1.2611 | 9984 / 9958 |

Individual paired ratios span 0.9268–1.1468 and 0.9305–1.0445 respectively.
These millisecond process measurements have substantial calibration noise, so
neither median establishes a throughput regression or improvement. The resource
lease excludes other registered RustGM jobs, not unrelated host workloads.
The earlier unexplained scalar RSS increase of approximately 594 KiB remains an
unresolved observation from its own checkpoint; the different scalar workload
and baseline here do not explain or erase it.

| Closed inline bodies | Median wall (ms) | Median RSS (KiB) | Registry logical bytes | Retained body logical bytes | Code logical bytes |
| --- | --- | --- | --- | --- | --- |
| 16 | 4.406 | 9246 | 4864 | 46080 | 7808 |
| 64 | 6.238 | 9996 | 19456 | 184320 | 31232 |
| 256 | 12.553 | 11402 | 77824 | 737280 | 124928 |

For each width N, actual source index appends, type publications, preparation
visits, checked inline CFG constructions, value publications and started frames
all equal N. Registry lookup probes equal 15N and interpreter steps equal 5N.
Logical registry/body/code charges equal 304N/2880N/488N. The ordinary control
creates no inline registry or inline retained body; the const-call control also
has zero inline counts. These are observed graph/index work and logical storage
invariants, not claims that all scheduling, sorting, allocator bytes or process
RSS are linear in every source family. Existing unit scaling separately covers
8/32/128 roots and early/late checked-CFG reuse.
