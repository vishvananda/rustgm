# Inferred C enum tags in byte storage

This bounded family admits `repr(C)` enums without a written integer tag by
consuming the concrete tag already retained by the target ABI. It does not infer
layout from bytes or extend the public value/provenance schemas. Default Rust,
tagless and niche enum layouts remain required work.

The frozen plan contains 23 independently declared cases under
`obj/parallel/pa27/c-enum-bytes/declared-first.json`. All hypotheses match pinned
Rust 1.98.0/2024/x86-64 object outcomes: 18 positives and five semantic negatives;
all 18 positive Rust executables return zero. The immutable `aa88b3e3` candidate
rejects every case at the old byte-family boundary. Sixteen positive cases belong
to this C family; two additional positives explicitly preserve default-layout
and tagless backing as required gaps. Their rejection is not a semantic contract.
Independent diagnostics/hashes/native results are in `first/qualification.json`.
The existing CHECK helper uses explicit `--cap-lints=allow`, as in the payload
checkpoint; original input bytes are retained.

The first candidate matches all 23 declared three-view outcomes. C positives
include signed/unsigned inferred 32/64-bit tags, common payload placement,
records/arrays, reference/slice fields, distinct closed generic instantiations,
copy/repair/fill, alignment and a pre-existing field pointer across a tag change.
Invalid tag, partial tag, active uninitialized field and invalid bool rejections
now reach actual validity checks. The two retained non-C positives remain explicit
unsupported profiles. Exact results are in `implementation-first/qualification.json`.

The production change is limited to the readonly enum shape gate and checked
field canonicalization. A written integer repr must still equal the retained
physical tag type. Without one, the declaration must be `repr(C)` and the tag
must belong to the retained target C ABI I32/U32/I64/U64 family. Tag offset/extent
and exact owner/VariantId/field facts retain their existing checks. No discriminant
or layout walk is added to each access. The existing cold full-TypeId index,
iterative active-payload codec, initialization masks and symbolic provenance are
reused unchanged. Default Rust types do not become supported merely because an
ABI record happens to have a tag.

Canonical C enum field pointers reuse ByteOffset, retaining physical coordinates
across tag changes and preserving all incoming write restrictions. Raw transport
remains separate from typed tag/payload decoding; current bytes, initialization,
reference validity, alignment, liveness and bounds are never supplied by a warm
tag index. No Value, View, Origin, Pointer, StoredShape or byte Storage field is
added. The permanent draft outcome records distinguish Rust outcomes, frontend
profile outcomes and required gaps explicitly. No activation or references are
changed; combined release/performance qualification is integrator-owned.

Focused qualification passes all 23 source contracts and 16 literal CPPGM/Rust
native comparisons for this C family. The two non-C positives retain their
independent Rust-native success and explicit frontend gaps, without CPPGM-positive
claims. All 21 enum/Reader private tests and the 21 earlier payload source
contracts pass; strict all-target/all-feature Clippy, audit (1712 files, zero
findings), layout, upstream and suite mapping pass. Evidence is `closing-first/`.

The tag-width x variants x repeated-read grid uses tag widths 4/8 bytes,
8/32/128 variants, 8/32/128 reads and 0/1024 unrelated types. Cold work is
76/308/1428 steps for four-byte tags and 84/316/1436 for eight-byte tags. Warm
reads cost width + floor(log2(variants)) + 3 steps each; retained storage
41/65/161 slots is unchanged by repeated reads or unrelated types. These are
logical accounting observations, not wall-time or RSS neutrality claims.

The single active-text pass matches all 8360 qualified PA1–24 outputs from
`0b995f43`. The same 51 stale local sidecars remain; compared with newer public
`08ad7822`, only its eight already-known PA6/PA20 reference changes differ. No
unexpected output changes or reference regeneration occur. GDB measures
StoredShape at 32 bytes and private byte Storage at 120 bytes, unchanged.
`text-first/{snapshot.json,qualification.json,layout.log}` retain exact hashes
and deltas.

Three frozen real-source scaling inputs (8/32/128 C variants) are under
`scaling-inputs/`, with predeclared hashes/outcomes in `declared-scaling.json`
and independent object/Rust-native plus all-three-view candidate outcomes in
`scaling-first/qualification.json`. They are preparation for later integrator
measurement, not timing observations or CPPGM native comparisons.

Focused commands, inside the ordinary resource/storage wrappers:

```sh
cargo test --manifest-path dev/Cargo.toml --locked --test suite_memory c_enum_bytes:: -- --include-ignored --nocapture
cargo test --manifest-path dev/Cargo.toml --locked --lib enum_ -- --nocapture
cargo test --manifest-path dev/Cargo.toml --locked --test suite_memory payload_enum_bytes::payload_enum_byte_source_contracts -- --exact --nocapture
```

The subsequent unique no-tag checkpoint supersedes only the tagless positive
boundary while preserving its source bytes. The C companion record and test
expectation now accept that source; `docs/pa27-tagless-enum-bytes.md` records its
independent contract. Multi-variant default/niche storage remains required.
