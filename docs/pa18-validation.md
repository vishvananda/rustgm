# PA18 release validation

This document retains release and subsequent checkpoint evidence. Following PA20
projection normalization, partial callable inference and tuple coercion ordering,
the active suite has 437 cases: 272 accepted and 165 rejected.
Its former associated-type boundary is retained byte-for-byte as an
accepted control in `dev/tests/projection_paths.rs`, awaiting PA20's grading contract.
See `pa18/README.md` for the current suite counts.

The partial callable inference checkpoint adds three declared source controls in
`callables/`: independent borrowed callback inputs, an inferred whole higher-ranked
function-pointer argument, and rejection of an escaping borrowed result. Pinned
rustc agrees with all three declarations; the two accepted programs also execute
value assertions through both rustc and CPPGM. That checkpoint's complete PA18 check
covers 434 declarations, 267 executions and three codegen-only comparisons, with
one documented later-stage acceptance difference. Oracle generation adds nine
sidecars and preserves all 1,293 preexisting sidecar hashes. Qualification and
the hash review are retained in
`obj/pa20-work/projected-callable-arguments-v2/`.

The tuple coercion checkpoint adds three alias-runtime controls: a later field
reborrows using context inferred from an earlier field, nested block tails retain
that context, and the first field still moves when no reference context was yet
available. Outcomes are declared before independent rustc and native qualification.
The current suite requires 437 declaration checks, 269 executions and three
codegen-only comparisons. Its evidence is retained under
`obj/pa20-work/partial-projection-normalization-v1/`.

PA18 exposes demanded generic execution through the public LowIR CLI. The
maintainer implementation previously had the complete instance path behind a
library API; the CLI now selects it before the ordinary inspection body builder.
It performs original template proofs, ordinary source-body proofs and demanded
concrete proofs once each. Earlier views keep their separate contracts.

## Contract, fixture and source ledger

All 425 fixtures declare outcomes before oracle generation. Pinned rustc checks
all 425, with four explicit later-stage acceptance differences. There are 262
accepted programs: 253 clean executions, six qualified panic-aborts and three
deliberately divergent compilation-only controls. The other 163 reject; 159 also
reject in rustc. None of the three compilation-only cases is executed or included
in the 259 executable-comparison count.

Every fixture is original RustGM material. The manifest records the originating
maintainer table or new public-contract responsibility. Declaration/proof cases
append an empty entry where needed; this intentionally validates unused bodies.
Runtime cases retain in-source value or cleanup counter assertions. Exact duplicate
source cases are represented once. No rustc-generated diagnostic or IR is an oracle.

| Public responsibility | Fixture clusters (count) | Owning implementation |
| --- | --- | --- |
| Canonical arguments, inference and function identity | arguments (10), functions (39), entry (2) | `types/arguments.rs`, `types/parameters.rs`, `types/infer/arguments.rs`, `sema/declarations/generics/`, `sema/bodies/generics.rs`, `sema/entry.rs` |
| Alias substitution, kinds, defaults and query cycles | aliases (60), aliases-runtime (26), default-queries (30), defaults (5) | `sema/declarations/generics/{arguments,defaults,constants,captures}.rs`, `types/substitute/parameters.rs` |
| Built-in capabilities and parameter environments | bounds (23) | `types/parameters/{environment,predicates}.rs`, `sema/bodies/bounds.rs`, `sema/declarations/generics/bounds.rs` |
| Concrete fields, representations and aggregate inference | aggregates (5), constructors (19) | `types/aggregate/instances.rs`, `sema/nominals.rs`, `sema/bodies/nominals.rs`, `body/build/constructors.rs`, `abi/layout.rs` |
| Template ownership, initialization, lifetime and loan proofs | ownership (34), regions (19), loans (27), operands (11), proofs (4) | `body/build/templates.rs`, `body/promote/`, `borrow/templates.rs`, `borrow/regions/`, `borrow/loans/`, `borrow/check/incremental.rs` |
| Demanded function values, constants, promotions and recursion | callables (25), demand (7) | `types/instance.rs`, `body/instantiate.rs`, `body/instantiate/contracts.rs`, `lower/instances/{demand,dependencies,proof}.rs` |
| Generic destructor coverage, method binding and actual cleanup | destructors (19), drop-headers (56) | `sema/declarations/destructors/coverage.rs`, `sema/destructors/instances.rs`, `body/drops/glue/cache.rs`, `borrow/destruction/dependencies.rs`, `lower/cleanup.rs` |
| Explicit later-stage ownership | boundaries (4) | Current semantic rejection; methods PA19, associated items PA20, opaque returns PA21 |

Paths above are relative to `dev/src/`. The owning handout freezes the supported
profile, limits, deterministic instance/symbol ordering and existing LowIR dialect.
The file and architecture audit checks the implementation; no blanket Clippy lint
suppression was added. Actual instance storage, sharing, capture avoidance and
proof-token invariants also remain in the maintainer tests.

## Migration and independent evidence

The source bytes of PA12's generic constructor and PA17's generic destructor
boundary move unchanged into PA18. Their expected outcomes now accept; old
rejection sidecars are removed. PA12 has 106 cases and PA17 has 206. Their remaining
sidecar bytes are unchanged. Their manifest receipts are regenerated explicitly
alongside PA18 after independent qualification, rather than manually editing
receipt hashes or expected outputs.

The native harness now recognizes `native.kind: "codegen"` with a required reason
and no runtime outcome fields. It produces both native binaries but never runs
them. Harness tests reject ambiguous declarations and ensure the execution hook
is never invoked for such a control. Existing accepted cases retain mandatory
execution. Reports distinguish executable from compilation-only comparisons.

`rustc.emit: "obj"` is limited to complete programs and makes monomorphization
part of independent acceptance checks. Growing callable/destructor rejections
cannot be qualified by metadata-only emission. All compiler failures, signals,
timeouts, exit 86 and unqualified runtime aborts still fail validation.

Initial evidence before oracle generation is retained under `obj/pa18-work/`:
`pa18-independent-v2.log` and `pa18-native-v1.log`. The generated receipts are
`pa18/reference.json`, `pa12/reference.json` and `pa17/reference.json`. Final
regression and publication evidence is recorded below after the gates complete.

## Performance protocol

The immutable pre-change CLI is `drop-integration-candidate-v3` from commit
`6a45310`. The new CLI is `cli-release-candidate-v1`; its 323 production/config
source files are frozen in `cli-release-sources-v1`, with hashes in
`cli-release-v1-source-sha256.json` and binary identity in
`cli-release-v1-freeze.json`. The generic cleanup comparisons use the prior
`drop-integration-driver-v3-bin` maintainer API because the previous public CLI
cannot compile those programs. Those comparisons therefore span entrypoints.

Every job uses CPU 2, two warmups, four A/A calibration observations and three
paired ABBA blocks, retaining wall time, child CPU time, RSS, context switches
and full/count output equality. No RustGM builds or tests overlap timing.
A/A and paired observations are retained, including range variation; a median
alone does not establish a regression threshold or improvement.

The first run completed six jobs, then rejected an inherited declaration-only
record input because it had no main. No timing observation or correctness pass
was recorded for that input. The failed attempt is preserved in
`cli-release-v1-perf-failure.md`. New immutable copies add an entry and value
assertions; all three sizes execute successfully through rustc and CPPGM before
measurement (`cli-release-v2-independent-perf.json`). The second run measures
only the remaining seven jobs. Earlier source/binary snapshots and every completed
observation remain unchanged.

Across 13 completed jobs, all 234 observations and full/count equality checks
are retained in the two result files and their per-job `obj/perf/` receipts.
The following are the largest input sizes in each family:

| Workload | Scale | Prior median ms | CLI median ms | Change | CLI RSS KiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| ordinary-cleanup | 2048 | 155.98 | 161.93 | +3.8% | 21274 |
| ordinary-distinct | 2048 | 419.53 | 449.23 | +7.1% | 44514 |
| generic-records | 4096 | 197.57 | 196.92 | -0.3% | 42440 |
| repeated-drop-api-to-cli | 2048 | 474.54 | 471.40 | -0.7% | 39578 |
| const-drop-api-to-cli | 2048 | 447.18 | 447.73 | +0.1% | 36766 |
| partial-fields-api-to-cli | 2048 | 1664.49 | 1665.39 | +0.1% | 78572 |
| thin-reference-functions | 4096 | 493.12 | 494.22 | +0.2% | 77448 |

Ordinary cleanup/control medians rise about 0.7–7.1% across their six sizes.
Generic record controls range from a small decrease to a small increase;
several paired ranges overlap. Generic cleanup API-to-CLI comparisons are close
in this run but span different entrypoints. These observations do not support
a blanket speedup or zero-regression claim. The empty-template fast path avoids
a redundant dependency walk; it does not remove the new per-body proof assembly
cost. Earlier ordinary output remains byte-identical.

## Completed local release gates

All five final gates passed against the frozen production sources:

- `make test`: 750 frontend tests and 3,773 PA1–PA18 text-oracle comparisons.
- Release-mode Cargo: the same 750 frontend tests.
- `make test-native`: 15 explicit maintainer native tests, 1,117 course
  executable comparisons and three separate compilation-only comparisons.
  The existing 16 PA13 Reference/rustc runtime differences retain their declared
  outcomes. PA18 introduces no new runtime differences.
- `make test-rustc`: all active independent acceptance, literal and layout checks,
  including PA18's object-generation controls.
- `make lint audit-rust test-harness test-upstream`: rustfmt and Clippy
  all/pedantic with warnings denied; 476 audited Rust files with zero findings;
  27 harness tests; assignment discovery; 24 pinned upstream fixtures and 126
  pinned Reference files.

The terminal receipts and complete logs are `cli-release-v1-validation.json`
and `cli-release-{all,release,native,independent,quality}-v1.log` under
`obj/pa18-work/`. `cli-release-v1-reverified.json` rechecks their hashes,
frozen compiler sources/binary, moved input bytes, manifest/reference identities,
all 13 performance receipts and the separation of compilation-only evidence.
The initial report's only mismatches were the two obsolete boundaries moved
into PA18; no remaining earlier oracle was changed to conceal a regression.

PA19–PA21 remain planned and required by the active goal. This release completes
PA18's bounded assignment contract, not all four assignments or Rust conformance.
