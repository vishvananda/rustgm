# Opaque byte storage

This development slice consumes an already published full-TypeId opaque storage
head. It does not reveal that head to source type equality, trait selection,
lifetime proofs, move/drop checking or callable selection. Missing defining
heads remain a required readiness gap; equal layouts never establish a mapping.

`Reader::storage_type(TypeId)` returns ordinary types unchanged. For an opaque
application it reads `Store::storage_head`, requires a non-opaque canonical head
and both completed ABI layouts, and checks every copied layout property and the
exact retained shape identity, field ranges, tag or niche identity. It performs
no layout construction, inference, ancestry walk or global-store scan.

The existing iterative byte-family DAG, encoder (both frozen values and expanded
cells) and decoder select this physical type after preparing the original
footprint. Their caches retain complete original TypeIds; an opaque application depends
on its checked head in the existing DAG, so shared physical children are
validated once. Scalar validity, active canonical enum
variants, closure environment identity, initialization masks and symbolic word
permissions use the exact head's existing consumers. Decode returns the existing
Constant representation under the caller's original logical demand. The root's
Backing.ty, location, logical bounds, raw alignment anchor, source contract,
move/drop authority and pending-operation identity remain unchanged. No public
Value, View, Origin, Pointer, Frame or ABI Shape change is part of this slice.

Readonly product inspection follows the checked physical head. General
shared-mutation/value inspection retains its existing conservative opaque
boundary; no public Copy/Sync/SharedMutation type query changes. Pointer/reference
read duties still execute freshly on each selected value. Structural union
field inspection remains forbidden. Existing images keep their physical image
owner and explicit unsupported metadata export boundary; no head is guessed from
an Image or bytes. The readonly Reader.field_types inspection follows the same checked head for
constant graph products; ordinary source field selection is unchanged. Field
projection, data emission and instance dependencies retain their existing typed
entry points.

The initial independent inventory lives in
`obj/parallel/pa27/opaque-string-backing`. Original source bytes and all declared
hypotheses remain frozen. Rust 1.98 accepts eight initial positives and rejects
two negative controls; separately declared division-helper versions remove an
unrelated unresolved panic macro from candidate checking. Opaque scalar, record,
full generic applications, union snapshots and closures currently stop at the
unsupported byte family. String reference/raw-word roundtrips stop in symbolic
word encoding and remain separate required work, including UTF-8/reference
validation. Reference and non-Copy ownership controls expose a distinct missing
prepared opaque head; their acceptance is not claimed by this codec change.
The first non-Copy local-return hypothesis rejects in Rust with E0493; a later
ManuallyDrop/known-repr(C) ownership transfer is independently Rust/native valid.

Private controls cover exact and missing heads, incompatible copied layouts,
separate closed instances, invalid scalar payloads, restricted/expired pointer
values, non-Copy stored payload representation, zero-byte capture identity and
bounded work. Width/repetition/shared-head and unrelated-store axes must show
work proportional to touched storage and retained recipes, with warm repeats
not scanning the global Store. Source/native fixtures remain PA27-owned. Root
owns combined release, text-oracle review and comparative performance.

The prepared-head source checkpoint has five initial supported positives and
four semantic negatives; three independently qualified wide-array source/value
controls use widths/repetitions 8/32/128. Pointer-add and ptr::read fixtures live
under PA28. Three Rust-positive reference/owned cases and one restriction
negative still stop before the codec at missing prepared heads; they are
REQUIRED_GAP records with the correct desired outcomes, not rejection contracts.
All original failed hypotheses remain under the evidence root.

Private shared-head validation observed `5 * width + 11 * applications + 6` logical
steps on the declared 8/32/128 grid, then one step per warm application. Selected
byte reads used four steps per access regardless of width or unrelated-type
noise. The memory reference invariant changes actual initialization after a warm
word decode, restores it, then expires the referent; each access must revalidate.
This is physical validity evidence, not a source lifetime proof.

Closing qualification passed nine private invariants, twelve supported source
cases across all three views and eight literal native value comparisons. The
fresh reference test checked four duties and paid two; both failed accesses were
rejected before publication. All 8,362 PA1–24 text comparisons remained exact;
strict Clippy, architecture audit, harness/layout and pinned upstream checks pass.
The exact receipts and unchanged binary/source hashes accompany the handoff.
