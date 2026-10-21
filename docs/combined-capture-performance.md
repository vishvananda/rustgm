# Combined capture and nested-output performance

This development study measures compiler snapshot `f8939df8`, combining the
sparse loan-expiration index, nested capture propagation, complete original-body
output interfaces, written inline lifetimes, unsafe capture precision, canonical
operator method lookup and AtomicU32 compare-exchange. It is not a PA24–PA28
assignment release. Later test/oracle-only changes retain this compiler snapshot.

The immutable baseline is the public captured-execution compiler `68d3a32e`,
SHA256 `76a94721c579c2e008d0c6b1d3b873c0fb96058d975292cc608c709781c3bff2`.
The candidate is
`a32e0b10fa373fb9f2b240fbefe38f6b8983697b42cd6b54c29c120f7f71d489`.
Its independent source/native and lint/audit preflight is recorded in the
validation worktree's `obj/parallel/nested-integration/preflight-complete.json`.
Pinned executable provenance, declarations, failed preflights, raw observations
and completed results are retained under `obj/parallel/combined-performance/`.

Thirty-one input slots were declared before timing. Twenty-one retain exact
baseline/candidate output bytes. Three atomic RMW inputs now use different LowIR
symbol identities following canonical compare-exchange schema registration;
their original differing outputs remain recorded, and only candidate measurements
are reported. Six new packed-capture/output-interface inputs initially lacked
the CLI executable mode's required `main`; the original valid Rust library inputs
and both compilers' missing-entry diagnostics remain unchanged. Explicit entry
functions were declared and independently checked before timing the replacement
inputs. They have no accepted baseline implementation and are candidate-only.

The remaining live-borrows input at width 512 is independently valid Rust but
hits the existing value-liveness seed budget in both compilers. It remains an
unsupported input, not a passing performance observation. No timing was discarded
or repeated after inspecting its result.

The protocol uses pinned rustc 1.98.0 object qualification, immutable binaries,
frozen source hashes, CPU 2, two warmups, four A/A calibration observations and
five ABBA blocks with three executions per observation. Candidate-only inputs
use B/B calibration and B observations. Every timed output must match its own
frozen expected bytes. RSS samples and Callgrind instruction counts supplement
wall time. An exclusive resource lease excludes other RustGM jobs; unrelated
host load remains possible.

| Shared input | Instruction change | Paired wall median | A/A spread |
| --- | ---: | ---: | ---: |
| Ordinary, 8 | +0.42% | -3.24% | 2.64% |
| Ordinary, 32 | +0.57% | +0.98% | 3.49% |
| Ordinary, 128 | +0.56% | +4.65% | 2.46% |
| Adjacent captures, 128 | +0.73% | +3.80% | 1.65% |
| Precise captures, 32 | -8.39% | -4.55% | 17.20% |
| Precise captures, 128 | -20.40% | -12.08% | 5.81% |
| Precise captures, 512 | -29.77% | -26.28% | 3.44% |
| Live borrows, 32 | -23.96% | -10.42% | 3.21% |
| Live borrows, 128 | -30.15% | -26.39% | 1.89% |
| Shared reborrows, 512 | +0.06% | +0.00% | 1.21% |

The large expiration-heavy families retain substantial work/time reductions.
Ordinary and adjacent-capture inputs at width 128 show wall-time increases beyond
their calibration spread; this is not a claim of neutral ordinary-code performance.
Other shared families' instruction increases are at most 1.11%. Shared median
RSS differences range from -724 to +408 KiB. The separate isolated-expiration
study remains recorded in `docs/loan-expiration-index.md`; comparing wall times
across the two studies would not isolate a compiler change.

| Candidate-only family | Widths | Instructions (millions) | Median wall time (ms) |
| --- | --- | --- | --- |
| Packed captures | 8 / 32 / 128 | 2.55 / 5.79 / 18.69 | 3.60 / 4.44 / 6.55 |
| Output interfaces | 8 / 32 / 128 | 7.85 / 29.57 / 116.67 | 4.79 / 9.60 / 27.02 |
| Atomic RMW | 8 / 32 / 128 | 5.91 / 18.82 / 70.68 | 4.92 / 7.08 / 16.16 |

The new input families' work grows approximately linearly over these sizes,
consistent with their focused structural work invariants. This does not establish
whole-frontend linear complexity: seed collection and live access/conflict work
remain separate costs, and generic output specialization and receiver adapters
remain unfinished language work.
