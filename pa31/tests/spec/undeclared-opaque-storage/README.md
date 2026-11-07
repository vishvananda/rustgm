# Undeclared opaque applications during storage preparation

Four no_core source-core libraries for opaque storage preparation. An opaque
result's application is interned before its declaration is published, and that
declaration may wait for its clauses (here a GAT const argument
`{ size::<u8>() }`) or fail after interning. A generic const fn evaluation in
between prepares storage for every interned type. Such an application has no
hidden type yet: it waits for its own identity, exactly like a declared opaque
whose defining body is unchecked, and never fails the caller.

The controls cover a declared opaque before a `size::<Self>()` length (the core
`to_be_bytes` shape), a waiting GAT-argument opaque, its mismatched-equality
opponent (E0271) and an opaque that fails after interning (E0106), which in the
census previously blamed the next length with "missing storage opaque
declaration". Outcomes were declared before the frozen rustc 1.98.0 oracle;
both are recorded in `outcomes.json` with the fixture amendments made before the
re-runs.

The lang-item prefix is the exact pinned Rust core fragment from the census
batch 2 controls, with its attributes and comments. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/undeclared_opaque_storage.rs` checks every case, and
`dev/tests/census_mode.rs` keeps the failed opaque item-local in a census. See
`docs/parallel/census-batch4.md`.
