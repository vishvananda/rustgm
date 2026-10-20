# PA22 projected storage proofs

The [PA22 release](pa22-release.md) supersedes the implementation-status notes
below. This document preserves its checkpoint evidence.

PA21 is released. PA22 remains planned: this checkpoint completes additional
projected-tail coherence and proof-sharing work, not the raw-pointer contract
or the PA22 release.

## Correctness

Coherence excludes a `Sized` blanket implementation only when the overlapping
header equations establish an unsized storage tail. It follows concrete,
structural and normalized projected tails. An unresolved generic projection
does not establish disjointness. Each header keeps distinct lifetime parameters
and its own lifetime premises; the selected normalization must satisfy those
premises before it can justify disjointness.

Fifteen independently rustc-qualified source cases cover sized/unsized controls,
generic concrete tails, open projections, nested tails, lifetime bounds and
`Copy` blankets. Three new native programs select the intended implementation
beside a `Sized` blanket, including a lifetime-dependent projected tail.

That native lifetime case exposed a hidden `Sized` selection which started a
separate projection proof with the wrong allocation frontier. Hidden conditions
now descend on the existing iterative selection stack. Their child evidence,
scope and private allocation range accompany the parent proof. The new evidence
invariant checks both direct and blanket selection at two caller frontiers,
including a conditional `'static` requirement which the caller cannot satisfy.

## Repeated proof work

The first profile showed repeated selection of the same concrete suffixes in
nominal formation, storage-shape normalization, body bounds and unsizing.
Sharing only one physical unsizing walk did not address these repeated semantic
queries.

Completed concrete subproofs now survive related selection queries within the
same immutable caller and mode. Each proof node gets a completion flag from
its own evidence and its children. Reuse excludes witnesses, lifetime debts,
retained `Sized` requirements, provisional declarations, assumption-dependent
branches and nested proof scopes. Negative searches and cycle cutoffs are not
exported. The source selection remains available for subsequent associated
definition lookup.

Nominal checking and projection cursors retain these facts across roots.
Validated body checking also retains eligible facts across separate normalization
sessions and canonical trait checks. Each session still publishes its own
lifetime evidence; normalized type identity alone supplies no permission.

The work invariant covers 16, 32, 64 and 128 nested projected wrappers. Initial
proof work is bounded by the number of wrappers, and repeated roots use bounded
lookup work. Separate invariants ensure retained builtin requirements and
conditional lifetime evidence survive repeated selection.

## Development measurements

The focused comparison used immutable binaries, five frozen independently
rustc-qualified inputs, A/A calibration, five paired ABBA blocks, three executions
per observation, eight RSS observations per input and Callgrind. Every emitted
output matched byte-for-byte.

The baseline is the coherence/lifetime fix before proof sharing. The intermediate
implementation retained nominal/cursor proofs; the final candidate additionally
retains eligible body proofs. All measurements remain in:

- `obj/pa22-work/coherence-perf-v1/`
- `obj/pa22-work/proof-reuse-perf-v1/`
- `obj/pa22-work/proof-reuse-perf-v2/`

| Projected depth | Final median ms | Baseline instructions | Final instructions |
| --- | ---: | ---: | ---: |
| 4 | 5.61 | 10,788,085 | 7,739,760 |
| 16 | 9.08 | 114,616,186 | 27,239,954 |
| 32 | 14.24 | 437,585,233 | 53,971,825 |

At depth 32 the median paired wall-time reduction was 82.34%; instructions fell
87.67%. The final instruction count approximately doubles from depth 16 to 32.
This supports the bounded-work improvement over this range, not a universal
complexity claim for arbitrary Rust trait programs.

The ordinary control changed by -0.072% in instructions and the structural
64-wrapper control by +0.278%. Wall-time observations include noise and are
retained individually. No general speedup is inferred from those controls.

## Qualification

Focused validation is complete at `obj/pa22-work/proof-reuse-v3/`: formatting,
strict release Clippy, architecture audit, work/evidence invariants, affected
source/native controls and all 7,040 PA1–PA21 byte cases passed.

The full 49-gate checkpoint passed at
`obj/pa22-work/proof-checkpoint/validation-v1/`, using candidate SHA-256
`1a0b00f09ce414dfff8aa64463135dc4b7daaaeee336de8fc603d45693eda70a`.

| Check | Result |
| --- | --- |
| Release maintainer tests | 1,760 passed |
| Explicit native maintainer groups | 108 passed |
| PA1–PA21 public byte oracles | 7,040 passed, no oracle changes |
| PA12/13/16/17/18/19/20/21 independent source checks | 5,087 passed |
| Affected independent native checks | 3,346 passed; 1,978 execution comparisons and three codegen-only comparisons |
| Strict Clippy, formatting and architecture audit | Passed; 1,049 Rust files, zero findings |
| Audit tests, harness, discovery, upstream, backend and layout checks | Passed |

These independent suites preserve the previously declared acceptance differences
and PA13's sixteen documented runtime differences. They are not a claim of
unqualified rustc parity.

The broad 16-input matrix at `obj/pa22-work/proof-checkpoint/performance-v1/`
compares the published storage checkpoint with this exact qualified candidate.
All outputs match. Projected depths 4/16/32 take 4.89/8.71/13.29 ms; depth 32
uses 87.65% fewer instructions with an 82.86% median paired wall-time reduction.
The thirteen other inputs change by -0.044% to +0.435% in instructions.
One small binder-depth control measured 9.39% slower in wall time (3.61 ms),
despite a 0.435% instruction increase. The predeclared follow-up used eight ABBA
blocks and fifteen executions per observation, with ordinary-equality and
shared-input controls. It did not reproduce a consistent binder slowdown:
the median paired difference was -0.52%, with both positive and negative blocks.
The ordinary-equality and shared-input differences were -0.73% and -3.43%;
their A/A spreads were 9.50% and 72.08%, respectively. These noisy observations
support no general wall-time speedup claim. Both runs remain intact at
`obj/pa22-work/proof-checkpoint/performance{,-followup}-v1/`.

The redundant `dev/target` Cargo cache was removed, recovering about 14 GiB.
Build and validation commands retain the storage guard and use `obj/rust`.
Immutable comparison pins remain protected until their replacement is published.
The full qualification guard removed another 7,245 generated files, about
7.87 GiB, while preserving inputs, diagnostics and measurement receipts.
