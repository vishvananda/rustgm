# Sequence equality, record metadata and nested C aggregates

Compiler cutoff `39f6fac2985f4c32ae9480cb0954d0808305ce56` implements
array/slice equality, closed named records across crates, and recursively
classified C aggregate fields. The feature contracts and remaining boundaries
are in [sequence equality](pa26-sequence-equality.md),
[record metadata](pa29-closed-records.md) and
[nested C aggregates](pa30-composite-abi.md). PA25–PA30 remain planned and
incomplete; this publication does not activate an assignment.

## Qualification and references

The combined candidate passed 2,452 regular release maintainer functions and
246 explicitly selected native functions, strict all-target/all-feature release
Clippy, formatting, the architecture/file audit (1,616 files, zero findings),
90 harness tests, grouped registration, layout and upstream verification.
Default and all-feature production configurations reuse the identical executable.
Focused source/native checks for the three features remain recorded in their
owning documents and are included in the combined maintainer qualification.

Of 8,360 active PA1–24 text cases, 8,358 complete triplets matched exactly.
The two unchanged PA21 `hidden-equality.rs` sources still reject at byte 48:
an opaque result exposing only Copy does not provide PartialEq. Their stderr
now reports the ordinary trait obligation instead of the former scalar-only
operation boundary. Acceptance, stdout and exit status are unchanged.

Before generating those two sidecars, the final candidate passed PA21's 748
independent Rust checks and native suite (450 checks, including 264 executable
comparisons). Both pinned backend handshakes passed. Scoped generation and
review preserve all other 2,242 PA21 sidecars and add one provenance batch.
The installed tree passes all 8,360 active text cases exactly, the 90-test
harness, grouped registration, layout and upstream checks. No other references
or manifests change.

Candidate SHA-256:
`f46b8699a71cfb949e217f286f0acf1fc60b4a4af959d5c51dc8361a90667a6d`.
Combined qualification is retained in
`rustgm-validation-work/obj/parallel/records-composites-checkpoint-v1/`;
publication checks and reviewed installation are in
`obj/parallel/records-composites-publication-v1/`. Combined cleanup reclaimed
13.73 GiB and losslessly archived one large diagnostic. Earlier failed batches
and their diagnostics remain retained.

## Performance

The baseline is the previous public candidate with SHA-256
`e7ff3ee3c7d6882d83e646cc24e31a208fb1d1639052147fe94a94388d1df615`.
All 48 frozen source hypotheses were independently qualified. Eleven inputs
succeed under both binaries, with byte-identical stdout and, where requested,
metadata. The other 37 exercise newly supported behavior and carry no speedup
claim. Each shared input retains A/A calibration, five ABBA blocks with three
executions per observation, eight RSS samples and both Callgrind profiles.
The exclusive lease excludes registered RustGM jobs, not unrelated host work.

Shared instruction counts increase by 0.257%–3.300%. No shared input crosses
the declared 5% instruction threshold or the combined 512 KiB/5% RSS threshold;
the largest shared median RSS increase is 466 KiB. The ordinary512 control is
43,009,528 baseline instructions and 43,119,996 candidate instructions (+0.257%).
Its candidate median is 12.01 ms, paired wall change -2.73%, with 2.84% A/A
spread. Other timings are noisier; these observations do not establish a
general speedup or tight wall-time neutrality.

Thirteen independent 8/32/128 series cover scalar equality/export, sequence
calls/length/unrelated declarations, C aggregate depth/roots/unrelated
declarations/array length, and record import/export function/record counts.
No series crosses the declared scaling trigger. Positive incremental
instruction ratios range from 3.94 to 4.14 per fourfold input step. Sequence
length costs 6,921,462 / 6,923,451 / 6,923,160 instructions; large C array lengths
cost 987,976 / 988,613 / 988,570. Those near-constant axes support the intended
shared body/element algorithms; they are not whole-compiler complexity claims.
The previously attributed generic-argument cleanup quadratic cost remains a
separate required follow-up in `pa25-associated-call-checkpoint.md`.

Frozen plans, every observation and analysis are in
`obj/parallel/records-composites-performance-v2/`. Earlier preparation failures
are retained: an unsupported panic macro in import observers was replaced by
a separately qualified bounds-check observer, and one missing receipt row was
reconstructed from its exact preserved successful observation. Neither changes
compiler results or selects favorable timings.

Results SHA-256:
`81cf35dca31109d5313ae3b93478b0b9ec385b54edc2da546b79a6433f27ee6d`.
Plan SHA-256:
`72ac1f8999b91eb556bc6250b30fdae69f61ad9b2a98cc42d6c08c2dbbfebe2f`.
