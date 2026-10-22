# Typed byte-addressed raw access proposal

This approved preimplementation proposal starts at accepted `213f1e21`. The
qualified implementation is recorded in `pa27-byte-access-review.md`. It is not an assignment completion or
release claim. PA27/28 remain planned; the current explicit reinterpretation
guard is retained until the producer and every consumer are qualified together.

## Independent declaration

`obj/parallel/pa27/raw-byte-access/declared-{first,second,third}.json` preserves
50 original hypotheses and exact input hashes. The pinned profile is Rust
1.98.0, edition 2024, Linux x86-64 little-endian, with the explicitly declared
`--cap-lints=allow` independent policy. `first/`, `second/` and `third/` contain
object outcomes, valid native executions and three baseline frontend views.
Baseline source is `dc2bbb6fa22b0ad716832d4196784a45e30e39de11bbca52d7109f00f49c6a4f`;
immutable binary is `acba916b15d454c69e9ae07416c438ecd0522830539c73ec0d577ddc7890ecac`.
The exact binary hash in `first/baseline.json` is authoritative.

Accepted controls include primitive and array casts, record/nested offsets,
subobject-relative casts, unaligned transfers, shared/mutable reconstructed
references, float/bool/char interpretation, pointer words, zero-sized reads,
published raw pointers/references and offset-word transport. Opposing controls
cover bounds, alignment, invalid bool/char, padding initialization, expiry,
shared-derived writes, numeric provenance and pointer-word numeric extraction.
A fitting selected prefix of a larger logical record/array is independently
accepted. A one-past field address accepts; a beyond-one-past address rejects.
Packed-base by-value access and a separately formed unaligned field pointer
retain opposing outcomes. No negative/precondition-violating native claim is
made.

The `u32::from` aggregate-reference hypothesis was rejected because stable
`From` is not const. Its original bytes/diagnostics remain; the explicitly named
`as u32` control isolates the accepted raw/reference behavior. The original
`.add(2)` positive remains a separate canonical method-preparation gap; logical
field/index offsets do not claim that API route.

## Retained address representation

Add internal `ProjectionId::Byte(u32)` and reuse the existing 16-byte projection
record as `{base, byte_offset}`. Add readonly `Origin::ByteOffset {base,offset}`.
Pointer, Constant and Place must retain their measured sizes; the new case fits
the existing projection tag and Origin's larger Subsequence payload. Actual
layout invariants will verify this rather than treating it as an ABI promise.

A crate-private evaluator operation takes an existing complete pointer value,
a checked relative displacement and a selected logical TypeId. Only a machine
consumer with retained ancestry/layout can produce the projection; there is no
public numeric/root-plus-offset pointer constructor. The incoming pointer's
write restriction survives. Metadata is cleared only for a selected sized
address. Nested selections normalize to one original allocation and absolute
byte offset, avoiding repeated full-ancestry traversal.

Keep ordinary physical Projection and Subsequence identities distinct. A byte
view's selected type is logical; it must not be used as the allocation's stored
type or interpreted as an ordinary canonical field. Existing DST physical/logical
field rules remain unchanged on the ordinary path.

## Execution access

Use a private typed StoragePath terminal with Range or ByteSelection
`{offset,ty}`, preserving the existing two-word terminal payload envelope where
the host layout permits. No byte offset may be silently treated as an index.
Fields-only consumers reject unhandled byte terminals until explicitly adapted.
Ordinary paths retain their current fast path.

Raw field/index selection computes offsets from completed logical layout and
checks logical indices independently. The actual selected leaf extent is checked
against original physical allocation bounds. Raw alignment remains anchored at
each actual raw-dereference target; a subsequent dereference replaces that
anchor. Forming a reference additionally checks the selected reference target.
The oversized logical owner is not an artificial full-object bounds requirement.
Address arithmetic observes the independently qualified one-past and liveness
rules, without granting a read or restoring write authority.

First reinterpretation converts an eligible ordinary local root once to the
existing exclusive byte backing; all ordinary and byte aliases then use that
backing. Immutable roots may retain a private encoded backing after readiness,
but each access must independently check current static evaluation context and
read authority before using the cache. Selected typed decoding uses the current
codec, initialization masks, symbolic fragments and pending-duty cursor. It
must not snapshot or scan the whole root on every small read/write.

A typed store validates existing authority and commits one encoded selected span
without reading or dropping overwritten storage. Raw transport preserves masks
and fragment identity; numeric extraction of pointer-bearing bytes remains
forbidden. Complete pointer/reference/function/slice/vtable demands retain
ordinary metadata, liveness and validity duties. No source equality, lifetime,
unsizing or trait proof follows from matching storage coordinates.

## Exhaustive consumers and phase boundary

Audit References/Reader/location, physical word ancestry and reconstruction,
Machine path/raw/reference/read/write/length/range/count/fill consumers, byte
pending state, final reachable-result/escape validity, lower data relocations
and dependency traversal, constant-view graph/writer, and semantic contents.
Byte-origin rows need an explicit reviewed spelling such as
`byte-projection ... base=<location> offset=<u64>`; ordinary rows remain exact.
PA29 Image/relocation export remains explicitly unsupported.

For ordinary source dereferences that resolve to a byte-origin place,
`places::Work::Read` can schedule existing `Work::Execute(key)` using that exact
checked NodeKey/result/context. This reuses the machine codec and readiness
protocol, with no new Target/Requirement or semantic byte decoder. General
byte-reference structural pattern expansion remains explicit required follow-up
if it cannot consume that expression route; it must never manufacture Product
fields or reuse a mismatched query identity.

## Work and qualification

Charge location/projection slots, cached physical ancestry, backing conversion,
selected codec work and actual fragment intersections before growth. Access
permissions and static context are never cached. Qualify width-by-repeat and
depth-by-repeat controls, unrelated-type invariance, selected-footprint decode
counts, repeated small writes, suspended static reads and two reads separated by
a write. Preserve ordinary controls and every initial failed hypothesis.

This family does not close numeric/null reconstruction, whole unsized raw views,
shared-derived interior mutation, new atomic widths, numeric intrinsics or the
remaining enum/opaque/closure/string backing families. Full release, performance,
active manifests and oracle publication remain integrator-owned.
