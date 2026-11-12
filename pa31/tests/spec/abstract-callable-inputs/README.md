# Abstract callable inputs (source core)

Eleven no_core source-core libraries reduce pinned core's `Iterator::max`
(`iter/traits/iterator.rs:3255`) and `max_by_key`, checked against rustc 1.98.
They use the `default-const-closures` prefix, with a reduced `Ordering`, `Ord`
and `Iterator` added per case.

A function item whose generic inputs are inferred only through a callable bound
must meet an abstract projection input such as `Self::Item`. Its reference
inputs take fresh signature lifetimes, while the identical projections beneath
them still match.

- `max` (core's form) and `max-call` pass `Ord::cmp` to `max_by`, and
  `max-by-key` passes core's local `compare` over `(B, T)` through `Map` and an
  opaque `key` closure;
- `projection-item`, `generic-item` and `unbounded-item` pass `Ord::cmp` and
  generic local fn items for an `I::Item` input;
- `parameter-item` (a type parameter input) and `qualified-item`
  (`<I::Item as Ord>::cmp`) are controls;
- `max-missing` drops `Self::Item: Ord` (E0277), and `projection-mismatch` and
  `projection-distinct` pass a concrete item or one item for two projections
  (E0631).

Outcomes were declared before the frozen rustc 1.98.0 oracle and before any
RustGM run; both are recorded in `outcomes.json` (11/11 agree). It keeps Rust's
MIT/Apache-2.0 attribution; see root NOTICE and `third_party/sources.json`.
`dev/tests/abstract_callable_inputs.rs` checks every case.
See `docs/pa24-abstract-callable-inputs.md`.
