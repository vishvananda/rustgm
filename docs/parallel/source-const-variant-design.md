# Selected const cleanup: proposed initialization interfaces

Proposal after 12eef81a8. No production enum changes are made by this document.
The existing initialization fixed point remains the producer. The actual cleanup
plan remains the owner of selected obligations. Eligibility is distinct from
execution and from explicit whole-type const Destruct bounds.

## Coordinates and ownership

Proposed private records under ownership/variant_liveness (names illustrative):

```rust
struct Coordinate(u32);
struct TagRange { start: u32, end: u32 }
enum Step {
    Root,
    Tuple(u32),
    Nominal { variant: u32, field: u32 },
    Element,
}
struct Part {
    parent: Option<Coordinate>,
    step: Step,
    children: Range,
    variants: Range,
}
struct Variant {
    ordinal: u32,
    fields: Range,
}
struct Tracked {
    coordinate: Coordinate,
    initialized: Anchor,
    tags: TagRange,
}
enum Anchor { Path(PathId), Shallow(u32) }
```

These are private analysis coordinates under a completed actual aggregate owner,
not resolver definitions, FieldIds or source projections. A nominal step's actual
owner and bounds are authenticated against Definitions and the existing
MemberPosition adapter; existing mentioned FieldIds must map to that exact step.
Root semantic TypeId comes from the actual body place. A selected obligation
carries that root and its coordinate chain, never a field type copied from the
physical shape's region-only fast path.

One function-level optional partition is owned by the existing Refinements,
retained in initialization::Facts. A structurally shared descriptor DAG is keyed
by the existing canonical physical type within this one analysis. It is a shape
index, not source or proof authority. Instantiated occurrences provide distinct
mutable tag ranges. Recursion follows owned fields only; references/pointers end
it, and arrays reuse existing sparse segments rather than expand their length.
No field source syntax is parsed again, and no source member identity is invented.

Unmentioned payloads get these bounded analysis coordinates, not fabricated Body
projections or read permissions. Mentioned paths reuse the existing PathId;
unmentioned fields inherit the existing parent's shallow initialization anchor.
Descriptor construction checks aggregate kind/policy and authenticated variant and
field counts before appending. It prepays descriptor/occurrence/index storage,
tag bits and their full per-block incoming table against the existing 64 MB and
32-million-work limits before replacing the incoming table. Shape DAG reuse avoids
walking the same declaration for every local. No enums means no partition or tag
allocation. Existing physical Paths and their read/move proof are unchanged.

## One fixed point and replay

Tag possibilities use extra bits after the existing physical path bits in the
same initialization Word<u64>::initialized plane. They do not enter physical path
intervals, must-read bits or mutable-binding written bits. A dedicated possible
bit query ignores the must plane. The current Word join already unions this
may-plane, so branch joins retain all possibly active variants without a second
solver or a set cross-product.

The partition indexes each real statement once into:

```rust
struct Transfer { copies: Range, fills: Range }
struct Copy { source: TagRange, destination: TagRange }
struct Fill { destination: TagRange, value: bool }
```

Actual typed constructor/argument lists authenticate the mapping before it is
recorded. Evaluate(Read) copies the selected source place's tags into its actual
owned temporary. Aggregate constructors set their own selected variant and copy
nested argument tags into corresponding fields. Store copies the already-evaluated
value tags to its real destination. Unknown parameter/call results start with all
variants possible. Storage/ownership end clears the corresponding tags.

The transfer_block and replay joins use the SAME indexed transfer. A bounded
scratch range captures copy sources before Flow::statement consumes operands or
replacement cleanup runs. Publication occurs only after the existing cleanup and
BodyFacts::transfer; it cannot change old-value cleanup tags early. This is needed
for aggregate operands and moving reads. The scratch is reused per statement and
is sized/prepaid from the largest recorded transfer. No CFG/source scan occurs
per local or cleanup.

Ownership clearing must also clear contained tag ranges. A preorder index maps
each physical subtree to its contained tag span; range operations avoid scanning
all locals. A moved source on one incoming branch must not leave a stale Bad tag
that pollutes a Good-only still-owned branch. Constructor/store mappings override
the normal unknown fill only after their checked transfer completes. Existing
IsVariant branch edges intersect the corresponding may-tag range and retain the
current payload exclusion law. Cached booleans remain excluded from that authority.

Calls never infer a returned tag from a callee body. Mutable/raw exposure cannot
leave cached constructor certainty usable after possible alias writes. The affected owned subtree becomes unknown when its mutable/raw
address/reference escapes, with a sticky exposure bit until storage ends;
future whole stores must not restore certainty while an old alias may survive.
This uses an indexed affected subtree, not all-local invalidation at each call.
Unresolved alias profiles stay unknown. The independently frozen unused
mutable and raw borrow cases actually reject E0493 too: their original positive
predictions remain retained with corrections. Shared borrows of noninterior data
can retain tags only under actual existing mutation policy. Full alias precision must reuse the existing place/loan origins rather
than install another points-to authority.

## Selected obligations and proof

At each existing symbolic/native cleanup selection, traverse its initialized
coordinate tree once, using the current Point plus possible-tag bits. Retain an
indexed range of obligations on that actual cleanup plan:

```rust
enum Obligation {
    Whole { root: TypeId, path: Coordinate },
    OwnDrop { root: TypeId, path: Coordinate },
}
```

Whole at a selected field means const Destruct of THAT semantic field. OwnDrop
means the actual nominal Drop implementation only; it is not a whole nominal
Destruct claim. No selected enum root is granted a whole-type claim merely because
one variant's leaves are eligible. Empty selected variants can have no field
obligation; actual own Drop still runs first. Never/Suppressed/union policies and
zero-length explicit whole-type bounds retain their existing distinctions.

Pipeline resolves paths through actual declaration templates and Bound::joint,
checks real owner/kind/ordinal/count at every nominal step, and selects the normal
const Drop/Destruct goal in the exact defining environment. Existing Claims needs
a typed subject distinction (Destruct vs OwnDrop); an own Drop proof must never
satisfy a whole Destruct requirement on the same TypeId. Loan legality consumes
these SAME plan obligations and exact selected claims, not cleanup.live_type as a
whole-root substitute. Existing whole live_type remains for ordinary drop-check
regions and partial-root restrictions.

Partial Struct roots retain their frozen whole-type eligibility restriction.
Partially moved enum roots likewise must preserve the Rust E0493 distinction;
tag possibility is not permission to hide a moved-field static cleanup obligation.
The implementation must distinguish inactive payload exclusion from physical
partial moves using the unchanged read/move initialization plane. Conditional
uninitialized Good-only roots are independently positive and cannot be mistaken
for partially moved roots. These distinctions require point queries on actual
initialization, not a tag-derived permission.

Prepared/finish seals and current arena/owner/environment/Copy provenance continue
to govern both stages. Storage and proof growth are paid before publication.
Code activity still uses actual cleanup access. The interpreter fence remains
until real scheduling and destructor execution exist; selected proof alone never
creates an executable operation.

## Independent controls

reference-v1 froze tagmv/tagret/tagout/tagbr/tagfld/tagnst/taggen/taglif positives
and tagset/tagbad/tagarg/tagpar E0493 negatives. The same batch froze runtime59,
valid reached CTFE cleanup and null-write E0080, plus invalid original E0493.
reference-v2 adds own const/nonconst Drop, shared/mutable/raw borrows,
conditional uninitialized cleanup and replacement ordering. It corrected three
predictions: const Drop on an enum containing N is E0367 (N lacks const Destruct),
and even unused mutable/raw borrows of Good(C) lose precision and reject E0493.
The original predictions/source remain; tagow2 is the separate genuine const-own
Drop positive with all fields const eligible. Original cldef,
clcall, partial Struct and all previous predictions remain unchanged.
