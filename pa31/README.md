# RustGM Programming Assignment 31: Compiler/library bindings

Status: **planned**. This is a buildout brief, not a released grading contract.

The [range/nominal-value checkpoint](../docs/parallel/source-range-main-join.md)
qualifies actual source range records and real nominal CoerceUnsized value
conversion. Ordinary type, region, move/Drop and selected implementation proofs
remain authoritative; shared completed prefixes remove quadratic inference and
sizing work. Full inclusive-call syntax, derives/modules/iteration, wider unsize
terminals, const and authenticated transport remain required. This is a finite
checkpoint, not completion of this assignment or an original library artifact.

The [ResumeTy/get_context checkpoint](../docs/parallel/resume-context-main-join.md)
uses actual ordinary nominal/function completions and preserves body/unsafe
proof. Its native companion executes the exact pointer helper with authored
Context support; full original Context/Waker/derive and async/metadata remain
required. Unchanged original core now reaches IntoFuture::into_future.

The [CoercePointee source checkpoint](../docs/parallel/coerce-validation-main-join.md)
preserves actual trait headers and requires authenticated builtin origin for
validity implementations. Full derive generation, generic custom auto traits,
coercion/dispatch consumers and metadata remain required. The unchanged original
core frontier advances to ResumeTy; this does not release PA31.
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

The [complete Drop source packet](tests/spec/full-drop-source/README.md) qualifies
the original provided Drop/pin_drop bodies and actual ordinary, pinned and generic
cleanup. Unused alternate signatures retain ordinary header legality; exhaustive
registration and actual execution still require their checked contracts. The
cleanup callable can be the provided trait method while the implementation
overrides pin_drop. Its real body, dispatch, substitutions and loans must be
proved normally. Wider pin projection, async cleanup, CTFE and authenticated
transport remain required; this finite contract does not release the assignment.

The [VaArgSafe/VaList declaration checkpoint](../docs/parallel/source-va-declarations.md)
retains ordinary actual headers, fields, implementation domains and ownership.
It supplies no invented Copy, target layout or variadic runtime permission.
All original valid c_variadic feature requests, full standard marker/module
closure, actual cursor init/read/copy/end/cleanup and metadata remain required.

The [UnsafePinned source packet](tests/spec/unsafe-pinned-source/README.md)
qualifies actual nominal identity, interior storage and niche suppression,
including a direct-field owner. Its22 independent source outcomes and4 native
observers preserve real trait implementations, normal loans and field cleanup.
Noalias/retag optimization, complete constant mutation, full Pin/async behavior
and authenticated interior-storage metadata remain required. This finite source
contract does not release this assignment or build original core by itself.

The [UseCloned source checkpoint](../docs/pa31-use-cloned-source-checkpoint.md)
authenticates the actual canonical trait and preserves ordinary impl, member,
Clone and Copy obligations. Two real source native companions match pinned Rust.
Its use-expression/capture/const/metadata consumers remain explicit required work;
source admission alone does not complete LIB-152 or this assignment. Unchanged
original core cleared UseCloned and then reached FnPtr/FnPtrAddr.

The [function-pointer checkpoint](../docs/pa31-function-pointer-checkpoint.md)
now qualifies their actual source ownership, builtin pointer evidence and
standard typed address shim with38 complete source outcomes and native Rust
equality. Address conversion does not invoke the pointer and preserves actual
ABI/safety and higher-ranked/scoped lifetime contracts. Broader demanded method
profiles, const and metadata remain required. Original core now stops at the
real AsyncDrop declaration; see PA42/43's source-signature/destruction boundary.

The [FusedIterator source packet](tests/spec/fused-iterator/README.md) retains the
actual pinned marker source and ordinary Iterator/Item implementation proof.
Language-role completion grants no blanket implementation or generated body;
genuine gen coroutine state/exhaustion and authenticated transport remain separate
PA42/43/29 requirements. This finite source contract does not release this PA.

The [Pin source packet](tests/spec/pin-source/README.md) qualifies an actual
prerequisite: nominal identity and ordinary `new_unchecked` source bodies retain
real unsafe/const, move/borrow and field-destruction checks. Pinned receivers,
implicit pin construction, Unpin/projection, async/future behavior and PA29
transport remain required. This does not move the original AsyncDrop frontier.

The [Deref/receiver checkpoint](../docs/pa31-deref-receiver-checkpoint.md) now
qualifies actual source Deref/Target, real shared/mutable method/operator calls
and exact smart-pointer receiver proof. Its31/12 source outcomes and two native
observers match pinned Rust. Wider profiles, const, metadata, Unpin/projection
and actual async state/destruction remain required. Original core still reaches
the same AsyncDrop declaration.

The [request-order checkpoint](../docs/pa31-request-order-checkpoint.md) extends
the same cumulative source payer through the shared checked ordering kernel.
Metadata and ordinary lookup keep their finite default limits. The original
core frontier is now source lint-policy storage; no library artifact is claimed.

The [inventory-budget checkpoint](../docs/pa31-inventory-budget-checkpoint.md)
shares authenticated source/inventory work and retained storage without granting
permission. Standalone fallback and finite sticky failures remain required.
The [const inherent checkpoint](../docs/pa25-inherent-block-checkpoint.md)
qualifies actual group/member policy and PA25 runtime/const eligibility without
merging them. The [field-policy checkpoint](../docs/pa31-field-policy-checkpoint.md)
adds direct/inherited aggregate field policy and repairs wider const-call
regressions while preserving imported-policy fences. Original core now reaches
unsupported intrinsic declarations; real operation contracts and PA29 transport
remain required.

The [negative-coherence packet](tests/spec/negative-coherence/README.md) covers
actual with_negative_coherence/negative_impls permission and strict type/trait
owners. PA20 owns typed overlap/refutation proofs; PA29 owns imported policy.
Conditional lifetime/Sized debts must be paid from actual header premises.
Ordinary-use guards, auto-negative legality and metadata fences remain distinct.

The [unadjusted ABI packet](tests/spec/unadjusted-request/README.md) binds actual
abi_unadjusted feature/macro authority to a distinct convention. PA28 owns the
scalar/aggregate/intrinsic ABI consumers, PA8 owns opaque grammar and PA29 owns
transport. Its qualified scalar path advances the original source frontier;
aggregate/vector calls and authenticated convention metadata remain required.

The [default-body packet](tests/spec/default-body-stability/README.md) specifies
rustc_default_body_unstable as direct body policy, separate from stable API and
const eligibility. Real local provided/override execution and external inherited
use have different rules. Imported policy/body composition, full Drop/Pin behavior
and original library artifacts remain required.

The [renamed-feature packet](tests/spec/renamed-features/README.md) specifies
old_name declarations and hard former-name request rejection, with independent
local and actual imported consumers. Its canonical feature projection preserves
first-declaration, collision, const and release-version behavior; full authenticated
relationship transport and original library artifacts remain required.

The [shared feature inventory](../docs/pa31-shared-feature-inventory-checkpoint.md)
reuses one capture-time canonical index and immutable spelling data without
merging policy owners or permissions. Deferred conflicts, const/default policy,
old wire fences and the original inventory limits remain intact. Its4096-function
crate completes source/type checking; original core's remaining inventory phases,
backend-reliability cfg contract and actual library artifacts remain required.

The [language-item declaration packet](tests/spec/compiler-language-claims/README.md)
checks shared target, generic and foreign-only rules for the remaining raw-source
inventory. Profiling hooks keep actual bodies; declaration recognition alone
does not supply builtin semantics, metadata transport or original artifacts.

The [macro-valued language-item packet](tests/spec/language-values/README.md)
shares the actual source-owned attribute expansion path with diagnostic names.
Claims still require real feature, target, arity and duplicate validation. Its
22 appended panic_const declarations retain every older ordinal and grant no
unimplemented compiler assertion, const-panic or runtime permission. Real marker
bodies supply independent native controls; full panic consumers and metadata
remain required.

The [type-reflection source checkpoint](../docs/pa31-type-reflection-source-checkpoint.md)
qualifies actual `type_info`/`type_id` declaration targets and unrestricted generic
forms, while retaining intrinsic/provenance and old-metadata boundaries. Its
ordinary field body executes with the pinned backend. Original core advances to
the `compiler_move` profiling marker; real reflection, nominal/value publication
and genuine imported consumers remain required.

The [transmutability source checkpoint](../docs/pa31-transmutability-source-checkpoint.md)
qualifies actual `transmute_trait`/`transmute_opts` declarations and canonical
named-boolean assumption decoding. The source default method retains its real
union/ManuallyDrop algorithm. Actual all-values layout/validity/reference/lifetime
selection, coinductive evidence, implicit RustGM/native use and imported transport
remain required; this declaration/value boundary does not complete LIB-124 or PA31.

The [private layout kernel](../docs/pa31-transmutability-layout-relation.md)
qualifies compressed value languages and actual ABI padding/tag/union/reference
data. Source-scoped selection and execution remain required. The published header
compiler now advances unchanged core to `type_info`; prioritize that actual source
blocker while retaining the complete transmutability and transport requirements.

The [source NoDrop checkpoint](../docs/pa31-no-drop-checkpoint.md) qualifies the
actual trait and union Copy fallback through canonical component proofs and real
native relay execution. Field queries consume a compact shared type fact for
the pinned structural shortcut; it grants no trait, value or API permission.
Source wrapper completion precedes its union consumers. The
[MaybeUninit constant checkpoint](../docs/pa31-maybe-uninit-const-checkpoint.md)
qualifies actual source methods, opaque union storage and reached constant
intrinsics. The [native checkpoint](../docs/pa31-maybe-uninit-native-checkpoint.md)
also qualifies the three original panic/value/destructor observers through real
source bodies. General PA29 imported transport and the remaining runtime profiles
stay required; this does not activate PA31.

The [optimization-intent packet](tests/spec/source-optimize/README.md) freezes
real size/speed/none semantics, executable owners, feature and macro-envelope
authority, hard duplicates and none/inline conflicts. The source producer retains
typed intent in sparse existing headers; current O0 behavior grants no new
const/unsafe/borrow or body authority. Old metadata profiles fence unencoded
intent. Authenticated imported optimizer policy and original library artifacts
remain required alongside source qualification.

The [method-dispatch declaration producer](../docs/pa34-dispatch-declaration-checkpoint.md)
retains the actual trait's array/boxed-slice policy with source ownership,
feature/macro authority, bounded payments and old-profile metadata fences.
The [cross-edition packet](../pa34/tests/spec/method-dispatch-policy/README.md)
preserves real adjusted-receiver, qualified-call and genuine Box requirements.
Receiver execution, mixed-origin method-name editions, custom-auto prerequisites
and authenticated policy transport remain required; this does not complete the
source binding or runtime units.

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

The [ordinary specialization-trait packet](tests/spec/applicable-traits/README.md)
and [checkpoint](../docs/pa31-applicable-traits-checkpoint.md) attach policy to
actual traits and validate all their implementations, including non-overlapping
ones. Preserve real feature/macro authority, always-applicable conditions,
supertraits, unsafe-marker precedence and existing sizing metadata. Complete the
retained quantified and general imported-policy requirements under PA20/29;
local declaration/dispatch evidence does not complete these assignments.

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

The [forced dyn contracts](tests/spec/forced-dyn-policy/README.md) require the
actual trait principal/supertrait restriction independently of language-item
spelling. Preserve ordinary impl/generic/static use, lazy unused aliases and
marker-only objects. Share authenticated defining headers and cached typed
rejections, preserving feature/name allowance, failed budgets and exact sizing
wire flags. General policy transport and actual Field/reflection consumers are
required independently of accepting the annotation.

The [fundamental source contracts](tests/spec/fundamental-policy/README.md)
separate real struct argument locality from trait compatible-world reasoning.
Preserve the distinct fundamental feature, actual macro-name allowance, source
owner, ordinary runtime behavior and existing sizing metadata. General policy
transport and original nominal/callable declaration/body graphs remain required;
no sizing-language role or compiler operation may stand in for an ordinary owner.

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

The [source-owned iteration checkpoint](../docs/pa26-source-iteration-checkpoint.md)
now compiles actual tagged next/into_iter calls with real Option variants,
ordinary source-call effects, borrow/move proof and native normal/break cleanup.
It shares Try's existing cold per-body plan owner and preserves the no-bootstrap
fence and ordinary hot layouts. Authentic imported protocol transport, remaining
families and whole assignment release checks stay required. The independently
hashed source packet lives in PA31's source-iteration directory.

The [contract builder/capture checkpoint](../docs/pa31-contract-builders-checkpoint.md)
qualifies real generic const helper bodies, actual source callable Copy evidence,
closure capture field proofs and typed-wildcard reborrow evaluation. Existing
caller assumptions, source traits, lifetime debts and const/safety checks remain
required. Contract intrinsic/runtime transformations, imported body/value/API
transport and complete assignment publication remain outstanding.

The [diagnostic value checkpoint](../docs/pa31-diagnostic-values-checkpoint.md)
expands actual nested/source/aliased attribute macros, preserving literal values,
original owners and inline/external module lookup. A paid module-item queue
retains outer wrappers when inner attributes introduce another owner. The full
original core clears that declaration and next reaches
rustc_allow_const_fn_unstable(const_eval_select) in panic.rs. Real const feature
allowances and imported stability/provenance remain required; PA31 stays planned.

The [const-function allowance checkpoint](../docs/pa31-const-fn-allowances-checkpoint.md)
now qualifies that exact lexical contract, including intrinsic feature opponents,
genuine versioned metadata, a fresh imported const consumer and native `7 7`.
Declaration-only queries preserve body-demand boundaries. The unchanged original
core clears the attribute and reaches the actual ManuallyDrop language item;
source wrapper semantics and all original library artifacts remain required.

The [source wrapper checkpoint](../docs/pa31-source-wrappers-checkpoint.md) now
qualifies local ManuallyDrop/MaybeDangling headers, real constructors/layout,
separate cleanup policies, scoped CTFE validity and4 native observations. Full
CFG proof retains the actual lifetime rejection;64 CLI constant observations
cover32 cases. General nominal publication/imported consumers remain PA29 work.
The successor [MaybeUninit constant checkpoint](../docs/pa31-maybe-uninit-const-checkpoint.md)
clears that original role boundary and reaches transmute_trait in
mem/transmutability.rs87. Runtime panic, imported storage/validity consumers and
all original library artifacts stay required.

The actual source coroutine family retains `CoroutineState`, `Coroutine`, Yield,
Return and resume declaration identities independently of their ordinary names.
Accept unused legal alternate declarations according to pinned Rust; compiler
consumers require the exact selected schema and full arguments at demand. Ordinary
enum construction, projection, trait selection and manual native resume calls use
the existing typed producers. Generated coroutine input/yield/return/state belongs
to PA42, and captured loans, Pin, cancellation/drop to PA43; no role supplies those
operations or a blanket implementation. Preserve the complete original coroutine
module and independent wrong-owner/shape/argument/lifetime opponents when closing
this source prerequisite.

The [ordinary coroutine checkpoint](../docs/parallel/coroutine-source-checkpoint.md)
qualifies twelve independent source outcomes and real manual resume output23/47.
Joined Field capture and concrete cleanup planning controls pass, and original
core advances past coroutine role admission to panic_bounds_check. This does not
prove complete original-module typing, generated coroutine execution or a library
artifact. Preserve the remaining source/runtime/transport requirements above.
