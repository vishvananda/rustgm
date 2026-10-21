# Scalar metadata interface checkpoint

`support::crates` owns stable `CrateIdentity` and `ExternalDef` values and a
bounded canonical binary codec. External identities contain a validated crate
name, explicit stable disambiguator, semantic path and definition kind. They
contain no local definition, type, source-node or symbol-pool indices. Import
remapping into each compilation's arenas is a separate consumer.

Schema 1 begins with `RGMETA\r\n`, little-endian schema ID 1 and three fixed byte
IDs: target 0 means Linux x86-64, language 0 means Rust 1.98.0 with edition 2024,
and profile 0 means closed scalar ordinary nongeneric Rust function exports.
These assignments are permanent. A different target, language or profile needs
a different ID or schema; it cannot reuse zero with changed meaning. Unknown
IDs fail before inference. Scalar tags are explicit values 0 through 16 for
unit, bool, char, u8/u16/u32/u64/u128/usize, i8/i16/i32/i64/i128/isize, f32/f64.
Unit is allowed only as a result. Unsafe permission is explicit; const-body
permission, other calling conventions and variadic signatures are rejected.
This describes profile zero. Profile one additionally transports closed scalar
named values, including unit; see [the value contract](pa29-scalar-values.md).
The original function-only encoding is unchanged.

Dependency identities and function paths have canonical sorted order. Owner
ordinal zero identifies this artifact's crate; no foreign owner may appear in
its export table. Unknown owner ordinals, duplicate identities, foreign exports,
noncanonical order, invalid UTF-8/NFC paths, the standalone `_` path component,
truncation and trailing bytes all fail before resolver allocation. Semantic raw
keyword spelling legality belongs to source resolution, rather than this path
encoding. Rust symbols encode complete crate and path identity into ASCII;
Exact symbols retain their original UTF-8 bytes and never enter that encoder.
The pinned backend's stricter exact-name vocabulary remains a lowering check.

Limits are 16,000,000 artifact bytes, 1024 dependencies, 100,000 combined function/value rows,
1,000,000 path/type atoms and 8,000,000 name bytes. A path has at most 64
components of at most 256 bytes; a signature has at most 256 parameters. Each
untrusted table count is checked against its limit and remaining input before
allocation. Every output append checks the artifact bound. These are logical
payload bounds, not measurements of allocator overhead or total compiler heap.

Validation passed formatting, check, 17 focused codec/root/naming/initializer
tests including existing backend-dependent initializer controls, strict
all-target/all-feature Clippy and the architecture/file audit. After the final
path/tag changes, the four codec tests, strict Clippy and audit passed again.
The tests check reproducible ordering and round trips, every truncated prefix,
unknown header/type/permission categories, foreign IDs, duplicates and oversized
tables. Evidence is in `obj/parallel/pa29-codec/`; the final source manifest
SHA-256 is `25b61c8ad0631316c9bd25c188c80dc91b630196d7a507070bc51ba32a8a8568`.
Final resource receipt `b1a3824f58a948caad68b9ba01472cda` exited zero.

This checkpoint introduces a neutral interface, not separate compilation by
itself. Dependency graph validation, resolver/semantic remapping, public export
collection, crate-aware lowering, library CLI transport and independent
multi-crate native execution remain the next end-to-end slice. Generic bodies,
nominal/lifetime contracts, constants/allocation graphs and closure contracts
remain outside this initial wire profile. Neither assignment is activated.
