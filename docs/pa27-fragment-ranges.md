# Owned byte-array range transport

This checkpoint extends `cef289cd` from complete initialized byte arrays to
owned U8 array-rest/range snapshots. It does not activate PA27 or PA28. General
raw/unsized range values and the remaining backing families are still required.

## Typed selection and transport

The private type-only `Evaluator::byte_array_range_type` validates the original
canonical `[u8; N]` owner and checked `from + length` bound, then interns the
selected `[u8; length]` storage type. It grants no conversion, lifetime, access
or write proof. Range, StoragePath, Constant, Pointer, Value, View and Origin
fields and layouts are unchanged.

Only Value-purpose U8 spans intersecting symbolic fragments take the Image
snapshot path. Source byte bounds remain independent of logical array bounds.
One selected byte/mask span is copied, and indexed intersection clips each word
fragment while retaining its original target, source-word offset and write
restriction. Nested selections keep these coordinates. Numeric ranges retain
ordinary Product/Repeat paths; ReferenceValidation retains scalar decoding.
Neither a cached snapshot nor a complete pointer word makes symbolic bytes
valid integer elements. Final arrays still reject all remaining provenance.

The source contents consumer validates the retained physical owner and exact
selected array type/extent/layout before allocating. It charges range lookup,
selected bytes and fragments through the existing query budget. Ordinary product
and byte-range contents keep their previous path. There is no new query/provider
graph or public value schema.

Range stores accept only the matching canonical array Image, encode its selected
span once, then commit one update to the original backing. This also handles an
initially numeric destination; it does not scalarize pointer bytes or retain a
divergent cell tree. Ordinary non-Image assignments keep their prior paths.
The cheap presence predicate now checks actual Image storage instead of requiring
a union family: raw ptr::read can produce an array Image without any union.

Metadata-only pattern lengths track the retained physical type through cells and
frozen products. On encountering an Image they check its owner and remaining
projection/storage offsets, without structurally reading a union field or its
bytes. Actual reads still enforce initialization, alignment, provenance,
reference validity and publication. The original accepted const-return fixture
first exposed this independent consumer and remains unchanged.

## Independent outcomes

Twenty-five exact source inputs have separately declared outcomes in
`pa27/tests/fragment-range-outcomes.json` and
`pa28/tests/fragment-range-outcomes.json`. Pinned Rust is 1.98.0, edition 2024,
x86-64, with explicit cap-lints=allow for the existing CHECK helper. Inputs,
hashes, original hypotheses, object/native observations and cef baseline outcomes
remain under `obj/parallel/pa27/fragment-ranges/{first,followups,words,runtime}`.

Controls include whole/prefix/suffix/interior/nested ranges, clipping both ends
of two words, partial const-function returns, empty ranges, two stores into
initially numeric storage, and ordinary numeric/reference ranges. Thin pointer,
function, slice data/length and vtable words roundtrip. Shared-derived writes
reject while an original writable pointer survives a separate shared reborrow.
The runtime-range-roundtrip source executes the owned rest copies at runtime;
it is distinct from native programs whose new range operations are const-folded.

The original nested-prefix positive hypothesis was disproved: its final scalar
bindings demand provenance bytes and pinned Rust rejects them. Its exact source
and failed hypothesis remain, now classified as a semantic negative. The original
transient uninitialized-rest acceptance is a separate CTFE oracle difference.
This compiler continues to enforce typed initialization, with no native claim
for that source. Neither observation weakens reference or publication validity.

## Focused checks and work

Four machine invariants cover clipping/restrictions, cache/reference demands,
exact bounds/type/metadata-only owner checks, and one backing update. The source
contents invariant checks owner/type/bounds and budget failure before retention.
The synthetic empty-source fixture supplies an explicit positive query budget;
its earlier default-budget failure and later exhaustion check are retained.
No production work limit changed.

At fixed selected width eight bytes, charged work per warm range snapshot is
207/271/335 for 8/32/128 source fragments. Repetition counts 8/32/128 scale
exactly with those per-operation counts; root bytes are not copied or scanned
on every selected read. Repeated roots reuse the canonical selected type without
growing Store. These are logical bounds, not wall-time/RSS neutrality claims.

Strict Clippy, file audit, source/native and active-text receipts are listed in
the handoff. Full release, performance measurement, assignment manifests and
reference publication remain integrator-owned. No backend or reference change
is part of this checkpoint.

The closing source/native contract passes all 25 cases in all three views
(17 accept, eight reject), with 17 exact literal native comparisons. Four machine
range invariants, the source contents invariant and all 20 byte-storage
invariants pass, as does the previous 29-case array-fragment source contract.
Strict all-target/all-feature Clippy, audit (1701 files, zero findings), layout,
upstream and suite registration pass. The final 8,360 active PA1–24 outputs
exactly match immutable published `0b995f43`; the same 51 locally stale
sidecars remain, with zero unexpected differences and no regeneration.

Receipts are under `obj/parallel/pa27/fragment-ranges/`, in
`implementation-fourth`, `closing-tail-second` and `text-first`. The final production fingerprint is
`801cd42a05347600a3874661fd185d7851a0b613497a27d0b8fa9781fd56a077`;
the immutable candidate SHA-256 is
`611062e02b62c4cab259ef77061ce8dd68aa55bf5955261a43c2eb455a84e1bd`.
The failed default-budget private fixture, strict lint findings and duplicate
suite-registration attempt remain in the earlier receipts; none is a language
rejection or a passing check. No duplicate registration remains.
