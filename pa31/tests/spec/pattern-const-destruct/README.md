# Pattern types in const Drop fields

Six no_core source-core libraries whose const Drop impls own pattern-type
fields:

- `pattern_type!(*const T is !null)`, core's `NonNull` shape over it and core's
  array `Drain` field shapes (NonNull, a raw pointer, a mutable reference and
  PhantomData);
- the raw pointer base without, and a type parameter base with,
  `T: [const] Destruct`;
- an opponent: a type parameter base without that bound (E0367).

A pattern type is `[const] Destruct` exactly when its base type is. Outcomes
were declared before the frozen rustc 1.98.0 oracle; both are recorded in
`outcomes.json`. Their drafts were explored with rustc first, so this is a
regression record of that exploration. The prefix is the source-const-drop
`cond2.rs` prefix: the pinned Rust core sizing, Destruct and Clone/Copy
declarations and that directory's authored required-member Drop contract,
with an authored builtin `pattern_type` macro declaration. They keep Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/destruct_patterns.rs` checks every case.
See `docs/parallel/census-batch7.md`.
