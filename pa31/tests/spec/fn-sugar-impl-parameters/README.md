# Impl parameters determined by callable sugar

Eleven no_core source-core libraries for impl parameters that only a
parenthesized callable bound determines, as in core's `FromFn`, `RepeatWith`,
`Map`, `FilterMap` and `MapWhile`: `impl<T, F> Iterator for FromFn<F> where
F: FnMut() -> Option<T>`. Sugar quantifies its clause even when it binds no
region. Such an empty binder is an ordinary impl condition, so `T` is
determined with the selected impl and `Self::Item` normalizes.

The controls cover the where-clause and inline forms, a caller premise, a
captured impl lifetime inside the sugar output, the explicit `FnOnce<(),
Output = ...>` form, and three opponents: a mismatched result (E0308), an
output that is not an `Option` (E0271), and a region mismatch. Three controls
stay recorded gaps. `elided-lifetime` binds a region, so its condition still
waits for a resolved parent. `map-adapter` selects now and then stops on a
cause that occurs without sugar as well: override conformance of a repeated
`Self::Item` clause. `const-call-adapter` passes since a user callable-trait
impl keeps its own impl candidates (`spec/user-callable-output/`). Region
errors in generic bodies are outside this analysis.

Outcomes were declared before the frozen rustc 1.98.0 oracle. The first nine
were declared before the implementation changed and the two lifetime controls
before their oracle. Both declarations and one fixture amendment, made before
its re-run, are recorded in `outcomes.json`.

The lang-item prefix and the `Tuple`, `Fn`, `FnMut` and `FnOnce` declarations
are exact pinned Rust core fragments with their attributes and comments, from
`spec/builtin-binop-types/` and the original core `marker.rs` and
`ops/function.rs`. They keep Rust's MIT/Apache-2.0 attribution; see root
NOTICE and `third_party/sources.json`.

`dev/tests/invariants/fn_sugar_impl_parameters.rs` checks every case. See
`docs/parallel/definition-lineage.md`.
