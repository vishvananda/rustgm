# Concrete private graph, mode and marker proposal (source only)

Successor branch pa31-specialization-graph-v1, still at qualified64c9f507.
No production graph/schema edits. New observations40237725892 supersede ONLY
incorrect predictions, not source bytes. All numeric new type sizes below are
UNMEASURED; budgets explicitly use size_of/capacity instead of guessed sizes.

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

## First verifier component (unqualified source checkpoint)

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
