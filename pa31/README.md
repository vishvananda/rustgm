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

The [advisory source checkpoint](../docs/pa31-advisory-source-checkpoint.md)
validates actual insignificant-destructor declarations and the independent
optimization restrictions without inventing write, cleanup or language
permission. Current semantic consumers retain real destruction and mutable
stores; broader advisory lint parity remains deferred under the compilation-first
plan. The next unchanged-core source marker is tracked separately from releases.

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

The [source Writable-suppression contract](tests/spec/source-no-writable/README.md)
validates actual `rustc_no_writable` word/gate/owner/duplicate semantics. The marker
suppresses an optional LLVM argument optimization; it grants neither write nor
unsafe/const/ABI permission. Current local and imported executable bodies emit
no such assumption. A future optimizer must preserve authenticated intent across
imports or remain conservative before using the assumption. The source draft has
independent Rust evidence and qualified local/native behavior. Broader imported
optimizer intent and original library graphs remain required.

The [local legacy const-generic contract](tests/spec/source-legacy-const/README.md)
validates actual free-function generic headers and index lists. It does not
rewrite same-crate calls or grant const/unsafe permission. Current metadata
profiles fence the required external index/call protocol; Rust-positive external
legacy calls remain declared requirements in PA29. Local source/signature,
same-crate modern calls and native behavior are qualified; the external protocol
remains incomplete.

The [const-item advisory contract](tests/spec/source-const-item-calls/README.md)
validates actual inherent and trait-impl methods without changing const, unsafe
or mutation legality. Its generated meta attribute allowance follows the real
attribute envelope; forwarded whole items retain caller provenance. Local/native
behavior is qualified, while the broader advisory lint producer remains deferred.

The [confusable-name contract](tests/spec/source-confusable-methods/README.md)
and [suspension-advisory contract](tests/spec/source-suspend-advisory/README.md)
have independent160/160 Rust outcomes and joined candidate qualification of all80
sources. The [joined checkpoint](../docs/pa31-cell-advisory-checkpoint.md) records
actual mutation, const and destruction native results and the one-pass transient
annotation frontier. Confusables now clears the original Cell source boundary;
core next reaches an ABI-affecting attribute in ffi/va_list.rs (PA30/LIB-111).
Broader diagnostics, async ownership, imported intent and original artifacts
remain required.

The source `rustc_promotable` declaration checkpoint retains its actual attribute,
configured owner and direct const-stability witness. Word, list and value forms
follow the pinned parser's marker semantics. This header promise grants neither
const/unsafe permission nor permanent storage: marked call borrow promotion and
foreign marker transport remain explicit required work. Required CTFE and ordinary
calls continue through their real source bodies. The
[`source-promotable` contract](tests/spec/source-promotable/README.md) keeps these
boundaries separate.

Source-derived impl headers preserve parsed type/lifetime/const formals and
where constraints, omitting declaration defaults from impl parameters. The
[bounded derive checkpoint](../docs/pa31-bounded-derive-transparency.md) retains
the real ManuallyDrop source and its remaining full-runtime requirements.
`rustc_pub_transparent` requires authenticated internal-attribute permission,
word form, a unique active owner marker and checked `repr(transparent)`; it does
not expose fields. Its external private-field eligibility must survive metadata
before transparent source types can cross that boundary.

The [source union Copy checkpoint](../docs/pa31-source-union-copy-checkpoint.md)
reuses genuine generic headers, source Copy premises and unit marker proof.
Physical field and Clone checks remain required; the generated empty Copy impl
never reads a payload. Original MaybeUninit expansion and independent native
copies pass, while whole original runtime/validity/metadata and the separate
source union Clone recipe remain required. The next original expansion blocker
is the Debug field-array helper recipe for the seven-field FnPtr.

The [closed scalar-pattern checkpoint](../docs/pa31-closed-scalar-patterns-checkpoint.md)
qualifies actual range formation, named typed constants, validity and source
Clone/Copy/native execution for the Nanoseconds profile. Retain the actual base
representation and complete source constraints. Open/exclusive and signed
alternative forms require the real RangeMin/RangeMax/RangeSub declarations,
selected source constants/bodies and authenticated internal desugaring allowance.
The [unsigned zero-niche checkpoint](../docs/pa12-unsigned-niche-checkpoint.md)
covers the finite closed unsigned `1..=MAX` carrier through nullable layout, source
constructors/accessors, CTFE bytes and scalar/C transport. Its authored range
invocations and minimal Option/Drop protocols remain distinct from the complete
original NonZero/ZeroablePrimitive/Option/Drop graphs. Generic endpoints, partial/
signed/char niches, refinement metadata and complete original methods remain
requirements of the owning assignments.

The [source range member checkpoint](../docs/pa31-source-range-members-checkpoint.md)
registers actual associated identities and completes their shared const-trait
declaration contract. Prepare genuine MIN/MAX types and Sub signature first;
preserve partial-family pending state and source/Store incarnation checks.
Declaration knowledge grants no selected implementation, body, API or const
permission. Source helper execution and metadata transport remain required.

The [source endpoint execution checkpoint](../docs/pa31-source-endpoint-execution-checkpoint.md)
qualifies authored MIN/MAX/Sub dispatch through typed declaration continuations,
genuine caller-location data and actual method/constant conformance. Its native
observer matches pinned Rust; original integer/char/panic helper bodies, generic
endpoints and metadata remain required.

The [original NonZero method checkpoint](../docs/pa28-source-unchecked-transmute-checkpoint.md)
executes unchanged new/get through actual ZeroablePrimitive projection and
source TransmuteUnchecked. Its closed u8 inner foundation is explicit; original
open helpers, full inner/library graphs and metadata remain required.

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

Ordinary `rustc_nounwind` functions use their real source bodies and the checked
abort/non-unwinding ABI profile. Preserve actual word/internal-feature, const
and unsafe checks; the marker grants no intrinsic or callable permission. The
[finite checkpoint](../docs/pa31-ordinary-nounwind-checkpoint.md) leaves PA39's
unwind boundaries and the original unlikely/ColdPath graph explicit.

The [MIR-inline restriction](tests/spec/source-no-mir-inline/README.md)
authenticates actual internal-feature permission and a unique active nonforeign
function/method owner. The current frontend has no Rust MIR inliner; adjacent
native inline hints retain their meanings. A future MIR-like pass must first
preserve authenticated imported intent. The source marker grants no const,
unsafe, API, layout or intrinsic capability. The complete original pointer and
MaybeUninit body graph remains required independently of this header contract.

The [incoherent inherent permission contracts](tests/spec/source-incoherent-inherent/README.md)
retain the original Error header and independently qualified local/cross-crate
controls. `rustc_has_incoherent_inherent_impls` is an authenticated, gated target
permission. A foreign nominal or dyn-principal implementation additionally needs
its own internal-attribute feature and Allow on every actual inherent method.
Local dyn ownership follows the principal trait and uses ordinary local rules.
Preserve complete Self arguments, auto traits, lifetimes, receivers, coherence,
privacy, const and unsafe checks. The target permission and validated incoherent
candidate index must survive genuine cross-crate transport; the local-only Allow
attribute is not itself exported. Existing narrower metadata profiles must reject
unsupported transport explicitly. This contract remains required implementation.

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
adds the explicit version5 source profile with paid exact symbol transport and
separate provider/client native objects. Original nominal source transport,
ordinary-provider publication and whole assignment/library artifacts remain required.

The [Debug array checkpoint](../docs/pa31-source-debug-arrays-checkpoint.md)
retains the pinned field-helper cutoff, real arrays and final unsized indirection.
Its independent observer is explicitly authored test code; full original
formatter execution remains required. The actual source
[enum Default contracts](tests/spec/enum-default/README.md) preserve unit helper
ownership, replay and ordinary Self variant resolution without invented generic
bounds. The [UnsafeUnpin checkpoint](../docs/pa31-unsafe-unpin-checkpoint.md)
preserves genuine source auto-trait proof and correct implicit sizing. The
[wrapping-offset checkpoint](../docs/pa31-source-wrapping-offset-checkpoint.md)
records actual unsafe/const source calls, root-preserving modular arithmetic and
distinct inbounds checks. None of these finite components activates PA31 or
supplies a whole original library artifact.

Source intrinsic item signatures and opened Body/Code Calls use distinct binder
contexts. The [borrowed-call checkpoint](../docs/pa31-borrowed-source-calls-checkpoint.md)
requires exact caller-root matching without lifetime erasure, including actual
runtime and CTFE consumers. Derive storage must pay actual capacity growth and
parser token/splice work, preserving separate attempted and retained budgets;
see the [image accounting checkpoint](../docs/pa31-derive-image-storage-checkpoint.md).
These finite proofs preserve every original library and parent release gate.

Compilation resource limits are explicit finite inputs, with unchanged defaults,
immutable owner snapshots and sticky exhaustion. The three options documented in
[the testing guide](../TESTING_AND_REFERENCES.md) do not grant source permissions.
The [provided-profile migration](../docs/provided-profile-migration.md) retains
historical bytes without restoring false auto-trait sizing premises. Corrected
current rows have their own version; complete original source transport and
whole assignment release remain required.

The [union checkpoint](../docs/pa31-source-union-copy-checkpoint.md) now includes
the real source union Clone recipe and independently matched scalar/array native
observers. Its original escape expansion passes; typed nested field macros and
the actual scalar ranges used by niche_types remain separate source consumers.

The [typed field-macro checkpoint](../docs/pa31-derive-type-macros-checkpoint.md)
now preserves genuine nested macro captures and reaches the actual range parser.
Scalar pattern validity and the authentic range-helper protocol remain required;
successful copying does not grant those semantics or source body permission.
