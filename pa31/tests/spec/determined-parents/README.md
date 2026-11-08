# Parent arguments determined by the parent's own equalities

Six no_core source-core libraries with `min_specialization`, checked against
rustc 1.98. Core's `SpecExtend` parent (`spec/specialization-fixpoint/`)
carries the equality `I: Iter<Item = T>`. Once its `T` is inferred, rustc's
`min_specialization` check reads that equality before it looks for repeated
parameters and `'static` arguments.

`check_always_applicable`
(`rustc_hir_analysis/src/impl_wf_check/min_specialization.rs`) applies
`check_static_lifetimes` and `check_duplicate_params` only to
`unconstrained_parent_impl_args`. That function reads the parent's associated
equalities once, in declaration order. It omits each parameter that occurs in
a constraining position of an equality's result unless that equality's
projection, or an earlier equality's projection or result, names it
(`constrained_generic_params.rs` `parameters_for`).

- rustc's own `check_duplicate_params` example: `impl<S, I: IntoIter<Item =
  S>> Tr<S> for I` beside `impl<T> Tr<T> for V<T>`. The parent arguments `[T,
  V<T>]` repeat `T`, but the equality determines `S`. Its guard is the same
  pair without the equality;
- a `'static` argument for a parent lifetime that the equality determines,
  ordered by a specialization marker on the child, and its guard over a trait
  argument;
- both lifetime pairs without the marker (E0119). `specializes` ignores region
  constraints, so each holds in both directions. The first declaration
  predicted that the determined pair specializes. The oracle disagreed, and
  that is recorded. The marked pairs were declared afterwards, before their
  own oracle (`second_declaration`).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
candidate run of these bytes; both are recorded in `outcomes.json` (5/6
agree). The prefix is `spec/function-pointers/foundation.rs` with
`min_specialization` added. It keeps Rust's MIT/Apache-2.0 attribution; see
root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/inferred_parent_obligations.rs` checks every case. See
`docs/parallel/census-batch11.md`.
