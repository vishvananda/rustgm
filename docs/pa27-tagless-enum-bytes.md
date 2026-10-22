# Unique no-tag enum byte storage

This checkpoint admits a Rust enum only when its exact closed retained ABI
layout has no tag and its declaration and physical layout have one canonical
variant. It reuses `StoredShape::Product` with that `VariantId`, existing physical
field offsets and the bounded iterative byte codec. It adds no Value, View,
Origin, Pointer or stored-shape variant. Default multi-variant and niche layouts
remain required work; a sole variant does not authorize guessing another
layout's tag or validity rules.

The frozen independent plan is
`obj/parallel/pa27/tagless-enum-bytes/declared-first.json`. Pinned Rust
1.98.0/2024/x86-64 matches all 19 declared object outcomes: 15 positives and four
semantic negatives. All 15 positive Rust executables return zero. Fourteen
positives belong to the supported family; the other preserves the unchanged
multi-variant default-layout source as a required positive gap. The immutable
`3608eadd` baseline rejects all 19 cases at its earlier byte-family boundary.
Exact source hashes, diagnostics and native results are retained under `first/`.
The existing CHECK helper uses explicit `--cap-lints=allow`; fixture bytes are
unchanged between independent and candidate qualification.

The first candidate matches all 19 declared three-view outcomes. Supported
cases cover typed and byte-initialized roundtrips, records, nested enums,
references and slices, distinct closed generic fields, non-Copy fills, field
pointers formed before/after conversion, zero-sized variants/fields and transport
of uninitialized padding. Partial initialization and padding reads reject at
actual initialized-byte checks; wrong reference storage rejects at bounds and
local reference escape rejects through ordinary lifetime proof. These outcomes
are recorded in `implementation-first/qualification.json`. The original
`pa28/tests/spec/default-enum-bytes/tagless-still-required.rs` is preserved
byte-for-byte and its former profile rejection is explicitly superseded in the
C outcomes/tests. The original multi-variant source stays an accepted-Rust gap.

The readonly Reader shares exact nominal owner, variant count, physical field
count and field-layout validation with tagged enums. Field selection preserves
full closed TypeId, canonical FieldId/ordinal, physical bounds and checked child
type. Tagless sole-variant fields keep ordinary typed projections because their
layout cannot move when the variant changes; tagged enum fields retain their
previous physical ByteOffset rule. Conversion creates one exclusive backing,
so earlier field pointers and later typed projections observe the same writes.

Encoding requires the exact sole variant and field count for both immutable
products and thawed cells, including zero-sized values. Decoding constructs that
canonical variant and visits its actual physical children. Existing byte masks,
symbolic fragments, reference duties, alignment, bounds, storage liveness and
incoming write restrictions remain independent. Warm family/layout facts never
supply value validity. A shared-derived alias remains restricted without
poisoning an independently writable pointer. No enum tag index is allocated for
this no-tag family, and ordinary roots avoid byte materialization until needed.

The first closing build retained two test-only wiring failures (missing `Read`
import and an assertion requiring Debug on a private byte ID); no semantic
outcome is inferred from that compile failure. The preceding fmt/audit passed
with zero findings. Focused closing evidence is retained under `closing-first/`
and its corrected successor. Release/performance qualification is owned by the
integrator; no assignment activation or generated references are changed.

Required PA27/28 work remains: multi-variant default/niche enum byte layouts,
other unqualified backing families and provenance/reconstruction profiles,
shared-derived interior mutation, complete lint/validity coverage, aggregate/ZST/
wide volatile transfers, further atomic widths/accessors/fences/update methods
and remaining numeric/intrinsic families. Historical narrower boundaries in the
prior checkpoint records are superseded only by their later qualified families,
not by this no-tag slice.

The first corrected private run exposed an invalid test assumption: ordinary
projection formation retains fresh view IDs rather than interning them. Four
identity/validity controls passed; the two tests comparing view IDs failed.
The corrected assertions compare retained original-root/field coordinates and
verify cross-view reads/writes, preserving the original failure evidence.
Repeated-work retention claims concern the machine byte backing and cells;
ordinary fresh reference-view records are not claimed to be constant storage.

A later private test reached the internal typed `write_place` primitive without
its required `check_raw_write` caller gate. The corrected test exercises the
production check-then-store protocol and checks that a rejected shared-derived
write leaves the writable alias unchanged. Production validation is not moved
or weakened. The repeat grid includes that caller-side check in each iteration.

Final focused qualification passes all 19 three-view source contracts and 14
literal CPPGM/Rust native comparisons; the remaining multi-variant positive
retains only its independent Rust-native success. All 27 enum/Reader private
regressions and the 23 C source contracts pass. Strict all-target/all-feature
Clippy, audit (1714 files, zero findings), placement, upstream and suite mapping
pass. Evidence is under `closing-fifth/`. Natural enum-field-selection and
aggregate-projection helper extraction resolved the two strict function-size
failures; no lint allowances or permission changes were added.

The complete check/store/read grid uses payload widths 8/32/128/4096,
8/32/128 iterations and 0/1024 unrelated types. Cold conversion costs
57/105/299/8297 charged steps. Every warm iteration costs 46 steps, independent
of payload extent and unrelated types. One backing remains throughout, with
machine backing/cell retention 26/27/30/170 logical slots respectively and no
enum-tag index. Those figures exclude the ordinary fresh reference-view arena;
these are logical work/storage observations, not wall-time or RSS claims.

The single active-text pass matches all 8360 PA1–24 outputs from the qualified
`0b995f43` baseline. The 51 stale local sidecars remain unchanged; compared with
`08ad7822`, only its eight already-reviewed PA6/PA20 sidecars differ. This is an
isolated older-base comparison, not a claim about later public reference edits.
There are no unexpected deltas and no reference regeneration. GDB measures
StoredShape at 32 bytes and private byte Storage at 120 bytes, unchanged.
Exact candidate/source hashes, reference archive hash and deltas are retained in
`text-first/{snapshot.json,qualification.json,layout.log}`.

The next bounded recommendation is default multi-variant enums with an actual
retained integer tag, preserving canonical variant/payload identity and physical
field pointers. That needs independent legal-source controls and an explicit
Reader capability review; it must not infer niche decoding or portable raw layout
from the existing default tag. Niche/optimized representations remain a separate
required representation family.

Focused commands, inside the ordinary resource/storage wrappers:

```sh
cargo test --manifest-path dev/Cargo.toml --locked --lib tagless_enum_ -- --nocapture
cargo test --manifest-path dev/Cargo.toml --locked --test suite_memory tagless_enum_bytes:: -- --include-ignored --nocapture
cargo test --manifest-path dev/Cargo.toml --locked --lib enum_ -- --nocapture
cargo test --manifest-path dev/Cargo.toml --locked --test suite_memory c_enum_bytes::c_enum_byte_source_contracts -- --exact --nocapture
```
