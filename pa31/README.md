# RustGM Programming Assignment 31: Compiler/library bindings

Status: **planned**. This is a buildout brief, not a released grading contract.
Work prerequisites: [ordered dependency units](../docs/assignment-order.md). The source profile is Rust 1.98.0, initially edition 2024
and Linux x86-64. PA34 adds Rust 2021; PA40 closes the remaining editions.
See the [library bootstrap plan](../docs/library-bootstrap-roadmap.md) for
implementation priority, source pins and the migration of unreleased numbers.

## Dependency boundary

The `source-declarations` unit establishes local identity, signatures and
attribute/feature authority before PA25/26/28 consumers. It excludes runtime
intrinsic, formatting and panic execution. The separate `source-runtime` unit
consumes those actual producers. Moving all of PA31 earlier would recreate the
old cycle; both units and the complete contract below remain required.

## Required implementation

Register source-declared language items, diagnostic items and intrinsic
identities through validated attributes and ordinary crate ownership. Validate
signatures, generic parameters, duplicate/missing items and feature gates. One
typed registry must drive semantic selection, const eligibility, capability
queries and lowering; path spelling alone grants no compiler privilege.

Inventory the active compiler contract of Rust 1.98.0 source revision
`88d9e12ae178fab0fb5cc050a94da85685d449ea`: `no_core`, internal attributes,
builtin macros/derives, layout properties and unstable language features.
Classify attributes by semantic effect; never silently ignore an active semantic
requirement. In this revision, `SizedTypeProperties` supplies layout constants
and atomic aliases use `Atomic<T: AtomicPrimitive>`; test the real protocols.
Authenticate builtin declarations separately from invocation availability. An
unused known declaration retains its actual macro kind without granting an
unimplemented expansion permission. Unknown claims and wrong-kind invocations
still fail. The [live original-library tracker](../docs/library-build-blockers.md)
records current hard source boundaries and qualified checkpoint scope. Preserve
real diagnostic policy while using explicit capped observations to prioritize
compilation; deferred advisory producers and strict qualification remain required.

Source-derived impl headers preserve parsed type/lifetime/const formals and
where constraints, omitting declaration defaults from impl parameters. The
[bounded derive checkpoint](../docs/pa31-bounded-derive-transparency.md) retains
the real ManuallyDrop source and its remaining full-runtime requirements.
`rustc_pub_transparent` requires authenticated internal-attribute permission,
word form, a unique active owner marker and checked `repr(transparent)`; it does
not expose fields. Its external private-field eligibility must survive metadata
before transparent source types can cross that boundary.

An authored `prelude_import` requires its distinct crate feature and actual use
owner. Select the genuine module separately from ordinary lexical globs; preserve
lookup precedence, private binding lineage, hygiene and `no_implicit_prelude`,
including actual trait-method discovery. Compiler builtins require authenticated
identity. The [source prelude checkpoint](../docs/pa31-source-prelude-integration.md)
records the finite local/native profile. PA34 owns automatic edition-specific
core/std prelude injection; explicit authored paths are not rewritten by edition.

Library feature requests also depend on enabled local/dependency stability
declarations. Implement their inventory, regular/const API checks and macro
allowance provenance; a compiler feature whitelist cannot replace them. The
[stability investigation](../docs/pa31-library-stability.md) supplies independently
qualified seed cases. The [local stability candidate](../docs/pa31-local-stability.md)
implements local declaration ownership, feature inventory and separate regular/
const policy for the selected source Location consumer. Dependency/default-body,
complete public effective visibility and foreign policy transport remain required.
Actual declaration/const/origin sizes and native scope are recorded in the
[source assertion milestone](../docs/source-assert-implementation.md#source-location-derive-consumer).

The [offset_of source contracts](tests/spec/source-offset-of/README.md) retain the
exact pinned mem macro and independently observed generic enum/niche behavior.
Builtin syntax uses its real name-token authority, including hard gates after cfg
removal. Enum offsets separately require the actual crate feature. Preserve typed
container/field captures, privacy and substituted member identity; defer generic
inline constants until concrete layout is available. Distinct fields of the same
callable instance require distinct reached-call operands, so the size/alignment
result cache cannot store field offsets. DST/static-alignment, original runtime
consumers and intrinsic/source transport remain part of the complete requirement.

Define explicit source-sysroot selection and crate identity. Replace bootstrap
library factories with real declarations without duplicate identities or a
synthetic fallback. Ordinary Option/Result/Iterator methods belong to upstream
core. PA26 retains ownership of desugaring, derives and formatting semantics;
this assignment connects them to declarations loaded from source.

Bind Tuple, FnOnce and its associated Output declaration before admitting
intrinsics whose signatures depend on them. Preserve rust-call tuple spreading
and ordinary versus const-callability obligations. Qualify const_eval_select
with source, ownership and native controls; accepted runtime function pointers
and upstream compiler ICEs must not be mislabeled as language rejections. See
the [pinned callable evidence](../docs/library-bootstrap-inventory.md#callable-selection-followup).

Establish the minimal panic-abort entry/runtime boundary and no_core declaration
packages. Keep general unwinding at PA39. Host/build configuration lives outside
the frontend library; no environment reads enter semantic hot paths.

The [selected source handler checkpoint](../docs/pa31-source-panic-handler-checkpoint.md)
executes genuine Location/PanicInfo/panic/handler bodies through LowIR and the
native backend, with normal/drop and handler/abort traces. Its selected local
profile does not complete full Formatter support, metadata interoperability or
the original core/alloc/std artifact requirements.

The maintainer implements compiler responsibilities in `dev/src/` and introduces
typed interfaces before text views. Follow [spec.md](../spec.md), the pinned Rust
Reference and CPPGM's efficient representation principles. Parse once, retain
completed facts and avoid speculative or repeated whole-graph work. The compiler
implementation remains idiomatic safe Rust; strict Clippy all/pedantic and the
syntax-aware file/architecture audit are required.

The [source-artifact/readiness checkpoint](../docs/source-artifact-readiness-checkpoint.md)
adds actual imported-body/full-instance proof, early source implementation
readiness and the original reference-valued anonymous selector's native `7 1`.
This is a bounded implementation checkpoint. Provided/default body environments,
generic/imported early callers, actual UB policy and attribute consumers remain
required before the original library and assignment can complete.

## Tool surface

Extend the existing `dev/rustgm --emit-lowir` view and explicit crate-build interfaces.
Earlier views remain available. Freeze deterministic ordering, diagnostics,
source/configuration identities and any new build-summary text contract before
activation. Whole-library acceptance includes actual linked execution through
the configured CPPGM backend; text output alone is insufficient.

## Tests to develop with the reference

- Lang-item identity and signatures.
- Intrinsic declarations and const eligibility.
- Builtin expansion and capability answers.
- Source panic/unreachable builtin delegation: authentic defining-crate identity,
  unchanged argument tokens, edition selection through internal macro allowances,
  and opposing imported-wrapper editions. The
  [predeclared source contracts](tests/spec/source-panic-macros/README.md)
  reproduce the preceding original-core declaration blocker. The
  [combined checkpoint](../docs/library-lookup-panic-checkpoint.md) qualifies
  local 2021/2024 delegation; foreign transport and Rust 2018 remain required.
  The combined allowance successor passes `assert_eq` and reaches the scoped
  `assert_matches` declaration, line 170. The qualified scoped-declaration
  component reached builtin `cfg_select`, line 235. The qualified cfg_select
  successor reaches the `format_args` builtin attribute at line 994. The combined
  formatting successor reached `concat_bytes`, line 1126. The qualified
  [scoped/byte successor](../docs/scoped-bytes-checkpoint.md) reaches
  `autodiff_forward`, line 1553. Stricter attribute checking separately exposes
  the earlier primitive coherence policy at `core/src/lib.rs:65`; implement its
  real semantics. Track current probe evidence in the
  [library blocker tracker](../docs/library-build-blockers.md).
- Formatting claims and calls must use actual source nominal and constructor
  identities, with ordinary signature, lifetime, visibility and const checks.
  The [formatting contracts](tests/spec/source-format-macros/README.md) cover
  declarations and literal-only calls. Nonempty arguments, evaluation order and
  temporary lifetimes remain required. Byte concatenation owns PA8's builtin
  semantics and PA31's source binding; retain exact typed operands and rejection
  rules when implementing the newly observed `concat_bytes` blocker.
- Scoped member identity and actual lookup containers. Preserve the method
  identifier independently of trait binding names. An item or statement macro
  can create a block boundary even if expansion leaves it empty; parenthesized
  expression tails retain their expression category. The independent
  [scope contracts](tests/spec/declarative-macros/method-scopes/README.md) include
  caller/definition imports, ambiguity and captured-name opponents.
- Implementation members must map to their actual trait declarations after
  identifier adjustment. Reject duplicate canonical mappings across different
  source contexts and preserve namespace/member-kind distinctions. The
  [conformance contracts](tests/spec/declarative-macros/member-conformance/README.md)
  cover methods, associated types and constants, including captured identifiers.
- Duplicate and forged bindings.
- Panic-abort runtime boundary.

Place each fixture in the earliest assignment owning all features it exercises.
Keep original upstream bytes, attribution and checksums. Declare expected outcomes
before generating oracles; use pinned rustc independently and retain negative
controls. Register the complete selected dependency graph and native consumers,
not just reducers or successful submodules. Never hand-edit generated references.

Trait aliases needed by original source, including `ptr::Thin`, require their
actual declaration identity, exact feature/token authority and complete
substituted predicates/associated equalities. Reuse PA5 grammar and PA21's typed
bound machinery. Diagnose cycles, invalid impl owners and object restrictions
after expansion. Preserve disabled-syntax warnings and generic/where-clause
scope. The [planned source contracts](tests/spec/trait-aliases/README.md) retain
pinned acceptance/runtime observations; they do not activate this assignment.
Imported alias contracts need genuine PA29 transport, with no ordinary-trait
substitution or permission inferred from parsing.

## Completion and review

Publish the semantic/lowering boundary, nonempty `tests/manifest.json`, text
formats and generated references together after independent outcome review.
Until then, this assignment must fail explicitly and stays outside pass totals.
Run affected focused source/native controls during development. At publication,
run the required coherent checkpoint checks from
[TESTING_AND_REFERENCES.md](../TESTING_AND_REFERENCES.md), including prior active
suites, strict Clippy, file/architecture audits, harness checks and affected
performance measurements. Use the resource/storage guards and scoped cleanup.
Do not activate a stage merely because a reduced example or an empty suite passes.

The focused [callable const eligibility checkpoint](../docs/pa31-callable-const-eligibility.md)
records independently qualified source contracts and the remaining original
const-callable execution requirement; PA31 remains planned.

The [runtime callback checkpoint](../docs/pa31-runtime-const-select.md) records
actual evaluated G selection, owned tuple transfer, constrained value origins
and the separately retained CTFE/full-staged-provider boundaries.

The [source Add/Sub checkpoint](../docs/pa31-source-add-sub.md) joins genuine
operator headers, defining const member requests and actual selected source
body/Code proof. Full forwarding macro CTFE and three source/native consumers
pass without bootstrap factories. Dynamic checked macro panic closure, broader
const/default/imported calls and whole assignment release remain required.

The [finite lint expectation prerequisite](../docs/pa40-lint-expectations-checkpoint.md)
provides genuine deprecated macro reexport controls for the source prelude.
The original prelude's later `hidden_glob_reexports` expectation and the other
original source lint producers remain explicit requirements. The authored
reducers do not establish a whole original crate build.

The unpublished [feature-predicate investigation](../docs/pa31-feature-bound-predicates.md)
records genuine trait/function/impl requirements and 78 independently joined
source/profile metadata outcomes. Typed proof, source production, transport and
native implementation remain required; diagnostic caps grant no feature permission.

The [finite source exact-C linkage checkpoint](../docs/pa31-source-exact-linkage-v5.md)
adds an unpublished version5 source profile with paid exact symbol transport and
separate provider/client native objects. Original nominal source transport,
ordinary-provider publication and whole assignment/library artifacts remain required.
