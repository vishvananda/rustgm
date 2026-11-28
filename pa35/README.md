# RustGM Programming Assignment 35 — Upstream crate acceptance

Build and execute consumers over real, unmodified upstream crates using
RustGM's source-built Rust 1.98.0 standard library and the pinned CPPGM
backend.

## Required upstream crate graph

This assignment grades the compiler's own locked upstream dependency graph
for the Rust 1.98.0 Linux x86-64 panic-abort profile:

- `unicode-ident` `1.0.24` (edition 2021)
- `tinyvec_macros` `0.1.1` (edition 2018)
- `tinyvec` `1.10.0` (edition 2018, features `alloc`, `default`, `tinyvec_macros`)
- `unicode-normalization` `0.1.25` (edition 2018, features `default`, `std`)

Preserve exact upstream source checksums, per-crate editions, feature
selections, and target configuration without patching crate sources. Broader
upstream crate graphs (`itoa`, `serde_core`, `serde_json` with `memchr`/`zmij`)
are deferred to the follow-on acceptance track (PA45).

## Tool surface

Run `make setup-rust-src` once from the repository root to verify and unpack
the pinned Rust 1.98.0 library sources and the four locked crates (or set
`RUSTGM_RUST_SRC` and `RUSTGM_CRATE_CACHE` for offline replay).

Build every target crate in strict mode with zero bring-up records, link
through CPPGM, and verify byte-identical LowIR reproducibility across clean
builds.

## Grading suite

Run the grading suite from the repository root:

```sh
make -C pa35 test
```

The suite (`pa35/tests/suite.json` and `pa35/tests/unicode-consumer-outcomes.json`)
compiles, links, and executes all four rustc-verified consumers three times
each in a fresh empty directory:

- `xid`: full Unicode scalar sweep of `unicode_ident::is_xid_start` / `is_xid_continue`
- `normalization`: NFC, NFD, NFKC, NFKD, quick checks, stream-safe, and CJK variants
- `identifiers`: XID identifier lexing and NFC interning as used in RustGM's lexer
- `small-vectors`: `tinyvec` `ArrayVec`, `TinyVec`, and `SliceVec` inline-to-heap transitions
