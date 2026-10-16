# Testing and references

This checkout owns the maintainer solution. `dev/rustgm-ref` invokes the built
solution, just as reference wrappers do in cppgm-extended's solution checkout.
Default tests compare candidates with **committed text**, not a live solution.

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

Only the explicit `ref-test` targets regenerate sidecars. The generator first
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

PA9 has 201 full-program type-view fixtures. Its rustc checks retain default
literal lints. Four manifest entries explicitly defer initialization/write
permissions, lifetime proof or constant-call validity to the later owning
assignment. The type view retains those facts without claiming those obligations
are proved. CLI tests check deterministic output, count/full-view agreement and
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

PA18 has 438 generic LowIR cases: 273 accepted and 165 rejected. There are
270 executable comparisons and three deliberately divergent codegen-only
controls. Their `native.kind: "codegen"` requires a nonempty reason and forbids
runtime outcome fields; both compilers produce native code, but neither binary
is executed. Counts report these separately. All other accepted LowIR cases
retain the mandatory execution/outcome contract.

An explicit `rustc.emit: "obj"` selects object generation in the independent
acceptance checker. Growing-instance rejection and demand-only controls need
that stronger check because metadata emission does not monomorphize bodies.
It is valid only for complete programs. One PA18 rejection fixture declares
a later PA21 boundary accepted by rustc; there are no new runtime differences.
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

PA19 has 1,069 cases: 439 semantic inspection and 630 LowIR cases, with 247
accepted native programs and no skipped executions. Four later-stage boundaries,
one inspection-only lifetime control and three redundant-binder compiler
differences have explicit independent statuses and reasons. See
[the validation review](docs/pa19-validation.md). `make test-rustc` and the root
native target include PA19; `make test-native-pa19` selects its execution lane.
Its former associated-type rejection likewise moves to accepted maintainer
coverage until PA20 releases its complete contract.

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
bounds now permit owned associated values in these controls; declared result
abilities and projected borrows retain their separate unfinished boundaries.

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

PA1–PA19 are implemented drafts, not a claim of full Rust frontend conformance.
Planned PA20–PA38 fail on invocation and are excluded from pass counts. Promote
a stage only after handout/reference/fixtures are coherent, independent checks
pass, the source-to-feature ledger is reviewed and all prior stages remain green.
Keep maintainer measurements and logs in ignored `obj/`; keep student handout
contracts and required fixtures tracked. Student export is not implemented yet.

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

## Storage and cleanup

Generated output is bounded separately from validation evidence. Builds and
validation commands use `scripts/storage.py`: it checks free space before work,
serializes protected commands with cleanup, and removes disposable artifacts after
successful test or measurement batches. The local reserve is 40 GiB; when
`GITHUB_ACTIONS=true` the default is 8 GiB for its smaller ephemeral filesystem.
`RUSTGM_MIN_FREE_GIB` explicitly overrides either default. Incremental
Cargo caching is disabled in these commands.

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
