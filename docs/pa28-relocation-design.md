# Symbolic byte relocation checkpoint

Status: qualified development profile, not assignment activation or complete
memory-intrinsic coverage. Symbolic fragments, thin/function words and independent
slice/vtable metadata now execute through the byte backing. Frozen independent
inputs and all retained failures are under
`obj/parallel/pa28/relocation-transport/`.

## Independently established contract

The corrected 21-case object inventory and 13 Rust native controls pass with
Rust 1.98.0, edition 2024, x86-64. Four additional vtable-word cases and two
native controls pass. Exact sources and hashes precede execution in
`declared.json`, `declared-v2.json` and `declared-vtable.json`.

- Split, reverse-order and two-stage copies reconstruct the same pointer.
  Equal pointers encoded in distinct storage cells also reconstruct. Different
  allocations or different offsets within one allocation do not mix.
- Data pointer provenance and slice length are separate words. Length can come
  from another slice or a numeric metadata store. The resulting length controls
  indexing, independently of the original backing allocation's larger extent.
- Vtable words are symbolic. Words and halves from the same canonical vtable
  reconstruct across different data pointers; different vtable identities do
  not mix. Reading a vtable as an integer rejects.
- Function pointers and full thin/slice/object pointer metadata survive copying.
  Shared-derived write restrictions survive reconstruction; an original writable
  pointer remains writable after a separate shared reborrow. Expiry still rejects.
- Numeric reads of pointer bytes and demanded partially overwritten pointers
  reject. The original hypothesis that an unused final union could retain a
  partial pointer was wrong: Rust rejects partial pointers in final constants.
  That unchanged rejecting source and its initial expectation remain recorded.

Transient fragment transport must therefore differ from final publication.
A fresh token per encoding is incorrect, and a token for an entire fat Constant
incorrectly ties independently transported data and metadata together.

## Private storage and readonly image surface

Keep Value, View, Origin, Pointer, Constant and ordinary projection records
unchanged. Extend only the lazy Image record and ImageView, plus private byte
Backing/Snapshot and an optional per-machine word arena.

```rust
enum WordTarget {
    Value(Constant),
    Vtable { concrete: TypeId, target: TypeId },
}
struct ImageFragment {
    offset: u64,
    source_offset: u8,
    length: u8,
    target: WordTarget,
}
```

`Value` admits only an existing data-pointer Constant with metadata removed
without changing its location or write restriction, or an existing canonical
function Constant. Numeric addresses are not symbolic provenance. Vtables come
only from existing checked object metadata. These are borrowed read views;
publication validates invariants and callers cannot install arbitrary facts.
ImageView adds `fragments: &[ImageFragment]`. View::Image remains unchanged.

The private word arena interns complete word identities, not encoding events.
Data identity uses the original allocation root, physical byte offset and
WritePermission, independently of which equivalent projection was encoded first.
Reconstruction selects an actual declared subobject, rejects ambiguous coincident
views, and rechecks live access authority. Function identity retains the complete
canonical Instance. Vtable identity uses the same erased concrete/object storage
shape as existing vtable preparation; the original checked target remains only
for emission. Erased identities never prove a lifetime or become semantic types.
Private fragments use `(offset: u64, source_offset: u8, length: u8, word: u32)`.
Adjacent intervals coalesce only for the same word and consecutive source bytes.

An optional boxed BTreeMap indexes fragments per converted root. Predecessor and
range queries touch only intersecting fragments, avoiding a full-root provenance
scan for each small read/write. Snapshot intervals are sorted and bounded by the
selected span. Copies clip/translate source intervals once; numeric writes split
or remove only intersecting fragments. Existing bytes/mask remain independent:
symbolic pointer bytes are initialized but never exposed as placeholder integers.

Measured Rust prototype sizes on this toolchain are Constant 32 (unchanged),
WordTarget 32, private fragment 16 and public image fragment 48 bytes. Image grows
40 to 56 bytes, converted Backing 56 to 64 and Snapshot 48 to 56 using lazy boxed
fragment containers. Ordinary untouched roots allocate none of this storage.
The layout source/output and compiler command are retained beside the controls.

Preflight must charge word arena/index capacity, conservative tree-node capacity,
fragment splits and snapshots before allocation, alongside existing bytes/masks.
The conservative reservation is 512 bytes per retained fragment and 1024
per interned word plus headers/temporary capacity, rather than pretending only
logical payload bytes are retained. Work charges interval lookups, intersected
fragments, copied bytes and publication normalization. Scalar-only empty-fragment
paths stay allocation-free; ordinary roots retain their typed path.

## Producer, access and publication APIs

Add a typed word codec facade beside the existing values byte codec. Encoding
consumes a checked type/value and existing pointer parts; rebuilding consumes a
complete data word plus independently decoded length or vtable metadata. Rebuild
can retain or restrict original write authority, never upgrade it. Shared
references restrict authority and reuse existing checked-reference validation;
mutable references require existing writable authority. Raw pointer decoding
creates no reference. Existing bounds/alignment/liveness/static rules remain.
Wide reference and general numeric/reinterpretation profiles require separately
qualified access support; preserving wide raw pointer metadata grants none.

Expand canonical union family storage from primitive tags to closed retained field
types, including byte arrays and pointer/function words. Rename the private
ProjectionStorage::UnionScalar classification to UnionField, preserving every
owner/ordinal/type check and offset-zero rule. Structural Reader.field remains
forbidden for unions. Primitive AST evaluation keeps its current fast path;
nonprimitive union constructors use the existing checked-body execution request,
with no sema-to-interpreter dependency or source-context reconstruction.

A typed numeric scalar read rejects intersecting symbolic fragments. A pointer
word read requires all initialized bytes, complete ordered coverage and one
canonical word identity. Fat pointers combine that data word with their current
scalar length or canonical vtable word. No guessed addresses or selected union
active members exist.

Transient memory may contain fragments. Named/associated/inline final results
and immutable constant publication must reject incomplete or misordered pointer
words, even in an unread union. Validate symbolic target liveness using existing
origin rules; do not impose validity of every alternative union field. Const-fn
intermediate returns retain the ordinary caller/owner boundary rather than
inventing an extra final-constant boundary.

## Required consumer audit and evidence

- Image preparation/read/write/snapshot and Cell conversion share word codecs.
  Every typed/projected/range consumer continues using the sole root backing.
- Result and source escape traversal visits image word targets and checks final
  completeness. SharedMutation remains separate from symbolic storage validity.
- Constant view emits deterministic symbolic interval/target rows; pointer bytes
  cannot appear as literal zero bits. Existing scalar-only Image output stays
  byte-identical. Graph traversal includes allocation aliases, functions and
  vtables without synthesizing a falsely typed standalone pointer value.
- Instance dependencies visit function/vtable words. Lower/data emits numeric
  runs plus complete existing eight-byte relocations with canonical addends,
  reusing constant_pointer/object_table/function demand and compiled-entry proof.
  Partial final words diagnose before emission. Padding zeroing never changes
  initialization or provenance facts.
- PA29's current scalar-only metadata profile stays an explicit boundary; this
  change does not silently serialize aggregate Images across crates.

Complete thin, slice, function and object Images have independent source/native
controls. Scalar-only Image text is unchanged. Symbolic byte positions render as
`@@`; sorted fragments render as
`offset:source_offset:length:pointer:aN:write=shared-derived` (or `writable`),
`function:TYPE`, or `vtable:CONCRETE:TARGET`. Native emission uses existing checked
relocations and the unchanged pinned backend, not placeholder pointer bits.

## Qualification and bounded continuation

The final inventory is 55 independently object-qualified source inputs, each
checked in types, LowIR and constants views (165 matches), plus 28 literal-source
Rust/CPPGM native pairs. Literal native sources retain their exact frozen bytes,
including leading crate attributes; rewritten native inputs and their originals
have separate hashes and receipts. Final function/object Images specifically
exercise relocation emission and callable/vtable dependencies. The unchanged
pointer-neighbor source now accepts and has a paired native value control.

Thirteen private byte invariants and two bounded shape-fold invariants pass.
Widths 8/32/128 with repeats 8/32/128 keep selected eight-byte interval work
logarithmic in retained fragment count and linear in repeats. At 128 repeats,
steps are 36,480 / 44,672 / 52,864; backing retention is unchanged by repetitions.
The pending-reference controls decode 9/33/129 nodes for 8/32/128 reference fields,
with 16/64/256 validation attempts and 8/32/128 paid duties. They retain a decoded
result and next unpaid duty across exact instruction/frame retries, pin mutable
backing while parked, clear on success/error, and decode fresh after a write.
No typed result or destination store publishes before readiness.

The bounded shape fold shares the ordinary traversal, with no ordinary Bound
layout change. It budgets retained cache/pending state and rebuild scratch before
growth; canonical Store allocation remains a separate account. Tiny budgets
reject, warm hits need no extra capacity, and 4096 unrelated types do not affect
shared-root work. Widths 8/32/128 take 35/131/515 visits. The prerequisite is the
separate internal handoff `6fdadb3d`; fixed-map Restore ownership is PA25 work.

Actual layouts are Constant 32, Cell 48, Backing 64, Snapshot 56, WordTarget 32,
ImageFragment 48, ImageView 56 and private Fragment 16 bytes. Unconverted roots
retain the ordinary typed path; symbolic maps/word/fold state are lazy. No new
64 KiB cap is imposed on ordinary byte backing. Existing Image publication limits
remain explicit.

Closing evidence is in `closing-seventh/`: strict all-target/all-feature Clippy,
1607-file audit with zero findings, layout/upstream/group registration, all focused
controls, and all 8283 active PA1–24 text comparisons pass. Source fingerprint
is `d303e15d8b2d7167b13fa280e92a5ac71176afc9fbaf5a270ed3bfdf2a68b7d6`;
candidate SHA-256 is
`56d7deb5280227f35e5e1b933d06760166b1a2f5ec88d397eb24b99034bd767a`.
The backend hashes remain those in `backend.lock.json`. These are charged work
and functional checks, not a new throughput or RSS measurement.

The first unused-partial-pointer hypothesis and the two implicit-static
borrowed-object hypotheses were disproved by pinned Rust and remain in the
original receipts. Corrected scoped borrowed objects accept in both encoding
orders; a reconstructed reference to their expired local cell rejects. Physical
field selection ignores only an outer Dynamic region while preserving pointer
mutability, exact shape/arguments/binder and ordinary source lifetime authority.

## Explicit remaining boundaries

The companion `pa28/tests/relocation-outcomes.json` distinguishes invalid source
outcomes from profile boundaries. Three raw slice-index positives still reach
`constant raw whole-range access is not implemented`; an ordinary counterfactual
has the existing raw reinterpretation boundary. Original object reference-cast
inputs stop at the scalar-cast boundary; typed-let equivalents qualify this
transport family without claiming that PA23 route. A dynamic-reference escape
also stops at existing raw reinterpretation, so it is not counted as lifetime
validity evidence; the sized reference-cell escape is the opposing semantic
control. The interior-reference temporary rejects for expired backing before the
interior-reference gate and is recorded accordingly.

General numeric/null/one-past or ambiguous subobject reconstruction, strings,
function-as-data reinterpretation, arbitrary enum/opaque/closure byte backing,
more general reference reconstruction and byte fills remain required work.
PA29 Image/relocation export remains unsupported. These boundaries are not
assignment exclusions or a PA27/28 completion claim.


The final narrow regression tail is in `closing-tail/` and
`closing-tail-second/`. Five raw-access private tests, raw-read/raw-store groups,
union-reference/address/direct-borrow groups and scalar Image groups pass,
including ignored native controls and their bounded-work tests. The first tail
retains the stale pointer-family assertion failure. Six unchanged earlier
pointer/array/unit-family sources were then independently requalified (all
accepted); only their former unsupported test expectations changed. Exact
initialized Image payloads now replace the old scalar-family rejection assertion.
A separate mixed-family literal native program also matches pinned Rust/CPPGM.
Historical `scalar-union-outcomes.json` and old contract observations are not
rewritten; the companion relocation contract lists these supersessions.

Final strict Clippy, audit (1607/0), layout and upstream checks pass after these
test-only changes. The all-dev-Rust fingerprint is
`a48ccdcb77b276680a4738f6b3826cefc138f82676d0e9d14fdc9519bd178037`.
Every production source hash and the candidate binary are verified unchanged
from the 8283-case text run; only tests/declared contracts changed afterward.
No new full Cargo matrix, broad native matrix, timing run, reference generation,
assignment activation, backend change or public push was performed.

The next recommended bounded feature is canonical `ptr::write_bytes`: it can
reuse one proved destination span, fill bytes/mask and remove intersecting
symbolic fragments in one batch. Its exact unsafe/const/Sized identity, zero-byte
contract and runtime lowering need the normal separate proposal and independent
controls. Raw slice indexing, object reference-cast/reconstruction, remaining
provenance profiles, aggregate volatile operations and additional atomic/numeric
families remain required work; this recommendation does not close them.
