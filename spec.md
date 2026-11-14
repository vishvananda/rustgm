# Compile-time-first Rust frontend architecture

This is the normative architecture of the maintainer reference frontend.
It adapts the principles of cppgm-extended's `spec.md`, pinned in
[docs/cppgm-spec.md](docs/cppgm-spec.md). Preserve its attribution in
[shared/CPPGM-NOTICE](shared/CPPGM-NOTICE). The C++ implementation is a guide to
efficient representations and algorithms, not an implementation of Rust rules.

Assignment IDs identify ownership, not a speculative execution sequence. The
[adopted dependency plan](docs/assignment-order.md) separates local declarations
from source execution, runtime kernels from symbolic CTFE, and declaration
metadata from completed value/body transport. Consume those typed producer
boundaries; do not introduce a second parser, authority map or fake library
body to bypass an unavailable predecessor.

## Scope and pipeline

Target Rust 1.98.0, first edition 2024, on Linux x86-64. Freeze language release,
Unicode tables, edition, target, panic mode and overflow-check mode in each
relevant test manifest. PA34 adds the Rust 2021 crate profile; PA40 closes the
remaining editions. Original std's legacy-edition dependencies (cfg-if 2018,
rustc-demangle 2015) bring forward exactly the edition rules they exercise,
each with independent controls; every other 2015/2018 rule stays rejected by
name until PA40. PA31–PA36 require original upstream core, alloc and std,
with real-crate acceptance beginning after core and expanding with alloc/std.
Rustc-private binary/metadata compatibility
and a drop-in Cargo/rustc interface remain separate objectives.

Classify target-feature language requests against the fixed selected target.
Pinned foreign-architecture requests have no consumers on x86-64 and need no
ordinary binding owner or library instability rows. Keep their explicit known
set separate from selected-target attribute validity, genuine x86 gate bits,
body requirements, cfg truth and instruction support. Unknown spellings still
fail. The [architecture-request packet](pa41/tests/spec/architecture-requests/README.md)
checks source opposition and ordinary native/configuration equivalence.

Use the shared typed `support::edition::Edition` and select each rule's defining
context explicitly. Select a macro's expression fragment grammar
once from its defining edition, retaining the completed fragment kind for matching
and memoization. Token provenance remains separate from that grammar selection.
Only edition-sensitive syntax demands a token's originating edition. Borrow this
context through the existing parser input provider, and use retained expansion
origin ranges; do not grow ordinary tokens or relex generated spelling. A parsed
opaque fragment retains its identifier validity when converted to attribute
paths. Keep that evidence scoped to the actual grafted ranges, rather than
assigning the output stream or neighboring tokens the capture's edition.
Copied origins retain exact byte ranges. Typed builtin vocabulary instead uses
an explicit synthetic-origin kind in the same record, bound to the checked
definition edition, generated interval, anchor and actual introduction/context.
Validate that producer/replay once; never reinterpret nonmatching lengths as
copies or infer a macro feature allowance from an edition. Missing provenance
still fails. Source try gates follow their actual endpoint span-composition
rule, sealing both keyword and construct contexts in the existing sparse gate
owner; this does not change keyword-only rules for other syntax.
Const closures keep a distinct wrapper around the ordinary closure node and
reuse that same sparse expression owner. Its kind follows the actual wrapper;
do not add a gate tag or const flag to every ordinary node/closure. Authenticate
the keyword, child, spans and source contexts, then check const_closures at the
actual const keyword. The pinned compiler keeps this gate as an error after cfg
removal; it does not use try-block endpoint composition or warning behavior.
Its expr_2021 matcher also excludes a top-level const closure, even with the
feature enabled. Parsing and feature authority grant no const creation,
call-trait or CTFE permission. Charge each sparse append before publication even
when retained capacity lets a failed attempt retry without allocation.
The [PA34 context foundation](docs/pa34-edition-context.md) records the independent
mixed-edition controls and the remaining work before enabling Rust 2021 in the CLI.

External block safety follows the parsed introducer's originating edition;
explicit member safety qualifiers still require an explicitly unsafe block in
both editions. For supported unsafe linkage attributes, query the actual name
token's edition, independently of a macro-generated wrapper or owning item.
Use existing bounded origin ancestry only when bare syntax demands it. Missing
required provenance must fail; ordinary call/access permissions remain separate.
The [external declaration checkpoint](docs/pa34-extern-editions.md) records the
paired source and native controls without enabling a new driver profile.

Temporary scope narrowing follows the defining if introducer or block delimiter
edition. Preserve binding scopes independently of temporary endpoints; older
block tails and if-let scrutinees inherit the enclosing temporary lifetime.
Use the existing checked conditional-chain facts and paired delimiter origins,
including when an unsafe qualifier or captured tail has another edition. Keep
ordinary scope facts and storage-exit events unchanged. The
[temporary scope checkpoint](docs/pa34-temporary-editions.md) includes opposing
borrow cases and actual destructor traces for both editions.

Never-coercion fallback uses the compiling crate's physical unit edition,
including inside macro-generated bodies. After body constraints settle, default
only unconstrained general roots targeted by never coercions to unit in 2021
or never in 2024. Preserve numeric domains, already constrained types, ordinary
unknowns and existing dependency wakeups. Token-origin ancestry is irrelevant to
this rule; imported original bodies retain their own checked inference results.
The [fallback checkpoint](docs/pa34-never-editions.md) records independent
cross-edition macro and runtime trait-selection controls.

A block has no tail separate from its statements, as in rustc. When
configuration removes a written tail, the last enabled expression statement
without a semicolon becomes the tail. A braced macro statement is never
promoted. Without such a statement the block has no tail. Every other
expression statement without a semicolon must have type `()`, which a
diverging value coerces to. That includes an interpolated `$s:stmt`, since a
`;` written after it is an empty statement. See the
[configured block tail checkpoint](docs/pa31-configured-tails.md).

For edition-sensitive explicit pattern modifiers, consume the parsed flags and
the introducer's original token edition. A complete pattern can span captured
syntax from multiple editions; do not require one edition for its full span.
Only those modifiers demand origin lookup. Preserve ordinary move/loan checks
after selecting the default binding mode; legacy syntax is not proof of Copy or
mutable access. The [pattern checkpoint](docs/pa34-pattern-editions.md) records
the paired Rust 2021/2024 semantic and native controls.

Rust 2015 and 2018 keep every rule that changed only in 2024. Each of their own
differences selects a defining context or fails with a typed legacy-rule
diagnostic naming the edition, never the 2021 rule. Before 2021 a macro's `pat`
fragment excludes top-level alternatives, selected by definition edition. A
closure whose introducer originates before 2021 captures each used root variable
whole, with the strongest mode required below it, and a wildcard mention still
captures. In Rust 2015, a `use` path with a plain first segment and a `::` path
are crate-root relative, by that first token's edition. The compiling crate's
edition decides whether its injected `core` or `std` is a nameable root item.
The [legacy edition checkpoint](docs/parallel/legacy-editions.md) lists the fenced rules.

```text
source image -> Rust lexer -> token-tree/parser/expansion/resolution worklist
             -> syntax with canonical semantic facts
             -> typed Rust body CFG -> initialization/ownership/region checks
             -> drops, demanded instances, closure/async transformations
             -> typed LowIR -> explicit text backend adapter -> CPPGM native
```

The external LowIR serialization is an authorized temporary backend boundary.
The [current backend checkpoint](docs/backend-current-checkpoint.md) pins all
three CPPGM tools to 17b024f69. Its object profile also records the optional
`assembly` capability. Rust `asm!` remains fenced until PA41 lowers checked
assembly bodies to it. Host-object units use the locked object driver's
parser/validator directly; the standalone executable-profile reader cannot
validate every host ABI class. Every object input retains the `.lowir` suffix
gate. Emit `convert trunc exact` only with a real representability proof, as in
the 0/1 comparison carrier for atomic min/max. Ordinary casts and wrapping masks
retain their semantics. Available vector/debug/equivalent-entry vocabulary
requires actual typed producer, feature, ABI and identity proofs before use.
Do not import C++ aliasing or overflow assumptions from source type names.
Reified intrinsic bodies consume the same typed parameter representation as
their callers. A canonical Ordering with a scalar byte ABI is already a value;
only an actual object parameter supplies a tag address. Preserve hidden result
offsets and all dynamic order validation in compare-exchange and retry adapters.
No Rust frontend phase may communicate by printing and reparsing token, syntax,
type or diagnostic text. After an in-process backend becomes available, pass
typed LowIR directly. All backend-visible facts must have a LowIR spelling.

The explicit PA29 `module-nominals-v10` source profile preserves actual named
struct declaration addresses, generic arguments, complete owner-ordered fields,
module visibility and typed constructor/projection operations. Imported records
reuse ordinary Members, aggregate formation and original-body proof. Prepare and
pay destination batches before an exclusive validated publication; foreign/stale
owners and incomplete fields cannot become usable storage. Canonical structural
argument spines use one lazy, budgeted index per actual spine, rather than a full
rescan for every field. Versions0–9 and Provided retain their separate bytes and
authority. Pin/fundamental/destructor policy and broader nominal shapes remain
explicit transport requirements; matching layouts grant no type identity.

### Upstream library boundary

Unused source Drop declarations retain ordinary Rust header legality. Enforce the
executable destructor profile when exhaustive registration finds actual impls;
declaration completion alone supplies no cleanup, Copy or const authority. A
pin-only override retains its real checked receiver and implementation mapping.
Its cleanup recipe names the actual provided `Drop::drop` callable and signature,
then substitutes the concrete Self through the charged canonical callable path.
The original provided body passes normal materialization and trait/loan checks;
do not synthesize its forwarding operation or call the pinned override with a
mutable-reference ABI. The recipe's exact callable identity remains mandatory.
Use the interned signature's cached parameter summary for the direct-call fast
path, preserving bounded shared recipes for generic applications and arrays.
Only a genuine member's own provided Drop body may explicitly call destructor
methods. Authenticate both its actual trait parent and body identity; a closure,
inline constant, subtrait body or impl does not inherit that exemption. Broader
pin projection/pin_v2, async destruction, CTFE and transported body contracts remain
separate required work. The [full-source packet](pa31/tests/spec/full-drop-source/README.md)
records the finite independent and native contract.

AsyncDrop source implementations share ordinary nominal/trait/lifetime coverage
with synchronous Drop while retaining a separate cold family. Source feature
async_drop preserves its actual library request identity; only the synchronous
companion condition depends on that request. Ordinary and const Drop keep their
actual declaration flags. Async methods never enter synchronous call-denial or
receiver indexes. Compiler-owned async helpers authenticate actual headers and
selected types; neither the written placeholder nor a missing direct async entry
proves runnable cleanup or recursive absence. Generated state/poll/cancellation
and authenticated transport remain required before executable publication.

Source language-item vocabulary is append-only and driven by one declarative
inventory. Preserve existing ordinals and diagnostic-only spellings. Validate
new targets, written own/associated-owner generic counts and foreign-only policy
from configured syntax and resolved ownership, before reserving claims. A known
name is no completed builtin proof: unsupported semantic consumers must fail
before ordinary trait, layout, drop or auto-trait processing could assume their
absence. Profiling hooks execute their actual ordinary bodies; current codegen
adds no profiling annotations. Fence old metadata profiles until they encode the
actual compiler contract. The [PA31 declaration packet](pa31/tests/spec/compiler-language-claims/README.md)
binds independent positive/opposing cases and preserves unimplemented semantics.


Validate active source optimization/advisory annotations against their real
feature and configured target. Reuse transient per-owner state or existing cold
drafts when the current backend has no corresponding optimization or advisory
producer; do not add ordinary nominal permission rows, fake absence knowledge,
write privileges or a reason to skip destruction. Preserve independent inline,
const, unsafe, lifetime and body checks. Expression annotations consume the
actual parsed root request or token-origin macro allowance in every crate mode.
The [source checkpoint](docs/pa31-advisory-source-checkpoint.md) records the
current backend boundary and deferred advisory work.

Validate rustc_doc_primitive as a documentation label on an actual module, under
rustc_attrs or its real attribute-envelope allowance. The pinned compiler accepts
any string, including unknown/empty names. It rejects unsafe wrappers, malformed
forms, wrong targets and hard duplicates. Inner and outer captures of the same
parsed module share duplicate state; use a sparse prepaid draft scoped to the
existing owner walk, then discard it. The retained AST already owns the original
label. It grants no primitive type, language item, layout, body/const/unsafe or
privacy permission and needs no semantic metadata fence. The
[PA40 documentation-label packet](pa40/tests/spec/primitive-documentation/README.md)
records source, native and original-core evidence separately from whole attribute
closure and original library artifacts.

Retain `optimize(size|speed|none)` as typed, source-owned intent, using the existing
configured-owner walk and sparse header promises. Validate its optimize_attribute
request, actual attribute-envelope macro authority, function/method/closure body,
forms and hard duplicate rules. None conflicts with inline and inline(always),
but accepts inline(never), in either attribute order. Source presence, checked
absence and unknown imported policy are distinct; queries authenticate original
images, callable syntax, effects and bound function identities. Old metadata
profiles reject unencoded intent. The locked O0 backend remains conservative;
future MIR/backend optimizers must consume authenticated local/imported intent
before selecting optimization or inlining. The [source optimization packet](pa31/tests/spec/source-optimize/README.md)
records primary and independent evidence; it does not complete PA31/40/29.

Explicit negative coherence is a declaration query, separate from positive trait
use. Select stable, negative or strict mode from the real crate request and
actual configured type/trait owner. Unify complete headers first, then freeze
unknown intersection inputs as rigid source parameters before proving a
negative. Retain argument correlations, Sized coverage, supertraits and scoped
lifetime debts. Pay debts only from those headers' own premises through the
existing relation solver; a missing implementation, cutoff or unpaid negative
is not proof. Do not run positive member/supertrait/physical-capability checks on
a negative impl. Keep auto's separate applicability rules and ordinary-use
refutation guards. Store strict owners sparsely through the shared attribute
and definition walks; existing metadata profiles must reject unencoded policy.
The [negative-coherence packet](pa31/tests/spec/negative-coherence/README.md)
records independent source predictions and the remaining transport boundary.

An overlap condition whose quantifier binds no region, as parenthesized
callable sugar writes, is an ordinary where clause, as in rustc 1.98; one that
binds a region still proves no disjointness. The
[callable sugar coherence controls](pa31/tests/spec/callable-sugar-coherence/README.md)
record independent outcomes.

Attribute feature queries follow the pinned gate's actual construct span.
Unstable builtin attributes use their retained `#` through `]` envelope: an
allowing macro can generate `#[$meta]` while the captured name keeps its caller
context. A forwarded whole item retains its original `#` and gains no allowance.
Use the existing paid macro session and authenticate the retained attribute,
configured owner and source image before publishing the declaration. Do not
change general token provenance or grant a captured name a new permission.

Const-function feature exemptions belong to exact checked bodies. Parse their
identifier lists once from the configured attribute, validate the real const
target and attribute-envelope authority, and retain sparse sorted function and
feature rows in the cold library inventory. They do not become crate feature
requests, intrinsic gates, constness or unsafe permission. Unmarked staged const
bodies retain their own recursive stability obligation; nested bodies do not
inherit exemptions. Transport actual implicit-body proof and lexical grants
separately from API policy in versioned availability data, and recheck the
defining body's actual callees after decoding. Older absence is not authority.
Declaration-only policy and execution-admission queries use authenticated
declaration headers; construct defining environments at actual body demand.
Keep the full original/instance proofs before executable Code publication.
The [const allowance checkpoint](docs/pa31-const-fn-allowances-checkpoint.md)
records source, intrinsic, imported const and independent native controls.

A recursively const-checked body (const-stable, or implicitly or explicitly
const-stable-indirect) may call an intrinsic that is marked
`rustc_intrinsic_const_stable_indirect`, or one whose source fallback body is
itself safe to expose by the same rule. A bodiless or explicitly const-unstable
intrinsic is not exposed, as in rustc 1.98.

Local legacy const-generic indices belong to the actual free function. Complete
their contract only when its owned const formals and typed signature are
published; preserve list order, repeated indices and nested function-pointer
lifetime binders. This never rewrites a same-crate call. External legacy calls
require the real metadata indices and argument-extraction protocol, with the
ordinary const, unsafe and API checks; profiles without that protocol remain
explicitly fenced.

Follow the [library bootstrap plan](docs/library-bootstrap-roadmap.md). Compile
ordinary library definitions, generic bodies and algorithms from pinned upstream
sources. Bootstrap declaration/body factories are temporary: a source sysroot
must replace them without duplicate identities or a synthetic fallback. Do not
implement ordinary missing methods or containers inside the compiler.

The compiler supplies validated source-declared language/diagnostic items,
built-in expansion, intrinsic semantics, primitive/layout rules and the internal
features actually used by the pinned sources. A typed registry drives signature
checking, const eligibility, capability answers and lowering. Semantic identity
includes declaration and owning crate; familiar names are not implementation
evidence. Transport checked generic/const bodies and their retained contracts
through deterministic crate metadata with indexed external-to-local remapping.

Intrinsic declaration recognition consumes the pinned compiler's complete name
inventory. Retain a distinct typed identity for each name while sharing immutable
signature recipes. Check owned generic counts, canonical parameter identities,
Rust ABI, safety, structure, real nominal language identities and bound-region
relationships once, before publishing the reserved signature. Prepay bounded
recipe work through the existing source ledger; never traverse an arbitrary
actual type graph or grow ordinary nodes to carry this cold declaration data.
Recognition grants no executable operation or target capability. Reached calls
without a kernel must fail before an authored body could stand in for intrinsic
semantics; old metadata profiles must reject unencoded contracts. Preserve the
ordinary intrinsic reification prohibition. The [declaration checkpoint](docs/pa28-intrinsic-declarations-checkpoint.md)
retains independent outcomes, required positive gaps and scaling evidence.

Ordinary `rustc_specialization_trait` policy belongs to the actual trait,
independently of language-item identity. Reuse the configured-owner annotation
walk, source witnesses and typed specialization basis. Implementations require
their real specialization request or impl-token allowance even without an overlap;
crate Min checks remain mandatory when requested alongside Full. Verify their
complete inputs and own conditions, allowing well-formed requirements and the
elaborated consequences of other always-applicable traits. Unsafe marker policy
takes precedence, as in the pinned compiler. Share cumulative payment across
annotation queries and implementation verification; never restart a failed
ledger or use an impl's own Self:Trait as a premise. Preserve existing sizing-role
wire flags, while ordinary unencoded trait policy fails publication/import.
The [applicable-trait checkpoint](docs/pa31-applicable-traits-checkpoint.md) retains
quantified-predicate work and the other assignment requirements explicitly.

Selection in a crate that requests specialization follows rustc's winnowing.
Of the applicable candidates proven for one fully known goal, a candidate that
another applicable candidate specializes is dropped, so the most specific impl
is selected before the graph closes. A contested pair is ordered once, by one
pairwise comparison whose own declaration facts the selection may wait for.
Closure later authenticates the recorded relation's source witness and its
feature authority. Candidates that neither specializes stay ambiguous, and an
overlap without an ordering remains a coherence error. An associated type that
the selected impl omits comes from its nearest defining ancestor, with its
arguments translated through each relation, as rustc's `assoc_def` does. Under
`min_specialization` only a root may define one, so a child that overrides its
parent's associated type is rejected.

A child is compared with its parent as rustc's `specializes` does: the child's
trait reference is equated with the parent's fresh instantiation, lifetimes
included, so each parent header lifetime is named by the child's corresponding
one without resolving any region constraint. `min_specialization` still rejects
a `'static` or repeated lifetime among the parent's arguments. The
[lifetime specialization controls](pa31/tests/spec/lifetime-specialization/README.md)
record independent outcomes.

Every parent obligation is then registered under the child's environment and
solved to a fixpoint. A goal that is still ambiguous waits until another goal
is proven, so a parent parameter that only a where clause names is inferred
first, as in core's `SpecExtend` pair. Ambiguity that no proof resolves means
the child does not specialize. It is never an error of its own, and it never
rejects the impl. Before the repeated-parameter and `'static` checks,
`min_specialization` omits each parent argument that the parent's own
associated equalities determine, as rustc's `unconstrained_parent_impl_args`
does. Both rules have recorded independent outcomes:
[fixpoint](pa31/tests/spec/specialization-fixpoint/README.md) and
[determined-argument](pa31/tests/spec/determined-parents/README.md) controls.

`min_specialization` compares the two impls' clauses as rustc's
`predicates_of` holds them: a `[const] Tr` bound is the ordinary `Tr` clause,
and a `const Tr` bound is `Tr` plus its own host-effect clause, which is never
a marker clause. Constness is checked by the ordering instead. A parent that is
const in more cases than its child, because the child is not a const impl or
cannot prove the parent's `[const]` bounds, orders nothing. The
[const bound controls](pa31/tests/spec/const-specialization-bounds/README.md)
record independent outcomes.

An override's bounds are compared with its trait method's bounds in the impl's
normal forms, as rustc normalizes the impl's param-env first. The trait
method's own bounds, substituted for the impl, and the override's bounds are
normalized through the impl's checked associated definitions and then ordinary
selection, each quantified clause inside its own binder. An associated equality
keeps its projected side as written and normalizes only that side's inputs and
its result. An override that repeats a `Self::Item` bound, or names the impl's
definition of it, states the same requirement; a different or added bound is
still stronger.

A quantified associated equality among an impl's conditions whose result names
a type or const parameter the impl header leaves undetermined also gives the
trial one instance at fresh regions, one per bound region. That instance is
implied by the universal condition, which is still proven in its own scope after
the parent resolves; it only lets the trial infer the parameter, as rustc's
placeholder instantiation does for core's `Scan` and `MapWindows`. A parameter
inferred to name one of those regions cannot leave the trial, so that selection
fails. An impl header that names a projection of its own parameter, as core's
`impl<R: Try> ControlFlow<R, R::Output>` does, is normalized under the impl's
bounds, formed on demand before the header is published, as a signature is under
its own. Rustc's `Self` in an impl's bounds is the unnormalized header, so when
any bound of the impl names its own `Self` (`where Self: Sized`, `Option<Self>`,
`R: Same<Self>`), its header keeps each projection as written and forms its
environment after publication. Once every impl environment is complete, each
such projection is normalized again under it, and a result other than the
written projection is rejected by name: the header must stay rigid.

An impl trait reference argument that its path leaves to a default, as core's
`impl<T> FromResidual for NeverShortCircuit<T>` leaves `<Self as Try>::Residual`,
is that default instantiated by the preceding arguments with `Self` set to the
impl's self type. It is normalized by the routine a written argument uses, under
the same rules, before the trait header is published, and the reference stays
active until it is. Its normal form is kept beside the default by impl and
argument index, never under a source node. A default whose normal form does not
satisfy the parameter's bound is rejected (E0277). The
[defaulted impl argument controls](pa31/tests/spec/defaulted-impl-arguments/README.md)
record independent outcomes.

Every trait path, whether an impl trait reference, a bound, a where clause, a
qualified projection prefix or a shorthand bound, instantiates an omitted
default with the preceding arguments and the trait's `Self` as the path's
subject in one simultaneous substitution. The default's retained alias
requirements are expanded with the same mapping, and a nested default's
requirements map their own subject as they map their arguments, so a default
naming another trait's member (`R = <Self as Source>::Item`) requires
`Subject: Source` where it is used. A `Self` written among a trait path's
arguments is not the path's subject: inside a trait's items it is that
trait's `Self`, so `fn take<T: Fill<Self>>` requires `T: Fill<Self>`, not
`T: Fill<T>`. Supertrait elaboration maps `Self` separately and is unaffected.

Fundamental policy belongs to the actual struct or trait and uses its distinct
`fundamental` feature or genuine attribute-name macro allowance; `rustc_attrs`
and sizing-language identity are not substitutes. Reuse the configured header
owner, source image and shared sticky contract ledger. Nominal orphan locality
looks through all type arguments of a foreign fundamental wrapper in source
order, preserving an uncovered parameter before the first local type. A local
nominal remains local regardless of its marker, and a fundamental trait object
does not become a transparent nominal. Memoize shared visited type subtrees
once per foreign-wrapper traversal, with paid storage and an allocation-free
ordinary root path. Separately, closed fundamental trait obligations may exclude
future upstream implementations; open/downstream inputs remain possible.
Internal consumers reuse completed typed declarations, not repeated AST scans.
Existing exact sizing-role wire flags stay supported; publication of unencoded
general nominal/trait policy must fail explicitly until authenticated transport
is implemented. General coinductive and forced-dyn policy have their own real
solver/object requirements and must not be inferred from fundamental acceptance.
The [fundamental checkpoint](docs/pa31-fundamental-policy-checkpoint.md) records
the independently qualified local profile and explicitly required import closure.

Forced dyn incompatibility is an independent policy on an actual trait. Share
the existing defining-header owner and fixed policy witnesses with fundamental,
while preserving their distinct targets, feature/name-token permissions and
semantics. A restriction grants no deny-impl, specialization, auto-trait,
coinductive, intrinsic or source-body capability. Check the actual principal and
supertrait graph using the existing demand-driven dyn query. Retain forced
rejection as a typed incompatible contract so shared ancestors and repeated
demands reuse the cached failure; do not flatten inherited member prefixes.
Preserve lazy unused aliases and the absence of a principal in marker-only auto
objects. Shared failed source payment cannot become policy absence. Existing
exact sizing-role wire flags remain supported; general unencoded policy must
fail publication rather than disappear. The
[forced-dyn checkpoint](docs/pa31-forced-dyn-policy-checkpoint.md) records the
source contract, independent observations and remaining transport/Field work.

Implement obligations required by the pinned language and actual consumer.
Keep declaration checking, value validity, optional optimization and advisory
lint claims distinct. A documented niche guarantee does not invent a field
proof requirement, and a lint-only pointer-return annotation does not grant
runtime pointer validity. Before adding a stricter prerequisite or another
proof/state graph, verify the real Rust rule and reuse its existing typed
producer/consumer. Preserve uncertain cases as explicit unsupported behavior.

Establish structural, identity, visibility and required coherence invariants at
the owning import/phase boundary, then reuse that immutable typed owner and its
indexes. Do not add equivalent provenance wrappers or repeat whole-catalog
validation for each demand. Expensive unrelated imported elaboration may stay
Pending when it cannot affect lookup, coherence or the selected proof; required
source declaration/body checks and actual original/instance obligations remain.
Implement a complete supported consumer path before broadening the profile.
Unsupported forms must diagnose at their actual boundary rather than requiring
every future form to be implemented before any supported source can execute.

Index inherent trait-object candidates by the actual principal trait identity in
the existing cold method owner. Keep complete Self arguments, auto traits,
lifetimes and receiver applicability in the ordinary proof. Shape identifiers are
not a disjointness proof: local principal methods retain ordinary unused-member
overlap checks. Authenticate a local principal against its exact active trait
source owner; imported principals require their own target permission and
validated method-index transport. Primitive incoherent lookup policy stays
separate. See the [PA23 contract](pa23/tests/spec/members/local-dyn-inherent/README.md).

An incoherent inherent target marker belongs to its actual defining declaration,
separately from a method's Allow permission. Retain this sparse promise in the
existing cold source-header owner and bind it in the existing definition walk;
do not create ordinary nominal header rows merely to record absence. Source
queries authenticate the current image, configured effect and target kind.
Keep old imported profiles Unknown and explicitly reject exports that would
lose target authority or its validated inherent candidate index. Local ownership
does not require Has. The [source checkpoint](docs/pa31-incoherent-target-checkpoint.md)
retains the wider foreign/metadata requirements.

Header diagnostic and advisory contracts use complete versioned cold columns.
Authenticate local declaration, configured owner and retained source incarnation;
join imported declarations to their actual catalog coordinates once. Reuse the
existing owner/effect and declaration passes for positive completion. Missing
entries, unvisited owners and older metadata remain Unknown; KnownAbsent needs
an actual completed producer witness. Borrow retained reasons on warm reads,
charge attempted work and retained bytes, and preserve aggregate limits on
failure. Do not grow ordinary syntax, body or declaration records for these
columns. Provider declaration warnings occur once, while imported unused-result
diagnostics use the consumer's lexical lint policy and use location. Select an
explicit publication profile and preserve legacy bytes; contract presence never
grants execution or trait permission. The
[versioned transport checkpoint](docs/source-header-contract-transport.md)
records exact-byte, fresh-consumer, origin, budget and layout controls.

Historical provided-envelope version zero retains its exact decode/encode bytes
as opaque data. Current unpaid provided rows use the distinct version two and
correct auto-trait Self sizing; never recreate the old implicit MetaSized premise
to preserve an obsolete producer assertion. Before ordinary or private semantic
materialization, validate this boundary once with paid typed role/application
identities. Source-site equality and advisory classification cannot authorize a
premise. Unresolved foreign canonical auto-trait Self bounds remain unsupported
until dependency-role provenance is complete. See the
[semantic migration](docs/provided-profile-migration.md).

Unknown source diagnostic-item strings remain exact inert data in an optional
cold column of the existing reservation owner. Preserve declaration and
attribute identity, including empty, Unicode and NUL-containing strings, without
adding a compiler role or execution permission. Reuse the shared name interner;
its paid insertion charges hashing, probing, comparisons and capacity/rehash
work before mutation. A failed payment preserves the previous table and IDs.
Ordinary insertion uses the same algorithm without a second table or owner.
Do not publish these rows through an older metadata profile.

Demand diagnostic attribute expressions from their retained token trees once,
using the existing opaque registry, parser and macro worklist. Keep the original
attribute/effect/owner witnesses and a cold typed expression root; decode the
completed literal rather than reparsing rendered output. Module attributes use
the module's macro scope, including its imports, while an external module's
attribute still belongs to its declaration file. Keep those identities distinct.
Restore the captured textual scope when visiting a delayed value. Charge syntax,
opaque payloads and retained receipts before publication. Direct literals and
ordinary source allocate no value column. Irreducible non-literals reject before
ordinary body/scoping work; detached metadata does not enter the ordinary
one-file-per-module node-scope index. Preserve the real eager parent dependency
using a checked compact call identity, including derive members and retirement.
Module wrappers with inner attributes retain every affected outer owner in a
paid queue keyed by the actual module item. Replay those owners when its scope
is created; the nearest wrapper alone does not own all declaration attributes.
Do not recover this association with a per-module scan of every file owner.
Language-item and diagnostic identities share this source-owned expression
mechanism. Language claims decode the completed string under their actual owner
and then run the same feature, declaration-target, generic-arity and duplicate
validators as literal claims. Direct strings remain borrowed and demand no value
column. The [language-value packet](pa31/tests/spec/language-values/README.md)
binds actual macro scope, shadowing and opaque forwarding, the complete original
panic_const declaration family and retained compiler assertion/runtime/metadata
requirements. Previous language-item ordinals remain fixed. Cold completion
storage covers the actual vocabulary with paid word-wise work; ordinary records
do not grow. Other attribute-expression consumers retain their own PA8/31/40
contracts. No computed name grants feature allowance, const body or imported
execution permission.

The source `use_cloned` role reserves an actual ordinary trait, then completes
through the existing role ledger only after its canonical Self, declaration
owner and source inputs authenticate. The pinned generic requirement is
unrestricted, not zero parameters. Preserve actual flags, members and written
bounds; the standard library's Clone supertrait is a source requirement, not an
invented restriction on every declaration of this role. Normal trait selection,
provided bodies and original-body ownership proofs remain mandatory. Neither
role reservation nor completion supplies Copy, Clone or auto-trait evidence.
The [UseCloned packet](pa31/tests/spec/use-cloned/README.md) retains the separate
operation and transport requirements. A use expression or capture must choose
Copy, then real Clone when UseCloned is proved, otherwise move, from the original
generic environment. Retain that typed choice through instantiation; do not
reclassify an unbounded generic body from its later concrete type. Clone must
execute the selected real method, with one operand evaluation and ordinary
borrow/const checks. Use-closure formation owns its selected capture value;
closure call kind still follows its body. Keyword/macro permission and imported
identity need their own authenticated consumers. Unimplemented operations and
unencoded transport must fail explicitly. Reuse borrowed canonical headers and
the cold completion set; do not add ordinary node/body fields or reparse output.

Source FnPtr evidence belongs to the completed actual `fn_ptr_trait`, not its
spelling or a reserved claim. Check canonical Self and zero trait arity; prepare
the actual required `fn_ptr_addr` method before authenticating its parent,
membership and callable. Preserve normal declaration bounds, flags and generic
rules, including Rust-valid unused alternative method declarations. Builtin
selection accepts actual function pointers with their complete signature,
safety, ABI and binders, retaining current inference, formation and real caller
obligations. Function items, closures, references and raw pointers receive no
such evidence directly. Do not require the pointer's arguments/results to Copy.

Coherence decides `T: FnPtr` from the outer Self type alone, as rustc 1.98
does: a rigid type that is not a function pointer has no evidence whatever its
unresolved arguments; variables, parameters, projections and opaque types are
not rigid. Stable coherence still treats a reference or fundamental nominal
over an unknown type as one a downstream crate could own. Negative coherence,
which core requests, applies rustc's builtin `impl !FnPtr for T` to each such
rigid type, so core's reference `PartialEq` impls are disjoint from its `FnPtr`
blanket. That absence needs no formation of the Self type. The
[function-pointer coherence controls](pa31/tests/spec/fn-pointer-coherence/README.md)
record independent outcomes.

The [function-pointer packet](pa31/tests/spec/function-pointers/README.md)
qualifies the ordinary standard address method through a demanded typed body:
one read of the actual Self pointer and a cast to `*const ()`, never a call.
Authenticate the exact selected method, source, scoped signature and closed
receiver. Abstract free caller lifetimes into an independent body contract
without erasing higher-ranked binders or changing the selected owner signature.
Only exact body/contract validation produces the private factory permit. Keep
primitive and function-pointer permits distinct inside the shared typed enum;
neither grants an ordinary template, const or caller lifetime exemption.
Reuse the existing instance worklist/memo, budget, ownership and cleanup gates.
Source factories prepare their own Copy/cleanup contracts and charge retained
growth rather than substituting physical Copy. Borrow canonical producer data;
do not add dense fields or a second global registry. Broader demanded signatures,
const and authenticated PA29 transport remain required and explicitly fenced
where unsupported. Declaration completion alone cannot supply their execution.

The source `pin` role completes through actual struct identity, typed fields and
the existing nominal ledger. Its pinned declaration rule adds no generic-arity,
field-count or representation requirement. Preserve ordinary field destruction;
role identity supplies neither Copy nor a drop-suppression policy. The actual
`new_unchecked` inherent method retains its source body, generic/lifetime domain,
unsafe/const effects and ordinary proof path. Authenticate its real owner and
declaration item in its source/body lifetime frame. Public callable signatures
relocate early/late coordinates and cannot be compared directly to declaration
signatures. Unused named lifetimes may extend the completed body domain after
signature formation. A zero domain consumes the declaration transaction's
completed empty scheme, never a lookup-created fallback. Publish the existing
cold completion bit only after those producer facts exist. Do not allocate a
second contract map or grow ordinary type/body records.

The [Pin source packet](pa31/tests/spec/pin-source/README.md) distinguishes this
producer from required pinning consumers. Rust-valid unused alternate declarations
do not qualify an implicit pin-construction profile. Such a consumer must check
the actual Pin owner, selected constructor, operands and result. Pinned receiver
validity requires genuine Deref/LegacyReceiver or applicable receiver-feature
proof and exact Self/lifetime identity; merely unwrapping a known nominal is
insufficient. Actual Unpin, unsafe projection, async formation, cancellation and
destruction keep their own obligations. Unsupported metadata retains its fence;
local declaration completion grants no imported execution permission.

The [source dereference/receiver checkpoint](docs/pa31-deref-receiver-checkpoint.md)
uses actual Deref/Target identity, ordinary selected methods and retained receiver
loans. Open callable binders through the existing call-signature interface before
using their children. Refine mutable places in the existing selected-call record;
lowering evaluates the actual receiver once. Smart receivers follow a bounded,
cycle-checked chain of genuine LegacyReceiver/Deref evidence and preserve full
Self arguments and lifetime debts. Selected impl arguments prefix anonymous
lifetime captures separately from named parameter slots: captured associated
definitions use the existing identity-mapped family substitution cache. The
positional shortcut is valid only for uncaptured owners. Trait selection short-circuits on its existing normalized-input evidence before inspecting a declared opaque head: a fresh inference context does not inherit that structural marker. Preserve the enclosing normalizer's actual formation/lifetime proof; semantic views retaining region obligations are not full acceptance. Broader demanded
profiles, const, authenticated metadata and async state/destruction remain
required; neither source completion nor receiver proof supplies those semantics.

Actual Receiver/Target roles share ordinary trait/associated-owner completion;
they are distinct from Deref and LegacyReceiver. Unused alternate headers keep
their source legality. Demanded arbitrary self types authenticate the actual
feature context, completed implementation, parent Target and full Self arguments.
Use one bounded, paid, cycle-checked type chain for declaration validation and
method owner discovery. Discovery queries existing owner/name indexes and matches
the method against the original outer receiver; its proof is no executable
dereference, target-field access, loan or adjustment permission. Selected calls
still consume their actual signature, full arguments, region and safety proof,
and evaluate the real receiver exactly once. Raw pointers retain their separate
feature and unsafe-call obligations. Broader shadowing, trait discovery, const
and authenticated imported bodies remain required.

Actual source Future/Output/poll roles authenticate their canonical declarations.
Async signatures create genuine opaque identity, capture Self/lifetimes/type/const
inputs and retain the authored Output equality separately from future storage.
Ordinary nested RPIT hidden inference remains active; the final async future is
not an authored hidden value. Required methods remain bodyless and authored
bodies check their declared output/effects. CFG demand rejects unsupported async
state construction before publication; it never executes the body eagerly or
substitutes a unit/empty future. Actual Pin receiver proof composes with those
same signatures. See the [joined checkpoint](docs/pa42-async-signatures-checkpoint.md).

Authored future storage uses Kind::Future with the existing opaque identity and
complete mixed captures. Reuse the existing opaque row for its optional prepaid
shape and derived application memo, and the shared bounded capture folder for
all inputs and declared Output. The checked body's internal return stays
separate, including nested opaque outputs. Layout is an anonymous state/input
sequence, without invented tuple, closure or declaration identity. Poll, Ready,
Pending and Context complete through ordinary source producers; exact demanded
polling schemas remain separate. Raw captures determine Send/Sync, liveness and
destruction; neither Context nor Output becomes an owned capture. Keep
!Unpin/!Copy/!Clone and authenticate warm applications. Storage data grants no
Future method, checked Original, generated body or execution permission. The
[storage prerequisite](docs/parallel/future-runtime-data-checkpoint.md) retains
constructor/poll/drop/await/cancellation and transport as required work.

Selected future applications consume the real checked deferred original through
the pipeline coordinator. Authenticate its current store, source body, full
capture frame and output before materialization. Keep actual defining clauses
as formation debts; never add the callee's requirements to caller assumptions.
One application owns its materialized body and optional nonempty debt payload.
Actual source Future selection uses the existing nominal formation frontier and
full Self type; Output selection authenticates the complete projection, parent,
member schema and family arguments. Storage identity and Future evidence remain
separate from constructor, poll, drop and cancellation execution authority. See
the [selected application](docs/parallel/future-runtime-selected-checkpoint.md)
and [selector](docs/parallel/future-runtime-selector-checkpoint.md) checkpoints.

Actual AsyncFn/AsyncFnMut/AsyncFnOnce evidence retains its own source hierarchy,
complete tuple arguments, real returned Future and Output/family proof. Reuse the
ordinary callable matcher, scoped substitution and shared solver frontier; do not
alias these roles to Fn. A selected method descriptor is data correspondence in
the existing paid registry, not body authority. Invocation wrappers must consume
the authenticated full rust-call method, map all inputs/results through one paid
lifetime fold, unpack the real tuple and invoke the actual receiver. The ordinary
body/region/loan/cleanup/lowering pipeline proves execution. Keep authored async
lending, state/poll/drop and imported body authority separate. See the
[finite callable checkpoint](docs/parallel/lending-callable-checkpoint.md).

Generated poll publication consumes one actual selected source materialization
and its verifier completion while the real body remains borrowed. Move the same
physical/region/loan seal through the existing finish boundary; a const completion
cannot substitute for it. Retain authored storage scope/source identities and
ordered exits, remapping only checked CFG coordinates. Authenticate the complete
actual poll Instance, three public binder slots, execution signature and canonical
Pin/Poll/Ready fields; prepare newly formed nominal applications through the
existing aggregate producer before immutable layout/ownership queries. Reprove
the generated graph before publication. Compiler-owned state access grants no
ordinary user Copy permission. The [finite scalar stage](docs/parallel/scalar-poll-join.md)
retains capture/await/cancellation/unwind/recursive cleanup/metadata requirements.

The SourceCore expansion view is `--emit-expanded-crate` (or its count form),
which runs actual configured expansion and binding completion. The PA7
`--emit-expanded` session has no source context and must reject source options.

Free unnamed constants retain real declaration identities, item keys, scopes and
all initializer typing and evaluation obligations. Omit only their lexical name
binding and declaration-name wakeup. Associated constants require names. Retain
actual visibility and emit the real unused-visibilities lint through existing
lexical policy; a public placeholder creates no exported name. Unsupported
metadata profiles reject the real constant inventory instead of dropping it.
One cold owner retains authenticated source warning causes. Success and terminal
errors share the same bounded collector and canonical warning carrier. Macro
projection uses the actual introduction; physical source and syntax context
govern duplicate warnings. Preserve the primary error and distinguish unavailable
source authentication from exhausted warning budgets. Keep the optional dense
policy table's aggregate per-node cost explicit. See the
[unnamed-constant checkpoint](docs/unnamed-free-constants.md).

Finite lint expectations belong to the same configured attribute producer, with
owner/effect/list identities, retained reasons and authenticated fulfillment.
Keep one-byte levels and four-byte semantic node row IDs; selected settings and
completion scopes live in paid cold rows. Borrow rows on ordinary reads. Actual
selected uses produce events before warning deduplication; declaring an API or
matching its spelling cannot fulfill an expectation. Keep unsafe permission and
API availability separate. Close expectations only at the successful request's
completed lint boundary, retaining untouched demanded scopes as pending. Self-
expecting unfulfilled expectations cannot suppress their own final diagnostic.
Checking a parent function does not complete an expectation inherited by an
unchecked nested body. Join unfinished bodies with their actual selected
expectation identities once per request; unrelated bodies must not prevent a
completed function scope from closing. Share this census with whole-source
completion, charge temporary indexes before allocation, and publish no final
diagnostic after a failed payment. Do not demand bodies to finish lint bookkeeping
or scan all declarations separately for each expectation. The
[selected-scope checkpoint](docs/pa40-selected-scope-completion.md) records the
opposing source, work and request controls.

During library bring-up, preserve `allow`, `deny`, `forbid` and `expect` policy,
but prioritize compiler behavior needed by actual source consumers over adding
advisory lint producers. An explicit `--cap-lints=allow` observation may defer
those producers without fabricating fulfillment. Keep that evidence separate
from strict qualification; the cap grants no syntax, feature, type, unsafe,
API, CTFE or metadata permission. Deferred producers remain named assignment
requirements and strict failures remain visible.

Authenticate the effective cap and sticky diagnostic budget before entering an
optional event producer. When Allow suppresses every event, macro deprecation
does not traverse target/module ancestry or revalidate wide source witnesses for
each call. The configured owner walk still parses and retains all annotations,
including malformed, duplicate and feature-gated deprecation errors. Known
expectations remain pending; suppression never produces fulfillment. Keep the
uncapped path and actual macro definition/execution checks unchanged. This
cap-aware boundary adds no cache or common record field.

Deprecation release data is decoded once into the existing sparse boxed
annotation, sharing the stability-version parser and pinned target release.
Actual Library staging controls version meaning: exact TBD is inactive, numeric
future releases are inactive only for staged APIs, and ordinary arbitrary
strings remain active. Validate staged since/note legality in the real owner
parser before publication, independently of use and lint cap. An inactive query
returns no event or expectation fulfillment while retaining the annotation's
actual source origin. Macro, early-warning and semantic consumers share this
result; nearest-annotation identity and explicit metadata fences remain intact.
See the [release checkpoint](docs/parallel/source-deprecation-release.md).

Legacy imported admission asserts no deprecation absence, and local deprecated
exports stay closed until explicit transport exists. The
[finite checkpoint](docs/pa40-lint-expectations-checkpoint.md) records its scope
and remaining producer, transport and release obligations.

Retain a deprecation's decoded replacement suggestion and exact argument span
in its existing cold annotation. The actual `deprecated_suggestion` crate feature
is required, including for macro output; an internal macro allowance does not
replace this gate. Preserve ordinary source feature requests without creating
source-provider authority. A monotonic hint skips validation work for producers
without suggestions, and an actual global request avoids an owner scan. Fix-it
rendering is separate from this metadata prerequisite.

Hidden glob reexport events consume completed namespace/context-aware resolver
contributions and actual successful import selections. Missing imported-origin
edges require information; only positively retained Direct selections terminate
a chain. Memoize visited binding/import identities across exported roots, pay
growth before mutation, and retain sparse authenticated causes in the same lint
owner. A private single import owns its event at its source site. Resolve public
glob export through exact alias chains rather than matching ultimate targets.
Close this early lint at completed name resolution without additional body
demands. The [finite checkpoint](docs/pa40-hidden-glob-reexports-checkpoint.md)
records namespace, effective visibility, work/storage and original-source controls.

Import lineage and lint diagnostics keep their own cumulative cost histories.
Do not transfer all prior resolver allocations into the lint producer's separate
storage ceiling. Prepay new diagnostic vector growth, witness construction and
publication with the existing lint owner; import traversal stays paid by its
original meter. Check both failed owners and the actual cap even before an empty
result, and publish no partial cause vector. Keep all exact source and binding
witnesses under both uncapped and Allow policies.

During one immutable exported-import walk, memoize complete selected-source
identities from the existing lineage intern table. Each destination still checks
its actual edge and target and records its own exported glob before reusing that
source proof. Authenticate a new selection against the exact current binding
before publishing its memo. The memo is local to that walk: source ordinals alone
cannot authorize reuse after a changed resolution. Fan-out does not require a
second visited entry for every destination that shares the same source selection.
Pay growth and failed attempts before mutation, retaining missing-edge and stale
selection errors and the original finite limits.

Import fan-out shares immutable selected-source records within the existing cold
lineage owner. Each destination/import edge keeps a checked dense source index.
Intern the complete selection: source scope, name, namespace, hygiene context,
target, visibility, origin and ambiguity, including explicit textual terminals.
Changed selections allocate distinct snapshots; never backpatch older edges from
a mutable latest-binding cache. Prepay edge-map, interning-map and source-vector
growth before publishing any entry, retaining failed charges and unchanged limits.
The [storage checkpoint](docs/import-lineage-storage-checkpoint.md) records the
original-core measurement and fan-out controls.

Ineffective unstable trait-impl events consume completed local stability rows and
actual resolved self/trait header paths, preserving alias and generic-argument
identity. Missing/imported policy is required information. Retain exact underlying
impl, wrapper and attribute-effect witnesses in a paid cold column of the existing
producer; take diagnostic spans from those authenticated attributes. Never reparse
bodies or add body demands to close this header-only lint. Local feature bounds
suppress this lint without granting availability, and untransported bounds fence
exports. Keep one-byte levels and four-byte dense policy IDs; the sixteenth lint
uses cold Policy152 and policy/completion row160 bytes. Charge owner traversal, header
work, allocation and failed attempts before mutation. The
[finite checkpoint](docs/pa40-ineffective-unstable-impl-checkpoint.md) records actual
source, invalidation, scaling and original-core boundaries.

Unsafe-operation and mutable-static-reference warnings use that same sink and
the actual retained lexical lint level. Explicit unsafe blocks satisfy the
operation policy; Allow stays silent and safe contexts still require unsafe
permission. Deduplicate original node/code identities before physical macro
projection, preserve warnings before later errors and charge failed attempts.
Do not allocate policy tables for the default path or add another warning owner.
`allow_internal_unsafe` applies to the separate unsafe-code lint; it does not
mask either of these policies. See the [qualified warning repair](docs/unsafe-policy-warning-gap.md).

Implicit compound syntax reserves canonical bootstrap identities after source
allocation, through the same cold owner as explicit language traits. Activate
the shared semantic builder only for a real typed nonprimitive/reference RHS;
retain scalar headers/factories as absent. Source-binding modes keep their
actual declarations. Implicit operator demand creates no lexical import or
method-cache entry. Preserve complete family publication and cycle checks;
see [assignment demand](docs/implicit-assignment-demand.md).

Source free-callable metadata preserves written constness, actual unsafe
signatures and defining body/context identity independently of API availability.
Reconstruct imported cleanup obligations as unpaid requests from authenticated
originals; never serialize prepared execution permission. Populate the existing
cold call-effect inventory from the actual declared-input prefix, with prepaid
growth and atomic publication. Distinguish caller feature requests from local
API policies when exporting ordinary client libraries: only a completed,
nonstaged inventory with no local regular/const policies or implications proves
that no such policy needs transport. Unknown inventory is required information.
The explicit version-eight source profile additionally retains each actual
checked unsafe call context: safe, explicit unsafe block or unsafe function.
Collect only discharged obligations in prepaid sparse body-owned storage;
authenticate the current caller/body/node/signature when reconstructing a call.
This context is neither a paid proof nor API, const or executable permission.
Keep every earlier profile's bytes and admission rules unchanged. Source type
recipe keys retain caller context whenever the existing transitive callable
summary says they contain context-owned application handles, including named
function items and their containers. Noncallable recipes retain shared keys;
do not add another scan, summary or cache to recover this ownership.
The [source-callable checkpoint](docs/pa29-source-callable-effects.md) records
both profiles, native consumers, linear costs and retained families.

An authenticated rustc_comptime declaration has effective compile-time
eligibility distinct from written const syntax. Its own body and genuine
constant roots use Const; ordinary const functions retain conditional effects,
and ordinary runtime or closure bodies gain no implicit constant context.
Legitimate function-pointer erasure retains its ordinary runtime behavior.
Validate even unused marked bodies through the existing defining-body callback
and single original-CFG owner, not a fake call or duplicate preparation cache.
Regular/const API, unsafe permission, promotion and intrinsic execution remain
separate contracts. Metadata profiles without this effective marker reject its
transport, including otherwise const-capable profiles. The
[compile-time-only checkpoint](docs/pa31-source-comptime-checkpoint.md) records
the actual source/native opponents and remaining reflection/library work.

Comptime absence on a canonical factory function comes from its authenticated
registry, current kind, binding scope and nonimported owner. It does not create
a source header or executable body proof. Imported ordinary callables instead
require their real imported declaration; a source callable whose resolver node
disappeared cannot masquerade as foreign. A budget-only ledger created after
declaration attributes were checked must inherit that completed phase before an
early initializer queries const-body context. Do not substitute a missing row
or pending coverage for a checked absence.

Generated iteration retains two distinct callables on a For plan. Check both
through the typed callable API before publication; the source node is not Call
syntax and cannot identify one callee. Reuse the existing completed plan and
canonical Option variants without reparsing, extra passes or common fields.

Source-owned iteration binds actual required methods tagged next/into_iter and
actual Option/Some/None declarations. The Iterator marker and parent trait
spellings do not confer call proof. Reuse the current body's paid protocol-plan
owner for both Try and for-loops, carrying the real For AST children, two generic
call contexts and current source witnesses into CFG construction. Source mode
must not reserve synthetic iterator/range metadata. Keep ordinary inference,
projection, const/unsafe, ownership and region checks on both real calls; use
actual variant ordinals and payloads. Batch solved-plan publication and prepaid
retention rather than creating another global For map or scanning prior loops.
The [source iteration checkpoint](docs/pa26-source-iteration-checkpoint.md)
records exact linear work/storage, native drop opponents and retained transport
requirements. Imported protocols still require authentic declarations and bodies.

Source contract helper tags are free-function identities with no fixed generic
arity. They confer no const/unsafe permission or replacement body. Consume the
real generic bodies and source call constraints. A selected source Copy/Clone
marker has no implementation/method target: use the same authenticated ordinary
role classifier for open and canonical call goals. For closures, prove that
actual trait on each typed stored capture in the current query/caller, retaining
nominal formation, lifetime and suspended-child evidence. Keep one cursor in the
existing optional language-condition owner; do not replay prior fields or use
physical layout as trait proof. A genuine contextual reborrow evaluates its
place even when a typed wildcard discards the result; only an unadjusted wildcard
remains a non-reading mention. The
[contract builder checkpoint](docs/pa31-contract-builders-checkpoint.md) records
concrete/generic/projected opponents, native effects and linear work. Contract
check intrinsic execution, attribute transformations and imported authority
remain separate required consumers.



The source rustc_no_mir_inline restriction authenticates the feature, active
function/method owner, exact effect, nonforeign definition and source image.
Use a prepaid transient declaration draft inside the existing attribute and
definition walks; do not add a persistent absence or permission query. There is
currently no Rust MIR inlining pass locally or after import, so the restriction
requires no native inline(never) mapping or new metadata fence. Existing native
inline hints, const/API/unsafe obligations and code generation remain independent.
A future MIR-like inliner must first retain the authenticated imported contract;
missing encoded intent cannot become permission. See the
[owning source contract](pa31/tests/spec/source-no-mir-inline/README.md).

Source Confusables and MustNotSuspend reuse that transient owner walk. Perform
one draft lookup per actual declaration and visit only its present annotations;
adding marker kinds must not multiply unrelated definition work. Keep the first
annotation inline and allocate cold extra storage only for mixed declarations.
Authenticate each actual image, namespace and impl/nominal owner before completion;
constructor values and associated Self rows cannot complete a type annotation.
Confusables validates every repeated nonempty string list without creating aliases.
MustNotSuspend has its own feature, word/string reason and hard duplicate rule.
Neither grants const, unsafe, mutation, ownership, drop or suspension permission.
Prepay parsing and attempted growth and retain sticky failure. The
[joined checkpoint](docs/pa31-cell-advisory-checkpoint.md) records fixed annotation
overhead as unrelated definitions grow, actual native bodies and deferred broader
diagnostic/import obligations.

Local structural inherent ownership consumes the actual crate policy and complete
member inventory. Head/name/context buckets exclude unrelated work; full Self,
const arguments, caller conditions and visibility still decide applicability.
Retain every declaration for validation, while publishing the real lexical
representative once per structural owner and exact contextual name. Nominal
overlap validation remains separate. Assemble owners from genuine autoderef
steps; autoref is a later receiver adjustment. Rebuild real reference prefixes
once for final unsizing, without inventing reference owners for a bare array.
One cold name-work counter covers ordinary and structural contexts, including
failed queries. Ordinary local feature allowances share authenticated header
machinery through a distinct cold owner; they do not become source providers or
disable the prelude. See [the structural owner checkpoint](docs/primitive-inherent-checkpoint.md).

Trait implementation restrictions retain parsed clause nodes and resolve their
module path once through the existing import fixpoint. Validate the real
declaration and implementation module ancestry; visibility grants no additional
implementation permission. Syntax feature checks include parsed disabled
headers and authenticate actual macro-token origin. Ordinary readable metadata
can assert an unrestricted domain only through its checked canonical profile,
trait header, owner and inventory. Missing source nodes or map entries are not
policy evidence. Restricted source publication stays closed until its explicit
metadata contract is implemented. See the
[domain checkpoint](docs/implementation-domains-core-checkpoint.md).

Source macro imports and qualified calls use actual declarations and the existing
import dependency worklist, including forward definitions and renamed targets.
A final `option_env` spelling cannot create bootstrap Option or builtin macro
definitions inside a source provider. Keep builtin expansion semantics separate
from ordinary macro lookup; resolving an authored macro with that name grants
no compiler role. The [source macro path controls](pa31/tests/spec/source-macro-paths/README.md)
record the original whole-core failure and independently qualified inputs.

Source formatting produces typed syntax from decoded literal data and selects
the actual `format_arguments` nominal's constructor through the shared inherent
method query. Preserve the invocation's contextual lookup, readiness dependencies,
visibility, full signature, lifetime and const obligations. Lower its checked
call through ordinary body operations; a library-value tag cannot fabricate a
layout or replace execution of the source constructor. Nonempty argument support
must preserve evaluation order, single evaluation and source-owned formatting
roles. `allow_internal_unsafe` is lint metadata and never authorizes an unsafe
operation. See the [formatting contract](docs/pa31-source-format-contracts.md).

Source primitive methods use actual inherent declarations and ordinary body
proof. A same-named module does not replace a primitive type: after ordinary
relative reference lookup, a missing primitive-headed member may retain the
canonical primitive associated prefix. Successful module members, nominal/alias
shadows, privacy, ambiguity and import rules keep their ordinary authority.
Do not manufacture a length operation in place of the real str/slice/pointer
method chain. See the [source method checkpoint](docs/source-assert-implementation.md)
and [PA6 fallback controls](pa6/tests/spec/primitive-members/README.md).

Source byte concatenation reuses the eager producer's parsed argument arena and
typed literals. Authenticate the actual source-declared builtin before preparing
inputs, including opaque captured payloads. Validate operands and compute the
checked output length before reserving the result; charge requested work and
payload before allocation, including overflow and failed attempts. Fill the
owned byte string through one further bounded typed walk, preserving byte order
and zero-repeat element validation. Neither pass reparses source or rendered
spelling. Keep source role permission separate from ordinary eager builtins,
formatting constructor proof and foreign API admission. The
[byte contracts](docs/pa31-source-concat-contracts.md) retain independent source
and native observations, budget opponents and measured storage costs.

Library feature requests come from enabled source declarations and authenticated
dependency inventories, separately from compiler language capabilities. Complete
their validation after configured expansion; unknown requests grant no capability.
Retain regular, const and default-body API stability as distinct declaration facts,
separate from callable effects and each caller's proof. Compute owner inheritance
and feature indexes once, then check actual uses without whole-crate rescanning.
Const API policy targets are distinct from semantic constness: actual ordinary
traits, const/static declarations and crate roots can own policy, while a const
annotation on an import supplies no target override. Associated functions,
constants and types inherit the real container's unstable policy without gaining
const execution. Direct function annotations still require actual written
constness or authenticated comptime eligibility; keep forbidden target checks.
Reuse the existing optional const axis and parent-first scope traversal. The
[owner-policy checkpoint](docs/pa31-const-policy-owners.md) retains the original
provided-body compilation failure and its unchanged source/native successor.
When a library has an authenticated SourceCore or local macro cost owner, capture
and completion borrow that existing payer. Inventory work and attempted storage
accumulate with earlier source work under its immutable configured limits;
accounting supplies no semantic permission. Retained inventory storage stays in
the cold source meter across later macro memo checks. Prepay a conservative live
draft bound, release capture-only indexes at completion, then keep the completed
policy/declaration/import/const storage reserved. Standalone staged libraries
without that cost owner retain their finite fallback; ordinary records gain no
policy or permission fields. Reserve owner tables by their scope, declaration
and import axes, and prepay the used-policy set before insertion. The
[inventory-budget checkpoint](docs/pa31-inventory-budget-checkpoint.md) records
qualified bounds, scope/layout controls and the unchanged original inputs.
Request canonicalization borrows that same authenticated source payer; it must
not apply the metadata ceiling to already-paid source inventory work. Reuse the
fallible index-sort kernel with a caller-owned charge callback. Pay comparison
bytes, index work and attempted scratch, checking scratch in addition to the
retained inventory. Publish the final label permutation only after all fallible
work succeeds. Preserve sticky failed debt, default metadata and ordinary lookup
limits, finite cardinality and the inventory's original cumulative history.
Aggregate and variant fields inherit stable as well as unstable regular policy
from their actual parent. Retain one prepaid cold field/parent/policy index;
the typed field producer authenticates each configured source field against it.
Direct policy overrides inheritance; unknown rows and copied parents are errors.
Keep const-marker target rejection, visibility, ordinary field semantics and
imported availability distinct. Charge growth before publication and retained
capacity in the shared inventory meter. Add no per-field hot policy column or
sibling scan at use sites. The [field checkpoint](docs/pa31-field-policy-checkpoint.md)
records actual source/native controls and remaining transport requirements.
Represent required unstable features as non-type clauses with Store-owned canonical
identities. Keep item assumptions separate from immutable crate enabling and
regular API permission; a lint cap supplies none of these proofs. Staged libraries
must use actual caller assumptions. Share paid cold premise/index storage, retain
requirements before lifetime-only filtering, and authenticate the real caller
when materializing a checked instance. Never substitute the callee's own marker
as evidence or drop an untransported clause into an older metadata profile. The
[typed checkpoint](docs/lib060-feature-predicate-core-checkpoint.md) records the
implemented ports and remaining source/transport work.
Macro-internal allowances follow exact defining/captured token provenance. The
local header implementation combines complete identifier lists and repeated
attributes, stores canonical NFC membership separately from exact raw source
lexeme/Attribute/Forest witnesses, and shares one immutable cold header with its
actual macro definition. Unknown names remain inert data. A borrowed Report
session checks the raw token's outer mark, authenticates the published header's
actual attributed owner and memoizes positive and negative header/frame results.
Warm queries still validate identity and budgets; captured tokens cannot inherit
a literal's allowance and ancestor grants are not unioned. Retained-origin links
remain a separate per-use traversal cost and must be measured with depth times
uses. This provenance query does not authorize foreign API use or const calls;
those consumers and their authenticated metadata remain required. The two panic
edition consumers test the precise `edition_panic` membership.

Generic source attribute classification pays the existing cold attribute-policy
producer's bounded read work, separately from macro permission/header/frame
work. Retain the source macro State's sticky failure check before classification
and all genuine permission and storage charges. Ordinary source allocation
behavior remains unchanged; a general cost payer grants no feature permission.
Traversal frame publication and frame storage have different costs. Releasing a
frame keeps reusable capacity; do not charge another allocation when a sibling
reuses that slot. Prepay actual capacity growth and its copy work through the
existing payer before publishing the row or extent. Retain every attempted read
and publication step, including failed attempts, and distinguish work/storage
exhaustion without changing either limit. Persistent source owner/witness/call
records keep their own charges. See the
[frame-storage checkpoint](docs/lint-policy-frame-storage-checkpoint.md).
Retain attribute witnesses in exact owner-local arrays: checked compact headers
and one flat typed-tree payload. Count and prepay both arrays before allocating;
validate each header's exact coverage, order and final extent on reuse. Empty and
nonlint attributes remain positive source witnesses. Do not allocate a separate
vector per attribute or discard provenance to meet a storage limit. The
[compact-witness checkpoint](docs/lint-policy-compact-witnesses.md) records actual
layout, bounded-work opponents and the unchanged original-core observation.
See [the classification checkpoint](docs/source-attribute-classification-work.md).
See [the local allowance handoff](docs/pa31-internal-allowances-handoff.md) for
qualified component evidence and remaining integration requirements. The
[stability investigation](docs/pa31-library-stability.md) records the independent
cases and required implementation seams. The
[local inventory candidate](docs/pa31-local-stability.md) uses a single cold,
resolver-owned policy carrier and the existing configured-owner traversal.
Publish it with source reservations only after both drafts complete. Follow
actual module/scope and import-target identities, excluding local and synthetic
bindings from API inheritance. Stability-bearing metadata must reject until its
transport and use consumers exist; regular policy cannot grant const-callability.

Reached source offset queries consume evaluated variant/field operands and the
ordinary completed aggregate Layout. They must retain current source-header and
checked-call proof on both cold preparation and warm reads. An Instance-only
size/alignment result cache cannot own field offsets. Publish source Option's
physical nullable policy from actual declaration/field facts before nominal
completion can expose a concrete layout; reuse the existing cold table and
prepay retained growth. No whole-store rescans or parallel layout authority.
See [the reached kernel and remaining builtin obligations](docs/pa31-reached-offset-kernel.md).

Source-role semantic attributes are validated claims on exact declarations.
Sparse item feature sites share one source-incarnation witness, transaction and
monotonic retention envelope across default modifiers and trait aliases. Keep
whole alias-item spans distinct from default keyword spans; neither source copies
nor macro root replacement may erase their token authority. Defaultness queries
must filter alias rows, so aliases cannot allocate specialization state. Retain
actual alias identities and lexical Self/generic scopes without ordinary trait
owners, associated member scopes, impl entries or synthetic Self:Alias evidence.
Typed expansion must share canonical dependency edges rather than repeatedly
flattening nested alias diamonds. See the
[source-site checkpoint](docs/trait-alias-sites-checkpoint.md) and
[canonical clause checkpoint](docs/trait-alias-clauses-checkpoint.md). Canonical
headers own full declared slots and typed dependency edges. Store written clauses
once in the ordinary generic predicate table under the actual alias owner;
expansion, default checks and unused-declaration formation borrow those same
clauses. Feed the actual alias schema to ordinary scoped-region validation.
Inherited edges stay separate instead of copying their clauses into each default
or header. Seal the graph before use and distinguish unfinished, failed and empty
contracts. Substitution-only expansion may share identical actual declaration,
subject and full-argument applications within one Store and binder context.
That membership is not proof evidence. Compose nested universal alias edges
through the existing validated typed-input opening, preserving unused slots and
callable locals. Retain one canonical surrounding binder per application instead
of copying its ancestor path; the [binder checkpoint](docs/trait-alias-binders-checkpoint.md)
records changed-path scaling. Retain original/normalized inputs and
binder identities when their consumers are implemented; incomplete consumers
and imported transport keep explicit fences.

Direct aliases without explicit formals, lifetime-bearing or quantified clauses,
const requirements or inherited alias edges feed their canonical predicates into
ordinary body, ownership and instance checking. Declaration formation remains
mandatory even when unused. This local execution boundary creates no alias trait
owner, object principal or metadata identity. Public aliases still reject through
the existing export profile; wider forms retain their explicit execution fence.
The actual reduced source `Thin` consumer checks unit metadata through full
lowering and rejects the slice metadata mismatch.

Alias naming consumes the declaration-Self view, while proof retains the complete
contract. Filter original typed subjects before actual substitution or result
normalization can make another formal or associated result equal Self. Reuse the
same dependency graph, charged substitutions and ordinary member/binder machinery;
do not manufacture a trait identity or clone a second flattened graph. Preserve
written edge subjects and existing unreduced clause facts. The
[naming checkpoint](docs/trait-alias-naming-checkpoint.md) records the independent
counterexample, actual consumer and selected-table cost. Associated-family input
prefixes include every emitted alias requirement, including equalities and
builtin clauses. Associated-result aliases use the same bound worklist and
canonical expander, but their promises stay outside that input prefix. Preserve
implicit result Sized separately. Object principal multiplicity is separate
from deduplicated proof/naming clauses. The
[combined checkpoint](docs/runtime-header-integration-checkpoint.md) records
actual projection-use opponents and unchanged alias execution/export fences.

Legacy bootstrap diagnostic compatibility authenticates declarations against the
existing fixed canonical registries, actual kind/public binding and nonimported
identity. A missing node is insufficient. This compatibility creates no completed
source/imported header, KnownAbsent attribute or execution permission; genuine
source and artifact contracts keep their own producer/Unknown rules.
An optional promotion probe reuses that authenticated bootstrap ownership and
returns Unknown for a source-only promotable fact. It must neither demand a
nonexistent source-header inventory nor invent checked attribute absence. Missing
or forged registry ownership remains an error. Restrictions whose absence grants
call permission retain their separate complete-coverage requirements.

Final trait-method queries distinguish a missing source node from an authenticated
imported required declaration. Read the actual catalog handle and matching
declaration header: explicit body absence establishes that the required method
cannot be final. A provided body still requires its actual final policy; no
default modifier or unavailable body environment can establish that absence.

Source Abort requires the actual current attributed header, complete call
instance and reached caller proof. Consume a transient witness for the final
Evaluate and original Unreachable edge before emitting the existing terminator;
stop subsequent cleanup only on that path. Do not fabricate a return value or
use an interposable library abort symbol for this intrinsic. Const calls,
reification, imported origins and assertion/panic construction remain independent
contracts; the [runtime checkpoint](docs/source-abort-runtime-checkpoint.md)
records the genuine owned-scope and native opponents.

Source Transmute consumes an authenticated full call and the existing pre-monomorphization
size relation. Runtime lowering moves through owned ABI storage without dropping
the source; destination destruction remains ordinary. CTFE shares the existing
byte codecs and paid reference continuation, preserving pending duties and
original allocation provenance. Function-pointer formation preserves the genuine
canonical target and destination pointer type; it does not require identical
signatures or grant const invocation. The actual runtime call uses the destination
ABI with normal unsafe and ownership checks. Do not add a parallel origin graph,
signature-recovery cache or synthetic trampoline. The
[formatter checkpoint](docs/formatter-pointer-erasure-checkpoint.md) records real
const formation, borrowed-owner runtime calls and the combined integration.

Source TransmuteUnchecked is a distinct attributed identity with the genuine
indirect-const/header policy. Keep ordinary transmute's pre-monomorphization
size relation separate from unchecked body typing. Complete call/instance,
current source and unsafe/API/const proofs still precede shared runtime/CTFE
reinterpretation. Reached mismatched constants fault; destination validity,
initialization, moves and provenance are unchanged. Unsupported runtime layouts
and old declaration/body export profiles remain explicit boundaries. Reuse
existing descriptors and byte/layout kernels rather than adding a parallel
operation or origin graph. The [unchecked checkpoint](docs/pa28-source-unchecked-transmute-checkpoint.md)
qualifies genuine outer NonZero new/get with an explicitly closed inner carrier.

Source `arith_offset` consumes its actual attributed const unsafe declaration,
complete one-type instance and checked reached call. Runtime and CTFE evaluate
both operands once, including zero-sized elements. Multiply the signed word by
the selected element size modulo the target word width and preserve the pointer's
original provenance and write restrictions. Leaving an allocation is permitted;
accessing memory still requires the ordinary validity checks. Flatten repeated
symbolic offsets to the original root, avoiding a growing projection chain.
Share existing layout and byte-index kernels without weakening in-bounds offset
semantics. Unsupported value representations and untransported source contracts
remain explicit boundaries. See the
[wrapping-offset checkpoint](docs/pa31-source-wrapping-offset-checkpoint.md).

FunctionItem signatures retain their function binder; an opened Body or Code
Call already stores roots in the caller. Match complete item substitutions by
canonical readonly lowering of exactly that outer binder, preserving nested
local binders. Validate opened Calls separately against exact caller-level
input/result IDs, zero own binders, safety, convention and arity. Retain the raw
item signature for actual target identity. Reuse these contracts across source
transmute and wrapping offsets, including dependency, runtime and reached CTFE
checks; the type comparison grants no source, unsafe or const permission.
Charge attempted lookup work on success and failure and publish no result after
failed payment. See the
[borrowed-call checkpoint](docs/pa31-borrowed-source-calls-checkpoint.md).

Share structural layout readiness and field instantiation across constant
storage and source intrinsic consumers. A layout answer supplies physical data,
not trait, source-call or memory-access permission. Authenticate the reached
source call before demanding layout. Retain pending declarations distinctly from
completed failures, and reuse the existing substitution graph rather than
implementing a second layout or field walker. Charge actual cold dense-row
initialization, graph work and failed attempts; completed warm queries must avoid
rebuilding the dependency graph. The
[source storage checkpoint](docs/source-storage-contracts-checkpoint.md) records
the genuine associated-constant wrappers and native evidence.

Closed constant layout with projected field templates consumes the existing
stored-field Preparation/Query and a fresh concrete normalization proof.
Independently discharge retained lifetimes before readiness or layout-cache use;
shared structural recipes grant no new caller authority. Allocate disposable
regions beyond the actual BodyRegions frontier. Charge graph, caller and nested
work between each producer step, including failures, before advancing another
child. Missing nominal field templates retain their declaration wait. See the
[projected-layout checkpoint](docs/pa31-closed-projected-layout.md).

A generic body's transmute of a parameterized nominal with projected field
templates, such as core's `Atomic<*mut T>`, normalizes those fields in a
premise-free session. Selection uses only validated implementations, and the
proof is discharged closed. The published storage keeps parameters but no
projection, and no caller replays its proof. A premise-free build re-enters a
parameterized root that an ordinary build published, once, to reach nested
fields. rustc prefers a body premise over an implementation and keeps such a
projection rigid. So a body whose elaborated premises include a trait that
declares associated types, or a quantified premise, keeps the closed boundary
when its transmuted storage reads those fields. This holds even after another
body has published that form. See the
[parameterized storage checkpoint](docs/pa31-parameterized-storage.md).

Without the receiver features, a source method call still discovers inherent
owners along the receiver's genuine LegacyReceiver/Deref chain when some
inherent method of that name declares a validated smart receiver, such as
core's `SyncView::as_pin_mut(self: Pin<&mut Self>)`. Matching uses the
original outer receiver, as with the feature chains. The chain's proof opens
the caller's bound lifetimes into fresh regions beyond the body frontier. It
grants no adjustment, loan or region evidence. Other method names skip the
chain. See the [pinned method call checkpoint](docs/pa43-pinned-method-calls.md).

A source comparison whose operands are not two primitive scalars gives
`Rhs` a fresh variable, as rustc's `check_overloaded_binop` does. It refines
its own `Self: PartialEq<Rhs>` (or `PartialOrd`) goals where it can, and only
then coerces the right operand to `Rhs`. Core's `*const T != *mut T` thus
compares two `*const T`. Two raw pointer operands, under one shared reference,
are then builtin scalars, so the coerced right type must equal the left. The
coerced comparison stays a primitive operation. See the
[comparison right operand checkpoint](docs/pa31-comparison-rhs.md).

Borrow checking renumbers each operand's regions, so that comparison's `Self`
is the left operand's type with fresh regions. Its goal uses that `Self`, and
the left borrow constrains it. A body lifetime parameter thus never reaches the
`Rhs` selected beneath the method signature's binder. A right operand whose
coercion to `Rhs` is an identity stays in its own type and is borrowed in it.
The borrow is constrained to the `Rhs` parameter, as a receiver borrow is.
Invariant operands with different lifetimes fail the prepared lowering
lifetime proof, which rustc's borrow checker also rejects. See the
[comparison body lifetime checkpoint](docs/pa31-comparison-body-lifetimes.md).

A destructuring assignment `assignee = value`, with a tuple or `_` assignee, is
rustc's `{ let <pattern> = value; leaf = binding; ... }`. Tuple shells take the
tuple of their elements' types, `_` takes a fresh variable, and every other
element is a place leaf that must be assignable as `leaf = value` is. The
right operand coerces to the assembled type and is evaluated into a place
first. Each leaf's place is then evaluated, and its field moved or copied
into it, from left to right. `_` and unread fields stay in the right
operand's place. Rest, array, struct and tuple struct assignees fail closed.
See the [destructuring assignment checkpoint](docs/pa31-destructuring-assignment.md).

An `if` or `match` gives its branches one type, as rustc's `CoerceMany` does.
An expected reference to a slice, a `dyn` type or a nominal storage tail is the
common type, and each branch coerces to it. Without an expectation, branch
types join in order. A later reference coerces to the earlier one, unless
reference depth or mutability chooses the other direction. Between two
references of the same depth and mutability, an earlier array referent unsizes
to a later slice referent. Other earlier-to-later unsizings fail closed. See the
[branch unsizing checkpoint](docs/pa31-branch-unsizing.md).

A closure without an expected signature gets fresh parameter types. Its body
can give a parameter a reference type whose region the body creates, as in
`|a, b| Cmp::cmp2(a, b)`. That region is external to the closure body, as
rustc's borrow checker makes every free region of a closure signature. It
becomes an output port: the parent gets a fresh region, the body a universal
slot, and the body's requirements on it become the parent's obligations.
Imported input regions keep their existing creator alias identity. Generic
noncapturing instances match complete typed roots: inherited arguments stay
exact, public late slots stay rigid, and private input ports connect outside
the public binder. Keep every repeated-region equality and projection debt in
the instance proof. Public and body proof signatures retain separate identities
and require physical ABI agreement. Noncapturing source originals retain their
creator's declared trait context with child-owned aliases; instances reprove
their own reads. Structural marker siblings allocate distinct proof witnesses.
See the [closure input region checkpoint](docs/pa31-closure-input-regions.md) and
[inferred-input repair](docs/pa24-inferred-inputs.md).

An array pattern without a rest fixes an array length that is still an
inference variable, as rustc's `check_array_pat_len` does: the array equates
with `[T; n]` for the pattern's `n` elements. A rest with an unknown length and
a const parameter length fail closed. See the
[array length pattern checkpoint](docs/pa31-array-length-patterns.md).

A closure whose signature inputs mention regions imported from its creator
bridges every creator-owned universal. Requirements between those regions that
the body cannot prove become creator obligations, as rustc's closure
requirements do. A closure also assumes the lifetime requirements implied by
its creator's signature, but not projection requirements. rustc instead
propagates a type-outlives requirement to the creator, which proves it with the
same implied bounds. An annotated elided parameter lifetime stays in the
closure's own binder. See the
[closure creator requirement checkpoint](docs/pa31-closure-creator-requirements.md).

A `match`, `if let`, `while let`, let-else or `let` whose value is a captured
place walks its selected patterns, as rustc's capture analysis does. These
patterns use the place shared:

- a multi-variant enum variant test;
- a literal or range test;
- a slice length test;
- a `ref` binding.

A `ref mut` binding borrows the place mutably. A by-value binding copies it when
the binding's type is Copy, and otherwise moves it. A use below an enum variant
or an index projection captures the scrutinee place. A use below a field
projection or a dereference would capture a more precise place and fails
closed. That does not apply in a whole-variable edition, or to a `move`
closure's leading dereferences. A scrutinee that no pattern uses is only
mentioned. A compound `let` pattern on a computed value consumes the value. See
the [closure capture pattern checkpoint](docs/pa31-closure-capture-patterns.md).

A trait premise proves a requirement of the same or a weaker effect, as rustc's
`BoundConstness::satisfies` does. Every premise proves an ordinary
requirement, a `[const]` premise also proves a `[const]` requirement, and a
`const` premise proves all three. An always-const supertrait, such as
`SimdElement: const PartialEq`, therefore gives `T: [const] PartialEq`. A
`[const]` requirement with both a `const` and a `[const]` premise for the same
inputs stays ambiguous, as in rustc. A scoped equation between two
requirements still compares effects exactly. See the
[always-const premise checkpoint](docs/pa31-always-const-premises.md).

A body that opens its declaration's lifetimes at universal aliases carries the
declaration's certified projection premises. Every opened source must be one
of the declaration's trait premises or a `Sized` subject it assumes, such as
the `Sized` elaborated from a `Clone` premise. The opened
`P: Pattern<Searcher<'a>: Clone>` premise thus relates a goal for the impl's
fresh region invariantly, and the region equality goes to the lifetime proof.
See the [GAT region premise checkpoint](docs/pa31-gat-region-premises.md).

The bounded sizing profile requires actual sizing identities, complete headers
and deny-explicit-impl restrictions before retaining fundamental, specialization
or coinductive markers; those markers grant no general implementation or cycle
permission. Forced dyn restrictions consult the completed actual role, while
diagnostic identities and field-read hints grant no move or body permission.
PartialEq and PartialOrd diagnostic hints may describe ordinary module traits
without declaring their language roles. Preserve these hints separately; an
actual language claim must have the matching owner, and operator consumers still
require their completed language identity and full operation contract.
Keep this state in fixed cold provider tables and preserve real provided bodies
and conditional Environment clauses. The [capability checkpoint](docs/pa29-provider-capability-checkpoint.md)
records actual checked originals and containing layouts.

Pointee declaration production seals the actual trait, Metadata member and
DynMetadata nominal contracts in one cold shared owner. Retain their real
source-incarnation witnesses and completed declared prefixes; a derived
environment suffix may grow without changing the declaration. The
[declaration checkpoint](docs/pointee-source-declarations.md) supplies no
automatic implementation, metadata normal form or imported execution permission.

Imported executable originals have authenticated external declaration/body/site
coordinates. Install their complete canonical header graph before original
proof, without inventing source AST nodes or marking body proof complete when
headers become ready. Keep declared and relocated callable signatures distinct.
Derive mandatory formation and flagged Sized requirements for every actual
formal, including unused parameters; preserve explicit `?Sized`. Original
assumptions do not pay substituted instance obligations. Retain a single cold
owner for immutable imported templates/contracts, with exact owner validation
at every borrowed view and ordinary per-instance proof on execution.

The defining crate checks every required source body before export. Retain the
complete family and raw body contracts, but reconstruct imported defining
environments and original proof on actual body demand. Unrequested bodies remain
explicitly Pending, never empty or proved. Reuse completed original proof only
under the same immutable body, Store and environment; concrete instances retain
their fresh ordinary obligations. Declaration conformance and genuine namespace
installation use declaration facts independently of body readiness. Imported
names alone grant no semantic or execution permission. The
[execution plan](docs/library-bootstrap-acceleration.md) records this reviewed
policy and the unchanged complete default/const transport requirements.

The explicit source module-availability profile uses tag 10, row version 3.
Complete declaration and binding rows retain regular/const policies, real
declaration/site or binding/site origins, and positively checked private absence.
An immutable imported catalog owns the canonical remap; missing legacy policy
stays Required. Check actual target and module-prefix availability independently
of local feature inventories. Reexport annotations retain provenance without
erasing the target gate or introducing an extra ordinary-call requirement.
Availability grants no body, const execution or provided-default permission.
Build ancestry intervals once and sort source-owner candidates once; charged
binary predecessor/containment queries replace repeated whole-candidate scans.
Keep per-use and dense body records unchanged. The
[transport checkpoint](docs/pa31-availability-transport.md) records genuine
fresh-client/native evidence, counterexamples and bounded cold/warm work.

The explicit module-feature-predicate profile uses tag 10, row version 4, with
all v3 availability and namespace columns. Preserve whole non-type requirements
as paid canonical spellings; they neither define library features nor replace
API permission. Fresh primitive trait-method bodies retain real member links,
unpaid formation inputs and fresh dispatch before executable publication.
Closed scalar operations carry sixteen little-endian bytes whose kind comes
from the actual value type. Reuse scalar width, bool and Unicode validation,
preserving floating bits exactly. Keep these bytes inside the existing cold
operation payload; do not enlarge dense body/node records or add a literal
lookup table. Prepay the reconstructed scalar pool, then validate exact types,
bits, sequential slots and complete storage against the immutable source view.
Old v0-v3 bytes and admission rules, and the separate Provided profile, retain
their boundaries. Nominal bodies, promotions, exact C linkage and const execution
remain separate requirements. See the
[v4 checkpoint](docs/lib060-feature-predicate-metadata-v4-checkpoint.md).

The ordinary source module-definitions profile5 adds cold exact-callable rows
bound to actual declarations, root originals and exact symbol bytes. Reuse the
checked foreign registry for C signatures and linkage. Fresh clients import
those symbols; they do not define provider bodies again. Prepare names and
callable records before the existing infallible publication boundary. Charge
registry storage and rehash work only for actual capacity growth: repeatedly
paying the entire existing map for each small provider is quadratic even when
no allocation occurs. Keep old source/Provided wire bytes and fences intact;
nominal, receiver, generic/const and ordinary-provider extensions require their
complete consumers. See the [v5 checkpoint](docs/pa31-source-exact-linkage-v5.md).

Local staged `implied_by` relationships belong to exact policy IDs, including
independent regular/const annotations. Retain them in one paid cold inventory
vector; reuse the completed library-feature definition index for target
validation. Language-feature requests do not invent library definitions, and
relationships do not become global or transitive predicate assumptions. Keep
existing artifact profiles closed until their per-policy relationship columns
and fresh permission consumers are complete. See the
[ConstParamTy checkpoint](docs/pa31-const-parameter-derive-prerequisite.md).

The source `CoercePointeeValidated` role preserves its actual ordinary trait
header and required members. Every explicit implementation, including negative
and blanket implementations, requires the genuine builtin derive origin. Read
that origin through the existing selected-member transaction, whole source/header
and incarnation authentication shared with `ConstParamTy`; documentary
`automatically_derived` attributes and other builtin derives confer no authority.
Require the completed actual target to be a nonempty transparent struct at normal
implementation validation. Gate these queries on the actual role identity before
reading origins or fields; ordinary traits with the same spelling retain their
normal behavior. This supplies neither automatic implementation evidence nor
nominal coercion, dispatch, executable derive output or metadata permission.
Full builtin generation and generic custom auto traits remain required work.

`ResumeTy` shares the completed ordinary nominal producer with `Context` and
coroutine state. `get_context` shares the defined-function header adapter with
`from_yeet`; schedule its actual header through the existing declaration worklist
before completing the role ledger. Preserve all written fields, generic and
unsafe/const flags, body identity and ordinary ownership/region checks. A resume
tag supplies no generated future, special pointer conversion, automatic trait
evidence or lifetime permission. Authored dependency companions and full original
standard-library source qualification remain distinct acceptance requirements.

Range language declarations reuse that ordinary nominal producer. Complete the
actual inherent inclusive constructor with Pin's shared signature/body-frame
adapter, preserving unused alternate-header legality. For source record forms,
select the completed real language struct and infer its normal lifetime, type
and const arguments through the same constructor kernel as written paths.
Use actual named fields, adjusted syntax context, privacy and substituted field
types; never force a single shared endpoint type or declaration-order operands.
Evaluate start before end, then assemble values in actual field order so loans,
moves and Drop remain ordinary. `new_range` is a captured crate feature selecting
its genuine Copy-named declarations and `last` fields; those spellings supply no
Copy evidence. No fake lexical application row, bootstrap source fallback,
source reparse, retained range map or wider dense node is required. Legacy
inclusive syntax authenticates both actual completed source header images and
uses the ordinary selected-call table, callable instantiation, late-binder opening,
argument refinement/coercion and call finalizer. Its real body initializes the
private fields, including exhaustion state; const evaluation and native calls
consume that same body. No protocol-plan variant or dense node field is added.
Early array-length constants still require nominal outlives preparation; their
explicit-call and syntax reductions remain Rust-positive requirements. Full
module/derive/iteration and metadata closure remain acceptance requirements.

C string literals use the completed actual `CStr` language declaration's item
type and a static immutable reference. Unused alternate headers retain ordinary
legality; literal typing does not invent constructor arguments or validate a
storage shape prematurely. Materialization consumes the actual transparent
single byte-slice field, its prepared ABI offset and length metadata. Keep a real
sized array of the actual signed or unsigned character element as immutable
backing, with the nominal reference retained as its logical view. Key literal
locations by canonical bytes and physical type; equal bytes do not replace typed
storage identity. Intern the implicit NUL in the existing byte arena without a
temporary payload copy, preserving ordinary interning across word boundaries.

Runtime, promotion and CTFE consume those same typed text facts and one storage
producer. Real source methods retain normal const, unsafe, lifetime and loan
checks. Completed ABI cache readers neither run a second solver nor invent thin
metadata on a miss; their requesting producers pay the real query work. Byte-word
reconstruction retains initialized provenance fragments and the original static,
write, alignment and extent checks. The finite zero-offset full-byte-tail view
uses the genuine source field and sized backing, with signed/unsigned byte views
admitted only by the existing checked raw conversion. It grants no ordinary
slice coercion, arbitrary nominal DST interpretation or authenticated metadata
transport. Whole modules, derives, constructors and library artifacts remain
separate required qualification.

Source Reborrow and CoerceShared keep ordinary declaration/implementation proof
separate from operation selection. A contextual nominal destination must exist
before checking its expression; later equality cannot turn a previous move into
a loan. Preserve selected implementation equalities and use the actual selected
destination as the operation's loan type. Propagate contextual borrowing through
transparent blocks to the actual source place, without duplicating effects or
materializing a move through a reference. Prepare the real runtime storage via
the shared typed field producer; matching physical layouts supply no permission.

Carry sparse, authenticated source/body/instance/store/operation/place inputs
through template substitution, native copying and immutable const snapshots.
The existing optional source catalog owns semantic Reborrow selections. Preserve
the ordinary compilation-root layout and charge actual lazy map/row retention
through its established payer; keep body inference goals and checked CFG inputs
in their own existing owners.
Preflight their context before remapping. Release pending goal indices at the
inference-arena boundary. Loans, guard protections, initialization, storage,
unsafe access and promotion remain ordinary; no implicit Clone call is introduced.
Validate operation coverage in one statement pass with an existing sorted index,
not a full statement scan per claim. Declared field width per operation remains
an explicit scaling dimension. The [operation checkpoint](docs/parallel/reborrow-operations-checkpoint.md)
binds source/native/MIR observations and required wider/transport work.

Growing constructor inputs must share completed inference prefixes. Reuse the
existing postorder occurs/freshening walks to record immutable graph facts;
holes, projections, lifetime-bearing fresh positions and callable-template
rebuilding retain their ordinary work. A normalized current callable signature
does not prove its declaration template free of new projection occurrences.
Do not intern canonical types merely to obtain a completion certificate.
Canonical sizing conjunctions share the existing body's completed-proof owner
with individual selections in the same fixed caller and mode. A concrete,
debt-free positive proof of an actual source sizing trait also proves that exact
subject's nominal formation. Its formation-only wrapper retains `source=None`,
the original allocation envelope and existing frame equalities; it selects no
parent trait. This reuse is restricted to root scope and zero callable depth,
with current role validation. Plain builtin Sized, pending, provisional, cyclic,
scoped or debt-bearing answers supply no such certificate. Require doubled-depth
scaling for both inference and canonical proof work, with unrelated declarations
leaving those counts unchanged; a single faster traversal does not establish a
linear constructor pipeline.

Former-name links share that same cold relationship owner with an explicit kind;
they never enable the canonical feature. Project canonical feature definitions
once, keeping the first unstable declaration and checking stable/unstable or
release-version conflicts, including const policies. Old names are opaque strings,
not extra feature definitions or implication targets. Validate root requests with
a prepaid borrowed index, preserving canonical-name collision precedence; do not
scan every alias per request or semantic use. Current release means the pinned
Rust language version, independently of the host compiler. Keep each actual policy,
const eligibility, metadata provenance and unencoded-relationship fence. See the
[renamed-feature packet](pa31/tests/spec/renamed-features/README.md).

Build the canonical library-feature index during policy capture and reuse it at
completion. Share validated spelling data while retaining each actual policy's
owner, kind, release and relationship identity. Keep release conflicts deferred
to the original completion boundary; a later malformed attribute must retain its
prior error precedence. Borrow temporary parsed argument text rather than
allocating since/issue strings. Prepay lookup bytes, unique validation/copy/
insertion, map growth and retained-spelling rehash work before mutation. Failed
payment publishes no canonical entry. Drop the cold index after its final phase;
no extra map belongs in the completed inventory or ordinary body/AST records.
The [shared-feature checkpoint](docs/pa31-shared-feature-inventory-checkpoint.md)
qualifies the unchanged resource limits, real4096-function source/type consumer,
paired output equality and calibrated measurements. Preserve exact source and
configuration provenance when refreshing original-library observations.

Default-body stability is a third direct policy axis, distinct from regular API
and const stability. Capture all three with a single typed owner/axis index;
retain exact source policy identity without adding fields to ordinary syntax or
bodies. The pinned attribute target list includes bodyless declarations; do not
manufacture bodies or reject valid targets by assuming only provided methods.
Default-body policy is not inherited API instability or const eligibility. Local
implementation bodies and explicit overrides use the ordinary checked pipeline.
Only external inheritance requires the body feature; an existing provider-side
impl remains callable through its stable API. Old profiles without complete body
policy and provided-original transport must reject publication. The
[default-body packet](pa31/tests/spec/default-body-stability/README.md) binds
local/native controls and real cross-crate reference consumers. The
[qualified source checkpoint](docs/pa31-default-body-stability-checkpoint.md)
retains the required imported consumers and original Drop/Pin boundary.

The explicit source Provided-contract profile uses tag 11, row version 1. Retain
one advisory-column owner, the complete Provided family and numbering, actual
namespace bindings, and positive policy that supported headers have no written
`default`, `final` or `track_caller` contract. Missing old columns stay unknown;
all older first encodings remain unchanged. Resolve a fresh named executable
through the same Catalog/reservation and selected service, then demand its real
body Environment, original proof and normal fresh instance obligations. Ordinary
impl overrides of ordinary provided methods remain legal; modifier absence is
not specialization finality. The [encoded execution checkpoint](docs/encoded-provided-methods-checkpoint.md)
records actual carry/borrow native execution and the still-unsupported richer
modifier, const/default-body and regional contracts.

The explicit compiler-operation profile uses envelope13/version11, preserving
source10, Provided11 and native-link wrapper12. Only this new profile prefixes
its optional const-body trailer with presence. Authenticate actual source helper,
signature, opaque owner/ordinal/full captures, Future/Output/member schemas and
registration before classifying a body as CompilerOperation. Original and
operation coverage is exclusive; a single paid shared sparse BodyRef index joins
real ordinary rows while legacy dense profiles retain no index. A symbolic
operation owns no placeholder Original, locals, CFG or executable permission.
Reserve one actual imported opaque identity per defining declaration/ordinal and
authenticate current store/catalog/schema/complete bounds before both cold and
cached result reads. Share the existing declaration-input and substitution
kernels; imported proof sites must never masquerade as AST coordinates. The
first profile has validated two-capture/zero-own-projection-argument bounds;
wider argument families require indexed dependency enumeration before admission.
Actual constructor/poll/drop/await/cancellation and full source/body/policy
transport remain required. The [checkpoint](docs/parallel/deferred-operations-v11.md)
and its evidence distinguish fresh type proof from executable future authority.

Actual VaArgSafe and VaList source declarations complete through the ordinary
trait-header and nominal producers, without fabricated unsafe/Copy/arity or
layout rules. Real implementation domains, trait selection, fields, borrows,
destruction and forced-indirect ABI policy remain authoritative. Declaration
completion does not permit variadic initialization or reading arbitrary storage;
those consumers require actual source/target/lifetime and operation proof.
Keep original Rust-positive c_variadic requests and actual full standard source
closure as required work. See the
[source checkpoint](docs/parallel/source-va-declarations.md).

The actual ellipsis parameter stays in the original syntax parameter list while
public callable types contain only fixed ABI inputs. One real internal region
and owned VaList local join source/header/parameter/type/binding identity to its
unique entry StorageLive. A borrowed body-owned Start plan authenticates that
publication before initialization and typed va_start emission. Semantics expose
typed source data without importing body CFGs. Authenticate actual marker and
PhantomData templates separately from region-only physical field views, using
the completed real nominal fields, variance and existing layout cache. Direct
va_arg requires its actual declaration, complete application, independently
opened reference binders and normal reached-call/report/effect proof; va_copy
and va_end execute their actual source fallback bodies. The direct-operation
classifier must not intercept unrelated declaration families. Const Drop
declarations follow ordinary actual const-trait/impl conformance, without
granting CTFE eligibility. Cursor CTFE, nominal const cleanup, broader ABI and
authenticated body/local transport remain explicit requirements in the
[runtime checkpoint](docs/parallel/source-va-runtime.md).

Explicit integer-repr fieldless enums reuse the existing scalar-object ABI
adapters. A single paid layout query authenticates the actual nominal enum,
representation, source/physical variants, discriminants, zero fields, tag offset,
size and alignment. Its primitive result is storage only: preserve source enum
identity, validity, Copy/Drop and call permissions. Never reuse a zero-excluded
validity Carrier for this fact. Reuse existing per-type Context representation,
C Plans and composite-node caches so repeated values/signatures do not repeat
the variant walk. Construction still writes the real selected variant tag.
Payload/default-repr/custom-alignment/wider transparent profiles remain named
requirements. See the [enum checkpoint](docs/parallel/source-c-enum.md).

Private callable preparation consumes the actual reserved declared/callable
signature pair and complete outer-to-inner formal scope, including unused
parameters and inherited Self. Share the ordinary callable registration kernel;
retain the real named owner and anonymous Function types. Build the scope index
once. A warm hit still authenticates the catalog, source arena, declaration,
body, signature and exact scope record before reusing readiness, without
traversing its ancestry again. Prepay actual template capacity growth, temporary
owner validation and selected regional occurrences; unrelated declarations and
the entire type store are not per-call allocation bounds. Failed attempts retain
their charged work and peak, and cannot publish partial readiness. This private
structural preparation grants no defining Environment, original body proof,
call-instance proof or wider metadata-profile permission.

Private selected recipes use one fixed shared memo with a sole publishing
worklist; readers authenticate its actual root, Store arena and allocation.
Reuse the ordinary signature checker and preserve explicit missing/pending/ready
states. Certify real named type formals and inherited impl Self during the same
collector walk, without guessing prefixes or cloning a second type table. The
[mapping checkpoint](docs/private-signature-mapping-checkpoint.md) records cold
storage, constant warm work and remaining owner/body proof obligations.

Raw original Contract parameters retain the source iterator's inner-to-outer
scope order. Callable applications retain their separate outer-to-inner formal
order. Validate each using the same actual scope/slot identities and complete
inherited counts; never reorder the original body or guess a Self position.
Imported body preparation maps each raw formal through existing prepared
declaring/parent scope cells, verifies the exact leaf and port ParameterIds,
and preserves missing/duplicate/reordered/foreign failures. Reuse the dense
scope index and charged constant-time lookups instead of rebuilding flattened
formal tables or walking ancestry per demand.

Private numbering records bind actual catalog/body/raw-row/callable/arena
identity and reuse the source allocator's checked unit-local issuance order.
Prepay selected logs, inverse scratch and alias ledgers; preserve failed work,
capacity costs and absent versus present-empty ledgers. Recorded readiness
does not bind a defining regional Environment or change reservation availability.
Separate body/availability-owned regional ports and their actual proof remain
required before original or substituted-instance execution.

A nonzero signature may use the same defining-environment kernel only after
the actual Declaration origin, empty regional inputs and supported frames are
authenticated and both preexisting builtin and region payloads are absent.
Absent and present-empty payloads are distinct. Prepay real alias vectors, map
growth/coexistence and Arc storage before the shared finish kernel; reconcile
fresh owned payload without charging shared declaration tables again. Bind the
completed body memo to both its actual immutable recording and Environment
allocations, retaining allocation charges while weak keys survive. Full ledger
scans stay cold; warm checks authenticate the complete owner/signature/scope and
both identities in constant work. This grants no body Variable or declared
lifetime availability, original proof, instance proof or public execution.

Private defining-environment preparation reuses the canonical predicate
translator and Environment producer through sealed ordinary and reservation
ports. Authenticate actual Input sites and complete inherited/unused formals.
Distinguish declaration trait premises from regional body assumptions. An
unchanged zero-domain Declaration body may share its actual declaration Arc;
an actual empty Contract body requires a separate neutral body Environment and
positive function/scope/body/binder identity, never that declaration Arc.
Conditional default and nonzero origins require their own complete consumers.
Never insert neutral declaration maps merely to make a private query succeed:
that changes existing source origins and metadata bytes. Prepay selected work,
real lazy collection growth and collision comparisons, including minimum vector
capacities; warm canonical hits must not allocate or scan unrelated originals.
See the [private environment checkpoint](docs/private-defining-environments-checkpoint.md)
and its [empty Contract successor](docs/private-empty-contracts-checkpoint.md).

Source-module namespace preparation borrows genuine ordinary trait headers
without inventing language roles. Retain canonical declaration/module targets
and actual aliases, move the existing columns and body catalog once, and use
fallible charged sorting rather than unfolding paths. Reserve/install/freeze
the actual Catalog transactionally; failed attempts publish no completed owner.
Foreign module bookkeeping cannot prove local ancestry or implementation
permission. Private namespace readiness grants no public profile or restricted
trait transport. A later versioned contract must carry complete implementation
domains and advisory facts, validate legitimate provider-owned implementations,
and reject unauthorized fresh foreign implementations separately from the
deny-explicit-impl policy. Keep each metadata consumer's admission explicit.
See the [private namespace checkpoint](docs/pa29-module-namespace-inspection.md).

Role validators consume borrowed declaration headers rather than reconstructing
source syntax. The source adapter validates the actual declaration, separate
Self owner/slot, flags, parent and completed member/input inventories before
borrowing them. Keep source vectors directly owned and preserve required versus
provided method bodies. Header validation grants neither imported role authority
nor executable proof. The [neutral header view](docs/pa29-neutral-role-headers.md)
prepares shared validation; authenticated source-client publication and its wire
profile remain a separate reviewed transaction.

Direct generic helper calls retain one indexed dependency closure, distinct
from the acyclic lexical owner forest. Callable recursion must not become a
false owner-cycle error. Derive each complete target/argument signature once;
reuse its structure and goal templates, never its caller proof. Each actual call
reconstructs formal well-formedness, declared predicates and input/result
coercions under its original Environment. Preserve unused formal obligations.
Canonical dependency sorting observes key sizes once and charges comparisons
without rescanning bodies. The [direct-call metadata contract](docs/pa29-direct-call-transport.md)
defines row version 1 and preserves exact version-zero bytes. Source-language
roles, unsafe/indirect calls and wider types retain their explicit boundaries.

Reserve imported graph storage from actual declaration-owner summaries during
the existing topological walk. Count nodes/edges once, deduplicate reached owners
in reusable scratch, and retain shared dependency edges instead of flattened
per-root closures. Separate conservative preflight reservations from observed
retained payload and reject partial publication on either work or storage limit.
An artifact's total size repeated for every declaration is a quadratic bound,
even when each actual lookup is indexed.

Every target Rust dependency must pass through RustGM. Explicitly declared host
tools may bootstrap build-time execution, but rustc-built target libraries or
frontend fallbacks cannot satisfy upstream acceptance. Keep build/configuration
orchestration outside the frontend. A whole selected crate build and independent
native consumers are required; source reducers guide work but do not replace it.

The fixed bootstrap-capable x86-64 target includes bare target_has_atomic,
separately from its width values and the gated target_has_atomic_load_store
predicate. Target truth grants no source feature or macro permission. Preserve
override rejection and keep host bootstrap environment out of the compiler
library. The [atomic predicate checkpoint](docs/bare-atomic-cfg-checkpoint.md)
compares pinned stable/bootstrap behavior and records the explicit configuration
implementation hash change in its derived original-core manifest. Hardware,
CLI target inputs, source bytes and finite limits are separate invariants; do
not conceal a corrected target fact behind an ad-hoc --cfg override.

The four target_has_reliable_f16/f16_math/f128/f128_math queries share the
actual cfg_target_has_reliable_f16_f128 gate and genuine definition-site macro
allowances. Reuse demanded-use provenance and the paid macro-frame memo; caller
captures gain no permission. The query feature enables neither float language
feature nor arithmetic. Target truth stays owned by the selected backend: all
four values remain false for the fixed CPPGM profile and cannot be forged through
custom cfg. Preserve the packed feature word and existing ordinary records.
The [float-reliability checkpoint](docs/pa41-float-reliability-checkpoint.md)
qualifies both source editions, actual source-body execution, bounded repeated
queries and byte-identical predecessor/candidate configuration constants. A
configuration-source edit still requires a new explicitly requalified manifest;
retain old source/configuration bindings and the source macro metadata fence.

Actual source UnsafeCell and UnsafePinned publish their paid interior-storage identity in the
same one-time transaction as its real field shape. Reuse the existing owned
storage query, writable-global, promotion and niche consumers; do not infer
Copy, Sync, Freeze or destruction policy from the role. Ordinary shared and
mutable reference loans retain their proofs. Source-mode empty/single-field
Rust enums publish physical nullable eligibility from their actual field groups,
independently of Option identity, before stable layout queries. Two single-field
variants may also qualify when one field is zero-sized and no more aligned than
the other's existing carrier. Select the orientation per instantiation, trying
the declared order before its reverse; retain the actual variant identities.
An ineligible two-field instance keeps its existing tag layout. Re-derive the
same rule from validated imported field groups, without transporting policy bytes.
See the [zero-sized companion checkpoint](docs/pa31-zero-sized-niche-companions.md).
Source policies also admit bool's unused byte value2. Keep that permission
independent of the shared physical carrier cache and the historical course policy.
Every constant, validity, runtime, cleanup and initializer consumer uses the actual
carrier value; the enum travels as a byte, while its bool payload still requires0
or1. A nonzero companion is never zero-filled data. See the
[bool niche checkpoint](docs/pa31-bool-niches.md). Reusing additional unused values
from an already niche-encoded payload remains part of general scalar niches.
Reuse the
canonical carrier cache, block carriers through interior storage and preserve
explicit representations and the separately selected historical course profile.
General scalar/multifield niches and authenticated interior policy transport
remain required. The [source packet](pa31/tests/spec/unsafe-cell-source/README.md)
retains the original native layout failure and independent opponents.
UnsafePinned uses this same policy before layout, including a direct field with
no inner UnsafeCell. The role does not refute Freeze, Unpin or UnsafeUnpin:
actual source implementations supply those trait proofs. Shared-to-mutable
borrows and field destruction remain ordinary. Current lowering emits no
Rust noalias/retag promises; adding those requires authenticated UnsafePinned
coverage and UnsafeUnpin proof. See the
[independent source packet](pa31/tests/spec/unsafe-pinned-source/README.md).

Source auto-trait selection preserves explicit positive and negative identities.
A refutation is not positive evidence, and an applicable explicit implementation
family replaces its structural default even when its conditions are unpaid.
Validate that family's headers before publishing a completed absence. Retain
nominal well-formedness prerequisites before using physical shape for source
Freeze, sizing or ordinary destruction. Share immediate formation dependencies
in a query-local DAG keyed by the resolved type, surrounding callable depth and
immutable caller scope; flattening each nested prefix repeats quadratic work.
Formation-only nodes supply no trait implementation or dispatch permission.
Never cache an absence that discards an unpaid prerequisite. Preserve active
cycle checks, binder-local deferral and complete child obligations. See the
[source Freeze checkpoint](docs/pa31-source-freeze-proposal.md) for the measured
depth, width, unrelated-node and cold/warm controls.

A method receiver step asks only whether its candidate applies. A refuted trait
requirement makes the step inapplicable, and lookup continues to the next
adjustment. This includes a conditional negative whose own lifetime equations
are unpaid, such as core's `impl !Clone for &mut T` at the `&&mut T` step of
`t.clone()`: no positive candidate completed beside it. The step publishes no
negative fact or cached absence. Ordinary obligations, path conditions and
coherence keep the existing refutation guards. See the
[negative receiver checkpoint](docs/pa31-negative-receivers.md).

Auto traits receive no implicit Self `MetaSized` supertrait; written bounds and
ordinary trait, parameter and alias defaults remain authoritative. Source
UnsafeUnpin uses its genuine unsafe auto identity and existing formation,
negative-impl, caller and coinductive proof machinery. Append source identities
without changing old role ordinals, bootstrap marker inventories or wire
admission. This proof grants no optional noalias optimization. See the
[UnsafeUnpin checkpoint](docs/pa31-unsafe-unpin-checkpoint.md).

Actual source Copy and Clone have ordinary compiler-defined marker rules for
unit, function items and function pointers. Unit has no child obligations;
nonempty tuples need their own complete child proofs and receive no evidence
from the zero-child rule. Match the completed registered trait and
empty arguments; keep explicit, pending and partial candidates authoritative.
Reuse the same caller-local formation DAG for nominal types in the callable
signature or item arguments, preserving lifetime and declaration obligations.
Signature parameters and results need not implement Copy. Marker evidence grants
neither an executable Clone body nor const-call permission, and it cannot enter
the sizing-only reuse cache. Ownership consumes the resulting exact body-local
Copy decision through its existing checked interface, without a physical fallback.

A source Copy impl's target follows rustc 1.98's `type_allowed_to_implement_copy`.
Scalars other than `str`, `!`, raw pointers, shared references and arrays are
targets without fields; an array's element is not checked. An ADT target proves
its fields. A unique reference, `str`, a slice and every other kind are no
target. rustc also rejects a written Copy impl for a tuple or a fn pointer
(E0206), which have builtin Copy; this compiler still accepts one whose elements
are Copy, a recorded gap. The [Copy target controls](pa31/tests/spec/copy-impl-targets/README.md)
record independent outcomes.

Union field eligibility consumes the actual source Copy trait application when
source language items are selected, preserving generic premises under that same
identity. Keep the canonical builtin bound for the bootstrap profile. The
physical-field Copy implementation proof and Clone supertrait remain independent
requirements. A source union Copy derive emits only its real empty marker impl;
whole-union copying never selects or reads an active payload field. See the
[union checkpoint](docs/pa31-source-union-copy-checkpoint.md) for source/native
and opposing field, privacy, safety and identity controls.

Source union Clone emits the genuine TrivialClone marker and Clone impls, with
Copy plus the relevant trait before the written bounds on each type formal.
Retain lifetime/const/default/where syntax through the existing typed header
walker. Its ordinary AssertParamIsCopy<Self> type check and `*self` body require
real Copy evidence; helper spelling grants none. Generate no field reads, retain
ordinary struct/enum rules and keep the output independent of union field count.

Derive field macros retain their actual Type grammar and selected captured
forest through typed copying and grafting. Prepay real capture/vector/tree
growth; preserve delimiters, contexts, origins and opaque parsed handles. The
registry and ordinary feature checks remain authoritative. Never flatten those
handles, copy an unrelated source arena or reparse rendered field spelling.
See the [typed macro checkpoint](docs/pa31-derive-type-macros-checkpoint.md).

Derived field constants copy selected typed children and rebase selected inline
literal IDs into the destination pool, with prepaid growth. Calls retain the
actual callee and ordered argument list through the existing checked walker;
Path/Applied/Qualified nodes preserve real generic types and spans. This syntax
transfer grants no callable or const-execution permission. Deferred token
literals keep real tokens and new pending-materialization IDs. Include pending
owner/literal slices in O(1) payload accounting; do not scan or copy an unrelated
source pool. Byte-backed values require their actual owner, never a shallow
stored handle. Preserve spans, parsed captures and downstream semantic/feature
checks; copying syntax grants no callable or evaluation permission. See the
[constant-subtree checkpoint](docs/pa8-derived-constants-checkpoint.md).

Scalar pattern types retain the captured pattern AST and evaluate endpoints
through ordinary typed constant continuations. Reify a captured plain endpoint
binding as an expression only in the fresh builtin tree, preserving its symbol,
hygiene and span. Never reparse its spelling. Intern paid, cold canonical
intervals by actual base and target-width endpoint bits; structural comparison
and normalization transport contents rather than store-local range IDs. Signed
intervals may wrap in raw bit order. Validate source ordering separately from
base validity, and check actual initialization, provenance and char validity
before the interval constraint. Preserve the base representation when lowering;
an integer refinement is not a nullable pointer. Genuine NonNull retains its
independent ABI rule. Open/exclusive/alternative forms require their actual
source helpers and complete const/API/desugaring authority. Generic endpoints,
integer enum niches and refinement metadata stay explicit requirements. See the
[closed-range checkpoint](docs/pa31-closed-scalar-patterns-checkpoint.md).

A non-null pattern type's base is a raw pointer or a type parameter, and
substitution must instantiate a parameter base with a raw pointer. Every other
base or instance is rejected by name. rustc 1.98 also accepts integer and
reference bases, but they have no defined non-null representation here.
DispatchFromDyn relates two pattern types only under the same constraint; a
changed pattern field proves its relation through such an impl, as core's
`pat.rs` does.

RangeMin, RangeMax and RangeSub identify genuine associated declarations of one
const-trait owner; the trait's spelling grants no authority. Prepare their actual
declaration types and method signature before completing the shared contract,
without demanding an initializer or body. Unused partial families remain pending.
Associated constants reuse their actual retained owner/image witnesses; they do
not enter the function/trait attribute API or acquire invented KnownAbsent facts.
The paid cold contract authenticates its original Store and four source images;
warm queries allocate nothing and inspect a fixed set of actual identities.
Keep comparably sized cold provider/import arrays inline rather than adding heap
allocation and a new budget owner just to satisfy an enum size threshold. Any
narrow Clippy expectation requires the measured layout and a specific reason.
The identity query supplies no selected implementation, API, const, body or
desugaring permission. Old wire profiles remain fenced until genuine transport.

Anonymous constants and named declaration-time initializers retain their actual
caller identity and Const effect. Demand caller-location data only from solved tracked calls and
validate genuine Location/Phantom fields before publication. Resume selected
implementation conformance with a retained touched-obligation cursor and actual
per-owner method/constant kernels; no all-implementation scan or descriptor
proof substitutes for conformance. Source binary default readiness yields the
exact parsed binding through existing default work, preserving strict operator
validation. During dispatch, validate sealed CallerSite identities before and
after transporting callable types through the same proved Bound as operands;
source owner, value and location remain fixed. Empty-site bodies remain sparse.
See the [endpoint checkpoint](docs/pa31-source-endpoint-execution-checkpoint.md).

A demanded associated constant needs its actual initializer body checked as well
as its declared type. Resume the existing initializer-body request for the
selected provided definition; declaration readiness alone cannot satisfy an
execution request. Preserve its real substitution, defining context and API,
const, unsafe and storage proofs.

A declaration-time constant whose reached Copy decision selects a source
implementation, field template or contract still queued in the declaration
worklist waits for that one fact. It then repeats the same caller-scoped
decision. Only that decision's own pending failure becomes the wait. Other
failures keep their cause, and consumers that cannot wait report the pending
failure unchanged. The wait is an ordinary header dependency edge, so existing
cycle detection covers it. A published fact that does not resume its constant
is an error. Pending selection is never answered from declaration-level
availability. Early scalar validation admits an impl whose subject is one of its
own type parameters or a nominal that applies exactly its own type parameters
in order. For a nominal subject it queues the field template and demands only
the subject's reached lifetime contracts, which complete inference republishes.

Constant evaluation executes a validated source `assume` itself and never its
fallback body: a true condition returns unit and a false one is a
constant-evaluation error, as rustc's E0080 is. A declaration-time body that
relates instances of generic lifetime nominals before the crate-wide variance
and outlives inference demands only those nominals' reached variances and
outlives contracts. A nominal whose field template is still queued becomes one
ordinary field wait; a second wait on the same nominal is an error. Complete
inference later republishes the same declared results. Only a source library
demands these contracts early; an ordinary crate keeps its existing order, and
its declaration-time relations of such instances remain a recorded gap.

Exact canonical Self receiver proof can publish after the real function's
signature, captured lifetimes and environment complete. Receivers with different
nominal arguments still require the full directed proof and completed nominal
facts. Retain a cold candidate iterator across declaration waits; do not rebuild
and scan each growing ready prefix. This continuation must preserve ordinary
body/work record widths and the original receiver inference graph.

Before an early body borrow proof consumes a nominal, prepare its genuine
variance and outlives dependencies. Reuse the existing field-template solvers,
including original fields, explicit declaration bounds and cyclic propagation.
Solve only new dependencies and use completed rows as immutable inputs; repeated
wrappers around a shared dependency must not rescan that dependency. Source
producer payments cover attempted scratch work/growth and persistent publication,
including failed paths; warm queries retain no storage. A missing row supplies no
invariant assumption or empty lifetime contract. Final whole-declaration checks
remain required independently of these early producers.

Ordinary `rustc_nounwind` functions consume the checked current target and their
actual calling convention. C and unadjusted are non-unwinding; Rust and rust-call require the
fixed target's abort policy. Read that immutable fact from the same configuration
owner as cfg, never from a user-defined option or environment. Match supported
ABIs exhaustively so a future unwind ABI needs an explicit boundary rule.
Preserve the source function/word/internal-feature checks and execute the real
ordinary body. The marker grants no intrinsic identity, const eligibility or
unsafe permission, and does not change an ordinary function-pointer type. No
extra body/node field or per-call scan is needed in the current abort profile.
PA39 still requires actual unwind boundaries, cleanup and transport. The
[ordinary nounwind checkpoint](docs/pa31-ordinary-nounwind-checkpoint.md) retains
the original unlikely source and the separate real ColdPath dependency.

Unadjusted ABI identity is distinct from Rust, C and rust-call in declaration
facts, type interning, generic substitution and reified pointers. Its actual
abi_unadjusted request or introduced macro authority is required. Reuse completed
literal payloads for opaque ABI grammar, retaining a transparent constructor's
source context and preserving payload provenance. A forwarded whole item grants
no new allowance. Share one permission session over the existing header columns;
root permission skips the scan. Ordinary node and calling-convention rows stay
fixed. The [unadjusted ABI checkpoint](docs/pa28-unadjusted-abi-checkpoint.md)
qualifies scalar source execution and records independent raw-versus-C aggregate
signatures. Reject unsupported aggregate/SIMD lowering before accidental C or
hidden-result substitution, and reject unencoded convention metadata. Those
positive aggregate/intrinsic/transport requirements remain open; scalar backend
compatibility does not establish them.

Source unchecked_sub/sub_with_overflow use their actual generic Copy declaration
and checked caller/instance, with integer kernels selected only after concrete
substitution. Keep ordinary lookalikes ordinary. Runtime unchecked subtraction
does not compute an unused overflow flag; CTFE still diagnoses unchecked overflow.
The overflowing tuple uses the shared checked-arithmetic kernel and exact tuple
ABI. Convert a LowIR machine predicate to the Rust bool carrier before storing it;
the enclosing field layout does not change a Value's actual width. ColdPath
executes its genuine source fallback under the authenticated retained marker and
normal original/body/const permissions. See the
[subtraction checkpoint](docs/pa28-source-subtraction-checkpoint.md), including
the real native width failure that structural checks alone did not catch.

The actual local `rustc_do_not_const_check` marker controls its authenticated
defining body independently of reached-call eligibility. Retain completed
provider/header/function/source identity and the ordinary runtime body checks;
the marker supplies no const execution permission. A reached marked ordinary
call still requires executable const Code. Wider declaration profiles retain
their own source and body contracts.

Source `ptr_guaranteed_cmp` authenticates its actual safe const Rust declaration,
one Sized type argument, exact pointer/result signature and reached proved call.
Reuse WordAddress, Origin and actual layouts to distinguish proved equal,
proved different and unknown values without reading pointed storage. Preserve
allocation/static identity, offset, alignment residues and expired provenance.
Runtime calls execute the real ordinary fallback body. Primitive same-type raw
pointer equality consumes its normal source trait obligations before existing
pointer lowering; explicit method calls execute their selected source bodies.
Ordinary pointer comparison retains its const restrictions. The
[pointer checkpoint](docs/parallel/source-guaranteed-compare.md) records actual
fallback mutation, explicit-method and native provenance controls.

Cold advisories on no-body functions retain their actual source owner in the
existing header ledger. Bind required methods through their real trait member
scope and foreign declarations through their resolver-owned active foreign
block. Required methods retain unused-attribute policy; foreign declarations
retain ordinary ABI, safety, const and definition rules. Capture ordinary local
rows only when needed; SourceCore reuses its retained rows. Activate the existing
prepaid definition-binding walk with transient pass state, using matched header
lookups rather than a second identity map or repeated whole-inventory scans.
The [prototype checkpoint](docs/parallel/cold-prototype-headers.md) records
source replay, actual owner, bounded-noise and real foreign-call controls.

Source Debug derives use the pinned small-field helpers through five fields,
then real field-name and typed `&[&dyn Debug]` arrays with the larger helpers.
Preserve source labels, field order, enum binding provenance and the extra
reference for a possibly-unsized final field. Share the emitter across shapes
and charge linear output work; helper recipes grant no library-body execution.
See the [array checkpoint](docs/pa31-source-debug-arrays-checkpoint.md).

Derive emission pays for actual source, origin, token and field capacity growth,
including copying on reallocation. Reused capacity is not a new allocation;
characters inside one identifier are not independent token slots. Charge source
copying, lexical scanning and each token initialization separately. Prepay parser
scratch and syntax from actual emitted token/splice counts, preserving typed
subtree and final owned-output charges. Keep attempted work/storage, retained
payload and sticky failure accounting distinct. Never increase a limit to hide
repeated capacity payment or speculative token estimates. The
[image-storage checkpoint](docs/pa31-derive-image-storage-checkpoint.md) records
the original-core diagnosis and fixed-setup-separated scaling controls.

The macro-contract and library-inventory ledger is bounded by its admitted
source, for runaway protection rather than as an efficiency target. Its allowance
starts at a fixed setup floor of one million work units and 64 MiB each of
attempted storage and retained payload. The first parse of each authenticated
source identity (the crate root, file modules and `include!` sources) then grants
2 work units, 64 attempted bytes and 8 retained bytes per original byte, before
any charge reads that file. Expansion output, copied module or include instances,
aliases and repeated requests grant nothing, so superlinear expansion still
exhausts it. Explicit `--source-contract-*` values are optional upper caps: each
effective limit is the smaller of its cap and the granted allowance. Zero is a
real cap; every accepted value is finite. Snapshot the caps once when either
source-core or local macro contracts are created. A later grant never refunds
attempted charges or clears the first sticky exhaustion cause, and configuration
changes cannot reset an existing state. Limits grant no language, API, unsafe,
const or metadata permission. Record any changed resource profile separately
from target configuration and lint caps; the original-core observer requires
matching manifest and producer caps, passes them as compiler arguments and
records null for an absent cap. Never suppress real operations. The
[budget checkpoint](docs/parallel/macro-budget.md) records the sizing attribution.

The source lint-policy ledger (owners, witnesses, frames, calls, reads and their
storage) is bounded the same way. It starts at a fixed setup floor of ten million
steps and 64 MiB of attempted storage, the former whole-crate limits. Each
admitted source identity then grants 8 steps and 64 attempted bytes per original
byte. A producer created after admissions is granted their total before its first
charge, and an existing producer is granted each later module or `include!`
admission before expansion visits it. Expansion output and copied instances grant
nothing. An exhausted producer is never granted again, so its first failure stays
sticky. The [census batch 3 record](docs/parallel/census-batch3.md) records the
payer attribution and the linear and superlinear controls.

Efficiency comes from removing repeated work and from the complexity and scaling
rules under "Ownership, allocation and complexity", not from a fixed whole-crate
number. When a phase's required work and retention are linear in its input, bound
its ledger by a documented constant factor of that phase's authenticated input
graph, such as its definitions, owners, effects, AST nodes, edges and retained
attribute tokens, as declaration checking does.
Never reuse a constant from another domain, such as a metadata decoding bound.
Compute the bound before the first charge that reads its input; a ledger whose
input is admitted incrementally grants at each admission. Keep failure sticky. A
proportional bound never justifies repeated or superlinear work: never raise its
factor to absorb either, and qualify it with paired linear-scaling inputs and an
adversarial superlinear control.

Source enum Default requires one configured unit variant whose actual helper
is published by the selected builtin derive. Bind that choice to its owner,
effect, capture and completed derive receipt; replay also rejects a missing
choice. Keep selection and paid scratch local to the item, without a second
catalog census or owner map. Preserve written generic/const/where requirements
and omit invented payload Default bounds. Emit the real `Self::Variant`
constructor. Ordinary concrete enum Self paths reuse existing associated-type,
variant, hygiene and privacy resolution; abstract Self grants no constructor.
The [enum contracts](pa31/tests/spec/enum-default/README.md) retain opposing
helper, header, replay, construction and work controls.

Share completed ordinary source MetaSized/PointeeSized answers across nominal
roots in one immutable caller. Eligibility comes from the checked actual sizing
producer; arbitrary builtin evidence cannot opt in. Preserve the full goal,
root scope, empty lifetime/formation debt and noncyclic/nonprovisional status.
Revalidate the actual declaration role on publication and both direct and
iterative reuse. A wrapper may inherit eligibility only from an identical-goal
eligible child, with every sibling and normalization prefix paid. Keep the
reuse owner absent on ordinary scalar paths and keep hot query records unchanged.
The [whole-caller sizing investigation](docs/pa31-source-sizing-reuse.md) shows
why isolated query DAG counters must be complemented by full typed source calls:
separate roots can repeat a complete nested suffix despite each query being linear.

Nested sizing proofs charge each completed child's own and nested work exactly
once to their owning budget; repeated builtin calls charge only their new delta.
Keep reported own counters distinct, saturating budget failures sticky and the
actual nested-owner guard named. Reuse completed physical properties and sizes
only under the exact immutable caller and implementation view. Structural
answers never replace nominal formation or selected trait/lifetime evidence;
exclude provisional and private-region results. The [nested proof checkpoint](docs/parallel/nested-proof-reuse.md)
records linear normalization counts, the remaining affine-work target and
PA31's independently checked source reductions.

A flat ordinary Deref query may retain already-selected implementation sources
reachable from its own parent proof and follow same-member result suffixes
before the original lifetime restoration. Retain the whole parent proof and
charge graph edges and definition steps. Exclude scoped, universal, placeholder,
retained-Sized, provisional and cyclic selections, const-qualified roots, own
member conditions, GAT inputs and mapped or equality-dependent environments.
Unsupported cases retain the ordinary normalizer. Store completion detail cold
without growing Evidence/Selection. The
[selected-chain repair](docs/parallel/nested-proof-ci-repair.md) includes an
independent unpaid-lifetime opponent and the original128-layer receiver.

Source sizing follows rustc 1.98's hierarchy. `PointeeSized` holds for every
type. A rigid projection is `MetaSized` or `Sized` only from an environment
assumption that names it or from its associated type's own item bounds: its
written bounds, the implicit `MetaSized` of a `?Sized` result or a declared
`Sized`. After input normalization, an
environment sizing assumption over another projection that cannot yet be
compared does not defer such a goal when those item bounds, or for
`PointeeSized` every type, prove it; any other answer stays deferred. A
normalized rigid projection whose item bounds do not prove `MetaSized` or
`Sized` is absent, never a selection failure. The [projection
sizedness](pa31/tests/spec/projection-sizedness/README.md) and [normalized
projection sizing](pa31/tests/spec/normalized-projection-sizing/README.md)
controls retain the accepted shapes and their opponents.

A `const` or `[const]` trait bound implies its ordinary trait bound, as rustc
1.98 elaborates it. Any query for the ordinary trait over an environment
premise, including the shape walk that keeps a parameter's projection rigid,
accepts the effect-qualified premise, so the projection it names is well formed
inside a nominal argument and keeps the caller's lifetimes without
normalization. An effect goal still requires its qualified premise. The [const
bound projections](pa31/tests/spec/const-bound-projections/README.md) controls
retain core's `as_deref`, `as_deref_mut`, `Pin::as_ref` and `<[T]>::get`
shapes, their guards and their opponents.

A const impl proves its trait's `[const]` supertraits and where clauses as
`[const]` goals, as rustc's wfcheck proves the trait's const conditions, under
the impl's own `[const]` premises and their `[const]` supertraits. Its plain
supertraits stay ordinary obligations. A `[const]` goal still needs a const
impl or a `[const]` premise, and a `[const]` premise does not prove a `const`
one. The [const supertrait controls](pa31/tests/spec/const-supertrait-requirements/README.md)
record independent outcomes; they include core's const callable reference
impls through the callable sugar controls.

Method lookup ignores constness, as rustc 1.98's probe does. A method of a
const trait resolves through an ordinary impl or an ordinary premise: the
trait's `[const]` supertraits are lookup conditions only as their ordinary
bounds, and a `[const]` supertrait and its elaborated ordinary bound are one
condition. The chosen callable still proves its complete clause under the
use's effect, so the same call in a const context through an ordinary impl
or premise is rejected, and two applicable trait methods are ambiguous. The
[const supertrait lookup controls](pa31/tests/spec/const-supertrait-lookup/README.md)
record independent outcomes for core's `<*const VTable>::cmp` shape.

Callable sugar such as `F: FnMut(T) -> U` publishes its bound and output
equality inside a binder even when it names no lifetime. Method lookup reads
a binder that quantifies no lifetime and captures nothing outside it as its
body, an ordinary condition. A condition read this way that equals a flat
clause of the same scope, such as the callable's implied `MetaSized`, is the
same condition and is checked once. A binder that quantifies a lifetime
still requires a scoped consumer. The [quantified lookup
controls](pa31/tests/spec/quantified-lookup-conditions/README.md) record
independent outcomes for core's `Drain::new` shape and the bound-region
shapes that remain rejected.

A trait object, like a slice or str, has no fixed layout. The crate-wide layout
pass demands no static layout for a declaration whose last field is, or ends
in, a trait object, as rustc 1.98 accepts such declarations; any other unsized
field is still rejected. The [dynamic tail
layouts](pa31/tests/spec/dynamic-tail-layouts/README.md) controls retain core's
`Request`, tuple, record and transparent tails and their opponents.

An analysis-only declaration census that fails a crate-wide declaration pass
for any reason other than sticky budget exhaustion still runs every later pass
exactly as ordinary checking would, with census isolation suspended. Each later
pass's first error becomes a `lookahead` record that names its pass and the
terminal record it follows; it may derive from that failure, changes no count
and never lets bodies run or the census qualify. The terminal record names the
failed pass. Ordinary checking still stops at the first error. The [core
census mode](docs/parallel/core-census-mode.md) record describes the census.

Actual source Unsize selection checks both endpoint formations even after a
conversion mismatch or an unresolved peer. Pointee shape alone does not pay
nominal lifetime requirements: retain their existing symbolic region/type-outlives
debts as input proof nodes. Deferred selection stays deferred; completed absence
requires every inherited normalization and formation debt to be paid. Preserve
those conditions across cold and warm lookup. The
[source unsizing checkpoint](docs/pa31-source-unsizing.md) defines the actual
Unsize/CoerceUnsized roles and the retained runtime/CTFE boundaries.

Actual source nominal Unsize consumes completed same-owner storage-tail rules,
equates every unchanged type/const/lifetime input and proves the actual final
field Unsize as an ordinary child. Keep both endpoint formations and lifetime
debts even on a mismatch or deferred child. Reuse the existing typed field
argument environment so source/target element inference survives; do not require
a resolved whole nominal application before selecting its field. The
[nominal source packet](docs/pa31-nominal-unsize.md) retains ordinary inference,
negative, native and bounded-work evidence. Layout grants no conversion proof.

A source projection awaiting normalization may generate erasure requirements for
a known object target without first exposing its physical head. Every Sized,
principal, associated equality, auto-marker, lifetime and endpoint requirement
must prove before completion; failure cannot refute an unknown-head conversion.
Known object upcasts retain their intrinsic-only principal restriction. Abstract
internal auto-marker requests must select the actual source trait application,
including declared-result and parent premises, before structural fallback. The
[projected erasure packet](docs/pa31-projected-object-erasure.md) retains source,
negative, native and bounded-work controls and separate tuple-factory requirements.

Unsizing discovery compares the complete canonical input shapes of independently
normalized associated results before treating their argument-layout identities as
a forced mismatch. Region-only differences supply neither equality nor conversion
permission. Each complete opaque input shape is compared once; other queued
siblings remain checked. Ordinary comparison, raw equality, parent and lifetime
proofs retain their existing boundaries. The
[associated-result input packet](docs/pa31-projection-argument-discovery.md)
includes native payload/borrow witnesses, complete input distinctions and nested
8/32/128 work bounds.

Conditional source const calls retain declaration requirements separately from
caller permission. Prepare immutable requirements once for each complete Code
instance, then prove them under the actual caller environment at every reached
statement, including warm Code reuse. Consume a private continuation bound to
the exact frame, code, statement, target and checked contract before invocation;
resuming or replaying another use cannot inherit permission. Keep the optional
input table cold and move the already proved Contract into retained rows rather
than cloning it again or widening every Body/Value. Charge retained rows and
suspended continuations to the existing storage limits. Actual const impls may
satisfy ordinary uses, but only coarse candidate indexes erase effects: canonical
headers, exact goals and completed proof keys preserve them. The
[reached-call checkpoint](docs/pa31-reached-const-payment.md) establishes the
argument-free profile. The [closed callable execution checkpoint](docs/pa31-const-callable-execution.md)
adds a complete owned function item and a `u8` argument, with actual Invoke
execution. Preserve empty quantified binders when copying requirements, then
open them with the existing binder-aware fold during each fresh payment.
Closed direct-call validation consumes the actual completed signature: primitive
scalar operands and owned safe Rust function items may have different argument
and result types. Use the existing full callable substitution for inherited and
own formals, indexed once per body rather than rescanning unrelated declarations.
Precharge every attempted signature, generic argument and evaluated operand read
against the configured cumulative interpreter budget at lookup, payment, resume
and consumption. Warm lookup, malformed values and failed resumes retain their
attempted work; failure clears the one-use continuation. Structural validation
never supplies a caller proof. Conditional callback targets need their own
Invoke-bound payment; open or lifetime-bearing contracts, aggregate/borrowed
callable arguments, intrinsic callbacks and CTFE cleanup remain separate
requirements.

Retain each source call's defining owner and effect through a compact indexed
handle into the current catalog. Share one immutable descriptor per semantic
body rather than repeating its full context at every call. Validate the actual
owner on lookup; a copied handle cannot authorize another catalog or body.
The defining effect remains distinct from the reached caller's Const payment.
Keep ordinary calls allocation-free and preserve the common operation and node
fact layouts. Preflight bounded descriptor storage before publishing any rows.

Source Try blocks and question-mark operations use one authenticated protocol
plan per actual operation. Select real Try/FromResidual methods, associated
Output/Residual/TryType and ControlFlow variants from their source declarations;
implicit calls demand the same ordinary callable templates as written calls.
Reuse lexical frames for nearest try/function/closure targets and normal cleanup,
move and region proof. A retained plan ordinal belongs to its actual source body
and current catalog. Grammar admission never grants imported protocol transport
or unsupported typed heterogeneous inference.

Caller tracking belongs to the completed source function header and its actual
Rust ABI. Reuse sparse call-context and callable-input owners for sealed site
data; the presence of metadata never establishes a completed callable-target
proof. Canonical nominal constructors are known untracked through the existing
constructor inventory and actual callable/result owner, not through missing
function headers. Legacy imported headers and local source-client functions
use their positively authenticated catalog/profile to establish the closed
untracked ABI. Empty caller-site data keeps the current call-context storage
budget and cannot borrow a missing local source-header producer. Actual new
site data retains its source-owner prepayment. Unknown policy remains an error.
Keep the source signature unchanged and distinguish tracked native signatures
explicitly.
For a trait callable, its symbolic declaration alone cannot determine whether
the selected implementation needs caller data. Borrow the completed selection
at the actual callee use, match its substituted subject and complete trait
application, and query the same direct/inherited/default member resolver as
dispatch. Preserve the implementation's separate captured lifetime prefix and
named generic arguments with the shared implementation mapping. This header
lookup supplies no new trait, API, const, lifetime or body permission. Charge
the touched evidence and member searches; do not repeat selection or scan all
implementations. Retain the original symbolic callable in the prepared site
until dispatch rewrites both the operand and sealed site through its proved
substitution. Ordinary nontrait calls retain their existing fast path.
Runtime forwarding and CTFE frames consume the same prepared static Location
values. Store immutable snapshot site data once as a typed footer after legacy
prepared operands, with indexed queries rather than per-call scans. Count its
actual alignment, every allocated legacy operand's width and each enclosing
owner's growth. Function-pointer/dynamic adapters and policy transport require
their own real consumers before admission.

Derive a function's defining context from its actual completed declaration and
direct associated owner. Runtime, const function and const-trait default are
distinct; a default's conditional defining legality never makes its body an
executable const function. Retain an implicit conditional Self premise only in
that actual default's body environment. Abstract member checking consumes the
typed Call, complete target and original clause provenance; a familiar name,
cached Code or copied effect handle grants no execution authority.

Collect a required cleanup carrier for each actual const-trait default before
proof. Prepare it after source Copy/environment completion, binding the exact
function, source, original signature and Environment owner. Consume it only for
the proved active cleanup Schedule; moved and proved dropless values introduce
no destruction demand. A direct rigid parameter's explicit conditional Destruct
premise and its ordinary Copy no-drop premise retain separate validated role
identities. Clear defining-only claims during specialization. Missing carriers,
unprepared roles and foreign environments reject rather than becoming empty
proofs. Index claims once per actual environment and charge failed work as well
as success; preserve the allocation-free ordinary path. The
[defining cleanup checkpoint](docs/pa31-defining-cleanup-preparation.md) records
the focused profile and its remaining projected, composite, quantified and
imported-owner obligations.

Actual source const destruction shares the existing heterogeneous trait-goal
frontier for genuine const Drop and substituted semantic fields. Preserve the
one initialization result and cleanup plan through preparation and loan proof;
do not replay initialization or clone the CFG to recover cleanup activity.
Eligibility and physical cleanup activity are distinct: a partially moved Struct
still needs its whole-type eligibility proof. Bind transient completion to the
current arenas, immutable body, verifier, Copy evidence and environment. Recheck
only contracts with newly selected lifetime debts before normal loan completion.
Publish a checked future arena witness only after every staged loan proof passes.

Drop coverage proves a binder-scoped bound, such as paren-sugar `FnOnce(T)`,
inside its own scope; a const Drop impl's `[const]` bound is checked as the
ordinary bound inside the same binder. Region coverage ignores quantified trait
bounds and rejects a quantified outlives body by name. A pattern type is
`[const] Destruct` exactly when its base type is.

Native instance demand may mark an authenticated Function materialization as
runtime after the real checked-caller join. Its original and separate CTFE
materializations retain their const context; exported instantiation does not
inherit that native-only choice. Active interpreter cleanup stays fenced before
Code publication until actual destructor scheduling and execution exist. Selected
enum leaves, initializer-owned eligibility and complete VaList cleanup remain
required. The [source destruction](docs/parallel/source-const-drop.md) and
[runtime evidence](docs/parallel/source-const-runtime-evidence.json) distinguish
these completed proof joins from the remaining execution work.

The [combined producer/defining checkpoint](docs/source-defining-producer-checkpoint.md)
qualifies this boundary alongside raw source-provider collection, including the
actual original's Required-to-Prepared transition and immutable Environment.
Its width/noise measurements distinguish indexed preparation work from the cost
of parsing and checking unrelated source. Source rows still require validation,
authenticated transport and fresh client proofs before execution.

Validate raw source row domains through borrowed header, binder and type-recipe
indexes. Traverse the binder forest once, resolve bound-depth queries in that
traversal, and answer later ancestry queries from preorder intervals. Summarize
shared recipe DAGs once and check each consuming root against its actual scope;
do not multiply query width by binder depth or copy a body's enclosing context.
Canonical late-region indices must be unique within their function even when
the underlying slot references differ. Charge attempted work, actual retained
capacity and peak scratch storage, including failures. These data indexes grant
no descriptor or import authority: substituted signature equality, operation
typing, complete obligations, authenticated remapping and fresh client proofs
remain separate requirements. See the
[source domain index checkpoint](docs/pa29-source-domain-index-checkpoint.md).

Callable signature equations do not establish const-callability. For actual
source Fn/FnMut/FnOnce roles, nonordinary use validates the completed declaration,
source constness, prepared effect and exact substituted full function item.
Reuse the typed dispatch instantiator and charge its work; closed subtrees must
not be rescanned for every caller. Ordinary callable uses retain the early path
without reconstruction. Function pointers cannot supply a const implementation;
missing headers/templates remain pending, while conditional item/member debt
retains its complete Instance until scoped proof is implemented. Reference
subjects use actual source impl selection rather than effect-erasing forwarding.
The [eligibility checkpoint](docs/pa31-callable-const-eligibility.md) records
source selection independently of the closed execution profile above. A complete
signature alone grants neither an execution entry nor another call's payment.

Captured const calls consume the actual checked closure original, its selected
entry, full instance and ordinary region/loan proof. Retain and reuse originals
through the existing indexed source-component owner; specialize the borrowed
original through the same materialization and publication kernel as functions.
Keep displaced source contracts separate from primary contracts during Copy
preparation. Share one paid body index, reject duplicate source inputs, and
consume each primary body exactly once with its exhaustive ownership checks.

Executable capture records belong to their final proved Code. Store the optional
shared context and ordered call rows in the existing operand vector, binding the
current arena, owner, source, optional instance, statement/site, destination,
receiver and evaluated full target/signature/result. Prepay context/Rc payloads
and vector growth. Build records in linear statement/call work, perform charged
indexed reached lookups, and validate frame replay in constant time. Ordinary
bodies allocate no capture context. Source snapshots, another Store or copied
operand rows cannot supply execution authority. Generic/conditional caller
debts, receiver adapters, capture destruction and imported transport retain their
own required proofs. The [execution](docs/parallel/const-closure-execution.md) and
[component reuse](docs/parallel/const-closure-components.md) checkpoints record
the qualified finite local profile and remaining requirements.

Runtime `const_eval_select` lowering consumes the actual registered declaration,
proved caller body and reached Call, preserving its logical type arguments and
public/opened signatures. Evaluate every operand once in source order, then
spread the owned tuple into the evaluated runtime callback. The const callback's
obligations remain required, but its unselected executable body is not demanded
solely by this operation. Constant selection needs a separate reached-call
implementation; a source placeholder body cannot supply it.

Keep a function item's constrained logical type distinct from its evaluated
producer and executable target. Index retained checked Constrain edges once per
actual body, then apply the existing callable-target mapping once to the original
producer. An origin table describes structure and does not pay the coercion or
callee proof. Prepare it lazily only for function-item selection, with bounded
attempted work, actual vector payload capacity and sticky failed preparation.
Function-pointer-only and ordinary bodies allocate no origin table. Each
subsequent lookup is indexed; repeated or shared-prefix coercion chains cannot
trigger a fresh walk per selector. The
[runtime selector checkpoint](docs/pa31-runtime-const-select.md) records the
source/native contracts, ownership controls and remaining const profile.

Constant selection authenticates the current evaluated selector, actual callback
and owned tuple against the reached Call in its proved Code. Demand only the
selected constant callback through ordinary function entry, preserving its full
instance and public/opened signatures. Suspension must retain operand values,
the original return destination and one-use caller payment; cached callback code
does not pay another reached use. The
[constant selector checkpoint](docs/pa31-const-select-ctfe-progress.md) distinguishes
qualified source profiles from the remaining binder and conditional-call debts.

A shared-reference selector retains the caller's already-proved whole
input/output relation at its exact Call. Bind that cold descriptor to the
actual finalized Code object, full selector and callback instances, and owning
input row. Matching signature shapes or cached code cannot substitute for this
identity. Finalize the named or anonymous owner before preparing the row;
preflight both owners' storage and publish Code and row together only after
all checks pass. Keep public and owned callback signatures distinct, preserve
the actual late binder, and use ordinary function entry and reference provenance
checks. Named demand constructs its declaration owner before proof; anonymous
expression demand cannot acquire that permission by relabeling a cached result.
Keep ordinary bodies free of descriptor allocations and measure caller width,
warm reuse, owner depth and failed publication separately. The
[reference selector checkpoint](docs/pa31-const-select-reference-progress.md)
records the supported profile and remaining anonymous-reference and broader
callback requirements.

Prepare an owned executable snapshot while its exact Body, Contract and solved
region/loan proof are available. Reuse the solved callable mapping producer,
moving its sparse map into the existing Code field before publication. Empty
static solutions do not scan values or allocate a target map, and original-only
proofs do not build Code. Charge attempted mapping work and preflight snapshot
payload within the existing execution budget. Catalog registration consumes the
owned proved snapshot; later matching body metadata cannot authorize a new CFG.
Keep source witnesses and live conditional scopes as separate prerequisites,
and validate mode/index storage before publishing any executable entry.
The [integrated constant selector qualification](docs/source-ctfe-selector-checkpoint.md)
records output equality, actual reached execution, bounded work controls and
the remaining source callback profiles against this publication contract.

An actual source drop-glue helper is a compiler hook, not an executable empty
body. Its runtime adapter consumes the exact registered declaration, full proved
instance, retained source/body signature pair and caller's destruction recipe.
Ordinary non-hook bodies allocate no adapter query. A dropless answer requires
the same immutable type/definition inventory and completed empty registry;
missing structural or proof data is not evidence of no destruction. Keep sized
thin-reference cleanup distinct from required unsized metadata, owned-box order
and constant-evaluation cleanup. Slice cleanup retains one proved element recipe
and consumes the actual length-bearing wide reference. Validate the selected
reference layout before loading its data/length pair; preserve the public Rust
ABI and pass that pair only to the internal cleanup entry. Share the ascending
array/slice loop without enumerating runtime elements during compilation.
Zero length executes no destructor, while zero stride still executes once per
logical element. Complete-empty cleanup still requires its ordinary proof and
registry checks. Keep recipe traversal, emitted code and inherited dense query
storage measurements distinct. The [sized runtime checkpoint](docs/pa31-source-drop-glue.md)
and [slice continuation](docs/pa31-slice-drop-glue.md) record these boundaries;
owned-box, unsized-tail and CTFE cleanup remain required.

Dynamic helper recipes retain the complete object type and use the caller's
proved vtable metadata. A null drop entry performs no call; a nonnull entry
executes the complete concrete cleanup through the established internal ABI.
Do not change public Rust pointer-pair layout or vtable field offsets. The first
nonempty vtable exposure may precede any drop-helper root: retain its lazy exact
recipe family across table drains, separately from owned-body cleanup proof.
Empty exposure batches leave it absent. Bind that family to stable identities
of the exact type and aggregate arenas before every cold or warm lookup; moves
and append-only growth preserve identity, replacement does not. No address reuse
or same-shaped foreign arena can inherit a cached dropless answer. Charge query,
recipe, worklist and failure storage together under the existing budget, and
poison failed work before warm reuse. The [dynamic cleanup checkpoint](docs/pa31-dyn-drop-glue.md)
records source/native controls and the cold record-size tradeoff.
Internal features active in an earlier library must work then, regardless of
later broad edition/target closure. Measure complete library time and peak memory
as well as changed-path scaling, with existing resource and cleanup limits.

Build orchestration may use an explicitly pinned Cargo unit graph as a package
planner, outside the frontend. Its compiler adapter permits only version and
empty-input target-information queries; Cargo never compiles a target dependency.
Retain each unit's features, edition, full profile and dependency renames. Host
and target units for the same package are distinct: a build script executes on
the host while carrying the configuration of its consuming target. Traverse the
selected graph once, retaining dependency-first order and diagnosing cycles,
missing nodes and unsupported protocol fields. Graph rows and Cargo path IDs
are invocation-local, not portable crate identities or persistent cache keys.
Isolate package-planner introspection switches from compiler-query environments;
enabling Cargo's unit graph must not enable additional target cfg capabilities.
Pair a host script with its exact compilation unit and package snapshot, retaining
distinct compile and execution profiles. Validate every supported profile setting
and reject unsupported ones. Under the course profile, a library target that
lists `dylib` beside `rlib` or `lib` (original std) builds only its rlib and
records the omitted crate type in its unit receipt. Consumers link std
statically, as rustc does without `-C prefer-dynamic`. A target with only
`dylib`, `cdylib` or `staticlib` stays unsupported. A script that catches a
rejected compiler probe must still fail the outer task; probe failure is not
evidence of language incapability.
Retain actual package lint levels, priority order and check-cfg expectations in
host compilation. An explicit per-package lint cap is a separate authenticated
input, included in receipts and artifact keys; do not infer it from source kind
or drop manifest policy. The finite frontend profile accepts explicit
`--cap-lints=allow` (also the spaced spelling), with uncapped behavior as the
default. Carry this typed immutable cap through package, expansion and semantic
request identity. Suppress only diagnostic producers/sinks; retain source
attributes, genuine expectation identities and independent language, feature,
unsafe, API and CTFE checks. Unknown optional expectations may remain witnessed
debt under the explicit cap, never fulfilled or evidence of strict completion.
Capped source observations require separate manifests recording the effective
cap and cannot close an uncapped library goal. Other cap levels remain required.
The [const/macro diagnostic packet](pa40/tests/spec/diagnostic-directives/README.md)
recognizes diagnostic-only requests without installing semantic capabilities.
Check textual literal validity from retained attribute forests, including disabled
owners and nested configuration input; keep optional suffix policy separate from
expression literal interpretation. Under the authenticated Allow bring-up profile,
defer diagnostic option rendering without allocating rendering tables or weakening
actual const/trait/unsafe/macro checks. Custom hard-error text is not suppressed by
rustc's lint cap: its rendering and imported metadata remain explicit unfinished
requirements, with feature-enabled uncapped directive use currently rejected rather
than qualified. Without the actual crate feature, ignore diagnostic metadata in
strict compilation too; internal macro allowances do not enable these handlers.
Keep this diagnostic-only request state separate from semantic permissions.
Documentation options have distinct hard feature and token-origin cfg gates.
The [documentation packet](pa40/tests/spec/documentation-gates/README.md) uses the
syntax-owned cfg grammar and a flat occurrence collector shared with expansion.
Metadata validation performs no configuration lookup and cannot remove program
owners. Preserve malformed-predicate and literal failures independently of caps;
cache crate gate queries only within the shared owner walk, retaining each actual
macro occurrence's permission. Broader documentation diagnostics/output and
imported metadata remain required.
The [advisory request packet](pa40/tests/spec/advisory-requests/README.md) recognizes
intra_doc_pointers and strict_provenance_lints without operation permissions or
binding-mode changes. Rustdoc pointer-link resolution does not create program
bindings; optional provenance casts retain their existing legality and unsafe
checks. Under an explicit Allow cap preserve deferred expectations, not fake
producer events. Reject unsupported strict explicit provenance producers honestly;
keep required positive warn/expect and rendering/transport qualification open.
Native library dependencies are separate from foreign symbol identities and
LowIR. Evaluate a link predicate with the compilation's actual immutable cfg
inputs only after its complete owner survives cfg removal. Retain the result on
that configured attribute; preserve demanded query gates, capture ordinals and
token origins through the existing provenance checks. A false link condition
cannot remove declarations, grant unsafe permission, or suppress malformed link
metadata. Macro allowances do not substitute for the `link_cfg` crate request.
Keep dependency records sparse, bounded and in source order, with no ordinary
per-function row or library-name insertion into the symbol namespace. Sort mixed
cfg uses at most once per owner, never a growing crate-wide prefix.
The external native manifest is explicit filesystem transport consumed only by
the LowIR backend bridge; missing transport must fail before publishing outputs.
Each crate artifact owns only its local dynamic dependencies. Final linking
visits each imported artifact once, without repeatedly copying transitive plans.
An optional versioned envelope preserves old metadata bytes for crates without
dependencies, rejects nested envelopes and enforces combined budgets. Static
archive bundling, modifiers and other kinds retain explicit required boundaries;
do not translate LLVM intrinsic names into ordinary C linkage.

LLVM exact names have a cold typed catalog in the existing symbol-name owner.
Classify a reserved name during interning and reuse its operation/width fact;
ordinary foreign records gain no per-function field. Check the first actual
foreign link name against the item's crate feature or authenticated item-span
macro allowance. Query permission once per pass, share the allowance session,
and construct any foreign-owner index once rather than rescanning declarations
or growing macro headers for each use. The crate-level permission set has two
fixed u64 words: the real LLVM gate crosses bit63; the compact macro grammar
mask stays one byte and no function/body row gains feature storage.
At demand, validate normalized input/result integer widths and the existing ABI,
then emit one typed shim per selected callable. The scalar unary kernels use
parallel bit-group transforms with logarithmic work in the fixed width, preserving
logical shifts and ordinary signed/unsigned result views. Reached unsupported
operations and LLVM storage cannot fall through to the native C symbol path.
The [LLVM packet](pa28/tests/spec/llvm-linkage/README.md) retains the required
vector, target, floating, immediate/poison, address and imported consumers.

The manifest's `public-dependency` Cargo feature controls
retained target graph edges; it grants no host Rust or compiler-query capability.
Unsupported host edge modifiers, tools, workspace inheritance and other unstable
manifest policy remain explicit errors. Parse the manifest once per
package and reuse its environment and lint arguments across host units.
Planning receipts are not compilation results. The
[planning adapter](docs/crate-planning.md) supports frozen local path inputs and
authenticated crates.io archive snapshots. Preserve original package bytes and
validate the complete registry lockfile set, including inactive dependencies.
For local packages, freeze the complete declared source tree, including directory
entries and file modes. Keep portable caller-declared origins and inventory hashes
separate from invocation-local Cargo path IDs. Changed or newly added files must
invalidate the snapshot. Generated output belongs outside those immutable trees;
source snapshots alone do not close workspace configuration or rerun inputs.
Hash each package once per validation boundary, then use indexed source ownership
to bind its units; do not rescan each package per host/target unit. Execute each
selected host compilation/run once in dependency order. Share a compiled script
across its host/target executions while retaining separate configuration and
generated outputs; apply own-script effects to the exact consuming unit and
pass links metadata only along immediate script edges. Use one host library
inventory rather than copying transitive dependency closures per consumer.
Bound aggregate graph outputs and append completion receipts; rewriting the
growing graph after every process is quadratic. Publish no result until the
package/configuration/artifact/generated-input closure has been revalidated.
Invocation-local completed results do not establish a persistent cache key.
Complete local
source/configuration closure, per-crate edition semantics and the target executor
remain required before upstream acceptance. Cargo's implicit sysroot is not a
selected package unit: add original source-sysroot jobs explicitly.

Target crate inputs distinguish the complete metadata artifact inventory from
source-visible dependency aliases. The driver loads each canonical artifact
path once across both kinds of input, under shared count and byte budgets.
Transitive metadata validates the dependency graph without publishing an extern
prelude name; only explicit direct aliases grant that binding. Retain ordinary
identity, visibility, graph and proof validation for inventory-only artifacts.

## Source, lexing and parsing

Read source in bulk; retain immutable source images and compact file/offset
identities. An ordinary LF source requires no normalization copy. A CRLF file
may keep one normalized view beside its original for physical diagnostics.
Do not build per-character records. Source line/column lookup is cold and lazy.
Identifiers use pinned Unicode XID and NFC rules when interned; raw spelling
and semantic identity are different facts.

The lexer yields typed tokens with borrowed source ranges on demand. Ordinary
syntax consumes a bounded cursor. Retained token trees are allowed for macro
definitions, invocations, procedural-macro interfaces and source provenance;
they are not an excuse for successive owning token streams of the whole crate.
Raw strings and nested comments must scan linearly in input size, with bounded
delimiter lookahead. No token spelling allocation on the ordinary path.

Parse each source or newly produced expansion fragment once, apart from bounded
grammar checkpoints and macro matcher alternatives. Macro fragment matchers
call the ordinary Rust parser. Retain captured opaque fragments and hygiene
identities; never stringify them to recover syntax. Memoize repeated matching
states and diagnose resource limits rather than retrying exponentially.

Macro expansion and resolution form a worklist with explicit unresolved,
in-progress, complete and failed states. Do not repeatedly scan the whole crate
after each new item. Spans carry compact source and expansion context IDs.

Authenticate source-declared builtin identity and macro kind independently of
execution availability. Retain known unused declarations without manufacturing
an implementation. Wrong-kind or unavailable invocations fail before matcher,
capture and expansion work. New declaration-only roles must not grow ordinary
bootstrap tables or grant permission through an alternate attribute consumer.

Compile each builtin's actual source declaration grammar before replacing its
expander with the authenticated registry kind. Source form continues to govern
textual versus item visibility, hygiene and publication. Typed lookup traverses
existing textual scopes for the expected kind, preserving the unfiltered cache
and unresolved imports. Charge skipped candidates and warm failures; a filtered
miss never becomes an unfiltered absence. Share the actual kind classifier
across scoped and legacy publication without adding another cache or owner.

Metavariable concatenation belongs to typed template grammar and transcription.
Its macro_metavar_expr_concat feature is independent of macro_metavar_expr.
Consume actual captured identifiers/literals and preserve generated-name hygiene
through the ordinary token/name producers; never reparse a rendered template or
skip unknown operations inside a later cfg branch. Declaration admission alone
does not establish invocation, generated-name lookup or execution.

Keep concatenation operands in the existing flat macro node/edge arenas and
select captured values through the actual repetition binding path. Reuse the
canonical name arena for generated identifiers; do not attach an owned String
to every token or transcription piece. Read forwarded literals from their
retained typed payload and original spelling, preserving fragment restrictions
without rerunning lexical or expression grammar. Generated spelling requires a
synthetic source origin with the actual definition context and edition, even
when an operand was captured at the call site. Bound operand, emitted-byte and
repetition work; feature checks grant syntax authority only. The independently
qualified [protocol sources](pa31/tests/spec/metavariable-concat/README.md)
separate definition admission, actual invocation and generated-name resolution.

The pinned concat parser requires the actual crate feature even for generated
macro definitions with an internal feature allowance. Preserve that distinction
from invocation-time allowance consumers. Literal symbols may still be typed
DeferredLiteral payloads: validate suffix/radix through the shared literal
producer without evaluating arbitrary-width integer spelling. The original
source and typed fragment identity remain authoritative.

Count, ignore, index, length and escaped-dollar templates require the actual
crate's macro_metavar_expr feature, independently of concatenation. Compile
their operands once into the existing template grammar and emit typed numeric
tokens with definition-site origin. Reuse the winning binding trace and active
repetition frames; counting must retain the pinned first-empty-branch behavior.
Build a preorder interval index only when a real subtree count needs it, and
share typed driver sets and active capture cursors across nested substitutions.
Prepay index growth and failed queries through the existing transcription work
limit. Do not rescan a whole matched subtree for every count or walk every
ancestor again for every emitted capture. Preserve ordinary token/IR widths and
measure wall time separately from bounded-work growth. The
[expression protocol](pa31/tests/spec/metavariable-expressions/README.md)
records independent outcomes, native execution and remaining measurement limits.

Keep actual expansion ancestry separate from a token's syntax context and its
physical source/edition provenance. A captured name does not determine the
contexts of the surrounding generated macro body. Mark each definition token
from its own retained parent; preserve captured contexts. Cache item and local
normalizations when interning a context, with distinct typed normalized keys.
Mixed-transparency rebasing shares definition prefixes by normalized call-site
context while final interning retains each actual invocation identity. Measure
first-miss depth across distinct call-site contexts as well as cache hits.

Share one append-only actual invocation ancestry index between textual lookup
and resolved bindings. A nested query must not walk the complete parent chain
again for each name or stripped mark. Keep the physical root allocation-free;
retain one parent, depth and skew-binary jump per actual invocation. Publication
combines adjacent equal jump lengths with amortized constant append work, while ancestry queries
skip complete blocks and split only the block containing the requested depth.
Compare against an independent parent-walk oracle and measure depth and query
width separately, together, and after unrelated sibling publication. This index
provides ancestry only; it grants neither hygienic equality nor privacy access.

Index local shadow histories by binding entry and normalized context, retaining
every declaration version needed by definition-site visibility. Unrelated
contexts must not be scanned for duplicate checks or historical lookup. A
single-local entry needs no history allocation; older shadows use an ordered
predecessor query. Scoped item bindings and import dependencies must likewise
carry their normalized item identity, including per-segment path contexts.
Hygienic selection does not grant visibility or library-feature permission.
Check semantic type visibility from the physical body module after publishing
solved types, adjustments and selected calls. As in rustc 1.98, an anonymous
constant of an item signature (a length, const argument, discriminant or item
annotation, and the constants nested in it) skips this type check but keeps
associated item and field privacy; one nested in a body keeps it. A
hygienically selected private
function still has a private function-item type; a public function pointer may
erase that identity inside its defining module. Walk signatures, nominal and
trait identities, associated items and generic arguments through completed typed
facts. Opaque interfaces expose their declared bounds and captured arguments,
not their hidden representation. Reuse visited type nodes within the immutable
body instead of flattening shared graphs at each expression. Primitive-only
checks allocate no graph storage; build physical module ancestry once, lazily,
and charge attempts and edges against the work budget, including failed uses.
Retain immutable import paths once and share them across dependency retries;
waking an import must not copy its complete segment sequence.
Flatten grouped import trees with one append/truncate prefix buffer and retain
each published leaf once. Preserve every segment's actual identifier context,
the explicit alias context and the glob star context independently. Root-only
paths need no context allocation. Do not mistake eliminating prefix copies for
eliminating repeated semantic scans of those prefixes; measure both dimensions.

Carry the same borrowed name/context view through lexical lookup, qualified
segments and builtin-prefix preparation. Pool complete path scratch buffers;
keeping only the name vector would recreate contextual scratch on every body
reference. Record actual macro definition scopes beside invocation ancestry and
actual container introductions beside generated scopes, in one cold owner rather
than fields on every scope or definition. Container adjustment caches only pure
identity results; caller-dependent visibility remains separate. Self/super tokens
each retain their own context, and visibility paths must still name direct
ancestor modules.

Trait-method discovery visits actual item scopes using the method identifier's
context, while collecting all trait bindings in each reached scope, including
opaque trait names. Preserve item/statement-macro block boundaries before
expansion, even when their output is empty. Value-only scopes and parenthesized
expression tails do not create those boundaries. Partition member contexts
before copying candidates, retain both original and walking contexts in lookup
memoization, and check actual container identity and caller visibility separately
from reusable candidate lists. The
[trait discovery checkpoint](docs/pa31-trait-discovery.md) records independent
scope opponents, linear sibling work, cold storage costs and remaining consumers.

Glob journals retain source keys while contributions and withdrawals retain the
mapped destination keys. Reverse only the synchronized glob/use context mapping;
an ordinary context adjustment is insufficient. Reuse a transformed suffix for
names sharing a context and keep invalid provenance from publishing answers.
Scope lookup results and missing names are not immutable context facts: later
binding publication must still wake their exact dependency keys. The
[item lookup checkpoint](docs/pa31-item-lookup.md) declares the implementation
boundary, structural opponents and cold storage costs before qualification.

Keep root bindings in the flat table and allocate a separate context index only
for non-root item keys. Both indexes share declaration ordinals and first-binding
order; removing a binding retains its key. Size root slots by root population,
not by all contexts. Import waiters, candidates, publications and withdrawals
retain the full key. Name-only metadata must reject retained non-root items until
their provenance can be transported. Snapshot import journals at first-binding
order tails, not entry creation ordinals: an old local-only entry may become an
item later, and a declaration batch must not enter its own initial snapshot.
Capture tails once per existing indexed scope and materialize each journal only
on demand. Record this cold snapshot cost and context-bearing key sizes.

Written lifetime quantifiers retain lexical identity, including an explicit empty
`for<>`. A where-clause binder owns its subject and all its bounds; a trait-position
binder owns only that bound's arguments and associated equalities. Decode each
shared subject once in its source context and relocate captured subject types
with one cached structural shift. Preserve unused slots, enclosing captures and
nested callable binders through normalized and original predicates. Never flatten
these clauses for a consumer that only handles ordinary predicates. Local
shorthand projections may infer bound lifetime arguments in their own body;
signatures need explicit arguments when those quantified locals occur.

Parse a higher-ranked trait binder before its const modifiers into the existing
Bound fields. Optional relaxation cannot combine with that binder. Preserve the
actual binder scope and feature checks; consuming an anonymous impl Trait bound
must not erase either. Invalid modifier order and modified lifetime bounds fail
through the ordinary parser rather than being reinterpreted downstream.

## Semantic identity and demand

Intern names, types, declarations, trait goals, instances and layouts with
compact typed IDs. Canonical equality is O(1). Dense per-node facts belong on
nodes or dense side arrays; sparse composite query keys may use hash tables.
Rendered names/types are never semantic keys. Deterministic output must not
force ordered node-based containers onto the lookup hot path.

Associated type projections retain their declaration identity, Self, complete
trait application and separate family arguments. Their immutable schema fixes
the declaring trait, result sizedness and declared Copy/Clone promises.
A projection is not a nominal type:
equal projected results do not establish equality of their inputs. Keep its
inputs invariant and preserve them through parameter/lifetime substitution at
the actual binder depth. An explicit dependency flag also identifies closed
projections that still need normalization. Structural properties and occurrence
sets are not implementation evidence or an injectivity proof. Concrete layout,
Copy/equality, destruction and lifetime queries must not infer result properties
from an unfamiliar constructor or silently treat it as a primitive.

Shape queries preserve the binder depth of captured associated inputs, including
empty callable binders. Match trait premises and explicit Sized subjects by
lowering only the surrounding prefix into existing canonical identities. Keep
nested local binders unchanged; a captured local cannot become an outer premise.
Share substitution's dependency walk and typed rebuild dispatch, but perform
these lookups without interning or cloning the assumption environment. An
immutable assumption already owns every canonical child of its root, so absence
can withhold permission without asserting that the type itself cannot exist.
Cache such misses only for that fixed environment. Keep ordinary root caches
separate from lazily allocated scoped caches and charge all lookup work.

After retaining input formation, a unique exact parent premise and complete
associated equality can resolve a projection in the caller's canonical namespace.
Use the existing member and trait indices. Competing ordinary or quantified
clauses and associated result rules retain scoped selection; an equality alone
never proves its parent. Family input requirements still need their own proof.

Lifetime-only substitution shares immutable environment partitions whose clauses
have no escaping lifetimes. Maintain that summary when inserting or substituting
clauses, including quantified result types, rather than scanning the entire caller
again for each request. Share parameter abilities and unchanged builtin, trait
and equality storage; clone a partition only when it changes. Parameter
replacement must still visit closed partitions because it can introduce captured
lifetimes. Shared lookup data never carries a query's inferred equations or proof.

Builtin promises are conditional declaration facts, not parent formation
or normalization authority. Register them before interning applications; never
change them beneath cached queries. Copy entails Clone and both entail Sized.
Copy queries may consume these promises without exposing the result's structure;
only Copy can remove possible destruction and opaque drop-region requirements.
Clone alone cannot remove those duties. Preserve separate input and lifetime
proofs, and never infer an input ability from the result. Close builtin promises
over resolved Self supertraits once, before projection schemas are registered.
Use the already decoded declaration bounds, with a bounded monotone worklist;
ordinary direct-builtin declarations need no closure allocation. Bounds on other
parameters or associated members grant no Self ability. An alias that later
normalizes to Self remains a separate required predicate, whereas parentheses
preserve the resolved Self identity. Keep the original complete predicates for
formation and implementation validation; these early bits are no substitute for
those proofs and must never be patched beneath cached type properties.

A static that is not `mut`, not foreign and not thread-local requires a
`Sync` type, as rustc's well-formedness check does. With source bindings, the
`sync` lang item must exist; when it is absent, the static is rejected with
rustc's own text, "requires `sync` lang_item". Otherwise the static's type
proves the source trait's application with the static's declaration proof.
Explicit positive and negative impls replace the structural default, and a
source raw pointer otherwise follows its pointee. The built-in library proves
its structural `Sync`. A `static mut` needs neither the item nor the proof,
and an unprovable type is rejected with "shared static type must implement
Sync". See the [shared-static checkpoint](docs/parallel/shared-static-sync.md).

Auto-trait permissions are separate from Sized, Copy and Clone. Structural
auto-trait queries use `(type, marker)` nodes: reference rules can change the
required marker, and callable signatures are not stored values. Inspect actual
substituted fields rather than all nominal type arguments. Traverse shared
subgraphs once and propagate failing leaves over reverse dependencies before
accepting recursive components. A provisional cycle cannot become a completed
positive proof. Suspend at missing field declarations and projected obligations, retaining
the latter's lifetime evidence on the requesting selector's existing stack.
Immutable structural facts may be cached; caller permissions and witnesses must
remain in their own proof scope. Object auto bounds add no vtable method slots.

Source dynamic receivers use the genuine DispatchFromDyn declaration and ordinary
implementation selection. Open the complete method signature and its declaration
environment, retaining late binders and unused formals; body-specific aliases do
not replace that environment. A query-local target may reuse the genuine Unsize
formal only after proving it absent from the signature and complete scope. Keep
the actual Self/target/principal premises local, and authenticate source headers,
arena identity and completed declaration/impl state on warm entries too. Receiver
compatibility grants no vtable, instance, body or metadata execution permission.
Preserve written impl bounds rather than inventing an Unsize premise on every
Dispatch impl. The [source receiver checkpoint](docs/source-dispatch-from-dyn-checkpoint.md)
records its source and native consumers.

Explicit auto-trait impls use typed builtin identities in the ordinary header,
overlap and candidate indexes. Any explicit impl replaces the structural default
for its whole nominal family, including applications that fail its conditions.
Reserve pending alias headers before selection; their absence is not evidence
for automatic implementation. Retain ordinary type, const and lifetime evidence.
Coinduction requires an identical complete predicate and proof scope along an
uninterrupted auto-trait path. A shape-only repetition may terminate search but
cannot grant permission. Preserve static lifetime positions in that cutoff.
Shared cyclic subgoals may reuse completed provisional proofs only in their active
auto segment; never cache a bare cycle assumption. An exact auto predicate exposes
no associated outputs, so its full witness DAG can be shared in the same scope.
A failed or deferred child invalidates the entire segment, across nested scopes,
before another field or candidate proceeds. Goals without contextual lifetimes
use the caller input scope; predicates mentioning local lifetimes retain their
original universe. Mark dependent proof nodes transitively, and exclude them
from completed and cross-query caches until their component is established.

An associated type definition is checked under its impl's immutable environment,
including when no caller uses it. Prove its well-formedness, declared result
bounds and lifetime requirements; the definition itself cannot add assumptions.
Match required types by declaration identity and type namespace, retain the
selected definition with the validated implementation, and share lifetime queries
across all associated definitions of that impl. An ordinary alias's deferred
well-formedness policy must not weaken an associated type contract.

Associated families compare their input domains before checking their results.
Map the trait and impl parameters simultaneously into the impl's complete
captured lifetime scope. To reject stronger impl requirements, use the declared
type/const identities as rigid inputs; the impl's implicit Sized promises must
not become assumptions. Then use the impl's own identities and requirements to
prove formation of the declared application and the definition's result bounds.
An omitted requirement is not inherited merely because the trait declared it,
including when the result erases that input. Elaborate these explicit input
assumptions through the shared trait machinery, excluding the family's result
promises. Discharge each family's lifetime proof in its own scope, registering
captured parent parameters once and distinguishing them from own lifetimes.
An empty input contract is complete only after parameter/bound preparation;
an unfinished contract cannot stand for an empty one.

Method and constant conformance must normalize each family application through
the ordinary selection proof, even when its definition was checked locally.
Retain the implementation's written type: a normalized scalar does not erase
the original application's formation requirements. Only the declared method
signature grants implied lifetime conditions; constants grant none. Conditions
captured by nested callables may escape to the enclosing method, while conditions
on those callables' local lifetimes remain obligations in their own scopes.
A nominal or family input clause mentioning its own quantified lifetime cannot
grant an implied bound. Filter the whole clause before decomposing its subject;
independent components do not escape a dependent condition. Reuse the same typed
capture/lowering machinery for quantified and surrounding callable binders, while
preserving explicit static bounds and vacuous quantifiers.
An impl lifetime erased from the executable signature still represents an
independent universal when checking its written formation requirements.

Publish completed family input lifetime schemas separately from result promises.
Share the typed parent/member substitution engine with ordinary normalization,
including Self's separate declaration identity and the complete capture prefix.
Prepare applications during the existing structural outlives traversal before
immutable queries, keyed by member, Self, full trait inputs and own arguments.
Cache substituted conditions, never caller permissions. Exclude dependent whole
quantified clauses from implication and defer whole conditions containing local
callable lifetimes before extracting components. Repeated preparation must reuse
completed applications; charge first preparation and mapping to bounded work.

Naming a family application in a nominal field does not imply its input bounds.
The declaration must establish those bounds independently, including when a
concrete definition erases them. Retain the original projection's proof debt
through normalization and discharge it at the final declaration lifetime gate.
Checked signature inputs may supply ordinary implied conditions to default method
bodies; a result promise cannot grant a lifetime bound on its inputs.

Compute required GAT where clauses from retained typed signatures and written
associated result bounds. Collect complete argument positions across every use
within one member; intersect contributions from distinct members that mention the
family. Method outputs participate in finding uses, while only inputs contribute
implied well-formedness. Exclude static lifetime arguments and callable-local
lifetimes from candidate bounds. Lift outer captures without changing their
formal argument positions. Constants and members of another trait do not
contribute. Keep an empty contribution distinct from an absent use.

Index the member-to-family dependency edges once. Count each contributing member
once per candidate bound and publish only unanimous requirements. When a family's
required set grows, revisit its dependent result-bound uses through a bounded
worklist. Tentative requirements belong only to this analysis; they never modify
caller environments. Finally prove every derived requirement from the family's
actual declared environment. Use the original predicate prefix when separating
written input/result clauses from later supertrait elaboration.

Batch candidate lifetime questions within each immutable declaration environment.
Reuse the existing typed conjunction, alternative and projection-identity tests,
without forcing their components as obligations. Unknown relationships return
false; unfinished proofs, foreign identities and exhausted work are errors.
Cache assumption reachability once per queried source lifetime so asking many
targets does not repeat the same graph walk. Ordinary contracts without families
must not allocate this analysis or scan unrelated member signatures.

Shorthand associated type lookup keeps naming separate from projection proof.
Index written parameter bounds by lexical owner and resolved subject; elaborate
only a demanded bound's positional inputs, without publishing an unfinished
caller environment. This allows `where T: A, T::Out: B` independently of clause
order. Ordinary alias bounds can name a member but cannot authorize its use.
Trait Self also sees method-local Self bounds; trait-impl Self starts from its
own trait application and supertraits, excluding unrelated impl where-bounds.
Concrete types and inherent Self do not infer a trait qualifier. Generic defaults
require explicit trait qualification in the pinned profile.

The source inherent-type extension retains this ordinary projection route when
its actual feature is disabled; definitions still require their own permission.
With permission, trait-impl Self probes its actual ADT's type-name bucket before
trait fallback. Complete pending owners, preserve real member ownership and
hygiene, and distinguish raw name presence from shape, applicability and privacy.
A private or differently specialized inherent alias still prevents fallback to a
trait member. Only an absent raw name or a non-ADT Self resumes the ordinary
projection. Reuse the existing buckets and cumulative lookup budget; no global
scan, speculative alias body, new query flag or dense cache grants this decision.
Full generic/where/lifetime inherent-alias proof remains separately required.

Cache member candidates by complete predicate and canonical name, retaining
type, const and lifetime arguments. Rebase captured lifetime slots into the use
binder before lookup; duplicate identical applications are harmless while
distinct applications remain ambiguous. Run the selected typed projection through
the ordinary scoped normalizer and retain its formation and lifetime obligations.
The candidate cache grants no permission and does not select an implementation.

Opaque projected values in checked generic bodies retain move and possible-drop
requirements unless Copy is proved. Cleanup conservatively retains every input
lifetime and parameter, as runtime liveness already does; this sufficient route
does not infer reverse input bounds from an associated result. Concrete execution
still requires normalized storage and selected destruction. The conservative
input route is not the complete projected-result drop-checking contract.

Associated constant declarations retain their types independently of optional
initializers. Check missing and extra members by resolved owner and name. The
validated implementation publishes the final default/override identities. A
separate declaration-only member table may publish one prepared owner's
shape-checked constant. Request that owner through an exhaustive compatible
candidate query; unrelated pending headers do not delay its member contract.
This grants neither method access nor whole-impl validity. Retain that member's
normalized expected and actual types, selected definition and all formation debts.
Final conformance consumes this exact contract
after nominal variance/outlives facts complete, proves lifetime coercion under the
immutable impl environment and agrees with every earlier selected identity. No
pending member contract may escape the declaration pass. Equal normalized result
types do not erase formation debts from their inputs.
Substitute the trait's Self and mixed arguments into the impl environment once;
normalize projected types using that impl's checked associated definitions.
Structural types must agree, while reference results may satisfy the trait by
outliving its declared result. Preserve invariant positions and every proof debt.

Constants introduce no parameters or predicates of their own, so their annotation
and initializer checks share the completed owner's immutable generic environment.
Retain captured lifetime binders in both semantic checking and initializer CFG
contracts. Required declarations may name unsized types, but actual initializers
must produce sized values. Check type formation and lifetime requirements even
for unused declarations with no value. Type checking is separate from demanded
evaluation: an unused associated constant must not be evaluated merely because
its declaration or implementation was checked.
Before executable lowering, reject non-const calls in every initializer, including
unused values and branches that evaluation would skip. Recognize tuple/enum
constructors by retained callable identity; a function pointer to a constructor
does not grant const-call permission. Earlier type inspection remains separate
from this eligibility proof.

Associated value uses retain a constant instance identity with the complete owner
arguments, including parameters absent from the result type. Solve those argument
lists with ordinary inference; do not invent function types for constants. Keep
inherited bounds, lifetime relations and each caller's selected implementation
proof separate from the structural value cache. Defaults and overrides enter the
selected initializer's own argument environment.

Build associated initializers as checked templates, including unused values and
closed owners. Promotion may retain an eligible but unevaluated value; required
constant evaluation still diagnoses its faults when demanded. The pinned profile
also diagnoses cycles among closed trivial initializer aliases before demand.
Selected dependency graphs include branches that value evaluation can skip.
Retain callable dependencies in those templates as complete function-item types.
Substitute the selected initializer's owner arguments before selecting trait
method overrides, including function pointers nested in products. Each caller
retains its own selection proof; a cached structural target grants no permission.
Callable bodies are executable dependencies, not constant-evaluation edges.

Share completed dependency closures by the selected instance's complete, closed
arguments and implementation-availability mode when their proof is empty. Every
transitive initializer must have completed typing; an absent dependency list
before that point is not an empty graph. Propagate incompleteness through shared
diamonds. Caller environments, universal mappings and outstanding lifetime debts
disable this cross-caller reuse. Root selection still proves the requesting use.
Member readiness consumes the completed closure before collecting its edges;
a cached value alone cannot skip those checks. Counting only fresh selections
would miss a quadratic scan whose every selection is a cache hit.

Closed dispatch goals may share an independently established implementation proof
once candidate headers are frozen. Key that proof by the complete predicate and
availability mode. Select in an isolated inference context, retain every committed
equality, restore all implementation arguments through the proof's equality
components, and require every lifetime obligation in an empty environment.
Structural closedness only establishes independence; it never grants permission.
Type-outlives requirements decompose actual components without assuming unfinished
nominal formation contracts. All declaration conformance gates remain mandatory.
Publish only targets whose complete arguments contain no private or external
identities. Otherwise retain the original proof and selected arguments in the
requesting body without repeating selection. Caller scopes and universal mappings
keep their own proof even when an independently checked closed target exists.
This lets static lifetime-bearing constants reuse both selection and dependency
work without promoting a cached structural target into implementation evidence.

Evaluate the retained expression template under a distinct instance context.
Node, place, contents and in-progress dependency states must use that context;
ordinary constants and statics retain their global identities. Substitute stored
types and const arguments without modifying shared source facts. Share repeated
values by selected identity and complete arguments, including persistent data,
while preserving independent callable, lifetime and destruction proofs. Bound
both instance discovery and the expanded evaluation work.

Finalize enum discriminants after implementation validation. Their associated
values use concrete dispatch and the shared instance evaluator; retain selection
obligations under the requesting anonymous constant body. Publish each fresh
region interval before lazily typing another initializer, which can allocate
additional regions. Apply the shared capture check before evaluation: a constant
initializer that ignores its arguments does not permit a discriminant to mention
an enclosing generic parameter. Declaration-time const arguments have separate
dependency requirements and cannot borrow this later phase's validated
implementation state.

Prepare qualified and implicit trait value paths with explicit declaration work
items for their prefix, selection and written arguments, including inherent
selection and its trait fallback. A nested const argument
must not drain an unfinished parent declaration continuation. Reuse completed
path facts, and prepare the selected initializer in its own lexical and generic
context. Do not pre-scan an associated declaration's default before value
selection: an override can remove that dependency, and eager preparation can
invent a cycle. Unselected initializer templates still undergo their ordinary
body and eligibility checks. Const arguments reached after implementation validation may
use concrete value dispatch, retaining proof under their anonymous constant body.
Queue typed const-argument evaluation ahead of the consuming application. The
application reads only a completed value keyed by source identity and expected
type. If evaluation discovers an unprepared selected initializer, suspend with
that exact definition and prepare its declaration dependencies in its own scope.
Preserve expression/instance caches, active cycle states, work continuations and
the work budget across suspension. Resume the requesting anonymous body without
retyping it or reselecting its already inferred dependencies. Initializer path
readiness is separate from body type checking and completed value caching.
Reuse cleared body work vectors, the expression scan buffer and call-typing
storage across suspension. Clear caller evidence before returning that storage;
neither active inference nor a previous caller's permissions belongs in scratch.
Probe completed method-path caches only after recognizing a relevant value path.
Nested shape queries must retain declaration-cycle detection across suspension.
Do not inspect unrelated implementation bodies or restart a long query for each
dependency. Early signature and default-argument queries use declaration-only
selection explicitly. Suspend the existing value continuation for
exact member-type requests, preserving typed root facts and selected-instance
readiness. Normalize only that member's type dependencies: unrelated associated
definitions can depend on its value. Do not eagerly require all method signatures
or all associated definitions to authorize one constant. Whole-impl, unused-item
and coherence checks remain mandatory before completed facts escape.

Reserve resolved implementation trait identities and conservative outer subject
heads before elaborating headers. Share the iterative raw-head query with Drop;
aliases and defaults must not evaluate constants or construct semantic types just
to exclude an unrelated owner. Unknown heads remain possible candidates. Index
pending owners by trait and head, retire them without scanning other buckets, and
withhold a candidate answer until every compatible header is prepared. Header
completion does not validate its implementation or authorize executable operations.

When body trait selection needs a header, retain its immutable caller environment,
candidate cursor, isolated trial equations, proof graph and inference round.
Yield the exact owner through the existing declaration worklist. Repeated polls
of the same unresolved owner do no solver work. Reserve every allocated trial
lifetime before yielding, including discarded alternatives, then advance the
allocation floor past intervening declaration work. Record foreign gaps without
walking or rebasing earlier frames. Final evidence carries explicit owned intervals;
result restoration and lifetime proof must not claim identities in those gaps.
Keep the contiguous synchronous path and caller-assumption shortcut. Globally
shared closed-dispatch proofs still require the frozen candidate inventory.

Once an associated constant path resolves, schedule its declared value-type
prerequisites before anonymous body inference, just as for an ordinary constant.
Reuse the nominal field-template worklist; do not prepare unrelated declarations
or repeatedly expand symbolic field instances.

Index inherent and trait member names before early value queries; these entries
carry identity and owner heads, not executable authority. Obtain a declaration
caller's immutable environment directly from its binder owner. A whole-crate
snapshot made during an early query can miss later declarations and must not be
repeatedly rebuilt to compensate.

Trait-header member naming uses a temporary graph of decoded Self supertrait
identities, separate from proof contracts. Reject identity cycles before expanding
arguments. Index declarations by member name and prepare complete positional inputs
only along routes reaching that name. This permits `trait A: B<Self::Out>` without
demanding B's arguments while naming A's own result. Map the trait's synthetic
Self binder to its actual declaration owner for bound lookup and proof scope.
All written header predicates must be typed before publishing immutable caller
environments. Retain raw projected roots for later formation proofs after impl
declarations are available, even if normalization erases the original inputs.

Classify an owned header result promise by its complete formal parent application,
including Self and every type, const and lifetime argument. Feed that promise to
the associated result contract and impl conformance, separately from required input
predicates. A bound on an inherited result remains an explicit caller requirement.
Closed builtin-property checks normalize in the existing declaration proof scope
and retain selected lifetime obligations; structural caches grant no permissions.
Associated type names participate in qualified member lookup without shadowing
enclosing lexical aliases, including hygienic lookup.

Associated result bounds retain complete clauses, including the separate
subjects of nested result equalities. Elaborate their arguments through the
ordinary bound worklist in the owning trait's lifetime scope. Validate the bound's
own type and lifetime requirements even when no implementation or caller exists.
Conformance substitutes trait Self and mixed arguments together; an already
elaborated implementation result stays in its original scope. Share that mapping
across the implementation's result contracts and retain every resulting proof
obligation. A result contract never adds assumptions to the implementing owner.

Quantified result declarations retain their local binder separately from the
owning trait's captures. Substitute the declaration's inputs beneath that binder
before inserting an already elaborated implementation result at the same depth.
Unused implementations must establish the whole quantified contract, including
its equality and lifetime requirements. Uniform builtin properties may inspect a
clause's bound kind; this classification never provides unbound proof evidence.

A user-trait bound on an opaque associated result is derived only after the
original parent application has been proved. Retain that parent evidence with
the result rule, including lifetime equalities and fresh region intervals.
A derived rule records its complete selected predicate separately from caller
assumptions and implementation bodies. Recursive result bounds are queried on
demand; they do not extend the eager ordinary-supertrait closure.

Close finite supertrait consequences of result clauses once in their declaration scope,
and index these templates by the requested trait identity. Substitute only those
candidates into a query's projection; retain every same-trait alternative for
inference and ambiguity checks. Mapped clauses may share a cache keyed by the
complete canonical projection and requested trait. Enable these caches only
after all result clauses and trait headers are complete; invalidate them if the
headers change. They store typed clauses, never permission for a caller. Every
query still proves its own parent application and matches every type, const and
lifetime argument. Wide result contracts must not be expanded afresh for each
independent caller scope.

Quantified result selection owns fresh witness intervals and records result
provenance distinct from free caller assumptions. The selected graph retains its
parent application proof, including when the mapped rule cache was already warm.
Only ordinary result lifetime promises enter the scalar outlives index. In the
pinned profile, a quantified result outlives requirement does not grant a generic
caller an unconditional static result; an explicit ordinary static bound still
does. Retain the complete quantified requirement for implementation validation.

Symbolic value liveness may conservatively require every projection input to
remain live. This sufficient route preserves Self, trait and family arguments,
uses the bounded component walk, and respects empty runtime aggregates. It does
not reveal exact output components or imply new input assumptions from an output
bound. Closed property queries and explicit result-outlives proof retain their
separate normalization and scoped-evidence requirements.

Type-outlives queries retain opaque associated results as typed atoms, separately
from scalar lifetime and type-parameter components. A reference to an associated
result implies a bound on that result only. It grants no bound on Self, another
input or another associated member. Keep the original binder depth on each atom;
resolve captured lifetimes in the actual proof scope, including empty binders.
An atom involving a locally bound lifetime cannot be decomposed into input
assumptions when it escapes that binder.

Prove an opaque result bound with an applicable environment clause or with all
of its input components. Input components include reference referents even when
the reference lifetime is locally bound by a function pointer. Dropping that
lifetime must not erase an unbounded type parameter or nested associated result.
Match complete typed applications; if two applications
can match only when their lifetimes are equal, retain that equality as a condition,
not an inference equation. Introduce ordinary constraints for the forced input
route when no alternative is available. Keep alternatives in a shared, bounded,
iteratively evaluated Boolean graph. Verify them after ordinary lifetime
inference and again against actual runtime extents. The same alternative must
satisfy both phases; unrelated successful branches cannot supply half a proof
each. No type verification may turn a requirement into an assumption or grant a
projection a type parameter's implicit body extent. Ordinary graphs without
projected tests allocate no verification arena.

Directed callable relations may encounter the same opaque result beneath different
actual binder scopes after local inference freshens surrounding references.
Compare complete projection inputs in those scopes, preserving nested local
binders and every trait and family argument. Equal inputs suffice for equal
results; a result comparison cannot infer input lifetime equality. Retain any
required input equality as conditional verification through both symbolic and
runtime checking. An identical resolved input needs no verification allocation.

Projection normalization uses the common bounded type fold. Normalize inputs
before asking semantic selection for a result; suspend with typed requests when
selection needs the store or declaration worklist. Replacement types retain the
request's binder context and are normalized before their parents are published.
Cache only within one immutable proof scope, and diagnose recursive results or
exhausted work. A prepared impl may supply declaration work, but its complete
conformance remains mandatory before executable facts are published. Missing
lifetime or equality evidence must never be dropped to manufacture a normal form.

Keep semantic normalization in an owned cursor shared by synchronous and resumable
consumers. A selected-definition wait retains the exact owner, member, complete
arguments, lifetime restoration and unfinished fold. Resumption consumes that
selection without invoking its provider or probing implementations again. Preserve
the shifted environments and callable scope map across waits and completed roots.
The owner reserves emitted proof regions before other work allocates identities,
then resumes the same proof at the new frontier with explicit foreign gaps.
A synchronous consumer still requires every selected definition to be complete;
an owned cursor alone does not make its calling operation resumable.

A projection waiting for an implementation header retains its opened caller,
inference goal, immutable environment and exact selection continuation. Its local
provider runs once. Keep a live child's allocation watermark separate from the
parent's published proof interval; reserve both before yielding and resume the
child beyond foreign allocations. A completed child attaches its owned intervals
under the existing callable scope without scanning gaps or duplicating constraints.
Discarded alternatives contribute no lifetime authority. Concrete fallback lookup
remains independent of caller assumptions, and may subsequently wait for its
selected associated definition.

Declaration-only queries may instead own a private canonical lifetime namespace.
Restore selected arguments and result identities before rebasing their completed
evidence into the declaration sink. Such a private template cannot reserve or
authorize body identities. Body annotation queries use actual reservations and
retain a dedicated scoped sink across waits so another query's completion cannot
advance their proof over a live child. Keep the synchronous path allocation-light;
the resumable interface does not require copying ordinary caller environments.

A suspended normalizer has already normalized its request's inputs. Carry that
fact into selection instead of starting another normalization of the same graph.
The enclosing query retains every input proof, including obligations erased from
the resulting type. Within trait search, a derived child retains those proof
nodes itself before it can enter the completed cache: discarding its original
candidate must not discard its lifetime equations. Include earlier existential
intervals when deciding whether that result is reusable without rebasing. Visit
shared evidence nodes once. Represent accumulated input evidence with persistent
prefix links; copying every previous child into each new result creates quadratic
proof storage. Newly discovered implementation premises still need
their own normalization; an input proof never grants a new trait premise.

Before opening lifetime aliases for a normalized opaque subject, consult the
immutable canonical premise indexes. A sole identical caller or result clause
needs no equations, fresh lifetime universe or lifetime-erased cycle key. Retain
the requesting normalizer's input evidence and publish only that exact premise.
Index projected caller clauses by trait and associated declaration; preserve all
competing clauses and rebuild the index when mapping an environment. Possible
caller or result equality rules, additional candidates and nonidentical arguments
require ordinary scoped selection. This shares structural work without caching
permission across callers or skipping a required lifetime relationship.

A qualified declaration projection consumes its subject's already checked type
fact in the same annotation scope. Rewalking every previously checked prefix
turns a linear source chain into quadratic normalization work. Retain the subject's
formation evidence in that scope and normalize the newly assembled outer node.
Substituted trait defaults and implementation replacements can introduce new
projections and still require ordinary checking. A root-scope fact does not
authorize the same graph beneath a new callable binder. Deferred alias and header
formation requirements retain their existing delayed proof boundaries.

Instantiating with no type or const bindings preserves the existing canonical
type graph. Do not materialize, copy and re-intern rigid nested inputs for an
identity substitution. Actual substitutions retain their usual simultaneous
mapping and binder shifts; a later substitution cannot change the original graph.

Callable boundaries in residual types create independent normalization frames.
Requests carry their actual scope identity, parent and local lifetime count;
depth alone cannot distinguish sibling universes. Keep suspended frames in a flat
worklist, charge their combined storage and work, and preserve empty binders.
Shift caller assumptions beneath the complete ambient prefix before selection.
Retain scoped evidence immediately in the actual declaration or body allocator,
reserving fresh intervals before another query can use them. A body evidence
stream can span interleaved allocations only with explicit excluded intervals;
it never owns the foreign identities between its segments.

Open surrounding projection lifetimes with a single checked mapping shared by
the goal and its assumptions. Preserve nested local binders and reserve existing
body inference identities before allocating selection variables. Restore a
fully determined result only through equality components anchored in the caller or
static; an unanchored result lifetime is an error. Partial body inference may
instead determine an unknown input containing fresh existential lifetimes. Retain
those identities only within the selected proof's exact owned interval, and keep
that whole proof in the body. This does not permit unmapped foreign identities or
change declaration-template restoration. Shift restored captures beneath
the result's nested binders without capturing their local slots. Substitution is
not proof: retain every equality, outlives and type-outlives obligation used during
selection, including when the result no longer mentions their lifetimes.

Declaration-time body projections retain their exact query, selected member,
complete arguments, restoration map and proof while awaiting a definition. Commit
partial-input equations once, before yielding; resume result substitution and its
nested normalization without selecting the implementation again. Keep the produced
expression term at an explicit completion boundary so waits do not replay body
inference. Reserve only newly emitted proof identities before each wait, resume
allocation at the current global frontier and publish retained evidence once.
Repeated callers of an already typed initializer reuse its completed facts.
Structural work and retained state must grow with distinct dependencies, rather
than the product of dependency depth and the number of callers. Storage preparation
and other synchronous consumers require their own owned operation boundaries;
this body projection continuation does not make them resumable implicitly.

Group declaration projection obligations by their actual binder and discharge
them against the completed owning environment. Independent annotations in the
same scope use disjoint fresh intervals and share the directed proof. Local
annotations instead retain obligations in the body's allocator and ordinary
lifetime gate; they must not append to a declaration gate that has already run.
Keep plain, closed projection queries off the lifetime-fold allocation path.
Opening a request does not require copying an immutable caller environment whose
retained region summary certifies that no surrounding lifetime can change. Share
that environment even when the request needs private aliases; nested local binders
keep their identities. Caller environments with captured lifetimes still use the
same checked substitution as the request and retain its restoration equations.
Sharing permissions never adds the requested goal to the caller's assumptions.

For captured caller regions, share the canonical premise index and transform only
the buckets consumed by a fresh query when its actual request supplies the whole
mapping. Authenticate the source environment, type arena and exact base index.
Keep query-local substitution, equations, restoration and proof debts separate
from shared structure. Quantified or incompletely mapped forms retain the eager
path. A mapped view cannot expose raw canonical clauses as current assumptions;
consumers must use the indexed view or explicitly materialize it. Charge attempted
inspection, mapping and retained allocation, including failed queries. Verify the
algorithm on actual complete-source consumers as well as structural controls; the
[projection-opening qualification](docs/pa31-projection-opening-sharing.md) records
the unchanged scaling gate, original failures and measured cold-layout costs.

Projection selection in a signature waits for its owner's immutable parameter
environment, including requirements inherited by methods. Generic lifetime
arguments also create this dependency. Nominal defaults demand the existing
nominal contract; an ordinary alias's written bounds do not grant permissions.
Retain worklist dependencies and completion states instead of selecting against
whichever assumptions happen to have been published so far.

When a bound subject or trait argument contains a projection, elaborate its typed
identity before normalizing it, including nominal and alias wrappers.
A private, immutable snapshot of all typed bounds supplies the
assumptions for normalization; publish the final environment only after the bound
roots complete. Struct and enum contracts use that same workflow, with an explicit
typed continuation that completes the owning nominal only after its bound roots.
Do not make an unfinished nominal environment visible merely to break a cycle.
Preserve the original predicate separately from its normal form.
Its original parameter/lifetime occurrences determine declaration-time versus
instantiation-time checking and early lifetime binding. A generic projected bound
may normalize to an unsatisfied concrete requirement without rejecting an unused
generic function; actual calls still must prove that requirement. An unrelated
generic parameter does not defer a genuinely closed written bound.
Normalize the retained type argument roots before rebuilding complete trait
applications; do not reconstruct parameters from rendered output or infer a
projection's inputs from its result. Bounds inside nested callable binders and
dependencies between unresolved projected assumptions require their own scoped
normalization support.

Associated result equalities are typed predicates with two retained roots. The
right-hand side participates in parameter/const occurrence checks, lifetime
capture queries, substitution and inference with the same mapping as the
projected left-hand side. Import both graphs into a probe through one mapping;
keep the probe's variables isolated from the caller. Normalizing either root
does not discard the equality or its remaining const arguments. Result equality
cannot establish equality of a projection's inputs.

Keep equality assumptions separate from builtin abilities and trait membership.
An equality alone grants neither Sized nor Copy/Clone, nor membership in its
projection's trait. Preserve competing results in an immutable environment;
unused contradictory assumptions need not have a unique normal form. Opening
the environment must map both roots and retain the actual lifetime evidence.
An unsupported equality proof must fail explicitly, never fall through a
builtin-only proof loop or disappear when importing candidate obligations.

Preserve graph sharing across the complete predicate, including repeated large
type arguments. Discover and rewrite projected inputs with one mapping per query;
resolve all predicate roots with one read-only traversal cache. Reuse successful
complete input equations only within the same relation and enclosing binder
scope. Compatible later equations preserve those equalities; a different root
pair still requires proof. Keep original projection roots and independent holes,
charge each input, and retain lifetime obligations. Sharing must not turn equal
projected results into evidence that their inputs are equal.

Ordinary aliases and generic defaults retain conditional projection requirements
as sealed typed dependency graphs. Instantiate a contract only at an actual use;
an overridden default contributes no requirements. Retain dependencies even when
an enclosing projection erases their result. Share identical instantiated uses
within the actual declaration binder or body, never across proof scopes, and
bound graph traversal and argument work. Freshen each template's private region
interval before substituting caller arguments; numeric overlap must not turn a
caller lifetime into a template variable. A failed expansion invalidates its scope.

Distinguish an unproved deferred projection from a proved symbolic result. An
ordinary alias may retain unproved type formation, including a closed projection,
until use. Dependent generic defaults similarly retain their requirements; closed
nominal and trait defaults still require declaration-time proof. Ordinary alias
bounds supply no assumptions. Its bare parameter scope identifies type and const
parameters without granting its written predicates. Declaration selection may use
registered derive contracts before fields finish, but final derive validation is
mandatory. Check retained type formation even if an outer alias erases the result
and the lifetime proof is empty; completing that check does not overwrite the
actual annotation's type.

Unused trait-alias defaults validate their directly written closed bounds through
the ordinary parameter-default proof. Preserve the alias's canonical clauses;
do not recursively expand inherited alias edges into extra default obligations.
The pinned compiler accepts an unused default that violates such an inherited
bound. Aliases without defaults need neither graph expansion nor temporary
clause storage for this pass. The broader alias execution/export boundary remains
explicit until its consumers are complete.

Retain the implicit Sized premises of selected implementations and their recursive
conditions when alias normalization removes the original inputs. Alias parameters
can later receive unsized arguments despite their declaration spelling. Substitute
these premises at each use and check them in the actual caller environment. Track
their actual callable scopes across dependency graphs, separately from the local
substitution depth. Derive the total binder depth from that scope tree. Partial
annotations with only liftable Sized premises share the
result's inference variables and retain those premises through body completion;
they do not infer erased inputs from an associated result.

Nested callable annotations retain their enclosing alias/default proof owner and
an explicit universal scope tree. Complete each binder's slot count before sealing
the contract. Cache source scopes as they are discovered; captured-lifetime lookup
uses packed ancestor jumps rather than repeated parent walks. Instantiate each
private witness in its actual universe, keeping sibling scopes distinct and caller
inference variables outside younger binders. Empty binders preserve coordinates
and consume work without introducing new universal lifetimes.

Retain quantified evidence through body collection, specialization and selected
item rewriting. Substitute caller parameters beneath each block's local binder
prefix, and prove the resulting requirements in the same graph as ordinary caller
constraints. Scope IDs are local to one evidence object and must be remapped when
combining independent objects. Partial alias annotations retain deferred projections
and lifetime proofs until their actual inputs are determined. Residual projected
types and erased formation requirements normalize in their retained callable
scopes. Share an immutable binder plan with pending queries so moving or remapping
the original proof cannot change their scope identities. Import each required
scope prefix once per destination evidence object; never clone the full tree for
every requirement or drop its evidence to produce an executable type.

Shared body proofs retain an optional source-label table alongside their blocks.
Label only newly appended evidence, preserving the first demanding root when
normalization reuses an already-proved application. Move new root constraints
into a labelled root block without changing allocation ownership or binder depth;
never copy an earlier proof prefix merely to remember another source. Grafting,
substitution and extension retain those labels. Labels address caller-owned
diagnostic entries and grant no lifetime authority. Validate their source range
before registering constraints; ordinary single-source proofs need no label table.

Completed body nominal requirements consume canonical universal clauses in one
body-local query cache. Reuse declaration selection with immutable caller
assumptions, isolated inference per selection, shared conjunctive openings and
source-labelled retained evidence. Reserve each emitted interval immediately;
resumption and final publication must exclude intervening ordinary allocations.
Publish once after all completed body roots, without copying prior proof prefixes.
Ordinary requirements need no quantified state.

An opening alias names its universal slot directly in the proof graph; it is not
an existential justified by a conditional equality. Preserve that identity through
grafting, rebasing and substitution without letting older variables capture a
younger universal. Before publishing shared body evidence, restore the caller
aliases from their declared identities in one charged fold beneath each block
and nested callable binder. Leave private witnesses and foreign ownership gaps
unchanged. Structural aliases must not transfer a later failure to an earlier
demand or the body as a whole.

Body projection queries also observe partial input changes. A unique validated
implementation may determine type or const inputs from its header before numeric
fallback; an associated result cannot choose between competing headers. Watch
only the query's dependency leaves, coalesce queued notifications, and cancel
completed queries. A change notification is neither canonical readiness nor proof.
Keep canonical notifications in dependency-completion order. Deliver partial
notifications only after pending dependency updates, so a resolved input uses
the ordinary normalizer without a redundant partial query.
An entirely unknown Self cannot select a caller assumption or implementation;
wait for a known head or numeric domain before allocating query lifetime aliases
or opening its graph. A deferred attempt must not change source provenance.

Demand resolution may reuse scratch capacity within its inference body, but never
reuse resolved results across separate requests: equations and projection
publication can change the graph. Clear active markers and pending work after
failed traversals as well as successful ones. Bound retained hash-table capacity
relative to the current traversal before clearing it, so one large query cannot
make every later small query scan the same large allocation. Known-type queries
keep their allocation-free path.

Body calls normalize substituted `Copy`, `Clone`, `Sized` and type-outlives
predicate subjects through the
same deferred worklist before querying builtin properties. A property of the
selected result cannot choose its inputs or erase the original application's
formation and lifetime obligations. Composite subjects retain their own nested
scopes; ordinary known types and unresolved root variables need no traversal
allocation when they contain no present projection.

A full trait-proof retry can wait for input resolution when one candidate's
matched header preserves every input hole and its remaining premises are only
builtins. A single independent type parameter beneath already-known reference
or pointer heads, with no trait arguments, gives this certificate directly from
the checked header. Other layouts use an isolated probe: binding, domain
narrowing or merging original holes invalidates the certificate. Competing
headers, applicable caller assumptions and recursive trait premises retain
partial retries. The certificate only controls
scheduling; canonical selection must still prove the complete implementation.
Do not copy and probe a wide unchanged input graph after each leaf assignment.
Check scheduling before opening the full query lifetime environment, and do not
export a result graph merely to establish input preservation.
Symbolic result transport is a separate operation and may progress before that
complete proof; it must retain the original input and formation obligations.

Before exposing a symbolic result, count every compatible validated header,
including candidates whose equations would refine existing inputs. Result types
cannot remove competing headers. When more than one header is compatible, each
candidate's trait conditions are selected in its probe, and a candidate is
discarded when the ordinary selector finds one of them absent. This mirrors
rustc's winnowing, which discards a candidate whose nested obligation fails:
`impl<I: Iterator> IntoIterator for I` competes with no impl for `&[?E; 0]`,
since `&[?E; 0]: Iterator` has no candidate. A deferred, pending, proven or
refuted condition keeps its candidate. A candidate without trait conditions
is never discarded, so the scan exposes nothing at the second compatible
unconditional header, before instantiating the rest; a family of such impls,
such as every integer `Shl` for `?int << ?int`, costs no instantiation per
further impl. See the
[blanket candidate winnowing checkpoint](docs/pa31-blanket-candidate-winnowing.md).
For a unique compatible header, instantiate
its associated definition with the same type/const bindings as its header and
export only an input-preserving graph. Scoped variable equations intersect numeric
domains in either argument order. They must not reject a numeric hole merely
because the implementation parameter is still unconstrained.
For a sole indexed candidate with one independent type parameter, exact known
reference/pointer heads can supply that parameter and its lifetime captures
directly. Repeated captures must agree exactly, and every capture must be supplied;
other cases retain the isolated probe. This shared header match supplies no premise
proof. Result substitution reuses the original input term and shifts captured
lifetimes beneath any result binders without copying the unchanged input graph.

Publication keeps an independent application root sharing the original input
identities, while redirecting the result root and waking its blocked equations.
The original dependency ticket and proof scope survive publication. Completing
the result does not cancel the input query: ordinary selection must still prove
its implementation and formation requirements, and its final normal form must
agree with the published result. Reserve and retain header lifetime equations
before exported types refer to their private identities. Premises remain pending,
even when the result erases all type or const inputs. This does not impose a
global requirement to solve unused const holes: ordinary aliases may erase them.

Partial alias applications use their retained unreduced definition so declaration
normalization cannot erase source-time inference or coercion context. An unresolved
projected expectation supplies no reference, array-unsizing or callable hint.
Ordinary result equalities may wait for normalization; implicit conversions and
reborrows cannot be introduced retrospectively when later inputs become known.

Freshening a stored function item reconstructs its signature from the immutable
callable template. Register newly constructed projection occurrences even when
no new lifetime was allocated; an earlier occurrence's proof does not authorize
them. Retain this work as a typed result of freshening, and register the whole
result in its enclosing scope so nested callable binders remain intact. Detect
projected templates and arguments during the existing traversal. Ordinary
callables must not allocate a normalization worklist merely because their
signatures were rebuilt. New normalization duties never discharge the original
parent application, implementation premises or caller lifetime obligations.

When a header requires a nested projection's normal form, register the nested
query at its actual callable scope and subscribe the enclosing application to
that result. Publication wakes only those dependents; it does not discharge the
nested query's original input proof. Recheck an enclosing application's current
inputs after that wakeup: an inner result may have erased holes still retained
by the inner proof. Cache input callable scopes by parent and inference-term
identity so retries preserve empty frames and never allocate replacement binders.

Reuse completed canonical body normal forms within the body's immutable caller
environment when the root has no ambient or free lifetimes and selection leaves
no lifetime obligations. Cache only after the complete selector succeeds; partial
results and failures supply no entry. Queries with scoped evidence retain their
ordinary proof path and private region intervals. Ordinary types bypass the
normalization worklist.

Run partial selection in an isolated input graph. Share one opening map with the
caller environment shifted beneath the actual ancestor binders. Restore the
selected input predicate before committing equations to the body: ambient slots
are rigid, nested callable slots may alpha-rename, and an outer inference hole
cannot capture a younger universal. Retain all selection requirements at the
original scope before installing the normalized result. If another query has
already normalized the same root, cancel its redundant notification without
selecting or publishing the evidence a second time.

Cache the body's projection environment with caller universals expressed as their
existing body aliases. Declaration predicates and their aliased copies describe
the same permissions; they must not become competing assumptions after opening.
Use only the established universal-to-alias mapping here, not arbitrary pending
region equations. Keep distinct caller lifetimes distinct and retain the ordinary
lifetime proof of any equality required by a selected implementation.

Opening a fully determined projected callable can introduce caller aliases
without a generic call or pending projection. Record that event in the body's
cleared call workspace and compare shape-equation operands in that same
representation, reusing the existing substitution cache. Ordinary nonprojected
calls need no new work. Otherwise an opaque projected argument may retain
bound slots while its callable parameter uses equivalent aliases, making valid
code depend on an unrelated earlier generic call. Preserve the written return
contract and ordinary lifetime proof; this alignment grants no equality between
different projection inputs or unrelated lifetimes.

Check constraining lifetime positions before erasing projections through
normalization. A lifetime occurring only inside a projected function-pointer
input does not constrain the pointer's output, even when a selected impl would
normalize that input to a reference using the lifetime. Preserve the unreduced
typed signature for this check, including projections introduced through aliases;
normal forms alone cannot establish this declaration constraint. Opaque input
positions still require valid binder slots.

Use original occurrences when detecting unconstrained impl lifetimes and when
classifying a function lifetime as early or late: projected inputs are
nonconstraining and original output occurrences remain significant after
normalization. Late instantiation then supplies the slots actually used by the
normalized executable signature. These are distinct queries over retained facts.

Retain unreduced annotation, alias and argument identities in sparse records
allocated only when projections occur; ordinary node facts keep their normal
types. Propagate those identities through completed typed children and cached
parameter/lifetime substitutions. Default arguments use their own append-only
unreduced prefix, so a preceding projected argument is not silently replaced by
its normal form in later defaults. Alias parameter-occurrence checks consume the
unreduced definition. Executable shapes, inference and lowering continue to use
normal forms, without reversing projected results into argument equations.

Built-in semantic traits retain their own definition identities; a user item
with the same spelling cannot acquire language behavior. Resolve aliases and
qualified paths through ordinary typed namespace facts. Demand external metadata
only when used, keep it separate from the crate root, and retain configured
prelude policy per module so lookup does not rescan attributes.

Trait member access is inherited in the resolver's owning scope, including
members installed by expansion. A macro invocation that shares its enclosing
scope must not reset that scope's associated-item metadata. Nested method-body
items retain their ordinary access. Anonymous type imports retain the final
binding by import identity, with scope and hygiene available through the existing
import records. Reprocessing an import withdraws its old anonymous target before
resolving the replacement; method lookup must consume these facts rather than
resolve source paths again.

Single imports preserve each available namespace at the narrower of its source
and requested visibility. At least one selected namespace must permit the
requested visibility; an unavailable name remains unavailable. A public type
can therefore retain an available crate-visible constructor or same-named
function through a public alias, without exporting that value publicly. Public
values similarly retain an available restricted type. Completed ordinary lookup
still checks each use. Consume the retained real source selection once, as in
the [namespace visibility checkpoint](docs/reexport-namespace-visibility-checkpoint.md).

Retain trait/impl owners and their direct associated declarations in one shared
inventory after name resolution. Keep owner, Self and member identities separate;
generic parameters and nested method-body items are not direct members. Preserve
resolver declaration order, including appended expansion definitions, independently
of hash-table iteration. Destructor discovery consumes this inventory rather
than rescanning all definitions. Receiver storage is shared by declaration checks,
body checking, scopes and CFG construction. Ownership alone is not evidence of
an elaborated signature, valid impl, selected method or executable body.

Elaborate each inherent impl target under its original type/const parameter
identities. Method environments inherit those identities and capture owner
lifetimes in a separate prefix before the method's own early and late slots.
Callable parameter lists validate an ordered declaration-owner path, including
foreign, repeated and reordered parameter rejection. Keep anonymous and unused
impl lifetime slots in the source proof environment.

Index inherent functions once by canonical nominal owner and resolved name after
signature elaboration. Partition non-root declaration contexts before shape or
applicability work; a use visits only its own context ancestry. Normalize direct
declaration identities separately for coherence. At a use, adjust against the
actual impl container and check the actual caller's visibility before selection.
Keep contextual indexes below the existing lazy lookup owner, preserving ordinary
record layouts and root-only allocation behavior. The
[inherent identity checkpoint](docs/pa31-inherent-member-contexts.md) records
structural work bounds, diagnostic compatibility and remaining trait consumers.
Qualified paths retain their resolved owner and typed
prefix; explicit type qualification uses the ordinary annotation worklist.
Selected methods enter ordinary function-item inference, template demand and
lowering. The full callable parameter order includes inherited impl arguments,
but only the method's own parameters consume its turbofish. Instantiate owner
lifetime captures as fresh body regions, including unused captures, then equate
the instantiated impl target with the typed prefix. Check inherited predicates
as well as method predicates. A declaration or required trait signature alone
does not authorize executable dispatch.

Trait-qualified callable uses keep the trait prefix's arguments separate from the
method's arguments. Infer inherited Self and omitted arguments through the ordinary
callable graph; explicit prefixes and ordered defaults constrain those same slots.
Generic bodies retain the trait declaration as a symbolic callable and prove its
complete requirements without guessing an implementation for a rigid parameter.

A type argument can introduce projections into a signature whose template contains
none. Record those occurrences during the existing argument binder-shift traversal
and register the instantiated signature for scoped normalization. Unknown variables
retain their dependency identities; the traversal summary is scheduling information,
never proof that a projected result exists. Ordinary generic calls keep their path
without a second graph scan.

Fully qualified `<T as Trait>::method` paths consume the resolver's retained Self
annotation, trait identity and canonical member name. Select a direct method from
the trait's existing member scope; inherited supertrait members require their own
trait qualifier. Bind the written Self annotation into the same callable inference
graph, preserving annotation holes, aliases and lifetime scopes. The trait prefix
has type-path arity and default rules: omitted required arguments fail, and defaults
apply even without explicit arguments. Reuse ordinary argument preparation and
dispatch after this qualification; no additional source parsing or impl search is
needed for selecting the named declaration.

Implicit `T::method`, `Self::method` and `<T>::method` lookup also retains and
binds the written Self annotation. Applicable inherent methods take precedence;
otherwise probe methods from parameter bounds before methods from lexical trait
bindings. These probes consume only the prefix and caller environment. Arguments,
return expectations and the method's own bounds do not disambiguate the name.
An implicit trait prefix infers trait arguments without applying type-path defaults.

Build trait visibility from final resolver bindings, including anonymous imports
and the retained candidate of an ambiguous glob import. Deduplicate aliases by
trait identity. A trait impl also makes its implemented trait eligible throughout
its own lexical scope, including nested and generated methods/items. Consume the
resolved trait declaration once in the existing scope index. This is lexical
eligibility, not a named import, bound priority, fixed impl arguments or implicit
supertrait visibility. Ordinary applicability and ambiguity still govern calls.
Lazily index direct methods by name in each queried scope and
share inherited candidate lists across scopes that introduce no matching methods.
Index parameter-to-trait bounds only when a caller needs implicit lookup; preserve
the complete goals for applicability. Unimported homonyms must not multiply work
at each use. Bound indexing, scope traversal and candidate probes have explicit
work limits. Enum variants retain their existing lookup and constant-evaluation
paths before method fallback.

Before proving and lowering a concrete body, select its trait operands through the
validated impl registry. Map each declaration to its override or checked default
using the sorted member table. Keep the selected impl captures separate from the
method's own early lifetimes and type/const arguments. Normalize body universals
into fresh aliases for selection, retaining every equality, outlives requirement
and allocated variable in that body's proof graph.

Selected impl-method signatures can introduce projections after the original
trait operand was normalized. Normalize the selected function item before caching
or rewriting body operands, using the dispatcher's existing concrete proof and
region allocator. Preserve nested callable scopes and every formation obligation,
including those erased by the resulting type. A selected target with no projections
finishes directly; repeated uses share its checked target and retained evidence.

Replace actual function-item types and operand definitions throughout the concrete
CFG and its contract, including callable types inside generic arguments. A shared
type fold preserves nested binders and memoizes the type DAG. Promotion and demand
then consume these selected identities. Initializer selections also supply the
constant value reader's callable targets, so named constants and statics cannot
retain a declaration-only symbol. Caller-dependent dispatch caches belong to the
body proof scope; independently proved closed selections use the separate shared
certificate described above. Generic templates and their source facts remain
reusable by other instances.

Retain a selected function's public callable signature separately from the
specialized body's opened signature. Validate the original template and contract
before transporting their namespace, then retain the actual completed signature;
later caller premises can enlarge the proof universe without changing that
signature's binder. Source Copy decisions bind both exact signatures, the full
instance, original owner and immutable environment. Dispatch folds the owned
signature with its contract while preserving the selected public identity. These
are constant-size identity checks over retained typed facts; never rescan a body
or reconstruct a signature from its final universal count to establish pairing.

Validate unused inherent methods for overlap after completing owner signatures.
Use isolated existential header equations, preserving repeated type and const
parameters. Impl predicates constrain overlap; method predicates, visibility,
receiver spelling and lifetime restrictions do not disambiguate declarations.
Coherence asks whether a compatible instantiation can exist, separately from
whether a generic body may perform an operation. Local conditional derive
contracts constrain this possibility; absence of Copy/Clone on a foreign type
head is not itself a negative coherence fact.

Implicit absence additionally needs a local/fundamental trait and orphan-closed
inputs at every ordinary nested trait goal. The source-free canonical language
inventory has a pinned compiler-owned implementation world; recognize it only
through its actual schema and lack of a source/imported owner. Actual source
and imported traits retain ordinary orphan rules. A nonfundamental nominal root stays
upstream regardless of its unknown arguments: inspect that root once, rather
than materializing and recursively walking every generic subtree. References
retain their actual referent locality; unknown/fundamental/open roots remain
conservative. A matching impl may depend on an open nested trait argument that
a downstream crate can make local. Reuse the existing Deferred answer so such
absence cannot be cached or propagated as disjointness. Explicit negative proof
and cycle cutoffs retain their separate rules. No Receiver-specific exception,
name waiver or extra permission/cache table is involved.

Share each owner-pair proof across method names. Partition closed targets by
canonical lifetime-erased shape, including callable binder erasure, before
probing possible pairs. These erased shapes are index keys only. They must never
replace source proof types or executable lifetimes. Keep deterministic diagnostic
order independent of hash iteration and bound inference, folding and pair work.

Qualified method selection consumes only the typed prefix and caller environment.
Surrounding call arguments, result expectations and method turbofish must not break
an ambiguous prefix lookup. Filter inaccessible candidates and unsatisfied impl
predicates before selection; check the selected method's own requirements afterward.
Caller parameters remain rigid. Unknown prefix arguments may remain possible,
but they do not acquire permissions in the completed body.

Partial annotations, inferred aliases and ordered defaults use one inference
implementation shared by bodies and prefix queries. Import only the reachable
type/const graph into each isolated candidate context, preserving variable
equivalence classes and domains. Failed equations never constrain the source
context. Built-in candidate properties consume actual caller assumptions and
current implementations, separately from compatible-world coherence queries.
An associated projection with unresolved inputs remains possible during this
filter. Inspect only its reachable inference graph, charging every resolution
visit; unresolved inputs grant no builtin or trait permission. Resolved
projections retain the named normalization boundary until that consumer can
normalize them under the caller's actual premises. Resource or graph failures
remain errors. The selected callable independently proves its complete bounds.
After selection, ordinary body inference commits owner arguments and performs
the complete ownership, lifetime and lowering checks.

Reserve inherent member names and conservative nominal heads before elaborating
implementation headers. A declaration-time lookup waits for the matching
nominal/name owners and owners whose raw head is unknown. Index those dependencies
by name and nominal identity; do not scan unrelated implementations on each use.
Publish members only after their owner's complete environment is ready. Retain
the typed prefix across header work, and freeze source ordering separately from
the order in which dependencies complete. A resolved name or raw head is never
an applicability proof.

Type-qualified path applicability owns its inferred prefix, candidate iterator,
isolated trial equations and deferred predicate cursor. A pending trait header
resumes that exact search after the declaration worklist prepares the requested
owner. Completed candidates and proof ancestors must not be replayed. Preserve
inherent precedence, bound-before-lexical groups and same-group ambiguity.
Receiver lookup shares candidate binding and conditions, but remains synchronous.

Applicability proofs are disposable probes. Their immutable caller and prefix
predate the wait; resume fresh allocation beyond both the private watermark and
the current body-region frontier. Do not publish those trial regions or create a
body reservation for them. The selected ordinary operation must independently
prove every bound and lifetime requirement. This private-probe rule does not apply
to authorizing body or projection evidence, whose regions require reservations.
Lifetime-only applied prefixes use ordinary annotation argument substitution and
formation checks, including lifetime arity and caller outlives obligations.

Retain the candidate partition built for overlap checking instead of maintaining
a duplicate header table. Early lookup can build that same partition for a complete
nominal/name bucket before whole-crate overlap validation. Closed prefix lookup
consumes that partition; unknown prefixes use bounded probes within the nominal/name bucket. Reuse the same shape
folder across headers and prefixes, so repeated aliases do not repeatedly traverse
their type DAG. Property queries cache known and unknown subtrees only after a
probe's equations are fixed and under its immutable environment.

Impl applicability shares complete trait-goal equations with body checking. Cache
caller assumption indexes once per scope, normalizing body lifetime aliases before
indexing. Immutable caller clauses own their structural trait/subject index; repeated
projection queries share it while retaining separate inference, lifetime openings
and proofs. Charge construction once and each lookup separately. Extending or
substituting clauses invalidates derived indices, including parameter-trait lookup;
copy-on-write updates must preserve every existing caller's permissions. Coarse
subject keys never substitute for complete-goal matching or selected evidence.
A missing match for a rigid goal excludes a candidate; unresolved evidence
may remain possible until the selected call has all argument/result constraints.
Only proved evidence authorizes a body operation. Canonical nominal and trait
argument requirements use the same complete-goal matcher under rigid caller
parameters. A blocked opaque equation against one caller assumption does not
invalidate an independently equatable assumption. Continue probing alternatives
in isolated contexts; two equatable assumptions still defer selection. If no
assumption matches, retain the normalization requirement instead of reporting
absence or permitting an unrelated implementation. This does not normalize the
caller environment or infer a projection's inputs from its output. Unresolved
candidate inputs defer the query without committing any alternative. Prefix
applicability may remain possible until ordinary call argument/result constraints
settle; a completed body still needs a unique proof. Retrying the same goal after
inference must use its new inputs, not a cached provisional decision. Retain
inferred lifetime equalities with their
owning declaration or
body; prove both directions using the ordinary region engine and completed nominal
contracts. An outlives relation in one direction alone is not equality. Keep exact
assumption lookups cheap and build broader indexes only when needed. Drop coverage
folds the complete predicate into nominal parameter order. Map resulting lifetime
equations back to the original impl slots and prove them in the destructor coverage
graph, where target lifetimes are universal and extra impl lifetimes are existential.
Extra slots must not strengthen the nominal contract. Apply builtin property
queries after trait equations fix their inference inputs. Method-own bounds remain
checks on the selected callable. Prove disjoint target shapes before requiring trait
coherence; missing local implementations do not establish negative evidence.

Goal/header equations may contain inference variables on both sides. Share the
scoped callable relation with canonical assumption matching, preserving local
lifetime correlations while allowing alpha renaming and unused binder slots.
A variable on either side cannot capture a surrounding callable's local lifetime;
outer lifetimes are rebased when entering or leaving a callable. Equalities of
those outer lifetimes remain obligations, including explicit trait arguments.
Lifetime-erased header overlap equations establish possible compatibility only.

Body trait goals and inherent-method conditions consume the validated impl registry.
Reuse its complete closed-application and outer-head partitions; probe each possible
header in an isolated inference graph. Conditions are obligations under the caller's
environment, never additional caller assumptions. Traverse recursive trait requests
with an explicit worklist and a total work budget; an active cycle is not evidence.
Cache completed requests that allocate no new lifetime identities, and collect a
shared proof graph only once. Reusing evidence with new existential lifetimes requires
fresh rebasing before it can enter a different proof.

Memoize a fully resolved failed trait request only when its whole search completed
without an inductive cycle cutoff. A cutoff depends on the active proof stack;
neither it nor its failed ancestors is a reusable negative answer. Keep deferred
inference and exhausted-work errors separate from absence. The memo belongs to
one immutable caller environment, declaration/validated query mode and region
scope; its keys preserve complete trait arguments and binder identities. It must
not survive a change of assumptions or implementation availability.

Each candidate allocates lifetime captures from an isolated view of the caller's
next free identity. Only a selected proof publishes those variables, equalities,
outlives and type-outlives obligations into the ordinary region checker. Retain the
selected impl owner and its typed arguments at the body use. Failed alternatives
must not publish variables or constrain the caller; possible applicability alone
still cannot authorize an operation in a completed body.

Prove canonical universally quantified obligations in an isolated selection context.
Open their whole predicate with fresh private aliases, equate those aliases to local
universals inside a retained proof scope, and graft every selected implementation
witness into that scope. Enclosing captures remain in their ancestor universe.
Structural selection success still requires the final directed lifetime proof;
static-only or correlated implementations cannot grant a stronger universal bound.
Builtin root evidence has its own provenance and cannot stand for a trait method.

Repeated quantified obligations in one declaration state may share the proof already
retained there. Key this reuse by the complete substituted clause and selection
mode; the caller environment and outer substitution stay immutable. Retain even
unsatisfied pending lifetime debts, and never cache absent, deferred or exhausted
selection as successful evidence. Ordinary states allocate no quantified cache.
This canonical route cannot export a child's inferred type/const binding to a
caller hole, flatten scoped alias Sized requirements, or substitute for instantiating
a universal caller assumption.

Dot calls retain the resolved receiver, canonical method name, lexical scope and
written arguments. Type the receiver before selection. For each built-in reference
dereference level, probe the value, shared borrow and mutable borrow in that order.
Reuse the inherent candidate partition and isolated applicability equations;
discard impossible receiver depths before importing an inference graph. Receiver
mutability, moves and loan legality remain obligations of the selected call.

At each candidate receiver, inherent methods precede parameter-bound methods,
which precede lexical trait methods. Bound priority constrains Self to the parameter
that supplies the bound. Retain that origin for both dot calls and implicit type
paths. The selected callable's exact implicit `Self: Trait<parameters>` predicate
must use caller assumptions; matching only the trait name could incorrectly
replace its arguments with an unrelated blanket implementation. Multiple stated
applications can still be resolved by ordinary argument inference. Other predicates
of the selected method keep normal implementation proof, and an explicitly written
trait qualifier keeps normal trait selection. The same trait declaration may remain
a lexical candidate for another Self, such as a reference to that parameter. Peel the retained receiver
reference shape and consult existing assumption/implementation head indexes before
copying a probe graph. These filters can exclude impossible candidates; positive
matches still require complete equations and validated implementation evidence.
Apply this subject-head filter only when the peeled declared receiver is the
actual owner's `Self`. A wrapped receiver such as `Pin<&mut Self>` has a different
outer nominal; bind its real receiver signature through the existing applicability
probe before testing the implementing Self's complete trait obligations. Do not
query the wrapper's implementation bucket as though it were Self, or copy and
bind the same signature in a second probe just to recover the inner type.
Integer and floating-point receiver variables keep their numeric domains and the
ordinary inference/fallback rules. A completely unknown Self is not guessed.

After the final built-in dereference, consider an array's slice type and its shared
and mutable borrows. Keep the actual place and borrow typed as an array; publish
the array-to-slice conversion explicitly for the region proof and body lowering.
Trait Self is inferred through the selected receiver signature, including when
Self is a reference. Only inherent calls bind the nominal impl target separately.
The hidden receiver argument belongs to every typed method-call node, regardless
of whether it has inherent-owner metadata.

Prepare written method type annotations and constant dependencies in the ordinary
declaration worklist without selecting a method. After selection, validate kinds
and arity against the method's own parameters, then evaluate constant arguments
through the shared capture and scalar-constant machinery. Hidden callable values
must satisfy the same nominal and well-formedness requirements as ordinary callee
expressions. Callable templates and complete owner arguments remain shared with
qualified function-item uses.

Retain selected callable identity, dereference count, borrow type and receiver
parameter type as semantic facts. Validate the hidden callable and actual/expected
receiver value types through the ordinary body path, including field preparation
for generic lifetime instances without their own expression node. Body construction
evaluates the receiver place once, before explicit arguments, and feeds its adjusted
value as the first ordinary
call argument. Implicit mutable receiver borrows use the existing two-phase loan
model; explicit source borrows retain their own earlier loan events. Borrow regions
keep expression provenance, and ordinary ownership, destruction, instance demand
and LowIR lowering consume the resulting body. Do not synthesize or reparse an AST
call to implement receiver adjustments.

An alias used as a value prefix with no written type/const arguments infers all
those arguments, including slots with defaults; explicit type qualification
retains ordinary type-annotation arity and default rules. Store this distinction
in the shared partial-annotation facts, not synthetic syntax. Enum variants take
precedence over same-named inherent functions for associated paths. Alias and
explicit type prefixes reuse the enum's resolved bindings and constructor facts.

Receiver syntax binds the whole value or reference. A reference directly to Self
supplies the output-elision lifetime; nested reference receivers use ordinary
input elision. Explicit receivers retain their terminal type and prove equality
with Self under declared and implied lifetime assumptions. Canonically identical
targets take the direct path. Other targets use the shared directed type-relation
engine in `types`, also consumed by body lifetime checking. Method bodies pass the
ordinary ownership/region pipeline even when no call demands their code.

Trait Self is an implicit possibly-unsized type parameter with its own owner.
Explicit trait parameters retain their source indices; method environments inherit
both owners. Required method signatures check well-formedness while permitting
unsized parameters and results. Default bodies require ordinary sizedness and
ownership proofs under their declared bounds. Required signatures never create
CFG templates or executable symbols. Validate owner bounds and closed parameter
defaults even when a trait or impl has no methods; unchecked trait implementations
must not pass merely because required methods have no bodies.

User trait applications have canonical declaration and argument identities, with
Self retained separately as the predicate subject. Preserve every type, const and
lifetime argument through inference, substitution, parameter-use queries and body
universal aliases. Arguments are not synthetic nominal types. Complete predicates
participate in closed/default checks even when their subject alone is concrete.

Prepare trait headers before dependent bounds. Share ordinary path argument and
default elaboration, then instantiate each trait contract through one cached,
capture-safe mapping of Self and its explicit parameters. Expand only requirements
on Self into caller assumptions; parameter and composite-subject bounds require
separate evidence. Retain the decoded subject classification in each completed
header: an alias that reduces to Self remains a caller requirement, even though
its canonical type is identical. Result-rule closure and builtin/lifetime promise
queries must use that same distinction. Diagnose supertrait cycles by declaration
identity before expansion, and deduplicate diamond paths under a bounded worklist. Arbitrary trait
names confer no builtin Copy, Clone or Sized behavior.

Trait impl headers share generic/lifetime elaboration with inherent and builtin
Drop owners. Retain the complete trait predicate, owner and source once. Every
owned type/const parameter must constrain Self, the trait application, or an
associated result whose complete inputs are already determined. An ordinary
where bound alone does not constrain it. Validate target and argument well-formedness,
trait requirements and safety under the impl's own environment. Input types imply
lifetime well-formedness facts, never new Copy or user-trait assumptions.

Argument-position `impl Trait` introduces a distinct unnameable type parameter
for each input occurrence. Resolution retains its function owner and source but
adds no lexical binding or hygiene entry. Elaborate its written trait bounds with
ordinary generic declarations, inference, calls and lifetime contracts. Require
at least one trait bound and reject precise `use<...>` captures in input position.
Anonymous parameters follow written parameters in typed metadata, remain separate
when two inputs have the same bounds, and never consume a turbofish argument.
Method conformance checks anonymous versus named parameter identity as well as
arity and requirements; replacing one spelling with the other is not transparent.
Nested callable types do not introduce input parameters for their enclosing item.

Return-position opaque declarations keep a canonical identity and complete
captures separate from private defining-body results. Every return path shares
the same hidden inference term. Keep the public callable signature intact when
building its CFG: the CFG result uses the solved private result, including
nested opaque occurrences and early returns. Caller inference, equality and
trait permissions consume published promises, never storage representations.

Implicit return captures follow the edition of the parsed `impl` introducer's
exact token origin. Rust 2021 free and inherent functions capture only lifetimes
written in the opaque bounds; trait declarations and trait implementations keep
automatic captures in both supported editions. Type and const parameters remain
captured, and an explicit `use<...>` list overrides implicit selection. A where
clause does not add a written opaque-bound capture. Collect resolved lifetime
identities during the existing bound traversal, before alias normalization can
erase them; intersect bound-local identities with the defining signature's slots.
Do not reparse source, walk the entire crate or infer captures from normalized
trait arguments. The legacy inventory belongs to the pending opaque declaration,
not every type, owner or CFG node. Token origin here differs from never-type
fallback, whose rule belongs to the containing compilation unit.

Resolve precise capture lists to existing parameter identities and retain
declaration order. Require every type/const parameter, reject anonymous input
parameters, duplicates and misplaced lifetimes, and require all lifetimes used
by published bounds. Preserve actual captures in caller liveness; an erased
representation is never evidence that captured inputs can be forgotten. Published outlives promises use the shared associated/opaque result closure:
a unique declared bound constrains the result without inventing a reverse bound
on its captures. Locally quantified targets remain scoped and do not become static.
Trait precise capture lists retain every parent trait parameter, including lifetimes
and implicit Self.

Retain unresolved hidden lifetime occurrences with the solved private result;
never publish them in a portable representation. Build opaque defining bodies
through the same scoped CFG/template path even without type/const parameters.
Register member obligations while consuming the existing typed obligation stream,
after ordinary variables and quantified scopes have their identities. Promotion
and borrow-place provenance precede solving. Choose only captured root universals
or static, and retain the selected equalities as obligations in the final proof.
Publish portable hidden types only after the defining region and loan checks.
Restoration caches belong to one capture list: equivalent names in different
precise lists do not grant permission to use each other's spelling.

Coordinate an early defining proof above the semantic, CFG and borrow phases;
none of those phases may depend on the coordinator or executable lowering.
Retain the original defining CFG for the later template pipeline. Incremental
lifetime collection consumes each appended semantic fact once, orders a body's
variables before its obligations and seals completed contracts against late
additions. Ordinary programs keep the direct collection path. The final template
and instance gates still verify the complete program before execution.

Promotion precedes an early defining proof, including promotion of returned
references. Share its report-owned constant index and cleared evaluator storage
across demanded bodies and the final template pass. Index only new declarations
and initializers; a chain of defining dependencies must not rebuild an index of
the whole crate for every function. Do not retain active queries or a previous
body's permissions in that cache. Count actual preparation, consumed lifetime
facts and CFG builds separately, including growth in unrelated declarations.

Share captured lists, candidate sets and assumption reachability within each
body. Required SCCs determine dependency order before making a conservative
choice. Match the pinned compiler's least candidate comparable with all remaining
choices; incomparable alternatives can therefore select static. Topological
candidate order and shared reverse reachability avoid repeatedly searching a
long assumption chain. Retain deterministic source identities, bounded work and
the ordinary graph's allocation-free absence of member state. SCC ranks are
requested only by member inference. No inference choice adds a universal
assumption or replaces the later storage/loan checks.

Auto-trait queries may demand a defining body through an opaque input. Retain
that body's solved finalization frontier, canonical proof query, completed-proof
cache and reserved private lifetime intervals across the dependency. Do not replay
validators or publish the hidden representation while its canonical obligations
remain pending. Quantified requirements preserve their scope during resumption;
opaque reveal is a separate dependency edge, not structural coinduction. Keep
ordinary dense body scratch until an actual suspension, then retain only that
owner's touched nodes and bindings. Observe canonical proof work separately from
ordinary inference and bound retained suspended storage on growing dependencies.

Completed child bodies also own deferred constant-pattern work. Preserve those
batches independently of the caller stack, then combine completed batches once
in declaration order. An unfinished parent's request suffix is not a child scope.
Restoring its state must neither discard child exhaustiveness checks nor repeatedly
copy descendants' batches through every ancestor.

Storage preparation visits appended canonical opaque applications once and
substitutes their complete captures. Cache flattened storage heads independently
of canonical kinds. Reuse them for concrete trait dispatch, layout and cleanup;
symbolic destruction retains possible cleanup without revealing hidden types.
Validate by-value finiteness through the existing template-layout machinery,
including unused and generic defining bodies. Function-item signatures are not
physical storage edges. No opaque application authorizes execution until the
ordinary template, instance, region, ownership and destruction proofs complete.

Trait-method opaque outputs participate in ordinary normalization as a typed alias,
with the complete capture application retained through proof scopes, header waits
and symbolic output inference. A proved implementation stores its selected written
result template, impl parent bindings and trait method lifetime coordinates.
Anonymous outputs and named families share selection and simultaneous substitution;
never invent synthetic definitions or infer a parent from a non-injective output.
A symbolic result may reveal structure before all method inputs settle, but the
original query still owes every input and implementation premise. Caller assumptions
keep generic outputs opaque; only independently selected concrete implementations
expose their declared refinement. Conformance uses the trait's original signature
for implied well-formedness, then proves the actual output's separate requirements.

Defining recursive equations keep the entire opaque capture application. Register
private hidden roots only in the actual defining body, never in selection probes.
Ordinary use of a recursive result may retain its public identity; equating it with
a defining hidden root or concrete result requires complete capture equality.
Those equalities remain lifetime proof obligations even for a shape comparison.
Retain explicit successful defining uses for private coercion proof substitution;
do not globally rewrite callable identities or reveal another capture application.
Only an otherwise unconstrained recursive defining root receives the pinned unit
fallback, after ordinary never-coercion handling. Infinite hidden storage still
fails the existing finiteness check. Ordinary inference contexts allocate no
opaque-defining table, and all additional traversal remains charged and iterative.

Callable signatures inside bound subjects and arguments keep their own lifetime
elision rules, while projection formation still waits for the complete written
caller environment. Traverse those signatures when retaining raw bound roots;
skipping lifetime-elision validation must not skip their deferred projections.

Associated result bounds retain their typed projection roots until all sibling
result contracts exist. Validate those roots before the enclosing predicate,
using their original binder and written source location. Family equality bindings
resolve their own type, const and lifetime arguments through the ordinary argument
worklist. Publish a bound environment only after every binding argument completes;
resume the entire pending batch once, rather than rescanning it for each binding.
Retain the complete original family selector as a formation root independently of
its equality result: an equality that reduces an output to `u8` cannot erase the
family's input outlives, trait or projection requirements.

Prepare raw typed impl bounds before normalizing their projected annotations, as
for function bounds. An opaque projection can finish this declaration pass after
its formation proof; retaining the projection must not reschedule the completed
bound environment. Structural shape checking requires its exact Self and trait
application in the immutable environment or a separately retained normalization
proof for the complete root. Keep those structural query caches separate: a
cached shape cannot authorize a raw query without parent evidence. A complete
shape under the caller's exact assumptions finishes directly; only a missing
proof or an insufficient structural property demands scoped normalization. Check
every type input, and use the associated schema, completed conditional declaration
promises or an explicit Sized assumption for the result.
Projection input well-formedness retains the surrounding callable depth. Input
lifetime requirements are not the opaque result's outlives or live-value
components; result properties still need their own proofs.

Rigid method shape comparison distinguishes a completed type mismatch from an
invalid identity or exhausted work. Publish a cached positive pair only after
all its children complete. A mismatch or error discards pending work while
retaining completed subgraphs; repeated small failures must not scan or clear
the capacity of a large historical cache. Finish markers reuse work-record
padding and keep the iterative traversal proportional to the visited DAG.

Associated result contracts do not expand recursively into caller environments.
Close ordinary Self supertraits first, then their immediate owned result clauses
and those clauses' ordinary implications. Retain their finite equality, builtin
and lifetime facts, while user-trait result applications use demand proofs instead
of additional caller assumptions. Further associated results are queried at the
finite projection demanded by the program. Recursive declarations such as
`trait A { type Out: A; }` and their equivalent owned header clauses must terminate
without raising work limits or withholding promises at deeper demanded results.

Bindings on an owned result, such as `Self::Out: Other<Value = X>`, belong to
that result contract, including bindings completed after header member lookup.
Retain the complete projected equality subject and both sets of trait arguments.
Index direct result equalities and ordinary Self-supertrait equalities by member;
map a demanded candidate set once per complete result application. A cached rule
is declaration data, not permission: retain the parent proof before applying it.
Select that bucket from the outer projection member before resolving or
materializing its input graph. An absent bucket needs no input traversal,
substitution or allocated empty rule list. Metadata lookup leaves unresolved
inputs and normalization permissions untouched; later publication can change
which outer member, if any, the term denotes.

Equality candidates match complete parent inputs in isolated probes. Identical
rules from explicit and derived evidence are harmless; conflicting complete
matches remain ambiguous. A blocked equation involving an opaque input supplies
no input equality and must not hide a different exact match. Without a complete
match, keep the projection opaque; this is not negative trait evidence. Never
infer equality of Self or its arguments from equality of associated outputs.

Projected arguments in associated member bounds retain their written identities
until the complete result contracts exist. Reuse the header formation worklist
and original binder to discharge those requirements afterward. Keep this proof
separate from result equations, including unused declarations and recursively
related results; an equation cannot manufacture a missing parent implementation.

Keep immutable canonical projection schemas separate from properties learned
while typing complete headers. Share the completed conditional Copy/Clone/Sized
table across immutable environments; it supplies no parent formation evidence.
Both declaration and body property checks retain their scoped normalization debts
before using a derived opaque result's structural properties.

Decode member-written and owned-header result outlives bounds through the ordinary
lifetime binder. Close only the result's ordinary Self supertraits when preparing
its finite lifetime templates. A scoped outlives query maps the declared regions
through the actual parent arguments and adds verification alternatives for that
opaque result. These alternatives never become bounds on Self or trait inputs.
Concrete normal forms retain their independent formation debts, so a conditional
implementation cannot use its output promise to prove its own prerequisites.
Unused implementations must establish every promised result lifetime under their
actual bounds; unconstrained impl lifetimes remain rejected before normalization
can erase their occurrences. Preserve the separate sufficient proof through live
input components and all existing projection-identity and binder-scope guards.

Determine impl parameters from unreduced typed occurrences. Seed injective
positions in Self and the trait application. An associated equality contributes
a directed dependency from every type, const and lifetime input to the injective
positions of its result; projected result inputs are non-injective and cannot be
inferred backwards. Exclude an equality on the implementation's own complete
trait predicate. A worklist wakes only dependents of newly determined parameters,
so declaration order is irrelevant and unseeded cycles remain unconstrained.
Use the same analysis for inherent and trait implementations, after completing
their bound environments. Skip dependency-graph construction when the header
already constrains every owned type/const parameter. This declaration check
determines parameter identity; actual selection must still prove every equality
and carry its type and lifetime evidence through the ordinary body gates.

Keep prepared headers separate from validated implementations. Declaration proofs
may use prepared headers as conditional rules, through an explicit declaration-only
selection entry point. This permits closed bounds, forward supertraits and cyclic
declaration well-formedness without depending on source order. Every header still
passes coherence, contract and member validation before body analysis receives the
facts; every executable body is checked before lowering. An active cycle of impl
conditions remains insufficient evidence. Select using the original declared bounds,
not the additional supertrait assumptions derived from those bounds, which can
otherwise manufacture circular conditions. Preserve their prefix before elaboration.

Normalize a declaration's captured lifetime binder to fresh free aliases before
inference, including the same substitution in its immutable assumption environment.
Nested callable binders remain local. Reserve disjoint lifetime intervals for proofs
contributing to the same owner and after existing method-conformance existentials.
Retain alias equations, selected impl equations, outlives and type-outlives obligations
together. Closed bounds use no caller assumptions and cannot justify themselves.
Fields, parameter defaults, signatures and initializer types all retain their proofs.
Drop coverage maps the complete proof back from nominal to impl parameter coordinates,
while unused impl lifetimes remain existential. Body checks that allocate aliases
after inference completion publish their equations under that same body's identity.

Member-bearing implementations require matching associated declarations and signature
conformance. Retain overrides separately
from default declaration identities; executable demand must dispatch through the
selected implementation. Index overlap candidates by trait and
rigid subject, then by complete lifetime-erased application for closed headers.
Share existential bindings across Self and every type/const argument when probing
compatibility. Lifetime erasure may establish structural disjointness, never body
permissions. Charge candidate-pair storage before allocation and equation work
during probes; predicate-sensitive coherence remains a separate proof.
Instantiate and equate the actual implementation heads before materializing their
conditional assumption graphs. A structural mismatch needs no unrelated trait
condition expansion. Compatible heads retain their own type/const substitution
and lifetime mapping, then pay and prove the complete conditions through the
existing coherence kernels. Keep reference and iteration queries on their full
condition path. Scaling controls must exercise both disjoint heads and compatible
heads with growing conditions, alongside real/open overlap and failed payment.

Before instantiating either header, ask whether the two trait references could
unify at all, as rustc's `overlapping_impls` asks `args_may_unify`: Self, then
each argument, eight constructor levels deep. Parameters of both impls,
projections, opaque types and lifetimes may unify with anything. Rigid
constructors compare kind, mutability, array length, tuple and function arity,
function safety and ABI, principal trait and nominal identity. A rejected pair
pays one step per compared pair of nodes and no instantiation; a difference
below the depth, behind an alias or under a parameter's projection reaches the
complete equation. A reserved condition costs one step in the pair's overlap
ledger and, for a source crate, one in the source-header ledger, whose byte
ledger alone pays the bytes of newly reserved conditions on regrowth. The
[deep reject controls](pa31/tests/spec/deep-reject/README.md) record independent
outcomes and audit every coherence and specialization control.

Trait application index and cycle keys share one lazy structural fold owned by
the canonical type store. Its result is an opaque comparable key, never a usable
predicate or lifetime proof. Erasure has no lexical context: shared callable
tails use the same structural cache entry at every binder depth. Ordinary
substitution still distinguishes those depths. The fold retains its cumulative
one-million-step resource limit, bounding derived index growth; requesting
queries also charge the incremental transformation work. Restore the cache on
failure. Candidate availability, active cycles and completed selection answers
remain separate caller/query facts and cannot be cached in the type store.

Method conformance checks unused implementations too. Adjust each implementation
identifier against the actual trait container, then match its exact namespace,
normalized context and member kind in the resolver's retained member table.
Reject two implementation declarations mapping to the same canonical trait
member, even when their original identifier contexts differ. Retain every actual
member in the inventory; a spelling-keyed overwrite must not hide a duplicate.
Require every nondefault method and reject extra members. The
trait method's type/const parameters are rigid proof inputs; using the impl's
parameter identities would assume a stronger implicit Sized requirement. Prove
the impl method's requirements under the impl owner and substituted trait method
contract, while checking its body under its own declared assumptions.

Associated types and constants consume the same identity rule before early
declaration queries. Publish canonical type-member links before a prepared impl
header becomes selectable; normalize through those links, not through spelling.
Constant readiness builds one identity map per actual impl and reuses it during
final conformance. These maps contain declaration identity only, never a signature,
body, lifetime or execution proof. Reuse the resolver's exact member table rather
than maintaining another parallel name/context index.

Construct that conformance environment from the actual impl's bounds and the
trait method's own predicates. The impl-header check already proves the containing
trait's requirements. Reintroducing those inherited templates as substituted
caller assumptions can hide concrete supertrait associated definitions. Preserve
explicit method predicates, even when they repeat a parent bound: generic and
lifetime-dependent applications can remain opaque under those predicates.

Validate associated definitions before using their normal forms in method
conformance. Keep a local index of those checked definitions, shared by the impl's
methods. A local result applies only to its exact declaring item, Self and complete
trait application, shifted into the request's callable scope. This index does not
publish a validated implementation; all method checks still precede that publication.

Normalize each component of the already-opened conformance signature without
introducing another outer function binder. Share completed normalization work
within one fixed environment, proof scope, implementation availability and local
definition provider. Retain nested binder mappings and every ordinary selection
obligation in that session's proof. Repeated parameters must not rewalk the same
captured impl receiver graph. Body callable instantiation also schedules projection
normalization when the signature itself contains projections, including when this
is the first projection work in the body.

Match early lifetime arguments positionally. Instantiate impl late lifetimes
existentially against universal trait late lifetimes; unused late declarations
do not alter the contract. Share capture-safe substitution, shape equality and
directed lifetime relations. Preserve nested callable binders. Trait input
well-formedness supplies implied lifetime facts for contravariant parameter and
covariant result checks; structural types, receiver presence and safety still
must agree. This permits weaker input lifetimes and stronger results without
granting unproved bounds to the implementation body.

Trait argument well-formedness and declared requirements are checked even for
unused declarations. Lifetime requirements consume the completed nominal contracts
and shared directed relation engine. Assumptions from the original signature and
explicit bounds remain distinct from obligations imposed by a trait application.
Predicate inference shares the ordinary type/const equivalence classes, and
completion rejects unresolved arguments that appear only in an obligation.

Inferred associated applications retain Self and every trait/family input in
the type graph, including shared const variables and the argument-kind boundary.
Parameter substitution, binder mapping and isolated imports preserve these
identities. A projection remains a noninjective operation: its result cannot
equate its inputs or inherit their layout, Copy or lifetime properties.
Freshening its inputs is not a substitute for freshening a selected result.

A non-refining query may transport symbolic result structure back only
while preserving every original type and const hole. Refuse input bindings,
domain narrowing, merged distinct inputs and escaping fresh query holes before
allocating destination terms. Reuse original variable identities through one
shared reverse mapping, with bounded traversal of shared structure. Lifetime
equations and semantic proofs stay with the caller; transporting a result does
not authorize implementation selection or discharge a formation requirement.

Selected input refinements use a separate transport; do not relax non-refining
probes. Import only reachable input holes through one mixed-root mapping.
Record original holes during that same import traversal. Do not rediscover them
with a second graph walk, or allocate retained mappings and wakeup tables for
queries that have no holes or deferred conditions. Initialize owned query contexts
in place instead of returning and copying large inference state between factories.
Reconcile current caller constraints on resumption and before commitment, keeping
partial structures, numeric domains and shared type/const equivalence classes.
Close exported lifetime identities and validate the combined graph before
allocating or binding caller terms. Reuse unchanged caller holes. Retain query
lifetime equations in the selected proof's scope; new equations between closed
caller types belong to the actual caller. This transport supplies no selection
or lifetime proof authority.

Ordinary body selection may publish a unique partial candidate's input structure
while retaining its complete candidate, unresolved conditions and lifetime debts.
Try all compatible headers before publication, including pending alternatives;
never guess an unknown Self from the registry. Keep retained children in a flat
arena on the iterative search, with stable slots across resumption. Reserve private
lifetime allocations before yielding to body or declaration work, and preserve
foreign allocation gaps. This root-scope transport does not export a witness from
a universal proof scope or authorize higher-ranked source syntax.

Quantified call requirements retain the complete predicate and its binder arity,
including an empty binder. Substitute owner captures before instantiating shared
type/const inputs; open the requirement only inside its isolated selection query.
Ordinary children in that same scope may refine its inputs. A nested quantified
implementation condition still resolves its parent first and cannot publish a
local witness into the parent. Keep the ordinary call metadata unallocated.

Export only private lifetimes actually reached by a selected caller input. Give
each one a stable fresh witness in the older caller scope and retain its equality
to the private lifetime in that lifetime's original proof scope. Unused private
witnesses remain free to depend on universal slots. Direct universal aliases and
unowned identities cannot be exported. Numeric allocation order does not determine
universe ownership; caller witnesses can be allocated after private query IDs.

Retain the export fold across suspension. Append only newly allocated source
intervals, preserving caller gaps and omitted universal aliases; never reclassify
an earlier interval beneath cached canonical substitutions. Map shared inferred
roots together anew after equations change. Index private proof ownership by
ordered reservation intervals and append only newly created bridge equalities.
The completed-root path matches the opened predicate inside its isolated scope
before closing caller inputs, just as partial publication does. Register and prove
the complete scoped evidence before execution; publishing a type is not proof.

Observe deferred conditions and body goals through their own type/const dependency
leaves. A condition consumes its current inputs, including progress it just
published; that publication must not immediately retry the same child at every
ancestor. Route shared body notifications to their original projection or trait
owner. Retry only affected goals, retain exact header continuations, and default
numeric variables only when the ready queues are empty. Cancel completed watches.
A readiness event remains separate from complete trait and lifetime proof.

Reconcile shared input equations as one batch. For graphs without callables,
share structural pair matching and check the final graph for cycles once across
all roots. Temporary cycles stay within the equation operation, and a failed
context must be discarded. Callable inputs use strict scoped equality so local
slots cannot escape and alpha renaming adds no caller debt. Keep the ordinary
single-equation occurs checks unchanged. Share binder folds across exported
roots only while their input graph is stable; do not reuse results across later
equations. Charge query, validation and export work to the caller's budget.
Bulk pair memoization owns its temporary storage; do not leave later small
equations repeatedly clearing a bulk query's retained hash-table capacity.
A single unknown against an unexpanded canonical leaf may use ordinary scoped
equality directly. A canonical root against a composite pattern still requires
the joint path: cached descendants may have been expanded by earlier constraints,
making separate occurs checks repeatedly traverse the same shared prefixes.
If every pair instead has a fully determined, projection-free, non-callable
anchor, successful shared matching establishes the other side's finite shape.
That batch needs neither a separate callable scan nor a final cycle traversal.
Any unanchored pair retains both checks; callable anchors retain scoped equality.

Keep completed inference subtrees without projections as canonical identities
within their owning context. Materializing a known input preserves that identity.
Do not intern types ahead of ordinary demand: published semantic views retain their
type allocation order.
Repeated resolution, isolated imports and exact canonical input comparisons reuse
these identities rather than traversing the same shared graph for every bound.
Unresolved and projection-bearing parents remain uncached so later equations and
projection publication stay visible. Preserve the original graph and lifetime
equations; a structural identity carries no formation or caller permission.
Retained entries are bounded by the context's terms, with scalar identities derived
directly and composite cache storage allocated only when needed.
Existing postorder instantiation/import walks may also retain immutable structural
summaries before canonical resolution, without interning any type. Only graphs
without type/const holes or projections qualify. Region-free graphs retain their
roots across lifetime-only mapping; parameter/item substitution and shape erasure
still run normally. Isolated queries may publish canonical identities actually
demanded for imported region-free completed graphs. This publishes no lifetime or
trait permission, leaves original inference graphs intact, and occurs only after
successful input reconciliation. A failed query cannot publish those identities.
Within one call substitution, equivalent complete quantified declaration clauses
need one obligation. Deduplicate with hashed binder-comparison keys, including
every captured input and associated result, before building the inferred goals.
Retain the original declarations and the chosen clause's complete binder/proof;
ordinary requirements and obligations from different calls remain separate.

Canonical-input readiness uses dependency watches on unresolved type and const
leaves. Unions, bindings and explicit fallback wake only affected watches; newly
exposed compound terms add their own leaves without rescanning resolved ancestors.
Deliver each readiness event once and charge the shared inference work budget.
Watches retain no proof authority: the semantic caller still owns source identity,
binder scope, normalization, evidence and suspended equations. Collect dependencies
from every retained proof, normalization and Sized root, including erased inputs.
Wait only on the parameters those roots use: waiting on an unrelated output or a
dependent default can prevent an otherwise solvable query from making progress.
Schedule individual projected occurrences rather than whole output aggregates.
Traverse shared ordinary structure once per scope, preserve callable binder counts,
and leave a projection's input normalization to that query. An input also used
outside the projection has its own occurrence. Deduplicate an identical projected
term in the same scope without discarding its separate formation contracts.
Lifetime and parameter substitution must register the projected terms they create,
including projections supplied through arguments or defaults when the original
outer template contains none. A copied graph cannot inherit a finished ticket
from its original term.

Deferred equality and shape equations constrain ordinary siblings while retaining
blocked associated-result leaves. Never bind an output hole to a projection merely
to avoid suspension, and never infer projection inputs from an output equation.
Preserve the original relation kind and resume only awakened leaves; unchanged
siblings must not repeat work or lifetime equalities. Completion rejects remaining
equation or formation debt, including when a normalized result erased its inputs.

A proved opaque associated result has an inference-term identity distinct from
an unresolved projection operation. Retain each occurrence's original application
and scope even when results share a canonical type. Determined inputs remain
immutable; unresolved inputs retain their actual inference variables. Publishing
an unchanged opaque result must still wake suspended equations. Equal opaque
results never imply equations between distinct projection inputs. Region mapping
creates new formation obligations; value-lifetime freshening does not rewrite an
opaque operation's inputs as if they were exposed reference fields.

Concrete body instantiation normalizes substituted associated results before
layout, cleanup and lowering. Compare the substituted callable signature with
the selected signature after normalization, and retain implementation lifetime
debts in the instance contract. Map caller lifetimes using the same substitution
context as parameter arguments. The body's signature binder is its existing
universal scope; callable types nested inside it introduce their own binders.
Do not copy an existing proof's quantified ancestors to a sibling proof and reuse
its private variables there. Extensions of such scoped projection debts remain
rejected until their original scope and allocation ownership can be preserved.

Body annotations retain source callable scopes before descending into their typed
children. Alias-local binders extend that ambient prefix. Rebase proof-private
identities before substituting actual arguments, and preserve both prefixes when
aliasing caller universals. A fresh query copies only its ancestor chain, not every
previous sibling scope. Scoped proof publication preserves gaps in private region
intervals and the ownership of nested witnesses. Synchronous body normalization
reserves intervals in its exclusive local proof before publishing the contiguous
result once; it uses validated implementations and the existing lifetime gate.

Qualified local associated paths with inference holes retain resolved member
identities and separate subject, trait and member arguments. Reuse annotation
argument preparation and defaults; do not require a canonical subject before body
inference. A unique parent header may refine inputs while its premises remain
pending. Retain that exact search, its owned lifetime intervals and its enclosing
callable scopes until all premises finish. Exposing a selected result shape never
discharges the parent or the complete original application. Open a surrounding
scope stack through one shared graph traversal, preserving each binder's slot
identity and lowering older captures by the complete removed depth.

Declaration normalization can publish selected evidence before waiting for an
associated definition whose dependencies allocate regions in other bodies. Keep
the same body proof and query-to-binder map across those waits. Record foreign
allocation intervals explicitly; an enclosing numeric range does not own its
holes. Append only newly allocated evidence and exclude those holes when building
the lifetime graph. Traverse owned intervals and scoped blocks together once,
without rebuilding earlier binder prefixes at every dependency. Keep ordinary
proof storage unchanged until interleaving actually occurs. Caller substitution
preserves this ownership; streamed body evidence cannot enter the fresh declaration
template rebasing or grafting paths.

A body query's selected implementation may carry a quantified condition, whose
binders that selection opened below the query's own scope. Its streamed
evidence keeps them there, through an owned graft that never rebases the
evidence as a template. Map each of its scopes below the query scope, attach
its owned intervals and scoped blocks in allocation order and keep every
foreign interval excluded; a foreign interval that crosses an owned block is
an error. A streamed child without binders keeps the flat attachment above.

Projection and trait progress run before numeric fallback and body publication.
Normalization never substitutes for the original well-formedness proof. Isolated
method-prefix queries still reject pending projection proofs until that query mode
supports them; body proofs cannot leak into a speculative candidate. General
opaque-result properties and quantified premises in implementation search remain
PA21 responsibilities. Associated constants use the retained declaration, selected
initializer and complete-instance machinery described above.

Source From support remains required under LIB-105. Its language identity must
be separate from diagnostic identities: only an actual pending positive
From<float> obligation with Self resolved to f32 supplies the pinned f32 fallback
preference. Explicit types, other subjects, completed obligations and ordinary
or diagnostic-only homonyms retain their normal behavior. Reuse the numeric
dependency frontier and actual changed-root waiters; do not scan every pending
goal per call or introduce another numeric cache. Preserve projection barriers,
attempted-work payments, hot widths and unsupported metadata capabilities.
For a unique retained candidate, map live nested preferences through the actual
validated input-publication mappings. Completed proof nodes and rejected or
ambiguous candidates supply no hint. Keep traversal iterative and confined to
that selected subtree, preserving its scope and universal export witnesses.
The receiving body or projection owner cancels silent child tickets before
reselection and on completion. Such tickets cannot seed fallback themselves or
publish resolution notifications. Within one unsuccessful fallback sweep,
deduplicate inspected roots so shared mixed frontiers do not rescan the same
waiters quadratically; discard that temporary state before binding any root.
The [source From contract](docs/pa31-source-from-plan.md) records independently
qualified original-header, generic, alias, ordering and native controls. The
finite v9 source/fresh-client/native profile is qualified; wider imported and
original-library graphs remain required.

The explicit source `module-conversions-v9` capability must transport the actual
From role and numbered ordinary generic originals without enabling Provided
authority. Preserve complete defining type parameters and body-local lifetime
allocations through the existing cursor and recorded domain, demanding them
after declaration preparation instead of eagerly mapping every body region.
Construct the genuine defining Contract environment once and authenticate that
same owner and environment on readonly reconstruction. Header readiness is
distinct from body-environment readiness. Reuse existing maps and allocators;
do not add another per-body cache or grow common Body/Header/State records.
Closed primitive cast opcode7 carries the actual operand index and result type,
never a serialized cast proof. Revalidate both primitive types and derive the
operation with the normal semantic classifier. Old source/Provided profiles
retain their exact bytes and reject unsupported conversion operations. The
[conversion plan](docs/pa31-source-from-plan.md) records actual body, private alias
and fresh-client qualification and the remaining broader transport requirements.
Keep canonical numeric materialization separate from proof completion: advance a
paid monotone cursor through actual selected children without completing pending
Nodes or granting candidate authority. Imported bound consumers collect genuine
parent declaration clauses while preserving each header's own prefix and owner.
Choose that input protocol from the actual typed imported owner, rather than
crate linkage presence. Selected Provided consumers retain their authenticated
installed scope-input view; ordinary source headers require the paid parent
walk. Neither representation grants an implementation premise or executable
permission. The [profile repair](docs/pa29-call-input-profile-ci-repair.md) records
real full-family regressions, restored negative controls and native consumers.
Reconstruct selected methods with complete type/lifetime arguments and validate
evaluated SSA temporaries in their actual value domain. Build one flat applied
domain inventory and make one calls pass; do not rescan a subtree for every root.

Associated bindings in trait bounds retain resolved member identities separately
from positional arguments. Complete ordinary trait headers before resolving
header bindings through the supertrait graph, then publish all equality predicates
before freezing environments. Discard declaration-lookup application caches at
that transition. A member declared directly by the named trait takes precedence;
otherwise every inherited declaration participates in ambiguity. Preserve trait
arguments, defaults, raw identifier identity and the original projected roots.
Equalities on Self's associated types are implied by the trait's contract; a
requirement on another type parameter is not automatically implied.

Unused generic equality assumptions may disagree, including with a result's
declared Sized or Copy bound. Retain them without choosing a result. A requested
normal form needs a unique matching equality and a separate trait-premise proof.
Match complete input equations in isolated contexts; commit only the selected
equations and retain their lifetime obligations. Never equate rigid input or
result parameters merely because competing assumptions mention the same output.
Closed declaration equalities need independent implementation proofs without
using themselves as assumptions. Their normalization owns an explicit declaration
proof scope. Body equalities use the actual body's allocator and lifetime gate.

After proving a projection's trait premise, explicit equality assumptions still
take precedence. If no equality supplies a result, a completely concrete application
may use an independently selected impl despite a repeated caller trait bound.
Require the entire Self/trait application to contain no type or const parameters,
external lifetime variables or unresolved projections before this fallback. Static
lifetimes and locally bound callable lifetimes can be concrete. Cache this
distinction separately from the existing substitution flags, which also count
static lifetimes as free. Select without caller assumptions, respect
declaration/body implementation availability,
and retain all selection evidence and result-region restoration. This rule does
not remove explicit generic bounds or infer projection inputs from a result.

Instantiate both sides of a call's equality in the shared inference graph.
Schedule their normalizations and a suspended equality before inference finishes;
normalization may determine an output hole without guessing a projection's inputs.
The result has its own well-formedness and lifetime requirements. Its separately
retained trait premise supplies projected-subject formation. A proved opaque
normalizer response does not bypass any remaining shape, lifetime, ownership or
execution gate.

Index equality assumptions by associated declaration, then distinguish their full
inputs within that bucket. Remapping an environment preserves declaration order,
deduplicates newly identical assumptions and rebuilds its index. Cache whether
outer-region mapping may be necessary, including result-only captures. An
environment certified to contain no such regions can use canonical inputs
directly; unrelated equalities must not be cloned for each wide-list projection.
The type store's region summary remains conservative for static lifetimes.

Implementation selection treats associated equalities as proof work on its existing
iterative stack. A normalizer suspends at each trait premise and resumes from that
selected implementation or caller assumption; it must not start a fresh recursive
solver with independent cycle/cache state. Preserve the original projected query
and commit its complete input equations separately from the normalized result.
Result equations can determine output holes, never unrelated projection inputs.
Undetermined inputs defer until other premises provide enough information.
A normalizer's trait premise that is concrete, naming no parameter,
projection, non-static free region or escaping binder, selects only
implementations first, as rustc discards a global where clause when an impl
applies. The caller's clauses are tried only when no implementation proves it,
by reopening the same request once. Such an implementation-first frame neither
reuses nor records a cached decision for its goal. Scoped callable requests and
trait goals outside a normalizer keep the ordinary order. See the
[global clause premises checkpoint](docs/pa31-global-clause-premises.md).

A proved equality's result is installed as a normalization installs one. When
its normal form is a rigid projection, as for core's
`impl<I, U> Iterator for Flatten<I>`, whose `U` is fixed only by `IntoIter = U`,
the normal form's projection roots need no further normalization, so an
unresolved result hole binds to it. A later result for the same hole must prove
equality and never rebinds it. A result that is itself a projection with an
unresolved input still waits and never infers that input. The controls in
`pa20/tests/spec/contracts/proved-results/` record independent outcomes.

The same stack normalizes projected subjects and type arguments of trait premises,
and projected builtin conditions after other equations settle. Retain the original
input graph beside the selected normal-form view. Replace only independently
proved projection roots; keep ordinary type/const holes shared, including holes
beside a projection inside a tuple or nominal argument. A normalized result must
never infer its projection's inputs. Exact canonical operation identity may prove
congruence without exposing an opaque result's structure or properties.
The input plan retains the original predicate and its callable binder tree.
Identify each replacement by both its source scope and inference term: a shared
term beneath sibling binders is not one proof occurrence. Rewriting preserves
empty binders, captured lifetime depths and unrelated inference holes, and refuses
incomplete or conflicting replacement sets. Share transformed DAG work within
each scope across all predicate inputs; repeated roots must not cause quadratic
rebuilding. This structural plan grants no normalization permission by itself.
Open callable occurrences in the existing selection query's scope tree. Keep each
occurrence's normalizer and result-restoration cache separate, including when a
later root returns to a previously visited outer scope. A normalizer request may
introduce further callable binders; attach its selected equations at that actual
scope and close its result before resuming the enclosing fold.

Restore private result lifetimes through one incremental index of the retained
selected proof graph. Visit each selected node once across roots and scopes;
canonicalize local bound equations to their query aliases before indexing them.
Each closing context admits the original caller frontier and its actual ancestor
aliases, never earlier siblings merely because their allocations have smaller
numbers. Share the equation index by ownership transfer while preserving separate
scope mappings and caches. Cached substitutions keep their supporting equations;
the index neither discharges universal debts nor authorizes discarded alternatives.
Preserve a typed normalization-needed result from the early assumption lookup,
normalize the query and retry against the caller's concrete assumptions. A still
incomparable opaque operation defers; it cannot establish cached absence.

An input wrapper retains the original query, every normalization child and the
selected implementation or assumption. Allocate implementation-private regions
after normalization children, and preserve their complete interval through the
parent proof. Cache the original canonical predicate, not merely the normalized
result; partial queries become canonical only after independent holes settle.
When publishing a universal query, compare its retained original input graph
with that wrapper's original goal inside the opened scope. The rewritten result
is justified by the attached normalization children, not by equating it directly
with a projection operation. Export input refinements only after that scoped
comparison, preserving the final lifetime proof for normalized outlives conditions.
Builtin conditions with ordinary subjects keep their direct proof path. Suspend
only projected ones on the shared stack, retaining pending Sized and outlives
requirements for the caller's existing gates. General opaque outlives properties
and original callable formation contracts remain required; normalizing a premise
does not complete those separate declaration and lifetime responsibilities.

Equality memo keys retain both canonical roots in the immutable caller environment.
Cycle-dependent failures remain uncached, and evidence with private lifetime
allocations cannot be reused without fresh identities. Selected result lifetimes
stay in their child proof intervals until the enclosing equality and caller's
lifetime gate prove them. Receiver applicability may discover a candidate with
pending requirements; the ordinary callable still proves all selected bounds.
Normalization beneath callable binders in implementation equality premises still
requires scoped selection evidence; the current explicit boundary must remain
until that proof representation is integrated.

Apply each tuple field's coercion before checking the next field. Context inferred
from an earlier field can establish a later mutable-reference reborrow, including
through a block tail or nested tuple. A reference consumed before that context
exists remains a move; independent type holes do not acquire a shared context.
Array elements retain their common-type join before final element coercions.

Shared references also reborrow through an established reference expectation,
including ordinary call arguments, annotations and contextual branch/tail
propagation. Reuse the selected source-place conversion, fresh inferred region,
ordinary ancestry/loan checks and instance proof; reference kind supplies no
source Copy/Clone implementation. An unknown generic value expectation or an
inferred binding remains a move even if later equality discovers a reference.
Shared contextual unsizing starts from the same reborrow before the existing
metadata conversion. When source unsizing is selected, authenticate the current
CoerceUnsized header and prove its actual implementation and Unsize/formation
premises through ordinary trait obligations on that reference view. Classification
alone grants no conversion, Copy, imported-body or backend permission. Joining
types alone does not create this source context.
See [the shared call reborrow checkpoint](docs/pa16-shared-call-reborrow.md).

The unsizing mismatch probe is an exclusion query, not an equality or conversion
proof. A projected head awaiting normalization is unknown to that query; continue
checking other pairs for a known mismatch without binding inference variables.
Ordinary comparison retains its normalization fence, and the eventual body
equality or selected conversion must still discharge every actual obligation.
Invalid identities and exhausted traversal resources remain failures.

Caller-bound trait goals participate in the same inference graph as call arguments.
Import each probe with one shared mapping across Self and all type/const roots.
Index assumptions by trait identity and the rigid outer subject shape; do not scan
unrelated parameter subjects. Deduplicate identical assumptions before probing.
Failed alternatives never constrain the caller. Commit a unique match and retain
its actual assumption identity and every required lifetime equality. The ordinary
region checker proves those equalities; matching never erases lifetime obligations.

Function-pointer types inside trait applications use scoped alpha equality, not
function subtyping. Pair each callable's local lifetime slots bijectively as they
are used; renaming, reordering and unused declarations do not change equality.
Keep nested binder frames distinct, including callables with no local slots.
Only outer lifetime equalities may reach the caller's region proof, lowered by
the number of enclosing callable binders. An inference variable cannot capture
one of those local slots. Share ordinary shape/const equations and region mapping,
and cache rebasing for repeated variable uses. Canonical dependency flags keep
ordinary trait arguments on the existing equality path without scanning types.

An unconstrained whole Self is not inferred merely from an available caller assumption
or impl. Numeric variables retain their integer or floating-point domain: search
only compatible scalar heads, and infer a unique matching type when possible.
Apply numeric defaults only after ordinary equations and trait progress stop,
then retry the remaining goals. Candidate probes never apply numeric defaults.
Use the existing dependency identities and bounded ready queue for this frontier.
Distinguish visible numeric inputs from leaves beneath unresolved projections;
each hidden visit may upgrade once when normalization publishes a result. An
eligible seed yields to actual trait/normalization work before another default.
Drain real ready equations, including contexts with no watches, before resuming
fallback. Final publication rejects a live unresolved frontier; cancelled or
settled work does not block it. Charge growth, rehash and failed work through the
same cumulative inference payer. Do not add repeated whole-goal scans or an
independent scheduler/byte ledger. The
[operator checkpoint](docs/source-shift-or-checkpoint.md) records actual ordinary
watch costs and the valid shared-numeric-input case that remains unsupported.
Other arguments may be inferred, and a bounded worklist revisits ambiguous goals
after another goal constrains their shared arguments. Check declaration ambiguity
even for unused owners. Distinct lifetime applications that remain type-compatible
are ambiguous; identical repeated bounds are one assumption. A declaration-owned
exact bound with no competing index entry needs no inference probe. Bodies without
user trait call obligations take the empty fast path.

Resolve a destructor's impl target and receiver once. Retain the nominal item,
method identity, whole-reference receiver binding and captured lifetime slots.
Anonymous impl lifetimes are real slots, not fabricated declaration IDs. A
destructor must cover every instantiation of its nominal type: target lifetimes
are distinct universals; unused impl lifetimes are existential during coverage
checking. Prove declared requirements against nominal well-formedness, including
field-implied bounds, using the shared directed outlives graph. A receiver has an
independent late lifetime; reference mutability and binding mutability are
separate facts. A typed destructor is not an ordinary callable function value
and is not evidence that cleanup has been proved or emitted.
Index declared lifetime membership separately from first-use slot assignment;
capturing a wide impl binder must not scan all its declarations per parameter.
Reserve each actual Drop owner before evaluating any impl-target constants.
An exact outer-constructor query follows resolved alias heads, parameter forwarding
and only the defaults needed for that head; it does not evaluate const arguments
or stand in for complete target elaboration. Supplied arguments substitute once,
simultaneously. Finite self applications in defaults remain valid, while genuine
head/default cycles fail. Constant eligibility consumes these owner identities;
method bodies and cleanup still require completed implementations.

Generic Drop headers must cover the complete nominal schema. Invert direct,
distinct impl type/const arguments into the nominal parameter environment before
proving that impl and method bounds add no requirements. Include implicit Sized;
never use the impl's own assumptions to prove its conformance. Captured lifetime
slots are separate from named argument indices, including anonymous impl slots.
Use the shared outlives graph for lifetime and type-parameter requirements;
unused impl lifetimes remain existential. Inherited method environments retain
each parameter's original declaration identity. Classify such bodies as templates
until demanded concrete instances and their destruction plans are available.

Source const-definition qualification keeps one paid sparse partition in the
existing initialization fixed point and replay. Preserve root qualification,
selected semantic field coordinates, constructor tags and sticky qualifying
borrows independently of physical initialization, read/move access and loans.
A completed scoped Copy decision can seed a fresh runtime NeedsDrop cache before
compound demands; it cannot establish whole const Destruct. Retain the real Copy
and cleanup proof debts, refresh actual region contracts, and authenticate the
exact published defining state before accepting qualification, including empty
answers. Existing preparation/finalization paths keep the same initialization
owner and Seal. Original Bodies move with their genuine paired Requirements
rows; verify coverage/extents and prepay assembly overlap instead of guessing
body indices or cloning payloads. The bounded destruction query borrows the same
builtin matcher/radix DAG, counts growth once and keeps failure costs spent.
Unpaid warm reuse cannot bypass a bound. The
[selected-cleanup checkpoint](docs/parallel/source-cleanup-main-join.md) records
source/native and scaling evidence. Unit-result calls, interpreter cleanup,
conditional-parent history and wider authenticated transport remain separate
required consumers; qualification does not grant their execution permissions.

Template loan checking consumes complete scoped destruction requirements. When
no local, value or projection can need destruction and no cleanup sites remain,
reuse the checked initialization and storage facts with the ordinary loan checker.
Do not allocate an abstract cleanup plan for that case. Custom-Drop field move
legality, region proofs and initialization checks remain independent requirements.

When cleanup candidates do remain, inspect their proved initialization before
demanding recursive destruction properties. A definitely moved root or sparse
child has no cleanup access. Keep conditional and live paths on the ordinary
type, destructor and lifetime checks; this ordering cannot replace their earlier
typing, move legality or initialization proof.

Keep the checked inverse impl-to-nominal argument mapping on the completed Drop
implementation. Selecting a method for a concrete target consumes that mapping;
it does not repeat source matching. A runtime callable retains only target
lifetime captures and an independent late receiver binder. Unused impl lifetimes
and their bounds remain in the original method proof, rather than receiving an
arbitrary concrete witness. Validate that the selected receiver matches its
concrete nominal target, including reordered type/const arguments and nested
reference binders.

Destruction recipes for type/const instances use full canonical type identities
and prepared concrete fields, while retaining declaration field/variant order.
Lifetime-only recipes may retain their existing shared representation. An
unresolved generic destructor is distinct from a selected callable instance.
Separate retained recipe/query state from short-lived semantic borrows so new
instances can be discovered without rebuilding completed recipes. Actual cleanup
selection, original template proofs and instance ownership/lifetime/loan proofs
remain separate requirements before executable generic destruction.

The concrete instance scheduler also retains one unscoped destruction-property
cache across its demanded bodies. Full canonical types and completed normalized
fields determine these answers under one immutable custom-Drop registry. Each
body must still normalize its storage and retain its own formation debts before
consulting that cache. Independent symbolic environments keep separate queries,
and manual instantiation entrypoints start their own concrete cache. Sharing
structural answers never transfers a caller's lifetime or trait permission.

Prove each concrete body once as demand discovers it. Keep the immutable owned
body with its selected cleanup plan until executable assembly; match its source,
complete instance and original template proof. Assembly cannot discard cleanup
authorization or insert unproved bodies. Retain completed structural outlives
queries and recipe caches across append-only semantic discovery, including their
work budgets. Prove unused ordinary bodies without demanding their callees.

Discover destructor code from selected cleanup roots, following fields, variants
and nonempty array intervals with shared visited state. A cached recipe by itself
does not demand executable code. Enqueue selected methods on the ordinary instance
DFS stack before leaving their owner, so recursive destructor growth shares the
existing depth and storage limits. Prepare coercion destinations as well as
expression types: a distinct nominal lifetime instance needs its own field facts.

Keep dense liveness words for CFG joins, but enumerate only nonzero words when
recording point seeds. A compact summary tree preserves ascending identity order
without scanning every inactive entity at every cleanup event. Update summaries
on zero/nonzero transitions and rebuild them after dense joins; retain the same
work limits and charge enumeration and join work explicitly.
Keep independent entity bits for the fixed-point solution and for each move or
storage kill. During point recording, share immutable component sets and update
live component multiplicities on entity transitions. Seed each distinct named
lifetime or opaque parameter once per point, including a borrow's temporary
result requirement at its creation point. Rebuild multiplicities from the solved
entity state at block and cleanup-edge boundaries; never merge entity identities
because their current components agree. Operand coercion, ordinary use and
destructor queries retain their separate component policies. Bound construction,
transition and distinct-output work and retained storage; duplicate elimination
after enumerating all live entities at every point is insufficient.
Forward loan states use the same index for pruning, conflict checks and optional
inspection. Rebuild the occupied-word index after dense CFG copies, clear it when
the last loan in a word expires, and preserve two-phase activation separately.

Index loan expiration from sparse region-word boundaries once per body. At a
known consecutive CFG point, remove only the indexed expiring loan masks;
do not repeat region membership checks for every surviving loan. Dense joins,
nonconsecutive points and births outside the current region invalidate that
cursor and require the ordinary full membership check. Later region liveness
does not regenerate an expired loan. Keep full extents compact, account for
word-boundary continuations and omit exits beyond the point domain. Bound and
charge index construction and retained event masks as part of loan analysis.
An empty exit set allocates no point index. Choose direct dense rows only when
their size is bounded by retained exits and the index budget; otherwise use
sorted sparse rows over the same flat event payload. The numeric point domain
alone must never force proportional storage. Charge dense initialization and
prefix work, sparse sorting and lookup comparisons, and retain maximum-domain
empty/sparse controls alongside ordinary dense CFG tests.

Collect initial runtime lifetime seeds into one queued entry per region and
point word before propagating them. Seed collection already precedes propagation;
combine its bits in first-seen order instead of walking shared static or external
obligations separately for every point. Subsequent propagation retains only new
deltas. Keep the sparse result representation, local-sentinel exclusion and all
work/storage limits; charge both seed collection and completion of queued words.

The executable CLI enters this template/instance pipeline before the ordinary
inspection-only body builder. It must not duplicate promotion, constant or proof
passes. Earlier inspection modes retain their own contracts. Once explicit and
referenced implicit constructor templates have been discovered and original
proofs completed, an empty sealed template set permits skipping the generic
dependency walk: ordinary lowering already follows ordinary call/cleanup uses.
All ordinary source bodies must still be proved before this fast path.

Analyze generic bodies without reparsing syntax. A monomorphization key must
include all type/const arguments and semantic context affecting its result.
Trait solver caches include parameter environment, binder universe and mode;
negative results need equally complete keys. Each demanded fact is computed
once; cycle detection and explicit solver budgets are part of the contract.

A quantified clause binds the whole predicate: its subject, complete trait inputs,
outlives requirement and equality result share one local lifetime scope. Retain
an explicit clause variant and an interned binder body, leaving ordinary flat
predicates and inferred goals compact. Allocate binder storage only on demand.
Canonical identity validates representation; it supplies no formation or selection
evidence. Cache required local slot counts over shared type graphs, independently
of the enclosing clause's declared count. A new clause must not rescan a growing
shared tail merely to validate its binder. Reuse the existing occurrence fold for
capture detection and slot summaries, with their distinct summary operations.

Substitution of enclosing arguments preserves the clause's own binder, including
an empty binder that changes captured depth. Transform every predicate root at
that same depth, shifting inserted type arguments beneath nested callable scopes.
Separate root and scoped trait-application caches; a canonical application can
have different results at different binder depths. Opening a universal obligation
or instantiating a universal assumption requires a separate scoped proof operation.

Declaration predicates, their original pre-normalization forms and trait-header
consequences retain whole clauses. A metadata query may inspect a bound subject
or member owner without granting the unbound body as a premise. Associated-result
ownership compares the complete formal parent, shifting its lifetime parameters
beneath the result clause's binder. Consumers that do not own a quantified scope
must reject it explicitly; filtering it away is not successful validation.

Elaborating a universally quantified supertrait preserves its surrounding binder.
Adjacent universal binders can compose only through capture-safe slot remapping:
append inner locals after outer locals, lower the removed parent depth, and keep
nested callable locals distinct. Validate outer slots before merging so an invalid
parent index cannot become a newly appended inner slot. Share validation and
opening folds across the immutable elaboration batch, charge their work and retain
non-Self requirements separately from inherited caller assumptions.

Inserting a canonical type argument under binders is independent of its parameter
mapping. Share these structural capture shifts in the canonical store, keyed by
shift amount with type identity and nested binder depth inside each fold. Keep
replacements simultaneous: another parameter mapping must not rewrite an inserted
argument. Allocate this cache lazily, retain completed work and budget on errors,
and reject further work after exhaustion. Charge incremental shift work to each
requesting substitution as well as the cumulative store budget. No trait selection,
formation proof, caller hypothesis or implementation availability belongs here.

Immutable caller environments expose whole trait clauses. Keep ordinary predicates
compact and allocate quantified storage lazily; retain mixed declaration order
through capture substitution. Mapping an enclosing scope preserves every local
binder, including empty binders. Use cached type/application summaries at the
clause's actual depth to distinguish captures from local slots. Share parameter
occurrence walks and substitution memoization across the entire environment.
Quantification binds lifetimes, so it never exempts type or const inputs from
caller-membership checks. Unsupported quantified assumption kinds must fail
explicitly instead of becoming ordinary builtin or equality permissions.

Using a universal caller premise creates fresh existential aliases at the actual
requesting allocator frontier. Isolate candidate equations and inference until
one complete application is selected, then retain its private interval, original
clause and instantiated subject/trait arguments. No new universal proof scope is
introduced by this use. Preserve the allocation frontier across header waits and
exclude allocations belonging to other work. Unknown Self remains deferred; a
unique compatible premise may constrain other inference inputs. Competing clauses
must not leak equations or be omitted by an exact ordinary or projection shortcut.

Alternative quantified premises within one selection may share binder-opening
work at the same arity and trial frontier. Reusing canonical transformations does
not share permission: only the selected application exports its evidence and
owned aliases. Flat selection entry points reject environments they cannot consume
without binder ownership. Projection and body consumers retain quantified origin
explicitly; validation must confirm the original clause belongs to that caller.

Associated equality environments likewise retain whole clauses, indexed by the
projected member without dropping competing ordinary or quantified results. A use
opens its complete projection inputs and result with one shared binder mapping.
Match complete parent inputs in isolated inference before committing the unique
candidate; equal results never imply equal projection inputs. Fresh equality
witnesses begin after the selected parent proof at the actual allocator frontier.
Canonical normalization, body inference and nested implementation search must all
retain this interval and every input equation. Nested search places the interval
in its requesting proof scope and excludes it when that alternative is discarded.
Captured substitution preserves mixed declaration order, local binders and nested
callable scopes. Ordinary equality environments keep compact entries and reuse
their existing substitution storage.

A completed associated-result closure builds one exact equality index for that
same closure and shares the existing Normalizer across its consequence roots.
Result-bound selection still requires the actual parent proof; missing,
conflicting, cyclic or quantified equations grant no guessed replacement.
Observed normalization prepays canonical Store growth as well as its scratch
fold: nodes, child/lifetime lists, mixed arguments and trait applications,
including resume and binder completion. Lookup-only equality probes never insert
a canonical node. Use the shared interning/folding kernels with a transient
borrowed payer and retain failed payments cumulatively; add no duplicate DAG
walk or persistent accounting owner. Ordinary interning keeps its existing
canonicalization, probe and validation order.

When otherwise matching quantified alternatives differ only in unused or reordered
local slots, compare their complete clauses modulo local numbering. Preserve all
outer captures, nested callable locals and result correlations. Lazily cache these
structural keys by immutable clause identity; keys never replace a source clause
or authorize a proof. Retain one actual premise and its complete witness interval.
Ordinary and quantified alternatives keep their existing ambiguity rules.

Independent conjunctive universal goals in one immutable canonical declaration
state may share a universal scope and opening fold when their binder arity,
closed/open mode and implementation availability agree. These scopes contain
requirements only; they introduce no local hypotheses. Each selection keeps its
own inference and fresh implementation witnesses at the actual allocation
frontier. Commit a fresh scope only after selection succeeds, and retain every
goal's lifetime debts even when its selection is subsequently reused. Separate
callers, outer substitutions and binder arities cannot share this permission.
Measure both structural work and canonical storage across distinct goals with
growing shared tails; memoizing repeated identical goals alone is insufficient.

Implementation conditions retain whole quantified clauses after capture mapping.
First settle ordinary equations, then resolve the parent's complete type and const
arguments once. Universal children use isolated inference on the existing iterative
search stack; they cannot refine the parent with a local witness. A parameter that
can be determined only through a quantified equality remains deferred until that
inference mode is implemented. Share substitution and binder opening across one
candidate's conjunctive conditions, keyed by arity and its actual parent scope.
Different candidates do not share these universes or their completed answers.

Retain scopes, candidate cursors and private allocation ownership through header
waits. Once a universal scope has been introduced, reserve subsequent identities
monotonically so discarded alternatives cannot reuse its aliases. Keep an ordered
allocation ledger and sparse node ownership; collect only the selected graph's
scoped witnesses and constraints. The allocation envelope may retain unreferenced
reserved identities, but never a discarded candidate's equalities or permissions.
Assemble the selected scope tree and allocation blocks in linear passes, without
sorting intervals, recursively assembling proofs or copying ancestor prefixes.
Foreign intervals remain excluded. An empty quantifier still introduces a scope:
region-free evidence with quantified descendants cannot enter the flat answer cache.
Scoped answer caches include the actual universe; inductive cycle cutoffs remain
relative to the active search and cannot become completed negative facts.

Separate generic parameter identity, alias expansion and default-template
readiness. Explicit arguments demand no unrelated defaults. Omitted arguments
demand each needed template through the bounded declaration worklist; a default
can use an explicit self application or a later default whose template depends
on an already supplied parameter. Detect actual default cycles independently
of alias-body cycles, and check every unused default before publishing facts.
Elaborate alias bound subjects after their bodies are available; parsing those
bounds does not make them enforced requirements at alias uses.

Reserve nominal identities separately from complete field templates. Demand fields
on the same bounded declaration worklist, publish a complete group only after its
annotations finish, and distinguish field/default cycles from pointer recursion.
Index constructor owners once; initialize tuple-callable facts only when demanded
or during final completion. Register representation and derive selections before
early semantic queries can cache absence. Constant dependencies include referenced
aliases, constructors, immutable statics and named initializers, by-value shapes,
enum discriminant expressions and local cast annotations in their lexical contexts.
These prerequisites make values available; final unused-declaration, variance,
derive, lifetime and layout checks remain mandatory.

Value-form `Self` resolves to its actual impl owner before selecting a callable.
Suspend on that owner's header, retain the exact instantiated Self type and
select the nominal's existing tuple or unit constructor with its ordinary access
checks. Reuse the sparse constructor-use index, distinguishing a syntax prefix
from a retained Self type; the value node cannot also serve as its own nominal
annotation. Nested items may retain a closed outer Self constructor but cannot
capture impl parameters or free regions. Do not relax ordinary Self type lookup,
manufacture a constructor declaration or add a dense field to every body.

Early expression paths use the ordinary argument scheduler, including alias
variant selection, function values, partial annotations and consumed trailing
defaults. Nested const arguments own their dependency work and lifetime context.
Determine value readiness from each nominal declaration's field templates once;
follow actual type arguments separately. Do not expand symbolic field instances
to discover these prerequisites. Instantiate concrete fields after inference.
Prepare nominal declaration requirements separately by following canonical type
edges, including reference referents and trait-path inputs, without following
field definitions. Share completed requirement readiness by canonical type and
nominal identity. An active dependency is not a completed requirement or proof.
An unused referent must not demand its field layout merely to form its type.
Distinguish inferred annotation terms from value nodes. Annotation terms retain
their formation and lifetime checks without demanding field storage. Constructor
expressions remain values even when their type comes from an annotation. Keep
field receivers separate from cached nominal substitution environments: creating
an environment does not imply that any field was selected.

When inference discovers a missing field declaration, suspend at that exact
operation with the prepared receiver, inference graph, caller environment and
pending proof intact. Resume through the declaration worklist without replaying
the body or recursively draining declarations. Publish initializer typing only
after completion, and reject a dependency on an active initializer. Suspended
bodies retain sparse node and binding slots proportional to their own work;
neither parking nor constructing a replacement checker allocates whole-crate
slot arrays. Ordinary completed bodies may reuse dense scratch storage. Storage
selection must not change which implementation evidence is available.
Reuse the immutable source-work budget across declaration queries and resumptions;
reconstructing a query must not rescan all source units for each dependency.

Before evaluating a const argument, reject references to enclosing generic
parameters while allowing lifetimes bound inside the constant. Cache syntax
capture summaries using scope ancestry, preserving syntactic captures even when
type aliases erase them. Use compact node-indexed completion flags and sparse
capture records, so capture-free syntax needs no per-node hash entry. Standalone
const parameters retain their symbolic identity.

Retain runtime const operands as canonical const arguments, separately from
named constant definitions. Publish array index bounds after inference resolves
the receiver: a bound is either a target `usize` value or the original rigid
const parameter. A symbolic fixed array does not become a slice or a zero-length
array. Place overlap and move checks use the same conservative indexed-place
rules while the actual bound awaits substitution. Keep concrete node facts compact.

Resolve a use's defaults with one append-only substitution environment. Earlier
arguments never change, and inserted replacements are simultaneous. Successful
cached results therefore remain valid as later slots are appended, including
binder shifts. Intern the completed argument list once and retain that same
environment for the expanded body. Do not copy and intern every argument prefix
or create a separate cached substitution for each omitted parameter.

Infer nominal variance over completed declaration templates, with distinct
lifetime, type and const slots in declaration order. Compose variance variables
through nested nominal arguments using shared product expressions and reverse
dependencies; solve the finite lattice with bounded monotone propagation.
Never expand recursive field instances to infer variance. Callable binders
retain their own lifetimes. Recursive-only type/lifetime parameters remain
unused; const slots require exact equality even when no field uses them.
Directed region checking consumes the completed table for both lifetime and
type arguments, after structural inference retains the actual/expected types.

An equality-normalized field retains its original declaration dependencies for
variance. Preserve sparse original field lists by nominal and variant, with the
same arity and ordering as storage fields. Original projection inputs remain
invariant even if their result becomes a scalar; bounds alone cannot manufacture
field occurrences. Multiple declarations may share a normalized type without
sharing their original inputs. Layout and value operations consume the normalized
storage fields. Ordinary variants allocate no retained field lists, and inference
reads the original lists through the same bounded variance graph without field
expansion or mutation of canonical types.

Generic nominal fields are instantiated from completed typed declarations, keyed
by the canonical nominal type including every type, const and lifetime argument.
Reuse one capture-safe substitution across an instance's fields. Share the
declaration's variant order, representation and final discriminants; retain only
substituted field lists per instance. Prepare demanded fields before immutable
layout and ownership queries. Follow owned fields and derive argument obligations
iteratively, stopping at pointer/reference referents; equality separately demands
reference referents. Do not expand growing recursive pointer instantiations.
Implicit field dereferences also demand their selected receiver's fields, even
without a separate syntax node. Retain receiver discovery order when publishing
these canonical types; hash-map iteration must not determine type identities.
Failed preparation cannot publish unfinished descendants as prepared. This
availability check does not prove declaration validity, bounds or finite layout.

Keep raw substituted fields distinct from normalized concrete storage fields.
Normalize concrete fields through independently validated implementations before
ownership, destruction or layout queries can observe them. An unresolved concrete
field is a preparation error, not a conservative cached layout or Copy answer.
Lifetime-only nominal arguments also require substitution when their fields
contain projections. Cache that structural requirement once per declaration and
instance instead of scanning every field during each property query.

A normalized field list never grants another caller permission to form its raw
projection. Index the structural dependency graph once per canonical type and
retain its raw projection leaves in a persistent recipe DAG. Empty dependencies
vanish; a single dependency shares its child's recipe without copying its path or
flattening its descendants. Each body or concrete instance proves those leaves in
its own scope, including lifetime debts erased from scalar results. Share one
completed-recipe set and normalization session within that scope. Traverse owned
fields and derive argument dependencies with bounded worklists, stop at pointer
and reference referents, and publish completed structural lists separately from
the caller's evidence. Demand published body values and selected receivers, not
discarded provisional inference templates. Index projected declaration field
flags so signature-only projections do not trigger storage traversal. Keep
generic fields symbolic until actual instantiation;
template representation checks treat unresolved projections as unknown layouts.
Concrete instance discovery must include projections hidden in nominal fields,
not just projections visible in the nominal's argument graph.

A storage query owns its traversal and exact pending field while declaration work
completes. Release the shared registry before resolving a field or yielding a
dependency. Publish completed structural nodes with append-only recipe identities;
a child may complete the suspended parent's root without invalidating its pending
IDs. Incomplete shapes remain unavailable. Keep the caller's proof frontier and
expected raw-to-normal pairs separate, and publish its completed-recipe set only
after the entire proof succeeds. A shared structural root cannot erase a pending
normalization's obligations.

Body finalization owns solved types, predicates, item arguments and selected
evidence after consuming inference. Its storage batch retains one normalizer and
proof across roots and waits. Reserve only new proof intervals before yielding,
exclude intervening declarations' allocations on resume, and publish the body
facts once after preparation completes. Do not replay inference or completed
fields to recover a missing definition.

Projected field inference retains its caller lifetime context after the pending
normalization worklist drains. Later shape comparisons use the same universal
aliases; queue emptiness controls progress, not the meaning of existing terms.
For record constructors with omitted lifetimes, a known expected nominal with the
same owner can supply those input lifetimes before field substitution. Preserve
explicit written lifetimes and prove the usual formation and coercion debts.
An associated output equality cannot invent equations between its inputs.

The source from_yeet helper completes through the shared ordinary defined-function
header adapter. Retain its actual generic bounds, caller tracking, signature,
source body and FromResidual dispatch; a language tag grants no implementation,
const, unsafe or lifetime permission. Valid unused alternative headers remain
ordinary functions. Implicit yeet operations and imported consumers require their
separate actual operation/signature and transport contracts.

An ordinary generic factory returning a nominal with the expected owner uses the
same construction context before scheduling its arguments. Reuse the existing
unsizing-tail hint so an expected slice/object does not become a sized input.
Check equation compatibility without binding first: written or already fixed
inputs retain their actual type and may use a separate result coercion. A pending
associated projection supplies no early compatibility hint; leave it to the
ordinary deferred equations and projection proof. Preserve
normal lifetime, formation and final coercion proofs; a result hint supplies none
of those permissions. Do not specialize this rule to Pin or replay arguments.

Stop a construction-context tail walk at its actual completed Sized parameter:
a sized tail cannot need an array/object input hint. Partial callable detection
uses the existing inference Completed ledger, retaining stable subgraphs during
its own postorder traversal without early type interning or a second graph pass.
Unresolved type/const inputs and projections never receive a completed absence
certificate. Later equations can still expose a callable binder. Preserve ordinary
region, subsumption and conversion proof; cached shape facts grant none of them.

Before freshening a nominal value's lifetime inputs, preserve its existing identity
when completed raw projected fields prove every input invariant. Reuse the variance
solver for that conservative hint; unfinished external nominal edges grant no
information. An unknown or partially varying owner still uses ordinary inference.
Cache only completed-owner results, and keep full declaration variance validation
mandatory. This avoids cloning the same invariant input graph for each caller; it
does not cache projection permission or remove the caller's formation proof.

Complete nominal parameter schemas and declaration contracts independently of
field instances. Prove field and use-site predicates in their owning immutable
parameter environment; a cached predicate list is not a cached proof across
environments. Generated Copy, Clone and equality implementations have separate
assumptions. Propagate by-value parameter requirements over declaration templates
to reject infinite storage without expanding growing recursive instances.
Zero-length arrays still require their element alignment. Infer field-implied
outlives requirements over the same symbolic arguments, retaining type-parameter
requirements as well as lifetime pairs. Retain opaque associated results as
separate predicates, with complete Self, trait and family arguments. Propagate
these templates through nominal uses using shared joint substitution; do not
normalize during inference or infer input bounds from a result requirement.
Captured callable predicates lift to the owning scope, while predicates involving
callable locals remain deferred.

Prove related canonical nominal roots in one actual caller query when their
complete predicates are ready. Reuse completed formation and child selections
inside that query, preserving fresh witness intervals for each sibling root.
Finish with one union traversal of the real proof graph so shared suffixes do
not incur a fresh traversal for every root. Retain every root's region, equality,
sizing and selected-implementation debts; a failed or deferred root grants no
partial satisfaction. Builtin Sized keeps its existing shape producer. Body
queries use their actual alias substitution, caller assumptions, environment and
region start; declaration proof cannot stand in for body permission. Ordinary
single-root, quantified, pending and unsupported profiles retain their existing
producers. Query-local reuse never widens cross-caller evidence eligibility or
raises work/storage limits. Rigid receiver correspondence compares complete
arguments through the shared paid structural matcher; item identity alone is
insufficient, and the final directed lifetime proof remains mandatory.

Prepare projected requirements for requested canonical roots before immutable
lifetime proofs, including generated constructor and specialized body contracts.
Cache substitutions by complete nominal arguments and charge traversal, capture
queries and mapping to bounded work. Preparation follows structural inputs without
expanding nominal fields; ordinary contracts allocate no projected state. Cache
requirements, never proof permission. Static field requirements remain obligations
against explicit and independently implied premises. Scalar-only query interfaces
must reject hidden projected debts even when the nominal root has closed arguments.

Nominal requirements inside function pointer types retain their callable binder
depth. Defer predicates mentioning a surrounding late-bound lifetime, as required
by [RFC 1214](https://github.com/rust-lang/rfcs/blob/main/text/1214-projections-lifetimes-and-wf.md#checking-conditions).
Prove independent predicates in the caller's scope after capture-safe lowering;
a predicate's own nested binders do not make it deferred. Cache occurrence and
lowering queries within each immutable scope, with a shared work budget. Deferred
predicates grant no body permissions. At a call, check the instantiated inputs
and result even if an argument diverges or the result is discarded.

Cache immediate nominal requirements as whole clauses. Substitute complete
nominal arguments without opening a requirement's own quantifier, including an
empty quantifier. When deciding callable deferral, start a quantified body's
capture query inside that binder: its locals are not surrounding callable locals.
Independent ancestor captures lower through the callable scopes while retaining
the clause binder. Key capture application summaries by query depth, with a lazy
scoped table so ordinary queries retain their compact cache. Quantified outlives
clauses remain proof obligations; the separate ordinary lifetime-contract lane
does not discharge them. Formation retains universal evidence until the caller's
final lifetime gate and requires an explicit proof scope.

Nominal implication filters each complete outlives clause before extracting its
components. A predicate mentioning its own quantified lifetime contributes no
implied scalar or projected bound, even when other components are independent.
Lift an empty or unused quantifier without capturing nested callable locals.
Ignore non-outlives clauses only in this implication query; nominal formation
still checks their complete contracts. Share capture summaries and lowering
folds across canonical clauses, charge their work, and allocate scoped state
only when demanded. This implication filter is separate from elaborating an
explicit caller assumption and must not replace that consumer's rules.

Explicit caller outlives assumptions retain the whole quantified predicate.
A direct type parameter bounded by a local quantified target supplies an
all-lifetimes bound; a composite subject with that target supplies no component
bounds. For an independent target, elaborate only independent scalar components
and preserve opaque results without inferring permissions for their inputs.
Empty and unused quantifiers still lift owner captures correctly.

Body contracts retain whole outlives clauses. Elaborate an original parameter's
all-lifetimes permission before specializing that already-checked body; replacing
the parameter with a composite must not reclassify its established permission as
new source syntax. Method conformance instead elaborates after mapping the trait's
Self into the implementation subject, following the pinned compiler's override
rules. These operations have different callers and must not be conflated.

Batch quantified composite caller premises by their independent target lifetime.
Walk all subjects for one target together, visiting shared graph nodes once and
retaining opaque projected components at their original binder depth. Keep the
ordinary assumption path and the top-level quantified projection index separate.
Charge clause visits and traversal work; do not construct synthetic types or
cache a flattened component vector for every growing prefix. Different target
lifetimes and different caller environments retain distinct permissions.

Match universal projected premises against complete typed inputs. Preserve
repeated-slot identity, input/target correlations, fixed captures, member and
family arguments, and nested callable binders. An outer premise slot cannot
capture a requested callable's local lifetime. Captured-input equalities are
conditional tests, never inferred equations. Index lifetime-erased rigid shapes
only to select candidates; full structural matching remains the proof. Cache
matching recipes by both inputs, binder depth and actual proof scope. Share type,
list and argument traversal, persistent slot bindings and conditional guard sets
so growing shared prefixes do not require repeated walks or copying. Keep this
state lazy for ordinary environments. The source boundary remains closed until
all producers and consumers preserve these scopes and qualified source behavior.

Declaration lifetime validation retains inherited quantified outlives clauses
until the graph opens their fresh universal scopes. Requirements cannot become
caller assumptions. Method receiver and nominal static checks use the same
caller-clause consumer; ordinary scalar nominal checks retain their fast path.
Occurrence queries for captures and local slot counts also share argument and
callable-parameter list summaries by binder depth, across different owning types
and traits. A shared list never collapses its different surrounding scopes.

Method conformance substitutes whole clauses into the trait-owned parameter
environment. Keep caller assumptions separate from the implementation's required
conditions; prove non-lifetime clauses with scoped trait evidence and lifetime
clauses through the final directed graph. Captured owner lifetimes never become
universal permissions. The type/trait environment validates parameter membership
in lifetime clauses but leaves their proof to the retained lifetime consumer.
Consumers without a scoped equality proof must still reject that premise.

Deduplicate universal projected caller premises by their complete predicate body
when only unused top-level slot counts differ. This caller-only equivalence does
not merge required proof scopes or change definition-time binder classification.
Repeated equivalent premises must not multiply candidate scans for each goal.

Declaration formation for a quantified trait bound is a separate structural
query from using its logical implications. Under the pinned compiler's rules,
defer formation of each entire subject or type argument that mentions the
clause's own local lifetime. Lift independent inputs and individual inherited
requirements to the surrounding declaration without capturing nested binders.
An independent type argument still requires formation when the subject is local.
No deferred input supplies a caller assumption. Equality results and type-outlives
subjects keep their recursive term-formation rules rather than this whole-input
filter. Cache scoped formation by canonical clause within the trait-header epoch;
reuse the ordinary requirements cache and allocate scoped state only on demand.
Charge shared capture/lowering work and invalidate it when header bindings change.

Quantified builtin bounds normalize their subject with scoped projection evidence,
then defer formation of a whole subject mentioning a clause-local lifetime.
Lift independent subjects into the declaration's surrounding scope before checking
their nominal and shape requirements. Share capture and lowering summaries across
the proof's canonical inputs, including growing prefixes; ordinary clauses do not
allocate this scoped cache. Definition-time satisfaction uses the original whole
clause's classification and an independent builtin proof, never the bound's own
assumption. An empty quantifier preserves its structural coordinate shift; an
unused nonempty quantifier defers satisfaction until use.

Record each quantified builtin's completed independent formation input by both
declaration owner and clause identity. Body lifetime collection consumes these
typed facts and fails if they are missing. Normalization may depend on the owner,
so another owner's record cannot substitute for this one. A deferred subject has
an explicit empty record and grants no formation or outlives permission. Actual
body uses still check their own types. Instance witness import selects user-trait
premises separately from builtin requirements already proved at the call site.

Retain quantified builtin assumptions in a separate immutable environment index.
Whole-subject Copy, Clone and Sized permissions feed copy, shape, destruction and
drop-lifetime queries; they never imply reverse element bounds. Use shared
lifetime-erased structural fingerprints only to select candidates, then compare
their complete inputs with explicit binder coordinates. Captures retain their
enclosing identity, repeated local slots require one consistent substitution,
and a premise slot cannot capture a lifetime introduced by a nested callable.
Exact structural permission queries cannot assume an unproved lifetime equality.
Conditional matches require separately retained proof before authorizing an
operation or omitting cleanup.

Environment clones share completed builtin indexes and lifetime classes through
immutable ownership; extending a scope uses copy-on-write and never changes its
parent's permissions. Substitution rebuilds the mapped index directly rather than
cloning the original index before replacing it. Exact ordinary Copy and Sized
facts answer queries before structural matching or matcher allocation. An exact
Clone fact still needs further evidence for Copy, and cache scope checks apply
even when an exact fact answers the query.

For builtin permissions, derive named lifetime equivalence once from declared
and implied caller assumptions using an iterative strongly connected component
traversal. Include static's intrinsic outlives edges; exclude required body
constraints and arbitrary inference-variable equalities. Retain the resulting
classes with the immutable environment before copy, shape or drop query caches
are created. The body's registered universal aliases share their named classes.
Substitution transports the complete equivalence relation, closing classes when
mapped identities coincide. Nested callable locals and enclosing foreign binders
retain their independent identities. One-way outlives is not equality.

Share structural comparison and fingerprinting with universal outlives queries.
Memoize input-pair recipes and persistent slot maps across growing shared subjects;
cache completed permissions only within the originating immutable environment.
Keep original clauses for scoped substitution while deduplicating identical
candidate bodies with unused binder slots. Allocate matching state lazily so
ordinary region-free environments retain their direct lookup path. Lifetime-bearing
ordinary predicates share the index, but their caller slots are rigid captures,
not quantified substitution slots. Include that distinction in candidate and
recipe identity and preserve it through substitution. Exhausted query budgets
are errors, never failed-bound answers.

Bounded source consumers borrow a payer through the same builtin query,
fingerprint table, full matcher and persistent slot-map DAG. Prepay query-box,
work-vector, pending-child, memo-table and radix-node/union growth before
allocation, including table replacement and copy work. Do not clone growing
binding prefixes, construct a parallel solver or add a payer to ordinary bodies.
Cached results still pay their actual query work without charging fresh storage.
Keep the query's first failure unusable, including when a caller later asks
through its ordinary API; a wrong environment cannot poison another owner's
query. Return budget failures as errors, never Copy absence or a cleanup answer.
Payment grants no premise or directed lifetime equality. The
[paid-query checkpoint](docs/parallel/paid-builtin-query-checkpoint.md) records
the shared adapter separately from its pending symbolic cleanup consumer and
private execution checks.

A failed builtin query on an inferred lifetime-bearing argument may retain a
typed pending requirement with its source and immutable caller environment. It
does not establish Copy, Clone or Sized. Transfer those requirements with the
body's lifetime contract, including unused generic bodies. Only after the actual
region graph proves its constraints may mutual reachability extend the caller's
identity classes. Exclude the abstract LOCAL sentinel from that equality graph;
it does not identify distinct runtime borrow extents. Use the existing complete
copy/shape queries to discharge every pending requirement before issuing an
executable or template proof. Specialized contracts substitute and normalize the
requirements and verify their concrete subjects again without template assumptions.

Keep an independent own-trait formation requirement separate from the logical
clause. A clause-local input defers that declaration ambiguity check; it does not
make a later ambiguous goal usable. Check remaining declaration requirements
against only the caller assumption index in isolated inference, retaining no
trial equations or implementation permission. Equivalent already-opened goals
may keep one real premise and its owned lifetime interval. Distinct correlations
remain alternatives, and source membership and allocation-frontier checks precede
selection.

Recursive equality/outlives term formation starts at its actual clause depth.
Lift independent nominal requirements individually, including beneath a local
reference; defer only a requirement mentioning a surrounding local. A local
Sized requirement may be deferred without reporting the type as sized. Keep
formation shape caches separate from ordinary use checks and from shape answers
requiring retained normalization evidence. Neither a deferred condition nor a
completed structural shape supplies a lifetime well-formedness proof.

Body lifetime formation retains each equality result or outlives subject together
with its clause binder. Defer an individual outlives debt mentioning any local
before decomposing its type; still traverse child types for independent nested
debts, including projected Self/type inputs and nominal arguments. Trait clauses
instead consume the completed declaration's whole-input formation filter. None
of these formation checks grants a caller permission.

Within one body, stream only newly discovered formation requirements from each
actual source root. Share structural visits and canonical persistent component
sets across targets, keeping binder depth and component policy in structural keys.
Keep the incremental component walk while there is only one target/policy pair;
promote once to shared sets when another appears, retaining already published
debt identities and their first source. Reuse the bounded radix-map implementation
for set unions. Target cursors visit
shared set nodes, so repeated components in growing type prefixes do not cause a
fresh structural walk for each lifetime. Shared argument and callable-parameter
lists also have summaries. Charge structural edges, unions and published debts;
when both components and targets grow, the actual output relation can grow as
their product. Do not cache a flattened requirement vector for every growing
prefix. Validate each root's local slot count even when its graph was visited
before; charge repeated roots and capture work.
A fresh body needs fresh formation state and a separate assumption environment.

Specialization and dispatch preserve the formation binder during substitution.
Normalize projected formation terms inside real binder scopes and retain every
new selection debt until the final region proof. Detect projections present only
in lifetime contracts, including manually constructed contracts; declaration
cache population is not authority to skip them. Share concrete normalization
within each arity group, without converting its debts into assumptions.

Scoped term normalization opens enclosing owner captures beneath the clause
binder, preserving its local and nested callable identities. Related roots share
one cursor per fixed environment, universal arity and implementation availability;
each selected implementation still owns fresh witnesses. Retain all scoped
evidence until the final directed lifetime proof. Restore private selection
identities before closing the immutable owner aliases. Share opening, structural
normalization and closing folds across roots instead of scanning the accumulated
proof or a growing type prefix for each request. Unknown free identities fail.

Classify definition-time satisfaction from the original complete clause using
cached properties. Generic inputs, free lifetimes and nonempty local lifetime
binders defer satisfaction, including binders nested inside callable subjects,
trait arguments or equality results. A nonempty unused clause binder also defers;
an empty binder does not. Escaping-lifetime depth alone cannot detect a closed
callable's local binders. Closed projections still require normalization before
their independent definition proof. This classification is neither a formation
proof nor permission to omit the obligation when the declaration is used.

Parameter-default validation has a separate eligibility rule. Inspect the written
bound before substituting the default: any lifetime occurrence defers this check,
including static lifetimes and nested callable locals. Read retained original
projection inputs for this decision: normalization may erase or introduce a
lifetime. Index type/const occurrences in the normalized clause separately; an
erased type parameter does not become a new default obligation. An unused clause quantifier
does not defer it. Retain the whole eligible clause through substitution and proof;
a lifetime introduced only by a closed default cannot change the original
classification. Actual uses still establish all declared bounds. Index the sole
type or const parameter with shared summaries of canonical types, lists and
argument groups, including equality results and non-injective projection inputs.
Do not rescan every growing type prefix or retain a parameter set per prefix.

Constructor and pattern inference retain unresolved mixed arguments. Reuse one
substitution session across the selected receiver's field templates; do not
resolve the receiver before its fields have constrained it. Substitution under
callable binders shifts inserted outer lifetimes without capturing inner ones.
Written partial arguments, omitted arguments and omitted trailing defaults are
distinct inputs. Alias-qualified variants retain their resolved namespace prefix
and selected member rather than repeating path lookup in body checking.

Infer partial callable shapes by opening source lifetime binders only where the
destination supplies matching callable structure around its holes. A hole for an
entire higher-ranked function-pointer type retains that type's own binders. Keep
the original source for the separate directed subsumption proof. Give sibling
callable occurrences distinct fresh lifetimes, and memoize unchanged shared
subtrees without merging those occurrences. Charge this iterative shape walk and
its substitutions to the same inference work budget. Detect source binders from
the current typed graph before attempting resolution on the generic path; this
query must not intern partial shapes or freeze answers across later bindings.
Register projections in
instantiated generic signatures and nominal fields in their resulting scopes;
an earlier query for the input argument does not cover a substituted occurrence.
A body annotation resolved to a declaration type that still names a
projection, such as a shorthand `T::Assoc`, registers the same body
normalization as a qualified path. An earlier normalization of the same type
by a signature does not cover it. An isolated method prefix keeps the resolved
type. When a coercion binds an actual type to an unresolved expected type, and
a projection nested in the actual type has not normalized yet, the type is
related by equality without fresh regions, as a projection head already is. A
future normal form can complete that equality, but cannot supply a conversion
missing at the source point. See the
[body projection normalization checkpoint](docs/pa31-body-projection-normalization.md).

Retain actual callable scope kinds through partial annotations, normalization,
proof copying and grafting. Callable-local family input conditions defer under
RFC1214; explicit quantified goals and independent outer conditions remain
requirements. Filter complete clauses before opening their coordinates. Deferred
conditions grant no caller assumptions. Preserve explicit lifetime relations,
including relations between independent callable locals; binders alone grant no
ordering. Reuse immutable trait-name reachability
after header completion to avoid opening irrelevant quantified naming bounds;
actual member and implementation selection still consume semantic contracts.

After checking a call's arguments, progress its own ready trait requirements
before consumers require the result's structure. Reuse the indexed call owner,
notifications, suspended queries and complete evidence. Inspect existing input
summaries once without interning or walking unresolved suffixes; do not repeat
a wide result prefix after each ready goal. Register newly introduced result
projections in the same body normalizer. Unfinished requirements, number fallback
and final lifetime/ownership proof retain their ordinary boundaries. A tuple
field cannot invent an arity; check its index against the actual selected tuple.
The [PA24 callable-result checkpoint](docs/pa24-callable-results.md) records the
earliest reductions, independent outcomes and candidate qualification.

For projected callback inputs, exact complete canonical input identity may supply
signature equations after normalization waits. It grants no inverse projection
inference or trait/lifetime proof. Retain the original suspended query while an
input-preserving symbolic result template supplies receiver shape; an unfinished
candidate header prevents uniqueness. Refresh must reconcile every remaining
input pair, and discard unfinished resolution scratch after an unresolved walk.
The [projected callback checkpoint](docs/pa24-projected-callback-results.md)
records ordinary PA24, native PA30 and actual-source PA31 reductions.

Within callable input comparison, progress independent equations past a blocked
projection without inferring that projection's inputs. Share the comparison memo
and retain binder/capture/region discipline; a blocked comparison remains pending.
After signature inference introduces projections, recollect the normalization plan
once and prove every new occurrence. Already resolved projected inputs normalize
before this inference. Committing a selected function item may use those independent
equations, then requires exact complete canonical identity for any blocked result.

Check that identity for each blocked comparison, not for the complete input
tuple. A function item's late-bound inputs take fresh signature regions, so
`&'r S` and the bound's `&'a I::Item` differ as complete types even when their
blocked projections are identical. The independent region equations remain the
caller's lifetime obligations, and any other blocked comparison still waits.
Controls are in the
[abstract callable input checkpoint](docs/pa24-abstract-callable-inputs.md).

Closure capture collection visits a try block's body as a value use: its
from_output operation consumes the tail even when the completed result is
subsequently discarded by a wildcard. Reuse the ordinary iterative use walker;
keep feature authority, source protocol, nested-closure target and complete
ownership proof with their existing consumers. Type inspection alone does not
qualify moves. The [try-capture checkpoint](docs/pa24-try-captures.md) records
actual source execution, Fn/FnMut/FnOnce and bounded field/work controls.

A computed match inside a closure consumes its selected scrutinee, guard and
arm expression facts through the existing capture-use walker. Reuse the typed
match decision, preserving actual callable receiver use and complete ownership.
A captured-place scrutinee uses its selected pattern operations instead;
discriminant reads, wildcards and independent fields must not become blanket
value moves (see the closure capture pattern rule). Required controls remain in
the [match capture checkpoint](docs/pa24-match-captures.md).

A computed `if let` or `while let` scrutinee inside a closure is consumed as a
value use through the same walker; its pattern bindings are closure locals. A
captured-place scrutinee uses its selected patterns, as for a `match`. Controls
are in the [let-condition capture checkpoint](docs/pa24-let-condition-captures.md).

A helper returning an opaque callable may receive its inputs from the caller's
required callable bound. Map the opaque type's formal type, const and lifetime
captures into the owning inference graph as an ephemeral result; reuse indexed
declaration templates, and let one compatible declared callable clause supply
input equations through the existing Probe before projection readiness. Never
infer an associated projection's inputs from its result, read the hidden type or
cache unresolved caller terms; the original selector still proves the complete
trait, formation and lifetime requirements. Pending controls are in the
[helper input checkpoint](docs/pa24-flatten-helper-inputs.md).

A declared opaque callable may refine its capture inputs while a capture that
occurs only in its output is still unknown. Publish those input equations as
partial selection progress and restart the frame. The restarted query still
proves the complete bound. The callable Output is then the opaque's unique
published output equation, instantiated over its inferred captures and related
through the existing Probe after the inputs. This is forward normalization
through the declaration; never invert the output. A statement or assignment
whose type is a pending associated result, such as a `?` output, is not yet
definitely Never, so use its contextual head. Controls are in the
[helper output checkpoint](docs/pa24-flatten-helper-outputs.md).

A method candidate whose implementation header contains a projection, such as
`impl<R: Try> ControlFlow<R, R::Output>`, is normalized as rustc's probe does.
Relate its independent structure to the receiver, and turn each postponed
associated-result leaf into a projection-equality condition. The existing
method trait selection proves that condition under the caller's assumptions.
Never compare an unnormalized projection structurally. Projection-free headers
keep their shape relation. The selected owner binding registers the header's
projection occurrences with the body normalizer before relating it to the
prefix. Controls are in the
[projected inherent header checkpoint](docs/pa20-projected-inherent-headers.md).

A trait implementation candidate relates to its goal after the goal's inputs are
normalized. A projection left in that normalized goal is a rigid normal form.
Relate it structurally, as rustc 1.98 relates a rigid alias: it equals an
unknown or the same alias, never a parameter or another constructor. A header
that needs such an equality does not apply, so it is not an ambiguous
alternative. Read only the recorded rigid normal-form marks. A projection or
opaque result from the impl header itself still waits for normalization.
Controls are in the
[rigid candidate header checkpoint](docs/pa20-rigid-candidate-headers.md).

A caller premise that selects an inferred trait input, such as the `Rhs` of
`X::Item::e(a, b)` under `X::Item: E<Y::Item>`, binds it to the premise's own
projection after the call's inputs were instantiated. That projection has no
head until its ordinary normalizer runs. An early argument expectation treats it
as unknown. A reference coercion normalizes that referent on demand before
choosing dereferences. Never infer anything from the projection's result, and
leave every other input to its existing owner. Controls are in the
[premise input checkpoint](docs/pa20-premise-inputs.md).

A by-value use of a captured place keeps the closure callable by reference when
the captured type is Copy. Derive selections cannot see a source library's own
`impl Copy`, such as core's `NonZero<T>`. In a source library, a captured type
that derives leave non-Copy is classified by selecting the library's `Copy`
language trait under the body's assumptions. Like rustc's copy-modulo-regions
query, this only classifies the capture. It retains no evidence or region
equality, and absent or pending proof keeps the use a move. The body's source
Copy requirements prove every actual copy again. A repeated non-constant array
operand uses the same classification for sema's early acceptance; the body's
source Copy requirement for the repeat still proves the obligation. Controls
are in the [source Copy capture checkpoint](docs/pa24-source-copy-captures.md).

Specialization orders two proved implementation candidates as rustc's
winnowing does. A goal that still has inference variables is ordered only when
both candidates resolved its inputs identically, as core's three
`ZipImpl<A, B> for Zip<A, B>` impls do, so the choice cannot change inference.
Candidates that resolve the inputs differently, or that defer, keep the goal
ambiguous. rustc would still drop a specialized impl whose specializing peer
must apply; that remains a rejects-valid limitation. Controls are in the
[specialized trait input checkpoint](docs/pa20-specialized-trait-inputs.md).

A const closure defined in a const trait's default body is conditionally const
through that body's `Self: [const] Trait` premise. It meets a `[const]` callable
bound through its defining contract: the trait premise and the owner's `[const]`
predicates, proved under the caller's assumptions. A non-const closure in that
body still fails the bound. A const trait member publishes no const-fn call
effect, so eligibility may not wait for one. Controls are in the
[default const closure checkpoint](docs/pa25-default-const-closures.md).

A const fn item whose own predicates include `[const]` bounds, or a member of a
const trait, meets a `[const]` callable bound only when the caller proves its
conditions. Those are the item's `[const]` predicates and, for a trait member,
the trait's `Self: [const] Trait`, bound through the item's callable template,
which includes a member's `Self`. A `#[rustc_non_const_trait_method]` member is
rejected. Controls are in the
[conditional const item checkpoint](docs/pa25-conditional-const-items.md).

A canonical caller premise may bind an inference hole to its complete projected
input type. Use the shared scoped worklist; collect comparisons between projection
results for exact identity or the existing certified-normal-premise relation.
No result comparison may infer the projection's inputs. Generic deferred equations
retain their separate normalization gate and self-dependent-cycle behavior.
The [wrapped callback-premise checkpoint](docs/pa24-wrapped-callback-premises.md)
records ordinary, conditional const, actual-source and native reductions.

An exact declared callable Output premise may retain the checked source closure's
necessary lifetime conditions. Validate the premise first and match the complete
substituted Self using the same typed matcher as body entry selection. Normalize
formal and actual Self, retaining formal normalization in its original coordinates
until matching supplies the caller slots. Rebase private witnesses before restoring
those slots. The source caller proves the whole payload; only a checked caller
may turn it into callee assumptions. Manual instances still pay their debts.
Lift only the empty quantifier from callable sugar through the existing binder
operation; nonempty quantifiers retain their scoped proof consumer.

At a real call, a completely matching expected result may propose only that
callee's fresh late lifetime arguments. Keep rigid Self, type, const and member
identity, reject conflicting slots and nested-local escapes, and publish no
partial proposal after a mismatch. This chooses a sufficient instantiation and
does not infer an inverse law for a projection or equate caller inputs. Check
every actual argument, formation, result and loan requirement afterward. Unknown
expected results use ordinary freshening. Visit shared typed subgraphs once per
binder depth and charge the work to the existing inference budget.

Validate symbolic representation constraints without inventing concrete layouts
for rigid parameters. Keep failed symbolic queries failed and permit unrelated
queries afterward. The pinned packed-declaration restriction follows direct
named struct fields before substitution; instance size/alignment remains a
separate numerical query. A generic callable template is never an executable
body merely because its declaration exists. Concrete lowering requires a
demanded instance with its complete arguments and proofs.

Preserve complete callable identities through function values, constants and
promoted data. Check every generic template, including unused bodies, before
specializing its retained CFG. Substitute parameters by their declaration
identities. Assign function type/const parameter indices from their actual
argument vector after excluding function lifetime declarations; those lifetimes
belong to the separate signature scheme. Aliases and other nonfunction owners
retain their existing mixed lifetime/type/const argument positions.
Abstract lifetimes inside actual type arguments into a fresh instance proof
context while preserving nested binders and the template's own variables.
Reconnect each imported early argument with its original body slot, including
captured owner slots and noncontiguous early slots after late declarations.
Inspect the existing abstraction map; static and unused arguments need no fresh
universal. Drop callables use their checked nominal coverage mapping to reconnect
permuted captures, preserving unused impl proof slots.
Pair each specialized body with its own region proof and complete instance
identity, even when multiple bodies share an original source location.

Normalize retained instance type debts inside their original proof scopes.
Resume each proof at the actual allocation frontier, excluding intervals owned
by other proofs, and preserve its existing universal and private identities.
Group roots once by scope and share a normalization session within each group;
charge both root visits and structural work. Snapshot stable type-debt positions
before appending evidence and update only those positions afterward. Newly
emitted formation and lifetime debts keep their own types, targets and scopes.
Check shared graph width, independent scope width and binder depth separately.

Concrete caller-premise normalization preserves the original quantified clauses.
Share a normalization session within each actual binder-arity group, retaining
every newly discovered formation and lifetime requirement as proof debt. Only the
exact caller-proved premise supplies independent normalized components; a debt
cannot become a given. Collect those components with one multi-root walk per
target, preserving captured and nested binder coordinates. This instance-only
elaboration grants no additional permission to the original generic body.

Demand from a fully proved caller may retain the callee's exact declared trait
premises during projected specialization. Select their validated implementations
in a separate scope after substituting complete arguments. Retain existential
lifetime witnesses as independent constrained universals, with every equality,
outlives and type-outlives condition; keep nested callable binders local. These
givens belong only to that specialized body. A manually constructed instance
has no caller authority and must still discharge the selected implementation's
lifetime debts. Ordinary bodies without projected specialization do not perform
this selection, and lifetime-free predicates need no witness scope.

Quantified declared premises preserve their actual proof scopes during selection.
Eliminate private witnesses through the existing directed lifetime propagation
before importing any outer condition. Crossing a younger universal frontier uses
the solver's ordinary universe rule; an independent local universal cannot become
an outer given. Keep only conditions between original body lifetimes and static,
never arbitrary local witnesses. This conditional summary is not a successful
proof: only a separately established callee declaration premise authorizes its
use as an assumption. Normalization-discovered debts and manually constructed
instances retain their ordinary proof requirements. Unresolved projected tests
cannot be summarized as unconditional scalar relations.

After a caller's complete ownership, lifetime and cleanup proof, retained proved
outlives obligations may establish that an input lifetime equals static. Resolve
selected callable identities through those equalities before discovering demanded
instances. Store only changed targets on that sealed body; dependency discovery
and executable lowering consume the same table. Preserve unrelated free lifetimes
and nested binders. Neither an unproved body nor another caller may supply this
authority. The selected callee still proves its original template and every
implementation requirement. Requirements absent from its declared premises
remain obligations, never new assumptions supplied by normalization.
An empty static-obligation frontier must require no traversal allocation.

Schedule demanded instances iteratively, registering an instance before following
its dependencies so finite recursion terminates. Bound active recursive growth
separately from independent uses of one function. Check retained storage before
cloning a body. Follow actual calls, function-pointer reification and typed
constant allocations; a discarded zero-sized function item does not require its
body. Key promoted backing storage by concrete body identity, and preserve
runtime overflow and bounds checks when materializing its values.

Retain selected implicit tuple/variant constructors during type checking and
build their factory bodies only when referenced. These factories share the
ordinary field-transfer CFG builder. Their generic proof uses the nominal declaration's assumptions and
lifetime contract, with one shared immutable parameter environment per nominal.
Retain owned field transfers and returned-value scopes before specialization.
Prepare generic return fields in the original declaration context as well as
any expression context using free lifetime aliases.

Derived abilities constrain type arguments even behind raw/function pointers;
packed Clone and equality derives impose Copy bounds as well. Owned destruction follows
instantiated fields and skips zero-length arrays. Custom Drop instead retains
every argument's full outlives components, including Copy type parameters and
their reference lifetimes. Symbolic cleanup consumes the same field instances;
it never substitutes for selected concrete destructor methods and executable
recipes keyed by all relevant instance arguments.

Record selected methods, adjustments, casts, layouts and obligations where
later phases consume them. Lowering cannot repeat lookup or inference. Missing
facts are invariant failures, never a reason to fall back to text or rustc.

Declaration-time enum values have an explicit pending query registry, distinct
from finalized aggregate discriminants. Shape constants must demand those queries;
an empty pending registry never means a provisional zero is final. Reuse completed
values and the variant index across argument evaluations. Check all enum values,
including unused duplicates and overflows, before publishing final layouts.
Constant borrows consume retained typed alignment obligations, including dead
branches, without evaluating their places. Layout readiness shares the constant
work stack: complete required enum tags and by-value field layouts before caching
the layout, retaining zero-array alignment and stopping at pointers. Detect genuine
layout/value cycles. Reuse completed layouts and index appended borrow obligations
once; an unrelated enum inventory must not be rescanned for each argument.

Constant eligibility and body construction share lexical temporary-scope analysis.
Keep that analysis below body construction so declaration queries do not depend
on a later phase. Transferred results, static reads and references with static
lifetime extension do not imply destruction. Owned projection bases, update bases
and ordinary zero-length repeat operands retain cleanup obligations. The pinned
stable profile rejects cleanup of partially moved temporaries as well. Qualify
unexecuted branches too. Memoize resolved place roots and destruction requirements;
demand scope analysis only when a temporary may need destruction. Invalidate
absence-of-destruction results if the impl registry grows during declaration work.
Anonymous constants that emit lifetime obligations retain a typed proof body;
the ordinary region, loan and destruction checks must consume those obligations.
They are never emitted as native functions. Scalar constants without lifetime
facts need no additional body. Static lifetime extension supplies permanent
storage provenance independently of promotion, including for a value with Drop.
Annotations elaborated before constant-body inference already belong to that
constant's proof body, including inferred lifetimes subsequently erased by aliases.
Retain this ownership even when inference adds no new lifetime facts.

Constant patterns constrain types during ordinary body inference. Retain pending
value/range/coverage work in flat arenas with the originating file, module and
body and immutable caller environment; finish it after initializer typing in one
shared query session. Reuse resolved dependencies, selected operations and already
computed discriminants.
Cache demanded ordinary definitions and complete associated instances across
these phases; leave ordinary node facts compact and allocate no query
storage for bodies without constant-backed patterns. Do not instantiate a
whole-crate checker per use, repeat initializer typing, or interpret pattern
views. Deferred checks must retain the matching module's visibility context.

Associated pattern paths share expression argument inference and selection facts.
Their values must be known before function specialization. An unused rigid type
or const argument does not prevent a fixed initializer from supplying a pattern;
an actual dependency on that parameter does. Prepare selected dependencies,
including skipped branches, without evaluating skipped arithmetic. A caller's
trait assumption cannot identify an initializer when it retains type/const
parameters or free lifetimes, including `'static`. Fully global bounds can use
their known implementation.
Retain fresh selection obligations under the requesting body's lifetime proof;
cached values never supply another caller's permissions. Translate nominal fields
from the completed instantiated shape while preserving original field identities.

Demanded tuple/array constant values use flat shared arenas. Repeats retain one
element and their length; field/index queries select from that representation
without materializing the repeated array. Translate constants to the existing
selected product/scalar patterns with use-site provenance. Share repeated
product subtrees within a use; retain each projection's distinct place identity
for decisions and execution. Check type-level PartialEq obligations even for
empty products, then validate actual scalar leaves (including NaN rejection).
Reserve expansion work before allocating siblings so pending work is bounded.
Neither value storage nor pattern translation grows dense ordinary node facts.

Nominal constant products retain selected constructor and field identities from
typing; updates fill only missing fields from the shared base value. Validate
constructor callees and update bases even for zero-sized values or completely
overridden fields. Equality obligations are a separate cached type query from
actual structural-value eligibility. Check every enum field for the former and
only the active variant's values for the latter. Derived reference cycles use
an iterative greatest fixed point with reverse dependency edges: publish all
positive and negative results only after propagation completes. Count actual
nodes, dependencies and propagation work under an explicit query budget.

Wide floating literals retain exact target bits through literal, typing and
constant-image facts. F16/F128 conversion uses one bounded rational/sticky kernel
adapted from the frozen CPPGM implementation; existing f32/f64 paths stay direct.
Never round a wide literal through f64. Keep the quad payload in two 64-bit
words so it cannot enlarge ordinary scalar, node, body or LowIR value alignment.
Measure the actual canonical-table cost and preflight bounded limb storage.
Active literals and resolved primitive uses consume genuine crate or token-origin
feature authority. F16/F128 constant division and multiplication share that exact rational kernel:
retain the binary scale in the output exponent, handle subnormal ties and IEEE
special values, and never allocate scratch proportional to an exponent gap.
Finite multiplication preserves the full significand product in at most eight
32-bit limbs before rounding; no host floating intermediate may discard its low bits.
Actual selected source operator and const-call permissions remain required.
A known type grants no other unsupported arithmetic, cast, pattern, wire or ABI
operation. The [wide-literal checkpoint](docs/wide-literals-checkpoint.md)
records exact-bit, enclosing-layout, work and text-contract qualification.

Selected string and byte-string literals retain decoded bytes in one flat,
interned byte arena, addressed by sparse syntax-node facts. Constant evaluation
and body construction reuse those identities without decoding again.
Equality compares exact bytes; identifier Unicode
normalization must never affect text values. Share repeated text subpatterns
within a constant use while preserving each selected place. Byte-string patterns
translate to ordinary fixed-array or exact-slice decisions under the existing
preallocation work budget. String coverage partitions only the current query's
observed values plus one class for every other string. Do not scan unrelated
bodies or enumerate a string domain. Matching a value constant compares type
structure without equating its storage lifetime to the input lifetime; ordinary
expression constraints remain explicit. These pattern facts do not discharge
the region and loan proofs required for reference execution.

Demanded shared constant references use flat place records containing an optional value,
canonical type, access provenance and packed alignment. Separate place queries
from value reads so borrowing a non-Copy static or selecting its Copy field
does not move its container. Check Copy reads before consulting a definition's
cached value. Field/index queries consume selected identities and sparse
autodereference facts; they neither repeat lookup nor materialize repeated
arrays. Preserve provenance across projections and use the actual referent's
alignment after dereference. Borrowing requires alignment, while an owned field
read need not form a reference. Reference chains use bounded worklists for
evaluation, pattern translation and destruction. Keep ordinary node facts
compact; reference operations still require region/loan proofs before LowIR.
Intern static allocation identities before evaluating their initializers. Forming
an address or fixed projection uses typed storage facts; it must not introduce
a dependency on the stored value. Actual reads demand and cache location
contents through the bounded query worklist, after checking access and Copy
requirements. Complete reachable static backing with a growing identity queue
before lowering. Address cycles are valid storage graphs; initializer value
cycles remain errors. Constant patterns demand finite referent expansion and
diagnose cycles independently of executable pointer graphs.
Coverage derives root validity from the selected input place and retains it
through deferred binding checks. Explicit and automatic dereferences cannot
reuse owned-value inhabitation to prune variants with potentially invalid fields.

Floating pattern domains use constant-time bit/rank conversions. Numeric ranks
collapse signed zero, while a separate unordered class keeps NaN visible to
exhaustiveness. Reuse scalar interval indexes and composite coverage decisions;
do not enumerate floating values or add per-value domain storage. Execution
compares the original typed value and captures its original bits. Even a range
covering every numeric rank must test its input because it excludes NaN.

Dynamic-slice coverage retains exact/minimum lengths and typed start/end/rest
projections. Lazily collect tested positions from the current query's selected
roots, keyed by canonical slice type. Specialize only lengths at which a tested
start and end position can alias; keep the remaining length intervals shared.
Canonicalize aliases to one place identity, including nested patterns. Wildcards
and unconstrained captures add no alias points. Charge collection, position pairs,
specialization and graph work to the same bounded query; sort the completed alias
set once rather than using ordered node containers for hot-path membership.

## Rust body representation

A typed control-flow IR retains places, projections, moves, borrows, storage
lifetimes, initialization and abstract drop obligations. It is an executable
body representation, not a second copy of the complete syntax tree. Do not call
it MIR without qualification: CPPGM already uses MIR for machine IR.

Source inline assembly uses the existing parsed syntax and eager expansion
worklist. Preserve actual builtin declaration, template/placeholder spans and
captured operand expressions without rendering or reparsing Rust. Check actual
register, type, place, initialization, Copy, safety, options and feature facts;
an attribute or familiar vector name cannot establish layout or availability.
Keep one cold optional payload in the actual Body and indexed effect/output
operations in ordinary values. Evaluate inputs and output places left to right,
each place once, then store outputs left to right after the effect. Preserve
memory/flags/clobbers and real divergence in the executable CFG. Body cloning,
instance proof and storage accounting consume the same owner and revalidate
effect/output identities. Const eligibility and demanded execution reject
assembly; const and symbol operands need their own real value/relocation facts.
An absent body codec or backend capability diagnoses at that boundary. Never
replace assembly with Unit or ordinary stores. PA41 owns broader closure, but
the selected upstream target's required slice must work by PA32. See the
[assembly checkpoint](docs/source-inline-assembly-checkpoint.md) for implemented
scope, every-Body cost and remaining SIMD, transport and emission requirements.

The required x86-64 XMM consumer transiently joins the fixed configuration baseline
with authenticated actual body features. Consume genuine SIMD array fields;
exact supported lane bits, register availability and Copy are independent checks.
Input, output-only and tied Copy requirements use the existing deduplicated
source collector and the actual caller's completed ownership/region view.
Do not add another policy cache or body field. Baseline availability does not
weaken language call safety or grant body transport/backend capability. The
[vector join checkpoint](docs/source-asm-vector-join.md) records the actual source,
ordinary proof opponents, charged-work observations and remaining boundaries.

A `ymm_reg` operand types as rustc's x86 register table admits it: under
`avx`, a 32- or 64-bit integer or float scalar or a genuine vector of 128 or
256 supported lane bits, with the `x`, `y` and `z` modifiers. The initial
assembly profile rejects it before native lowering and module-body export
("inline assembly ymm_reg operand is outside the initial assembly profile"),
so no imported body carries one. Explicit ymm registers stay rejected by name.
See the [ymm operand checkpoint](docs/pa41-ymm-operands.md).

A `zmm_reg` operand types under `avx512f` as a 32- or 64-bit integer or float
scalar or a genuine vector of 128, 256 or 512 supported lane bits, with the
`x`, `y` and `z` modifiers. A `kreg` operand types under `avx512f` as an 8- or
16-bit integer, and under `avx512bw` also as a 32- or 64-bit integer or
pointer. It takes no modifier. Vector lane widths count `f16` lanes, as rustc's
`VecF16` types do. The initial assembly profile rejects both classes before
native lowering and module-body export, so no imported body carries one. See
the [AVX-512 operand checkpoint](docs/pa41-avx512-operands.md).

Target-feature policy reuses the two declaration/function owners as typed rows,
retaining declared safety, exact contributing source attributes/input trees and
the canonical feature closure. Query authenticated rows without reparsing. A
safe declaration can be called or coerced to a safe pointer only in a compatible
actual body context; explicit unsafe declarations keep that requirement. Nested
functions and constants do not inherit an outer body's features. CPU baseline
features do not waive language call safety; assembly availability is a separate
baseline-plus-body consumer. Unstable feature requests require actual crate
authority, independently of macro allowances. `#[inline(always)]` on any
target-feature function, generic, unused or associated, is rejected at the
hint, after its features, as pinned rustc does. The existing active-call map
retains selected target identity; its measured extra eight bytes per entry also
applies to ordinary calls. No new body field or second call-policy index is
needed. See the [target policy checkpoint](docs/source-target-feature-checkpoint.md)
for exact scope, source mutation opponents, costs and closed transport boundaries.

Nominal SIMD layout reuses prepared substituted aggregate fields through
`Definitions::simd_shape`; it returns NotSimd, Pending or an actual lane/count.
Retain a struct's rustc_simd_monomorphize_lane_limit in sparse, prepaid nominal
policy data, published before concrete layout. The configured-owner pass validates
actual struct target, rustc_attrs request or real attribute-envelope allowance,
name-value decimal string, unsafe wrapper and hard duplicate rules. Zero is a
valid limit; a pending generic declaration need not have an instantiated layout.
Compare the actual substituted lane count only for repr(simd), preserving ordinary
struct layout and all independent lane/element rules. Old metadata must reject
unencoded limits rather than lose them. The [PA41 lane-limit packet](pa41/tests/spec/simd-lane-limit/README.md)
retains real source-provider, original-library-path and imported requirements.

Do not infer vector shape from a source name, add a parallel type cache, or
substitute at this query boundary. Preserve the single array field at offset
zero. Rust arguments and results use the existing indirect object convention.
Qualified C128 uses SSE followed by SSEUP in one vector register; larger and
nested unsupported vectors reject before the generic memory fallback. The
`simd_ffi` gate applies to direct foreign-block declarations under the pinned
rules, independently of layout or target-call permission. Old metadata cannot
publish this representation until its actual codec is implemented. See the
[SIMD checkpoint](docs/source-simd-layout-checkpoint.md) for native evidence
and the exact supported lane and ABI profile.

Retain const-function permission by resolved declaration identity. A const
inherent impl supplies implicit constness only to its actual direct methods;
ordinary impl policy, nested declarations and stable API annotations supply no
constness. Validate the exact const keyword's genuine feature/macro authority.
Keep group const stability distinct from method eligibility and actual caller
proofs. Index public inherent members once by their completed nominal target,
then add their interface edges only when that nominal becomes reachable through
the existing typed public worklist. Prepay the index and visit each edge once.
Ordinary const callers reuse the existing scoped effect facts under one cold
global owner without constructing a compiler Catalog. Their partial attribute
budget pays caller-input work without granting completed source contracts.
Preserve the checked defining body's constant context. Executable lowering
materializes conditionally const function instances in runtime mode; unconditional
const functions and registered aggregate constructors retain their capability.
Classify actual inherited declared clauses, reusing prepared source summaries;
a constructor is no const-function conditional contract. CTFE materializes
separately and retains every concrete conditional bound and reached-call check. See the
[const inherent checkpoint](docs/pa25-inherent-block-checkpoint.md).
Function pointers carry no const-call permission, even when their original
value was a const function or constructor. Eligibility checks consume selected
CFG operations before demanded evaluation. Both conditional branches participate;
source after an unconditional transfer does not. Keep the earlier initializer
eligibility boundary separate from the still-developing general evaluator.

Conditional const closure eligibility retains the actual source role, completed
capture permission and const body identity. Bind the selected instance's defining
parameters and inherited lifetimes in their producer's declaration order. A
defining opaque result can precede its enclosing callable template; that order
supplies substitution coordinates, never executable permission. Ordinary
formation and signature requirements keep their existing producers. Prove the
additional defining const clauses through the existing scoped clause frontier,
retaining all child proofs, quantifiers and region debts before publishing
callable evidence. Share the frontier with async callable conditions and prepay
its work and retained storage. Do not duplicate ordinary constraints as const
selection work or upgrade an ordinary parent's effect.

A local const closure's source body can finish after parent call inference.
Retain the parent's suspended queries and unpaid goals while the existing flat
nested scheduler completes its remaining children. Retry that frontier once
after child publication, charging the repeated goal scan. Every parent trait,
const-call, lifetime and ownership check still runs before final publication;
child completion grants no assumption. Keep executable formal-order and actual
receiver checks separate. The [adapter checkpoint](docs/pa25-const-adapter-reductions.md)
records independently checked defining and execution requirements.

Constant cleanup eligibility consumes the already selected destruction schedule.
A possible destructor access is forbidden, including an abstract generic drop;
transferring an owned result does not destroy it. Reuse the initialization and
partial-move proof that selected that access, rather than rejecting every type
that could need destruction. Preserve the distinction between cleanup access
and the original type's separate drop-liveness requirements.

An implicit receiver dereference can create a place with no source expression
carrying its exact type. Retain its selected referent as a storage-preparation
root before wrapping it in an automatic borrow. Share the existing field-root
worklist and completed instantiation cache; repeated receivers must not repeat
substitution of their whole field list.

Reference coercions can retain actual source Deref/DerefMut calls separately
from the expression's original method, field or call operation. Prefer an
available built-in reference conversion or unsizing; exploring an unused source
Deref path cannot impose its conditional bounds. Only actual selected calls pay
Deref/Target, const-effect, formation, argument and lifetime requirements. Reuse
existing charged discovery, call instantiation, field/access steps, capture and
region proof. The last returned reference supplies ancestry for the final borrow;
a destination type or associated Target supplies no permission. Lower the operand
and each actual call once, including temporary cleanup and argument order.

A known result expectation supplies a shape hint before generic call inputs
when the unresolved result root reaches the actual formal input graph. Visit
shared input nodes once with charged work, excluding non-injective projections;
a result-only Self cannot supply an earlier argument prefix for trait selection.
Preserve existing function-pointer result context and later ordinary result
coercion. The hint cannot replace a source constructor's incompatible nominal identity or its
field inputs with a later Deref destination. Preserve compatible factory/unsizing
context and source-time prefix inference: a later argument alone does not rewrite
an already inferred wrapper input. The [reference coercion checkpoint](docs/pa26-borrow-coercions.md)
records independently checked contexts, const effects, captures and real runtime/
CTFE traces. Ordinary bootstrap, complete composition and transport remain required.

Share live storage tails persistently. An exit records the suffix it ends,
without copying the surrounding live-local list or repeating every storage-end
statement at every failure branch. Preserve reverse lifetime order and skip
already-closed nested scopes. Storage compaction is distinct from ownership
proof complexity: charge the paths/words actually visited and retain explicit
dataflow memory and work budgets.

Compute destruction requirements from canonical types with a bounded iterative
query, memoizing shared shapes and errors. References do not own their referents;
zero-length arrays do not require element destruction. Array length must not
expand query work. Programs without custom destructors need no pending-drop
metadata in the current closed type profile.

Prepare nonliteral array-repeat counts in the existing annotation traversal,
using the exact owning generic/region context and ordinary typed usize query.
Body inference consumes the retained source-keyed length; standalone const
parameters retain their symbolic array form. Share the checked callee CFG across
distinct count queries while charging and retaining each query's actual work
and code. Do not add another source scan, infer a count from rendered text or
use the declared array type as permission to skip the repeat expression's proof.

Pending body facts retain destructor identities, type requirements and candidate
store/storage exits. Reuse persistent live-stack ranges and existing exit IDs;
do not copy every local at every exit. These candidates can overlap and are not
an execution plan: initialization, moves, owned rvalue scopes, ordering and
destructor-induced liveness still need proof. The ordinary ownership, region
and loan entry points reject pending elaboration. The destruction
verifier discharges those obligations and attaches immutable selected plans and
glue recipes to checked bodies; only that completed proof can authorize cleanup
lowering. Raw body views may expose pending work explicitly. A value needing
destruction cannot be promoted to permanent storage, even while its cleanup
plan remains pending.

Promotion eligibility and evaluated values are distinct facts. A generic
temporary can qualify for permanent storage while its value awaits concrete
arguments. Retain that eligibility against the original typed value DAG;
storage, region and cleanup proofs consume it without inventing bytes. Unknown
repeat lengths still require eligible operands, and potentially failing division
or value-level indexing cannot become eligible merely because an operand is
symbolic. Preserve runtime effects and checks around promoted roots. Concrete
verification rejects unresolved promotion values; demanded instantiation must
evaluate them before executable lowering.

Retain destructible rvalue creation scopes while building the body, including
discarded values that never become borrowed places. Forwarding a block result
forwards its temporary scope without inventing a second object. Pattern captures
use their retained binding scope. Mark returned values as transferred before
local cleanup, preserving order relative to storage exits at the same point.

Initialization has a distinct may-be-initialized plane; immutable-binding write
history cannot substitute for it after moves. Share the existing transfer and
fixed-point implementation, with additional state retained only on demand.
Expose bounded point queries over retained entry states for cleanup planning.
Preserve sparse array element/gap partitions from move-path numbering when that
analysis requests them; never reconstruct these mappings from output or expand
unselected array elements. A whole-path mixed state can include partial moves
and does not authorize dropping the entire object behind one flag.
A fresh genuine enum test can exclude an inactive payload from possible symbolic
cleanup without changing must-initialized read/move permissions. Authenticate the
actual nominal owner and variant/field identities, require the test immediately
before its branch, and index sparse descendant intervals once. Cached booleans,
intervening writes and custom destructors confer no such exclusion. Preserve
original type liveness and concrete cleanup/CTFE restrictions. Untouched fully
initialized aggregates retain their cleanup fast path; do not replace it with a
recursive field scan at every exit.

Retained initialization also tracks destructible rvalues that never become
places. Derive their consumption once from typed aggregate/call operands,
constraints, stores and recorded returns. Share the existing solver and inspect
each consumed source before clearing its state. Preserve immediate disposal of
an ordinary zero-length repeat operand separately from transfer and constant
repeat instantiation. Retain the constant-path selection from type checking:
runtime coercions can wrap a constant, and blocks can forward its value without
being constant repeat operands. Neither array length nor text reconstruction
may expand this work. These consumption facts do not substitute for scope cleanup or its
destructor-induced loan and region uses.

Scope-aware initialization reuses the validated storage event index. Retain one
ordered owner list per scope, merging local declaration and temporary creation
order; traverse shared ancestor ranges at exits instead of copying live lists.
Merge return transfers with storage events at equal CFG points. Each successor's
cleanup starts from the same outgoing state. Scope and initializer endpoints
clear owned initialization, so loop reentry cannot resurrect an earlier value.
Materializing an already-owned value must not initialize it again.

Expose ordered cleanup candidates before their state changes, including scope
exits, explicit storage ends, replacement and discarded repeat operands. Overlapping
endpoints observe the state left by earlier ones. Borrowed and dynamically indexed
replacement destinations do not claim their containing pointer or array's move
path. These candidates still require field/array glue, conditional flags and
destructor-induced liveness before they become executable cleanup.

Constant cleanup consumes those same completed plan flags and actual Flags events.
Map each guard into that plan's exact domain and retain the complete active-call
inventory independently of its dynamic bit. Initialization bits belong to one
cold frame cursor bound to its actual instance, Code and once-bound footer; entry
initialization comes from real events, and loop edges never reset the bits. False
guards skip only the call, after actual target/header/State authentication. Reuse
the normal selected method, loans and owned child Unit continuation for true
guards. Prepay cold cursor/word storage and bit work before growth or mutation,
keep the ordinary frame footprint unchanged, and release storage once with its
owning frame. A flag supplies no const, Copy, destructor or loan permission.

Generic cleanup reuses the initialization replay and ordered event schedule.
Select possible destructor access separately from original-type drop liveness:
field moves can remove every runtime call while retaining an aggregate's lifetime
obligation. Abstract plans contain no native glue and cannot authorize lowering.
Scoped drop queries preserve opaque owned type parameters as well as named
lifetimes; Copy assumptions suppress only the exact proved subject. Ordinary
coerced operands retain their input components until consumption. Unknown-length
constant repeats instantiate elements; ordinary repeats require Copy and cannot
supply an owned destructible prototype. Do not guess whether a symbolic length
is zero. Sparse move partitions, bounded iterative queries and shared component
lists apply to these proofs as they do to concrete cleanup.

Internal match-guard views retain selected place identity and delay owning
captures until success. Protect tested places without runtime loads or address
formation for inactive variants. Track guard protection liveness across control
flow; a diverging path does not imply a later guard use. Internal view proofs
are distinct from the pending loans and regions of user reference values.
Build guarded pattern retries with shared first-match and resume continuations,
not an expanded Cartesian product. Preserve written alternative/field order and
end attempt-local storage on failure. Static path offsets may be cached as typed
recipes, but addresses must be formed where they dominate their actual uses.

After region and loan proofs, sized references lower to thin pointers and their
lifetime binders have no runtime representation. Collect address-observable
places sparsely. Materialize borrowed scalar temporaries once at their defining
evaluation, preserve object snapshots, and use persistent backing for promoted
roots without deleting their original effects or runtime checks. Borrowed
zero-sized values require aligned, nonnull backing; their addressable fields
retain canonical offsets even though value loads and parameters remain erased.
Reference pattern paths share persistent dereference prefixes alongside ordinary
offset recipes. Load each pointer in the current use block, restoring referent
alignment after dereference. Representation-preserving lifetime conversions may
reuse values; thin-to-fat coercions must construct metadata explicitly.

The course x86-64 representation for references and raw pointers to slices or
`str` is a 16-byte, eight-aligned object: a data pointer at offset zero and an
unsigned 64-bit length at offset eight. This is an explicit course ABI, not a
claim of a stable Rust ABI. Slice length counts elements, including zero-sized
elements; string length counts UTF-8 bytes. Unsized referents themselves still
have no fixed storage layout. Use the existing object calling convention for
wide values and retain metadata sparsely beside unsized place projections.
Array-to-slice conversions use the checked source length and compatible element
layout. Dynamic indexing checks that length before address calculation, even
when the selected value is zero-sized. Reborrows retain the source metadata.

PA22's metadata query follows the final field of a completed struct or tuple.
References, raw pointers and arrays stop that traversal. Its typed result is
thin, length-bearing, or unknown with the unresolved tail identity; declaration
bounds can establish a thin rigid tail without inventing its size or alignment.
The query consumes prepared field instances and normalized concrete storage,
memoizes shared suffixes and failed paths, and bounds work without recursive
host calls or whole-cache failure scans. It must not reveal an opaque hidden
type to justify public bounds. Metadata classification does not prove source
well-formedness, an unsizing conversion or a caller's `Sized` requirement.
Keep those proofs separate from the PA22 source rules.

Actual source raw-pointer composition authenticates the complete direct-call
instance and retains ordinary source associated-type obligations. Query the
existing ABI metadata owner to distinguish Thin/unit and Length/usize; pointer
width alone is insufficient. Evaluate both arguments, including erased unit,
without pointee reads or new access authority. Reference/access consumers retain
the existing sequence validity owner. The
[composition checkpoint](docs/pa28-raw-pointer-composition.md) records the source
sizing prerequisite, independent disagreements and remaining original wrapper,
dynamic metadata and transport boundaries.

Nominal sizedness follows declaration schemas, including full mixed argument
slots and projected storage conditions. Seal these schemas after fields complete;
reserved nominal identities must not permanently claim `Sized`. Retain the
original scoped demands for final checking when declaration proofs use
provisional size facts.
Shape queries preserve one frontier across a size dependency. An internal
failure releases that frontier while keeping completed independent results;
an invalid resume request must not discard a legitimate pending query.

Hidden projected `Sized` requirements descend on the requesting trait search's
own work stack. Carry the child selection's scope, allocator frontier and complete
evidence into the parent result. A separate synchronous sizedness query must not
invent a new caller lifetime namespace or discard the projection's debts.

Related canonical roots may share completed concrete trait proofs within one
fixed environment, implementation availability and builtin-retention mode.
Export a proof only when its entire selected graph is free of private witnesses,
lifetime constraints, retained builtin requirements, provisional sizes and
assumption-selected branches. Preserve the ordinary proof path otherwise.
Cache completed child proofs as well as roots; caching only roots still repeats
every suffix of a nested requirement chain. Check completion once when adding
each proof node rather than rescanning descendant evidence for each export.
The enclosing normalizer's delegated input authority is not a reusable root
proof. Member-parent selections stay local until their formation evidence is
attached. Body checking can retain eligible completed facts across normalization
sessions while each session continues to own its distinct lifetime evidence.

Built-in struct unsizing changes only parameters confined to its final field.
Several type or const parameters may change. Normalize and prove the actual
field conversion; associated-type marker arguments need not themselves unsize.
Follow one storage tail rather than expanding both marker and field proof trees.
Check ordinary argument compatibility without rescanning the same deep argument
at every wrapper. Projected field equality is independent of subtyping the outer
reference. Keep original reference ancestry and regions in the source proof;
mutable conversions borrow the original sized place before building metadata.

Explicit reference `as` casts for arrays and same-owner nominal struct tails
reuse the ordinary coercion and reborrow selection. Final nominal validation
requires its exact target, distinct source/target referents and retained terminal
conversion at the operand. Matching pointer metadata grants no conversion.
Keep original place/region/loan proof, evaluate the operand once and consume the
selected slice or object conversion during lowering. The
[nominal reference cast checkpoint](docs/pa22-nominal-reference-casts.md) owns
PA22 slice-tail reductions and PA23 object-tail counterparts; actual source
Unsize nominal-tail selection remains PA31 work.

Retain the terminal source array identity as a typed unsizing selection.
`Rvalue::Unsize` carries that identity through generic substitution, instance
normalization and lowering. The normalized physical field supplies the length,
which can differ from an array used as a marker argument. Lowering consumes
the selection and never reconstructs eligibility from matching pointer widths.
Contextual branch results select the declared DST destination before joining
different sized inputs. Borrowed constructors preserve element expectations
while inferring the source array length instead of constructing an unsized value.

Unsized field addressing needs prefix offsets and tail alignment, not an invented
fixed size for the complete object. Cache these partial layouts separately from
sized layouts and forward metadata through final-field projections. References
and pointers stop storage-tail traversal; recursive pointer prefixes must not
become by-value layout cycles. Physical classification may use completed
normalized storage but cannot grant a caller a semantic proof.

Raw pointer conversions erase only the outer reference loan; pointee region
obligations still use a pointer-shaped proof view. A mutable reference first
creates an immediate reborrow of its original pointee, preserving ancestry and
place permissions. Keep these adjustments sparse and clear them per body.
Prepare the exact pointee storage roots before finalizing cast metadata, including
early constant bodies. Explicit raw casts may reinterpret pointees; implicit
coercion variance is not an extra restriction on those casts.

`Rvalue::Address` checks initialization, mutability and access conflicts but
creates no loan or alignment promise. Direct raw-dereference address syntax
selects the original pointer without a memory read. Lower address casts through
existing equal-width pointer/integer copies and integer extension/truncation;
wide-to-thin casts select the data component. Pointer comparison includes length
metadata, with address-first lexicographic ordering. No backend vocabulary change
is needed.

Tail layout consumes the same completed packing declaration proof as sized
layout. Numeric alignment propagation through arrays/slices is not proof of
forbidden direct nominal containment. Raw address formation does not pay the
separate alignment obligation of a reference.

Literal backing is immutable and shared by canonical bytes. Empty strings and
byte strings receive nonnull backing with zero logical length. Persistent wide
values serialize both the address and length at their canonical offsets;
packed fields use the existing byte serializer and pointer startup fixups.
Metadata and literal identities must not enlarge ordinary scalar place facts.

An unsized constant pointer retains its selected length with the pointer value.
Its backing location keeps the original sized type, allocation identity and
physical field types. Subobject projections reuse that backing and forward tail
metadata; unsizing itself does not allocate a replacement object. Constant
evaluation and promotion consume the same typed array selection. Generic
promotion retains a deferred value until the selected length is concrete.

Absolute constant pointers retain address bits separately from optional length;
byte-backed raw pointers retain an explicit selected length rather than recovering
it from the literal. Unsizing can attach metadata without allocating storage.
Discarding and rebuilding metadata must not recover a stale array/string length.
Neither absolute address bits nor a constant backing identity authorizes constant
pointer comparison or pointer-to-integer casts, even in an unevaluated branch.

Dynamic slice patterns test exact or minimum length before selecting any element.
Suffix projections retain the tested minimum length as a typed fact; rest
projections retain excluded prefix/suffix counts. Loan overlap uses those facts
to distinguish disjoint pattern captures. An arbitrary runtime index does not
receive the same disjointness proof. Normalize nested rest projections without
enumerating elements. Pattern metadata reads are distinct from element reads
and moves, and retain initialization, storage and liveness obligations.

Ordinary and guarded decisions consume the same selected constructor conditions.
Guard retry recipes rebuild both addresses and lengths in their actual use
blocks; cached recipes must not cache branch-local runtime values. String tests
compare exact interned bytes after checking length. Emit a bounded-size comparison
loop instead of one instruction sequence per literal byte, and never load bytes
on a failed length test. Constant text patterns obey the same execution path.

Borrow checking runs before references and moves lose their source semantics
in LowIR. Region/dataflow analysis uses typed worklists and bounded lattices,
with per-body storage and complete dependency edges. Pattern usefulness and
trait solving must expose work counters and explicit complexity limits.
PA15's region inspection exposes the retained directed contract graph separately
from PA16's checked loan inspection. Render deterministic IDs, direct edges and
sparse point/loan words; never use inspection text as proof input. Optional loan
snapshots observe the final checked transfer pass, after pruning and call
activation. Executable verification must not allocate those snapshots or retain
the per-body inspection arenas. Counts describe the same facts as the full view.

Retain explicit whole-body runtime extents compactly. They mean exactly the
caller's seeds at every CFG point, not a permission inferred from a region kind
or an assumption. Propagate their closure along the proved directed requirements
before ordinary sparse seeds; the contract-only LOCAL sentinel never connects
runtime regions. Bound retained full-region entries, sparse words and pending
work. Membership and logical word counts must match expanded point propagation,
including empty bodies and partial final words. Sort the compact representation
only for inspection, then expand full extents lazily while rendering; never
recreate a region-by-word matrix just to print the view.

Destruction consumes initialization/move facts to elaborate conditional drops.
Async transformation accounts for borrows, pinning and cancellation cleanup.

## Crate calling-convention transport

Method-dispatch policy is an original trait declaration fact. Retain the actual
configured owner, source image, full trait header, bound definition and both
array/boxed-slice bits in the existing cold header promises. Parse its word list
once in the shared attribute walk, with genuine feature/macro authority and
prepaid storage. Both checked absence and presence require a current trait;
unknown imports never become absent policy. Older export profiles reject intent
they cannot transport. The method probe must consume the adjusted receiver
and method-name token's defining edition, independently of trait Self and the
whole call's span; qualified paths remain separate. Genuine Box identity cannot
be inferred from a nominal spelling. The [declaration checkpoint](docs/pa34-dispatch-declaration-checkpoint.md)
records the qualified producer and required receiver/edition/metadata closure.

Retain forced non-Rustic argument passing as typed nominal representation.
Authenticate actual struct, word, duplicate, feature/macro-envelope and
representation-conflict rules during the existing owner walk. Summarize actual
transparent payloads in the optional completed-layout cache with prepaid work
and storage; absent declaration families allocate no policy cache. Cache argument
overrides separately from normal return classification. Keep the original type
identity instead of expanding common passing records with a second layout.
Lower forced arguments to pointers to fresh owned snapshots and copy incoming
values into genuine callee storage. Preserve Copy/move/drop/loans, call safety,
Rust convention and independent return rules. An all-scalar/pointer physical
boundary uses the existing scalar LowIR signature; source callable identity still
selects conversion. This pointer mode is distinct from SysV MEMORY stack byval.
Reject the policy in older metadata profiles before erasing its intent. See
[the source/native checkpoint](docs/pa30-indirect-arguments-checkpoint.md) for the
qualified local family and required transport/runtime closure.

Source body transport must retain every raw input consumed by its original
proof, including CFG-derived local-borrow upper bounds and reborrow ancestry.
Share the existing typed place-bounds producer between the ordinary proof
adapter and export; do not duplicate its algorithm or serialize solved regions.
Run it once per authenticated collected original, charging traversal, emitted
rows and transient capacity growth before allocation. Repeated signature
requests use the retained recipe frontier rather than walking the CFG again.
Missing or substituted genuine inputs remain structural validation failures.

Crate metadata retains a fixed callable's ABI independently of its safety,
linkage and lifetime slots. Import into the existing typed callable convention;
never reconstruct this identity from a symbol spelling. Select the foreign
profile only when at least one exported callable has C ABI, and preserve the
exact wire bytes of older Rust-only profiles. The foreign profile explicitly
selects signature-local or shared nominal graph ownership. Validate the selected
graph once and reuse indexed type remapping; an ABI bit never duplicates the
graph or grants permission to call a const or generic body.

Profile eight adds an outer variadic envelope only when a published signature
actually uses C variadics. It composes in canonical order with public-name and
shared storage/type profiles; fixed signatures retain their exact old bytes.
Use a typed Rust/fixed-C/variadic-C convention so Rust variadics cannot be
constructed. Reject variadic bits outside that envelope, unknown bits, invalid
ABI combinations and redundant envelopes before importing the graph. Publishing
a real bodyless foreign declaration retains its checked import registry entry,
exact external symbol and safety; it never creates a local definition or const
execution permission. Imported calls retain ordinary promotion, argument-count,
ownership, lifetime and loan checks. See [the variadic metadata checkpoint](docs/pa30-variadic-metadata.md).

Unknown conventions, noncanonical ownership tags and a redundant all-Rust
foreign profile fail at decoding. Existing body, lifetime, ownership and
layout checks still precede artifact publication. A calling-convention extension
does not imply support for a new exported type, static or body contract.

Named storage has its own metadata profile and reuses the shared type graph.
Retain real external static identities, visibility, symbols and mutability;
distinguish owned completed initializers from foreign declarations and their
safety. Discover private backing only through completed values, without making
private names accessible. Register imported declarations before materializing
values so forward static addresses and self-reference cycles preserve identity.
Typed postorder value nodes may refer to named statics by field-ordinal paths;
the aggregate envelope also retains typed array-element steps with u64 indices.
Keep repeated array elements as one child edge and byte arrays as one bounded
payload. Array extents describe types, never decoder allocation counts. Validate
each field/element transition against its parent type and exact final referent;
do not reinterpret an array index as a field ordinal. Byte offsets and anonymous
allocation graphs require separate explicit support.
Validate roots, targets, field types, reachability and combined quotas before
allocating local IDs. Memoize discovery and value encoding across shared roots.
Imported owned storage still uses its original global symbol. Completed immutable
contents can satisfy constant reads; mutable and foreign contents retain their
ordinary restrictions on every demand, including raw access and cached backing.
Metadata supplies no invented syntax, initializer bodies or generic execution
permission. Older profiles retain their exact bytes when no named storage is
exported.

The explicit PA29 `module-branches-v12` source profile is `module-nominals-v10`
plus typed two-way `Branch` and checked `Panic` original CFG exits. A Branch
names a value and two successors of its own body; each fresh consumer re-proves
the bool condition, value dominance, initialization, moves, storage, regions and
drops per instance. Panic has no unwind or cleanup edge and lowers to the shared
abort exit. Earlier profiles and the version-eleven compiler-operation profile
reject both exits; a branch-free version-twelve export differs from its
version-ten bytes only at the version byte.

The explicit PA29 `module-bodies-v13` source profile carries the version-twelve
vocabulary as stable generic typed body IR nodes: one code per Statement,
Rvalue, Terminator and projection form in body IR order, an explicit arity, and
self-describing operands that reference the type, declaration, application and
region tables by index. It adds closed primitive comparison and bitwise Binary
values. A consumer rebuilds only the forms it can prove. Every other allocated
form, operator, operand kind or flag is rejected by name and never skipped,
erased or approximated. A wider consumer may accept more forms without a new
version. Supported rows keep version twelve's validation, owner/remap authority
and per-instance proof. Their work and storage are linear in the decoded rows.
Earlier profiles reject Binary values and keep their exact bytes.

Version thirteen also carries imported language roles and trait parameter
defaults. A role without a compact ID is written as escape 255 plus its
language-item spelling. Unknown spellings, and spellings that replace a compact
ID, are rejected. The spelling selects the role only; consumers bind it to the
row's declaration address, never to a name or path. The Future family and
`Send` keep version eleven. A provider re-runs the source operator contract
before exporting an operator role. A fresh consumer admits an imported operator
role only after re-authenticating its header, member and signature, and rejects
every other imported role by name. A defaulted type slot (tag 2) is admitted
only on a trait-owned slot whose default is that trait's `Self`. Binary flags
9-11 are checked Add, Sub and Mul: the consumer re-proves typing and lowers its
own overflow guard to the shared abort exit. No overflow decision is
transported, and flags 12-15 stay rejected by name. A direct function-item
formation is the owning request port of its application, and every
well-formedness root is demanded. Profiles without these rows keep their exact
bytes.

Version thirteen also carries declared associated results. A trait's result
is a kind-9 declaration with version eleven's header row. An implementation's
definition is also a kind-9 declaration; its header is a definition row
`(declaration, value)` whose value is a type recipe in the impl's own frame,
linked after the impl's method links by its trait member. Definition rows
follow implementation and link order. The result, definition and projection
columns precede the optional const-body trailer exactly when the declaration
table names a kind-9 declaration. A result admits no own inputs, formals or
region captures; its canonical projection is `<Self as Trait<P..>>::Member`
over the trait's own type formals, and its bounds are direct promises naming
no trait argument. Body ports compare projection types structurally. No
normalization, selection or conformance result is transported: a fresh
consumer binds each definition to its trait member, re-proves its formation
and bounds in the rebuilt impl environment, checks the impl's methods against
the definitions and normalizes every projection itself. An imported by-value
operator role is admitted with exactly its `Output` result and its `(Self[,
Rhs]) -> <Self as Op[<Rhs>]>::Output` method; indexing roles stay rejected by
name. Quantified normalization evidence that binds no region and records no
gap or relation is its flat evidence. Earlier profiles reject associated
results by name, and packets without kind-9 declarations keep their exact
bytes. A consumer's own operator expression resolves an imported role only
through its validated declaration identity, never a spelling or path, and
re-runs the same imported operator contract; the role's parameter defaults
arrive complete with its header. Without an imported or own role, the
expression stays rejected by name.

Version thirteen also carries the callable roles. A provider may declare the
`tuple_trait` marker, `FnOnce<Args: Tuple>` with its `fn_once_output` result
and rust-call `call_once`, `FnMut: FnOnce<Args>` and `Fn: [const]
FnMut<Args>`, each with exactly its header, Args parameter and call member;
the conditional effect is admitted only on these roles' own clauses. Type
recipe tag 9 is a non-unit tuple, and requirement tag 4 binds a declared
result (`ProjectionEqual`). Body IR node 29 is a call operator with its call
trait, caller-side signature and arguments, and borrow flag 1 is a two-phase
borrow; unique borrows stay rejected by name. Only the vacuous binder that
parenthesized sugar opens is lifted. A fresh consumer re-authenticates every
imported role and call method, and re-proves each call operator: the receiver
form of its call trait, its untupled inputs, its result as the callee's
declared `Output`, and the callee's call-trait obligation in the rebuilt
environment. No provider selection or proof is trusted. Closures inside
provider bodies and closure call entries stay rejected by name. Packets
without these rows keep their exact bytes.

Version thirteen also carries fn-item reification to closed Rust fn pointers.
Body IR value node 23 reifies a preceding Function value of function-item type.
The node's own type is the target fn pointer row, and no coercion decision is
serialized. A fresh consumer re-proves the item's applied signature against the
pointer, as a client's own `let f: fn(..) = item;` requires. Convention and
variadicness are equal, and `unsafe` may be added but never removed. The
pointer binds no region, so parameters and result compare exactly. A pointer
over a type parameter requests its own formation. The consumer then links or
instantiates the item as a direct call would. A reified constructor needs no
shim: each consumer defines its own constructor instance. Export rejects these
by name: higher-ranked and C pointers, closures, tracked functions and
explicit casts. Earlier profiles keep the generic rvalue text, and an older
reader rejects code 23 by name. The [reified-item
packet](pa29/tests/spec/reified-items/README.md) records independent outcomes.

Version thirteen also carries calls through closed Rust fn pointers. Body IR
value node 30 has a second shape, `(30, [Type, Value, Range, Flag])`: the call
signature, a preceding callee value of fn-pointer type, the arguments and the
call's lexical safety, coded as a request's. The read matches the whole
operand list, so a direct call keeps its three-operand shape. A pointer call
selects no target and owns no request row; its site is its one evaluation.
A fresh consumer re-proves the signature against the callee's pointer type:
equal unsafety, convention and variadicness, no bound region, one parameter
per argument, and equal parameter and result nodes. It re-proves the safety
with a direct call's matrix, at validation and over the rebuilt body, then
calls the pointer value in its own lowering. A safe callee records Safe, in
an unsafe block or not. Export rejects these by name: higher-ranked and C
pointers. Earlier profiles keep the direct-call text, and an older reader
rejects the node as malformed operands. The [pointer-call
packet](pa29/tests/spec/pointer-calls/README.md) records independent outcomes.

Version thirteen also carries zero-slot fn fields, with no wire change. A
struct's or variant's field whose type is a closed Rust fn pointer, or
contains one, travels as a callable-type binder that the nominal owns, with no
parent and no slots. Only the version-thirteen profile admits such a binder;
earlier profiles reject it as they did before. A fresh consumer forms the
field's fn type in that nominal's own header frame, and a called constructor's
payload in its own nominal's; the frame's owner must be exactly that nominal.
A call signature's callable-type frame must belong to a function, so a
nominal's frame never types a call. Export keeps rejecting higher-ranked,
early-region and C fn fields by name. An older reader rejects the binder by
name. The [callable-field
packet](pa29/tests/spec/callable-fields/README.md) records independent
outcomes.

Version thirteen also admits unsafe associated functions, with no wire
change. An inherent method, a trait member or an impl member may be
declared `unsafe`; its constness, convention and variadic terms keep their
fences, so an unsafe `extern "C"` method is still rejected, and a default
body keeps the provided-body fence. Its original and public safety must
agree ("source associated callable changed its actual original/public
safety"), and an implementation member's safety must be its trait member's
("source implementation member changed its trait member's safety"). Direct
and pointer calls inside or to an associated function follow direct Call's
matrix: an unsafe call needs an unsafe block or an unsafe caller, and a safe
call claims no obligation. Each consumer re-proves both rules at
validation and the matrix again over the rebuilt body, and its own calls of
an imported unsafe method need an unsafe context. Earlier profiles keep
"source nonordinary callable effect/safety/convention requires a wider
profile" for the declarations and "source unsafe associated Calls require
their wider actual profile" for the calls, and an older reader rejects a
version-thirteen packet by the same texts. The [guarded-method
packet](pa29/tests/spec/guarded-methods/README.md) records independent
outcomes.

Version thirteen also carries named constants of primitive and unit types.
Declaration kind 13 is a constant, parented by a module or a function. One
header row per constant follows the enum columns, in declaration order: its
type and its completed value, written as a const term. Body IR value node 17
reads a constant of the same artifact. No initializer travels. A fresh
consumer re-proves each value's canonical bits for its declared type, requires
each code-17 read to have its constant's type, installs that type as the
declaration's cell and seeds the value as a completed definition, as it does
for legacy imported scalar constants. An unnamed `const _` exports no row.
Export rejects these by name: aggregate, reference and pointer values,
associated constants and statics. Earlier profiles keep the declaration-kind
text, and an older reader rejects kind 13 by name. The [named-constant
packet](pa29/tests/spec/named-constants/README.md) records independent outcomes.

Version thirteen also carries enums. Declaration kind 10 is an enum, whose
nominal header has no record fields, and kind 11 is one of its variants,
parented by the enum and published once: a unit or tuple variant as its
value-namespace constructor, a record variant as its own type-namespace
definition. Exactly when the declaration table names a kind-10 declaration,
enum columns follow the associated-result columns: headers `(enum, tag,
variants)` in declaration order, where tag 0 is the default representation
and tag 1 is followed by an integer scalar; variants `(declaration, ordinal,
shape, discriminant, fields)`, where shape 0 is unit, 1 is tuple and 2 is
record and the discriminant is 16 little-endian bytes; and payload fields
`(variant, ordinal, type, site)`. Exactly when some variant row is a record
variant, a name column follows the payload fields: each record variant's field
names in variant-row and ordinal order, with no count, as their ranges imply
it. Ordinals are dense, ranges cover the columns contiguously, discriminants
fit the tag (or `isize`) and are unique, a data enum without a tag keeps
ordinal discriminants and a unit variant has no fields. A record variant's
names are canonical path components, unique within the variant; a positional
field has none. The shape alone fixes a variant's namespace: a consumer
installs a record variant in its enum's type namespace, requires a re-export
of it to bind the type namespace, gives it no value cell and never calls it.
Body IR node 14 is a `bool` constructor test `[place, variant]`; node 3 with a
second variant operand is a variant aggregate; cast flag 1 converts a
fieldless enum to an integer; a field projection whose owner is a variant is a
downcast field.
No layout, niche or match proof is transported: a fresh consumer re-derives
each enum's layout and nullable policy, re-checks every variant against its
installed enum, and over its own rebuilt CFG proves each downcast use by a
live constructor-test fact and each reachable unreachable exit as following
an impossible variant path or a never value.
Negative discriminants and `must_use` enums stay rejected by name;
lifetime-generic enums follow the nominal lifetime rule below. Packets without
enums keep their exact bytes, as do packets without record variants. An older
reader rejects shape 2 with "unknown source variant shape". The
[record-variant packet](pa29/tests/spec/record-variants/README.md) records
independent outcomes.

A version-thirteen function application may also name a tuple variant: a
called constructor, with the same `Function` value, `Call` value and call
request as a function call and no new node. Its type formals, premises and Self
are its enum's. Its signature is `fn(payload..) -> Enum`, safe, Rust ABI and
without a late binder, and its request carries no opening and no method. The
signature's parameters, the call's arguments and the variant's payload have one
length. A unit variant is never called. Constructor bodies are synthesized,
never exported originals. A consumer registers each imported tuple variant as a
callable over its enum's formals and binds each imported variant name once in
its enum's member scope; its own paths, calls and patterns resolve through that
binding. It synthesizes every constructor that its own or a rebuilt body
references from the re-authenticated enum header.

A version-thirteen module binding may also re-export a variant, as core's
`prelude::v1` does with `pub use crate::option::Option::{self, None, Some}`.
The binding is an ordinary module binding row in the value namespace whose
target is the kind-11 variant declaration; there is no new row or tag. Rustc
also binds a tuple or unit variant in the type namespace. That alias is not
emitted, just as the enum's member scope omits it, and only a type-namespace
alias beside the same variant's value binding may be dropped. A consumer
re-derives the binding's namespace from the target's installed parent: an
enum's variant binds its value constructor, and a struct parent is rejected by
name until struct constructors travel. Older profiles reject a variant
re-export by name, on both export and import. A type-namespace use of a
re-exported variant, such as a braced `Some { 0: x }` pattern, is a recorded
deviation.

A version-thirteen impl header may also have no trait application. That is
an inherent impl, and its member links name implementations with no trait
member.

- Its Self is a struct or enum of the same artifact, and it carries no
  `unsafe`, `const` or negative modifier.
- Its members are methods. Inherent associated constants and types stay
  rejected.
- Older profiles reject an inherent row by name.

A consumer re-reads the Self from its own installed type and records an
imported inherent owner distinct from every trait impl. It publishes each
method in its nominal's inherent bucket, so method lookup, auto-ref,
auto-deref and path calls select it as they select a local method. A call to
such a method is an ordinary function application.

A version-thirteen free provided original with no captures and no early
regions may also bind late region formals. Each named formal is a region
parameter declaration of that function. A header's original result is
admitted against its body's result by shape only. Every consumer therefore
re-proves, in the rebuilt environment, that the body's result coerces to the
declared result.

A version-thirteen original may also carry match guards.

- Local kind 3 is a guard view followed by its guard ordinal. It carries no
  declaration; its arm's binding local owns that.
- Action nodes 2 and 3 start a guard and keep it live; their flag operand is
  the guard ordinal. Node 4 binds a guard view `[local, place]`, and node 6
  re-selects a pattern place `[projection, anchor]`.
- Terminator node 1 is an imaginary edge `[real, imaginary]`. Its imaginary
  successor never executes.
- Guard columns follow the calls exactly when a statement starts a guard: guard
  rows `(tested range, view range)`, then their tested places. Every guard is
  started and owns one contiguous range of tested places and one of its own
  guard views, and every guard view lies in its guard's range.

No protection or decision proof is transported. A fresh consumer runs its own
guard ownership proof, requires a live constructor-test fact at every bound
view and re-selected place, and follows only real edges. As in local lowering,
a variant whose enum has no other variant needs no test. Older profiles reject
guard forms by name, and originals without guards keep their exact bytes.

A version-thirteen original may also use `str`, `!` and static text.

- Type recipe tag 10 is `str` and tag 11 is `!`. Neither has operands.
- Value node 7 is a text literal `[Flag(ordinal)]` of type `&'static str`.
  Text columns follow the guard columns exactly when a value is a text
  literal: one UTF-8 string per literal, with ordinals dense in value order.
- A coercion input whose reference conversion selects at least one
  dereference, with no unsizing and no two-phase borrow at either level, is a
  plain reborrow. A borrow whose outermost projection is a dereference may
  take a region variable without borrow-expression provenance.
- In a body with a two-way exit, each SSA snapshot dependency is evaluated
  earlier in its user's own block.

No literal or reborrow proof is transported. A fresh consumer checks each
literal's type and bytes and interns it itself, demands every coercion type,
and derives each reborrow's bounds from its own borrowed places. Older
profiles reject these forms by name, and originals without them keep their
exact bytes.

A version-thirteen function header may also carry two callable advisories.
The byte that marks an inline contract becomes a flag set: bit 0 is the
inline contract, bit 1 the written `#[cold]` advisory and bit 2 the written
`#[track_caller]` caller policy. Bit 3, declared target features, is
described below; higher bits are rejected, and older profiles reject bits 1
and 2 by name. Trait and nominal header rows carry neither
advisory. Caller tracking requires an exported original body on a free
function or an inherent method. A role row may also name a `#[lang =
"panic"]` function. No completion is transported: a fresh consumer re-proves
the named declaration as a free, const, safe Rust-ABI `fn(&'static str) -> !`
with no generics, and re-proves each caller-tracking flag against its own
installed signature and owner. Until caller locations transport, a consumer
rejects a call to a tracked imported function, both in its own bodies and in
rebuilt ones. Rows without the advisories keep their exact earlier bytes.

A version-thirteen struct or enum may also declare lifetimes. Each lifetime
is a region slot of the nominal, and its region slots precede its type slots.
The index of a region slot is its lifetime's declaration position. The index
of a type slot counts only the nominal's type slots. A field or variant
payload names a lifetime as region `Parameter { slot, depth: 0 }`, in the
nominal's own frame. The nominal's Self recipe and every application of it
carry the region arguments first, in slot order. A tuple variant's
constructor binds its enum's lifetimes late. Its applications carry the
enum's full arguments, regions first. No variance, implied outlives
requirement or well-formedness claim is transported. A fresh consumer
derives the variance of each lifetime and the implied requirements such as
`T: 'a` from the transported fields, as it does for a local nominal. Older
profiles reject a nominal's region slot by name. Rows that a consumer could
not re-prove are rejected by name:

- a declared lifetime bound such as `'b: 'a`;
- a method of a trait impl that captures a lifetime;
- an imported nominal's region argument in another provider's signature.

Packets without nominal lifetimes keep their exact bytes.

A version-thirteen inherent impl may also capture lifetimes, as in
`impl<'a> X<'a>` or `impl<'a> Slot<&'a u8>`. Its header frame binds the
captured lifetimes as its region slots, and its Self recipe names them as
bound regions of that frame. An impl type slot's ordinal and index count only
the impl's type slots. Each method of such an impl declares as many captures
as its impl, and its original and public binders both lead with the captured
slots, followed by its own late slots. Its receiver names exactly the impl's
Self, each impl region replaced by the same captured slot of the method's
original binder. A call to such a method opens only the method's own late
slots. The leading region arguments of its function application supply the
captures in slot order, and the call names no method request. A Borrow whose
referent differs from its place's type is admitted only where the original
records a plain one-dereference reborrow coercion from that reference to a
reference of exactly that referent. No region relation, outlives requirement
or result proof is transported. A fresh consumer rebuilds each method's
template with its captures as leading early parameters, as a local method
template binds them, and re-proves its body against the declared captured and
late results. Older profiles still reject such an impl's lifetime by name, as
they reject every nominal lifetime. Methods with early lifetimes or lifetime
bounds and methods of trait impls that capture lifetimes stay rejected by
name. Packets without such methods keep their exact bytes.

A version-thirteen original may also write, or borrow mutably, through a
mutable dereference reached from an immutable owning local, as rustc admits
`*c.n = v` for an immutable `c: Count<'_>` whose field `n` is `&mut u8`. A
path is unique when it is a local, a temporary, a field of a unique path or a
mutable dereference of a unique path. A place is writable when it is a
mutable local, a field of a writable place or a mutable dereference of a
unique path. Older profiles keep requiring a writable parent projection under
every dereference and reject such a write by name. No write proof is
transported: each consumer recomputes write access over the rebuilt places,
and its own borrow checker proves every write and loan.

A version-thirteen original may also read a field of a temporary, as in
`make().n` or `Self::new(text, 42).n`. Such a field is unique but never
writable, so a write or mutable borrow through it is still rejected by name
("source Store writes through an immutable dereference"). rustc 1.98.0 admits
that write; the rejection is a recorded deviation. Older profiles keep
rejecting the read.

A version-thirteen inherent impl method may also be `const`, whether or not
its impl captures lifetimes. Its header carries the existing const constness
and its original the existing const-definition context. Nothing else about
its constness travels. Validation re-proves the body as it re-proves a free
const original: every call names a const target of the same artifact. A fresh
consumer imports the method as a const declaration whose captured regions are
the leading early parameters of its template, and evaluates it in const
contexts through its own const checker. A header whose constness and
original context disagree is rejected by name. `const unsafe` methods, const
methods with early lifetimes and const methods of trait impls stay rejected by
name, and older profiles keep rejecting every const method. Packets without
const methods or temporary field reads keep their exact bytes.

A version-thirteen struct may also have a tuple or unit form. Its constructor
is a kind-11 declaration parented by the struct and carrying the struct's
name. It is published in the struct's module value namespace, next to the
struct's own type binding, and no enum column describes it. A tuple struct's
fields are named by their ordinals in canonical decimal (`0`, `1`, ...), which
no record field can spell, and a unit struct has no fields. Nothing else about
the constructor travels. A consumer re-derives its shape, payload, name and
visibility from its struct:

- a struct-parented variant is its struct's only one;
- every field of its struct is named by its ordinal;
- its name is its struct's;
- its binding visibility is exactly the narrowest of the struct's binding and
  its fields' visibilities, as rustc computes a constructor's.

A struct whose fields are named by ordinals has a constructor, and a record
struct has none. A tuple constructor's callable signature is
`fn(fields..) -> Struct` over the struct's formals and regions, as a tuple
variant's is, and a unit constructor is a value of the struct's type. Its
calls, paths and patterns use the existing function application, aggregate and
constructor-test rows, and constructor bodies are synthesized from the
re-authenticated struct header. Older profiles reject a constructor
declaration by name. `struct T();`, tuple and unit structs marked
`#[non_exhaustive]` and the `phantom_data` role stay rejected by name. Packets
without tuple or unit structs keep their exact bytes.

A version-thirteen type recipe may also be a raw pointer or a slice.

- Tag 12 is `[u8 12][u8 mutable][id pointee]`: `*const T` when `mutable` is 0
  and `*mut T` when it is 1. Any other byte is rejected by name. A raw pointer
  has no region, and its pointee may be unsized: `str`, a slice, or a
  `?Sized` parameter.
- Tag 13 is `[u8 13][id element]`: the unsized `[T]`.
- Tag 14 is an array, described below. Tag 15 is reserved for `dyn` trait
  objects.

No sizedness travels. Validation rejects a slice whose element row is `str`
or a slice. A fresh consumer interns each row in its own store and proves a
slice element sized from its own structural facts, including a parameter's
declared bound, before it builds `[T]`. A raw pointer is formed whenever its
pointee is, and a slice of a parameter is formed only with a formation input.
Signature comparison follows a pointer's or slice's child and requires equal
pointer mutability. Older profiles reject both forms by name ("source type
form requires an explicitly supported transport row"), and packets without
them keep their exact bytes.

A version-thirteen type recipe may also be an array (tag 14):
`[u8 14][id element][term length]`. The length is K's const term written in
place: `[u8 0][id ty][u128 bits]` for a value or `[u8 1][id slot]` for a
const parameter. Any other term tag is rejected ("unknown source const
term"). The reader appends each term to a dense cold column in type-row
order, so the row stays 20 bytes. A value's type must be the usize row and
its bits canonical for usize. A slot must name a usize const parameter.
Validation rejects an element row that is `str` or a slice. No sizedness
travels: a fresh consumer proves the element sized from its own facts. It
folds a value length to its own `[T; N]` and a slot to an array over its
const parameter, so each array type has one wire form. Signature comparison
follows the element and requires equal lengths. Version thirteen moves whole
array values: signatures, fields, locals, reads, stores and copies, and impl
headers over `[T; N]`. Element access keeps its fences by name: list and
repeat rvalues ("source rvalue requires a complete wider transport row"),
element patterns' constant-index projections ("source projection transport
requires its wider explicit contract"), and unsizing and indexing, which
stop before export without `CoerceUnsized` or `Index` declarations. Older
profiles keep the type-form rejection, and packets without arrays keep their
exact bytes.

A version-thirteen struct header may also be `#[repr(transparent)]`. The
nominal header's representation byte, its last, gains 2 = Transparent beside
0 = Rust and 1 = C. Only a struct header with no packing, alignment or tag
carries it, and an enum header with byte 2 is rejected. No layout or ABI fact
travels. A consumer interns the struct as transparent, and its own layout
pass re-runs the representation check and the rule that at most one field is
nontrivial over its own field layouts. Older profiles reject a transparent
struct by name ("source nominal representation requires its wider complete
contract"), and the older non-source record writer rejects it too. Packets
without one keep their exact bytes. Rewriting byte 2 to byte 0 relabels a
transparent struct as `repr(Rust)` without detection: the representation is
authenticated by the packet's inventory identity, not re-proved, and a later
ABI digest may close that gap.

A version-thirteen struct header may also be `#[repr(simd)]`: representation
byte 3 = Simd, read only under `body_nodes()`, while bytes from 4 up stay
unknown. Only a non-generic struct header with no packing, alignment or tag
carries it. A packed or aligned SIMD struct keeps "source nominal
representation requires its wider complete contract". A SIMD struct with any
type, const or lifetime parameter fails with "source generic SIMD record
requires its wider complete contract", stricter than rustc for now. An enum
header with byte 3 is rejected. No lane, count or layout fact travels: the
lanes are the struct's one field type, an ordinary tag-14 array row. A
consumer interns the struct as SIMD, and its own layout pass re-runs the SIMD
declaration and representation checks over its own field. SIMD shapes and
layouts are computed on demand, and an imported asm body's `xmm_reg` operand
is re-proved by the consumer's own 128-bit operand check, which fails with
"assembly SIMD operand requires its completed genuine representation" when the
header is not SIMD. Older profiles reject a SIMD struct by name, and the older
non-source record writer rejects it with "SIMD record metadata is not
implemented". Packets without one keep their exact bytes. Rewriting byte 3 to
byte 0 changes ABI-visible facts: alignment 16 versus 4, and vector-register
passing at C boundaries. An asm operand's re-proof rejects that rewrite by
name. Without one, it is a silent mismatch that, like the transparent rewrite,
is authenticated only by the packet's inventory identity and left for a later
ABI digest.

A version-thirteen function header may also carry its declared target
features as flag bit 3. After the inline payload come a u32 count and that
many names, each a u32 length and its bytes: 1 to 128 names of 1 to 32
bytes in strictly increasing byte order, exactly the explicitly enabled
features before implication. Only a free or inherent `unsafe fn` with the
Rust ABI and an exported original body carries the bit, and bit 3 with an
always-inline hint is rejected. Older profiles reject the bit by name
("source
target-feature rows require the version-thirteen header profile") and keep
the export fence for every target-feature function ("source
named/foreign/target-feature function metadata requires its wider
profile"). At version thirteen, a trait or trait-impl member, a non-Rust-ABI
or `const` function and a safe target-feature function keep named fences.
An unsafe inherent method carries the bit as a free function does. No
closure or table index travels. A fresh consumer re-proves every
imported bit-3 row on its own installed header (not a trait member, Rust
ABI, not `const`, declared `unsafe`) and resolves the names against its own
feature table, rejecting a name it does not know ("unknown source target
feature"). It calls the function through its ordinary unsafe signature, and
`LowIR` carries no target-feature attribute. The imported closure widens
only that body's asm CPU check, to the consumer's baseline plus the closure.
A forged addition to an imported function's declared target features (bit 3
on a function that had none, or an extra name) therefore widens that
imported body's asm CPU check to the consumer's baseline plus the claimed
closure. It is harmless while the initial assembly profile admits only
register classes that need baseline features (`reg`, and `xmm_reg` with
sse). Typing also admits `ymm_reg`, `zmm_reg` and `kreg`, but the profile
rejects them before export, so no imported body can use a forged `avx` or
`avx512f`. Any later item that lets the
profile admit a register class needing a non-baseline feature (`ymm_reg`,
`zmm_reg`, `kreg`) must revisit forged additions first: the inventory
identity, or an authenticated feature set, must cover them. Rows without
bit 3 keep their exact earlier bytes.

A version-thirteen role row may also name `PhantomData`: the existing spelled
form (tag 255, `phantom_data` and the declaration address), alone of the
Pointee family. Its declaration is an ordinary `repr(Rust)` unit struct
header, so no kind, variance, ownership or layout fact travels. A consumer
admits a nominal header only for this role and stages that struct as its own
`PhantomData`, which gives the formal its covariance, logical ownership and
zero-sized layout. The role is re-proved over the consumer's facts: a
module-owned unit struct with one type formal, no default and the bound
`T: PointeeSized` on the imported `PointeeSized` role. Every consumer also
infers each imported nominal's variances from its own field layouts and
rejects a bivariant type or lifetime formal as an integrity failure ("imported
source nominal has a bivariant formal"), so a deleted role row or a removed
field use is detected. Const formals are exempt, as in rustc. The moved-row
forgery targets a formal-free unit struct, because an honest packet cannot
carry another one-formal unit struct: its formal would be bivariant, and const
arguments do not transport. `Pointee`, its `Metadata` type and `DynMetadata`
stay fenced. Versions 10 and 12 cannot export the role, because unit structs
are version-thirteen-only ("source declaration kind is outside the initial
source profile").

A fresh consumer sets its nominal-requirement flag for every imported nominal
with a lifetime or type formal, or an own clause, exactly as a local nominal
does. Every body then prepares the storage of its value types, and runs the
nominal bound proofs. Before, only a lifetime-generic import set the flag. A
unit value of a type-generic import, such as `base::Opt::<u8>::None` or
`base::PhantomData::<u8>`, has no call or literal that prepares it on another
path, so its local could not be lowered ("generic aggregate requires
instantiated fields").

A version-thirteen type recipe may also be a pattern type (tag 16):
`[16][u32 base][u8 kind]`. Kind 0 is `!null` over a raw pointer recipe. Kind 1
is a closed scalar interval, two u128 endpoints of raw target-width bits, over
an integer or char recipe. Any other kind is rejected ("unknown source pattern
constraint"). No layout, niche or ABI fact travels. Every consumer re-proves
the pattern over its own base. A `!null` base must be a raw pointer. An
interval in its base's source order is interned under the local pattern
rules. A wrapped interval is split into `MIN..=end` and `start..=MAX` and
interned as the connected alternative, which only a signed base admits.
Endpoint bits beyond the base width, or a surrogate char, are rejected. The
exporter fences a `!null` pattern over a type parameter ("source non-null
pattern over a type parameter is not transported"). Older profiles keep their
type-form rejection. Two limits are authenticated only by the packet's
identity, as the transparent byte is. A pattern row rewritten to its bare base
still validates, though a consumer would then lay out `Option<NonNull<T>>`
at 16 bytes instead of 8. Endpoints are re-proved valid for the base, but not
equal to the provider's const values.

`NonNull`'s `rustc_diagnostic_item` travels as a version-thirteen role row
(tag 254): `[254][string spelling][address]`, after every language row and
outside the policy's role range. Its spelling comes from a closed table
holding only `NonNull`, so each later spelling is a separate admission. A
consumer requires the provider's own struct with a nominal header, at most
once, and re-proves it as a struct over its own remap. Versions 10 and 12
reject the identity by name ("source NonNull diagnostic identity transport
requires version 13") instead of dropping it. `rustc_nonnull_optimization_guaranteed`
has no byte: the guarantee follows from the `!null` field.

Version 13 also transports auto-trait declarations, with no new tag or bit. A
trait header's existing `AUTO` flag, alone or with `UNSAFE`, is admitted only
for an exact empty auto declaration: no written parameters, supertraits,
members, region captures or bounds, and a single unsized, unqualified `Self`
slot.
`CONST` stays rejected. A role-bound marker must also carry its local role's
exact flags: Freeze, Send, Sync and UnsafeUnpin are unsafe auto, and Unpin is
safe auto. `Send`'s `rustc_diagnostic_item` is the closed identity table's
second spelling. Its 254 row follows `NonNull`'s and names the provider's own
trait with exactly `UNSAFE | AUTO`. A consumer installs it in its own `Send`
role slot. `UnwindSafe` and `RefUnwindSafe` carry no identity and travel as
custom auto traits. A source client proves imported Freeze, Send, Sync, Unpin
and UnsafeUnpin structurally through the same role rule a source core uses.
Each marker bound is paid from the installed header budget, because a client
has no completed local header producer. Versions 10 and 12 keep their
by-name rejections of each declaration.

Version 13 also transports trait impls' own safety and polarity, with no new
tag, bit or row. An impl header's existing `unsafe` and `negative` bytes are
admitted for every trait impl. `constant`, and either modifier on an inherent
impl, stay rejected. Every trait impl's application already names a trait of
the same packet: the ledger rejects any other ("source implementation requires
its exact supported trait header"). In every version a negative impl maps no
trait member ("auto or negative trait implementations cannot declare
members"), and a positive impl maps every member of its trait. A trait header
that is not auto may carry `UNSAFE`. An unsafe impl's methods, with or without
a receiver, are admitted as conversion methods, on the same version-13 fact
that admits the impl's `unsafe` byte. Versions 10 and 12 keep their by-name
rejections of unsafe traits and of unsafe or negative impls. Admission is not
conformance. Each consumer re-reads every imported impl's row and requires
`unsafe == trait.UNSAFE && !negative` ("selected impl safety/member kinds
disagree with its actual trait"), rustc's E0199, E0200 and E0198 rules. An
impl of the packet's own auto trait then also requires:

- its Self obeys the declaring crate's auto Self rule, so a generic Self must
  be Sized;
- a negative impl on a nominal is always applicable: each declared input is
  one of the nominal's own inputs at Self, and Self's arguments are the impl's
  distinct parameters;
- a positive impl's requirements hold in its installed environment.

A negative impl of any other trait has no requirement, member or coverage
proof, as in rustc: it only denies its application, and it is never positive
evidence. The registry then completes each impl as validated in its own
polarity. A negative auto impl whose Self is not a nominal is trusted to its
provider: rustc checks it only in a crate without `auto_traits`, and declaring
the packet's own auto trait requires that feature. Imported custom auto traits
prove structurally as role-bound markers do, each re-checked by name as its
packet's version-13 declaration. A client's impl of an imported Freeze needs
`freeze_impls`.

A source client without a source core validates an imported impl when an
executable demand first needs it, not only in the eager walk after
declaration preparation. A Copy decision in a body that a client constant
reaches can wait on an unvalidated imported impl. The client then proves that
source-free header with the walk's own conformance, and the walk skips only
that owner's conformance: it still checks every overlap pair before any
owner. Selected mode installs no constant executor and keeps its text. A
client source header reached this way reports "client implementation
validation before the eager walk requires a source scheduling context", a
known rejects-valid. An imported impl related by the client's specialization
graph reports "imported implementation validation before its specialization
conformance order". The [scheduling-context
packet](pa29/tests/spec/scheduling-context/README.md) records independent
outcomes.

A client forms each imported nominal it names under that nominal's transported
where-clauses, as rustc does: its formation requirements are its sized
parameters' Sized edges and its original inputs, declared and implied, the
same inputs selection and coherence read. They are substituted at the
instance's arguments, with outlives omitted as for a source nominal, and
reported with the source twin's text and location. Negative coverage reads the
same requirements. Each instance's clauses are cached once. A walk that has no
caller-owned budget pays the installed header budget exactly the substitution
it attempted, and a charged walk, as specialization admission makes, pays only
its own closure. The [imported-formation
packet](pa29/tests/spec/imported-formation/README.md) records independent
outcomes.

Coherence proves a goal's absence for the crate whose coherence it proves, as
rustc 1.98's `trait_ref_is_knowable` does. A consumer re-proves a pair of
impls imported from one packet from that packet's perspective, over what the
packet could see: the consumer's own impls are never candidates there. An
imported impl's where-clauses enter coherence from the same original inputs
selection reads. Every other pair is proved for the consumer, including a pair
across two packets, until a multi-packet profile records which packet is
downstream. Absence is knowable only when no downstream crate could implement
the goal, and when that crate owns the trait or an input type (Self, then the
trait arguments) after `&`, `&mut` and `#[fundamental]` nominals. Closed
language traits and `#[fundamental]` traits also qualify, read from the
attribute fact. rustc 1.98's fundamental traits are `PointeeSized`,
`MetaSized`, `Sized`, `FnOnce`, `FnMut`, `Fn` and `Coroutine`, and its
fundamental nominals `Pin`, `FieldRepresentingType` and alloc's `Box`. rustc
takes the first owned input with no uncovered parameter before it. Closed
inputs have no uncovered parameter, so any owned input is that first one, and
a widening to open inputs must restore the order. A packet cannot carry a
`#[fundamental]` nominal, so an imported one is never peeled. `dyn` and
foreign types are never owned inputs. Both limits are conservative. The
[knowable-absence packet](pa29/tests/spec/knowable-absence/README.md) records
independent outcomes.

Exported `macro_rules!` definitions travel in their own version-one envelope
(tag 14), nested inside the native-link envelope and outside every core
profile. A crate that exports no macros writes no envelope and keeps its exact
bytes. Each row holds the crate-root name, the definition edition, the
`local_inner_macros` bit, the rule count, a canonical image and typed preorder
token-tree rows. The image spells the single delimited body. Two pieces
(tokens, open and close delimiters) are adjacent exactly when they were adjacent
in the definition source, and one space separates them exactly when whitespace
or comments did, so lexing the image keeps every token's jointness. Rows are
sorted by unique name. Only direct root `#[macro_export]` source definitions are
exported. Builtin rows and `allow_internal_unstable` lists need version 2
(below). Macro-generated definitions, doc comments in the body, scoped macros
and other checked definition-site permissions are rejected by name rather than
dropped. Decoding checks the closed edition, flag, class and delimiter codes and
that the rows lay out the image exactly. A consumer must not trust these rows: it
lexes the image once, rebuilds the rows and requires them to be equal, and then
recompiles every rule in the transported edition. It binds `$crate` to the
provider's crate identity, never to a name or alias. Rows are bound publicly in
the provider's imported root, for `use` and path forms. `#[macro_use] extern
crate` adds them to the macro-use prelude, where a later crate shadows an
earlier one. Generated spellings are never re-lexed. The [macro metadata checkpoint](docs/parallel/macro-metadata.md)
records the profile and its consumer requirements.

Version 2 of the macro envelope is written only when a table has a builtin
row or a non-empty feature list; otherwise the version-1 bytes are unchanged.
Its rows are sorted strictly by (module placement, name). Each starts with a
kind byte (0 declarative, 1 builtin), the placement path and the name. A
declarative row keeps the version-1 fields, is placed at the root, and adds its
sorted, unique `allow_internal_unstable` features. A builtin row has no image.
It carries the syntax (`macro_rules!` or `macro`), the expander identity as a
bounded identifier spelling, the kind (bang, attribute or derive) and sorted
derive helpers. A `macro_rules!` builtin row is placed at the root, and only a
derive row has helpers. Unknown codes, disorder and a needless version 2 are
rejected by name. A consumer binds a builtin row in the provider's own public
module at its placement, resolved by module identity and never by a name or
alias. It runs its own expander of that identity and rejects an unknown identity
or a changed kind by name. An imported expansion frame grants exactly its row's
re-authenticated features, charged per feature, as a local definition's
permissions would; `allow_internal_unsafe` comes only from a version-4 flag.
Macros 2.0 definitions other than version-4 semiopaque rows, and imported
expanders that need their provider's source declaration, other than the
version-4 panic delegations, remain rejected by name.

Every public macro binding of a crate's public modules is published, or the
export fails by name; none is dropped. A local definition has one definition
row, at its own binding when that is public, otherwise at the least public
(placement, name) key. Every other public macro binding, direct, imported or
glob, is a re-export row unless a version-5 glob row publishes it: kind byte 2, its placement and name, then an owner
ordinal (`u32`; 0 is the artifact itself, `k` its `k`-th sorted dependency),
the target placement and the target name. The target is the key of a
definition row of that owner, never another re-export, so chains flatten to the
defining row. Re-export rows sort with the other rows and alone also require
version 2. A consumer first binds every definition row, then binds each
re-export at its placement to the `DefId` of the owner's definition row, found
by crate identity and row key and never by name or alias. Every path to one
macro thus denotes one identity: globs bringing it twice are not ambiguous,
while explicit imports of one name still collide. `#[macro_use] extern crate`
enters every macro bound at the provider's root. Ambiguous bindings, an
unknown owner, a target that is not a definition row and a public macro binding
reachable only through a module alias are rejected by name.

Version 3 of the macro envelope is written only when a definition row is
`#[unstable]` or a builtin row has an `allow_internal_unstable` list;
otherwise the version-1 or version-2 bytes are unchanged. It keeps the
version-2 layout. A declarative row appends a stability tag (0 for none, 1
followed by the feature). A builtin row appends its sorted, unique feature
list and then the tag. A builtin's list passes the same checks as a
declarative one; `allow_internal_unsafe` and edition-panic contracts need
version 4. Every call of an imported unstable row, through any path,
import or re-export, requires either the crate's request for that feature or
a call span inside an expansion whose definition may use it; otherwise it fails
by name, as rustc's E0658 does. A row's feature is a library feature of its
dependency. A denied use of an unstable imported API whose site lies inside a
macro expansion is admitted only by that frame's allowance list, after
resolution completes; a use outside any expansion fails immediately. Source clients and staged consumers own the same imported-only
allowance ledger as ordinary crates whenever a dependency row carries a list.

Version 4 of the macro envelope is written only when a table has a
semiopaque row or a row with `allow_internal_unsafe`; otherwise the version-1
to version-3 bytes are unchanged. Every definition row (declarative, builtin
and semiopaque) appends one flags byte after its version-3 fields; bit 0 is
`allow_internal_unsafe`, and any other bit is rejected by name. Re-export rows
are unchanged. Kind 3 is a semiopaque `pub macro`
(`#[rustc_macro_transparency = "semiopaque"]`, multi-rule form): its placement,
then the declarative fields with `local_inner_macros` clear, its list, its
stability and its flags. A consumer binds it at its placement and compiles its
image once with `macro_rules!` hygiene in the row's edition. Transparent and
opaque `pub macro` definitions and the single-rule parameter form are rejected
by name. The edition-panic contract is the `edition_panic` member of a row's
list: a frame whose list contains it is skipped when choosing the panic
edition, exactly as for a local definition, and a list that disagrees with the
definition's parsed contract is rejected at publication. An imported `panic`,
`unreachable` or `assert` builtin row delegates to its provider's rows through
`$crate`, bound to the provider's crate identity. A Rust 2015 or 2018 call site
of `unreachable` delegates to `unreachable_2015`; pre-2021 `panic` and `assert`
call sites stay rejected by name. Source clients of every edition pass one
gate. Each pre-2021 rule they reach is selected or rejected by name, and Rust
2021 shares the before-2024 rules. A source client is either `#![no_core]`
with its source feature or `#![no_std]` over a provider named `core`. In the
second case, as for rustc, the injected `core` is the extern-prelude entry,
which Rust 2015 also names as a crate-root item. An `--extern core=` alias
names the direct dependency, which may be the provider itself or a crate
re-exporting it, such as `rustc_std_workspace_core`. The entry must be that
dependency's imported crate root. Without the alias, the entry is the
authenticated provider's root. The entry's public `prelude::rust_20xx` module
for the client's own edition is the client's glob prelude. Injected `core`
carries no `macro_use`, and `std` stays a reserved alias. A crate without that
prelude module is rejected by name. An imported row's
`allow_internal_unsafe` covers only sites in its own expansion frame, never a
nested expansion of another definition. A single `use` import binding an
unstable imported row needs the same feature or allowance as a call of it,
whether or not the row is called; a glob import is checked only at its module.
Builtin derive output roots each `crate::` path at the defining crate's
identity, as `$crate` does; an imported derive row does not yet run.
A derived tuple field index, such as the `0` of `self.0`, takes the syntax
context and edition of its field's first byte, as rustc spans it with the
field. Field privacy is therefore checked from the module, or the
`macro_rules` definition, that wrote the field, not from the derive's
definition. The index claims no copied bytes and grants no allowance. See the
[derived tuple field privacy checkpoint](docs/pa31-derive-field-privacy.md).

A derived impl of an item with type parameters also bounds each field type path
whose first segment names one of them, unless the path is that parameter alone,
as rustc's `find_type_parameters` does. Every such path in an active field's
type, such as the `I::Item` of `Option<Option<I::Item>>`, gets `P: Trait`
after the written predicates, once per occurrence. `Trait` is the bound the
derive adds to its type parameters, so `StructuralPartialEq` and an enum's
`Default` add none. A qualified path's own segments and constant expressions
are not walked as field type paths. A type macro in such an item's field is
rejected, as rustc rejects it. See the
[derived field path bound checkpoint](docs/pa31-derive-projection-bounds.md).

Version 5 of the macro envelope is written only when a table has a glob row;
otherwise the version-1 to version-4 bytes are unchanged. It keeps the
version-4 layout and adds kind 4, a glob re-export of a dependency's module
(`pub use dependency::path::*`): its placement, a `u32` owner (`k` for the
`k`-th sorted dependency; 0 is rejected) and the path of the target module in
that owner. A glob row has no name. It sorts before the named rows of its
placement, and glob rows of one placement sort by owner and target. A public
glob import whose source is a dependency's module publishes this one row
instead of the names it gives in any namespace, so no public-name, macro or
re-export row repeats them. A glob in a private module binds nothing outside
the crate and publishes nothing. The export rejects by name a glob in a module
that only a re-export reaches, a target module whose owner reaches it only
through a non-public module, and an owner that is not a direct dependency. A
consumer expands glob rows after it binds every artifact's definitions and
macro re-exports and before it binds crate aliases, each artifact after the
artifacts its rows name. The owner is found by crate identity through the
publication's dependency list, and the target and placement through the
owner's and the publication's public module identities, never by name or
alias. Every binding of the target that is visible outside the owner's crate,
in the type, value or macro namespace, is bound at the placement unless the
placement binds that name itself. When two glob rows give one name different
targets, the name is ambiguous, and only a use of it is an error, by name.
Rustc 1.98 reports such a use across crates through the deny-by-default
future-incompatible lint `ambiguous_glob_imports` and, when that lint is
allowed, selects one glob's target; RustGM keeps the use an error. Each row is
charged one unit and one per binding of its target module that it visits,
within a fixed budget. Older readers reject version 5 by name.

A crate's loader attributes travel in one flags byte that leads only the
outermost artifact: `[15][flags]` follows the 12-byte header, and the rest
of the artifact keeps its unflagged bytes. Bit 0 is `#![compiler_builtins]`.
It is written only when a flag is set. Unknown bits, an empty flags byte, a
repeated tag and a flags byte on an artifact nested in an envelope are
rejected by name, and older readers reject tag 15 as an invalid base profile.
`#![compiler_builtins]` is one inner word attribute of the crate root and
needs its internal feature, as in rustc 1.98; a second one, an argument and a
use below the crate root are rejected by name. After name resolution, rustc's
crate loader injects `compiler_builtins` into every crate except a
`#![no_core]` or `#![compiler_builtins]` crate, unless a loaded crate is
already `#![compiler_builtins]`. A consumer reads the flag from every artifact
of its dependency inventory. A `no_std` source client is the only crate that
loads a real core, so it is the only one that can need the injection: with no
flagged artifact it fails as rustc's E0463 does, and an `--extern
compiler_builtins=` entry without the flag fails by name. RustGM's built-in
core and std stand for a graph that already contains the crate. The injected
crate binds no name and adds no dependency row.
In a source client, an `--extern` name of a legacy artifact, which has no
source catalog, resolves to that artifact's imported crate root, so the
upstream shim's `use compiler_builtins as _;` resolves as in rustc.

## Lowering and backend facts

Lower only checked, typed facts. Give every emitted instance, drop shim, vtable,
closure and coroutine a stable emission identity and emit it once. Rust signed
overflow, saturating casts, zero-sized types, fat pointers and panic behavior
must not inherit C++ assumptions. In particular, do not blindly emit C++
`nowrap`, `typed` or alias facts. State only what Rust analysis proves.

Cleanup lowering consumes the proof's ordered statement, cleanup and flag-write
events. Preserve successor-specific cleanup on real CFG edges; imaginary proof
edges never generate runtime code. Retain no event tape for bodies with no
runtime cleanup. Share demand-emitted glue by canonical recipe identity, and
emit custom destructor calls before automatic field destruction. Variant tags
select only active fields. Array intervals use counted loops, including
zero-sized elements; array length must not expand emitted instructions. Use
bounded iterative emission, with field cursors preserving declaration order.
Give zero-sized destructor receivers stable aligned storage, and copy packed
fields into aligned backing before invoking their glue. Ordinary expressions
and cleanup share the same typed layout and address operations.

Use the shared LowIR model contract; serialized text is an output view. The
temporary backend adapter accepts LowIR only, validates it with CPPGM and then
builds native code. No oracle object or reference function may replace a failed
Rust compilation. General assembly and target intrinsics require an explicit
capability review before their first required workload and broad closure at PA41;
the initial backend is not assumed complete. A required backend extension is
separately reviewed work, not implicit authorization to change CPPGM.

Persistent data consumes completed constant values through a read-only typed
interface. Reserve global identity before following address dependencies;
statics retain their own storage and projections retain their original root.
Serialize canonical field offsets and exact scalar bits with bounded flat work.
Memoize zero-filled products and keep repeated zero data compressed. By-value
constant aggregates still produce independent runtime snapshots. When the
external backend cannot relocate an unaligned pointer field, retain its packed
offset and install the address before Rust entry code; never silently realign it.

## Ownership, allocation and complexity

Use owned arenas/slabs/vectors and compact IDs instead of Rc/Arc pointer graphs,
per-node boxes or recursively owned trees on hot paths. Prefer slice/range
children and reuse per-query/per-body scratch capacity. Bulk destruction must
follow phase ownership. Avoid an allocation per token, edge or solver step.

Each function owns mutable body/LowIR records. Editing one function must not
rebuild another. Release per-function scratch promptly and discard frontend
ownership before whole-program LowIR optimization. A future O0 in-process
backend should consume and release functions incrementally.

Lexing should be O(source bytes + produced tokens); parsing near-linear except
explicit language-required matching. Resolution visits relevant scopes and
candidates, not unrelated declarations. Lowering should be linear in emitted
IR; fixed points must make monotonic progress in a bounded lattice/worklist.
Any broader bound needs a documented semantic reason and adversarial tests.

The [frontend performance series](https://github.com/vishvananda/rustgm/compare/907dcd0c...d43ca889)
is the implementation precedent for subsequent work. Prefer removing repeated
work and reducing retained representation before tuning individual instructions:

- Intern names during the existing parse and pass their identities downstream.
  Keep common syntax compact, including single-component paths and optional IDs.
  Preserve Unicode and source-span behavior on byte-oriented lexer fast paths.
- Build scope parents and bindings in the existing crate walk. Use shared flat
  tables for lookup, with ordered records for observable declaration and import
  order; hash iteration must not define output. Fast deterministic hashing gives
  expected throughput, not a worst-case bound against colliding source keys.
- Number inferred regions within their actual body numbering unit. Every lookup
  retains its body identity; equal numeric IDs in unrelated units are not equal
  lifetimes. Nested bodies sharing an inference graph share a unit, while exact
  owner indexes avoid scanning a unit's unrelated allocation suffixes.
- Compile reusable signature shapes once. Share only substitution-invariant
  storage and CFG analysis across instances with the same template and promotion
  structure. Keep ownership, initialization, destruction, region and loan checks
  under each complete instance's own inputs and proof scope. Debug revalidation
  checks the structural sharing premise; public validation remains available.
- Borrow an already prepared source/import environment for repeated queries;
  do not clone its complete universal table or rebuild premise indexes per use.
  Bind each view to its actual graph and complete instance, retaining fresh proof
  for each distinct instance. Measure repeated uses, distinct instances and
  original-input width separately. The
  [source-artifact/readiness checkpoint](docs/source-artifact-readiness-checkpoint.md)
  records that scope boundary and its measured costs.
- Share immutable clause and parameter prefixes for method-local proof extensions.
  Retain only the actual additional rows and their opening; do not flatten an
  inherited map, rebuild its indexes or copy a principal's whole implied closure
  for every method. Empty declaration scopes may share their completed parent
  only when they add no own rows and preserve frozen promises and alias policy.
  Account for ordinary enclosing/payload growth as well as query rows, prepay
  failed growth, and vary method count and inherited depth together. Shared
  carrier storage never carries a completed caller proof across those queries.
- Reuse per-body recording and traversal buffers without retaining previous
  bodies' semantic authority. Answer empty work queues before allocating their
  continuation machinery. Box rare large diagnostics rather than inflating every
  successful result; keep ordinary values and identities inline.

For each such change, state the work removed, its cache or ownership boundary,
and the input dimensions that could still multiply work. Test those dimensions
independently and together, including cold, warm and invalidated states where
applicable. Record compact record sizes and bounded work/storage counts as well
as end-to-end measurements. Do not accept a faster common case as evidence that
a quadratic path has been removed. A representation-only change preserves
language outcomes. Separately qualify lowering rewrites such as arithmetic-width
overflow checks at their execution boundaries before regenerating affected
inspection references.

Removing speculative opening may remove unused types from an inspection arena.
Review the complete retained graph and its identity mapping, require identical
lowered output and independent native behavior for the affected source, then
regenerate only the affected oracle through a scoped course-runner receipt.
Keep byte-exact comparisons and all source proof obligations. Do not recreate
unused allocations merely to preserve historical arena IDs. The
[projection-opening checkpoint](docs/pa31-projection-opening-sharing.md)
records this review for an actual caller-premise case.

Telemetry observes work already being done: bytes/tokens, retained token bytes,
allocations or arena high water, solver goals, retries, region edges, demanded
instances and phase time. Disabled instrumentation must not change semantics.
Track throughput and peak RSS at several input scales. Compare frozen binaries
with identical inputs, A/A calibration and paired ABBA blocks; retain all raw
samples and verify output equality. Wall-time noise alone is not a regression.

## Safe Rust and audits

Forbid authored unsafe compiler code. The input language still includes unsafe
Rust; fixtures are not compiler implementation. Keep OS/process/environment
access at explicit driver/build boundaries, outside `dev/src/` environment
reads. Audit dependencies and feature sets before use.

Run rustfmt and Clippy all/pedantic with warnings denied. The file audit counts
code lines: 3000 per implementation module, 2400 for registered interfaces,
1800 for internal models; functions 240, or 360 for cohesive dispatch with at
least eight arms. Nesting warns past six and fails past eight. Do not hide logic
in macros, compressed lines, generated files or mechanically split modules.


## Associated semantic inspection

The maintainer semantic view consumes completed declaration/body facts. Reuse the
canonical type, definition, argument and region writers; do not reconstruct
semantic decisions from syntax or output. Owner/member inventories and per-owner
predicate vectors retain construction order. Sort sparse source-keyed maps only
at the view boundary. Count mode consumes the same facts without text rendering.

Expose trait/impl ownership, declared parameters and defaults, complete predicates,
validated method/default mappings, qualified paths, receiver adjustments and
selected implementation arguments. Preserve bound-selected lookup origin through
this view, authenticated by the selected member's identity. A syntax node can
also own implicit Deref or conversion calls. A caller-only lookup restriction
belongs to the selected member; each other actual call still proves its own
complete application through normal selection. Retain that member in the existing
sparse lookup index rather than granting or restricting proof by node alone.
Region/loan constraints remain explicitly pending: inspection succeeds
at the inference boundary, while LowIR emission retains every template, lifetime,
ownership, loan, cleanup and demanded-instance proof. The new view must not change
that execution boundary or mutate the analyzed facts.

## Trait semantic inspection

PA19 keeps `semantic-crate 1`. PA20 has a separate `trait-semantics 1` view,
sharing the earlier typed records and appending associated type/constant members,
result contracts, selected default/override identities, completed constant
dispatches and retained original/normalized type pairs. Read the existing sparse
unreduced-type store; do not introduce a duplicate normalization journal or rerun
selection to manufacture display facts. Count mode reads the same facts without
rendering or sorting. Full output sorts sparse maps by canonical source or
declaration identity at the presentation boundary.

A typed associated constant use can precede value dispatch. Show only dispatches
that have actually completed. Symbolic normal forms can remain opaque, and
lifetime obligations remain pending at this inspection boundary. Full LowIR
emission proves the later region, ownership, destruction and layout requirements.
The PA20 handout fixes row spellings, ordering and count meanings.

Closure indexing captures consume the actual selected Index/IndexMut receiver
borrow, separately from the result leaf's Copy capability. Keep the index and
selected method call at invocation time. Reuse the existing sparse adjusted
access owner for the actual reference prefix and subtract it exactly once during
place construction. Built-in sequence captures retain whole-container precision
and existing leaf-use rules; ordinary move, loan and lifetime proof remains
required. See the [selected index capture checkpoint](docs/pa26-selected-index-captures.md).

The existing lazy primitive-constant owner may publish typed char MIN/MAX through
normal scalar validation, CTFE and ConstArgument consumers. Keep integer MAX
width rules and generic-const restrictions unchanged. SourceCore always resolves
its actual associated declarations and initializers; bootstrap value selection
must never replace an absent or differently valued source definition. The
[char checkpoint](docs/pa26-char-constants.md) records ownership, scalar validity,
physical widths and retained failed setup predictions.

Primitive owned-left/shared-RHS Add/Sub methods reuse the existing selected
primitive descriptor, canonical Output/member and scoped signature substitution.
Validate local scalar storage shape separately from region identity: generic
regions are raised beneath the method function binder even when that binder is
empty. Exact region equality belongs to the canonical scoped substitution.
Factories retain ordinary loan/ownership/arithmetic proof and instance-bound
permits. Bootstrap primitive admission grants no SourceCore language-item
permission; actual source bodies and source binary/const/execution proof remain
required. See [the shared RHS checkpoint](docs/pa26-shared-value-operands.md).

Closed required source const-trait member requests authenticate the actual
provider, body owner, typed Call, complete selected/resolved callee and defining
const effect. This is defining legality, never implementation or execution
permission. Select concrete method operands through the existing materialized
instance/dispatch owner before borrow and Code proof. Keep abstract/default
entry fences. Ordinary CFG caches retain semantic originals; selected Code owns
its selected body and extended contract, including fresh dispatch region facts.
Reuse the existing original-template ownership and byte payers rather than
retaining closed bodies in a generic-template cache. The
[source operator checkpoint](docs/pa31-source-add-sub.md) records genuine Add/Sub
headers, macro CTFE and source/native consumers without a primitive factory grant.

Source `ManuallyDrop` and `MaybeDangling` consume actual reserved/completed nominal
identities, declared fields and representations. Keep total destruction suppression
separate from a union's field suppression. Pointer relaxation belongs to the first
actual MaybeDangling field's typed traversal context; never attach allocation,
write, borrow or API authority to a value because that context was visited. Include
type/context in visited keys, retain it through suspended byte duties, and start
ordinary roots without it. Reuse canonical field-list/array recipes and prepay
queue/table growth while charging live temporary capacity. Unknown source metadata
tails descend the existing scoped trait stack and retain actual child evidence;
physical metadata is not a caller's sizing proof. General nominal metadata must
carry genuine structs, enums, unions, generics, constructors and policies through
validated remapping before imported source roles can execute. See
[the source wrapper checkpoint](docs/pa31-source-wrappers-checkpoint.md).

Terminal constant results pay validation work without allocating a graph queue
or visited table. Demand pointer/layout checks only for actual pointer values;
scalar and callable terminals must not speculate about physical layouts. Keep
reference backing/escape checks on the terminal path. Products, repeated arrays
and images retain the shared typed graph traversal and its live-capacity payer.
The unchanged nested-readiness invariant must still reject overlapping execution
at the maximum budget that admits either initializer alone. See the
[CI repair](docs/source-wrappers-ci-repair.md).

The actual source mul_with_overflow operation retains its genuine one-type Copy
signature, caller/source/const proof and closed integer instance. Return wrapped
target bits and a separate overflow flag using the ordinary tuple layout; overflow
never acquires a panic path. CTFE reuses the checked/wrapping primitive evaluator.
Native narrow products share ordinary checked multiplication's widened kernel;
128-bit flag computation uses unsigned magnitudes, sign-dependent bounds and a
nonzero denominator, preserving both MIN/-1 orders and zero inputs. Keep this
flag-only operation separate from ordinary checked128 panic guards. See the
[PA28 multiplication checkpoint](docs/pa28-multiply-overflow.md).

The actual source overflow_checks intrinsic returns true in CTFE independently
of runtime configuration. Its runtime answer uses this driver's fixed checked
arithmetic policy. Reuse the actual zero-input checked Call, completed signature,
const declaration and source provenance; an authored fallback cannot replace its
intrinsic meaning. The inherited-overflow word supplies no new caller authority.
Keep UB configuration and frozen/deferred policy distinct. Selectable overflow
profiles and imported policy transport require their own authenticated data.
See the [overflow-query checkpoint](docs/pa31-overflow-query.md).

The actual source is_val_statically_known intrinsic may return either boolean.
RustGM chooses conservative false during constant evaluation after authenticating
its current completed declaration, formal Copy premise, exact signature and
checked direct Call. All argument expressions execute once; no pointee or layout
inspection supplies knowledge. An ordinary same-named function has no intrinsic
authority. Native execution requires the genuine provided body and ordinary
source/body/call proof. Imported provenance and indirect const-call permission
remain separate requirements. The [static-knowledge checkpoint](docs/pa31-static-value-knowledge.md)
freezes independent outcomes and actual native execution.

Strict slice-result validity accepts an array-unsized pointer that retains its
original zero-offset array place and actual length metadata. When no explicit
range is retained, validate that metadata prefix against the same actual array
backing, element, bound, alignment, initialization, static and write policies.
Do not invent length metadata, accept an unnormalized byte offset, rewrite the
pointer's provenance or reuse an earlier successful validity check after mutation.
The [whole-slice checkpoint](docs/pa31-whole-slice-validity.md) retains empty,
nonempty, role-lookalike and strict lifetime/initialization opponents.

Source NoDrop field rules consume the exact completed language identity and
ordinary source Copy/component proofs. Reuse the existing suspended component
frontier, retaining caller scope and every child debt; a builtin marker proof
supplies no executable method or const permission. Complete source wrapper
policies after field/sizing producers and before union queries, rather than
bypassing a pending ledger or repeating completion later.

Pure structural field shortcuts belong in canonical type facts. Compute the
pinned trivial-Copy shape once from immediate child facts in the existing compact
Properties bits. Keep it separate from actual source trait membership and value
validity. Do not rebuild a graph walk or allocation table per declaration: many
unions can share one deep type DAG. Read the fact directly and retain actual
membership proofs only within their valid caller. The
[NoDrop checkpoint](docs/pa31-no-drop-checkpoint.md) records independent source,
native and depth/declaration work controls, unchanged four-byte type properties,
and calibrated immutable A/B observations without overstating noisy wall time.


Executable storage must consume the caller's actual completed static solution
before Code publication. Reuse the same cached substitution that selects its
callable targets; keep raw source item contexts and original Body/Contract facts
intact. Opened Call roots already belong to the body, so their empty function
record adds no new binder scope. Finalize existing reference-selector descriptors
from that exact Code's argument tuple/result while the proved producer owns both;
later shape matches cannot authorize a new source relation. Preserve current
Code/owner/operand validation and strict image/backing type identity. Ordinary
bodies with no static replacements do not scan their storage graph or allocate
a target map. Share canonical dependency bits and one fold across roots, charge
attempted work and avoid a new parameter vector for an unchanged Call.

An ordinary union is an opaque byte-family leaf only after its real layout is
complete. Capability queries do not read inactive alternatives; typed construction
and selected-field reads still validate their actual values, masks and pointer
provenance. No language role or byte capability grants Copy, initializedness,
unsafe permission or executable methods. Source MaybeUninit uses ordinary source
method bodies and reached intrinsic contracts; general native profiles and
imported nominal closure remain required. The
[constant checkpoint](docs/pa31-maybe-uninit-const-checkpoint.md) records source,
owner, bounded-work and immutable A/B evidence and the new original-core frontier.

Runtime source layout intrinsics share completed concrete properties in a cold
optional owner, while every reached Call retains current-body, complete
signature/result, argument and evaluated-target checks. A shared plan supplies
data, never another caller's permission. Pay warm queries and prospective map
growth through the existing source contract ledger. Ordinary sources without
plans return before speculative value inspection. Assert-inhabited collects the
real source panic_nounwind function even for a concrete NOP, matching pinned
monomorphization; false assertions alone emit its normal body/ABI call with
initialized static text derived from completed types and resolved names. The
[native checkpoint](docs/pa31-maybe-uninit-native-checkpoint.md) records the actual
value/destruction/panic observers and remaining diagnostic/imported profiles.

Compact cold retained role tables independently of their public declaration
API. Checked niche identities preserve zero/absent parent and source origin,
reject unrepresentable IDs and keep caller/parent authentication intact. Measure
actual retained layouts before updating size guards; keep the existing
provider/imported difference bound and remove lint expectations made obsolete
by a structural improvement. No ordinary node, body or execution frame grows.

Canonical aggregate const data belongs beneath the existing optional generic
argument storage. Keep scalar bits, rigid parameters and aggregate identities
distinct. Intern complete typed products from immediate canonical child IDs;
nominal owner, enum variant and field order are part of identity. Normalize
uniform array sequences and compressed repeats consistently, including empty
arrays, without expanding large repeats or recursively hashing shared children.
Charge validation, collision comparisons and prospective growth before publication;
warm queries still pay work. Payload nodes and field edges belong to the const
producer, not unrelated trait-input charges or cache-invalidating extents.
Semantic producers separately validate actual fields, feature/marker authority,
initialized values and defining contexts. Data interning supplies none of that
permission. Ordinary and suspended generic-value queries publish canonical
`ConstId` results. Convert completed evaluator products iteratively, memoizing
each `(TypeId, Constant)` with its published field recipe and actual variant.
Keep the scalar path free of graph scratch. Pay bounded conversion work/storage
separately from AST work and, in source mode, against the completed local producer.
Readers retain a compact canonical handle and consume immediate children without
rebuilding the product arena. Preserve storage-type identity, array-length scalar
requirements, actual out-of-bounds faults and ordinary VM mutation/authority rules.
The [PA25 boundary and integration order](docs/pa25-aggregate-const-values.md)
records data/evaluator/reader controls and the remaining source declaration,
transmutability-relation and imported-artifact work.

Separate const-parameter identity registration from final source permission.
The actual aggregate language request and supported closed type are required
before registration; validated marker selection follows ordinary implementation
validation and checks unused declarations as well as demanded instances. Reuse
proof scratch within its actual owner environment and traverse the parameter
inventory once. Check real substituted nominal fields and every variant through
the existing shape producer. A generated marker impl still proves every child;
the shared derive header retains its real generic constraints. Keep language
requests distinct from library policy definitions and implied_by targets.
Const-argument lowering consumes the common typed initializer keyed by canonical
const identity; it must preserve owned snapshots instead of forcing scalar bits.

Transmutability source authority has separate declaration, value and relation
boundaries. Authenticate the actual unsafe trait, its two owned formals and its
deny-implementation/dyn/coinductive claims. Complete the actual nongeneric options
struct only after its four named boolean fields exist. Decode canonical values
against those real field ordinals and nominal identity; source order and scalar
bits confer no assumption authority. These constant-size checks use the existing
cold source ledger without growing ordinary body or syntax records. A completed
header/value does not prove the all-values layout, validity, reference or lifetime
relation. Preserve caller-scoped/coinductive obligations, the actual source
default method and required imported transport; a successful value transmute is
not a type-level transmutability proof. See the
[source checkpoint](docs/pa31-transmutability-source-checkpoint.md).

Layout-language structure is separate from trait permission. Consume completed
ABI fields/offsets/tags and opaque references through hash-consed products,
alternatives and counted repeats. Preserve correlated words, initializedness and
zero-size nominal guards. Use iterative memoized derivatives and paired states;
factor repeated/product structure before general comparison. Share interpretation
across both roots under the same safety policy rather than traversing a common
graph twice. Prepay work and growth, and retain recursive type, outlives and
Freeze atoms for the scoped solver. Structural equality or one successful value
reinterpretation cannot discharge those atoms. See the
[layout kernel qualification](docs/pa31-transmutability-layout-relation.md).

Reflection language declarations retain their actual source struct identities.
Pinned `GenericRequirement::None` supplies no generic-arity or field-shape rule;
validate the real representation independently at the consuming intrinsic.
Completion grants no type provenance, runtime hash, reflection operation or
imported authority. Actual TypeId production must preserve CTFE type provenance
and stable imported identity rather than treating arbitrary scalar bits or a
local canonical index as a trusted type. Retain old metadata boundaries until
the complete nominal/value and operation contract is transported. Share common
declaration target/arity facts without replacing dedicated trait, projection or
execution proof. See the [source qualification](docs/pa31-type-reflection-source-checkpoint.md).

FieldRepresentingType and Field headers consume actual ordinary nominal/trait
declarations. Complete independently present Base/Type/OFFSET members after
ordinary aliases/constants and owner schemas finish. Preserve genuine unrelated
trait and inherent owners, alternative unused shapes and ordinary initializer
errors. Reuse the existing declaration ledger; header completion supplies no
builtin implementation, projected type or offset value. Demanded consumers must
authenticate the actual representing application, selected base/coordinates,
eligible unpacked struct or tuple and required sizing/trait proofs. Base/Type
normalization retains that selected parent. OFFSET must execute the genuine
provided initializer and authenticate its reached field_offset intrinsic before
reading completed layout. Source field_of lookup keeps its own contextual
privacy and stability authority. Full reflection and authenticated transport
remain separate required consumers.

Builtin Field evidence retains a sparse seal in the existing source-trait ledger,
not every ordinary declaration or solver frame. Capture actual nominal/trait/
Base/Type/OFFSET identities and retained source images after their ordinary facts
finish. Canonical authored trait inputs precede synthesized Self assumptions.
Use ordinary scoped children for all real supertraits and sizing duties; a
representing type's Copy evidence never confers Copy on its selected field.
Normalize actual semantic field types with full mixed arguments and retain the
same selected parent/member/application through every result consumer. Missing
nominal contracts yield the ordinary typed worklist dependency. In validated body
checking, seal completed result data only at its actual first demand; earlier
work still belongs to the declaration coordinator. Authenticate the actual Report
again at instance/publication boundaries. The [qualified finite consumers](docs/parallel/field-proof-projection-checkpoint.md)
qualify actual OFFSET/default/layout execution in the
[complete native checkpoint](docs/parallel/field-offset-execution.md); complete
reflection and authenticated transport remain required.

A source Field constant selects its genuine declaration/default through ordinary
caller evidence. Field offset intrinsic recognition preserves its distinct typed
declaration identity and all actual defining conditions. Require the exact proved
Code/input row, complete current reached call, API/const policy and actual source
images before reading shared layout coordinates. Physical layout is not Field
proof or cached caller permission; an alternative authored default retains its
own type/value. Reuse the representing argument decoder and aggregate layout
preparation, without an intrinsic result map or dense executable fields.

Projection normalization consumes actual typed readiness from one retained
search; checking implementation headers cannot predict every source, nominal or
field dependency. Completed queries borrow caller facts. Move the fixed caller
snapshot into the continuation only on an actual wait, preserving every scoped
condition, allocation frontier and prior attempt. Keep existing proof-reuse
publication rules. Body normalization resumes in its own numbering unit; global
probe identities belong only to disposable probes. Fresh excluded trial holes
may publish only after the entire unallocated envelope is reserved through the
existing streamed proof owner, preserving every exclusion and constraint.
Already allocated prefixes and changed numbering units remain errors. See the
[Field execution and dependency checkpoint](docs/parallel/field-offset-execution.md).

Owned unsized function parameters require actual crate permission and direct
parameter identity. A lazy pass-local, prepaid index is shared across body
checks; ordinary sized bindings build no index. Keep parameter ownership
separate from ordinary locals/results, nested bindings, capture environments
and explicit Sized duties. Check each compound unsized expression before
published-type deduplication: equal types do not imply equal place authority.
Imported by-value contracts retain mandatory Sized goals until genuine
permission, metadata ABI and ownership proof are transported. A source feature
request cannot authorize a borrowed-reference substitute for an owned DST.

Source constant-pattern admission consumes actual PartialEq and value-specific
StructuralPartialEq identities, independently of bootstrap derive properties.
Query applicability modulo regions as the pinned language profile requires;
publish no trait or call evidence from that classifier. Retire private witnesses
without assuming their debts, and preserve the value query's permission and
ordinary pattern-borrow proofs. Visit only the selected variant's fields.
Consuming a source derive list retains its actual completed API policy and the
name token's feature context; a literal wrapper cannot authorize a captured
attribute path. The [structural-pattern checkpoint](docs/pa31-structural-patterns.md)
records finite source/native controls and the remaining transport/ordinary gaps.

Source layout preparation owns a persistent execution-provider work budget,
shared across field normalization, instantiation, pending/warm queries, storage
readiness and ABI demand. Charge all attempted work, including failures, against
the existing cap before publishing results. Mirror actual deltas to cumulative
BodyWork observations; unrelated inference/selection observations do not spend
that layout budget. Neither warm reuse nor a pending dependency resets it.
Physical answers supply no caller, projection, lifetime or const permission.
The [source layout work checkpoint](docs/pa31-layout-work-owner.md) retains the
reproduction, independent source/native outcomes and spent-work opponents.
