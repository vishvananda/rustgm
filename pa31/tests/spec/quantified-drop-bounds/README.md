# Quantified Drop bounds

Seven no_core source-core libraries whose Drop impls repeat paren-sugar
`FnOnce` bounds, which this analysis keeps in a binder scope:

- an ordinary Drop impl repeating its struct's `F: FnOnce(T)`;
- core's `DropGuard` shape, a const Drop impl with `F: [const] FnOnce(T)` over
  `ManuallyDrop` fields;
- a repeated higher-ranked `F: FnOnce(&T)`;
- opponents: an added `F: FnOnce(T)`, a stronger `FnOnce(T) -> u8` and an
  added higher-ranked `F: FnOnce(&T)` (each E0367), and a const Drop impl over
  plain fields that are not `[const] Destruct` (E0367).

Outcomes were declared before the frozen rustc 1.98.0 oracle; both are recorded
in `outcomes.json`. Their drafts were explored with rustc first, so this is a
regression record of that exploration. The prefix is the source-const-drop
`cond2.rs` prefix: the pinned Rust core sizing, Destruct and Clone/Copy
declarations and that directory's authored required-member Drop contract,
with core's `Tuple` and `FnOnce` declarations. They keep Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/binder_drop_bounds.rs` checks every case.
See `docs/parallel/census-batch7.md`.
