# RustGM Programming Assignment 27: Unsafe language

Status: **planned**. This is a buildout brief, not a released grading contract.
Prerequisite: PA26. The common source profile is Rust 1.98.0, initially edition
2024 and Linux x86-64. Later edition support is explicit at PA36.

## Required implementation

Unsafe operations, raw pointers, unions, packed access and validity.

Track unsafe permission lexically in the typed body checker. An `unsafe` block
permits its descendant expressions and closures; a nested item starts its own
context. An unsafe function requires unsafe permission at every call site,
including calls through a function pointer and selected methods. Its body follows
the pinned `unsafe_op_in_unsafe_fn` policy. Unsafe permission does not bypass
ordinary typing, initialization, ownership, packed-reference alignment or constant
evaluation rules.

Interpret retained `allow`, `warn`, `deny` and `forbid` attributes for
`unsafe_op_in_unsafe_fn` and `static_mut_refs`. Their lexical inheritance crosses
closure and nested item boundaries independently of unsafe permission. An
explicit unsafe block satisfies a denied unsafe-operation lint. Descendant
`allow` or `warn` cannot weaken `forbid`; `deny` retains its prohibition. Keep
the ordinary no-policy path free of allocated per-node policy tables.

Keep raw pointer dereferences distinct from reference dereferences in the typed
place representation. Raw accesses have no reference lifetime or inherited loan
promise. Evaluate and check the pointer storage before selecting its referent;
reject writes through `*const T` and moves of non-`Copy` referents. Forming a raw
address does not itself create a reference or its alignment promise. The direct
`&raw const *pointer` cancellation does not read its referent, while accessing a
field through the pointer still requires unsafe permission.

Mutable static storage requires unsafe permission for reads and writes. Keep its
declaration mutability separate from the permission to form a reference: the
edition-2024 `static_mut_refs` lint denies explicit and implicit references by
default. A direct raw address does not create such a reference. Retain the types
of static places actually used by each body so assignment cleanup can run an old
value's destructor before replacing it. Constant evaluation has its own mutable
static access restrictions, including inside an unsafe block.

Union fields share storage and therefore overlap during borrow checking. Exactly
one field initializes a union; reading or borrowing a union field requires unsafe
permission, while assigning a field does not. Enforce the pinned allowed-field
contract and represent union layout directly, without a discriminant or a fake
struct sequence. Validate demanded constant results using symbolic allocations,
initialization, alignment and scalar validity; never obtain host addresses or
execute input-language unsafe operations in the compiler implementation.

The current runtime union checkpoint places every field at byte offset zero and
initializes only the selected field's storage. Different fields overlap for
loans; subfields within the same selected field retain ordinary disjointness.
Moving a non-`Copy` field consumes the containing union. Field eligibility uses
the canonical `ManuallyDrop` identity, and implicit field cleanup is suppressed
while an explicit union destructor still runs.

Constant unions whose concrete fields are all primitive scalars use immutable
byte images with an independent initialized-byte mask. They have no active
member. A selected field read checks its byte range, initialization and scalar
validity, including bool and char representations. A field store replaces only
its bytes and preserves the remaining initialization facts. Copies retain an
immutable snapshot. Closed generic scalar families use the same rules.

The original demanded and unused invalid-boolean and uninitialized-byte inputs
remain unchanged and now reject for their respective validity failures. An
unread alternative field need not contain a valid representation. Native output
may choose zero for unspecified padding without marking those bytes initialized
for constant evaluation. The constants view prints explicit bytes and mask bits.

Pointer, reference, function and aggregate field families remain explicit
constant-execution boundaries. Scalar union-field raw addresses retain canonical
field ordinals over the same parent Image. Different field pointers alias its
current bytes, with initialization and scalar validity checked on every read.
Field stores publish a new parent Image while copied union values retain their
prior snapshots. Static field-pointer relocations retain byte offset zero. This first
image profile contains no pointer bits or relocations. Images are limited to
64 KiB each and a bounded 64 MiB arena; operations also consume the existing
execution work budget. These controls do not activate the assignment or create
generated text references. General provenance reinterpretation and the remaining
packed/pattern validity controls still require qualification before release.

Final constant storage follows logical aggregate reachability. A zero-length
repeat stores no fill value, so a raw pointer computed for its operand does not
escape through that array. The operand still executes and retains every const
eligibility and validity check. Nonzero repeats retain their dangling-pointer
checks, and discarded transient raw pointers do not become stored results.

Typed raw constant reads use symbolic allocation origins and existing initialized
cells. They check storage lifetime, requested type, bounds and physical alignment
from retained layout facts. Direct raw-address cancellation does not read its
referent. Exact-type raw stores to live local storage also check per-pointer write
permission before changing a cell. Shared borrows and promotions restrict that
pointer value; copies, calls, casts and field projections retain the restriction,
while raw constness alone does not introduce sharing. A separate shared reborrow
does not poison an earlier writable raw pointer. Writes retain the actual raw
access alignment anchor, so packed-base field assignment differs from assignment
through a separately formed unaligned field pointer. Static stores remain
forbidden independently of pointer mutability. No old value is read merely to
store a replacement; source initialization rules remain separate.

Exact-type shared references reconstructed through raw pointers validate live,
fully initialized ordinary symbolic storage and the actual leaf alignment.
They monotonically restrict that pointer's write authority without changing an
earlier writable alias. Packed raw-base by-value reads and source packed-reference
rules remain separate. Existing final reference escape checks still apply.
Physical direct-reference cells may differ only in their outer lifetime identity
when raw storage is checked: outer mutability and the exact referent type must
still match. This does not relax source lifetime proofs or recursively inspect
unused inner references. Different inner lifetime structure remains unsupported;
actual dereferences retain lifetime, initialization and per-pointer checks.
Mutable references through raw pointers additionally require existing writable
pointer authority and ordinary nonstatic storage. Success retains the identical
pointer; a cast to `*mut` cannot upgrade a shared-derived value. Copies, products,
function calls and a separate shared reborrow preserve an original writable alias.
This initial profile checks initialized storage even for unused references. Pinned
Rust accepts some unused null/expired/unaligned mutable borrows and read-only uses
of shared-derived mutable borrows; those forms remain explicit unsupported cases,
not qualified language rejections. Numeric nonnull ZST pointers and raw-derived
references to mutable/foreign or cyclic static storage also remain boundaries;
forming a reference is not itself a contents read.

Raw-derived references into scalar union Images retain the canonical typed
field projection. Reads and stores through them use the same current Image bytes
as other field pointers, with no active-member tracking or product-field cells.
Selected reads still check initialized masks and bool/char validity. Mutable
reconstruction retains its writable, initialized nonstatic profile; write-only
repair of uninitialized or invalid selected bytes remains unsupported. Direct
source `&u.field`/`&mut u.field` uses the same canonical projection and checked
reference rules. The typed UnionField capability does not synthesize a raw access
or grant unsafe permission; an existing raw-base anchor remains intact. Source
packed alignment and initialization/ownership checks stay independent. Pointer-
bearing and zero-sized/aggregate union families remain explicit boundaries.

Typed byte-addressed raw reads, stores and logical field/index offsets reuse the
original scalar/aggregate backing and symbolic fragments. Actual selected bounds,
initialization, raw alignment anchors, lifetime and incoming write permission
remain independent checks. A reconstructed reference also validates its complete
selected type and leaf alignment; an already normalized view retains its exact
pointer identity. These controls are reviewed in `docs/pa27-byte-access-review.md`.
Canonical const/mut `.add` now preserves checked offset identity and access restrictions.
Initialized whole `[u8; N]` snapshots retain symbolic fragments through copies and
const calls, without manufacturing numeric pointer bytes. Final byte-array
publication rejects all provenance; typed scalar and reference demands still
validate selected bytes. Owned U8 array-rest/range values preserve selected
fragments and original word coordinates. Range stores commit one selected
backing update. General raw/unsized ranges, numeric pointer reconstruction and
unsupported backing families remain required work. Shared-
derived interior-mutable writes remain unsupported. Unsafe permission grants none
of these missing constant capabilities. Foreign and mutable static contents remain
forbidden in this development profile.

Raw slice indexing selects an exact sized element from original array/rest
storage. Logical metadata bounds and the selected physical backing bounds remain
separate; oversized raw metadata does not grant access beyond that backing.
Reads, stores and reconstructed references retain their ordinary liveness,
initialization, alignment, static and per-pointer authority checks. Raw length
observes metadata only. Exact wildcard patterns do not select unused elements;
bindings, literals and other refutable children retain their accesses. These
controls do not implement whole unsized raw values or general range reconstruction.

Packed by-value copies, primitive compound assignment and raw address formation
remain distinct from creating a field reference. The qualified let-reference
and match-reference controls reject in the executable pipeline. Earlier type
inspection retains their alignment obligations under PA9's phase boundary.

Closure capture precision stops before a packed field access, at the union,
or before a raw-pointer dereference. Full typed leaf paths remain available for
ordinary access checking. Raw pointee writes capture the pointer prefix as a
read and can use an immutable `Fn` closure; separate pointer-slot writes retain
their mutable capture and `FnMut` requirement. Move captures preserve that
separation. Nested formation checks the same raw/reference and union/ordinary
field capabilities, including exact nominal field ownership. Capture truncation
grants no unsafe, alignment or constant-evaluation permission.

The isolated draft capture receipt preserves its original reference-output
boundaries. A separate integration receipt qualifies the unchanged packed and
union reference-output controls after the closure lifetime proof: both succeed,
while an unaligned returned reference rejects for packed alignment. Original
hypothesis disagreements for union Copy reads and direct raw field addresses
remain explicit. These checkpoints do not complete the assignment.

The maintainer implements this responsibility in `dev/src/` and introduces its
typed interface before generating text views. Consult `spec.md`, the pinned
Rust Reference, and the existing CPPGM implementation for efficient algorithms.
Do not introduce rendered text as transport between compiler phases.

## Tool surface

Extend `dev/rustgm --emit-lowir -o OUTPUT SOURCE`. Earlier stage views remain
available. Freeze the exact text vocabulary, deterministic ordering, error
contract and example outputs before declaring this assignment active.

## Tests to develop with the reference

required unsafe contexts; alignment; raw borrows; union reads.

Add spec fixtures, positive/negative controls and cross-feature regressions in
the earliest owning assignment. Use the course reference for text oracles and
pinned rustc for independent language checks. From PA11, execute emitted LowIR
with the configured CPPGM backend. Do not require new future-stage behavior
implicitly. An empty suite or a not-implemented result cannot pass.

## Completion and review

Publish the semantic and lowering boundary, tests/manifest.json and reference
sidecars together. Run all active earlier suites, lint, file/architecture audits,
and affected scaling measurements. Record independent checks before setting
status active in `assignments.json` and adding the stage to the root report.
Student export is a later operation: this repository owns the full solution.


Explicit `repr(u8)` and `repr(i8)` fieldless enums can use constant byte backing.
Sparse and signed tag bytes select exact declaration variants; invalid tags and
uninitialized typed reads reject. Raw copies preserve those bytes without
interpreting them, so a destination may be repaired before a typed read. Ordinary
enum values retain their existing product/variant identity, and padding remains
independently uninitialized. The draft `tests/enum-byte-outcomes.json` and
`docs/pa27-enum-bytes.md` record this bounded profile; payload, default-layout and
wider-tag enum storage remain required work.
