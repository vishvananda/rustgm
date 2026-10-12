# PA11 validation and executable handoff

PA11 implements the scalar executable profile in [its handout](../pa11/README.md).
The maintained frontend now compiles Rust source into typed LowIR that the
unchanged qualified CPPGM backend validates and executes. PA12–PA38 remain
planned; this is not a claim of aggregate, ownership, generic, library or full
Rust-language conformance.

## Architecture and scope

`body::scalar_check` verifies definite initialization and immutable-write
permission on a monotone CFG worklist. Its `Checked` result is required by
`lower::build`. This is a scalar proof, not a Copy/move/region proof. The body
model remains available at PA10 without prematurely claiming those obligations
are discharged. PA14 generalizes initialization to projections and partial moves.

`sema::entry` selects the resolved root main binding. `lower::Context` caches
native signatures by canonical type identity and emits demanded procedures by
DefId through an iterative queue. Per-function reverse postorder keeps value
snapshots before their uses; locals, arguments, temporaries and literals use
flat owned arenas. Call scratch is reused. Function-item values remain zero
sized until demand/reification. No phase repeats parsing, lookup, inference or
literal decoding. `lower`'s new audit dependencies on `resolve` and `literal`
are for already selected identities and decoded float bits.

Checked arithmetic includes 128-bit operations, mixed-width shifts and signed
minimum cases. Saturating float conversions branch before unsafe backend
conversions. A demanded helper per float width implements remainder with exact
integer significand reduction. Untyped local storage handles reverse bit casts
because the pinned backend does not select integer-to-float register copies.
No C++ overflow, alias or typed-memory promises are fabricated.

The external bridge still consumes LowIR only. The pinned `lowir2native`
already links its course runtime for 128-bit arithmetic/conversions and abort;
no backend/compiler modifications or lock updates were needed. Abort is exit
134 in that runtime, while pinned rustc aborts with SIGABRT. The harness checks
these explicitly rather than accepting arbitrary failures as matching panics.

Execution exposed a PA10 handoff correction: wildcard local bindings must not
read their place. `let _ = local` now emits no read; calls and block-valued RHS
expressions retain their effects. A new PA10 fixture and typed CFG/rustc vector
cover uninitialized scalar/unit locals discarded this way. Only the existing
PA10 shadowing oracle changes (one removed read and renumbered values); earlier
PA1–PA9 oracle bytes are unchanged. Reference receipts are refreshed normally.

## Independent and regression evidence

- PA11 has 194 declared fixtures: 180 executable comparisons (88 successful
  programs, 92 expected runtime panics), 11 Rust rejections, and three explicit
  phase boundaries for constants, statics and reference-bearing signatures.
- Full-value assertions cover all integer widths, isize/usize, signedness,
  overflow, mixed shifts, casts, bool/char, f32/f64 arithmetic, saturation,
  signed zero, NaNs, infinities and subnormals. The fixed remainder vectors
  cover another 128 finite operand pairs independently evaluated by rustc.
- Control-flow cases preserve PA10's declared results. Native tests cover
  recursion, direct/indirect calls, stack arguments, zero-sized arguments,
  imported/generated/never main, modules, includes and cfg selection.
- `make test-report`: 1,547 PA1–PA11 fixtures pass. Debug and release Rust suites
  each pass 213 tests. New invariants cover builder state, typed comparison
  results, block placement, demand, initialization joins, count/full-view
  agreement and 10,000 additions without recursive lowering.
- `make lint audit-rust test-harness`: strict Clippy/rustfmt pass; 140 Rust files
  have zero audit findings. Two audit tests and 19 harness tests pass. Native
  harness tests reject missing execution classifications and spurious failures.
- All earlier independent rustc checks pass, including the 48 PA10 fixtures.
  `make test-upstream` verifies 24 imported inputs and 126 Reference files.
  `make test-backend` passes using the unchanged lock.
- `make ref-test` regenerates every sidecar/receipt through declared outcome
  checks. Text grading and native execution remain separate, mandatory evidence.

Evidence is retained in `obj/pa11-work/` and `obj/native/pa11/evidence.json`.
The final native candidate is frozen with SHA-256
`6e991781fe1f8d134ab1f7791f2617314bb49d30e337c001916beee009b66f1d`.
The native receipt records candidate/manifest hashes, backend lock, compiler
version, commands and outcomes; raw stderr/stdout and validated LowIR are kept
per fixture. Ordinary hosted CI has no qualified CPPGM binary, so it runs the
frontend/text/invariant checks. Local native qualification is explicit and is
not represented as a hosted CI result.

## Performance

Measurements use frozen binaries and sources, full-output equality, two warmups,
four A/A observations and three ABBA blocks, pinned to CPU 10. No RustGM build
or test runs concurrently with measurements. Raw observations retain host noise.
The earlier body-stage release provides the unchanged-phase baseline; the new
LowIR stage receives its own initial scaling calibration. Full native backend
time is outside this frontend measurement.

The body-stage comparison is
`obj/perf/1791807690273694241/samples.json`. Full body text matches at every
scale. A/B medians at 4k/16k/64k procedures are 147.00/154.98,
675.65/666.03 and 2,776.58/2,801.13 ms. Candidate peaks are
36,692/137,876/542,752 KiB versus 36,648/137,836/542,680 KiB. The timing
differences lie within observed calibration/paired variation and do not show
a material regression or a speedup.

The initial demanded-LowIR calibration is
`obj/perf/1791807764105128481/samples.json`. Each measured procedure is called
from Rust main, so this measures actual lowering rather than undemanded bodies.
At 4k/16k/64k procedures, median times are 193.43/757.57/3,215.16 ms and
peak RSS is 46,200/175,624/693,480 KiB. Counts and complete serialized output
match between immutable snapshots at every scale. This establishes the first
LowIR-stage baseline; `make perf-lowir` reproduces the workload. It is not a
comparison with a different compiler or an executable-backend benchmark.
