# PA12 validation and aggregate handoff

PA12 publishes the concrete aggregate profile in [its handout](../pa12/README.md).
It joins PA1–PA11 in the active course report. PA13–PA15 remain the next
implementation objective; this checkpoint does not claim pattern, lifetime,
loan, destruction, generic or full Rust-language conformance.

## Architecture and feature ownership

The aggregate implementation uses canonical nominal identities, flat member
arenas and selected expression facts. Cached iterative layout queries consume
completed definitions and typed discriminant values. Body IR preserves places,
snapshots, index evaluation, struct-update field reads and zero-sized effects.
Executable lowering requires the owned-value proof and separately checks the
representation of every demanded value. It uses the existing CPPGM object
storage, transfers, pointer arithmetic and abort convention without changing
backend source, binaries or lock. The implementation history and earlier
measurements are retained in [the buildout record](pa12-buildout.md).

The layout inspection mode consumes these same cached facts. It completes all
queries before writing the canonical type, named definition, layout, variant
and offset tables. Count mode uses the same completed queries. No subsequent
frontend phase parses these tables. Unsized types remain explicit; an unsupported
sized representation fails without partial output. The CLI releases the body,
declaration and resolution arenas before writing LowIR.

The PA6 resolver now retains the concrete nominal `Self` target in its owning
scope, respecting nested-item capture barriers without adding synthetic
definitions or renumbering earlier outputs. Two PA6 fixtures own those name
rules; one PA8 fixture owns the macro interaction. PA12 fixtures exercise the
combined discriminant/type/layout behavior. Existing committed oracle bytes
for PA1–PA11 are unchanged; their source receipts are regenerated normally.

General constant functions, statements, loops, allocations and nonliteral array
lengths remain PA25 work. The supported scalar discriminant evaluator checks
ordinary type facts, cached dependencies and overflow, including `Self` paths.
General trait selection and procedural derives remain later assignments; the
concrete built-in Clone/Copy capability is explicit. References cannot be hidden
in aggregate bytes, including zero-length arrays, to evade the pending proofs.

## Declared tests and independent checks

The 110 PA12 fixtures declare outcomes before reference generation:

- 40 successful native programs cover aggregate construction, snapshots,
  direct/indirect calls and results, field/update evaluation order, checked
  indexing, repeats, packed/aligned fields, wide scalars, enum tags, configured
  and macro-generated items, external modules and zero-sized values.
- Eight native panic cases cover out-of-bounds reads, stores, nested arrays,
  wildcard place formation, zero-sized elements, empty arrays and huge lengths.
- 12 layout cases declare size/alignment and selected field offsets. The
  candidate and committed text must match those vectors. Independent rustc
  probes check guarantees for C, packed, aligned, transparent, arrays and enum
  representations. Course Rust source order and transparent ancillary ZST
  offsets are explicit exceptions; they are not rustc layout-equivalence claims.
- 43 invalid-source cases and seven explicit later-stage boundaries cover
  fields, constructors, visibility, types, storage cycles, representation
  conflicts, discriminants and unsupported obligations. Rustc acceptance is
  recorded separately from the course boundary outcome.

Every accepted LowIR case must declare a runtime outcome. The native runner
checks all 98 LowIR outcomes and executes all 48 accepted programs using both
rustc and the pinned CPPGM backend. Clean execution requires exit zero and empty
streams. Expected panic requires CPPGM exit 134 with empty streams and rustc
SIGABRT with a panic diagnostic. Layout inspection never substitutes for native
execution. The runner also requalifies all 194 PA11 cases / 180 executions.

The layout checker validates all 12 cases independently. CLI invariants cover
determinism, count/full agreement, recursive pointer shapes, huge ZST arrays,
reference footprints and failure without partial output. The manifest and
assignment must agree on the supplementary mode; undeclared modes fail.
The independent Rust checks preserve cfg arguments and the effective explicit
crate name. LowIR programs are checked as binaries, including entry validation;
type/layout inspection sources are checked as libraries.

The full course report contains 1,665 fixtures. Debug and release each pass
243 Rust tests. Strict Clippy/rustfmt and the audit of 162 Rust files pass with
zero findings. Two audit tests and 24 harness tests pass. Upstream verification
retains 24 pinned inputs and 126 Reference files. All earlier rustc checks and
the unchanged backend bridge remain required.

The release candidate is frozen at
`obj/pa12-work/publication-candidate`, SHA-256
`33f090e9361bd752b3b9240187bb398c6428bee10d2939f3c588d2706f865ff4`.
Local evidence is retained in `obj/pa12-work/publication-*.log`,
`obj/native/pa11/evidence.json`, `obj/native/pa12/evidence.json` and
`obj/layouts/pa12/evidence.json`. Receipts record candidate/manifest hashes;
native receipts include the locked backend and per-case commands and outcomes.
Hosted CI checks the text oracles, invariants, audits, rustc acceptance and
layout probes. Native qualification remains a separate local requirement
because hosted CI does not have the qualified C++ binaries.

## Performance qualification

Performance uses immutable candidate/baseline snapshots, frozen source hashes,
complete output equality, two warmups, four A/A samples and three ABBA blocks
on CPU 10. Each run retains all 54 observations. No RustGM build or test runs
concurrently. The prior published discriminant binary is the compatible
baseline; layout inspection receives its own initial A/A scaling workload.

At 4k/16k/64k functions, name-resolution baseline/candidate medians are
55.28/54.20, 216.15/218.11 and 915.78/925.58 ms. Largest-scale RSS is
138,356/139,372 KiB. The new nominal scope fact costs about 1 MiB at that
scale, with about 1.1% measured elapsed overhead. Full outputs and counts agree.
All observations remain in `obj/perf/1791825295845013542/samples.json`.

Scalar LowIR medians at the same scales are 211.85/210.33, 861.58/861.06 and
3606.27/3582.20 ms; largest-scale RSS is 698,380/700,844 KiB. Complete output
bytes and counts agree. These observations do not establish a speedup; they
show no material elapsed regression and a roughly 0.35% RSS increase. All
observations remain in `obj/perf/1791825321310728114/samples.json`.

The initial layout workload has 1k/4k/16k C records and fixed-array aliases.
End-to-end A/A medians are 22.90, 89.58 and 399.73 ms, with peak RSS
8572, 23,348 and 83,184 KiB. All observations and full-output checks remain
in `obj/perf/1791825420695763288/samples.json`. The preceding compiler has no
layout inspection mode, so this is scaling evidence rather than an A/B claim.
`make perf-layouts` reproduces this workload.
