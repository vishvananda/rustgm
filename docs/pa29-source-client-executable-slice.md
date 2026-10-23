# Source-client executable slice: concrete interface review

Source-only review on public604fe24a plus shared Call producer e091db86, replayed
as a13df8cb. No new wire descriptor, client policy or import publication is enabled.
The parked drafts remain unchanged in their evidence directory until this review
is applied. Their ScopeIndex algorithm is reusable; their incomplete row shape
is superseded by the complete variant below. The accepted full
[source body graph](pa29-source-body-graph-review.md) remains the requirement.

## First executable acceptance boundary

Use unchanged `pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs`
through provider checking, metadata encode/decode, actual source-client import and
native execution of unchanged runtime-client.rs (answer7). This provider contains
actual sizing roles, LegacyReceiver and both reference impls, Destruct, Freeze,
complete Clone/Copy, the actual u8 impls and unbounded identity<T>. Serialize and
reprove clone_from and u8 clone even though this first client demands identity.
No Native/bodyless replacement is used for either provided body in this slice.

The original const-client and noncopy-client (both7), unsized-client (reject),
full-attribute provider, all eighteen transport scenarios and full core/alloc/std
remain required. This first vertical family adds no Copy bound to identity.
Immediate opposing controls reject a missing actual Copy policy on demanded read,
an invalid unused clone_from body, omitted provided body, omitted conditional
Destruct premise, wrong Self/member owner, escaped region and malformed client
provider selection. A foreign row cannot be repaired by bootstrap fallback.

## One versioned payload, no fake legacy rows

Keep existing public legacy Rows and every v0/v1 wire field intact. Proposed
private storage and borrowed dispatch are:

```rust
enum PayloadData { Legacy(Rows), Source(Box<source::Rows>) }
pub enum PayloadView<'a> { Legacy(&'a Rows), Source(&'a source::Rows) }
impl Payload {
    pub fn view(&self) -> PayloadView<'_>;
    pub fn legacy_rows(&self) -> Result<&Rows, &'static str>;
    // Existing new(owner, legacy_rows, work) remains the legacy constructor.
    pub fn source(interface: &Interface, rows: source::Rows, work: &mut Work)
        -> Result<Self, &'static str>;
}
```

The existing optional boxed Interface payload remains the sole attachment.
Legacy source vectors remain direct. Source gets one cold row-container box;
there is no fake empty legacy function table, duplicate local-template Arc, or
new proof representation. Measure actual cold Payload/container retention before
adoption; Interface, Value24, Predicate12 and ordinary no-import allocation stay
unchanged. All readers dispatch once per artifact. A legacy-only consumer must
return an explicit unsupported-version error rather than skip Source rows.

The provisional source descriptor remains envelope9/version2/Rust2024/actual
roles. Assign no publication tag until complete roundtrip and ordinary proof of
the first executable closure pass. Rust2021 remains explicit unsupported here;
it is never inherited from the client. Old profiles0–8 and envelope9/v0/v1 retain
exact bytes and reject these additional forms.

## Exact table and identity domains

All indices are distinct wrappers. ArtifactRef addresses `[this artifact]` followed
by the Interface's exact sorted dependency identities; there is no duplicate
identity graph. DeclarationAddress is `(ArtifactRef, DeclarationRef)` and resolves
through that artifact's authenticated declaration catalog. Local slots, lexical
scopes, body sites and binder slots have their own namespaces. Local DefId,
TypeId, ParameterId, RegionId, NodeKey, Call-use rows and epochs never enter wire.

| Table | Complete row fields |
| --- | --- |
| Policy | defining Edition; exact provider ArtifactRef; present role range; explicit actual-source contract kind |
| Declaration | actual kind (module/trait/SelfType/function/type parameter/lifetime/formal/local); actual optional name; lexical parent; owner ordinal; visibility; declaration-site count |
| ParameterSlot | actual owner; full declaration ordinal; actual canonical type/region index; optional genuine declaration binding; type/region kind; for sized type slots, actual Sized DeclarationAddress rather than legacy permission |
| ParameterScope | owner; actual generic parent; contiguous own slots; complete captured-prefix count |
| Binder | owner; optional parent frame; Original/Public/Clause/CallableType/actual CallSignature(BodyRef,value) kind; exact count |
| Type | scalar; parameter SlotRef; reference(mutability, RegionRef, referent); callable(Signature); FunctionItem(FunctionApplicationRef); opened application signature |
| Region | Static; Bound(BinderRef,actual depth,slot); Parameter(SlotRef,actual depth); Variable(BodyRef,owned-region ordinal) |
| TraitApplication | exact addressed trait; subject TypeRef; complete ordered type/region arguments |
| FunctionApplication | exact addressed function/member; complete ordered type/region arguments |
| Clause | exact declaration/site; optional quantifier; full ordinary/Const/Conditional trait application, region pair or type-outlives; exact written-subject slot when supplied |
| TraitHeader | trait and genuine Self declaration/type; scope; captures; flags; complete members; written supertrait clause list; full input/declared-prefix ranges |
| ImplHeader | genuine SelfType owner; scope; exact self type; optional trait application; unsafe/const/negative flags; complete inputs and MemberLinks(trait-member address, optional actual override declaration) |
| FunctionHeader | declaration; actual parent/scope; original/public signatures and frames; complete total/early/late/captured region-slot scheme; complete formal declarations and actual receiver; input/declared-prefix ranges; declared const effect; explicit Absent or Original(BodyRef) |
| RoleBinding | ordered exact LangItem tag and actual provider DeclarationAddress; supported semantic attribute mask; separate diagnostic-only claims |
| Original | BodyRef, genuine declaration, result, site partition, actual local/parameter associations, scopes, projections, values, statements, blocks, arguments, exits and temporaries; exact owned-region declarations and call-request rows |

Trait flags use a checked mask (unsafe/const/auto and supported semantic attributes),
not eight bool fields from the parked draft. Unknown active bits reject. Source
fundamental/specialization/coinductive claims remain restricted to the exact
validated deny-impl sizing roles. Recording flags grants no solver cycle success,
implementation permission or const destruction.

Required clone has explicit Absent body and genuine formal declarations. Provided
clone_from, actual u8 clone and identity have explicit BodyRefs. Function/header
order is independent of body order: validate a one-to-one indexed mapping and
complete provided-body coverage. Callable recursion remains legal and separate
from the acyclic lexical/body-owner forest. A bodyless formal belongs to its
actual callable declaration, never to an invented BodyRef.

Canonical parameter-kind sizedness is structural original input only. A sized
slot must name the authenticated actual Sized role; import reconstructs its
actual source trait application as a caller goal, including unused formals.
Source scopes contain no legacy Copy/Sized clause substitute. Metadata validity
and the original Environment cannot pay the actual substituted caller goal.

## Actual operations and required goals

Source body value forms are Unit, Read(Place), Borrow(Place,Immediate), Constrain,
actual FunctionItem application and Call with exact signature/callee/arguments.
Place is Local or a checked prior Projection; first projections are Dereference
with exact reference mutability/region/referent. Statements preserve Project,
Evaluate, Store(Place,value), storage actions and initialized/mention checks;
Return/Jump/Unreachable and lexical cleanup tables remain exact. The u8 clone's
actual Read through a dereference is included. Unsupported operation kinds reject,
including For until separate into_iter/next context coverage exists.

Every Call retains `(original, value, site, target application, optional actual
trait-method request, opened signature, use effect)`. Export reads the completed
Operation::Call.source_use for the exact actual semantic owner and each real IR
Call's own target/application/signature. It never reconstructs effect from target
flags or one signature for a desugared source node. Imported effects are checked
against the authenticated original's context, independently of reached Const
payment. B owns that actual payment and continuation boundary.

Owned free-region rows come from exact original Variable declarations, retaining
scope/site and provenance requirements; no max-frontier padding is inferred.
Bound/static/parameter regions stay distinct. Derive mandatory formation,
reference/projection, reborrow/Constrain, argument/result coercion, trait, Store
replacement, initialization and cleanup goals from all operations. Preserve
additional source-original obligations and reject omissions/inconsistency; never
serialize Proved/Checked/Permit, a proof success bit or a source caller handle.

## Exact source-to-import ownership and methods

```rust
// Producer, consuming actual Checked originals and typed declarations once:
source::export(report, facts, checked_originals, interface, work) -> source::Rows;
// Resolver, private staging only:
SourceReservations::reserve(resolver, interface, source_rows, work) -> StagedRemap;
StagedRemap::authenticate(resolution, current_catalog, source_rows, work)
    -> AuthenticatedSourceRemap;
// Canonical metadata stage, no syntax fabrication:
SourceHeaders::ingest(imported, authenticated_remap, facts, work) -> SourceHeaders;
Trait::imported(current_catalog, authenticated_remap, typed_header) -> Trait<'_>;
// Original body owner, same immutable ImportedOriginals Rc as legacy import:
transported_originals::source_rows(headers, catalog, rows, facts, scratch, work)
    -> originals;
```

The remap owns dense artifact-declaration→real DefId and BodyRef→ImportedBodyId
indices and complete expected scope/kind/owner/ordinal data. It authenticates all
actual resolver declarations before catalog installation. Real trait Self and
impl SelfType remain distinct actual definitions; no DefKind::Impl or replacement
node is invented. Clause sites use sealed declaration origins; operation sites
use their exact BodyOrigin. Immutable original public signatures remain separate
from the actual owned execution signatures. Raw imported originals start with
execution=None; ordinary preparation may retain a distinct opened execution
signature, which must be represented by its exact checked transport rather than
overwritten with the public callable signature.

The associated Owner source proposal is Option<NodeKey> (measured32B owner/20B
kind), with an ImportedImpl canonical variant in the existing enum's payload
budget. None resolves only through the current existing origin catalog using
actual definition/self_definition, exact kind and authenticated coordinate. Apply
the same cold lookup discipline to implementation diagnostics. Source Some stays
a borrowed fast path; no second owner/origin map or fake source fallback.

Trait contract consumers need a borrowed source/imported header input view,
retaining source-owned Located vectors and written-Self NodeKey set unchanged.
Imported written subjects use authenticated declaration slots/sites; type equality
cannot turn an alias into written Self. Any cold enum/header size delta is measured
before adoption. Canonical source Predicate and all hot Body/Value fields stay
unchanged. Shared validators consume the current trait/role views and complete
canonical impl/member contracts, not a metadata-supplied validation bit.

## Client transaction and policy coordination

Requested Mode::SourceClient selects an exact provider dependency before bootstrap.
A private PreparedSourceClient is minted only from that dependency's validated
source payload, exact catalog/remap and complete header inventory. It supplies no
caller or execution evidence. The staged compilation uses no bootstrap roles,
prelude or library factories and never retries Bootstrap on failure.

Keep Package.source_core own-provider-only. Client language requests need a
separate cold request carrier, without copying library-feature names/capability
masks. Root owns Package.library and policy::prepare: coordinate the exact client
NoCore/request handoff there before adding its carrier. First unchanged no_core
clients need no implicit prelude. Do not claim ordinary source prelude support.

Reserve declarations and private root scopes; authenticate remaps; build canonical
headers/arguments once; validate real roles/impls; construct operations and all
mandatory goals; run ordinary promotion, eligibility, original/group and instance
proof; only then publish completed imported originals and executable mappings.
No alias/origin/role/executable row escapes a failed stage. Existing successful
artifacts retain their identities. Required absent members never become runtime
candidates. Source validity never pays a specialized caller or reached Const debt.

Source export uses Package::library_policy() and rejects all stability-bearing rows until exact
stability import/use consumers are implemented; no incomplete stable/const-stable
payload is emitted. The full-attribute target and actual library policy remain
required, separate from this semantic provider. All old profile golden tests,
foreign same-kind remaps, omitted unused goals/body, cross-owner calls/regions,
failed-budget rollback, cold/warm unpaid callers, and independent header/use/shared
DAG scaling remain qualification requirements. Work and aggregate old/new retained
and scratch budgets are charged before growth; no per-call whole header scan.

## Actual original phases and atomic export completion

The `source-model-third` observer retained the unchanged provider and passed all
five ScopeIndex controls. The paired generic `clone_from` and `identity` originals
were inspected successfully. Its added concrete u8 `clone` lookup failed at
`Checked::function_proof`, before observing that body: this index contains generic
executable templates, not ordinary concrete functions. This is a retained failed
phase hypothesis, not an unsupported provider or permission to omit the body.

`inspect_selected` proves generic originals before its selection callback.
`demand::prepare_ordinary` then constructs every actual nongeneric function,
including unused impl methods, from `body::concrete_prepared`. Dispatch,
promotion and constant eligibility precede the actual region/loan/cleanup proof
in `Builder::push`. After successful push, the Builder owns the actual immutable
`Proved` and the same local `Contract` is still available until the iteration ends.
The ordinary Proved does not retain that Contract and must not be relabeled as
the conditional-group `PairedProved` to fit an export API.

The corrected private observer borrows the actual stored Proved and that same
Contract immediately after push, scoped to exactly the actual u8 clone owner.
It records public versus owned signature and requires one visit. The generic
original assertions and original fixture remain unchanged. This observer is
test-only; it creates no runtime demand, proof row or persistent proof payload.

The production source exporter must split private staging from completion:
collect authenticated header ownership and generic neutral originals during
selection, append ordinary neutral originals from the successful post-push
boundary, and check indexed complete body coverage before finishing the payload.
The conditional-parent branch offers its corresponding boundary only after
complete original-group proof and all owned parent sessions succeed, before
paired Contracts are discarded. Both paths borrow proof owners; neither clones
proofs or retains all bodies a second time. The collected neutral rows share the
same aggregate work/storage budget and fail transactionally.

The staged artifact remains private until demand materialization, assembly and
lowering all succeed. Existing legacy export paths retain their current ordering
and bytes. An ordinary proof, later instance or lowering failure must discard the
entire staged source payload without publishing an Interface or executable map.

### Focused observation checkpoint

`obj/parallel/pa29-source-client-staging/source-model-third/observer-layout.stdout`
retains five passing scope tests and the failed generic-index hypothesis. Its
immutable private executable is retained under `pins/`, SHA-256
`792cb1326c76e0f37186b51e5b83b55eea3912eed9d758d1608acc7f7306dd0d`.
Fourth recovery passed the fresh architecture/file audit and stopped on the
test-only unused-self lint before execution. Fifth recovery added explicit RAII
retirement only, passed focused lib/tests strict Clippy, and ran the exact original
observer successfully; the previous audit and unchanged five scope cases stand.
Fifth's private executable SHA-256 is
`78227f80c0e9783f8c657b04565a7e4b29e592b4e74b1832fac6c12db656180a`.
All source maps, command inventories and failures remain in those directories.
No new runtime/native/text sweep was needed for this test-only observer change.

For this exact u8 clone, the actual public callable is one-bound-lifetime
`fn(&u8)->u8`, its execution signature is absent, and its Contract has one
universal. The body contains one parameter, reference dereference, read, four
statements and one return block. The retained raw normalization input owns
`[1..2]` and requires `free1 == bound0`; the independent source Copy query input
names actual Copy and u8. Neither relation can be inferred from the body rows or
omitted because the ordinary original already passed proof. This observed
signature pair is not a generic rule about every ordinary body.

The retained `clone_from` use-effect Ordinary is also an observation of the
then-current compiler, not the intended source rule. B's independent default-body
matrix subsequently established that a const-trait provided body must reject
nonconst calls at definition time. B owns the separate defining-context correction
without turning every ordinary impl into an executable const implementation. The
source profile must consume that corrected authoritative context before its wire
version can freeze; the previous trace remains preserved as pre-fix evidence.

The raw DTOs are still unpublished drafts. Before freezing their version, add
independent raw normalization inputs (exact owned intervals, diagnostic sites,
equalities, outlives and type-outlives) and source Copy policy/query inputs.
Quantified input beyond the supported binder form must reject explicitly.
These are obligations and allocation ownership, never serialized proof success.
Copy queries must be rerun in the exact environment; an unbounded identity move
does not acquire a Copy bound from having queried Copy.

### Approved collector shape

The private driver will retain its complete legacy wrapper and introduce:

```rust
fn inspect_staged<S, R>(
    report: &Report,
    facts: &mut Declarations,
    prepared: Prepared,
    select: impl FnOnce(&mut Declarations, &templates::Checked<'_>)
        -> Result<(Option<DefId>, roots::Exports, S), Error>,
    collect: impl FnMut(&mut S, &Declarations, &incremental::Proved, &Contract)
        -> Result<(), Error>,
    finish: impl FnOnce(&mut Declarations, Assembled, Selected<'_>, S) -> R,
) -> Result<R, Error>;
```

`inspect_selected` supplies S=() and a no-op collector. A cohesive borrowed
original-context carrier groups the existing checked templates and collector
through `demand::prepare`, `prepare_ordinary` and `paid_owners::build`, avoiding
an extra argument in each already full proof function. Ordinary callbacks borrow
the stored Proved with its exact successful local Contract. The paid path checks
the retained input pointer against its existing Arc index before owner extraction,
then collects only after all actual group proofs and parent sessions succeed.
Both paths reject mismatched owner/key/origin/signature; neither reconstructs a
paired proof or repeats verification. Neutral stage completion and executable
lowering remain separate gates, and source payload publication requires both.

### Required independent raw input rows, not yet encoded

The following addition is a design prerequisite for the source version, not an
enabled field layout. Index a complete `BodyInput` table by BodyRef independently
of operation rows. Each row retains the exact original universal frame, parameter
slot references, explicit region and type assumptions, original formation/coercion
obligations and raw normalization inputs. Input lookup always includes its actual
body owner; a header binder, scope envelope or coincident numeric region is not
allocation ownership.

Build one per-body allocation index from actual Variable declarations and each
raw normalization input's exact `owned_regions()` intervals. Record which input
owns each normalization slot. Preserve disjoint intervals and reject foreign or
overlapping ownership rather than padding a frontier. Every variable reference
then addresses that body's explicit dense allocation slot; header-bound, static
and parameter regions retain their separate representations. Import allocates
fresh IDs for those exact rows and translates all labelled equality, outlives and
formation goals through that single checked map. It never serializes a local
region number, allocation envelope, solver result or proof token. Unsupported
quantified input must reject before payload completion.

Body builtin input separately retains the actual Copy declaration-or-absence and
complete queried type applications. Validate query coverage against actual body
reads and reselect it in the exact imported environment, preserving the ordinary
move fallback. These queries neither assert Copy nor replace mandatory builtin
predicates. For this provider, u8 clone queries Copy(u8), while identity queries
Copy(T) without declaring a Copy bound.

Preserve three environment states explicitly: absent, the exact declaration
environment, and the environment derived from the original Contract by the
existing Copy preparation path. The u8 declaration has no separate body
environment; Copy preparation derives a present empty one from its Contract.
Neither a missing row nor an arbitrary same-shaped environment can stand in for
that derivation. A declaration with no environment cannot import additional
assumptions merely because the Template API stores an environment Arc. All
imported premise data must be checked against the actual complete declaration
header or the exact existing contract-derived construction before original proof.

### Private collector validation before binding factoring

The first staged-flow gate (`obj/parallel/pa29-source-client-staging/staged-flow-first/`)
passed fresh formatting, architecture/file audit, all-target strict Clippy and
six exact private cases. Execution took0.083s; this is not an A/B performance claim.
The cases cover partial collector failure, a real late uninitialized original,
actual faulting conditional child, completed paid group, cloned same-shaped
foreign Contract rejection, legacy unit-state exact LowIR/CFG/canonical-work
agreement, and the unchanged real-original observer. The explicit pointer check
uses actual solver-owned Contracts and completed proofs, not fabricated pairs.

The immutable private binary is `pins/rustgm-lib-tests`, SHA-256
`ae9bbd4fd393c391bfc28a8bc0fc3dda45b490ff22d8e049de4d385b7df20a35`,
354,795,400bytes. The previous observer binary was about322MB. New tests and
generic collector code both changed; the full difference is not attributed to
either without a comparison. A source-only followup will move collector binding
outside final-result generic scope and give the legacy no-op a concrete shared
function identity, preserving these successful observations and all semantics.

The factoring is now qualified by `staged-flow-second/checks.json`: fresh fmt,
audit, all-target strict Clippy, library build and the same six exact cases all
passed (0.060s execution). Binding depends on S/F, not final R, and legacy uses
one non-generic no-op callback. The private executable is341,387,864bytes,
SHA-256 `376f9bcd355d7398dee70d75a1bd502e11a92a27731a831f0a60a4bb859d19d4`.
These two observations differ by the reviewed binding factoring and its test
adaptation; no compile-time or runtime speed claim follows. The common context
remains two borrowed pointers, enforced by an early static layout assertion.
