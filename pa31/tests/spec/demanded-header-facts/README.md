# Demanded header facts

Five no_core source-core libraries whose trait impl self types use array
lengths computed by a source `const` `Sub` impl that follows them (core
`array_impl_default` shape). Outcomes were declared before the frozen
rustc 1.98.0 oracle and before the implementation changed; both are recorded in
`outcomes.json`. The lang-item prefix and the `Add`/`Sub` trait declarations
are exact pinned Rust core fragments with their attributes and comments. They
keep Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

Two later `static` controls read a negated `const` through another `const`
into an array length, with the `Neg`, `Sub` and `Add` impls after the static;
the opponent lacks `Neg` for `i32` (E0600). They were declared and checked
against the same oracle in census batch 2 and accepted after census batch 3.

`dev/tests/invariants/demand_header_facts.rs` checks every case, with the
pa20 `projections/header-demand` controls. See
`docs/parallel/census-batch2.md`.
