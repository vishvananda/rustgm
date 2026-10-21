# Generic closure specialization measurements

The development comparison isolates compiler commit `c0eab657` (A) against
`59fcb3e4` (B), before the later crate/global/atomic integration. Both are immutable
release binaries. A's SHA-256 is
`7d6236c080a2cc2e8b7a596c66221f65c24a8dd4c040c44834ef73bc31ee091e`;
B's is `0cc5e9d885671fc5433ee11908420cda8695f1fb46a0da4f7c85e20c8d3910ac`.

All 15 frozen sources passed independent Rust 1.98.0 object generation before
measurement. The matrix uses widths 8, 32 and 128 for ordinary functions,
contextual closures, generic function controls, generic closures and projected
generic closures. Generic inputs request each const-argument specialization
three times. Nine shared inputs produce byte-identical LowIR with A and B. The
six newly supported inputs retain A's diagnosed rejection and B's deterministic
successful output; their timings are not a speedup comparison.

The fixed plan retained 390 observations with three executions per observation,
120 separate peak-RSS checks and 24 Callgrind instruction counts. Shared inputs
use two warmups, four A/A calibration observations and five ABBA blocks. New
inputs use the same observation count with B. Every successful execution checks
its full output, and final checks verify binary and source hashes. One CPU is
selected throughout. The exclusive RustGM resource lease excludes registered
builds, but unrelated projects were active on the host.

| Input family | B/A instructions at 8 / 32 / 128 | B RSS at 128 | B time at 128 |
| --- | --- | --- | --- |
| Ordinary functions | −0.011% / −0.054% / −0.066% | 10,222 KiB | 7.82 ms |
| Contextual closures | +1.994% / +2.205% / +2.463% | 11,694 KiB | 19.50 ms |
| Generic controls | +0.245% / +0.292% / +0.290% | 10,326 KiB | 10.50 ms |
| Generic closures | new capability | 11,012 KiB | 14.40 ms |
| Projected generic closures | new capability | 11,782 KiB | 39.64 ms |

Generic closure instruction counts are 4.44, 14.76 and 55.60 million; projected
closure counts are 10.78, 36.50 and 139.95 million. Fourfold input increases
therefore produce approximately 3.3–3.8-fold instruction increases across this
matrix. These observations support roughly linear growth at the measured widths;
they do not establish an asymptotic bound for arbitrary programs. Exact instance
reuse and recursive-growth limits also have separate typed maintainer tests.

Shared-input RSS increases range from 518 to 866 KiB. B's executable text section
grows by 55,792 bytes; this does not by itself explain the RSS increase. A/A wall
spreads range from 1.90% to 42.24% on shared inputs, and reach 64.93% on one new
input. The apparent shared-input wall improvements are not a reliable speedup or
a tight regression bound. Instruction counts, memory and output equality are
reported separately; no five-percent wall-time neutrality claim is made.

All observations, qualification diagnostics, hashes, profiles and the fixed plan
are retained under `obj/parallel/closure-instance-performance/measurement-v1/`.
The driver is `obj/parallel/measure_generic_closures.py`; the wrapped run exited
successfully and removed its disposable qualification object. This study is a
development checkpoint, not PA24 release qualification.
