# Negative receiver candidates (source core)

Ten no_core source-core libraries reduce pinned core's
`Result<&mut T, E>::cloned` (`result.rs:1795`), checked against rustc 1.98.
Each case's first line names its prefix, which runs through the Fn traits of
`pa31/tests/spec/default-const-closures/count.rs`. Each case then adds a
reduced `Result`-like enum and core's reference `Clone` impls.

`t.clone()` on `t: &mut T` first tries the autoref step `&&mut T`, whose
`Clone` requirement `&mut T: Clone` meets core's negative
`impl<T: PointeeSized> !Clone for &mut T`. That negative matches only with
its own lifetime equations unpaid. No positive candidate exists either way, so
the step does not apply, and autoderef reaches `T: Clone`.

- `core-closure` is core's form, `self.map(|t| t.clone())`, and `annotated`
  writes the closure parameter's type. `direct` makes the call on a `&mut T`
  parameter;
- `reference-closure` and `reference-direct` add core's `Copy` and `Clone`
  impls for `&T`;
- `path` (`T::clone(t)`), `positive-closure` (no negative impl) and `shared`
  (the `&T` form) are controls;
- `missing` drops the `T: Clone` premise (E0599);
- `bound-obligation` requires `&mut u8: Clone` through an ordinary bound
  (E0277). Ordinary obligations keep RustGM's conditional-negative boundary.

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run; both are recorded in `outcomes.json` (10/10 agree). It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/method_selection/refuted_receivers.rs` checks every case.
See `docs/pa31-negative-receivers.md`.
