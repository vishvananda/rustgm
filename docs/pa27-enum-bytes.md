# Fieldless enum byte backing

This isolated development slice targets explicit `repr(u8)` and `repr(i8)`
fieldless enums. Payload, default-representation and wider-tag enum backing
remain required work. Runtime enum layout and ordinary enum constants are
already supported; this slice concerns existing constant byte storage.

## Independent contract

The first frozen plan contains 17 cases: eleven independently accepted object
and native controls and six semantic rejections. Eight accepted cases reach the
current unsupported byte-family boundary. Exact-type copying, ordinary enum
values and a nonnull aligned zero-size transfer already pass all three views.
The positive cases include sparse tags, negative tags, a single explicitly
represented variant and a const-function-computed discriminant. Demanded and
unused invalid tags, uninitialized tags and sparse holes reject independently.

Four additional frozen controls distinguish transport from typed demand. Rust
accepts copying initialized invalid or uninitialized tag bytes and repairing the
destination before a typed read; both programs execute successfully. Demanding
the unrepaired copies rejects. The byte transport path must preserve those
representations without invoking enum decoding.

Independent inputs, hashes, expected outcomes, diagnostics and native results
live under `obj/parallel/pa27/enum-bytes/`. They were declared before execution
in `declared-first.json` and `declared-transport.json`; qualification receipts
are `first/qualification.json` and `transport-first/qualification.json`.
Pinned profile: Rust 1.98.0, edition 2024, x86-64, panic abort; explicit
`--cap-lints=allow` keeps the existing check helper's source lint independent.

Five final controls independently cover nested array storage, two distinct enum
identities, a wrong-tag rejection after reading a different enum, both signed
boundaries and retained alignment padding. Their receipts are
`family-first/qualification.json`, bound to `declared-family.json`. Across all
26 cases, 17 Rust/native positives and nine rejections match the declarations;
14 accepted cases were baseline frontend gaps. Frozen bytes are copied unchanged
to the PA27/PA28 `enum-bytes` draft clusters and companion outcome records.

## Implemented typed boundary

`StoredShape::Enumeration { tag: abi::layout::Tag }` describes a completed
explicit U8/I8 tag layout. A readonly Reader facade borrows retained ABI
variants and exact member VariantIds, validating canonical owner and counts.
It does not reconstruct layout or infer an ordinal from a byte. The ordinary
StoredShape baseline is 32 bytes; private byte Storage is 112 bytes, measured
with GDB sizeof against the immutable previous candidate.

A lazy machine-owned index, keyed by the full closed TypeId, checks the entire
family once. Every ABI variant must have zero fields and pair with its canonical
member owner/ordinal. A sorted physical tag-to-VariantId index preserves sparse
and signed discriminants; negative discriminants are encoded at the actual
retained tag width, never treated as bare variant ordinals. Invalid and duplicate
physical tags diagnose rather than selecting an arbitrary variant.

Encoding requires the exact ordinary zero-field Product variant and initializes
only its retained tag bytes; padding remains uninitialized. Typed decoding uses
the scalar codec's initialization and provenance checks, then selects the exact
VariantId and constructs the existing ordinary Product representation. Neither
Value, View, Origin nor Pointer changes. Typed raw access continues to check
lifetime, alignment, bounds and the incoming write restriction independently.

The index preflights checked retained and scratch capacities and charges family
validation/sort plus each lookup. Ordinary roots allocate no enum cache. Warm
reads perform no full-family scan. Enum storage is terminal for declared physical
subobject selection; no fake tag field is introduced. Payload/default families
remain explicit unsupported capabilities rather than validity rejections.

## Focused qualification

All 26 declared cases pass all three frontend views; all 17 valid literal native
programs match pinned Rust. The nine negative cases now reject for invalid enum
discriminants or initialization, rather than the earlier unsupported-family
boundary. Raw invalid/uninitialized copy followed by repair remains accepted.

Ten selected private tests pass, including retained tag readiness. The new
invariants cover malformed count/owner/ordinal/tag/bounds, duplicate physical
tags, cold ordinary values, thawed cells, full closed const-argument keys, budget
failure cleanup and fresh validity checks on warm indexes. The 20 existing
byte-backing invariants also pass. Strict all-target/all-feature Clippy, file
and architecture audit (1706 files, zero findings), layout, upstream and suite
mapping pass. Evidence is `implementation-third/` below the enum evidence root.
Earlier borrow-check/lint failures and the pre-launch scheduling cancellation
are retained; they are not source rejection evidence.

The variants × repeated-read grid uses 8/32/128 variants, 8/32/128 reads, and
0/1024 unrelated types. Cold conversion work is 52/236/1164 steps; warm reads
cost 7/9/11 charged steps each. Retained storage is 35/41/65 logical slots and
unchanged across repeated reads. Unrelated type growth changes none of these
observations. Warm family-index hits require one step and no new storage
headroom. These logical invariants do not claim wall-time or RSS neutrality.

The single active-text pass covers all 8,360 PA1–24 cases and matches the
previously qualified published `0b995f43` outputs exactly. The 51 stale local
sidecars are unchanged. Compared with newer public `7117fe9f`, exactly its eight
already reviewed PA6/PA20 sidecars differ; this older isolated branch lacks those
unrelated changes. There are no unexpected output changes and no references are
regenerated. Exact deltas and hashes are retained in `text-first/qualification.json`.
GDB measures candidate StoredShape at 32 bytes and private byte Storage at 120
bytes: the former is unchanged, the latter gains one eight-byte lazy handle.
Ordinary execution allocates no enum index. The cfg(test) Storage is 144 bytes,
including its three existing test counters. The layout log is retained beside
the pinned immutable candidate. No active manifest, generated reference or assignment status
changes are made. The integrator owns combined release qualification and
publication.
