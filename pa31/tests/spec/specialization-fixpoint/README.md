# Specialization over where-clause-only parent parameters

Four no_core source-core libraries with `min_specialization`, checked against
rustc 1.98. They reduce census 11's record once its overlap budget was
repaired: "specialization implication still awaits a declaration dependency:
Some(None)" at core's `SpecExtend` pair (`iter/traits/collect.rs:574` and
`589`). The parent's `T` appears only in its where clauses, so its `T: Sized`
obligation cannot be decided until another obligation infers `T`.

rustc's `specializes` (`rustc_trait_selection/src/traits/specialize/mod.rs`)
equates the child's trait reference with the parent's fresh instantiation,
registers every parent obligation under the child's environment and solves
them together. A where-clause-only parameter is inferred from the child's where
clauses. Any error or leftover ambiguity means the child does not specialize.
It is never an error of its own, so the overlapping pair is then E0119.

- core's shape: the child `ExtendT: Extend<T>, I: TrustedLen<Item = T>`, and
  the same child over a concrete `u8`;
- opponents (E0119): a child whose `Extend<u16>` makes the parent's `T` `u16`
  against `Item = u8`, and a child whose two `Extend` and two `Item` bounds
  leave every goal that could infer `T` ambiguous.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run of these bytes; both are recorded in `outcomes.json` (4/4
agree). The prefix is `spec/function-pointers/foundation.rs` with
`min_specialization` added. It keeps Rust's MIT/Apache-2.0 attribution; see
root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/inferred_parent_obligations.rs` checks every case. See
`docs/parallel/census-batch11.md`.
