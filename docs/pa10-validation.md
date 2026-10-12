# PA10 validation and body handoff

PA10 implements the typed-body profile in [its handout](../pa10/README.md).
It does not claim ownership checking or native Rust execution; those remain
explicit later-stage responsibilities. The original PA10 release preceded PA11;
see `pa11-validation.md` for the later scalar checks and wildcard-place correction.

## Architecture and feature ownership

`body::build` consumes the expanded/resolved report and completed PA9 facts.
An iterative action stack constructs function-owned blocks, statement ranges,
locals, values and shared argument lists. It never repeats lookup, inference,
label resolution or literal decoding. Reads become snapshot values before later
effects; primitive compound assignment reads its place after the RHS. Branch
and loop results use typed merge locals. Storage boundaries follow actual
control paths, including condition temporaries and labeled transfers.

Canonical types remain owned by PA9. The body model retains pending region
equalities and explicit constrained conversions without proving lifetimes.
The body text adapter shares PA9's type/scalar/region vocabulary. No compiler
phase interprets a rendered view or invokes an external frontend.

The body's scalar pool avoids making every value record as large/aligned as
a 128-bit literal. On the measured x86-64 build, `Value` shrank from 48 to
24 bytes and `Rvalue` from 32 to 20. Literal bits remain immutable typed data;
their public text did not change.

Downstream construction exposed two PA9 handoff issues. Grouped binding wrappers
now retain their type term; the positive regression belongs in
`pa9/tests/spec/inference/grouped-pattern.rs`. Region-dependent coercions now
publish their destination type as an `equality` adjustment, so construction
does not infer it again. The existing region-boundary oracle gains that one
record; the four documented PA9 acceptance differences remain unchanged.

The audit permits `body` to consume typed `resolve` identities/reports and
`lex` operator enums. It still forbids dependencies on later lowering/native
phases. Integration tests may read course inputs; compiler and driver code
cannot read fixtures/references. Two audit unit tests cover these boundaries.
All size, nesting, unsafe, dependency and lint checks still cover test sources.

## Independent and regression evidence

- 47 PA10 course inputs have declared outcomes and committed text oracles.
  All 47 independently agree with pinned rustc acceptance; none use an exemption.
- The 32 execution fixtures agree with hand-declared results and rustc execution.
  The test-only CFG machine checks dynamic live/dead state, initialization at
  reads, and evaluated values. It covers calls/recursion, function-pointer
  selection, operand order, primitive compound assignment, nested short circuit,
  branch/loop joins, labeled transfers, shadowing and initializers. It is not
  linked into the compiler. Its vectors are checked against course source bytes.
- Typed tests retain pending region conversions and exercise 10,000 additions
  without recursive construction. Rebuilding the same facts gives identical
  body text, and count mode matches the full-view footer.
- All PA9 accepted fixtures also construct body IR. The source-package cases
  include external modules and `include!` projections.
- `make test`: 207 Rust tests and 1,352 PA1–PA10 course fixtures pass. Release
  `cargo test --locked` passes the same 207 Rust tests.
- `make lint audit-rust test-harness`: strict rustfmt/Clippy pass; 126 Rust
  files have zero audit findings; two audit tests and 16 harness tests pass.
- `make test-rustc test-upstream`: all earlier independent checks pass;
  24 imported fixtures and 126 vendored Reference files retain their checksums.
- Explicit `make ref-test` regenerates all receipts through the outcome gate.
  Earlier PA1–PA8 oracle bytes are unchanged. PA9 has its documented equality
  record and new grouped-binding fixture; its suite now has 199 cases.

Evidence is retained under `obj/pa10-work/`: `compact-full.log`,
`compact-release.log`, `compact-references.log`, `independent-final.log`,
the runtime directories and the manual pre-activation independent checks.
No build products or raw measurement artifacts are committed.

## Performance

All runs use immutable binary snapshots, frozen sources, full-output equality,
two warmups, four A/A observations and three ABBA blocks on CPU 10. No project
build/test ran concurrently with the measurements. Other host activity can
still affect wall time; raw observations are retained rather than discarded.

`obj/perf/1791803561030263048/samples.json` compares the PA9 release API driver
with retained equality adjustments at 4k/16k/64k functions plus a callee. A/B
medians are 149.35/146.40, 577.05/565.70 and 2,489.19/2,487.54 ms. Full facts
match, and largest peak RSS is 458,256/457,972 KiB. These timings show no material
regression and do not establish a speedup.

The initial public body-view baseline is
`obj/perf/1791803628935143236/samples.json`. Its largest paired samples were
noisy (2,991.06–4,775.28 ms), so the 4,628.76 ms candidate median is not a
meaningful comparison against another run's median.

The scalar-pool A/B comparison is
`obj/perf/1791803953630832446/samples.json`. Full emitted body bytes match at
every scale. Paired A/B medians are 160.96/162.45, 692.39/692.75 and
2,789.55/2,766.85 ms. Peak RSS falls from 37,372/141,004/556,036 to
36,672/137,836/542,660 KiB: 13,376 KiB saved at 64k functions. Small timing
differences are comparable to calibration variation; the memory saving is the
reason to retain the representation. `make perf-body-ir` reproduces the workload.
