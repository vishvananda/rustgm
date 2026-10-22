# Explicit-tag payload enum byte backing

This development checkpoint extends the existing explicit integer-tag index to
payload variants. Default-representation and niche layouts remain required work;
this is not PA27/PA28 assignment completion.

## Independent contract

The original frozen plan has 20 inputs: 17 independently accepted object/native
programs and three semantic rejections. Their bytes, hypotheses and SHA-256s are
in `obj/parallel/pa27/payload-enum-bytes/declared-first.json`; independent results
are `first/qualification.json`. The wider-tag baseline accepts the ordinary
exact-type copy control and rejects the other 19 at its unsupported enum backing
boundary. Sixteen of those rejections are valid Rust programs, not rejection
contracts.

A separately declared pointer-stability control forms an A(u32) field pointer
at byte offset four, changes the tag to B(bool), initializes its field at offset
one, then writes/reads the old u32 pointer and observes B(true). Rust object and
native outcomes are both zero. `declared-pointer.json` and
`implementation-first/pointer-qualification.json` retain this independent result.
The profile is pinned Rust 1.98.0, edition 2024, x86-64 and panic abort; explicit
`--cap-lints=allow` separates the existing check helper's source lint.

All 21 inputs now pass the declared three-view contract, including the three
rejections for invalid active bool, uninitialized active payload and invalid tag.
Inactive invalid scalar bytes and uninitialized inactive payloads remain legal.
All 18 valid literal native programs match pinned Rust through the locked CPPGM
backend. The permanent draft cases retain the exact qualified bytes, with
companion outcomes in PA27/PA28 `payload-enum-byte-outcomes.json`. They do not
activate an assignment or generate references.

## Typed implementation

The readonly Reader facade borrows the exact variant's retained field TypeIds
and ABI offsets after checking owner, VariantId, ordinal and member/layout
counts. Canonical FieldId selection also checks the stored child type and its
physical extent. It does not infer a variant from a field ordinal or reconstruct
layout. For example, repr(u8)'s individual variant layouts and repr(C,u8)'s common
payload placement retain their distinct offsets.

The cold per-full-TypeId index validates all variant fields once and preserves
the existing sparse/signed physical bits-to-VariantId mapping. The byte-family
walk visits all owned field families; tag cache hits do not certify current
initialization, provenance, payload validity or caller permission. Unsupported
child families still diagnose. Raw copying remains independent of typed enum
decoding and may transport invalid representations for subsequent repair.

Encoding initializes the retained tag and selected active fields through the
existing iterative codec worklist. Decoding validates the current tag bytes,
selects its exact VariantId and iteratively decodes only its active fields into
the existing ordinary Product. Inactive storage and padding do not acquire
initialization or provenance. Reference/function words use the existing symbolic
codec and selected-reference validity duties, including pending static reads.
No Value, View, Origin, Pointer or retained byte Storage field is added.

Every exact checked explicit-enum field projection normalizes to the existing
ByteOffset using canonical declaration and ABI facts. This deliberately replaces
the previous ordinal-only address even before byte conversion, so an old field
pointer retains its physical address after a tag change. Full incoming write
restriction, allocation identity, liveness and access alignment remain separate.
Default-layout enum field projection keeps its previous typed path.

The first actual field access converts a supported root once. Later field reads
and writes reuse that exclusive backing; they neither reconvert nor rescan the
entire owner. Complete typed enum reads still decode the selected payload and
therefore perform work proportional to that selected value. This is distinct
from an individual retained field-pointer access.

## Focused evidence

`implementation-third/qualification.json` retains all initial candidate outcomes.
`closing-second/` records 21 independent source contracts, 18 native comparisons
and 17 selected enum/Reader invariants. Tests cover physical field-pointer
stability, preserved SharedDerived restrictions, expired storage, malformed
owner/ordinal/field counts, fresh tag/bool/initialization checks, inactive storage,
ordinary typed enum values, allocation-budget failure cleanup and warm tag-index
reuse. The earlier compile/import and test-only naming failures are retained;
they are not source rejection observations.

The payload extent x repeated-field grid initially uses 8/32/128 payload bytes,
8/32/128 projections plus u32 writes/reads, and 0/1024 unrelated types. Cold work
is 102/150/344 steps. Warm work is exactly 44 steps per projection/write/read and
independent of payload width and unrelated type count. Each machine retains one
backing; retained slots 47/48/51 stay unchanged across repeated accesses. These
logical observations do not claim wall-time or RSS neutrality. The 4096-byte extent costs 8342 cold steps and 191 retained slots, while the
same 44-step warm cost and single backing remain unchanged. A warm enum-reference
read rechecks and rejects its expired actual referent. The final test replay also
converts payloads from thawed field cells, covering both frozen and mutable Cell
consumers.

The obsolete private test asserting explicit payloads are unsupported is
superseded by opposing default-layout unsupported versus explicit-payload
supported controls. No existing source bytes or expectations are changed.

The final focused tail passes strict all-target/all-feature Clippy, audit (1710
files, zero findings), all 18 enum/facade tests, 20 existing byte-backing
regressions and all 63 earlier fieldless source contracts. Layout, upstream and
suite mapping pass. The final thawed-cell replay passes five selected payload
tests and repeats strict lint/audit for that test-only change; previously passed
source/native results are retained. Guard admission refusals ran no checks and
are not observations.

The one active-text pass covers all 8360 PA1–24 cases and matches qualified
`0b995f43` outputs exactly. The same 51 stale local sidecars remain; compared with
newer public `08ad7822`, only its eight known PA6/PA20 reference changes differ.
There are no unexpected deltas and no references are regenerated. GDB measures
StoredShape at 32 bytes and private byte Storage at 120 bytes, both unchanged.
Exact source/binary/reference hashes, output deltas and layout evidence are in
`text-first/{snapshot.json,qualification.json,layout.log}`.

Focused commands (inside the ordinary resource/storage wrappers):

```sh
cargo test --manifest-path dev/Cargo.toml --locked --test suite_memory payload_enum_bytes:: -- --include-ignored --nocapture
cargo test --manifest-path dev/Cargo.toml --locked --lib enum_ -- --nocapture
cargo test --manifest-path dev/Cargo.toml --locked --lib const_eval::memory::bytes::tests:: -- --nocapture
```

The next bounded enum family is repr(C) without a written integer tag, using its
already-retained concrete ABI tag after independent qualification. Tagless
default layouts and niches remain distinct required work. The integrator owns
combined release/performance qualification and publication.
