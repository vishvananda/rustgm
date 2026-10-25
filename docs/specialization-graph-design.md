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
