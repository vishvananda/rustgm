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

PA8 checks all 93 expansion/hygiene programs with pinned rustc. A case may supply
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

PA11 has 194 scalar LowIR fixtures, including 180 executable comparisons with
pinned rustc and the qualified CPPGM backend. `make test-native` is required
locally; ordinary hosted CI has no qualified backend binary and checks the
committed text, frontend invariants and harness instead. Compiler signals and
exit 86 always fail. Explicit runtime-panic cases require CPPGM exit 134 with
empty streams and rustc SIGABRT with a panic diagnostic. Success requires clean
exit zero; source assertions compare full values. No native outcome is inferred
from a text-oracle match. See `docs/pa11-validation.md`.

Rustc has no LowIR output. From PA11, maintainer-generated LowIR is the course
text oracle, with independent run-pass checks through CPPGM versus rustc. Do
not grade LLVM IR or assume Rust's unspecified layout matches rustc. Test exact
layouts where guaranteed or where the course contract deliberately specifies
them. Keep runtime evidence separate from the raw LowIR text comparison.

PA12 has 109 fixtures: 40 clean native executions, eight expected bounds panics,
12 layout cases and 49 rejection or explicit stage-boundary cases. The manifest
declares `--emit-layouts` as a supplementary mode; a case's mode must be one of
the assignment's declared modes. `make test-layouts` checks declared size,
alignment and field-offset vectors against candidate/committed text and rustc
probes where guaranteed. Course Rust field order and transparent ancillary ZST
offsets have explicit exceptions. `make test-rustc` includes these layout checks
and PA11/PA12 acceptance checks, so hosted CI exercises them without a backend.
`make test-native` qualifies both PA11 and PA12; `make test-native-pa12` selects
the aggregate suite. Layout cases never count as native execution. See
`docs/pa12-validation.md` for the profile and retained evidence.

PA13 has 803 fixtures: 538 LowIR cases and 265 selected-pattern inspections.
All have pinned-rustc acceptance checks; thirteen explicit qualifications cover
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

PA1–PA14 are implemented drafts, not a claim of full Rust frontend conformance.
Planned PA15–PA38 fail on invocation and are excluded from pass counts. Promote
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
