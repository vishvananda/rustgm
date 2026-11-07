# Elided lifetimes in opaque result outlives bounds

Six no_core source-core libraries for `'_` in a result `impl Trait + '_`. As in
rustc, that `'_` is the elided output lifetime of the signature: the receiver's,
otherwise the only input lifetime. With no input lifetime or two without a
receiver it is an elision error (E0106), and a where-clause `T: '_` remains
outside every elision site (E0637).

The accepted cases are the core `filter_map` `find`, `flatten` and `step_by`
`nth` shapes: a `&mut T` parameter, an argument-position `&mut impl Tr`, a
`&mut I` with a second non-reference parameter, and a `&self` method with
another reference parameter. Outcomes were declared before the frozen rustc
1.98.0 oracle; both are recorded in `outcomes.json` with the fixture amendments
made before the re-runs.

The lang-item prefix is the exact pinned Rust core fragment from the census
batch 2 controls, with its attributes and comments. It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/elided_opaque_outlives.rs` checks every case. See
`docs/parallel/census-batch4.md`.
