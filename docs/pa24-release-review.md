# PA24 release review

This review consolidates the PA24 closure contract and release evidence;
historical checkpoint documents describe their own snapshots.
The required feature set has not been reduced to the passing inventory.

## Contract and coverage

The source profile is Rust 1.98.0, edition 2024, Linux x86-64, checked integer
overflow and aborting panics. The grading surface remains `--emit-lowir`, using
the existing deterministic LowIR vocabulary, typed calling convention and CPPGM
backend. Earlier inspection modes retain their existing proof boundaries.

| Required behavior | Owning fixture families and additional proof controls |
| --- | --- |
| Distinct anonymous types, contextual inference, generic inputs and nested bodies | `generic-context`, `generic-captures`, `initializer-owners`, `nested-captures`; closure type and inference invariants |
| Typed capture paths, prefix merging, shared/mutable/unique/value permissions | `captures`, `typed-capture-uses`, `typed-capture-native`, `adjusted-places`, `capture-copy-shape`; indexed capture forest invariants |
| Field precision, wildcards, discriminants, shared-reference truncation and source formation | `captures`, `formation-truncation`, `capture-inputs`, `pattern-policies`; source-condition and exact bridge-seal controls |
| Actual environment layout, Copy/Clone/auto-trait queries, moves and destruction | `captures`, `generic-captures`, `nested-captures`, `output-interfaces`; closure property and ordinary ownership/drop invariants |
| Fn/FnMut/FnOnce hierarchy, complete inputs/Output and receiver adjustments | `callable-dispatch`, `receiver-adapters`, `receiver-binding`, `execution-self`, `generic-receivers`, `nested-receivers` |
| Noncapturing function-pointer coercion and opposing captured/unsafe cases | `closure-pointers`, `reference-conversions`; callable declaration and selection invariants |
| Borrowed callable objects, quantifiers, projected inputs and output lifetimes | `erased-callables`, `captured-dispatch`, `object-input-context`, `pending-object-inputs`, `projected-captures`, `quantified-dispatch`, `reference-objects`, `signature-slot-ownership` |
| Original and demanded-body proof, returned references and unused children | `output-interfaces`, `formation-truncation`, `capture-inputs`; nested evaluation and original-template proof tests |

Capture fields determine storage and applicable structural abilities; unused
signature or owner parameters do not become stored fields. A move closure can
still implement Fn or FnMut. Shared, mutable and consuming adapters retain the
actual receiver permission. Quantified public slots, enclosing captures and
body-local lifetimes retain separate ownership through inference and proof.
Object or callable selection alone cannot discharge the source lifetime contract.

Packed/raw operations with their additional validity requirements are qualified
at PA27; owning library pointers and additional library methods belong to PA26.
Async closure execution belongs to PA32/PA33. Cross-crate transport of exposed
closure and opaque types belongs to PA29. These boundaries do not excuse a
failed ordinary closure proof or a missing PA24 capture/call operation.

## Frozen inventory and oracle review

Compiler snapshot `1b93560f110c67c706a69c6748538289ab3bb028`, default-feature
release binary SHA-256
`e6308578034792cd915c95a7564f96abbbd3cb5b13284a6056c06eeac9dd1db2`,
qualified all 467 existing PA24 source files without rewriting them. Their
previous independently declared Rust outcomes consolidate to 323 accepted and
144 rejected sources. Fifteen inputs are complete entry-free libraries, of
which ten accept and produce independently validated relocatable objects.

The first complete inventory executed all 313 accepted programs with rustc and
RustGM/CPPGM. Review identified two intentionally unused-body controls with empty
entrypoints. Their final contracts are code-generation-only, leaving 311 runtime
comparisons, two code-generation controls and ten object controls. The revised
contracts passed the independent source/native checks again before generation.
The `deny(unreachable_patterns)` library input was independently confirmed with
default lints enabled; no synthetic main was added.

Evidence is in `obj/parallel/pa24/release-inventory-v1/` and
`obj/parallel/pa24/release-oracles-v1/`. The latter contains generated candidate
sidecars and their compiler/fixture/case-contract provenance, followed by a second
467-case byte comparison. Those oracles are staged for review, not installed in
the course. Successful LowIR cases have empty diagnostics; the 144 rejections
have empty output and exit 1.

Review found one invalid semantic inference in this otherwise passing inventory:
`output-interfaces/escape-outer.rs` was rejected by the scalar metadata export
profile before its ordinary lifetime proof. Its source and language expectation
remain unchanged. The library driver now performs the original body proof before
export selection, using the same retained templates and proof pass. Both CLI and
typed API regressions require the actual lifetime failure, and metadata must not
be published. A valid public opaque result still retains its separately owned
PA29 transport boundary.

The corrected compiler independently requalified all 467 source outcomes and
their native classifications before generating `release-oracles-v2/`. Review
found exactly one changed sidecar, this case's diagnostic: the metadata-profile
error at byte zero becomes `required lifetime does not outlive its target` at
byte 47. Source bytes, declared outcomes and the other sidecars are unchanged.
Generation and a subsequent byte comparison both passed. These reviewed
sidecars remain staged; they have not activated PA24.

The correction's default-feature binary SHA-256 is
`ece395550188b349a3d9dae9b98b03616d7b362334c5179d8bbd7d6c5872e80a`.
`obj/parallel/pa24-library-proofs/qualification-v2/` records the complete
crate-linking, library-root and nested-evaluation tests, including their native
controls. Its subsequent audit rejected a test's compile-time fixture include.
The test now reads the same course source through the established runtime fixture
loader pattern; production sources were unchanged. `qualification-v3/` binds that
single test change to the earlier evidence and records the affected regressions,
formatting, strict all-target/all-feature release Clippy, file/architecture and
layout audits, and all 7,787 PA1–PA23 text outcomes passing. The corrected compiler
still requires the next combined full publication qualification.

The combined compiler at `0b89bf31` also includes the checked owned Ord helpers,
canonical Ordering comparisons and scoped unaligned transfers. Its focused
qualification at `obj/parallel/pa24-pa28-integration-v1/` passed formatting,
strict all-target/all-feature release Clippy, file/architecture and layout
audits, 90 private/affected tests including native controls, every PA1–PA23 text
case and all 467 corrected staged PA24 oracles. This establishes integration
compatibility; it does not substitute for the full publication suite.

The earlier frozen compiler's full release Cargo run exposed three stale test
expectations: an unqualified Ordering spelling was assumed unique, compressed
immutable reads reduced an exact allocation count, and a slice-indexing control
still expected its former unsupported result. Test-only commit `6e88cdb6`
preserves the qualified type-identity checks, retains an allocation ceiling with
an unrelated-type independence check, and adds a native value check for the
accepted slice case. Production sources are unchanged from `1b93560f`. The
repaired full release Cargo run passed 2,316 tests and its native maintainer run
passed 219 tests. The affected independent source/native checks also passed;
their receipts are in `test-repair-v2/` under the frozen sequence qualification.
The completed 45-input comparison is documented in `pa24-sequence-performance.md`.
This checkpoint was published as `6e88cdb6`; it does not qualify the later
combined compiler.

The PA24 release candidate is separately frozen at `c7fe46c0`, whose compiler and
tests are unchanged from the qualified `0b89bf31` integration above. The changes
since then are documentation and verified diagnostic archival. Its final functional
qualification completed under `obj/parallel/pa24-release-final-v1/` in the validation
worktree. All 43 commands passed: 2,329 full release Cargo tests, 222 native
maintainer tests, imported rustc/literal/layout checks, every earlier independent
source and native suite, and the complete staged PA24 source/native/text suites.
PA24 retains all 467 source outcomes, 311 executable comparisons, two code-generation
controls and ten library object controls. Its default-feature binary SHA-256 is
`c408b1205c7815a004309b471ee115db532c7dd3c3a070c0ca2e2904d858cd10`.

`retained.json` binds the exact unchanged compiler/tests to the prior formatting,
strict all-target/all-feature release Clippy, architecture/file/layout audits,
90 focused tests and all 7,787 earlier and 467 PA24 text comparisons. It also
binds the diagnostic-cleanup change to its independently passing 81-test harness
batch. Resource receipt `dc40d84695104c9784a9649a442f5eaf` closed with status zero.
Post-run cleanup removed 12,098 generated files (12.54 GiB of unshared blocks)
and losslessly archived one large type diagnostic, preserving source inputs,
observations and immutable pins. Later PA25 development remains outside this
frozen candidate and does not inherit its qualification.

The final 18-input changed-path comparison also completed under an exclusive
lease. All 1,404 timed executions preserve their qualified outputs; shared
instruction changes range from -0.174% to +0.951%. Timing noise prevents a small
speedup claim. See [the final measurement review](pa24-final-performance.md) for
the full method, independent library/program classifications and observations.
The earlier 45-input comparison and its open receiver-depth issue remain part
of the performance review.

## Publication checks

The reviewed v2 handout, manifest, 1,401 generated sidecars and provenance are
installed together with assignment status and root text/source/native selection.
The release-install receipt binds every payload file to its reviewed staged hash.
Post-install layout, harness and owning-target validation must complete before
the public main fast-forward. A planned assignment must not report an empty-suite
pass.

PA25–PA30 keep their full independent responsibilities and queued work. This
review supplies no evidence that those later stages are complete.
