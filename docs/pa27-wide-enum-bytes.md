# Wider explicit fieldless enum tags

This extends the qualified fieldless enum byte codec to explicit integer tags:
`u16/u32/u64/u128/usize` and `i16/i32/i64/i128/isize`, alongside the earlier
8-bit tags. It uses the retained x86-64 primitive width: usize/isize occupy eight
bytes. Payload and default-representation enum backing remain required followups.
No Value/View/Origin/Pointer, backend vocabulary or source proof authority changes.

## Representation and validity

The existing readonly enum facade now validates the complete tag byte extent.
Semantic discriminants are masked to the retained integer width; the 128-bit
case uses the full mask without shifting by 128. The sorted index retains all
128 bits and the exact canonical VariantId, so distinct variants with identical
low 64-bit words cannot collide. Signed negative tags use physical two's-
complement bits while ordinary Product values preserve their semantic variant.

Typed encoding initializes only tag bytes. Padding stays independently
uninitialized. Typed decoding checks the entire tag's initialization and symbolic
provenance before selecting its exact variant. Raw copies still preserve invalid
or uninitialized bytes without decoding them, and a destination can be repaired
before a typed read. Whole-root bounds, alignment, liveness, write authority and
static-context checks remain in the existing access machinery.

The private index entry grows from eight to 32 bytes to retain u128 keys. Both
scratch preflight and retained accounting use that measured entry size; ordinary
roots still allocate no enum cache. StoredShape and private Storage fields do not
change. The existing lazy handle and immutable per-type cache remain unchanged.

## Frozen outcomes and focused checks

The 37 frozen cases were declared before execution under
`obj/parallel/pa27/wide-enum-bytes/declared-first.json`; all independent hypotheses
match Rust 1.98.0, edition 2024, x86-64, panic abort. Explicit cap-lints preserves
the frozen CHECK helper separately from source lint policy. All 24 valid programs
execute successfully with pinned Rust. The prior 8-bit candidate rejects all 37
at its explicit byte-family boundary; these negatives are not earlier proofs of
invalid-tag validity. Original hashes, stderr and results remain in `first/`.

The candidate passes all 37 cases in all three views and all 24 literal native
comparisons. Coverage includes high-bit unsigned values, signed minima and
maxima, sparse/invalid tags, the full u128 key, little-endian arrays, partial
initialization, raw copy followed by repair, unrepaired demand and alignment
padding. Permanent unchanged inputs and companion outcome records live in
PA27 and PA28 `wide-enum-bytes` clusters.

Strict all-target/all-feature Clippy, architecture/file audit (1708 files, zero
findings), layout, upstream and suite mapping pass. Thirteen selected enum tests,
the earlier 26-case 8-bit source group and 20 backing invariants pass. Three new
private controls cover width × variants × repetition, padding/partial writes and
pointer-word provenance surviving partial numeric overwrite. Evidence is
`closing-first/` under the wide-enum evidence root.

The work grid uses 2/8/16-byte tags, 8/32/128 variants, 8/32/128 reads and
0/1024 unrelated types. Warm reads cost tag width + floor(log2 variants) + 3
charged steps each; repeated reads retain no extra backing/index capacity.
Unrelated type growth changes neither cold/warm work nor retained slots. These
are logical bounds, not wall-time or RSS neutrality claims. Exact observations,
final source/binary hashes, active-text comparison and layouts are in the handoff.
The integrator owns combined release qualification; no active manifest, text
reference or assignment status is changed by this checkpoint.


The single active-text pass covers all 8,360 PA1–24 cases and equals prior
qualified published `0b995f43` byte-for-byte. Local stale sidecars remain the
same 51. Compared with public `08ad7822`, exactly its eight reviewed PA6/PA20
sidecars differ on this older isolated branch, with no unexpected delta. GDB
confirms StoredShape32/Storage120, unchanged from the 8-bit checkpoint.
Afterward, one private test alone was strengthened and rerun: numeric zero is a
valid enum tag, so pointer-placeholder zero cannot mask a missing provenance
check. Production code and its frozen candidate remain unchanged. Exact
receipts are `text-first/`, `provenance-tail.log` and `handoff.json`.

At variants8/32/128, cold work is54/238/1166 for2-byte tags,
66/250/1178 for8-byte tags and82/266/1194 for16-byte tags. Retained logical
slots are41/65/161 across these widths; warm work follows the stated formula.
The corresponding earlier 8-bit index retained35/41/65 slots: the additional
retention is the explicitly charged full-width key, rather than repeated reads.
The old private wider-tag unsupported expectation is superseded; payload/default
unsupported checks remain, and a mismatched declared/retained tag still rejects.
