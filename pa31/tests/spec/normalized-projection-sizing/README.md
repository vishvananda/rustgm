# Normalized projection sizing

Five no_core source-core libraries whose sizing goals are rigid projections
in an environment that also assumes sizing of other projections. Core's
`Flatten` Iterator impl, `where I: Iterator<Item: IntoIterator<IntoIter = U,
Item = U::Item>>, U: Iterator`, assumes `<I as Iterator>::Item: MetaSized`
through `IntoIterator`'s implicit Self bound, and its `Option<U::Item>` needs
`<U as Iterator>::Item` to be sized:

- that impl's `fn next(&mut self) -> Option<U::Item>`, the same return type in
  an inherent associated function and in a free function;
- with `?Sized` items, `U::Item` as a `Takes<Rhs: MetaSized>` argument, from its
  implicit `MetaSized` item bound;
- opponent: with `?Sized` items, `Option<U::Item>` is not well-formed (E0277).

Outcomes were declared before the frozen rustc 1.98.0 oracle; both are recorded
in `outcomes.json`. Drafts of the first three shapes were run through rustc and
this compiler first, so those are a regression record of that exploration. The
prefix is the `projection-sizedness` prefix: the exact pinned Rust core
`PointeeSized`, `MetaSized` and `Sized` fragment with its attributes and
comments. It keeps Rust's MIT/Apache-2.0 attribution; see root NOTICE and
`third_party/sources.json`.

`dev/tests/invariants/sizedness_hierarchy.rs` checks every case.
See `docs/parallel/census-batch8.md`.
