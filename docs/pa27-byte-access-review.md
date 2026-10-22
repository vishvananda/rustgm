# Typed byte-addressed constant access

This isolated checkpoint starts at accepted `213f1e21`. PA27/28 remain planned.
The approved interface is recorded in `pa27-byte-access-design.md`; this review
records the implemented address/access profile, not assignment completion.

## Representation and authority

The internal Byte tag shares the existing projection arena's `{base, offset}`
record. The readonly `Origin::ByteOffset {base,offset}` names retained provenance;
there is no numeric/root-plus-offset pointer constructor. Relative selections
normalize to original allocation coordinates. Pointer metadata is cleared only
for sized selections, while original write restrictions survive every path.
Ordinary Constant/Location/Origin sizes remain 32/64/24 bytes; StoragePath remains
56 bytes, with a 24-byte optional Range-or-Bytes terminal. Existing ordinary
Place/Pointer/Projection layouts and the unallocated range path remain unchanged.

Logical field/index selection consumes retained owner, ordinal, type and layout.
Logical indices and selected physical leaf bounds are separate: a fitting prefix
of a larger logical owner is supported. A one-past address may exist, but a read
there fails bounds. Each raw dereference supplies its alignment anchor; a packed
base by-value field access differs from dereferencing a separately formed field
pointer. Reference formation independently checks the actual leaf alignment.
Wrong-allocation anchors reject even for unaligned operations or references.

Local roots convert once to exclusive byte backing. Immutable completed roots
can retain an encoded backing, but every demand still checks static eligibility
and current context. Typed reads use the existing initialized mask, scalar codec,
symbolic words and pending-duty protocol; no parallel stale Cell tree remains.
Stores update a selected span without reading/dropping overwritten storage.
Reference validation preserves an already checked normalized pointer identity;
the strict pending-duty equality check is unchanged. Shared reconstruction
restricts only that pointer, and mutable reconstruction requires existing write
permission. A separate shared alias never upgrades or poisons another pointer.

The source place consumer schedules the existing checked execution request at the
exact original NodeKey for byte views. Nested byte fields normalize through the
same retained layout. There is no second decoder or new query/provider identity.
Lowering follows original backing plus byte offset. The constant view adds only
`origin=byte-projection:aBASE:OFFSET contents=alias` allocation rows; ordinary rows
and pointer `write=` vocabulary remain unchanged. PA29 Image/relocation export
remains an explicit boundary.

## Independent inputs and retained hypotheses

`obj/parallel/pa27/raw-byte-access/declared-{first,second,third,fourth,fifth}.json`
retains 65 original source hypotheses and SHA-256s. Independent qualification is
Rust 1.98.0, edition 2024, Linux x86-64 little-endian, explicitly using
`--cap-lints=allow`; sources and declarations precede candidate observations.
Object, valid native and frozen-213 three-view receipts are retained in the
corresponding named directories. Frozen production hash is
`dc2bbb6fa22b0ad716832d4196784a45e30e39de11bbca52d7109f00f49c6a4f`, binary
`acba916b15d454c69e9ae07416c438ecd0522830539c73ec0d577ddc7890ecac`.
The tracked companion contracts preserve exact bytes in PA27 or PA28 according
to the latest feature used. No active manifest or text oracle is generated.

The original `u32::from` const hypothesis rejects because stable From is not
const. Its exact bytes and diagnostics remain; a separately declared `as u32`
control isolates the memory behavior. Numeric/expired raw-array length-only
observations have object/CTFE evidence but no native-validity claim. The original
unit-equality positive remains a PA26 hook dependency; `let _:()=N` separately
qualifies zero-sized memory access. The `.add(2)` and whole union/byte-array/pointer
roundtrip positives remain required follow-ups, unchanged. Typed field offsets
or raw-copy counterfactuals do not replace those programs.

Thirteen unchanged earlier fixtures are requalified in `supersessions-first/`.
Scalar read/store, valid bool, inner reference-region, shared/mutable reference,
union casts and owned reference-transfer positives supersede their earlier
unsupported expectations. Invalid bool now rejects for representation validity.
An unused shared-derived mutable-reference form and the scoped unsized-object
escape remain unsupported profiles; their new diagnostics are not evidence of
semantic rejection parity. Two additional frozen native value-use inputs check
the different-referent and inner-region results without rewriting those original
sources. The complete physical storage equality and ordinary source lifetime
proof rules are unchanged.

## Focused qualification

The closing receipts live under `obj/parallel/pa27/raw-byte-access/`. Source
catalogs compare all three public views with declared Rust outcomes; native
controls preserve literal bytes and compare pinned Rust with the locked CPPGM
backend. Backend hashes remain `2acc7cefeecc3752c4a15da4ddb1e65c9cb2b88e0032a23d1eca7ff36a005882`
(lowir) and `57f2c1156168eac7e653a4a193f27f124b635413e07de29940daf34a0b831fec`
(lowir2native). Native cases exclude all numeric/expired-address or invalid-access
hypotheses.

Private invariants cover layout density, monotone authority, current-byte aliasing,
one-past bounds, expiry, packed/reference alignment, wrong anchors and exact
normalized reference identity. Warm selected read/write pairs take 26 charged
steps per repetition at widths 8/32/128 with stable retained storage. Source
work controls vary width 8/32/128, depth 0/8 and repetition 8/32/128, plus unrelated
type noise. Their logical counts are not wall-time or RSS neutrality evidence.

Closing includes strict all-target/all-feature Clippy, syntax-aware audit,
layout/upstream/suite-map checks and directly affected source regressions.
Historical failed lint/test batches remain recorded; successful groups are reused
unless subsequent changes affect them. One active PA1–24 text pass is retained
for integrator review, without blessing sidecars. Combined release, performance,
reference publication and assignment activation remain integrator-owned.

## Completed handoff receipts

`closing-fifth/` records 67 independently checked source objects and 201 public
view outcomes: 44 supported positives, 20 Rust rejections and three retained
Rust-positive profile/dependency gaps. Forty-one literal native comparisons,
five private invariants and the 20-input source work grid pass. The original
13-case first replay deliberately retains the different-referent duplicate-ID
failure; `text-first/supersessions-final.json` records its corrected outcome and
all final three-view results. Ten earlier positive boundary expectations now
become successful. The shared-derived unused mutable-reference positive and the
scoped unsized-object escape retain explicit unsupported diagnostics; invalid
bool now rejects for actual validity. No original fixture bytes are rewritten.

`closing-sixth/` finishes strict all-target/all-feature Clippy, audit 1684/0,
layout, upstream, suite-map and the unattempted affected reference/store/union
and ordinary-location regressions. The only production difference after the
preceding successful native/producer run restores the existing no-provenance
diagnostic for unsupported numeric references; successful-path semantics are
unchanged. `text-first/dst-regression.log` records all three directly affected
DST source/native/work functions, including the original published-field cases.

The single active text pass covers all 8,360 PA1–24 cases. Every output equals
immutable published `0b995f43213413a74b42ee211725d0862f0674cb`; exact manifest and
input identities were checked before comparing its archived references. Local
references differ on the same 51 already published cases, with no unexpected
change. `text-first/qualification.json` retains exact sidecar hashes and paths.
No reference was written. Current production fingerprint is
`a6da280775c05b685287e963d24c530ba0ef43b7dda0c9b86fa2d40ca85e2a15`, immutable candidate binary is `61ed5199235fe857baaee12ef8585390eb211c69e60f42556a54dda4446a7dde`.

The next required slices remain the original `.add(2)` route and whole byte-array
fragment snapshot roundtrip. Neither is replaced by the now-supported logical
field offsets, counted transfers or offset-word-only counterfactuals. Full
release/performance and final handout reconciliation remain centralized.

`work-final/observations.json` retains all 20 actual source-work rows with
uncaptured output. At depth zero the repetition increments are 1,872 then 7,488
steps for each width; depth eight gives 3,792 then 15,168. Unrelated noise gives
exactly the same 5,551 steps/417 slots. That limited receipt recovery verifies
production and binary hashes unchanged from the completed text pass; it does not
repeat a broad functional or performance matrix.
