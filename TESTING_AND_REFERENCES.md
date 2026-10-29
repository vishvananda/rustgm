# Testing and references

This checkout owns the maintainer solution. `dev/rustgm-ref` invokes the built
solution, just as reference wrappers do in cppgm-extended's solution checkout.
Default tests compare candidates with **committed text**, not a live solution.

## Assignment completion and component publication

`implemented-draft` denotes an active draft text suite; `implemented` denotes
a released documented course profile and requires existing release evidence in
assignments.json. Neither state means arbitrary Rust compatibility. Planned
assignments still reject missing/empty-suite success.

[assignment-plan.json](assignment-plan.json) orders finite work units independently
of stable PA ownership numbers. Its implementation and publication states are
separate; a completed component cannot mark an unfinished whole PA implemented.
`make test-harness` and fastcheck metadata validation enforce dependency order,
retained requirements and publication gates. New component release manifests
must declare outcomes and every input before reference generation, reuse the
existing text/native/source validators, and retain all unclosed parent requirements.
Run `python3 scripts/assignment_plan.py` for the current ordered status. Fixtures exercising
reordered prerequisites declare `work_units` from their actual owning units;
only those units' transitive dependency owners and the active PA1–PA24 foundation
are admitted. Legacy cases retain numeric placement checks.

Finite components use the same runner while the parent PA remains planned.
Declare a canonical `paN/tests/units/UNIT.json` manifest in that work unit's
`manifest` field. Its nonempty `discovery` list claims complete directories or
explicit `.rs` inputs under that PA's tests; every discovered source must be a
primary or auxiliary. Each primary declares its owning `work_units`. Duplicate
primary publishers, overlapping discovery and unrelated future features reject.
The selected contract then runs explicitly:

```sh
python3 scripts/course.py paN --unit UNIT --app /immutable/qualified/rustgm
python3 scripts/assignment_metadata.py --unit UNIT
python3 scripts/check_layout.py --unit UNIT
python3 scripts/check_syntax.py paN --unit UNIT --app /immutable/qualified/rustgm
python3 scripts/check_native.py paN --unit UNIT --app /immutable/qualified/rustgm
```

Use the resource/storage wrappers for execution, as for other validation.
Generation uses the existing `--bless` and optional exact `--case` selectors
after independent outcome review. Sidecars stay beside their source inputs;
unit receipts and scoped batches live under `paN/units/UNIT/`. Full and partial
unit receipts bind their actual stage/unit/manifest identity and every output
channel. A unit cannot change an active parent's case contract or output bytes:
generate changed shared references through the parent first, then adopt those
same bytes in unit provenance. Oracle symlinks reject before execution/writing.

`make test`/`test-report`, `test-maintainer-native` and `test-rustc` also schedule
released units through the same text/native/independent runners. The
`--released-units` selector enumerates only declared releases; when none exist
it reports a skipped scheduler. An explicit planned PA, missing unit contract or
empty component still fails. Publication retains the owning scope, independent
and native evidence, release gates and every residual parent requirement; the
tooling alone does not release a component.

## Development checkpoints

Maintainer integration roots under `dev/tests/` are registered as modules in
13 binaries under `dev/tests/suites/`. `make test-harness` checks that every
root appears exactly once in an unconditional target. Add new roots to their
owning group; do not change test bodies or ignore markers to move a suite.
List the mapping with `python3 scripts/maintainer_suites.py list`.

Focused selection retains the original suite name:

```sh
python3 scripts/storage.py run -- python3 scripts/maintainer_suites.py run constant_patterns
python3 scripts/storage.py run -- python3 scripts/maintainer_suites.py run lexer --case lifetime_and_character_boundaries
```

Add `--ignored` to run a suite's backend and interoperability controls.
The runner enumerates the actual binary,
selects the exact outer module and executes its full test names with `--exact`.
Unknown cases and empty selections fail. `--list` shows that selected inventory.
Cargo's ordinary filters match substrings, so a bare `loans::` filter can also
select `template_loans::`; use the runner for suite selection. Apply the ordinary
resource/storage wrapper to direct Cargo commands.

Batch related selections to compile their targets once. The batch runner reads
Cargo's executable records, enumerates each binary, excludes ignored tests from
the ordinary lane, and runs the union of exact names once per binary. Missing,
overlapping and empty selections fail. It also checks the executed count against
the selected inventory; a successful zero-test process cannot qualify a batch.
Direct test launches restore Cargo's package metadata environment from the
explicit `dev/Cargo.toml` package fields, including `CARGO_PKG_NAME` and the
manifest paths. These overrides apply to inventory and execution without
changing the parent environment; receipts record only the package overrides.
This keeps compiler subprocesses in environment-sensitive tests consistent with
ordinary `cargo test`. Workspace-inherited metadata is rejected explicitly.
The private library lane accepts complete test paths, with no substring filters:

```sh
python3 scripts/storage.py run -- python3 scripts/maintainer_suites.py batch lexer constant_patterns
python3 scripts/storage.py run -- python3 scripts/maintainer_suites.py batch lexer --lib-case resolve::body_origin::tests::example
```

The private path above illustrates the syntax; choose an existing test from the
library inventory. Add `--release` for optimization-sensitive controls and
`--ignored` for native controls. Each batch creates a fresh directory under
`obj/qualification/focused/` with commands, raw output, timings, binary hashes,
exact inventories and terminal status. `--evidence-root obj/NEW-BATCH` chooses
that directory explicitly. A listing receipt is never recorded as a passing run.

The batch runner prunes disposable native outputs after each successfully
completed test target, before starting the next target. Its receipt records each
cleanup. Cargo caches, pinned binaries, source inputs and diagnostics remain;
failed targets retain their outputs for diagnosis. Custom multi-target runners
must use the same inter-target cleanup under their existing storage/resource
lease, rather than accumulating every native executable until the batch ends.

Object interoperability tests are always registered and retain their explicit
`#[ignore]` markers. Select them with `--ignored`, like the other native tests;
their pinned object backend and host C compiler remain required when executed.
The former empty `object-interop-tests` Cargo feature is removed. Default and
`--all-features` now compile the same production configuration, avoiding a second
frontend build merely to select native tests. No test body or ignore marker is
changed, and an unavailable native dependency still fails an explicit native run.

Development checks have a five-minute total wall-clock budget, including resource
admission, build, lint, audit and execution. Use the fast workflow for ordinary
edits and coherent development pushes:

```sh
make setup-audit  # once per checkout
make fastcheck FAST_ARGS='lexer'
make fastcheck FAST_ARGS='constant_patterns --lib-case complete::private::test_path'
make fastcheck FAST_ARGS='lexer --pa pa1'
make smoke
```

Replace the private path with a real complete test name. Select the smallest
set covering the changed behavior, a negative control and relevant callers.
The default is the lexer smoke suite, not a claim to cover arbitrary changes.
Use repeated `--pa paN` options for affected text-oracle assignments. The runner
builds/passes the debug CLI explicitly, avoiding `make test-paN`'s release build.
Planned assignments still fail and are never reported as empty passing suites.
`make smoke` uses that same runner and deadline for the shared push selection:
ordinary lexer cases, actual source roles/Pointee-to-LowIR, reached CTFE faults
and ownership, encoded generic runtime/constant imports, and edition/CLI
controls. Its two integration binaries share the production build; it requires
neither a private-library build nor the external native backend. `SMOKE_ARGS`
passes ordinary runner options, such as a justified local output cap.
Before compiler steps, the runner checks every active assignment's metadata with
the same validators as the full layout check. Pending generated references are
allowed here; the full layout check still requires their complete provenance.
`make fastcheck` itself applies resource/storage protection; do not wrap it in
another lease. Shared-controller admission never waits for a slot. Its default
absolute output cap is 16 GiB, including retained caches and pins; use
`--max-output-gib N` only with justified headroom for the actual worktree.
Admission refusal and deadline exhaustion fail the run. Do not increase the
five-minute deadline or silently omit tests to turn a timeout into success.
Partition unrelated target groups or investigate a slow build instead.

The `dev-fast` Cargo profile keeps assertions and overflow checks, disables
optimization and debug symbols, and enables bounded incremental caches under
`obj/rust/dev-fast` and `obj/rust/private-lint/dev-fast`. It does not rebuild release artifacts. Formatting covers
all implementation/test modules. The architecture audit reads content hashes,
reuses unchanged per-file syntax results, then reruns global duplicate,
filename-stem and ownership checks over the complete current file set. Audit
implementation, parser versions and role-table changes invalidate that cache.
Strict Clippy covers selected test targets and their library; integration/text
selections also check the CLI. It is not
followed by a redundant `cargo check`. Only selected test binaries are compiled.
Private selections compile the library harness once with `cargo test --lib
--no-run`, setting `RUSTC_WORKSPACE_WRAPPER` to the installed `clippy-driver`
and `CLIPPY_ARGS=-Dwarnings`. This checks `cfg(test)` with strict Clippy and
links the executable in the same compilation. Cargo builds actual dependency
libraries and reports the exact test executable; the runner does not infer an
executable from check-mode metadata or search old cache outputs. The pinned
Clippy driver tracks its arguments for Cargo cache invalidation. Normal host
compiler flags remain intact. A separate `cargo clippy --lib` excludes private
tests, while Cargo's `--tests` would select unrelated integration targets.
The harness has a stable build cache under `obj/rust/private-lint` so it cannot
evict ordinary Clippy results. Idle retirement also bounds this cache while
preserving each crate's newest warm session and explicit pins.
Bootstrap cold worktrees one target group at a time; combine related groups
after their caches are warm. Keep each command within the same deadline.
Private tests use complete names, integration suites retain exact outer-module
boundaries, and every selected name/count must actually execute. A new binary
no longer requires executing its complete private inventory.

For a new source fixture, declare its expected outcome and first try the frozen
input against an already qualified immutable development CLI. Record its hash,
input bytes and effective flags. Input-only probes need no Cargo rebuild or
new private harness; use the storage/resource wrapper and retain the result.
For source-provider fixtures, preserve the actual source-core identity and crate
arguments. A later compiler edit still requires affected current-source checks.
If private instrumentation is necessary, capture the bounded reachable typed
graph and exact binder coordinates together. Confirm the actual mismatch before
changing a shared kernel; a smaller reducer that passes is a control, not proof
of the proposed cause. Remove temporary instrumentation before qualification.

Each run saves raw output, step times, source-content fingerprints, binary
hashes and exact inventories under `obj/qualification/fast/`. A timeout is a
failed check with retained evidence, and owned compiler/test processes are
stopped. Source changes during the run invalidate its result. Keep source
unchanged until the command finishes. The launcher reserves time for process
cleanup within the deadline; it can stop earlier than 300 seconds.

Original-core discovery may declare an explicit finite `source_contract_limits`
object in both its manifest profile and producer binding, with `work`,
`attempted_bytes` and `retained_bytes` as decimal u32 values. The observer rejects
missing, mismatched or malformed declarations and emits the three corresponding
`--source-contract-*` compiler options. An omitted object preserves the legacy
defaults and passes no new options. Effective values are recorded in the command
binding and result. Keep changed-resource observations separate from earlier
default-profile receipts; a larger allowance is not evidence of less work or a
completed original crate. Select allowances from measured workload evidence and
retain every charge and sticky failure test.

Ignored/native tests remain explicit. Select them with `--ignored` and declare
all disposable output roots using repeated `--prune-under obj/OUTPUT` options.
Cleanup visits only those roots after successful targets, preserving diagnostics,
inputs and pins. Failed outputs remain for diagnosis. Do not scan every historic
`obj/` directory after each ordinary focused test. Keep the reusable dev cache
within the worktree's shared output cap; `make tidy-build` retires obsolete
Cargo caches. Release/performance commands continue to disable incremental builds.

Periodically run `make tidy-archive` between batches. It compacts inactive
PID-named scratch directories older than 24 hours into a lossless archive,
verifies every regular/hardlinked file's restored SHA-256, checks that originals
did not change, then removes the loose copies. Pins, active builds and managed
caches are preserved. `obj/archives/*.json` records the archive hash, original
directories and restore command. Archived evidence remains recoverable with
`tar -xzf obj/archives/NAME.tar.gz -C .`. This reduces the number of filesystem
entries that every mandatory disk-accounting scan visits. For a configured
controller, use `ARCHIVE_RESOURCE_ARGS='--max-output-gib N'` with the worktree's
justified cap; `ARCHIVE_ARGS='--older-than-hours N'` changes the retention age.

Focused passing evidence is sufficient for development integration and main
pushes. Retain pending broad checks explicitly in handoffs. Push/PR CI runs the
bounded smoke workflow plus harness/upstream/audit-cache contracts. Nightly
and manual CI run full release tests, all native maintainer tests, uncached
architecture audits, all-target strict Clippy, all active text suites, debug
library invariants and independent Rust qualification. Cancel superseded push
jobs rather than accumulating outdated builds. The fast CI smoke is deliberately
small; source-specific regression selections belong in the developer's receipt.

At assignment release, run the owning handout's complete acceptance checks,
all active earlier independent source/native suites and full debug tests.
Independently qualify new outcomes before generating/reviewing text oracles.
Fixture changes still need upstream verification; backend-contract changes
need backend qualification. These release requirements do not apply to each
implementation edit or ordinary development push.

Measure meaningful algorithm checkpoints with immutable binaries, frozen
inputs, A/A calibration, paired ABBA blocks and output equality. For routine
edits, prefer bounded-work and scaling invariants. Run the broad performance
matrix at assignment release or to investigate a regression; preserve every
observation and avoid repeating profile/release builds for related edits.

After checks pass, repeat only what later source changes or failures invalidate.
Documentation-only changes do not require rebuilding an unchanged compiler.
The existing `make test`, `make test-maintainer`, `make test-debug` and independent
qualification commands remain available for periodic/milestone verification.

For a failed combined run, preserve its failure and every completed target's
source snapshot, binary hash, exact inventory and result. Review the complete
source delta before retaining an unaffected target; a passing count alone is
insufficient. Rebuild and rerun every affected target, including native cases,
then run all active text cases with the corrected CLI. Changes to shared proof,
lowering or representation behavior generally invalidate the broader run.
Record retained and fresh results separately rather than describing recovery
as a fresh full pass. Feature work that changes keyword classification or cold
owner layouts should include the owning private classification/layout invariants
in its focused batch; all-target Clippy cannot check their runtime assertions.

`make test-host-builds` separately qualifies PA34's host dependency executor with
an authored offline Cargo graph and actual pinned host compilation. It declares
the expected generated source before either execution, runs an independent Cargo
build to its deliberate target stop, then compares the host executor's output.
The executor performs no target Rust compilation. CI runs this small check once;
it needs no RustGM rebuild or CPPGM backend. Receipts and original/generated
sources stay under `obj/qualification/host-graph/`, and scoped cleanup removes
host binaries and Cargo outputs. Python harness tests cover graph sharing,
context/metadata isolation, failed publication and bounded work separately.

The [source destruction and host build checkpoint](docs/source-destruction-editions-checkpoint.md)
qualifies 3,270 maintainer tests, 8,382 text cases and the new host DAG executor.
Its text sweep takes 19.14 seconds; optimized test compilation remains the main
checkpoint cost. It retains passed integration results across five assertion-only
layout corrections and reports the observed wall-time increases alongside the
instruction, memory and scaling gates.

The [source-trait checkpoint measurements](docs/source-traits-editions-checkpoint.md)
separate observed build, execution and text-suite costs under this cadence.
They are shared-machine observations, not a controlled harness speedup claim.

The [source sizing and generic-call checkpoint](docs/source-calls-sizing-checkpoint.md)
passes 3,362 fresh release/native tests and 8,382 text cases. Its text sweep takes
19.29 seconds; optimized compilation takes 17 minutes 13 seconds. It records
separate independent checks, retained failures, 59 performance inputs and
17.20 GiB of automatic artifact cleanup.

The [source const-call and slice checkpoint](docs/source-const-slice-checkpoint.md)
passes 3,400 fresh release/native tests and 8,382 text cases. Its 60-input
performance matrix has 39 equal-output comparisons and 25 new candidate native
preflights. No instruction, memory or scaling threshold fires; the report also
retains the observed wall-time increases and the corrected filename-only harness
failure. Functional cleanup reclaims 17.13 GiB.

The [source capabilities, dynamic cleanup and return-capture checkpoint](docs/source-capabilities-dyn-returns-checkpoint.md)
qualifies 3,424 maintainer tests: 2,206 retained integration results and 1,218
fresh library results after a test-only layout correction. All 8,382 text cases
pass. Its 37-input performance matrix retains 26 equal-output comparisons and
27 native preflights, with no instruction, RSS or scaling trigger. The record
preserves the original layout failure and separate benchmark preflight repairs.

The [local stability and callable checkpoint](docs/library-stability-callable-checkpoint.md)
qualifies 3,477 maintainer cases: 3,180 retained unchanged-target results and 297
fresh frontend results after a test-only feature-inventory phase correction.
All 8,382 text cases pass in 17.68 seconds. Its 48-input matrix has 26 equal-output
comparisons and 24 native preflights, with no instruction, RSS or growth trigger.
Source feature/inventory edits must include the `feature_requests` suite in focused
selection: loading may deliberately retain a request that completed expansion
must reject. The record preserves the failed full run and benchmark preparations.

The [constant selector checkpoint](docs/source-ctfe-selector-checkpoint.md)
passes 3,488 fresh release/native cases and 8,382 text cases in 18.23 seconds.
Its 51-input matrix has 48 byte-equal comparisons and 27 candidate native
preflights; no instruction, RSS or scaling threshold fires. Functional cleanup
reclaims 17.20 GiB. Documentation-only publication retains these exact qualified
compiler/test sources and does not repeat the full functional batch.

The [defining cleanup and source-producer checkpoint](docs/source-defining-producer-checkpoint.md)
passes 3,538 fresh release/native cases and 8,382 text cases in 19.35 seconds.
Its 55-input performance matrix retains 54 byte-equal comparisons and 31 native
preflights with no instruction, RSS or scaling trigger. A separately reviewed
typed-facts provenance change has no paired speedup claim. The report records
the original preflight failure and 17.20 GiB of functional artifact cleanup.

For a validation batch, pass `scripts/course.py --evidence-root obj/BATCH/course`
and include that directory in the storage guard's `--prune-under` roots. This
keeps the same fixture comparisons and complete receipts while avoiding a cleanup
walk through historical course runs. The default remains `obj/tests`; custom
evidence roots follow the storage guard's containment, cache and symlink rules.

## Oracles

Each manifest-listed `x.rs` has `x.rs.ref` (stdout), `x.rs.ref.stderr` and
`x.rs.ref.exit_status`. Compare bytes exactly; do not strip whitespace, reorder
nodes or normalize diagnostics. PA-specific handouts own their deterministic
format. Errors initially use source byte offsets, not checkout-specific paths.

`tests/manifest.json` independently declares expected success (0) or diagnosed
rejection (1). Exit 86, other statuses, signals and timeouts cannot be accepted
as rejection. There is no keep-going success loophole: collect all failures but
return failure if any fixture fails. Empty suites and missing oracles fail.

```sh
make test-pa1
make -C pa1 test
make test-report
make test RUSTGM_TEST_APP=/absolute/path/to/candidate
make ref-test-pa1
```

The course runner also accepts repeated `--case tests/.../name.rs` selectors.
It validates the complete manifest and discovery before executing only those exact
normalized paths; duplicates, unknown paths and empty selections fail. With
`--bless`, a selection updates only its sidecars after every selected declared
outcome passes. It preserves `reference.json` and writes a tracked scoped receipt
under `reference-batches/`, binding selected case contracts, inputs, outputs and
compiler sources to the preserved full-stage receipt. This is partial provenance,
not qualification or regeneration of the rest of the stage. Layout checks overlay
those explicitly selected facts while requiring complete current fixture and
case-contract coverage, including each case's effective mode. A legacy full
receipt without per-case contracts permits partial generation only while its
whole manifest hash is unchanged; refresh it fully before adding cases. Historical
batches with a different base must be explicitly superseded with exact retained
receipt hashes; missing, altered or unexplained historical batches fail validation.

Only the explicit `ref-test` targets and an authorized selected `--bless` batch regenerate sidecars. The generator first
checks every declared outcome and writes no reference if any differs. Review
the complete generated diff. `reference.json` records source and input hashes;
it is provenance, not an alternate expected answer. Never auto-bless in CI.

## Independent evidence

Text generation alone cannot prove the solution correct. Maintain hand-checked
small token vectors and invariant tests; compare full supported programs with
the pinned rustc for acceptance/runtime behavior. `make test-rustc` checks the
imported full-program fixtures independently. Raw lexer/parser fragments are
not automatically valid whole Rust programs and must not be classified that way.
PA2 adds `scripts/check_literals.py`: rustc evaluates numeric bits and textual
bytes and checks expression rejection independently of the decoder. Its output
is compared with both the candidate and committed oracles. Overflow linting is
explicitly allowed for this decoding stage; inference and contextual lints are
later assignments. Stream fragments and upstream programs are classified in the
manifest rather than silently treated as runnable value expressions.

PA3's `check_syntax.py pa3` checks complete programs and evaluates configuration
predicates with the pinned target/profile. Cases may declare `args` as explicit
`--cfg`/value or `--crate-name`/value pairs; the harness preserves argument boundaries and records them
in evidence. Reference receipts hash the manifest as well as source fixtures,
so changing configuration or expected outcomes requires an explicit review.

Complete entry-free sources use `rustc.kind = "library"`; object qualification
is supported without adding a synthetic `main` or rewriting fixture bytes.
A LowIR library case additionally declares
`library: {"crate_name": "example", "disambiguator": "fixture_1"}`. These are
bounded ASCII crate identities; a competing `--crate-name` argument is rejected.
The frontend receives the explicit library identity and rustc receives the same
crate name with its library object mode. An accepted native library case requires
`native.kind = "object"` and a nonempty reason, with no runtime outcomes. The
runner requires the separately pinned CPPGM object capability, verifies both
outputs are x86-64 ELF relocatable objects, records their hashes and removes them.
Object qualification makes no runtime equivalence claim. Programs retain their
existing executable and explicitly justified code-generation-only contracts.
The syntax runner honors `RUSTGM_TEST_JOBS`, or an explicit `--jobs` from 1 to 8.

PA4's `check_syntax.py pa4` checks expression fragments in explicit wrappers.
A mismatch between parser acceptance and rustc acceptance requires a manifest
reason (for example, type checking deferred to a later stage). Structural tests
check AST shape independently of generated sidecars; resource tests check deep
input, bounded lookahead and nonrecursive arena destruction.

PA5 checks full crate syntax and selected configuration-owner facts. Its typed
invariants check source spans, child ordering, a compact ordinary node layout,
inactive-subtree exclusion and diagnosed nesting. External-module and unresolved
type cases explicitly defer loading/resolution rather than claiming rustc
acceptance equivalence. Earlier expression-fragment oracles remain unchanged.

PA6 adds file-input cases with explicitly declared auxiliary module sources.
Discovery and receipts include those sources; independent checks copy only the
declared package into a fresh directory. Name-resolution invariants check target
identity independently of generated output, including shadowing and reexports.
File cases run from the repository root with relative requested paths, keeping
future `file!()` text oracles independent of checkout location. Requested source
aliases remain intact; canonical identities are only loader cache/cycle keys.
Auxiliary files belong to the requested primary file's package directory, even
when the primary itself is a symlink. Targets remain inside the assignment's
fixture tree. Harness evidence records the working directory.

PA7 independently checks all matching/transcription fixtures with rustc. Typed
invariants additionally verify ordered-rule fragment reuse, capture opacity,
repetition cardinality and doc-token provenance. The two deliberate acceptance
differences defer generated-syntax parsing to PA8. See `docs/pa7-validation.md`.

PA8 checks all 94 expansion/hygiene programs with pinned rustc. A case may supply
`environment` as a name-to-string map; null removes that name from the child
process. The same overrides apply to both compilers and are recorded in harness
evidence and the manifest receipt. Inherited runner variables are otherwise
preserved. No test variable is read implicitly from the developer's environment.
The optional rustc `default_lints: true` metadata omits the usual lint cap for a
case whose expected rejection depends on a default lint, such as incomplete
`include!()` expressions. Such a case is not an acceptance-boundary exception.

PA9 has 200 full-program type-view fixtures. Its rustc checks retain default
literal lints. Three manifest entries explicitly defer initialization/write
permissions or lifetime proof to the later owning assignment. The former
constant-call boundary moved unchanged to the PA25 draft with a declared
rejection once the maintainer implemented required initializer validation. The
type view retains deferred facts without claiming those obligations are proved.
CLI tests check deterministic output, count/full-view agreement and
empty stdout on semantic failure. See `docs/pa9-validation.md`.

PA10 adds 50 typed-body fixtures. Its integer CFG test executor verifies the
35 execution cases against both declared results and rustc, including dynamic
storage state and evaluation order. It is test-only and never implements a
frontend phase. Pending region constraints remain explicit in body IR.
See `docs/pa10-validation.md` for the feature handoff and measurements.

PA11 has 191 scalar LowIR fixtures, including 180 executable comparisons with
pinned rustc and the qualified CPPGM backend. `make test-native` is required
locally; ordinary hosted CI has no qualified backend binary and checks the
committed text, frontend invariants and harness instead. Compiler signals and
exit 86 always fail. Explicit runtime-panic cases require CPPGM exit 134 with
empty streams and rustc SIGABRT with a panic diagnostic. Success requires clean
exit zero; source assertions compare full values. No native outcome is inferred
from a text-oracle match. See `docs/pa11-validation.md`.

Maintainer Cargo tests that execute CPPGM are explicitly ignored in the default
frontend suite. Run `make test-maintainer-native` to qualify the backend and run
all backend-dependent tests; `make test-native` includes this target before
the course native suites. Missing configuration or changed backend bytes fails
that target. No test silently treats an unavailable backend as successful native
execution. Hosted CI runs the frontend suite and reports these tests as ignored.

Rustc has no LowIR output. From PA11, maintainer-generated LowIR is the course
text oracle, with independent run-pass checks through CPPGM versus rustc. Do
not grade LLVM IR or assume Rust's unspecified layout matches rustc. Test exact
layouts where guaranteed or where the course contract deliberately specifies
them. Keep runtime evidence separate from the raw LowIR text comparison.

PA12 has 110 fixtures: 41 clean native executions, ten expected bounds panics,
12 layout cases and 47 rejection or explicit stage-boundary cases. The manifest
declares `--emit-layouts` as a supplementary mode; a case's mode must be one of
the assignment's declared modes. `make test-layouts` checks declared size,
alignment and field-offset vectors against candidate/committed text and rustc
probes where guaranteed. Course Rust field order and transparent ancillary ZST
offsets have explicit exceptions. `make test-rustc` includes these layout checks
and PA11/PA12 acceptance checks, so hosted CI exercises them without a backend.
`make test-native` qualifies both PA11 and PA12; `make test-native-pa12` selects
the aggregate suite. Layout cases never count as native execution. See
`docs/pa12-validation.md` for the profile and retained evidence.

PA13 has 796 fixtures: 531 LowIR cases and 265 selected-pattern inspections.
All have pinned-rustc acceptance checks; eight explicit qualifications cover
pending reference proofs and the Reference's non-reference-constant autoderef
rule. Native qualification runs 364 programs, of which 348 have matching clean
execution. Eight signed-zero pairs (16 programs) deliberately assert the
different Reference/rustc guard-retry counts. `native.rustc_outcome` declares the
independent pass/panic outcome, and a nonempty `native.reason` is required when
it differs from `native.outcome`. Both programs still compile and execute;
signals, exit 86, timeouts and unqualified aborts still fail. Receipts and summaries
report these runtime differences explicitly. No inspection case counts as native
execution. See `docs/pa13-validation.md`.

PA14 adds 124 cases: 112 ownership views and twelve executable LowIR controls.
The independent acceptance and runtime expectations agree with pinned rustc
without exceptions. `--emit-ownership` and `--count-ownership` expose the same
move/initialization proof already required for LowIR; this is not a lifetime or
loan proof. Root independent/native targets include PA14. See
`docs/pa14-validation.md` for the sparse path/bitset contract and scaling evidence.

PA15 has 202 region-contract inspections: declarations, elision, variance,
explicit/implied bounds, callable universes and borrow-place provenance. PA16
has 304 cases: 185 inspections/rejections and 119 LowIR executions. All
acceptance outcomes agree with pinned rustc without exceptions. Their separate
`--emit-regions` and `--emit-loans` views expose actual typed proof facts; the
contract view alone cannot authorize execution. Twelve earlier executable
boundary cases moved to these owning assignments instead of retaining obsolete
rejections. The earlier type/body/pattern views retain pending directed region
facts; independently reviewed oracles reflect those facts.

`make test-rustc` includes both stages; `make test-native-pa16` selects PA16's
119 execution cases. The root native target includes PA16 and the explicit
maintainer native tests. See `docs/pa15-pa16-validation.md` for the release review.

PA17 has 205 LowIR fixtures: 135 accepted programs and 70 rejections. Two
accepted programs declare panic-abort; the other 133 require clean execution.
The former PA17/PA18 inherent-method boundaries now have accepted maintainer
coverage in `dev/tests/dot_methods.rs` and accepted PA19 course fixtures.
The root independent and native targets include PA17. `make test-native-pa17`
selects its course executions. Its counter fixtures and 33 order fixtures are
also read directly by the maintainer suite.

The explicit `native_drop_order` test instruments only a named no-op observer
and an entry count check after successful RustGM lowering. Rustc uses a printing
observer on the same program; each result must match a previously declared trace.
The native observer checks every callback value and final count, with negative
controls for wrong values and missing callbacks. Cleanup, flags, field addresses
and source control flow remain compiler-generated. This separate observer test
is not counted as an unchanged-source native comparison. Course text oracles
remain uninstrumented; 22 accepted counter programs check unchanged-source
cleanup effects through both compilers. See `docs/pa17-validation.md`.

PA18 has 450 generic LowIR cases: 280 accepted and 170 rejected. There are
277 executable comparisons and three deliberately divergent codegen-only
controls. Their `native.kind: "codegen"` requires a nonempty reason and forbids
runtime outcome fields; both compilers produce native code, but neither binary
is executed. Counts report these separately. All other accepted LowIR cases
retain the mandatory execution/outcome contract.

An explicit `rustc.emit: "obj"` selects object generation in the independent
acceptance checker. Growing-instance rejection and demand-only controls need
that stronger check because metadata emission does not monomorphize bodies.
It is valid only for complete programs. The former PA18 opaque-return boundary moved byte-for-byte to PA21 with its
accepted source and execution contract; there are no new runtime differences.
The former associated-type boundary is retained byte-for-byte as an accepted
maintainer control awaiting the PA20 grading contract.
The generic constructor/destructor boundaries moved from PA12/PA17 with unchanged
input bytes and newly qualified accepted oracles. Root independent/native targets
include PA18. See `docs/pa18-validation.md` for the feature ledger and release gates.

PA19 declaration impl proofs have maintainer coverage in
`dev/tests/trait_bounds/declarations.rs`: closed bounds, source order, cyclic
well-formedness, invalid headers and bodies, method conformance, nested callable
lifetimes, initializer/default types and Drop coverage. Every source control is
independently checked with pinned rustc; four executable programs also compare
RustGM/CPPGM with rustc. Function-item controls verify that an already-checked
signature keeps its own binders while item substitutions still satisfy declared
bounds. These controls also contribute to the PA19 course suite.

`dev/tests/trait_bounds/callables.rs` checks nominal requirements inside function
pointer binders: deferred local predicates, independent closed/outer predicates,
nested callable scopes and actual call obligations. Acceptance is independently
qualified with pinned rustc. Its explicit native test executes bounded pointer
inputs, outputs and trait lifetime arguments through both compilers.

The PA19 dispatch maintainer tests in `dev/tests/trait_bounds/dispatch*.rs` add
50 acceptance controls and 33 executable programs for `Trait::method` calls.
They cover required methods, defaults and overrides, recursive/default forwarding,
type/const/lifetime arguments, dependent defaults, generic function values, and
constant/static function pointers. Pinned rustc independently checks acceptance;
the explicit native test compares execution through CPPGM and rustc. Type-fold
unit controls separately check binder capture and shared-DAG work.

The `trait_bounds/qualification*.rs` tests add 47 acceptance controls and 28
executable programs for `<T as Trait>::method`, covering explicit Self, inference
holes, defaults, aliases, imports, macros and captured lifetimes. Their unsafe-call
control separately checks rustc acceptance and RustGM's existing PA27 boundary.
The `trait_bounds/implicit*.rs` tests cover `T::method`, `Self::method` and
`<T>::method`, including bounds, inherent fallback, enum variants, imports,
macro scopes and arguments that must not disambiguate a name. Executable checks
verify the chosen body. Concrete and rigid scaling controls combine 1,024
unimported homonyms with 1,024 independent call scopes, using both type paths and
dot calls.

The `trait_bounds/receiver*.rs` tests add 93 acceptance controls and 50 native
comparisons for trait dot calls. They cover candidate order, defaults, scoped
traits, reference Self, numeric inference, array unsizing, captured lifetimes,
returned borrows and two-phase arguments. The former PA18 trait-method boundary
is preserved verbatim as an accepted maintainer/native control. Deep receivers at
64 and 96 reference levels require one full candidate probe.

The maintainer `--emit-semantics`/`--count-semantics` view has four focused checks
in `dev/tests/semantic_view.rs`: canonical selection records and defaults, repeated
process determinism, count agreement, inherited Drop/const arguments, later-region
rejection and output failure propagation. It shares declaration/body inference with
the type view and reports pending lifetime constraints explicitly; complete
execution proof remains in `--emit-lowir`.

PA19 has 1,095 cases: 440 semantic inspection and 655 LowIR cases, with 262
accepted native programs and no skipped executions. Three later-stage boundaries,
one inspection-only lifetime control and three redundant-binder compiler
differences have explicit independent statuses and reasons. See
[the validation review](docs/pa19-validation.md). `make test-rustc` and the root
native target include PA19; `make test-native-pa19` selects its execution lane.
Its former associated-type and associated-constant rejections have accepted
maintainer coverage until PA20 releases its complete contract.

The partial projection maintainer controls in
`dev/tests/partial_projection_selection.rs` independently qualify 25 source
outcomes, six native programs and wide inferred inputs against pinned rustc.
`inference_changes.rs` and `scoped_projection_inputs.rs` check dependency
notifications, bounded graph work and binder-safe input commitment. These tests
do not activate the planned PA20 or PA21 grading contracts.

`dev/tests/retained_projection_results.rs` independently qualifies 80 source
controls for associated results available before all implementation inputs are
known. Eight native programs also check the resulting values and borrow behavior.
Coverage includes shared type/const result identities, competing headers, retained
formation and lifetime obligations, source-time coercion context and nested input
normalization inside callable binders. The inference tests separately check root
publication, numeric domain intersection and dependency notification invariants.
These remain maintainer coverage for the developing PA20/PA21 contracts.

`dev/tests/invariants/proof_cache.rs` also bounds solver work for recursive result
chains through depth 128, with linear bounds on both work and proof graph edges.
`projection_normalization.rs` checks linear work for appended checked subjects
through depth 512; new arguments, replacement graphs and callable binder scopes
still demand independent normalization. Reused normalized-input evidence must retain
the original lifetime equations even if its first candidate is discarded, and
fresh implementation witnesses must keep that evidence out of the cache which
does not rebase regions. `type_inference.rs` checks that identity instantiation
does constant work on a shared graph with mixed type/const parameters and nested
callable lifetimes, while later actual substitution preserves the original graph.
The canonical-premise invariants also bound canonical type growth through 512
lifetime-bearing projections without allocating new proof lifetimes for identical
clauses. Competing caller clauses, mapped lifetime environments and caller/result
equality rules must retain the full selector. The nine independently qualified
cases in `associated_declarations/canonical_queries.rs` include six native
positives, erased formation debts, missing/reversed lifetime relationships,
nonidentical result lifetimes, aliases and both kinds of associated equality.
`associated_declarations/proof_streams.rs` qualifies eight outcomes for callable
projections whose selected definitions demand nested associated constants. Four
accepted cases execute through rustc and CPPGM. An early constant argument forces
actual interleaving, while the same projected callable alias is also used in the
ordinary body. Widths 4, 16 and 48 retain two proof objects and 16, 64 and 192
callable scopes, crossing 28, 112 and 336 foreign region identities respectively.
The test requires both interleaving and linear retained scope storage; late-only
annotations cannot establish that contract. Unit tests check private ownership,
caller substitution, sibling binders, template rebasing rejection and a foreign
interval exceeding 500 million IDs that must allocate no private variables.
The 37 independently qualified cases in
`associated_declarations/field_normalization.rs` include 28 native positives for
projected nominal fields: records, tuples, enums, nested and distinct instances,
aliases, lifetime-only arguments, arrays, references, function pointers, Copy,
Drop and transparent layout. Generic constructors cover explicit and omitted
lifetimes, annotated locals and nested expected types. Negative controls retain
field typing, lifetime, variance and moved-value failures.
`invariants/stored_fields.rs` checks that raw
fields remain available after storage publication, independent callers retain
separate erased lifetime debts, and repeated shared fields take linear work
through width 256. Shared dependency recipes bound per-caller work through depth
256 and retain growing frontiers linearly through 128 distinct projected leaves.
The source-level constructor regression bounds field instantiation work through
depth 96, excluding discarded inference templates. These cases do not release PA20.

Concrete instance cleanup has a work invariant in
`dev/src/lower/instances/proof/tests.rs`: distinct callees sharing a wrapper chain
reuse the completed destruction query through depth 64. The source controls in
`dev/tests/moved_cleanup.rs` independently qualify both moved and possibly live
generic values. Proved moved roots demand no recursive cleanup properties; live
branches still query the full type. Existing template, instance, field-move and
native destruction suites check the separate permission and execution contracts.

`dev/tests/opaque_body_values.rs` separates 26 declaration/body type-checking
controls from fifteen checks through LowIR and six native callable programs.
The type-checking controls do not assert ownership, destruction or lifetime
acceptance. Independent rustc object checks qualify their language outcomes;
general opaque values still need those later gates before becoming course
fixtures. `opaque_inference.rs` checks independent occurrence authority, wakeups,
result identity and lifetime mapping. The projection cases in `instance_bodies.rs`
also forge invalid typed instances after source checking: normalized signatures
must still agree, and an erased input must retain its selected impl's lifetime
requirement. PA20 and PA21 remain planned.

The same module has thirteen LowIR controls for substituted `Copy`/`Clone`
predicate subjects: eight accept and five reject. Seven accepted programs also
execute through CPPGM and rustc; the remaining positive checks an unused generic
body. Independent rustc object checks precede these expectations. Explicit result
bounds now permit owned associated values in these controls; general opaque
destruction and projected borrows retain their separate unfinished boundaries.

`declared_builtin_results.rs` adds nineteen LowIR controls for directly declared
result abilities: nine accept and ten reject. Eight accepted programs execute
through CPPGM and rustc; one checks an unused generic body. They distinguish
Copy from Clone, preserve member/input identity and reject invalid implementations.
`declared_abilities.rs` checks immutable schema promises and agreement between
Copy, possible destruction and drop-region queries, including composite results.
Independent rustc object qualification precedes the language expectations.
`inherited_result_abilities.rs` adds 29 LowIR controls for result abilities
inherited through ordinary supertraits: seventeen accept and twelve reject.
All seventeen accepted programs execute through CPPGM and rustc. Independent
object qualification precedes the expectations. The cases distinguish direct
Self bounds from alias subjects, unrelated arguments and other associated
members; cyclic declarations and unproved implementation lifetimes still fail.
The borrowed-result case formerly stopped at the outlives boundary; it now has
LowIR and native coverage in `result_lifetimes.rs`. These maintainer tests do not
activate PA20 or PA21.

## Placement and provenance

Every `.rs` fixture under an active assignment's tests directory must be in its
manifest. The earliest assignment owning all required features owns the test.
Use feature clusters in `tests/spec/`; unchanged upstream inputs live in
`tests/upstream/PROJECT/`. Never reformat test inputs. `make test-harness` checks
discovery and oracle presence; `make test-upstream` verifies original bytes,
including the pinned Reference under `doc/rust-reference/`.

`third_party/sources.json` pins repository, commit, source path, local path,
checksum, adaptation and applicability. Its license files and notices ship
with an eventual student export. Fetching imports is explicit and is not part
of ordinary tests. Do not copy upstream `.stderr`, `.rast` or `.txt` outputs as
our oracle: their representation and diagnostic contracts belong to upstream.

## Release discipline

PA1–PA24 are implemented drafts, not a claim of full Rust frontend conformance.
Planned PA25–PA44 fail on invocation and are excluded from pass counts. Promote
a stage only after handout/reference/fixtures are coherent, independent checks
pass, the source-to-feature ledger is reviewed and all prior stages remain green.
Keep maintainer measurements and logs in ignored `obj/`; keep student handout
contracts and required fixtures tracked. Student export is not implemented yet.

The [library bootstrap plan](docs/library-bootstrap-roadmap.md) makes original
core/alloc/std and real crates required. Keep small compiler-contract text oracles
alongside deterministic whole-crate build summaries and native output checks.
Freeze source/dependency/configuration provenance and whole-library performance
budgets before release. Target libraries must compile with RustGM; separately
record any host-tool bootstrap. A reduced reproducer is not whole-crate acceptance.

During PA13 buildout, floating-pattern tests record one explicit pinned-rustc
execution discrepancy. The Reference's `expr.match.guard.next` rule retries both
matching alternatives in `-0.0 | 0.0` after a false guard; RustGM does so, while
rustc 1.98.0 (`88d9e12ae`) performs one attempt. Eight separate source pairs
record the specified and observed results across widths, orders and constant
spellings. They are excluded from same-source runtime parity claims. Other
floating-pattern vectors use the same source and assertions for both compilers.
See [the buildout evidence](docs/pa13-buildout.md#floating-point-pattern-domains).

CI compares candidate output directly with the committed oracles in `make test`
and validates fixture/manifest provenance in `make test-harness`. It does not
regenerate sidecars. A reference receipt records the compiler used to generate
that oracle; changing compiler source does not rewrite the historical receipt
when the output contract is unchanged.

`projected_static_inputs.rs` checks that completed caller proofs preserve static
implementation inputs through projected results, nested calls, function-pointer
reification and separate bodies. Its sixteen independently qualified LowIR
controls include twelve CPPGM/rustc executions and four rejections. Region-graph
invariants compare reported static equalities with every satisfying small model;
instance tests reject pre-filled resolution tables and retain forged-instance
lifetime debts. A valid stored function item whose projection still needs
normalization remains an explicit investigation outside successful coverage.
An additional generated 1,024-call regression verifies shared static extents
within the runtime region budget. Independent region-set closure tests include
reordered and duplicate seeds; a large static fanout checks bounded propagation
without changing the point sets or raising resource limits.

`result_lifetimes.rs` retains seventeen independently object-qualified associated
result lifetime controls: nine accepted programs also execute through CPPGM and
rustc, and eight invalid programs remain rejected. `outlives_verification.rs` and
`opaque_outlives.rs` check conditional proofs against independent finite-set
models, actual binder scopes and runtime extents. The original captured
function-pointer shape gap now has coverage in `captured_shapes.rs`. Projected destructor contracts remain an explicit implementation boundary.
Nominal projected-template substitution now has coverage below.

`referent_bounds.rs` adds four independently object-qualified controls for
references inside projection inputs. A locally bound function-pointer lifetime
must not erase its referent's type requirement: the unbounded case rejects,
while a static type input, an implied static referent bound and an independently
bounded result through an alias accept. The corresponding typed invariant in
`opaque_outlives.rs` also checks nested projected referents and independent result
assumptions. These are lifetime-proof checks, not additional native executions.

`captured_shapes.rs` checks seventeen independently object-qualified programs:
eleven accept, six reject, and five accepted programs also execute through CPPGM
and rustc. They cover captured trait and Sized premises, empty and nested
callable binders, mixed arguments, aliases, wrong inputs, and normalization of
Sized call obligations. Four `captured_lookup.rs` invariants check canonical
lookup, cache isolation and bounded shared-graph work. `stored_projections.rs`
adds twelve object-qualified callback programs: eight accept, four reject, and
six execute through CPPGM and rustc. `scoped_result_relations.rs` checks scoped
identity, independent lifetime equality, complete mixed inputs, local binder
isolation and a bounded 10,000-level shared graph. These close the stored sibling
callback gap. A valid locally bound projection still needs a separate callable
formation proof contract; it is not classified as a language-invalid fixture.

`field_outlives.rs` adds six typed invariants for projected nominal lifetime
contracts: complete mixed arguments through recursive uses, requested preparation,
hidden obligations, output/input separation, callable scopes, and bounded shared
and chained graphs. `field_results.rs` adds twelve independently object-qualified
source controls (nine accepted, three rejected), including three CPPGM/rustc
executions. The invalid escaping reference and output-to-input bound controls reach
lifetime proofs. General projected-field layout and trait-declared associated
lifetime bounds remain separate valid unsupported investigations. Explicit
nominal result-static predicates now have the additional coverage below.

`aggregate_bounds.rs` checks 32 independently object-qualified nominal bound
programs: 22 accept and ten reject, with nine CPPGM/rustc executions. The cases
cover complete nominal bound environments, aliases, trait-argument ordering,
defaults, closed versus generic requirements, equality-normalized storage and
short-lived Self inputs with an equality-fixed static result. Four added
`nominal_variance.rs` invariants check retained declaration fields, complete
projection inputs, variant identity, arity and a 4,096-level shared graph.
The original source probe for dependent bound roots contradicted its predicted
rustc acceptance. The separately qualified explicit-prerequisite variant now has
the additional coverage below. General closed projected lifetime requirements
without a result equality remain incomplete.

`independent_assumptions.rs` checks 38 independently object-qualified programs:
23 accept and fifteen reject, with seven CPPGM/rustc executions. They distinguish
blocked opaque alternatives from independent complete matches, preserve bound
ordering and caller scope, and retain lifetime equality obligations and ambiguity.
Four assumption-index invariants check probe isolation, unresolved inference,
the blocked-only outcome and selected evidence. The unresolved-argument invariant
now retries the same inference graph after both matching and nonmatching inputs,
in both candidate orders. Contextual trait-argument cases cover return types,
arguments, function pointers, receiver calls, mixed const arguments and distinct
callers. Missing context, incompatible results, method-name ambiguity and unpaid
lifetime debts remain rejections; selected controls check the diagnostic gate too.

The constant-contract cases in `associated_declarations.rs` check 46 object-qualified programs through
complete LowIR construction: 22 accept and 24 reject. These cover unused
required/default/override declarations, inherent constants, mixed owner arguments,
projected types, captured initializer lifetimes, covariance, invariant references,
missing bounds, invalid declaration/body types and unused non-const calls. Constructors
remain eligible; function pointers and calls in skipped branches do not. Unused evaluation faults use
explicit lint attributes so default rustc lints remain enabled. A separate typed
invariant checks that default and override identities remain distinct and that
conformance alone does not evaluate their values.

The same suite adds 39 object-qualified value/cycle controls and 17 native
programs for associated constant expressions. They cover selected defaults and
overrides, complete owner arguments, repeated instances, dependent constants,
products, references, constructors, function pointers and invalid demanded
evaluation. Trait function pointers must select overrides even inside products;
runtime recursion through a pointer is not a constant dependency cycle. A typed
invariant distinguishes erased owner arguments in the evaluated-value cache.
Two separately named boundary controls establish rustc acceptance while checking
an explicit unsupported diagnostic for trait/generic constant patterns, preventing
silent evaluation of an unselected declaration. Closed inherent patterns retain
native coverage. These maintainer checks do not constitute PA20's public release;
selected patterns, type-level uses and the complete associated-item inspection
contract remain required work.

`associated_declarations/specialization.rs` declares thirty independent lifetime
outcomes, including twenty-one native positives. It covers early-slot identity,
distinct outlives relations, selected defaults/overrides, inherited bounds,
function pointers, nested late binders, private impl witnesses and caller
isolation. All sources are checked with pinned rustc object generation; accepted
native controls execute with both rustc and CPPGM. `instance_regions.rs` also
checks early identity without caller authority, while `instance_bodies.rs`
continues to reject a forged instance with unpaid implementation lifetime debts.
`destructor_instances.rs` separately checks the nominal-to-impl capture mapping
and preserves unused Drop proof slots. The ordinary template gate and each
caller's proof remain prerequisites.

`associated_declarations/early_values.rs` independently qualifies early signature,
default-argument, generic owner and member-type dependencies, with native positive
controls and unused-invalid-member negatives. `associated_resolution.rs` and the
PA6 name fixtures check that associated values require qualification and preserve
lexical target identities. The [member lifetime tests](dev/tests/associated_declarations/member_lifetimes.rs)
add fourteen independently qualified outcomes and eight native positives for retained
member contracts: source ordering, stronger/shorter references, directed outlives
bounds, projected formation debts and nominal covariance versus invariance.
The [value readiness tests](dev/tests/associated_declarations/value_readiness.rs)
declare 22 outcomes for suspended anonymous and named initializer bodies, repeated
reference dereferences, nested field dependencies, source ordering, unused
referents, retained lifetime debts and real dependency cycles. Independent rustc
outcomes and native execution complement counters for
body work and concurrently retained node/binding slots. Grow dependency count and
depth separately from unrelated source size; retained slot capacity measures
those slot tables, not the complete inference heap.
Dispatch dependency invariants measure repeated caller
work and reject reuse across availability modes or incomplete dependency diamonds.

The [semantic cursor invariants](dev/src/sema/projections/continuation_tests.rs)
exercise selected-definition waits using retained typed declarations. Shared graphs
of 8, 32 and 128 nodes must select each projection once, tolerate repeated waiting
without new selections, and reuse completed roots with constant additional fold
work. A separate test removes the implementation registry after selection and
checks retained mixed type/const arguments. Callable tests interleave foreign
allocation intervals and require the final lifetime proof to reject unpaid
outlives bounds. These are internal scheduling/ownership invariants; they do not
claim the body worklist can yet suspend every normalization consumer.

The body integration controls in
`dev/tests/associated_declarations/definition_readiness.rs` independently qualify
source outcomes for selected definition waits, declaration ordering, partial
inference and paid/unpaid lifetime contracts, then execute accepted cases through
the CPPGM backend. Its growth controls vary definition count and repeated callers
independently. Each distinct definition must cause exactly one wait and one
selection; adding callers must leave normalization work and suspended-slot capacity
unchanged. The proof inspection checks ownership across actual foreign allocations,
and the ordinary body-work bound includes the added callers. These counters do not
measure the entire inference heap. Separate restoration invariants permit fresh
existentials only inside the retained selection proof and reject foreign identities
and streamed declaration templates. Storage-only missing-definition paths retain
positive controls for unused projected fields and unselected enum payloads.
`quantified_lifetime_proofs.rs` also checks ordinary body relations that use a
retained witness: registration succeeds, unpaid outlives debt still rejects at its
original source, an explicit caller assumption pays that debt, and a foreign
identity remains invalid.

Closed dispatch controls in
`dev/tests/associated_declarations/independent_selections.rs` declare ten outcomes:
static reuse, paid and unpaid caller lifetimes, equal lifetime arguments, default
and override constants, complete mixed arguments, local callable binders,
unanchored impl lifetimes and unused invalid result contracts. Pinned rustc
object checks qualify both acceptance and rejection; accepted programs execute
through CPPGM. The lifetime gate is exercised through full instance lowering,
including unused templates, rather than stopping after body inference.

The dispatch dependency invariants separately cover availability modes, incomplete
diamonds, universally mapped callers, caller scopes, private-argument fallback,
and inherent constants with external lifetime arguments. The definition/caller
growth controls now observe actual dispatch selections, shared lookups, expanded
closure nodes and scanned member edges in addition to the body projection cursor.
They require N+2 dispatch selections for N definitions in this family and bounded
work when both definitions and callers increase. A high cache-hit count can still
represent a quadratic traversal, so hits and edge scans have explicit bounds too.
Only completed, independently proved closures may bypass member readiness scans;
reusing an evaluated constant alone never authorizes that shortcut.

The storage continuation invariants in
`dev/tests/invariants/suspended_recipes.rs` exercise child publication while a
parent query waits, append-only recipe identities, missing nominal templates,
reference referents and failed caller proofs. Deep shared diamonds force a wait
at each distinct projected field; repeated polling must perform no additional
structural or caller proof traversal. A completed structural recipe never grants
another caller permission to use its projections.

The source controls in `dev/tests/associated_declarations/recipe_frontiers.rs`
vary fields and callers separately, checking actual declaration suspensions and
proof identity ownership before native execution against rustc. Shared recipe
count is exactly fields + 1 for this family; structural steps and retained budget
bytes have separate additive bounds. These bytes are charged storage accounting,
not allocator measurements. Caller selections still grow as fields × (callers + 1),
so these tests do not establish whole-frontend linearity.

Early invariant-input controls in `dev/tests/invariants/invariant_inputs.rs`
compare conservative hints with the full variance solver. Missing templates must
remain uncached, external nominal edges grant no hint, and repeated completed
queries must take constant work. The field-normalization source controls retain
local-lifetime inference for mixed-variance owners and reject incompatible
assignments when a projected field erases an invariant input. Full declaration
variance validation and each caller's formation/lifetime proof remain required.

The quantified input controls in
[universal_inference.rs](dev/tests/invariants/universal_inference.rs) exercise
actual selection, partial publication, header waits, foreign allocation gaps,
later private witnesses and empty quantifiers. Its body tests install whole
canonical call clauses at the declaration/body boundary and run ordinary body
inference followed by region validation. The shared
[source cases](dev/tests/harness/witness_cases.rs) are independently checked
with pinned rustc object generation by
[witness_sources.rs](dev/tests/witness_sources.rs).
They distinguish older caller witnesses from implementation-local lifetimes,
including unused witnesses and nested callable binders. These typed boundary
tests do not claim public higher-ranked source admission; PA21 remains planned.
Growing condition chains measure inference work and owned lifetimes separately,
while [refinement_universes.rs](dev/tests/refinement_universes.rs) checks shared
exported type graphs. Neither counter is a measurement of the entire compiler heap.

## Storage and cleanup

Generated output is bounded separately from validation evidence. Builds and
validation commands use `scripts/storage.py`: it checks free space before work,
serializes protected commands with cleanup, and removes disposable artifacts after
successful test or measurement batches. The local reserve is 40 GiB; when
`GITHUB_ACTIONS=true` the default is 8 GiB for its smaller ephemeral filesystem.
`RUSTGM_MIN_FREE_GIB` explicitly overrides either default. Incremental
Cargo caching is disabled in these commands.

Scripts whose disposable outputs all live in known directories can pass
`--prune-under obj/path` before `--`; repeat the option for multiple roots.
The wrapper still checks space and holds the exclusive lock, then scans only
those directories. Nested or repeated roots are scanned once. Pins in any
ancestor remain protected, and explicit roots cannot select symlinks, Git
metadata or managed caches. Missing directories may be created by the command;
the wrapper validates their paths again before cleanup. Declare every output
root and keep the default full scan for suites with scattered output. This
avoids revisiting historical evidence after a small isolated qualification run.

For direct optimized checks, use:

```sh
CARGO_TARGET_DIR=obj/rust python3 scripts/storage.py run -- cargo test --release --locked --manifest-path dev/Cargo.toml
make tidy
# Also discard the rebuildable Cargo cache when it is no longer useful:
make tidy-build
```

Cleanup retains inputs, text outputs, errors, receipts and every timing
observation. It removes generated ELF executables and object/metadata files,
along with rendering scratch files; failed wrapped runs retain their artifacts
until explicitly pruned. `python3 scripts/storage.py prune` previews deletion.
Deletion records live in `obj/cleanup/`. Cleanup never follows directory/file
symlinks or enters another checkout, and it refuses live RustGM executables.

Pin the current immutable A/B archive directories with `.keep-artifacts` before
running measurements through the wrapper. The source, binary and input hash
checks still happen before cleanup, with no pruning during the timed command.
Retire old pins after replacing the baseline. Historical binary copies may be
pruned; their recorded hashes and observations remain, but rerunning a measurement
requires a restored binary whose hash matches. Old path references in historical
validation notes do not imply that disposable executables still exist.

Retiring a pin also requires an explicit scoped prune:
`python3 scripts/storage.py prune --apply --prune-under obj/old-comparison`.
Preview without `--apply` when identifying obsolete outputs, and check the
deletion inventory against the current frozen input and binary hashes. Automatic
successful-run cleanup only considers files written during that run, so it will
not reclaim an older executable merely because its pin was removed. Keep the
reserve unchanged when reclaiming space, and discard rebuildable Cargo caches
with `make tidy-build` when their compiled snapshot is no longer needed.

PA20 introduces `--emit-trait-semantics` and `--count-trait-semantics`, preserving
PA19's separate semantic text contract. Its 1,355 registered cases cover 546
semantic inspections and 809 LowIR checks, with 489 independent native executions.
One explicitly classified semantic-positive case defers an escaping-reference
error to the complete proof. See `docs/pa20-validation.md` for the seven families,
source provenance, count/row checks and reference review. `make test-native` and
`make test-rustc` include PA20; PA25–PA44 remain planned and fail explicitly.

Parameter-default checks use their own lifetime eligibility rule, separate from
closed definition-time bounds. The default-clause invariants retain unused
quantifiers through substitution and final scoped proof. Their source hypotheses
are independently qualified; they do not open PA21's higher-ranked source gate.
The PA18/PA19 default-requirement fixtures cover written static lifetimes, nested
callable lifetimes and actual use. PA20 additionally checks lifetimes erased or
introduced by projection normalization, using the retained original bound for
eligibility and the normalized clause for parameter indexing. Its positive and
negative controls run through full lowering and independent native qualification.

The shared parameter-occurrence invariants vary prefix depth and distinct bounds
sharing a wide argument list. They count actual query work and retained summary
entries, including constants, equality results and non-injective projection inputs.
These are bounds on the index, not on whole-frontend work or total heap usage.


PA21 adds `--emit-generic-semantics` and `--count-generic-semantics`, preserving
PA19/PA20 inspection vocabularies. Its 744 cases contain 298 semantic views and
446 LowIR checks, with 260 declared native executions: 256 pass and four expected
destructor panics. Every source is object-qualified with pinned rustc. The seven
families cover quantified scopes/contracts, complete GAT applications, opaque
identity/capture, variance, destruction and integration. Run `make test-pa21` and
`make test-native-pa21`; root report, independent and native targets include it.
The full release evidence and the one reviewed PA20 lifetime diagnostic update
are documented in [docs/pa21-validation.md](docs/pa21-validation.md).

The three new PA16 wildcard controls distinguish type liveness from value reads,
initialization and movement. The expired-reference mention rejects; an unused
reference and an uninitialized wildcard mention compile. Opaque variants belong
to PA21. Independent-source outputs now use `.o`/`.rmeta` suffixes so the shared
storage guard removes them after successful runs.


PA22 publishes 356 LowIR cases: 254 accept, 102 reject. Every source is independently
object-qualified with pinned rustc. Its 120 runtime comparisons include five
expected panics; 134 compile-only controls are reported separately. Root report,
source and native targets include PA22. The 28 added reference-unsizing cast cases
retain all 328 earlier cases and their exact byte oracles.
See [docs/pa22-release.md](docs/pa22-release.md) for qualification and performance.


PA23 publishes 400 LowIR cases: 263 accept and 137 reject. Pinned rustc
independently qualifies every declared source outcome. The native lane has
86 execution comparisons and 177 explicit compile-only controls. Its matrix
covers object annotations and lifetimes, dyn compatibility, quantified and
associated contracts, dynamic dispatch, upcasts, tails, marker implementations
and defining-body dependencies. The former PA19 future-object boundary moves
here with unchanged source bytes. One PA21 quantified forwarding proof now
uses the resumable canonical proof representation; its source, result and
LowIR behavior remain unchanged. The release evidence documents both changes.
Root source/native targets include PA23. See `docs/pa23-release.md`.

Focused native case selection uses the same exact manifest paths as text checks:

```sh
python3 scripts/resources.py run --max-output-gib 16 -- python3 scripts/storage.py run --prune-under obj/native/pa24/selected -- python3 scripts/check_native.py pa24 --app /immutable/qualified/rustgm --jobs 2 --case tests/spec/object-input-context/mutable.rs
```

Repeat `--case` to select related LowIR cases. Empty, unknown, duplicate and
non-LowIR selectors fail. A selected receipt records both its paths and the full
eligible count, under a unique `obj/native/STAGE/selected/` directory; it never
replaces complete-suite evidence. Full assignment releases still run the complete
native contract. `--released-units` cannot be combined with case selection.
