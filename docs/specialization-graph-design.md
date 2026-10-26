# Specialization graph proposal and verifier checkpoint

The initial concrete proposal below records the reviewed design from
qualified64c9f507. The later verifier-checkpoint and correction sections describe
the current unqualified production implementation; the full dispatch draft stays
preserved on pa31-specialization-graph-v1. New observations40237725892 supersede
only incorrect predictions, preserving their source bytes and raw results.
Unmeasured sizes remain explicit; logical budgets use actual types/capacities.

## Private types and ownership

Proposed Rust shapes, all private to sema specialization except an opaque checked
resolve policy result; no public constructor/key or wire representation:

```
struct EntryKey { index: u32, owner: DefId }
struct PairKey { first: EntryKey, second: EntryKey } // ordered registry indices
struct VertexId(NonZeroU32);
struct EdgeId(NonZeroU32);
struct Work { steps: usize, bytes: usize }
struct Graph {
    arena: types::arena::Identity,
    requests: CheckedCrateRequests,
    work: Cell<Work>,
    state: State,
}
enum State { Building(Box<Builder>), Sealed(Box<Forest>), Failed }
struct Builder {
    sources: HashMap<EntryKey, ImplWitness>,
    attempts: HashMap<PairKey, Comparison>,
    relations: Vec<Relation>,
}
enum Comparison {
    Pending(Box<Attempt>),
    Disjoint,
    Ordered(EdgeId),
}
struct Relation {
    child: EntryKey, parent: EntryKey,
    parent_arguments: GenericArgs, // canonical, expressed in child's identities
}
struct Forest {
    vertices: Vec<Vertex>,
    parents: Vec<Relation>, // only immediate checked edges, not all ancestors
    by_entry: HashMap<EntryKey, VertexId>,
}
struct Vertex {
    witness: ImplWitness,
    parent: Option<EdgeId>,
    depth: u32,
    preorder: u32,
    subtree_end: u32,
}
struct ImplWitness {
    entry: EntryKey,
    source: NodeKey,
    image: Arc<OriginalImage>,
    syntax: ItemKind, // exact actual Impl snapshot, not a name/hash
    predicate: Predicate,
    admission: Admission,
}
```

Only `Implementations` gets one optional Box<Graph> (+8B cold registry field).
Registry entry/Implementation/Function, hot syntax/resolution fields and
Report/Engine stay unchanged. Keys are constructed from the existing owner->entry
index and verified against actual owner/source/predicate/polarity. The graph is
owned by that one registry/facts arena, never cloned/moved to another registry or
exported. Impl source/current image/actual associated owner are authenticated at
production/seal and source-dependent lowering boundaries; arena/key checks are
still mandatory on internal immutable graph reads. Store append-only growth is
allowed; header mutation after seal is rejected.

Builder is dropped at sealing: all transitive proposals, completed pair memos
and pending inference state disappear. Forest retains only actual local witnesses
and immediate canonical parent mappings. Work/request witnesses survive this
transition, including failed-attempt charges; there is no fresh zero budget.
A failed proof returns its original canonical error and makes State::Failed
unusable; no partial executable graph survives an error.

## One early/late comparison owner and payment

Keep one canonical `compare_pair` reached by current validate_impl_pair from both
ordinary validation and early scalar readiness. Ordinary disjoint pairs keep the
existing queue/removal/no-Graph fast path. An ordinary overlapping final family
still fails without creating a usable graph.

Existing early Pairs::take removes pairs before comparison. On a possibly
specializing overlap, hand that pair once to the graph's PairKey slot BEFORE a
query can suspend. The queue can then retire its own slot because Graph owns the
remaining obligation. A repeated early/late request finds the same slot; it
resumes Pending or reuses completed outcome, not reruns overlap or implication.
The final validation drains graph Pending slots in addition to remaining old
queue entries. Missing/unprepared headers suspend with actual owner dependency;
they do not become a permanent negative result.

Attempt owns direction/cursor, rigid child owner, fresh-parent inference Context,
canonical parameter bindings, real declaration Environment, remaining typed
clauses/region obligations, optional selection::Pending and cumulative work plus
last_paid_work. It uses the existing counted resume pattern in
prerequisite_queries::Attempt. Resume charges ONLY the new work delta. The one
central charge function also mirrors that delta into existing body_work; caller
wrappers do not charge the same overlap/proof total again. Pair map probe/replay
work is separately charged, including successful memo hits and failed probes.
No inferred substitution or child evidence escapes until both orientations and
all parent obligations/regions complete.

The source registry inventory is append-only while building. New compatible
headers invalidate readiness by adding real pairs; no sealed graph is mutated.
A family cannot be used for specialization dispatch while compatible/unknown
pending headers or comparisons remain. Existing pending-head query machinery
continues to block affected candidates. Unrelated ordinary non-graph early
families retain their existing readiness path. Executable graph publication waits
for checked family inventory and conformance, not a second-candidate shortcut.

## Immediate parent, effective finality and ancestry

After all compatible pairs are complete, use their ordered memo relations. For
each child's proposed parents, fold a nearest candidate in O(number of proposed
parents): replace it only when the new parent is a proven strict descendant of
it. Every compared parent pair is itself an existing compatible coarse-head
pair; a missing relation is pending until inventory/comparisons finish, then
incomparable parents are a coherence error. Do not run a new all-impl crossproduct.
Reject cycles/equivalent/crossing relation results before publication.

For child->broad and child->middle plus middle->broad, retain child->middle and
middle->broad, with each mapping directly expressed in its own child variables.
Compose ONLY on a demanded walk using existing substitute::Bound and full
canonical arguments/captures. Canonicalize/charge each composition; never copy
ancestor bodies or flatten inherited environments/members. Immediate-edge min
checks run after this choice, matching pinned check_min_specialization's nth(1).

NEW actual correction: omitted inherited members of a normal intermediate impl
are effectively final for later overriding. The two original three-level
predictions failed with E0520 on both editions. Therefore checking only the
nearest ACTUAL defining ancestor's written default is insufficient. The forest
must retain/derive effective finality at every impl boundary. An explicitly
written default child member can remain overridable; an omitted inherited member
on a normal child closes that member for descendants. A default impl would have
separate propagation rules and remains explicitly fenced pending that required
consumer. The corrected positive chain explicitly reopens middle f as default.

Seal builds adjacency once, checked parent/depth and a DFS preorder/subtree_end
for the VALIDATED graph forest. Ancestry comparisons are O(1) on those derived
indices after actual EntryKey lookup. These are not source/AST span intervals or
hash-based ancestry claims. Construction checks parent edges/cycles and visits
each node/edge once; no vector of every ancestor is retained.

## Shared defining-body result and memo lifetime

Proposed result:
`SelectedMethod { definition: DefId, defining_impl: Option<DefId>, arguments: GenericArgs }`.
None identifies a real trait-provided body, not a missing implementation.
Only authenticated direct/parent member definitions and composed args can create
this result. Dispatch Arguments::target and trait-result/RPIT alias selection
consume the SAME resolver/result. Other method consumers (including cloning) are
audited; unsupported output projection cases remain explicit errors.

No new persistent method-result memo initially. A resolution walks only immediate
parents, with a256-depth retention limit and charged substitutions. Existing
Dispatch.selected and projection query caches retain their established facts/
canonical-type lifetime. Any future shared memo must key graph identity/selected
EntryKey+canonical arguments+declaration and preserve failure/work charges; an
owner-only or rendered-type cache is forbidden. Candidate preference uses sealed
ancestry only after both independent trials actually succeed; losing trial
inference/evidence is never committed. Unordered remains Deferred.

## Exact mode provenance: admission is not Min checking

Proposed opaque resolve-produced policy (not a name-only boolean):

```
struct CheckedCrateRequests {
    root: NodeKey, image: Arc<OriginalImage>, min: bool, full: bool,
}
enum Admission {
    CheckedCrateRequest,
    ExactImplToken { site: Span, context: SyntaxContextId, feature: AllowedFeature },
}
enum AllowedFeature { MinSpecialization, Specialization }
```

Requests come from the existing once-parsed/authenticated root feature producer.
Actual graph admission queries use the specializing impl's REAL typed Node span
(the analogue of pinned tcx.def_span), derived from its actual Item/owner, through
existing MacroAllowanceSession/Query. No caller-supplied tuple, spelling search,
new parser token scan or wrapper/ancestor allowance is authority. Retain current
SourceImage and exact context with the checked result; Attribute roles cannot
supply an executed allowance frame. The already-qualified active default keyword
gate remains a separate exact-keyword contract.

Pinned specialize::specializes permits checked crate min/full OR actual impl-span
min/full allowance. Pinned impl_wf_check.rs71 runs always-applicability whenever
the CRATE min flag is true, even with Full also set or a literal Full allowance.
No max(Min,Full) mode. Actual new observations corroborate Full-only structural
static ACCEPT, Full+Min/min+literalFull static REJECT, and Full lifetime-only
order still conflicting. Full never waives ordinary lifetime WF/constrained
parameter/coherence/implication/region rules. New literal impl admission passes;
forwarded caller impl admission rejects. Primary file hashes are retained under
primary/; no inference from a parent's keyword grants a child's impl authority.

## Distinct sparse local unsafe marker owner (review needed)

Propose ONE optional boxed LocalMarkers in existing cold must_use::Contracts
(+8B there; Declarations still its existing pointer). It is distinct from the
registry Graph and existing sizing HeaderAttribute::Specialization. This extra
cold choice is submitted for review before adoption.

Create the owner when this analysis has an active sparse default modifier or
encounters the marker. Reuse the EXISTING configured-owner/effect pass and
existing bind_definitions loop, without a new whole-crate owner scan: consume
marker forms once and bind each real local trait's completed Present/Absent row.
Source-owned trait rows retain DefId, declaration/owner NodeKeys, exact syntax,
OriginalImage, owner effect ranges and optional marker effect/span. Complete only
after actual local trait binding/validation; unvisited/missing is Unknown.

Only the actual trait with checked rustc_attrs/token authority and valid word
form gets Present. Wrong owner/form/gate is rejected as independently observed;
literal marker allowance succeeds, forwarded caller attribute rejects. A
same-spelling unmarked trait is Absent and gets no predicate exception. Imported
traits are Unknown regardless local map miss and explicitly denied; no old
profile/schema/external kind is changed. The ordinary sizing specialization flag
does not stand in for this marker. The consumer exempts ONLY the corresponding
actual trait predicate from Min's restriction, not from proof of that predicate,
its supertraits, or the parent obligations. Actual Clone/Drop selection/body
proofs stay required.

## Bounds and measurements before claiming qualification

Proposed limits: existing1m compatible pairs; at most1m retained comparison slots,
65,536 graph vertices/immediate parents,256 depth; graph10m accumulated work and
64MiB retention budget. These are diagnosed resource limits, not language rules.
Prepay Box owners, map/vector growth and simultaneous inference/record buffers
before allocation. Charge both directional successes/failures, pending resumes,
canonical substitutions and memo warm reads. Source-image Arcs are retained, not
copied source bodies. Failed/pending state never resets the envelope.

All new sizes/capacities are pending real measurement. Measure Registry (+8
proposed), cold Contracts (+8 proposed), Graph/Builder/Forest/Attempt/Relation/
Vertex/marker row, actual pair and selected-parent capacities. Preserve all hot
layouts and never-default/no-marker no-owner allocations. Scaling includes
ordinary8/32/128 nonoverlap buckets and real chain/fanout with cold/warm/failed
queries; no performance or allocation-count claim is inferred from oracle time.

## Member-specific proof and source progress

The successor remains uncompiled and unqualified. Source-only save `a10e9c73`
retains the graph/proof/marker draft; `e7ce67ce` cleanly merges published B v2
`a598b799`. Both its module-contract publisher and v1 still use the shared
selected-header/defaultness/marker presence boundary. No new public profile,
executable graph or imported marker/defaultness authority is inferred.

The new [member-demand contract](../pa31/tests/spec/specialization-graph/member-demand/contract.json)
freezes the exact Full own-child/static-parent opponent. Hosted run 40245869391
uses the unchanged pinned workflow and qualified public compiler baseline;
its two metadata and two actual native observations pass in 0.151553 seconds,
with `2\n` and no warnings. The 6 command statuses and every output hash are
retained in that contract. No candidate graph ran. The 4343-byte artifact
12617824460 has SHA-256
`a0bf4c60e859eb355b203b33596173058c3365a668d139bbdbc97ad8d2956d67`;
binding SHA-256 is
`4bfd4f102843928062ac1948a3afc25f092b6e4b12adce4f3d0b71048d39014d`.
The raw ZIP is authenticated in memory; guarded local hydration awaits recovery
above the unchanged 150 GiB reserve.

This result rules out blanket ancestor requirements during implementation
selection. Only an actually requested inherited member/result may demand its
actual defining header. Both public normal-form Query and the selection engine's
already-boxed Normalization need their own optional cold continuation (+8 bytes
each, still awaiting actual layout measurement). They share one typed
`Conditions { clauses, outlives }` producer, including implicit Sized, full
ordinary/quantified clauses and schema region pairs. A selected own child body
never demands the unused parent's header. No synthetic parent Self:Trait query
may authorize an inherited body. Existing pending scopes/proofs must reach the
ordinary final lifetime/body proof before result publication.

The source draft now has retained pair attempts, immediate-parent reduction,
ranked ancestry, parent-first conformance slots and member finality at each
immediate boundary. It also factors charged/no-op adapters over the existing
nominal, trait-application, capture and substitution producers. Growth callbacks
prepay typed collection/copy work and logical bytes before mutation; existing
shift/fold counters retain failed work. These changes add no fields to Store,
Bound, Requirements or Proof. Exact layouts, capacities and allocation-path
invariants remain unmeasured until the focused gate.

Capped multiplicities use one canonical demanded DAG with Kahn propagation,
including real child capture slots under their actual binder depths. Duplicate
edges remain distinct; function-item signatures are metadata rather than a
second set of generic inputs. Sole keeps its existing fields, set summary and
ordinary edge behavior. The bound is O(V+E+R+P) logical work/storage for demanded
(node,binder-depth) keys, edges, roots and parameters with ordinary hashing;
it is not an adversarial hash bound. Shared diamonds, many parameters, nested
capture slots, function-item inputs and failed payment controls are authored
and unrun. All prior 74/40 independent observations and 8 wrong chain predictions
remain unchanged; the new opponent adds only 2 metadata and 2 actual natives.

Graph execution is still closed in this draft. Required work before its focused
candidate gate includes requested-member resolution/substitution, both RPIT
continuations, dispatcher proof joins, coherent conformance/selection wiring
and actual mode/marker/finality/storage controls. Wider quantified implication,
projection-constrained Min parent arguments, associated-value/default-impl
specialization and imported graph/marker transport remain explicit required
followups rather than inferred absence or discarded goals.

## First verifier component (focused qualification)

The successor `pa31-specialization-verifier-v1` keeps the full dispatch draft on
`pa31-specialization-graph-v1`. The component connects actual coarse-head pair
checking to the one paid comparison memo, a closed immediate-parent forest,
crate-Min admission, conformance ordering and member finality. Omitted inherited
members are final at their immediate boundary. No successful analyzed facts or
artifact can escape this component: a completed graph returns the explicit
`specialization graph execution is not implemented` diagnostic. This is a
verifier checkpoint, not implementation of executable specialization.

One optional Graph belongs to the registry. Its shared Rc budget retains failed
work across pending queries, closure and failure. Local marker knowledge belongs
to the existing optional semantic contracts owner; genuine publication still
rejects present untransported markers, and imported missing metadata is Unknown.
No syntax, item, scope, definition, Store, Requirements or Proof field is added.
Actual enclosing sizes, graph/marker capacities and logical work/byte charges are
printed by the selected controls and remain unmeasured until qualification.
Logical retention accounting is not a measured allocator count.

The counted nominal producer shares the existing canonical traversal. A failed
query rolls back only its prepaid newly inserted visitation keys, preserving
previously completed dependencies and cumulative failed work. Shared canonical
clause caches may retain fully computed results; incomplete roots cannot become
cached absence. The ordinary uncharged path retains its existing behavior.

Minimal-specialization repetition uses one demanded canonical DAG with capped
path multiplicities, shared with Sole's parts enumerator. Collection is
O(V+E+R+P) logical work under the existing hashing policy, with V distinguished
by demanded node and binder depth. Duplicate edges and roots count; a shared
node does not cause a parameter-by-node result table or a traversal per parameter.

Focused hosted plan, six exact private groups only:

- `sema::specialization::component_tests::marker_authority_retains_actual_source_and_rejects_invalid_or_forwarded_requests`
- `sema::specialization::component_tests::marker_publication_is_closed_and_imported_contracts_remain_unknown`
- `sema::nominals::requirements::counted_tests::counted_formation_replays_real_clauses_and_rolls_back_only_failed_visitation`
- `types::parameters::uses::multiplicities::tests::specialization_counts_shared_diamonds_many_parameters_and_failed_work`
- `sema::declarations::specialization_tests::specialization_verifier_checks_immediate_finality_min_authority_and_closed_execution`
- `types::parameters::uses::sole::tests::default_occurrences_share_all_growing_prefixes`

Production and private strict Clippy run sequentially and both retain findings;
architecture runs alongside. A static failure prevents build/test execution.
The total hosted development gate stays below 300 seconds. No independent oracle
set, broad private/native suite or source-core observer is repeated here.

Executable continuation remains required: one checked SelectedMethod must feed
method dispatch and both RPIT normalization consumers. Only the actual requested
inherited member may demand its defining header and translated substitutions;
ordinary own-child dispatch must not collect unused ancestor debts. The retained
member-demand opponent confirms the own child method avoids the unused parent's
static bound. Full obligations, regions, schemas, callable/body proof, pending
state and source authority must be retained before execution opens. No Provided
or graph wire transport is granted by this component.

### Retained verifier observations and correction

Hosted40252035090, source b3b21245, passed metadata, format, global architecture,
production and private strict Clippy, CLI/private builds, and five of the six
selected controls. The actual verifier group failed its first corrected chain:
Min admission omitted structural WF clauses and rejected the implicit child
Sized obligation. This is a real producer deficiency, not a revised language
expectation. No graph/native/executable qualification follows from that run.
The independent graph/member-demand source bytes and all disagreements remain
unchanged; this correction runs no independent oracle again.

Observed logical costs from that exact source (not allocator counts):

| owner/control | widths or state | actual values |
| --- | --- | --- |
| LocalMarkers / Contracts | bytes | 88 / 448 |
| marker rows | 8 / 32 / 128 | capacity14/56/224; work194/846/3406; logicalbytes7912/33704/134088 |
| demanded DAG | widths8/32/128 | nodes18/66/258; edges27/99/387; work331/1311/5207; logicalbytes11984/48016/190032 |
| ordinary Sole / Requirements | bytes | 64 / 128 |
| shared Budget | bytes | 8; failed alias work10000002 then retry10000004, same owner |

The correction uses a distinct FormationRequirements wrapper around the same
Requirements state and traversal, preventing nominal-only visitation from
implying structural completion. Its shared Shape predicate emits Sized only on
array/slice element edges and non-tail tuple edges. Reference referent-outlives
clauses retain their actual binder. Bare parameters, pointers and callable
signature positions gain no blanket Sized. Wider dynamic/projection/closure/item
formation remains explicit. Pinned rustc wf.rs750–850/917 supplies these rules
(SHA2560f0f029b52e543386ba875d931dc8ea4e3ba3759a7c191923a51e0bdeff20a3d).

Before integration, dynamic environment and clause-index storage must also be
prepaid. The successor's nonbreaking charged adapters share the original field
mapping, structural fingerprint and canonical clause insertion algorithms.
They pay actual Arc copies, map/set/vector and bucket growth before allocation;
region equality reconstruction derives its bound from actual equation counts,
typed graph/SCC buffers and output maps. Failed callbacks remain charged.
The declaration adapter requires a lifetime-only folder and rejects an already
opened caller rather than materializing unpaid body/import opening state.
Ordinary adapters retain their original general behavior. No fields are added
to Environment, Shapes, ClauseIndex, Requirements, Proof, Store or hot nodes.

A budget failure during lazy ClauseIndex construction never installs a semantic
Err/empty result into the immutable source's OnceLock. Success alone publishes
the complete index. The same counted group now contains an actual captured
source caller with two real trait bounds and a quantified builtin: failed
mapping preserves the original; retry shares cumulative work; a partially built
index fails, retries both real rows and then reuses the same warm Arc. Structural
formation controls explicitly oppose arbitrary generic/reference/pointer/fn
Sized assumptions and callable-local lifetime escape. The successor remains an execution-fenced verifier; its focused qualification
and observed costs are recorded below.


### Qualified verifier source `9147a49a`

Hosted run 40256578419 is GREEN on source `9147a49a07af941991b84601492d64c8c2082fef`,
CI commit `ee0dd17aab0ff72428bac7b27b09936ea7163bfa`. All 2,689 frozen fingerprints
and all independent-fixture hashes match. Metadata, formatting, global
architecture, strict production and private Clippy, CLI/private builds and the
same exact six groups pass; six selected tests execute in 0.05 seconds. The helper
used 175.8 seconds within its 180 second work budget. Downstream harness, audit-cache
and upstream checks also pass. No reference, oracle, native or full-suite repeat
was used. Earlier failures remain retained and this checkpoint claims no native
specialization behavior.

Actual nocapture figures on that exact source:

| object/control | measured size or logical cost |
| --- | --- |
| Graph / registry / Budget | 56 / 240 / 8 bytes |
| LocalMarkers / existing cold Contracts | 88 / 448 bytes |
| Environment / Requirements / FormationRequirements | 64 / 128 / 128 bytes |
| three-vertex chain, either declaration order | 2 edges, depth 2, capacity 3; work 441, logical bytes 13,736 |
| actual unsafe-marker two-vertex family | 1 edge, depth 1, capacity 2; work 297, logical bytes 7,432 |
| Full structural-static two-vertex family | work 155, logical bytes 5,136 |
| literal Full-authorized family | work 177, logical bytes 4,896 |
| mapped captured caller | retained owned payload 1,372; cumulative logical charge 3,272; work 145/folder 18; failed index 13, retry total 26 |
| nominal rollback/retry | prior completed 2, final visited 6; failed work 4, total 41; logical bytes 208 |

Marker and DAG width 8/32/128 figures above are reproduced exactly by this run.
Ordinary Sole's growing-prefix counts 16/128/1024/4096 are 115/899/7171/28675.
Ordinary declarations retain no Graph; no-marker/no-default declarations retain
no LocalMarkers owner. Source-forgery, missing/forwarded authority, malformed
markers, actual v2 publication rejection and imported Unknown opponents pass.
Closed-forest insertion rejection and every immediate-boundary finality/Min
opponent pass. Failed budget aliases and partial lazy-index failures keep their
charges and cannot turn into checked absence.

The measured unchanged enclosing sizes are Syntax 312, Unit 688, Report 3040,
Node 36, Item 64, Scope 32, resolver Definition 36 and NodeFacts 80. No hot field or
persistent counter was added. Graph and LocalMarkers are the approved optional
cold owners. These are Rust type sizes, actual capacities and logical work/
payload accounting; they are not allocator-call counts, peak heap measurements
or a replacement for later performance qualification.

Metadata-only retained evidence lives in Worker C
obj/parallel/pa31/specialization-graph-v1/qualified-verifier-40256578419.
Fastcheck artifact 12622213761 (297,438 bytes) has ZIP SHA-256
f17aeae43761df6223cd5e5c9e07ecbd3093ea7a140a0ee9ae132e5fc67f0369;
its binding SHA-256 is a26272f768485cdb9659e223b30122319c9436a01bc2b41f048838f1081dc31a.
The compiled CLI hash is 3ae4b3c6127a962328454d5e0c13b140e7fe3b79e58936aa35725ba630237f47;
no executable was hydrated locally. The explicit execution fence remains in
place until selected methods, actual defining-header conditions, dispatch and
both RPIT continuations are complete and independently qualified.

### Execution successor source draft

The separate execution successor preserves the qualified verifier and its
terminal execution fence. Its new consumer code is uncompiled and unqualified.
`SelectedMethod` authenticates the actual requested trait member and complete
selected implementation arguments, walks only sealed immediate-parent edges,
and returns the actual defining member, implementation and translated arguments.
The defining-header producer remains shared with verifier admission. A child
which supplies the requested method collects no unused ancestor conditions.

Concrete dispatch retains the actual defining callable arguments and uses the
unchanged checked original/instance Contract path. It does not append the defining
impl's lifetime bounds as new caller requirements. Outer projection normalization
has a counted canonical condition continuation. Search normalization descends the
same producer's clauses on its existing iterative stack, then retains a composite
of the selected trait proof, actual defining-header children and schema outlives
debts. Family formation and inherited-result children share one binder-opening
kernel; its ordinary mode uses the original uncharged operations. No blanket
ancestor predicates are added to Trial. Projection region debts still reach the
existing body/scope proof joins.

New persistent storage is limited to one optional pointer on the cold outer
projection Query and one on the already-boxed search Normalization. New body-proof
owners, suspension shells and condition collection growth use the existing shared
graph budget. No source, item, scope, definition, Frame, Node, body, Store or ordinary
Requirements field is added. Layouts and actual changed-path charges remain
unmeasured. Complete source-incarnation/mapped-body authority, failure and replay
controls, quantified proof joins, both normalization consumers and actual native
selection must qualify before removing the fence. Existing marker/defaultness
publication guards and every imported/Provided execution boundary remain closed.

The [six new independent sources](../pa31/tests/spec/specialization-graph/execution/contract.json)
resolve a phase distinction which the original draft did not model correctly.
All 26 commands, four actual native executions and twelve wrong positive
metadata/build predictions are retained. Scalar dispatch accepts an inherited
static-parent method with a nonstatic child. A final inherited RPIT method is
accepted across specialization of a different scalar member, but the combined
RPIT/static-parent/nonstatic-child source is rejected for a real lifetime debt.
An actual default RPIT method is rejected even if the impl writes a concrete
return type; the conformance check must bind the actual trait result declaration.

Pinned [instance resolution](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_ty_utils/src/instance.rs#L17-L30)
normalizes instance inputs erasing regions, chooses the actual leaf definition,
translates its arguments under the post-analysis environment and erases the
result's regions. By contrast, [associated projection](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_trait_selection/src/traits/project.rs#L1958-L1976)
translates under the projection's actual parameter environment and retains the
associated item's own obligations. In RustGM, concrete demand already passes the
exact selected Instance, Template and original Contract through
`from_checked_caller`; no specialization-wide lifetime exemption or extra body
field is introduced. Source-incarnation and original/instance proof remain
mandatory. The new candidate tests and all candidate native executions are pending.

The first consumer checkpoint keeps public execution fenced and selects six
private groups. Each uses real sealed Checker facts after asserting the terminal
fence; no test-only production permission or fabricated implementation is added.
The body/instance group uses the unchanged pipeline and lowering proof and drops
the resulting program. That is consumer proof, not native execution or publication.

- `sema::declarations::member_execution_tests::sealed_method_dispatch_selects_nearest_defining_owner_and_complete_arguments`
- `sema::declarations::member_execution_tests::inherited_scalar_dispatch_preserves_nonstatic_caller_and_actual_definition_context`
- `sema::declarations::member_execution_tests::default_rpit_rejection_binds_trait_declaration_and_keeps_final_inheritance`
- `sema::declarations::member_execution_tests::checked_defining_bodies_and_instances_keep_original_contracts_while_public_execution_stays_fenced`
- `sema::declarations::member_execution_tests::selected_member_rejects_wrong_trait_arguments_arena_and_graph_owner_and_keeps_failed_work`
- `sema::declarations::member_execution_tests::rpit_evidence::both_rpit_consumers_keep_final_inherited_identity_and_real_parent_lifetime_debt`

Both strict compiler modes and the independent architecture audit precede any
build/test execution. No prior oracle, full private inventory or native matrix
is repeated in this checkpoint. Actual cold layouts and changed-path costs will
be retained from the selected controls; they are not yet measured.


The member-consumer checkpoint `4b195c57` is qualified by hosted run
40267493521: all six nonignored groups passed, with strict production and private
Clippy, metadata, formatting, architecture and the private build. Both receipts
match all 2,719 frozen compiler inputs and all 31 declared fixture/data hashes.
The helper completed in 150.41 seconds; no candidate native execution occurred.
The terminal execution fence remains present in that qualified source.

Actual cold sizes are Query 912, boxed Search Normalization 520, Frame 1,312,
Node 160, SelectedMethod 32 and ProjectionRequirements 144 bytes. The inherited
RPIT controls retain clause capacity 4, region capacity 0 and Search capacities
(4, 0, 1). Chain and reverse-chain dispatch consume 42 and 45 charged work
steps respectively, each with 264 logical retained bytes; failed work advances
10,000,464 to 10,000,465 on the same budget owner (13,896 logical bytes).
These are type sizes, capacities and logical budget charges, not allocator counts
or peak-memory measurements. The CLI SHA is
`5d608942f7b52bec94e455d5c5d6023bf0c244de6edea4129acf941a736609a9`.
The primary receipt SHA is
`2797bb924f3b3d54fa275accdb422a9004b7822a4c7376ebeef7b497f1c33ef5`;
the independent audit receipt SHA is
`ab752f2c3e9466bf37087ffa755d1554143f0d6f60ed4e51ed09d6a25feb2b80`.


The separate execution candidate replaces the terminal unsupported diagnostic with
a current sealed-registry check at declaration completion. That check pays for
validating the retained forest against its actual registry. Member consumers still
require their own selected declaration and complete defining arguments, and use
the unchanged original/instance proof. Default implementations, associated
values/types, imported specialization and public defaultness transport remain
closed by their independent guards. A sealed forest does not grant those features.

Before candidate execution, the bounded native plan declares seven exact providers
in both editions: both corrected chains, actual unsafe marker selection, generic
private parent substitution, selected clone/drop and lazy panic, inherited scalar
static-parent/nonstatic-child, and final RPIT inherited across scalar specialization.
All fourteen must compile with nonempty LowIR and empty compiler diagnostics,
then pass the locked backend and execute with status zero and empty streams.
The three actual default-RPIT declarations must reject in both editions; the
combined inherited RPIT/static opponent must reject with real lifetime evidence.
These eight negative observations require exit one, empty stdout and the declared
diagnostic category. No old independent oracle is rerun. The exact println-based
own-child observer is retained separately; no printing support is claimed.

The execution gate also reruns the six affected consumer groups and the existing
verifier finality/authority group, after both strict modes and architecture pass.
The old verifier helper now expects genuine successful declaration completion;
its Min, effective-finality, invalid-overlap and post-seal insertion opponents
remain unchanged. Candidate native results and this readiness change are pending.
