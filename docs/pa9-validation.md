# PA9 release validation

PA9 is an implemented draft of the scalar type-checking profile in
[the handout](../pa9/README.md). It is not whole-language acceptance. The earlier
[implementation checkpoint](pa9-progress.md) is historical.

The measurements and 198-case/204-test totals below record the `675671b`
release. PA10 subsequently added the grouped-binding regression (199 PA9 cases)
and an explicit destination type for region-dependent equality adjustments;
see [the body-handoff validation](pa10-validation.md). Earlier oracle bytes are
unchanged except for the documented equality record in the region-boundary case.

## Contract and ownership

`types::Store` canonicalizes typed shapes and computes structural sizedness and
well-formedness once, when inserting a shape. Its inference context owns flat
variables, constraints and shared constructor edges, with iterative occurs
checking and finalization. Per-body canonical-type lookup is sparse, avoiding
crate-sized initialization for each function.

`sema::declarations` consumes resolved identities for aliases, function
signatures and concrete local annotations. Binder/elision accounting uses source
parameter boundaries and resolved lifetime identities. Dependencies and alias
cycles use an explicit work stack. Configured-out and detached syntax is excluded.

`sema::bodies` checks scalar functions and annotated initializers. It publishes
solved types, decoded scalar bits, selected operators/casts, function-pointer
adjustments, assignment targets, control-transfer targets and pending region
equalities. Expected function-pointer types propagate through calls, returns,
assignments, blocks and loop joins. Numeric fallback follows constraints.
Break sites use a flat linked arena. The checker reuses node/binding scratch
and clears only touched entries between bodies.

Control transfers occupy the existing operation variant. Their targets are
source-unit-local node identities; they need neither an additional dense fact
field nor a repeated file identity. A smaller standalone target field still
left alignment padding in the dense record, so that intermediate representation
was discarded. The public type-view bytes did not change.

`sema::type_view` and the CLI expose `--emit-types` and `--count-types` only after
semantic analysis succeeds. Definition names are presentation data. No frontend
phase invokes rustc or reparses its own text. PA10 consumes these typed facts;
PA11 must execute actual generated LowIR through the isolated CPPGM bridge.

## Evidence

The PA9 manifest declares 198 full-program outcomes before reference generation.
All are independently checked with pinned rustc 1.98.0 and default literal
lints. Families cover inference, numeric limits, aliases/cycles, occurs checks,
casts, calls, expected-type propagation, control transfers, demanded shape
validity, lifetime elision and file/include expansion. Auxiliary files are
registered with their owning source package.

At the original release, four explicit phase-boundary cases differed from final rustc acceptance:
initialization and immutable writes belong to PA14, region proof to PA15/16,
and constant-call eligibility to PA25. The reference preserves these obligations;
it does not claim to prove them. No other PA9 acceptance difference is exempted.

Typed tests check canonical sharing, iterative deep algorithms, selected
operations, target identity, initializer types and numeric bits. Separate CLI
checks verify deterministic complete output, count/footer agreement, the fixed
primitive prefix, occurs-check diagnostics and empty stdout on semantic failure.
The include-error fixture checks source and expansion projection.

Validation commands and retained evidence:

- `make test lint audit-rust test-harness`: 204 Rust tests; 1,304 PA1–PA9
  fixtures; strict Clippy/rustfmt; 120 audited Rust files with zero findings;
  16 harness tests and complete assignment discovery.
- Release-mode `cargo test --locked`: the same 204 Rust tests.
- `make ref-test`: all active references regenerated through the declared-outcome
  gate. Earlier PA1–PA8 stdout/stderr/status oracles remain unchanged; their
  receipts refresh compiler-source provenance.
- `make test-rustc`: independent checks for every active stage, including all
  198 PA9 programs and the PA2 bit/value comparisons.
- `make test-upstream`: 24 pinned imported fixtures and 126 pinned Reference
  files retain their original checksums.
- `make test-backend`: the pinned LowIR parser/native backend builds and executes
  the bridge smoke program with exit 42 and empty stdout/stderr.

Logs live in `obj/pa9-work/final-*.log`. Oracle-generation logs are separate
from independent rustc evidence. No generated build products are committed.

## Performance

The end-to-end API benchmark freezes identical Rust source at 4,000, 16,000 and
64,000 scalar functions plus one callee. It includes expansion, resolution,
declaration elaboration and body typing. Each comparison snapshots both release
binaries, checks full emitted fact equality, warms both binaries, records four
A/A calibration observations, and runs three ABBA blocks on CPU 10. Every
observation, binary/input hash and full-output hash is retained.

The prototype-to-complete-profile comparison is
`obj/perf/1791800900342337549/samples.json`. A/B paired medians were
130.11/137.76, 567.74/618.44 and 2,425.15/2,550.58 ms. This is a measured 5–9%
time increase for the additional semantic work, not a noise-only claim. At the
largest scale, the then-separate control-target field increased peak RSS from
458,224 to 502,064 KiB across the measured samples. That finding prompted the
operation-variant representation above.

The representation-only follow-up is
`obj/perf/1791801663161152423/samples.json`. All full fact outputs match. A/B
medians were 212.50/211.30, 603.45/603.04 and 2,583.02/2,552.10 ms. Timing was
noisy (the largest A/A observations ranged from 2,503.32 to 3,704.01 ms), so
these observations do not establish a speedup. Peak RSS fell from 502,148 to
458,304 KiB at the largest scale: 43,844 KiB saved. The smaller scales fell
from 33,020 to 30,916 and from 128,324 to 117,180 KiB. This restores approximately
the prototype's memory footprint while retaining the added semantic checks.

The public `--count-types` path also has a full-view equality/scaling baseline
at `obj/perf/1791800968009479024/samples.json`: candidate paired medians
149.53, 622.02 and 2,583.77 ms, with peak RSS 33,244, 127,516 and 503,984 KiB.
This precedes the compact control-target change. `make perf-types` reproduces
the workload; API and public-driver sources differ in spacing and are compared
only within their own frozen-input runs.

## Required initializer integration

The cumulative maintainer now validates required constant initializers. The
unchanged `constant-call.rs` boundary moved to PA25 as
`tests/spec/migrated/nonconstant-call.rs`, with an independently declared rejection.
The current PA9 manifest has 200 fixtures and three deferred phase boundaries;
the historical release counts above describe their original snapshot.
